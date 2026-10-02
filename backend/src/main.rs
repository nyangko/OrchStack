// OrchStack 백엔드 진입점: DB 준비 → 라우터 구성 → HTTP 서버 실행

mod agent; // /profiles CRUD · /templates 조회
mod approval; // /approvals 조회 · 승인 · 거부
mod connection; // /connections CRUD · 한도 조회
mod decision; // /decisions 조회 · 답변 · 작성 중
#[allow(unused_imports, dead_code)] // sea-orm-cli 생성 코드
mod entity; // 테이블별 SeaORM entity (sea-orm-cli 생성물 · 직접 수정하지 않는다)
mod error; // 공통 에러 응답
mod event; // 명령 실행 틀 (상태 변경 + 이벤트 append + 발행)
mod exec; // 실행기: Claude Code · Codex CLI 비대화형 실행 (#13)
mod issue; // /issues CRUD
mod notify; // /notifications · /notify/* · /audit + 이벤트 → 알림 projection
mod project; // /projects CRUD
mod run; // /runs · Run 명령 · 실행기용 전이 함수
mod rule; // 하위 작업 규칙 엔진 (#67 · LLM 0)
mod runner; // runner 하위 Run 실행 · @REPORT 회수 (#67)
mod setting; // /workspace · /runtimes · /presets
mod skill; // /skills · /mcps · /skill-sources 조회 · 스킬 허용/차단
mod stream; // /projects/{sn}/snapshot · events · stream (SSE)
mod task; // /tasks CRUD + MoveTask + 배정
mod team; // /teams · /members CRUD

use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr, Statement, TransactionTrait};
use serde::Serialize;
#[cfg(test)]
use serde_json::{Value, json};

// 스키마 원본. 빌드할 때 파일 내용을 바이너리에 포함한다
const SCHEMA: &str = include_str!("../../data/sqlite.sql");

// Alpha는 로컬 1인 사용: 사용자 1 · 워크스페이스 1 고정
const UID: i64 = 1;
const WID: i64 = 1;

/// 연결 후 테이블이 없으면 data/sqlite.sql로 만들고 기본 사용자 · 워크스페이스를 넣는다
async fn connect(opt: ConnectOptions) -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect(opt).await?;
    let probe = Statement::from_string(DbBackend::Sqlite, "SELECT 1 FROM sqlite_master WHERE name = 'tbl_user'");
    if db.query_one_raw(probe).await?.is_none() {
        // 한 트랜잭션: 중간에 실패해도 반쯤 만들어진 DB가 남지 않는다
        let tx = db.begin().await?;
        tx.execute_unprepared(SCHEMA).await?;
        tx.execute_unprepared(&format!(
            "INSERT INTO tbl_user (uid, name) VALUES ({UID}, 'Me'); \
             INSERT INTO tbl_workspace (sn, uid, name) VALUES ({WID}, {UID}, 'OrchStack');"
        ))
        .await?;
        tx.commit().await?;
    }
    Ok(db)
}

/// API 문서 머리말
#[derive(OpenApi)]
#[openapi(info(title = "OrchStack API", version = "0.1.0"))]
struct Doc;

/// 라우터 + OpenAPI 문서. DB 없이도 만들 수 있다 (`openapi` 명령이 문서만 뽑을 때 쓴다)
fn api() -> (Router<DatabaseConnection>, utoipa::openapi::OpenApi) {
    OpenApiRouter::with_openapi(Doc::openapi()).routes(routes!(health)).merge(project::routes()).merge(issue::routes()).merge(task::routes()).merge(run::routes()).merge(agent::routes()).merge(team::routes()).merge(connection::routes()).merge(setting::routes()).merge(stream::routes()).merge(decision::routes()).merge(approval::routes()).merge(skill::routes()).merge(notify::routes()).split_for_parts()
}

/// 전체 라우터 (`/openapi.json` 포함). 테스트에서도 같은 라우터를 쓰려고 main에서 분리했다
fn app(db: DatabaseConnection) -> Router {
    let (router, doc) = api();
    router.route("/openapi.json", get(move || async move { Json(doc) })).with_state(db)
}

/// health 응답
#[derive(Serialize, utoipa::ToSchema)]
struct Health {
    /// ok | error
    status: &'static str,
    /// ok | down
    db: &'static str,
}

/// 서버와 DB가 살아 있는지 확인한다 (DB 연결이 끊기면 503)
#[utoipa::path(operation_id = "main_health", get, path = "/health", responses((status = 200, body = Health), (status = 503, body = Health)))]
async fn health(State(db): State<DatabaseConnection>) -> (StatusCode, Json<Health>) {
    match db.ping().await {
        Ok(()) => (StatusCode::OK, Json(Health { status: "ok", db: "ok" })),
        Err(_) => (StatusCode::SERVICE_UNAVAILABLE, Json(Health { status: "error", db: "down" })),
    }
}

