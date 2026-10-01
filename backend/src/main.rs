// OrchStack 백엔드 진입점: DB 준비 → 라우터 구성 → HTTP 서버 실행

mod agent; // /profiles CRUD · /templates 조회
#[allow(unused_imports, dead_code)] // sea-orm-cli 생성 코드
mod entity; // 테이블별 SeaORM entity (sea-orm-cli 생성물 · 직접 수정하지 않는다)
mod error; // 공통 에러 응답
mod event; // 명령 실행 틀 (상태 변경 + 이벤트 append + 발행)
mod exec; // 실행기: Claude Code · Codex CLI 비대화형 실행 (#13)
mod issue; // /issues CRUD
mod project; // /projects CRUD
mod run; // /runs · Run 명령 · 실행기용 전이 함수
mod rule; // 하위 작업 규칙 엔진 (#67 · LLM 0)
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
    OpenApiRouter::with_openapi(Doc::openapi()).routes(routes!(health)).merge(project::routes()).merge(issue::routes()).merge(task::routes()).merge(run::routes()).merge(agent::routes()).merge(team::routes()).split_for_parts()
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
            moved += (m.event_type == "TaskMoved") as i32;
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
    #[tokio::test]
    async fn openapi() {
        let (st, v) = call(&setup().await, "GET", "/openapi.json", None).await;
        assert_eq!(st, StatusCode::OK);
        for path in ["/health", "/projects", "/projects/{sn}", "/projects/{sn}/issues", "/issues/{sn}", "/issues/{sn}/tasks", "/projects/{sn}/tasks", "/tasks/{sn}", "/tasks/{sn}/move", "/tasks/{sn}/runs", "/runs/{sn}", "/runs/{sn}/sessions", "/runs/{sn}/stop", "/runs/{sn}/retry", "/runs/{sn}/review", "/runs/{sn}/approve", "/runs/{sn}/reject", "/tasks/{sn}/assign", "/profiles", "/profiles/{sn}", "/profiles/{sn}/caps", "/profiles/{sn}/fallbacks", "/templates", "/templates/{sn}", "/teams", "/teams/{sn}", "/teams/{sn}/members", "/members/{sn}"] {
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
}