/// 환경변수로 설정을 읽고 서버를 시작한다
#[tokio::main]
async fn main() {
    // `orchstack-backend openapi` : 서버를 띄우지 않고 OpenAPI JSON만 출력한다 (프론트 타입 생성용)
    if std::env::args().nth(1).as_deref() == Some("openapi") {
        println!("{}", api().1.to_pretty_json().unwrap());
        return;
    }
    let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://orchstack.db?mode=rwc".into());
    let addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".into());
    let db = connect(ConnectOptions::new(url)).await.expect("db connect failed");
    let listener = tokio::net::TcpListener::bind(&addr).await.expect("bind failed");
    println!("listening on http://{addr}");
    axum::serve(listener, app(db)).await.unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    /// 요청 1건을 보내고 (상태 코드, JSON 본문)을 돌려주는 테스트 도우미
    async fn call(app: &Router, method: &str, uri: &str, body: Option<Value>) -> (StatusCode, Value) {
        let req = Request::builder().method(method).uri(uri).header("content-type", "application/json")
            .body(body.map_or(Body::empty(), |b| Body::from(b.to_string()))).unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        let status = res.status();
        let bytes = res.into_body().collect().await.unwrap().to_bytes();
        (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
    }

    /// 메모리 DB. 메모리 DB는 연결마다 따로 생기므로 연결 1개로 고정
    async fn mem() -> DatabaseConnection {
        let mut opt = ConnectOptions::new("sqlite::memory:");
        opt.max_connections(1);
        connect(opt).await.unwrap()
    }

    async fn setup() -> Router {
        app(mem().await)
    }

    #[tokio::test]
    async fn health() {
        assert_eq!(call(&setup().await, "GET", "/health", None).await.0, StatusCode::OK);
    }

    #[tokio::test]
    async fn project() {
        let app = setup().await;

        // 생성: 시드 워크스페이스(wid=1)에 붙고, DB 기본값(default_branch = 'main')이 채워진다
        let (st, p) = call(&app, "POST", "/projects", Some(json!({"name": "OrchStack"}))).await;
        assert_eq!((st, p["default_branch"].as_str(), p["wid"].as_i64()), (StatusCode::CREATED, Some("main"), Some(1)));
        let sn = p["sn"].as_i64().unwrap();

        // 목록 · 수정 · 조회 · 삭제 후 404
        assert_eq!(call(&app, "GET", "/projects", None).await.1.as_array().unwrap().len(), 1);
        let (_, p) = call(&app, "PATCH", &format!("/projects/{sn}"), Some(json!({"name": "Renamed", "status": "archived"}))).await;
        assert_eq!((p["name"].as_str(), p["status"].as_str()), (Some("Renamed"), Some("archived")));
        assert_eq!(call(&app, "GET", &format!("/projects/{sn}"), None).await.1["name"], "Renamed");
        assert_eq!(call(&app, "DELETE", &format!("/projects/{sn}"), None).await.0, StatusCode::NO_CONTENT);
        assert_eq!(call(&app, "GET", &format!("/projects/{sn}"), None).await.0, StatusCode::NOT_FOUND);
    }

    /// 이슈 · 태스크 CRUD: 번호(num)는 프로젝트 안에서 이슈와 태스크가 함께 발급받고, Kanban 목록은 상태로 거른다
    #[tokio::test]
    async fn issue_task() {
        let app = setup().await;
        let (_, p) = call(&app, "POST", "/projects", Some(json!({"name": "P"}))).await;
        let ps = p["sn"].as_i64().unwrap();

        let (st, i) = call(&app, "POST", &format!("/projects/{ps}/issues"), Some(json!({"title": "I"}))).await;
        assert_eq!((st, i["num"].as_i64(), i["status"].as_str()), (StatusCode::CREATED, Some(1), Some("open")));
        let is = i["sn"].as_i64().unwrap();
        let (st, t) = call(&app, "POST", &format!("/issues/{is}/tasks"), Some(json!({"title": "T"}))).await;
        assert_eq!((st, t["num"].as_i64(), t["status"].as_str(), t["priority"].as_i64()), (StatusCode::CREATED, Some(2), Some("todo"), Some(2)));
        let ts = t["sn"].as_i64().unwrap();

        // 이슈: 수정 · 닫으면 close_at · 모르는 상태는 422
        let (_, i) = call(&app, "PATCH", &format!("/issues/{is}"), Some(json!({"status": "closed"}))).await;
        assert!(i["close_at"].is_string());
        assert_eq!(call(&app, "PATCH", &format!("/issues/{is}"), Some(json!({"status": "x"}))).await.0, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(call(&app, "GET", &format!("/projects/{ps}/issues"), None).await.1.as_array().unwrap().len(), 1);

        // 태스크: 수정 · 우선순위 범위 · 이슈/프로젝트 없음 404 · 상태 필터
        let (_, t) = call(&app, "PATCH", &format!("/tasks/{ts}"), Some(json!({"title": "T2", "priority": 0}))).await;
        assert_eq!((t["title"].as_str(), t["priority"].as_i64()), (Some("T2"), Some(0)));
        assert_eq!(call(&app, "PATCH", &format!("/tasks/{ts}"), Some(json!({"priority": 9}))).await.0, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(call(&app, "POST", "/issues/99/tasks", Some(json!({"title": "x"}))).await.0, StatusCode::NOT_FOUND);
        assert_eq!(call(&app, "POST", "/projects/99/issues", Some(json!({"title": "x"}))).await.0, StatusCode::NOT_FOUND);
        assert_eq!(call(&app, "GET", &format!("/issues/{is}/tasks"), None).await.1.as_array().unwrap().len(), 1);
        assert_eq!(call(&app, "GET", &format!("/projects/{ps}/tasks?status=todo"), None).await.1.as_array().unwrap().len(), 1);
        assert_eq!(call(&app, "GET", &format!("/projects/{ps}/tasks?status=done"), None).await.1.as_array().unwrap().len(), 0);

        // 삭제
        assert_eq!(call(&app, "DELETE", &format!("/tasks/{ts}"), None).await.0, StatusCode::NO_CONTENT);
        assert_eq!(call(&app, "GET", &format!("/tasks/{ts}"), None).await.0, StatusCode::NOT_FOUND);
        assert_eq!(call(&app, "DELETE", &format!("/issues/{is}"), None).await.0, StatusCode::NO_CONTENT);
    }

    /// MoveTask 1회 = 상태 변경 + 이벤트 1행(seq 증가)이 한 트랜잭션. 막힌 전이는 409이고 이벤트도 남지 않는다
    #[tokio::test]
    async fn move_task() {
        use crate::entity::tbl_log_event as ev;
        use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};
        let db = mem().await;
        let app = app(db.clone());
        let mut rx = event::subscribe();
        let (_, p) = call(&app, "POST", "/projects", Some(json!({"name": "P"}))).await;
        let (_, i) = call(&app, "POST", &format!("/projects/{}/issues", p["sn"]), Some(json!({"title": "I"}))).await;
        let (_, t) = call(&app, "POST", &format!("/issues/{}/tasks", i["sn"]), Some(json!({"title": "T"}))).await;
        let (ts, uri) = (t["sn"].as_i64().unwrap(), format!("/tasks/{}/move", t["sn"]));
        let events = || ev::Entity::find().filter(ev::Column::AggregateType.eq("task")).filter(ev::Column::AggregateSn.eq(ts)).order_by_asc(ev::Column::Seq);

        // todo → in_progress: 상태 바뀌고 TaskCreated(seq 1) 뒤에 TaskMoved(seq 2)
        let (st, m) = call(&app, "POST", &uri, Some(json!({"status": "in_progress"}))).await;
        assert_eq!((st, m["status"].as_str(), m["start_at"].is_string()), (StatusCode::OK, Some("in_progress"), true));
        let rows = events().all(&db).await.unwrap();
        assert_eq!(rows.iter().map(|r| (r.seq, r.event_type.as_str())).collect::<Vec<_>>(), [(1, "TaskCreated"), (2, "TaskMoved")]);
        assert_eq!(rows[1].payload_json, r#"{"from":"todo","to":"in_progress"}"#);

        // done까지 가면 done_at. 끝 상태에서 나가는 전이 · 같은 상태 · 모르는 상태는 409, 이벤트는 그대로
        let (_, m) = call(&app, "POST", &uri, Some(json!({"status": "done"}))).await;
        assert!(m["done_at"].is_string());
        for to in ["in_progress", "done", "nope"] {
            let (st, v) = call(&app, "POST", &uri, Some(json!({"status": to}))).await;
            assert_eq!((st, v["error"].as_str()), (StatusCode::CONFLICT, Some("conflict")), "{to}");
        }
        assert_eq!(events().count(&db).await.unwrap(), 3);
        assert_eq!(call(&app, "GET", &format!("/tasks/{ts}"), None).await.1["status"], "done");
        assert_eq!(call(&app, "POST", "/tasks/99/move", Some(json!({"status": "todo"}))).await.0, StatusCode::NOT_FOUND);

        // 커밋된 이벤트만 broadcast로 나간다 (409로 막힌 것은 없다)
        let mut moved = 0;
        while let Ok(m) = rx.try_recv() {
            moved += (m.event_type == "TaskMoved" && !m.payload_json.contains(r#""to":"blocked""#)) as i32; // blocked = stream_live 것
        }
        assert_eq!(moved, 2);
    }

    /// 프로젝트 · 이슈 · 태스크를 만들고 (태스크 sn, 담당 멤버 배정 여부)를 돌려준다. 배정 API(B-4)가 아직 없어 멤버는 SQL로 넣는다
    async fn task_of(app: &Router, db: &DatabaseConnection, assign: bool) -> i64 {
        let (_, p) = call(app, "POST", "/projects", Some(json!({"name": "P"}))).await;
        let (_, i) = call(app, "POST", &format!("/projects/{}/issues", p["sn"]), Some(json!({"title": "I"}))).await;
        let (_, t) = call(app, "POST", &format!("/issues/{}/tasks", i["sn"]), Some(json!({"title": "T"}))).await;
        if assign {
            db.execute_unprepared(
                "INSERT INTO tbl_agent_profile (wid, kind) VALUES (1, 'workspace'); INSERT INTO tbl_team (wid, name) VALUES (1, 'T'); \
                 INSERT INTO tbl_member (team_sn, profile_sn, name, role_name) VALUES (1, 1, 'm', 'Dev'); UPDATE tbl_task SET member_sn = 1;",
            ).await.unwrap();
        }
        t["sn"].as_i64().unwrap()
    }

    /// 대상(agg, sn)의 이벤트 (이름, seq) 목록
    async fn events(db: &DatabaseConnection, agg: &str, sn: i64) -> Vec<(String, i64)> {
        use crate::entity::tbl_log_event as ev;
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
        ev::Entity::find().filter(ev::Column::AggregateType.eq(agg)).filter(ev::Column::AggregateSn.eq(sn)).order_by_asc(ev::Column::Seq)
            .all(db).await.unwrap().into_iter().map(|r| (r.event_type, r.seq)).collect()
    }

    /// StartRun → Review → Approve: 이벤트 3행(seq 1·2·3)이 쌓이고 태스크는 최종 done
    #[tokio::test]
    async fn run_flow() {
        let db = mem().await;
        let app = app(db.clone());
        let ts = task_of(&app, &db, true).await;

        let (st, r) = call(&app, "POST", &format!("/tasks/{ts}/runs"), None).await;
        assert_eq!((st, r["status"].as_str(), r["num"].as_i64()), (StatusCode::CREATED, Some("queued"), Some(1)));
        let rs = r["sn"].as_i64().unwrap();
        assert_eq!(call(&app, "GET", &format!("/tasks/{ts}"), None).await.1["status"], "in_progress");

        // 실행기(#13) 대신 running으로 만든다
        db.execute_unprepared("UPDATE tbl_run SET status = 'running'").await.unwrap();
        assert_eq!(call(&app, "POST", &format!("/runs/{rs}/review"), None).await.1["status"], "review");
        assert_eq!(call(&app, "GET", &format!("/tasks/{ts}"), None).await.1["status"], "review");
        let (st, r) = call(&app, "POST", &format!("/runs/{rs}/approve"), None).await;
        assert_eq!((st, r["status"].as_str(), r["end_at"].is_string()), (StatusCode::OK, Some("completed"), true));
        assert_eq!(call(&app, "GET", &format!("/tasks/{ts}"), None).await.1["status"], "done");

        let names: Vec<_> = events(&db, "run", rs).await.into_iter().map(|(n, q)| format!("{q}:{n}")).collect();
        assert_eq!(names, ["1:RunStarted", "2:ReviewRequested", "3:RunApproved"]);
    }

    /// stop · retry(새 Run · 기존 불변) · reject(tbl_review + 태스크 복귀) · 실행기 전이 함수 · 막힌 전이 409
    #[tokio::test]
    async fn run_more() {
        use crate::entity::tbl_review as rv;
        use sea_orm::EntityTrait;
        let db = mem().await;
        let app = app(db.clone());
        let status = |sn: i64| { let app = app.clone(); async move { call(&app, "GET", &format!("/runs/{sn}"), None).await.1["status"].as_str().unwrap().to_string() } };

        // 담당 멤버 없음 → 409
        let ts = task_of(&app, &db, false).await;
        assert_eq!(call(&app, "POST", &format!("/tasks/{ts}/runs"), None).await.0, StatusCode::CONFLICT);
        db.execute_unprepared(
            "INSERT INTO tbl_agent_profile (wid, kind) VALUES (1, 'workspace'); INSERT INTO tbl_team (wid, name) VALUES (1, 'T'); \
             INSERT INTO tbl_member (team_sn, profile_sn, name, role_name) VALUES (1, 1, 'm', 'Dev'); UPDATE tbl_task SET member_sn = 1;",
        ).await.unwrap();

        // 진행 중 Run이 있으면 두 번째 시작은 409. stop → cancelled, 태스크는 그대로, 다시 stop은 409
        let runs = format!("/tasks/{ts}/runs");
        call(&app, "POST", &runs, None).await;
        assert_eq!(call(&app, "POST", &runs, None).await.0, StatusCode::CONFLICT);
        let (st, r) = call(&app, "POST", "/runs/1/stop", None).await;
        assert_eq!((st, r["status"].as_str()), (StatusCode::OK, Some("cancelled")));
        assert_eq!(call(&app, "GET", &format!("/tasks/{ts}"), None).await.1["status"], "in_progress");
        assert_eq!(call(&app, "POST", "/runs/1/stop", None).await.0, StatusCode::CONFLICT);

        // retry: 새 Run(2행) · 첫 Run 불변 · 진행 중 Run은 retry 불가
        let (st, r) = call(&app, "POST", "/runs/1/retry", None).await;
        assert_eq!((st, r["num"].as_i64(), r["retry_run_sn"].as_i64(), r["status"].as_str()), (StatusCode::CREATED, Some(2), Some(1), Some("queued")));
        assert_eq!(status(1).await, "cancelled");
        assert_eq!(call(&app, "POST", "/runs/2/retry", None).await.0, StatusCode::CONFLICT);
        assert_eq!(call(&app, "GET", &runs, None).await.1.as_array().unwrap().len(), 2);

        // 실행기용 전이 함수: 순서대로만, review · cancelled 목표는 거부
        run::run_to(&db, 2, "starting").await.unwrap();
        run::run_to(&db, 2, "running").await.unwrap();
        assert!(run::run_to(&db, 2, "review").await.is_err() && run::run_to(&db, 2, "starting").await.is_err());
        assert_eq!(call(&app, "POST", "/runs/2/approve", None).await.0, StatusCode::CONFLICT); // running은 승인 불가

        // reject: 없는 멤버는 422이고 아무것도 안 바뀐다. 정상 반려는 Run failed(rejected) · 태스크 in_progress · 리뷰 1행(round 1)
        call(&app, "POST", "/runs/2/review", None).await;
        let bad = call(&app, "POST", "/runs/2/reject", Some(json!({"member_sn": 99}))).await;
        assert_eq!((bad.0, status(2).await.as_str()), (StatusCode::UNPROCESSABLE_ENTITY, "review"));
        let (st, r) = call(&app, "POST", "/runs/2/reject", Some(json!({"member_sn": 1, "reason": "라벨 누락"}))).await;
        assert_eq!((st, r["status"].as_str(), r["fail_code"].as_str(), r["fail_detail"].as_str()), (StatusCode::OK, Some("failed"), Some("rejected"), Some("라벨 누락")));
        assert_eq!(call(&app, "GET", &format!("/tasks/{ts}"), None).await.1["status"], "in_progress");
        let reviews = rv::Entity::find().all(&db).await.unwrap();
        assert_eq!(reviews.iter().map(|r| (r.round, r.result.as_str())).collect::<Vec<_>>(), [(1, "rejected")]);
        assert_eq!(events(&db, "run", 2).await.iter().map(|e| e.0.as_str()).collect::<Vec<_>>(),
            ["RunStarted", "RunProgressed", "RunProgressed", "ReviewRequested", "RunRejected"]);

        // 반려 뒤 retry → Run 3, 다시 반려하면 round 2
        call(&app, "POST", "/runs/2/retry", None).await;
        db.execute_unprepared("UPDATE tbl_run SET status = 'running' WHERE sn = 3").await.unwrap();
        call(&app, "POST", "/runs/3/review", None).await;
        call(&app, "POST", "/runs/3/reject", Some(json!({"member_sn": 1}))).await;
        assert_eq!(rv::Entity::find().all(&db).await.unwrap().last().unwrap().round, 2);

        // 실패 전이 + Session 전이
        call(&app, "POST", "/runs/2/retry", None).await; // 이미 failed인 2 → Run 4
        run::run_to(&db, 4, "starting").await.unwrap();
        run::run_to(&db, 4, "failed").await.unwrap();
        assert_eq!(events(&db, "run", 4).await.last().unwrap().0, "RunFailed");
        db.execute_unprepared("INSERT INTO tbl_session (run_sn, member_sn, num) VALUES (4, 1, 1)").await.unwrap();
        run::session_to(&db, 1, "active").await.unwrap();
        run::session_to(&db, 1, "stopped").await.unwrap();
        assert!(run::session_to(&db, 1, "active").await.is_err());
        let (_, ss) = call(&app, "GET", "/runs/4/sessions", None).await;
        assert_eq!((ss[0]["status"].as_str(), ss[0]["end_at"].is_string()), (Some("stopped"), true));
        assert_eq!(events(&db, "session", 1).await.len(), 2);
    }

    /// 프로필 CRUD · 하위 매핑 조회 · 템플릿 조회 → 멤버 생성 시 live 버전 프로필 복사 · 팀/멤버 CRUD · 삭제 제한
    #[tokio::test]
    async fn agent_team() {
        let db = mem().await;
        let app = app(db.clone());

        // 프로필: 생성(DB 기본값) · 수정 · Trust 범위 · 종류 검사 · 목록 필터 · 삭제
        let (st, p) = call(&app, "POST", "/profiles", Some(json!({}))).await;
        assert_eq!((st, p["kind"].as_str(), p["trust_level"].as_i64()), (StatusCode::CREATED, Some("workspace"), Some(3)));
        let ps = p["sn"].as_i64().unwrap();
        let (_, p) = call(&app, "PATCH", &format!("/profiles/{ps}"), Some(json!({"effort": "high", "trust_level": 2}))).await;
        assert_eq!((p["effort"].as_str(), p["trust_level"].as_i64()), (Some("high"), Some(2)));
        assert_eq!(call(&app, "PATCH", &format!("/profiles/{ps}"), Some(json!({"trust_level": 5}))).await.0, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(call(&app, "POST", "/profiles", Some(json!({"kind": "x"}))).await.0, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(call(&app, "GET", "/profiles?kind=member", None).await.1.as_array().unwrap().len(), 0);
        // 폴백 체인: 같은 연결을 등급별 모델로 두 번 · 배열 순서 = sort · 모르는 tier · 없는 연결은 422
        db.execute_unprepared(
            "INSERT INTO tbl_runtime (sn, wid, code, name) VALUES (1, 1, 'claude_code', 'Claude Code'); \
             INSERT INTO tbl_connection (sn, wid, kind, provider_code, provider_name, name) VALUES (1, 1, 'subscription', 'anthropic', 'Anthropic', 'max');",
        ).await.unwrap();
        let fbs = format!("/profiles/{ps}/fallbacks");
        let chain = json!([{"runtime_sn": 1, "connection_sn": 1, "tier": "S"}, {"runtime_sn": 1, "connection_sn": 1, "tier": null}]);
        assert_eq!(call(&app, "PUT", &fbs, Some(chain)).await.0, StatusCode::OK);
        let v = call(&app, "GET", &fbs, None).await.1;
        assert_eq!((v[0]["tier"].as_str(), v[1]["tier"].is_null()), (Some("S"), true));
        assert_eq!(call(&app, "PUT", &fbs, Some(json!([{"runtime_sn": 1, "connection_sn": 1, "tier": "X"}]))).await.0, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(call(&app, "PUT", &fbs, Some(json!([{"runtime_sn": 1, "connection_sn": 9}]))).await.0, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(call(&app, "GET", &fbs, None).await.1.as_array().unwrap().len(), 2); // 실패한 교체는 기존 체인을 지우지 않는다
        // 명령 사용 여부: 기본 3개 차단 → checkout 켜기 · rm -rf 차단 추가 → 기본 목록 순서 뒤에 추가 명령
        let cmds = format!("/profiles/{ps}/commands");
        let on = |v: &Value| v.as_array().unwrap().iter().map(|c| (c["cmd"].as_str().unwrap().to_owned(), c["on"].as_bool().unwrap())).collect::<Vec<_>>();
        assert_eq!(on(&call(&app, "GET", &cmds, None).await.1), [("git stash".into(), false), ("git checkout".into(), false), ("git reset".into(), false)]);
        let v = call(&app, "PUT", &cmds, Some(json!([{"cmd": "git checkout", "on": true}, {"cmd": "rm -rf", "on": false}]))).await.1;
        assert_eq!(on(&v), [("git stash".into(), false), ("git checkout".into(), true), ("git reset".into(), false), ("rm -rf".into(), false)]);
        assert_eq!(v[3]["builtin"], false);
        assert_eq!(call(&app, "PUT", &cmds, Some(json!([{"cmd": " ", "on": true}]))).await.0, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(call(&app, "DELETE", &format!("/profiles/{ps}"), None).await.0, StatusCode::NO_CONTENT);

        // 템플릿(live v2 · 도구 정책 1개)과 draft 템플릿은 SQL로 넣는다 (템플릿 편집은 이 Task 범위 밖)
        db.execute_unprepared(
            "INSERT INTO tbl_agent_profile (sn, wid, kind, effort) VALUES (10, 1, 'template', 'high'); \
             INSERT INTO tbl_profile_tool (profile_sn, tool_code, policy) VALUES (10, 'shell', 'approval'); \
             INSERT INTO tbl_template (sn, wid, name, role_name, icon, color) VALUES (1, 1, 'Frontend', 'Frontend Developer', 'monitor', 'role-frontend'); \
             INSERT INTO tbl_template_revision (template_sn, profile_sn, version, status) VALUES (1, 10, 2, 'live'); \
             INSERT INTO tbl_template (sn, wid, name, status) VALUES (2, 1, 'Draft', 'draft');",
        ).await.unwrap();
        assert_eq!(call(&app, "GET", "/templates", None).await.1.as_array().unwrap().len(), 2);
        assert_eq!(call(&app, "GET", "/templates/1", None).await.1["role_name"], "Frontend Developer");

        // 팀
        let (st, t) = call(&app, "POST", "/teams", Some(json!({"name": "Core"}))).await;
        assert_eq!((st, t["kind"].as_str()), (StatusCode::CREATED, Some("project")));
        let ts = t["sn"].as_i64().unwrap();
        assert_eq!(call(&app, "PATCH", &format!("/teams/{ts}"), Some(json!({"max_concurrent_run": 5}))).await.1["max_concurrent_run"], 5);
        assert_eq!(call(&app, "POST", "/teams", Some(json!({"name": "x", "kind": "y"}))).await.0, StatusCode::UNPROCESSABLE_ENTITY);
        // 하위 작업 정책: 기본값 runner, 허용 밖 기본 방식 · 모르는 방식 · 0 상한은 422
        let tp = format!("/teams/{ts}");
        assert_eq!(t["spawn_mode"], "runner");
        let v = call(&app, "PATCH", &tp, Some(json!({"spawn_allow": "sub,fork,runner", "spawn_mode": "fork", "max_child_run": 2}))).await.1;
        assert_eq!((v["spawn_mode"].as_str(), v["max_child_run"].as_i64()), (Some("fork"), Some(2)));
        for bad in [json!({"spawn_allow": "sub,runner"}), json!({"spawn_mode": "x"}), json!({"spawn_allow": "fork,y"}), json!({"max_child_run": 0})] {
            assert_eq!(call(&app, "PATCH", &tp, Some(bad.clone())).await.0, StatusCode::UNPROCESSABLE_ENTITY, "{bad}");
        }
        assert_eq!(call(&app, "GET", "/teams", None).await.1.as_array().unwrap().len(), 1);

        // 템플릿 멤버: 새 프로필(kind member)에 설정 · 도구 정책이 복사되고 표시값은 템플릿에서
        let members = format!("/teams/{ts}/members");
        let (st, m) = call(&app, "POST", &members, Some(json!({"name": "진", "template_sn": 1}))).await;
        assert_eq!((st, m["role_name"].as_str(), m["icon"].as_str(), m["template_version"].as_i64()), (StatusCode::CREATED, Some("Frontend Developer"), Some("monitor"), Some(2)));
        let (ms, mp) = (m["sn"].as_i64().unwrap(), m["profile_sn"].as_i64().unwrap());
        assert_ne!(mp, 10);
        let p = call(&app, "GET", &format!("/profiles/{mp}"), None).await.1;
        assert_eq!((p["kind"].as_str(), p["effort"].as_str()), (Some("member"), Some("high")));
        let caps = call(&app, "GET", &format!("/profiles/{mp}/caps"), None).await.1;
        assert_eq!((caps["tools"][0]["tool_code"].as_str(), caps["tools"][0]["policy"].as_str()), (Some("shell"), Some("approval")));

        // draft 템플릿 409 · 없는 템플릿 422 · 템플릿도 역할도 없으면 422 · 빈 캐릭터는 역할만으로
        assert_eq!(call(&app, "POST", &members, Some(json!({"name": "a", "template_sn": 2}))).await.0, StatusCode::CONFLICT);
        assert_eq!(call(&app, "POST", &members, Some(json!({"name": "a", "template_sn": 9}))).await.0, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(call(&app, "POST", &members, Some(json!({"name": "a"}))).await.0, StatusCode::UNPROCESSABLE_ENTITY);
        let (st, m2) = call(&app, "POST", &members, Some(json!({"name": "하린", "role_name": "QA"}))).await;
        assert_eq!((st, m2["template_sn"].as_i64()), (StatusCode::CREATED, None));
        assert_eq!(call(&app, "GET", &members, None).await.1.as_array().unwrap().len(), 2);

        // 멤버 수정 · 상태 검사 · 쓰는 중인 프로필 삭제 409 · 멤버 삭제 시 프로필도 삭제
        assert_eq!(call(&app, "PATCH", &format!("/members/{ms}"), Some(json!({"status": "paused"}))).await.1["status"], "paused");
        assert_eq!(call(&app, "PATCH", &format!("/members/{ms}"), Some(json!({"status": "x"}))).await.0, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(call(&app, "DELETE", &format!("/profiles/{mp}"), None).await.0, StatusCode::CONFLICT);
        let m2s = m2["sn"].as_i64().unwrap();
        assert_eq!(call(&app, "DELETE", &format!("/members/{m2s}"), None).await.0, StatusCode::NO_CONTENT);
        assert_eq!(call(&app, "GET", &format!("/profiles/{}", m2["profile_sn"]), None).await.0, StatusCode::NOT_FOUND);

        // Run 기록이 있는 멤버는 삭제 · 팀 삭제 모두 409. 기록이 없으면 팀 삭제 시 멤버 · 프로필도 지워진다
        let t = task_of(&app, &db, false).await;
        call(&app, "POST", &format!("/tasks/{t}/assign"), Some(json!({"member_sn": ms}))).await;
        call(&app, "POST", &format!("/tasks/{t}/runs"), None).await;
        assert_eq!(call(&app, "DELETE", &format!("/members/{ms}"), None).await.0, StatusCode::CONFLICT);
        assert_eq!(call(&app, "DELETE", &format!("/teams/{ts}"), None).await.0, StatusCode::CONFLICT);
        db.execute_unprepared("DELETE FROM tbl_run").await.unwrap();
        assert_eq!(call(&app, "DELETE", &format!("/teams/{ts}"), None).await.0, StatusCode::NO_CONTENT);
        assert_eq!(call(&app, "GET", &format!("/members/{ms}"), None).await.0, StatusCode::NOT_FOUND);
        assert_eq!(call(&app, "GET", &format!("/profiles/{mp}"), None).await.0, StatusCode::NOT_FOUND);
        assert_eq!(call(&app, "GET", "/profiles/10", None).await.0, StatusCode::OK); // 템플릿 프로필은 그대로
    }

    /// assign: 태스크 member_sn · assign_by 반영 + 이벤트 1행. 해제는 NULL. 없는 멤버 422 · 보관 멤버 409 · 없는 태스크 404
    #[tokio::test]
    async fn assign() {
        let db = mem().await;
        let app = app(db.clone());
        let ts = task_of(&app, &db, false).await;
        let (_, t) = call(&app, "POST", "/teams", Some(json!({"name": "T"}))).await;
        let (_, m) = call(&app, "POST", &format!("/teams/{}/members", t["sn"]), Some(json!({"name": "진", "role_name": "Dev"}))).await;
        let (ms, uri) = (m["sn"].as_i64().unwrap(), format!("/tasks/{ts}/assign"));

        let (st, v) = call(&app, "POST", &uri, Some(json!({"member_sn": ms}))).await;
        assert_eq!((st, v["member_sn"].as_i64(), v["assign_by"].as_str()), (StatusCode::OK, Some(ms), Some("user")));
        assert_eq!(call(&app, "GET", &format!("/tasks/{ts}"), None).await.1["member_sn"].as_i64(), Some(ms));
        assert_eq!(events(&db, "task", ts).await.iter().map(|e| e.0.as_str()).collect::<Vec<_>>(), ["TaskCreated", "AgentAssigned"]);

        let (st, v) = call(&app, "DELETE", &uri, None).await;
        assert_eq!((st, v["member_sn"].as_i64(), v["assign_by"].as_str()), (StatusCode::OK, None, None));
        assert_eq!(events(&db, "task", ts).await.last().unwrap().0, "AgentUnassigned");

        assert_eq!(call(&app, "POST", &uri, Some(json!({"member_sn": 99}))).await.0, StatusCode::UNPROCESSABLE_ENTITY);
        call(&app, "PATCH", &format!("/members/{ms}"), Some(json!({"status": "archived"}))).await;
        assert_eq!(call(&app, "POST", &uri, Some(json!({"member_sn": ms}))).await.0, StatusCode::CONFLICT);
        assert_eq!(call(&app, "POST", "/tasks/99/assign", Some(json!({"member_sn": ms}))).await.0, StatusCode::NOT_FOUND);
        assert_eq!(events(&db, "task", ts).await.len(), 3); // 실패한 요청은 이벤트를 남기지 않는다
    }

    /// 문서에 모든 경로와 공통 에러 스키마가 있다
    /// 연결 CRUD (key_ref 응답 · 이벤트 미노출) · 한도 · 실행기 · 워크스페이스 · 프리셋 버전
    #[tokio::test]
    async fn setting() {
        let db = mem().await;
        let app = app(db.clone());
        db.execute_unprepared("INSERT INTO tbl_runtime (sn, wid, code, name, sort) VALUES (1, 1, 'codex', 'Codex CLI', 1), (2, 1, 'claude_code', 'Claude Code', 0);").await.unwrap();
        let v = call(&app, "GET", "/runtimes", None).await.1;
        assert_eq!((v[0]["code"].as_str(), v[1]["code"].as_str()), (Some("claude_code"), Some("codex")));

        // 연결: 생성 응답 · 조회 · 목록 어디에도 key_ref가 없고 key_hint만 있다
        let new = json!({"kind": "api_key", "provider_code": "openai", "provider_name": "OpenAI", "name": "work", "runtime_sn": 1, "key_ref": "orch.openai.work", "key_hint": "3f9a"});
        let (st, c) = call(&app, "POST", "/connections", Some(new)).await;
        assert_eq!((st, c["status"].as_str(), c["key_hint"].as_str(), c.get("key_ref")), (StatusCode::CREATED, Some("available"), Some("3f9a"), None));
        let cs = c["sn"].as_i64().unwrap();
        let (_, c) = call(&app, "PATCH", &format!("/connections/{cs}"), Some(json!({"name": "team", "key_ref": "orch.openai.team", "budget_warn_percent": 70}))).await;
        assert_eq!((c["name"].as_str(), c["budget_warn_percent"].as_i64(), c.get("key_ref")), (Some("team"), Some(70), None));
        assert!(call(&app, "GET", &format!("/connections/{cs}"), None).await.1.get("key_ref").is_none());
        assert!(call(&app, "GET", "/connections", None).await.1[0].get("key_ref").is_none());
        let payload: Vec<String> = { use crate::entity::tbl_log_event as ev; use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
            ev::Entity::find().filter(ev::Column::AggregateType.eq("connection")).all(&db).await.unwrap().into_iter().map(|r| r.payload_json).collect() };
        assert!(payload.iter().all(|p| !p.contains("orch.openai")), "{payload:?}");
        assert_eq!(events(&db, "connection", cs).await, [("ConnectionCreated".into(), 1), ("ConnectionUpdated".into(), 2)]);
        for bad in [json!({"kind": "x", "provider_code": "a", "provider_name": "a", "name": "a"}), json!({"kind": "local", "provider_code": "a", "provider_name": "a", "name": "a", "scope": "x"})] {
            assert_eq!(call(&app, "POST", "/connections", Some(bad)).await.0, StatusCode::UNPROCESSABLE_ENTITY);
        }
        assert_eq!(call(&app, "PATCH", &format!("/connections/{cs}"), Some(json!({"budget_warn_percent": 101}))).await.0, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(call(&app, "PATCH", "/connections/99", Some(json!({"name": "x"}))).await.0, StatusCode::NOT_FOUND);

        // 한도: 연결별 목록 · 없는 연결 404
        db.execute_unprepared(&format!("INSERT INTO tbl_connection_quota (connection_sn, period, unit, used_value, remain_percent) VALUES ({cs}, 'week', 'percent', 82, 18);")).await.unwrap();
        let v = call(&app, "GET", &format!("/connections/{cs}/quotas"), None).await.1;
        assert_eq!((v[0]["period"].as_str(), v[0]["remain_percent"].as_i64()), (Some("week"), Some(18)));
        assert_eq!(call(&app, "GET", "/connections/99/quotas", None).await.0, StatusCode::NOT_FOUND);

        // 삭제: 폴백 체인에 쓰이면 409, 빠지면 204 (한도도 함께 지워짐)
        let ps = call(&app, "POST", "/profiles", Some(json!({}))).await.1["sn"].as_i64().unwrap();
        call(&app, "PUT", &format!("/profiles/{ps}/fallbacks"), Some(json!([{"runtime_sn": 1, "connection_sn": cs, "tier": null}]))).await;
        assert_eq!(call(&app, "DELETE", &format!("/connections/{cs}"), None).await.0, StatusCode::CONFLICT);
        call(&app, "PUT", &format!("/profiles/{ps}/fallbacks"), Some(json!([]))).await;
        assert_eq!(call(&app, "DELETE", &format!("/connections/{cs}"), None).await.0, StatusCode::NO_CONTENT);
        assert_eq!(call(&app, "GET", &format!("/connections/{cs}"), None).await.0, StatusCode::NOT_FOUND);

        // 워크스페이스: 기본값 · 수정 유지 · 모르는 테마 422
        let w = call(&app, "GET", "/workspace", None).await.1;
        assert_eq!((w["ui_language"].as_str(), w["theme"].as_str(), w["is_onboarded"].as_i64()), (Some("ko"), Some("system"), Some(0)));
        call(&app, "PATCH", "/workspace", Some(json!({"timezone": "UTC", "default_repo": "orchstack/app", "is_onboarded": 1}))).await;
        let w = call(&app, "GET", "/workspace", None).await.1;
        assert_eq!((w["timezone"].as_str(), w["default_repo"].as_str(), w["is_onboarded"].as_i64()), (Some("UTC"), Some("orchstack/app"), Some(1)));
        assert_eq!(call(&app, "PATCH", "/workspace", Some(json!({"theme": "blue"}))).await.0, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(events(&db, "workspace", WID).await.len(), 1);

        // 프리셋: 종류 필터 · 버전 최신순 · 없는 프리셋 404
        db.execute_unprepared(
            "INSERT INTO tbl_instruction_preset (sn, wid, kind, preset_key, name, version, limit_tok, is_builtin) VALUES (1, 1, 'role', 'frontend', 'Frontend', 2, 400, 1), (2, 1, 'style', 'terse', 'Terse', 1, 200, 1); \
             INSERT INTO tbl_instruction_preset_version (preset_sn, version, content, source) VALUES (1, 1, 'v1', 'builtin'), (1, 2, 'v2', 'user');",
        ).await.unwrap();
        let v = call(&app, "GET", "/presets?kind=role", None).await.1;
        assert_eq!((v.as_array().unwrap().len(), v[0]["preset_key"].as_str()), (1, Some("frontend")));
        let v = call(&app, "GET", "/presets/1/versions", None).await.1;
        assert_eq!((v[0]["version"].as_i64(), v[0]["content"].as_str(), v[1]["version"].as_i64()), (Some(2), Some("v2"), Some(1)));
        assert_eq!(call(&app, "GET", "/presets/99/versions", None).await.0, StatusCode::NOT_FOUND);
    }

    /// B-8: 판단 요청 생성(내부) → 목록 범위 필터 → 작성 중 → 답변(상태 · 선택 · 이벤트 · 발행) → 재답변 409. 승인 요청 승인 · 거부 · 재처리 409
    #[tokio::test]
    async fn decision() {
        use crate::{approval::{self, ApprovalNew}, decision::{self, ChoiceNew, DecisionNew, QuestionNew}};
        use axum::response::IntoResponse;
        let db = mem().await;
        let app = app(db.clone());
        let ts = task_of(&app, &db, true).await;
        let opt = |code: &str| ChoiceNew { code: code.into(), label: code.into(), note: None, is_recommended: code == "A" };
        let qn = |t: &str| QuestionNew { title: t.into(), body: None, code_snippet: None, ref_json: None, options: vec![opt("A"), opt("B")] };
        let new = |level| DecisionNew { project_sn: 1, task_sn: Some(ts), run_sn: None, member_sn: 1, level, title: "D".into(), deadline_at: None, questions: vec![qn("Q1"), qn("Q2")] };
        assert_eq!(decision::create(&db, new(1)).await.err().unwrap().into_response().status(), StatusCode::UNPROCESSABLE_ENTITY);
        decision::create(&db, new(2)).await.unwrap();

        // 목록: 상태 · 프로젝트 · 태스크 범위. 질문 · 선택지를 순서대로 함께 준다
        let v = call(&app, "GET", &format!("/decisions?status=pending&task_sn={ts}"), None).await.1;
        assert_eq!((v.as_array().unwrap().len(), v[0]["questions"][1]["title"].as_str(), v[0]["questions"][0]["options"][1]["code"].as_str()), (1, Some("Q2"), Some("B")));
        assert_eq!(call(&app, "GET", "/decisions?project_sn=99", None).await.1.as_array().unwrap().len(), 0);
        let ds = v[0]["sn"].as_i64().unwrap();
        let (q1, q2) = (v[0]["questions"][0]["sn"].as_i64().unwrap(), v[0]["questions"][1]["sn"].as_i64().unwrap());
        let (b1, b2) = (v[0]["questions"][0]["options"][1]["sn"].as_i64().unwrap(), v[0]["questions"][1]["options"][0]["sn"].as_i64().unwrap());

        // 작성 중: 타이머 멈춤 · 두 번째는 409
        let (st, d) = call(&app, "POST", &format!("/decisions/{ds}/writing"), None).await;
        assert_eq!((st, d["status"].as_str(), d["is_timer_pause"].as_i64()), (StatusCode::OK, Some("writing"), Some(1)));
        assert_eq!(call(&app, "POST", &format!("/decisions/{ds}/writing"), None).await.0, StatusCode::CONFLICT);

        // 답변 검사: 질문 빠짐 · 다른 질문의 선택지 · 빈 답은 422 (상태는 그대로)
        let uri = format!("/decisions/{ds}/answer");
        for bad in [json!({"answers": [{"question_sn": q1, "option_sn": b1}]}),
                    json!({"answers": [{"question_sn": q1, "option_sn": b2}, {"question_sn": q2, "delegate": true}]}),
                    json!({"answers": [{"question_sn": q1}, {"question_sn": q2, "delegate": true}]})] {
            assert_eq!(call(&app, "POST", &uri, Some(bad)).await.0, StatusCode::UNPROCESSABLE_ENTITY);
        }

        // 답변: answered · 사용자 결정 · 선택 1개 · 맡김 · 이벤트 3개 · 구독자에게 발행
        let mut rx = event::subscribe();
        let (st, d) = call(&app, "POST", &uri, Some(json!({"answers": [{"question_sn": q1, "option_sn": b1, "text": "짧게"}, {"question_sn": q2, "delegate": true}], "review_needed": true}))).await;
        assert_eq!((st, d["status"].as_str(), d["decide_by"].as_str(), d["is_timer_pause"].as_i64(), d["is_review_needed"].as_i64()), (StatusCode::OK, Some("answered"), Some("user"), Some(0), Some(1)));
        let o = &d["questions"][0]["options"];
        assert_eq!((o[0]["is_selected"].as_i64(), o[1]["is_selected"].as_i64(), d["questions"][0]["answer_text"].as_str(), d["questions"][1]["is_delegate"].as_i64()), (Some(0), Some(1), Some("짧게"), Some(1)));
        assert_eq!(events(&db, "decision", ds).await.iter().map(|e| e.0.as_str()).collect::<Vec<_>>(), ["DecisionRequested", "DecisionWriting", "DecisionAnswered"]);
        // 버스는 프로세스 전역 — 병렬 테스트의 다른 이벤트는 건너뛴다
        let e = loop { let e = rx.recv().await.unwrap(); if e.event_type == "DecisionAnswered" { break e; } };
        assert_eq!((e.aggregate_sn, e.project_sn), (ds, Some(1)));
        assert_eq!(call(&app, "POST", &uri, Some(json!({"answers": [{"question_sn": q1, "option_sn": b1}, {"question_sn": q2, "delegate": true}]}))).await.0, StatusCode::CONFLICT);
        assert_eq!(call(&app, "POST", "/decisions/99/answer", Some(json!({"answers": []}))).await.0, StatusCode::NOT_FOUND);

        // 승인 요청: 모르는 동작 422 · 승인 · 거부 · 다시 처리 409 · 상태 필터
        let an = |code: &str| ApprovalNew { project_sn: 1, task_sn: Some(ts), run_sn: None, member_sn: 1, rule_sn: None, action_code: code.into(), title: "A".into(), detail: None, deadline_at: None };
        assert_eq!(approval::create(&db, an("x")).await.err().unwrap().into_response().status(), StatusCode::UNPROCESSABLE_ENTITY);
        approval::create(&db, an("pr_merge")).await.unwrap();
        approval::create(&db, an("run_extend")).await.unwrap();
        assert_eq!(call(&app, "GET", "/approvals?status=pending&project_sn=1", None).await.1.as_array().unwrap().len(), 2);
        let (st, a) = call(&app, "POST", "/approvals/1/approve", None).await;
        assert_eq!((st, a["status"].as_str(), a["uid"].as_i64(), a["decide_at"].is_string()), (StatusCode::OK, Some("approved"), Some(UID), true));
        assert_eq!(call(&app, "POST", "/approvals/2/deny", None).await.1["status"], "denied");
        assert_eq!(call(&app, "POST", "/approvals/1/deny", None).await.0, StatusCode::CONFLICT);
        assert_eq!(call(&app, "GET", "/approvals?status=pending", None).await.1.as_array().unwrap().len(), 0);
        assert_eq!(events(&db, "approval", 1).await.iter().map(|e| e.0.as_str()).collect::<Vec<_>>(), ["ApprovalRequested", "ApprovalApproved"]);
    }

    /// B-9: 스킬 소스 · 스킬 · MCP 조회, 허용/차단(이벤트 · 검사 실패는 차단 해제 409), 사용처(멤버 · 템플릿), 차단 · 미검사 스킬은 프로필 연결 422
    #[tokio::test]
    async fn skill() {
        let db = mem().await;
        let app = app(db.clone());
        db.execute_unprepared(
            "INSERT INTO tbl_skill_source (sn, wid, kind, name, sort) VALUES (1, 1, 'github', 'team', 1), (2, 1, 'builtin', 'base', 0); \
             INSERT INTO tbl_skill (sn, wid, source_sn, name, scan_status) VALUES (1, 1, 1, 'svelte-ui', 'passed'), (2, 1, 1, 'bad', 'failed'), (3, 1, 2, 'new', 'pending'); \
             INSERT INTO tbl_mcp (sn, wid, name, install_status) VALUES (1, 1, 'playwright', 'installed'); \
             INSERT INTO tbl_agent_profile (sn, wid, kind) VALUES (1, 1, 'member'), (2, 1, 'template'); \
             INSERT INTO tbl_team (sn, wid, name) VALUES (1, 1, 'T'); INSERT INTO tbl_member (team_sn, profile_sn, name, role_name) VALUES (1, 1, '진', 'Dev'); \
             INSERT INTO tbl_template (sn, wid, name) VALUES (1, 1, 'Frontend'); INSERT INTO tbl_template_revision (template_sn, profile_sn, version, status) VALUES (1, 2, 1, 'live'); \
             INSERT INTO tbl_map_profile_mcp (profile_sn, mcp_sn, access_mode) VALUES (1, 1, 'installed');",
        ).await.unwrap();
        let v = call(&app, "GET", "/skill-sources", None).await.1;
        assert_eq!((v[0]["name"].as_str(), v[1]["name"].as_str()), (Some("base"), Some("team")));
        let v = call(&app, "GET", "/skills", None).await.1;
        assert_eq!((v.as_array().unwrap().len(), v[0]["name"].as_str()), (3, Some("bad")));
        assert_eq!(call(&app, "GET", "/skills/99", None).await.0, StatusCode::NOT_FOUND);

        // 허용/차단: 보낸 필드만 · 0/1 밖 422 · 검사 실패 스킬 차단 해제 409
        let (st, s) = call(&app, "PATCH", "/skills/1", Some(json!({"is_enabled": 0}))).await;
        assert_eq!((st, s["is_enabled"].as_i64(), s["is_blocked"].as_i64()), (StatusCode::OK, Some(0), Some(0)));
        assert_eq!(call(&app, "PATCH", "/skills/1", Some(json!({"is_blocked": 2}))).await.0, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(call(&app, "PATCH", "/skills/2", Some(json!({"is_blocked": 0}))).await.0, StatusCode::CONFLICT);
        assert_eq!(events(&db, "workspace", WID).await, [("SkillUpdated".into(), 1)]);

        // 프로필 연결: 통과 스킬만. 차단 · 검사 실패 · 검사 전은 422, 연결은 그대로
        assert_eq!(call(&app, "PUT", "/profiles/1/skills", Some(json!([{"skill_sn": 1, "is_enabled": 1}]))).await.0, StatusCode::OK);
        call(&app, "PATCH", "/skills/1", Some(json!({"is_blocked": 1}))).await;
        for sn in [1, 2, 3] {
            assert_eq!(call(&app, "PUT", "/profiles/2/skills", Some(json!([{"skill_sn": sn, "is_enabled": 1}]))).await.0, StatusCode::UNPROCESSABLE_ENTITY);
        }
        assert_eq!(call(&app, "GET", "/profiles/2/caps", None).await.1["skills"].as_array().unwrap().len(), 0);
        assert_eq!(call(&app, "PUT", "/profiles/99/skills", Some(json!([]))).await.0, StatusCode::NOT_FOUND);

        // 사용처: 스킬은 멤버 '진'(on) · MCP는 멤버(installed). 템플릿 버전 프로필도 잡힌다
        let v = call(&app, "GET", "/skills/1/usage", None).await.1;
        assert_eq!((v.as_array().unwrap().len(), v[0]["owner"].as_str(), v[0]["name"].as_str(), v[0]["detail"].as_str()), (1, Some("member"), Some("진"), Some("on")));
        db.execute_unprepared("INSERT INTO tbl_map_profile_mcp (profile_sn, mcp_sn) VALUES (2, 1);").await.unwrap();
        let v = call(&app, "GET", "/mcps/1/usage", None).await.1;
        assert_eq!((v[0]["detail"].as_str(), v[1]["owner"].as_str(), v[1]["name"].as_str(), v[1]["detail"].as_str()), (Some("installed"), Some("template"), Some("Frontend"), Some("accessible")));
        assert_eq!(call(&app, "GET", "/mcps", None).await.1[0]["install_status"], "installed");
        assert_eq!(call(&app, "GET", "/mcps/9/usage", None).await.0, StatusCode::NOT_FOUND);
    }

    /// B-10: 판단 · 승인 요청 · Run 실패 → 알림(같은 트랜잭션 · event_sn · project_sn), 앱 규칙 끄면 안 만듦, 탭 · after · 묶음 · 읽음,
    /// 규칙 교체 검사, 채널(key_ref 없음) · 테스트, 방해 금지 설정, 감사 로그 필터
    #[tokio::test]
    async fn notify() {
        use crate::{approval::{self, ApprovalNew}, decision::{self, ChoiceNew, DecisionNew, QuestionNew}};
        let db = mem().await;
        let app = app(db.clone());
        let ts = task_of(&app, &db, true).await;
        let q = QuestionNew { title: "Q".into(), body: None, code_snippet: None, ref_json: None, options: vec![ChoiceNew { code: "A".into(), label: "a".into(), note: None, is_recommended: false }] };
        decision::create(&db, DecisionNew { project_sn: 1, task_sn: Some(ts), run_sn: None, member_sn: 1, level: 2, title: "재전송 제한".into(), deadline_at: None, questions: vec![q] }).await.unwrap();
        let rs = call(&app, "POST", &format!("/tasks/{ts}/runs"), None).await.1["sn"].as_i64().unwrap();
        run::run_to(&db, rs, "starting").await.unwrap();
        run::run_to(&db, rs, "failed").await.unwrap();

        // 알림 2건: 최신순 · 판단 요청은 확인 필요 묶음 + 프로젝트 · 바로가기, Run 실패는 오늘 묶음
        let v = call(&app, "GET", "/notifications", None).await.1;
        assert_eq!((v.as_array().unwrap().len(), v[0]["event_code"].as_str(), v[0]["group"].as_str(), v[0]["ref_type"].as_str(), v[0]["title"].as_str()), (2, Some("run_failed"), Some("today"), Some("run"), Some("T")));
        assert_eq!((v[1]["event_code"].as_str(), v[1]["group"].as_str(), v[1]["ref_sn"].as_i64(), v[1]["project_sn"].as_i64(), v[1]["is_action"].as_i64()), (Some("decision_request"), Some("need"), Some(1), Some(1), Some(1)));
        assert_eq!(call(&app, "GET", "/notifications?tab=need", None).await.1.as_array().unwrap().len(), 1);
        assert_eq!(call(&app, "GET", "/notifications?tab=run", None).await.1[0]["event_code"], "run_failed");
        assert_eq!(call(&app, "GET", "/notifications?tab=quota", None).await.1.as_array().unwrap().len(), 0);
        assert_eq!(call(&app, "GET", &format!("/notifications?after={}", v[1]["sn"]), None).await.1.as_array().unwrap().len(), 1);
        assert_eq!(call(&app, "GET", "/notifications?tab=x", None).await.0, StatusCode::UNPROCESSABLE_ENTITY);

        // 읽음: 개별 → 확인 필요 묶음에서 빠짐 · 전체 · 다시 하면 0
        let ns = v[1]["sn"].as_i64().unwrap();
        assert_eq!(call(&app, "POST", "/notifications/read", Some(json!({"sns": [ns]}))).await.1["updated"], 1);
        assert_eq!(call(&app, "GET", "/notifications", None).await.1[1]["group"], "today");
        assert_eq!(call(&app, "POST", "/notifications/read", Some(json!({}))).await.1["updated"], 1);
        assert_eq!(call(&app, "POST", "/notifications/read", Some(json!({}))).await.1["updated"], 0);

        // 규칙: 앱 승인 알림 끄면 승인 요청은 알림을 만들지 않는다 · 잘못된 칸 · 중복 422
        let rule = json!({"connection_sn": null, "event_code": "approval_request", "channel_kind": "app", "is_enabled": 0});
        assert_eq!(call(&app, "PUT", "/notify/rules", Some(json!([rule]))).await.0, StatusCode::OK);
        approval::create(&db, ApprovalNew { project_sn: 1, task_sn: None, run_sn: None, member_sn: 1, rule_sn: None, action_code: "pr_merge".into(), title: "A".into(), detail: None, deadline_at: None }).await.unwrap();
        assert_eq!(call(&app, "GET", "/notifications", None).await.1.as_array().unwrap().len(), 2);
        assert_eq!(call(&app, "GET", "/notify/rules", None).await.1[0]["is_enabled"], 0);
        for bad in [json!([rule, rule]), json!([{"event_code": "x", "channel_kind": "app", "is_enabled": 1}]), json!([{"event_code": "pr", "channel_kind": "sms", "is_enabled": 1}])] {
            assert_eq!(call(&app, "PUT", "/notify/rules", Some(bad)).await.0, StatusCode::UNPROCESSABLE_ENTITY);
        }
        assert_eq!(events(&db, "workspace", WID).await, [("NotifyRulesUpdated".into(), 1)]);

        // 채널 · 테스트: key_ref 없음, app 항상 준비, off 채널은 준비 안 됨
        db.execute_unprepared("INSERT INTO tbl_notify_channel (wid, kind, status, key_ref) VALUES (1, 'telegram', 'connected', 'orch.tg'), (1, 'email', 'off', NULL);").await.unwrap();
        let v = call(&app, "GET", "/notify/channels", None).await.1;
        assert_eq!((v.as_array().unwrap().len(), v[0].get("key_ref")), (2, None));
        for (k, ready) in [("app", true), ("telegram", true), ("email", false), ("desktop", false)] {
            let v = call(&app, "POST", "/notify/test", Some(json!({"kind": k}))).await.1;
            assert_eq!((v["ready"].as_bool(), v["delivered"].as_bool()), (Some(ready), Some(false)), "{k}");
        }
        assert_eq!(call(&app, "POST", "/notify/test", Some(json!({"kind": "sms"}))).await.0, StatusCode::UNPROCESSABLE_ENTITY);

        // 방해 금지 · 일일 요약: 저장 · HH:MM · 레벨 검사
        let w = call(&app, "PATCH", "/workspace", Some(json!({"dnd_start": "22:00", "dnd_end": "08:00", "dnd_bypass_level": 4, "daily_summary_time": "18:00"}))).await.1;
        assert_eq!((w["dnd_start"].as_str(), w["dnd_bypass_level"].as_i64(), w["daily_summary_time"].as_str()), (Some("22:00"), Some(4), Some("18:00")));
        for bad in [json!({"dnd_start": "24:00"}), json!({"daily_summary_time": "9:00"}), json!({"dnd_bypass_level": 5})] {
            assert_eq!(call(&app, "PATCH", "/workspace", Some(bad)).await.0, StatusCode::UNPROCESSABLE_ENTITY);
        }

        // 감사 로그: 최신순 · 종류 · limit · 모르는 종류 422
        db.execute_unprepared("INSERT INTO tbl_log_audit (wid, actor_type, kind, title) VALUES (1, 'user', 'KEY', 'k'), (1, 'member', 'BLOCK', 'git reset'), (1, 'user', 'BLOCK', 'git stash');").await.unwrap();
        let v = call(&app, "GET", "/audit?kind=BLOCK&limit=1", None).await.1;
        assert_eq!((v.as_array().unwrap().len(), v[0]["title"].as_str()), (1, Some("git stash")));
        assert_eq!(call(&app, "GET", "/audit", None).await.1.as_array().unwrap().len(), 3);
        assert_eq!(call(&app, "GET", "/audit?kind=x", None).await.0, StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn openapi() {
        let (st, v) = call(&setup().await, "GET", "/openapi.json", None).await;
        assert_eq!(st, StatusCode::OK);
        for path in ["/health", "/projects", "/projects/{sn}", "/projects/{sn}/issues", "/issues/{sn}", "/issues/{sn}/tasks", "/projects/{sn}/tasks", "/tasks/{sn}", "/tasks/{sn}/move", "/tasks/{sn}/runs", "/runs/{sn}", "/runs/{sn}/sessions", "/runs/{sn}/stop", "/runs/{sn}/retry", "/runs/{sn}/review", "/runs/{sn}/approve", "/runs/{sn}/reject", "/tasks/{sn}/assign", "/profiles", "/profiles/{sn}", "/profiles/{sn}/caps", "/profiles/{sn}/fallbacks", "/profiles/{sn}/commands", "/templates", "/templates/{sn}", "/teams", "/teams/{sn}", "/teams/{sn}/members", "/members/{sn}", "/connections", "/connections/{sn}", "/connections/{sn}/quotas", "/runtimes", "/workspace", "/presets", "/presets/{sn}/versions", "/decisions", "/decisions/{sn}", "/decisions/{sn}/answer", "/decisions/{sn}/writing", "/approvals", "/approvals/{sn}/approve", "/approvals/{sn}/deny", "/skill-sources", "/skills", "/skills/{sn}", "/skills/{sn}/usage", "/mcps", "/mcps/{sn}", "/mcps/{sn}/usage", "/profiles/{sn}/skills", "/notifications", "/notifications/read", "/notify/rules", "/notify/channels", "/notify/test", "/audit"] {
            assert!(v["paths"][path].is_object(), "{path}");
        }
        assert!(v["components"]["schemas"]["ErrorBody"].is_object());
    }

    /// 실패는 전부 { error, message } JSON이어야 한다
    #[tokio::test]
    async fn error() {
        let app = setup().await;
        let cases = [
            ("GET", "/projects/99", None, StatusCode::NOT_FOUND, "not_found"),
            ("GET", "/projects/abc", None, StatusCode::BAD_REQUEST, "bad_path"),
            ("POST", "/projects", Some(json!({})), StatusCode::UNPROCESSABLE_ENTITY, "bad_body"), // name 없음
            ("POST", "/projects", Some(json!({"name": "x", "team_sn": 99})), StatusCode::UNPROCESSABLE_ENTITY, "invalid_ref"), // 없는 팀
            ("PATCH", "/projects/99", Some(json!({"name": "x"})), StatusCode::NOT_FOUND, "not_found"),
            ("DELETE", "/projects/99", None, StatusCode::NOT_FOUND, "not_found"),
        ];
        for (method, uri, body, status, code) in cases {
            let (st, v) = call(&app, method, uri, body).await;
            assert_eq!((st, v["error"].as_str()), (status, Some(code)), "{method} {uri}");
            assert!(v["message"].is_string(), "{method} {uri}");
        }
    }

    /// 스트림에서 `marker`가 들어 있는 청크가 나올 때까지 읽고, 그때까지 받은 청크 전부를 돌려준다.
    /// 발행 버스는 프로세스 전역이라 병렬 테스트의 이벤트도 섞여 오므로 id가 아니라 내용으로 찾는다
    async fn until(body: &mut Body, marker: &str) -> Vec<String> {
        let mut got = Vec::new();
        loop {
            let f = body.frame().await.expect("stream ended").unwrap();
            let s = String::from_utf8(f.into_data().unwrap().to_vec()).unwrap();
            if s.starts_with(':') {
                continue; // 연결 직후 주석(: ok)
            }
            let hit = s.contains(marker);
            got.push(s);
            if hit {
                return got;
            }
        }
    }

    /// 스트림 요청을 보내고 응답 본문(스트림)을 돌려준다
    async fn open_stream(app: &Router, sn: i64, last: Option<i64>) -> Body {
        let mut req = Request::builder().method("GET").uri(format!("/projects/{sn}/stream"));
        if let Some(l) = last {
            req = req.header("last-event-id", l.to_string());
        }
        let res = app.clone().oneshot(req.body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert!(res.headers()["content-type"].to_str().unwrap().starts_with("text/event-stream"));
        res.into_body()
    }

    /// B-5: snapshot → 변경 → 연결된 SSE 클라이언트가 그 이벤트(id = 이벤트 sn · event = 종류)를 받는다. 다른 프로젝트 이벤트는 오지 않는다
    #[tokio::test]
    async fn stream_live() {
        let db = mem().await;
        let app = app(db.clone());
        let ts = task_of(&app, &db, false).await;
        let (_, other) = call(&app, "POST", "/projects", Some(json!({"name": "other"}))).await;
        let (st, snap) = call(&app, "GET", "/projects/1/snapshot", None).await;
        assert_eq!((st, snap["tasks"].as_array().unwrap().len(), snap["issues"].as_array().unwrap().len()), (StatusCode::OK, 1, 1));
        let last = snap["last_event_sn"].as_i64().unwrap();
        assert!(last > 0);
        assert_eq!(call(&app, "GET", "/projects/99/snapshot", None).await.0, StatusCode::NOT_FOUND);

        let mut body = open_stream(&app, 1, None).await;
        call(&app, "POST", &format!("/projects/{}/issues", other["sn"]), Some(json!({"title": "zz-other-project"}))).await;
        let (_, m) = call(&app, "POST", "/projects/1/issues", Some(json!({"title": "zz-live-marker"}))).await;
        let got = until(&mut body, "zz-live-marker").await;
        let s = got.last().unwrap();
        assert!(s.starts_with(&format!("id: {}\nevent: IssueCreated\n", last + 2)) && s.contains(&format!(r#""aggregate_sn":{}"#, m["sn"])), "{s}");
        assert!(!got.iter().any(|c| c.contains("zz-other-project")), "{got:?}");

        // DoD: Task move → 연결된 클라이언트가 TaskMoved 수신. blocked는 이 테스트만 쓴다 (move_task가 버스의 TaskMoved를 센다)
        call(&app, "POST", &format!("/tasks/{ts}/move"), Some(json!({"status": "blocked"}))).await;
        let s = until(&mut body, r#""to":"blocked""#).await.pop().unwrap();
        assert!(s.contains("event: TaskMoved\n") && s.contains(&format!(r#""aggregate_sn":{ts}"#)), "{s}");
    }

    /// B-5: 스트림이 끊긴 사이 이벤트 2개 → events?after= 로 2개 모두 순서대로, Last-Event-ID 재연결도 같은 2개를 먼저 보낸 뒤 실시간으로 잇는다. 재연결은 이벤트를 만들지 않는다
    #[tokio::test]
    async fn stream_catchup() {
        use crate::entity::tbl_log_event as ev;
        use sea_orm::{EntityTrait, PaginatorTrait};
        let db = mem().await;
        let app = app(db.clone());
        // 다른 테스트와 겹치지 않게 TaskMoved 대신 IssueCreated(고유 제목)로 센다 (move_task 테스트가 버스의 TaskMoved 수를 센다)
        task_of(&app, &db, false).await;
        let last = call(&app, "GET", "/projects/1/snapshot", None).await.1["last_event_sn"].as_i64().unwrap();
        for t in ["zz-c1", "zz-c2"] {
            call(&app, "POST", "/projects/1/issues", Some(json!({"title": t}))).await;
        }
        let (st, list) = call(&app, "GET", &format!("/projects/1/events?after={last}"), None).await;
        let kinds: Vec<_> = list.as_array().unwrap().iter().map(|e| (e["sn"].as_i64().unwrap(), e["payload"]["title"].as_str().unwrap().to_string())).collect();
        assert_eq!((st, &kinds), (StatusCode::OK, &vec![(last + 1, "zz-c1".into()), (last + 2, "zz-c2".into())]));
        assert_eq!(call(&app, "GET", &format!("/projects/1/events?after={}", last + 2), None).await.1.as_array().unwrap().len(), 0);

        let before = ev::Entity::find().count(&db).await.unwrap();
        let mut body = open_stream(&app, 1, Some(last)).await;
        let a = until(&mut body, "zz-c1").await;
        let b = until(&mut body, "zz-c2").await;
        assert!(a.len() == 1 && a[0].starts_with(&format!("id: {}\n", last + 1)), "{a:?}");
        assert!(b.len() == 1 && b[0].starts_with(&format!("id: {}\n", last + 2)), "{b:?}");
        // 보충 뒤 실시간으로 이어진다
        call(&app, "POST", "/projects/1/issues", Some(json!({"title": "zz-catchup-marker"}))).await;
        assert!(until(&mut body, "zz-catchup-marker").await.last().unwrap().starts_with(&format!("id: {}\n", last + 3)));
        assert_eq!(ev::Entity::find().count(&db).await.unwrap(), before + 1);
    }
}
