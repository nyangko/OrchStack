// OrchStack 백엔드 진입점: DB 준비 → 라우터 구성 → HTTP 서버 실행

mod agent; // /profiles CRUD · /templates 조회
mod ask; // /asks: 판단 요청 · 승인 요청 · Orch 제안 (조회 · 답변 · 승인 · 거부 · 진행 · 수정 · 취소 · 보류 · 무시) + 제안 실행 · 타이머
mod connection; // /connections CRUD · 한도 조회
#[allow(unused_imports, dead_code)] // sea-orm-cli 생성 코드
mod entity; // 테이블별 SeaORM entity (sea-orm-cli 생성물 · 직접 수정하지 않는다)
mod error; // 공통 에러 응답
mod event; // 명령 실행 틀 (상태 변경 + 이벤트 append + 발행)
mod exec; // 실행기: Claude Code · Codex CLI 비대화형 실행 (#13)
mod issue; // /issues CRUD
mod context; // 컨텍스트 조립기 (고정 접두 · 반복 · 상한 · 견적) + /tasks/{sn}/estimate · /runs/{sn}/context
mod orch_rule; // Orch 규칙 엔진 (LLM 0): 이벤트 → 제안 · 제안 실행 · 타이머 · 가드 · 멤버 대기열
mod orch; // PM Dock: /projects/{sn}/conversation · messages · /messages/* · /runs/{sn}/instruct
mod meta; // 완료 조건 · 의존 · 라벨 목록 · 저장 보기 · Diagram 배치
mod policy; // Orch 진행 정책 (팀 · 프로젝트의 orch_mode · timer_sec · level_json · guard_json 해석 · 검사)
mod notify; // /notifications · /notify/* · /audit + 이벤트 → 알림 projection
mod preset; // 프리셋 편집 · 가져오기 · 사용처 · 보고서 양식 · 미리보기
mod project; // /projects CRUD
mod run; // /runs · Run 명령 · 실행기용 전이 함수
mod rule; // 하위 작업 규칙 엔진 (#67 · LLM 0)
mod runner; // runner 하위 Run 실행 · @REPORT 회수 (#67)
mod setting; // /workspace · /workspace/profile · /runtimes · /presets
mod skill; // /skills · /mcps · /skill-sources 조회 · 스킬 허용/차단
mod stat; // /teams/{sn}/stats · /teams/{sn}/quota · /workspace/cost 집계
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
const USER: i64 = 1;
const WORKSPACE: i64 = 1;

/// 연결 후 테이블이 없으면 data/sqlite.sql로 만들고 기본 사용자 · 워크스페이스를 넣는다
async fn connect(opt: ConnectOptions) -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect(opt).await?;
    let probe = Statement::from_string(DbBackend::Sqlite, "SELECT 1 FROM sqlite_master WHERE name = 'tbl_user'");
    if db.query_one_raw(probe).await?.is_none() {
        // 한 트랜잭션: 중간에 실패해도 반쯤 만들어진 DB가 남지 않는다
        let tx = db.begin().await?;
        tx.execute_unprepared(SCHEMA).await?;
        tx.execute_unprepared(&format!(
            "INSERT INTO tbl_user (sn, name) VALUES ({USER}, 'Me'); \
             INSERT INTO tbl_workspace (sn, user_sn, name) VALUES ({WORKSPACE}, {USER}, 'OrchStack');"
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
    OpenApiRouter::with_openapi(Doc::openapi()).routes(routes!(health)).merge(project::routes()).merge(issue::routes()).merge(task::routes()).merge(run::routes()).merge(agent::routes()).merge(team::routes()).merge(connection::routes()).merge(setting::routes()).merge(stream::routes()).merge(ask::routes()).merge(skill::routes()).merge(notify::routes()).merge(orch::routes()).merge(orch_rule::routes()).merge(context::routes()).merge(stat::routes()).merge(meta::routes()).merge(preset::routes()).split_for_parts()
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
    // Orch 규칙 엔진: 이벤트 소비 + 1초 타이머 (테스트는 on · tick을 직접 부른다)
    tokio::spawn(orch_rule::listen(db.clone()));
    tokio::spawn(orch_rule::ticker(db.clone()));
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

        // 생성: 시드 워크스페이스(workspace_sn=1)에 붙고, DB 기본값(default_branch = 'main')이 채워진다
        let (st, p) = call(&app, "POST", "/projects", Some(json!({"name": "OrchStack"}))).await;
        assert_eq!((st, p["default_branch"].as_str(), p["workspace_sn"].as_i64()), (StatusCode::CREATED, Some("main"), Some(1)));
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
                "INSERT INTO tbl_agent_profile (workspace_sn, kind) VALUES (1, 'workspace'); INSERT INTO tbl_team (workspace_sn, name) VALUES (1, 'T'); \
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

    /// 대상 번호가 sn인 이벤트(kind)의 (actor_type, member_sn, user_sn)
    async fn actor(db: &DatabaseConnection, kind: &str, sn: i64) -> (String, Option<i64>, Option<i64>) {
        use crate::entity::tbl_log_event as ev;
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
        let e = ev::Entity::find().filter(ev::Column::EventType.eq(kind)).filter(ev::Column::AggregateSn.eq(sn)).one(db).await.unwrap().unwrap();
        (e.actor_type, e.member_sn, e.user_sn)
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
        let db = mem().await;
        let app = app(db.clone());
        let status = |sn: i64| { let app = app.clone(); async move { call(&app, "GET", &format!("/runs/{sn}"), None).await.1["status"].as_str().unwrap().to_string() } };

        // 담당 멤버 없음 → 409
        let ts = task_of(&app, &db, false).await;
        assert_eq!(call(&app, "POST", &format!("/tasks/{ts}/runs"), None).await.0, StatusCode::CONFLICT);
        db.execute_unprepared(
            "INSERT INTO tbl_agent_profile (workspace_sn, kind) VALUES (1, 'workspace'); INSERT INTO tbl_team (workspace_sn, name) VALUES (1, 'T'); \
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
        let rj = crate::run::rejects(&db, ts).await.unwrap();
        assert_eq!((rj, scalar(&db, "SELECT review_member_sn FROM tbl_run WHERE sn = 2").await), (1, 1));
        assert_eq!(call(&app, "GET", "/runs/2", None).await.1["fail_detail"], "라벨 누락");
        assert_eq!(events(&db, "run", 2).await.iter().map(|e| e.0.as_str()).collect::<Vec<_>>(),
            ["RunStarted", "RunProgressed", "RunProgressed", "ReviewRequested", "RunRejected"]);

        // 반려 뒤 retry → Run 3, 다시 반려하면 round 2
        call(&app, "POST", "/runs/2/retry", None).await;
        db.execute_unprepared("UPDATE tbl_run SET status = 'running' WHERE sn = 3").await.unwrap();
        call(&app, "POST", "/runs/3/review", None).await;
        call(&app, "POST", "/runs/3/reject", Some(json!({"member_sn": 1}))).await;
        assert_eq!(crate::run::rejects(&db, ts).await.unwrap(), 2); // round = 반려 Run 수

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
            "INSERT INTO tbl_runtime (sn, workspace_sn, code, name) VALUES (1, 1, 'claude_code', 'Claude Code'); \
             INSERT INTO tbl_connection (sn, workspace_sn, kind, provider_code, provider_name, name) VALUES (1, 1, 'subscription', 'anthropic', 'Anthropic', 'max');",
        ).await.unwrap();
        let pu = format!("/profiles/{ps}");
        let chain = json!([{"runtime_sn": 1, "connection_sn": 1, "tier": "S"}, {"runtime_sn": 1, "connection_sn": 1, "tier": null}]);
        let (st, v) = call(&app, "PATCH", &pu, Some(json!({"fallbacks": chain}))).await;
        assert_eq!((st, v["fallbacks"][0]["tier"].as_str(), v["fallbacks"][1]["tier"].is_null()), (StatusCode::OK, Some("S"), true));
        assert_eq!(call(&app, "PATCH", &pu, Some(json!({"fallbacks": [{"runtime_sn": 1, "connection_sn": 1, "tier": "X"}]}))).await.0, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(call(&app, "PATCH", &pu, Some(json!({"fallbacks": [{"runtime_sn": 1, "connection_sn": 9}]}))).await.0, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(call(&app, "GET", &pu, None).await.1["fallbacks"].as_array().unwrap().len(), 2); // 실패한 교체는 기존 체인을 지우지 않는다
        // 도구 · 파일 범위 · 규칙 · 가드: 전체 교체 · 값 검사(모르는 도구 · 중복 도구 · 모르는 정책 · 빈 패턴 · command 패턴 없음 · 모르는 시점)는 422
        let (st, v) = call(&app, "PATCH", &pu, Some(json!({"tools": [{"tool_code": "shell", "policy": "approval"}], "paths": [{"kind": "include", "pattern": "src/**"}, {"kind": "exclude", "pattern": "**/.env*"}],
            "guards": [{"name": "키 감지", "stage": "output", "pattern": "sk-"}]}))).await;
        assert_eq!((st, v["tools"][0]["policy"].as_str(), v["paths"][1]["kind"].as_str(), v["guards"][0]["is_enabled"].as_i64(), v["fallbacks"].as_array().unwrap().len()), (StatusCode::OK, Some("approval"), Some("exclude"), Some(1), 2));
        for bad in [json!({"tools": [{"tool_code": "x", "policy": "allow"}]}), json!({"tools": [{"tool_code": "shell", "policy": "allow"}, {"tool_code": "shell", "policy": "block"}]}),
                    json!({"tools": [{"tool_code": "read", "policy": "x"}]}), json!({"paths": [{"kind": "include", "pattern": " "}]}), json!({"rules": [{"action_code": "command", "title": "t", "policy": "block"}]}),
                    json!({"guards": [{"name": "g", "stage": "x"}]})] {
            assert_eq!(call(&app, "PATCH", &pu, Some(bad)).await.0, StatusCode::UNPROCESSABLE_ENTITY);
        }
        // 명령 사용 여부(응답 안): 기본 3개 차단 → rules의 command 규칙으로 checkout 켜기 · rm -rf 차단 추가 → 기본 목록 순서 뒤에 추가 명령
        let on = |v: &Value| v["commands"].as_array().unwrap().iter().map(|c| (c["cmd"].as_str().unwrap().to_owned(), c["on"].as_bool().unwrap())).collect::<Vec<_>>();
        assert_eq!(on(&call(&app, "GET", &pu, None).await.1), [("git stash".into(), false), ("git checkout".into(), false), ("git reset".into(), false)]);
        let cmd = |c: &str, p: &str| json!({"action_code": "command", "title": c, "pattern": c, "policy": p});
        let v = call(&app, "PATCH", &pu, Some(json!({"rules": [cmd("git checkout", "auto"), cmd("rm -rf", "block")]}))).await.1;
        assert_eq!(on(&v), [("git stash".into(), false), ("git checkout".into(), true), ("git reset".into(), false), ("rm -rf".into(), false)]);
        assert_eq!((v["commands"][3]["builtin"].as_bool(), v["rules"].as_array().unwrap().len()), (Some(false), 2));
        assert_eq!(call(&app, "DELETE", &format!("/profiles/{ps}"), None).await.0, StatusCode::NO_CONTENT);

        // 템플릿(live v2 · 도구 정책 1개)과 draft 템플릿은 SQL로 넣는다 (템플릿 편집은 이 Task 범위 밖)
        db.execute_unprepared(
            "INSERT INTO tbl_agent_profile (sn, workspace_sn, kind, effort, tool_json) VALUES (10, 1, 'template', 'high', '[{\"tool_code\":\"shell\",\"policy\":\"approval\"}]'); \
             INSERT INTO tbl_template (sn, workspace_sn, name, role_name, icon, color) VALUES (1, 1, 'Frontend', 'Frontend Developer', 'monitor', 'role-frontend'); \
             INSERT INTO tbl_template_revision (template_sn, profile_sn, version, status) VALUES (1, 10, 2, 'live'); \
             INSERT INTO tbl_template (sn, workspace_sn, name, status) VALUES (2, 1, 'Draft', 'draft');",
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
        assert_eq!((p["tools"][0]["tool_code"].as_str(), p["tools"][0]["policy"].as_str()), (Some("shell"), Some("approval")));

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
        db.execute_unprepared("INSERT INTO tbl_runtime (sn, workspace_sn, code, name, sort) VALUES (1, 1, 'codex', 'Codex CLI', 1), (2, 1, 'claude_code', 'Claude Code', 0);").await.unwrap();
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
        assert_eq!(call(&app, "GET", &format!("/connections/{cs}"), None).await.1["quotas"].as_array().unwrap().len(), 0);
        db.execute_unprepared(&format!("UPDATE tbl_connection SET quota_json = '[{{\"period\":\"week\",\"unit\":\"percent\",\"used_value\":82,\"remain_percent\":18}}]' WHERE sn = {cs};")).await.unwrap();
        let v = call(&app, "GET", &format!("/connections/{cs}"), None).await.1["quotas"].clone();
        assert_eq!((v[0]["period"].as_str(), v[0]["remain_percent"].as_i64(), v[0]["limit_value"].is_null()), (Some("week"), Some(18), true));
        assert!(call(&app, "GET", "/connections", None).await.1[0]["quotas"].is_array());

        // 삭제: 폴백 체인에 쓰이면 409, 빠지면 204 (한도도 함께 지워짐)
        let ps = call(&app, "POST", "/profiles", Some(json!({}))).await.1["sn"].as_i64().unwrap();
        call(&app, "PATCH", &format!("/profiles/{ps}"), Some(json!({"fallbacks": [{"runtime_sn": 1, "connection_sn": cs, "tier": null}]}))).await;
        assert_eq!(call(&app, "DELETE", &format!("/connections/{cs}"), None).await.0, StatusCode::CONFLICT);
        call(&app, "PATCH", &format!("/profiles/{ps}"), Some(json!({"fallbacks": []}))).await;
        assert_eq!(call(&app, "DELETE", &format!("/connections/{cs}"), None).await.0, StatusCode::NO_CONTENT);
        assert_eq!(call(&app, "GET", &format!("/connections/{cs}"), None).await.0, StatusCode::NOT_FOUND);

        // 워크스페이스: 기본값 · 수정 유지 · 모르는 테마 422
        let w = call(&app, "GET", "/workspace", None).await.1;
        assert_eq!((w["ui_language"].as_str(), w["theme"].as_str(), w["is_onboarded"].as_i64()), (Some("ko"), Some("system"), Some(0)));
        call(&app, "PATCH", "/workspace", Some(json!({"timezone": "UTC", "default_repo": "orchstack/app", "is_onboarded": 1}))).await;
        let w = call(&app, "GET", "/workspace", None).await.1;
        assert_eq!((w["timezone"].as_str(), w["default_repo"].as_str(), w["is_onboarded"].as_i64()), (Some("UTC"), Some("orchstack/app"), Some(1)));
        assert_eq!(call(&app, "PATCH", "/workspace", Some(json!({"theme": "blue"}))).await.0, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(events(&db, "workspace", WORKSPACE).await.len(), 1);

        // 프리셋: 종류 필터 · 버전 최신순 · 없는 프리셋 404
        db.execute_unprepared(
            "INSERT INTO tbl_instruction_preset (sn, workspace_sn, kind, preset_key, name, version, is_latest, content, source, limit_tok, is_builtin) VALUES \
               (1, 1, 'role', 'frontend', 'Frontend', 1, 0, 'v1', 'builtin', 400, 1), (3, 1, 'role', 'frontend', 'Frontend', 2, 1, 'v2', 'user', 400, 1), (2, 1, 'style', 'terse', 'Terse', 1, 1, 'terse', 'builtin', 200, 1);",
        ).await.unwrap();
        let v = call(&app, "GET", "/presets?kind=role", None).await.1;
        assert_eq!((v.as_array().unwrap().len(), v[0]["preset_key"].as_str(), v[0]["version"].as_i64(), v[0]["sn"].as_i64()), (1, Some("frontend"), Some(2), Some(3))); // 최신 행만
        let v = call(&app, "GET", "/presets/1/versions", None).await.1;
        assert_eq!((v[0]["version"].as_i64(), v[0]["content"].as_str(), v[1]["version"].as_i64()), (Some(2), Some("v2"), Some(1)));
        assert_eq!(call(&app, "GET", "/presets/99/versions", None).await.0, StatusCode::NOT_FOUND);
    }

    /// B-8 + #121: 요청(/asks) 한 곳에서 판단 · 승인을 다룬다 — 판단(질문 · 선택지 · 답 = option 배열 · 질문 index + 선택지 code) · 작성 중 · 답변 검사 ·
    /// 승인 · 거부 · kind에 안 맞는 동작은 409 · 이벤트 대상 ask
    #[tokio::test]
    async fn decision() {
        use crate::ask::{self, ApprovalNew, Choice, DecisionNew, Question};
        use axum::response::IntoResponse;
        let db = mem().await;
        let app = app(db.clone());
        let ts = task_of(&app, &db, true).await;
        let opt = |code: &str| Choice { code: code.into(), label: code.into(), note: None, is_recommended: code == "A", is_selected: false };
        let qn = |t: &str| Question { title: t.into(), body: None, code_snippet: None, reference: None, options: vec![opt("A"), opt("B")], answer_text: None, is_delegate: false, answer_at: None };
        let new = |level| DecisionNew { project_sn: 1, task_sn: Some(ts), run_sn: None, member_sn: 1, level, title: "D".into(), deadline_at: None, questions: vec![qn("Q1"), qn("Q2")] };
        assert_eq!(ask::decision(&db, new(1)).await.err().unwrap().into_response().status(), StatusCode::UNPROCESSABLE_ENTITY);
        ask::decision(&db, new(2)).await.unwrap();

        // 목록: 종류 · 상태 · 프로젝트 · 태스크 범위. 질문 · 선택지를 순서대로 option에 함께 준다
        let v = call(&app, "GET", &format!("/asks?kind=decision&status=pending&task_sn={ts}"), None).await.1;
        assert_eq!((v.as_array().unwrap().len(), v[0]["option"][1]["title"].as_str(), v[0]["option"][0]["options"][1]["code"].as_str(), v[0]["kind"].as_str(), v[0]["action"].is_null()), (1, Some("Q2"), Some("B"), Some("decision"), true));
        assert_eq!(call(&app, "GET", "/asks?project_sn=99", None).await.1.as_array().unwrap().len(), 0);
        assert_eq!(call(&app, "GET", "/asks?kind=approval", None).await.1.as_array().unwrap().len(), 0);
        let ds = v[0]["sn"].as_i64().unwrap();

        // 작성 중: 타이머 멈춤 · 두 번째는 409
        let (st, d) = call(&app, "POST", &format!("/asks/{ds}/writing"), None).await;
        assert_eq!((st, d["status"].as_str(), d["is_timer_pause"].as_i64()), (StatusCode::OK, Some("writing"), Some(1)));
        assert_eq!(call(&app, "POST", &format!("/asks/{ds}/writing"), None).await.0, StatusCode::CONFLICT);

        // 답변 검사: 질문 빠짐 · 다른 질문에 없는 선택지 · 빈 답 · 없는 질문 번호 · 같은 질문 두 번은 422 (상태는 그대로)
        let uri = format!("/asks/{ds}/answer");
        for bad in [json!({"answers": [{"question": 0, "option": "B"}]}),
                    json!({"answers": [{"question": 0, "option": "Z"}, {"question": 1, "delegate": true}]}),
                    json!({"answers": [{"question": 0}, {"question": 1, "delegate": true}]}),
                    json!({"answers": [{"question": 0, "option": "A"}, {"question": 2, "delegate": true}]}),
                    json!({"answers": [{"question": 0, "option": "A"}, {"question": 0, "option": "B"}]})] {
            assert_eq!(call(&app, "POST", &uri, Some(bad)).await.0, StatusCode::UNPROCESSABLE_ENTITY);
        }
        assert_eq!(call(&app, "GET", &format!("/asks/{ds}"), None).await.1["status"], "writing");

        // 답변: answered · 사용자 결정 · 선택 1개 · 맡김 · 이벤트 3개 · 구독자에게 발행
        let mut rx = event::subscribe();
        let (st, d) = call(&app, "POST", &uri, Some(json!({"answers": [{"question": 0, "option": "B", "text": "짧게"}, {"question": 1, "delegate": true}], "review_needed": true}))).await;
        assert_eq!((st, d["status"].as_str(), d["decide_by"].as_str(), d["is_timer_pause"].as_i64(), d["is_review_needed"].as_i64()), (StatusCode::OK, Some("answered"), Some("user"), Some(0), Some(1)));
        let o = &d["option"][0]["options"];
        assert_eq!((o[0]["is_selected"].as_bool(), o[1]["is_selected"].as_bool(), d["option"][0]["answer_text"].as_str(), d["option"][1]["is_delegate"].as_bool(), d["option"][0]["answer_at"].is_string()), (Some(false), Some(true), Some("짧게"), Some(true), true));
        assert_eq!(events(&db, "ask", ds).await.iter().map(|e| e.0.as_str()).collect::<Vec<_>>(), ["DecisionRequested", "DecisionWriting", "DecisionAnswered"]);
        // 버스는 프로세스 전역 — 병렬 테스트의 다른 이벤트는 건너뛴다
        let e = loop { let e = rx.recv().await.unwrap(); if e.event_type == "DecisionAnswered" { break e; } };
        assert_eq!((e.aggregate_sn, e.project_sn), (ds, Some(1)));
        assert_eq!(call(&app, "POST", &uri, Some(json!({"answers": [{"question": 0, "option": "A"}, {"question": 1, "delegate": true}]}))).await.0, StatusCode::CONFLICT);
        assert_eq!(call(&app, "POST", "/asks/99/answer", Some(json!({"answers": []}))).await.0, StatusCode::NOT_FOUND);

        // 승인 요청: 모르는 동작 422 · 승인 · 거부 · 다시 처리 409 · 상태 필터 · 판단에 승인 · 승인에 답변은 409
        let an = |code: &str| ApprovalNew { project_sn: 1, task_sn: Some(ts), run_sn: None, member_sn: 1, rule_title: Some("PR 생성".into()), action_code: code.into(), title: "A".into(), detail: Some("20분 초과".into()), deadline_at: None };
        assert_eq!(ask::approval(&db, an("x")).await.err().unwrap().into_response().status(), StatusCode::UNPROCESSABLE_ENTITY);
        ask::approval(&db, an("pr_merge")).await.unwrap();
        ask::approval(&db, an("run_extend")).await.unwrap();
        let v = call(&app, "GET", "/asks?kind=approval&status=pending&project_sn=1", None).await.1;
        assert_eq!((v.as_array().unwrap().len(), v[1]["action"].as_str(), v[1]["option"]["rule_title"].as_str(), v[1]["option"]["detail"].as_str()), (2, Some("pr_merge"), Some("PR 생성"), Some("20분 초과")));
        let (a1, a2) = (v[1]["sn"].as_i64().unwrap(), v[0]["sn"].as_i64().unwrap());
        assert_eq!(call(&app, "POST", &format!("/asks/{ds}/approve"), None).await.0, StatusCode::CONFLICT);
        assert_eq!(call(&app, "POST", &format!("/asks/{a1}/answer"), Some(json!({"answers": []}))).await.0, StatusCode::CONFLICT);
        assert_eq!(call(&app, "POST", &format!("/asks/{a1}/proceed"), None).await.0, StatusCode::CONFLICT);
        let (st, a) = call(&app, "POST", &format!("/asks/{a1}/approve"), None).await;
        assert_eq!((st, a["status"].as_str(), a["user_sn"].as_i64(), a["decide_at"].is_string()), (StatusCode::OK, Some("approved"), Some(USER), true));
        assert_eq!(call(&app, "POST", &format!("/asks/{a2}/deny"), None).await.1["status"], "denied");
        assert_eq!(call(&app, "POST", &format!("/asks/{a1}/deny"), None).await.0, StatusCode::CONFLICT);
        assert_eq!(call(&app, "GET", "/asks?kind=approval&status=pending", None).await.1.as_array().unwrap().len(), 0);
        assert_eq!(events(&db, "ask", a1).await.iter().map(|e| e.0.as_str()).collect::<Vec<_>>(), ["ApprovalRequested", "ApprovalApproved"]);
    }

    /// B-9: 스킬 소스 · 스킬 · MCP 조회, 허용/차단(이벤트 · 검사 실패는 차단 해제 409), 사용처(멤버 · 템플릿), 차단 · 미검사 스킬은 프로필 연결 422
    #[tokio::test]
    async fn skill() {
        let db = mem().await;
        let app = app(db.clone());
        db.execute_unprepared(
            "INSERT INTO tbl_skill_source (sn, workspace_sn, kind, name, sort) VALUES (1, 1, 'github', 'team', 1), (2, 1, 'builtin', 'base', 0); \
             INSERT INTO tbl_skill (sn, workspace_sn, source_sn, name, scan_status) VALUES (1, 1, 1, 'svelte-ui', 'passed'), (2, 1, 1, 'bad', 'failed'), (3, 1, 2, 'new', 'pending'); \
             INSERT INTO tbl_mcp (sn, workspace_sn, name, install_status) VALUES (1, 1, 'playwright', 'installed'); \
             INSERT INTO tbl_agent_profile (sn, workspace_sn, kind) VALUES (1, 1, 'member'), (2, 1, 'template'); \
             INSERT INTO tbl_team (sn, workspace_sn, name) VALUES (1, 1, 'T'); INSERT INTO tbl_member (team_sn, profile_sn, name, role_name) VALUES (1, 1, '진', 'Dev'); \
             INSERT INTO tbl_template (sn, workspace_sn, name) VALUES (1, 1, 'Frontend'); INSERT INTO tbl_template_revision (template_sn, profile_sn, version, status) VALUES (1, 2, 1, 'live'); \
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
        assert_eq!(events(&db, "skill", 1).await, [("SkillUpdated".into(), 1)]); // 대상 종류 skill

        // 프로필 연결: 통과 스킬만. 차단 · 검사 실패 · 검사 전은 422, 연결은 그대로
        assert_eq!(call(&app, "PUT", "/profiles/1/skills", Some(json!([{"skill_sn": 1, "is_enabled": 1}]))).await.0, StatusCode::OK);
        call(&app, "PATCH", "/skills/1", Some(json!({"is_blocked": 1}))).await;
        for sn in [1, 2, 3] {
            assert_eq!(call(&app, "PUT", "/profiles/2/skills", Some(json!([{"skill_sn": sn, "is_enabled": 1}]))).await.0, StatusCode::UNPROCESSABLE_ENTITY);
        }
        assert_eq!(call(&app, "GET", "/profiles/2", None).await.1["skills"].as_array().unwrap().len(), 0);
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
        use crate::ask::{self, ApprovalNew, Choice, DecisionNew, Question};
        let db = mem().await;
        let app = app(db.clone());
        let ts = task_of(&app, &db, true).await;
        let q = Question { title: "Q".into(), body: None, code_snippet: None, reference: None, options: vec![Choice { code: "A".into(), label: "a".into(), note: None, is_recommended: false, is_selected: false }], answer_text: None, is_delegate: false, answer_at: None };
        ask::decision(&db, DecisionNew { project_sn: 1, task_sn: Some(ts), run_sn: None, member_sn: 1, level: 2, title: "재전송 제한".into(), deadline_at: None, questions: vec![q] }).await.unwrap();
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
        ask::approval(&db, ApprovalNew { project_sn: 1, task_sn: None, run_sn: None, member_sn: 1, rule_title: None, action_code: "pr_merge".into(), title: "A".into(), detail: None, deadline_at: None }).await.unwrap();
        assert_eq!(call(&app, "GET", "/notifications", None).await.1.as_array().unwrap().len(), 2);
        assert_eq!(call(&app, "GET", "/notify/rules", None).await.1[0]["is_enabled"], 0);
        for bad in [json!([rule, rule]), json!([{"event_code": "x", "channel_kind": "app", "is_enabled": 1}]), json!([{"event_code": "pr", "channel_kind": "sms", "is_enabled": 1}])] {
            assert_eq!(call(&app, "PUT", "/notify/rules", Some(bad)).await.0, StatusCode::UNPROCESSABLE_ENTITY);
        }
        assert_eq!(events(&db, "workspace", WORKSPACE).await, [("NotifyRulesUpdated".into(), 1)]);

        // 채널 · 테스트: key_ref 없음, app 항상 준비, off 채널은 준비 안 됨
        db.execute_unprepared("UPDATE tbl_workspace SET channel_json = '[{\"kind\":\"telegram\",\"status\":\"connected\",\"key_ref\":\"orch.tg\"},{\"kind\":\"email\",\"status\":\"off\"}]';").await.unwrap();
        let v = call(&app, "GET", "/workspace", None).await.1["channels"].clone();
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
        db.execute_unprepared("INSERT INTO tbl_log_audit (workspace_sn, actor_type, kind, title) VALUES (1, 'user', 'KEY', 'k'), (1, 'member', 'BLOCK', 'git reset'), (1, 'user', 'BLOCK', 'git stash');").await.unwrap();
        let v = call(&app, "GET", "/audit?kind=BLOCK&limit=1", None).await.1;
        assert_eq!((v.as_array().unwrap().len(), v[0]["title"].as_str()), (1, Some("git stash")));
        assert_eq!(call(&app, "GET", "/audit", None).await.1.as_array().unwrap().len(), 3);
        assert_eq!(call(&app, "GET", "/audit?kind=x", None).await.0, StatusCode::UNPROCESSABLE_ENTITY);

        // 행위자: 알림 actor_type · member_sn은 이벤트 행에서 — 멤버가 만든 판단 요청(1) · Run 실패는 member, Orch 멤버가 만든 판단 요청은 orch
        let v = call(&app, "GET", "/notifications", None).await.1;
        assert_eq!((v[0]["actor_type"].as_str(), v[0]["member_sn"].as_i64(), v[1]["actor_type"].as_str()), (Some("member"), Some(1), Some("member")));
        db.execute_unprepared("INSERT INTO tbl_member (sn, team_sn, profile_sn, name, role_name, is_orch) VALUES (2, 1, 1, 'Orch', 'PM', 1);").await.unwrap();
        let q = Question { title: "Q".into(), body: None, code_snippet: None, reference: None, options: vec![Choice { code: "A".into(), label: "a".into(), note: None, is_recommended: false, is_selected: false }], answer_text: None, is_delegate: false, answer_at: None };
        ask::decision(&db, DecisionNew { project_sn: 1, task_sn: Some(ts), run_sn: None, member_sn: 2, level: 3, title: "배포 시점".into(), deadline_at: None, questions: vec![q] }).await.unwrap();
        let v = call(&app, "GET", "/notifications", None).await.1;
        assert_eq!((v[0]["event_code"].as_str(), v[0]["actor_type"].as_str(), v[0]["member_sn"].as_i64()), (Some("decision_request"), Some("orch"), Some(2)));
        assert_eq!(actor(&db, "DecisionRequested", 3).await, ("orch".into(), Some(2), None));
    }

    /// B-11: Orch 없으면 409 → 대화 열기 · 메시지 저장, 작업 제안 수정 · 진행(이슈 · 태스크 생성 · 배정 · 결과 카드 · 이벤트) · 재진행 409 · 취소,
    /// Orch 제안 처리(proceed · edit · cancel), 실행 중 지시(활동 기록 · 끝난 Run 409)
    #[tokio::test]
    async fn orch() {
        use crate::{ask::{self, Opt, ProposalNew}, orch::{self, Plan, PlanIssue, PlanTask}};
        let db = mem().await;
        let app = app(db.clone());
        let ts = task_of(&app, &db, true).await; // 프로젝트 1 · 팀 1 · 멤버 1(m)
        let v = call(&app, "GET", "/projects/1/conversation", None).await.1;
        assert_eq!((v["member_sn"].is_null(), v["messages"].as_array().unwrap().len()), (true, 0)); // 팀 · Orch 없음
        assert_eq!(call(&app, "POST", "/projects/1/messages", Some(json!({"content": "hi"}))).await.0, StatusCode::CONFLICT); // 팀 · Orch 없음
        db.execute_unprepared("UPDATE tbl_project SET team_sn = 1; INSERT INTO tbl_member (team_sn, profile_sn, name, role_name, is_orch) VALUES (1, 1, 'Orch', 'PM', 1);").await.unwrap();

        // 메시지: 대화를 열고 저장 · 빈 본문 422 · 없는 프로젝트 404
        let (st, m) = call(&app, "POST", "/projects/1/messages", Some(json!({"content": "로그인 개선해줘", "task_sn": ts}))).await;
        assert_eq!((st, m["sender_type"].as_str(), m["kind"].as_str(), m["task_sn"].as_i64()), (StatusCode::CREATED, Some("user"), Some("text"), Some(ts)));
        assert_eq!(call(&app, "POST", "/projects/1/messages", Some(json!({"content": " "}))).await.0, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(call(&app, "POST", "/projects/99/messages", Some(json!({"content": "x"}))).await.0, StatusCode::NOT_FOUND);

        // 작업 제안: Orch가 올림 → 수정 → 진행 = 이슈 1 · 태스크 2(첫째는 멤버 1에게 orch_auto 배정) · 결과 카드
        let plan = |n: usize| Plan { issue: PlanIssue { title: "Authentication 개선".into(), body: None },
            tasks: (0..n).map(|k| PlanTask { title: format!("T{k}"), description: None, member_sn: (k == 0).then_some(1) }).collect() };
        orch::propose(&db, 1, Some("이렇게 나눌게요".into()), plan(1)).await.unwrap();
        let ws = call(&app, "GET", "/projects/1/conversation", None).await.1["messages"][1]["sn"].as_i64().unwrap();
        assert_eq!(call(&app, "PATCH", &format!("/messages/{ws}"), Some(json!({"issue": {"title": ""}, "tasks": []}))).await.0, StatusCode::UNPROCESSABLE_ENTITY);
        let (st, e) = call(&app, "PATCH", &format!("/messages/{ws}"), Some(serde_json::to_value(plan(2)).unwrap())).await;
        assert_eq!((st, e["payload"]["tasks"].as_array().unwrap().len()), (StatusCode::OK, 2));
        let (st, p) = call(&app, "POST", &format!("/messages/{ws}/proceed"), None).await;
        assert_eq!((st, p["issue"]["title"].as_str(), p["tasks"].as_array().unwrap().len(), p["result"]["kind"].as_str()), (StatusCode::OK, Some("Authentication 개선"), 2, Some("command_result")));
        assert_eq!((p["tasks"][0]["member_sn"].as_i64(), p["tasks"][0]["assign_by"].as_str(), p["tasks"][1]["member_sn"].as_i64()), (Some(1), Some("orch_auto"), None));
        assert_eq!(call(&app, "GET", &format!("/issues/{}/tasks", p["issue"]["sn"]), None).await.1.as_array().unwrap().len(), 2);
        let c = call(&app, "GET", "/projects/1/conversation", None).await.1;
        assert_eq!((c["messages"].as_array().unwrap().len(), c["messages"][1]["proposal_status"].as_str()), (3, Some("proceeded")));
        assert_eq!(call(&app, "POST", &format!("/messages/{ws}/proceed"), None).await.0, StatusCode::CONFLICT);
        assert_eq!(call(&app, "POST", &format!("/messages/{ws}/cancel"), None).await.0, StatusCode::CONFLICT);
        assert_eq!(call(&app, "PATCH", &format!("/messages/{}", m["sn"]), Some(serde_json::to_value(plan(1)).unwrap())).await.0, StatusCode::CONFLICT); // 일반 메시지
        // 이벤트 대상 = 메시지 (project 대상에는 남지 않는다) · Orch가 올린 제안은 행위자 orch + Orch 멤버
        let kinds: Vec<String> = events(&db, "message", ws).await.into_iter().map(|e| e.0).collect();
        assert_eq!(kinds, ["MessagePosted", "ProposalEdited", "ProposalProceeded"]);
        assert_eq!(events(&db, "message", m["sn"].as_i64().unwrap()).await, [("MessagePosted".into(), 1)]);
        assert!(events(&db, "project", 1).await.is_empty());
        let a = actor(&db, "MessagePosted", ws).await;
        assert_eq!((a.0.as_str(), a.1, a.2), ("orch", Some(2), None));
        let a = actor(&db, "MessagePosted", m["sn"].as_i64().unwrap()).await;
        assert_eq!((a.0.as_str(), a.1, a.2), ("user", None, Some(1)));
        orch::propose(&db, 1, None, plan(1)).await.unwrap();
        let ws2 = call(&app, "GET", "/projects/1/conversation", None).await.1["messages"][3]["sn"].as_i64().unwrap();
        assert_eq!(call(&app, "POST", &format!("/messages/{ws2}/cancel"), None).await.1["proposal_status"], "cancelled");

        // Orch 제안: 목록 · edit은 선택지 필요 · 처리 후 다시 처리 409 · 모르는 동작 404
        let pn = |kind: &str| ProposalNew { project_sn: 1, issue_sn: None, task_sn: Some(ts), run_sn: None, member_sn: Some(1), kind: kind.into(), level: 1,
            title: "#130 QA를 하린에게 배정".into(), reason: None, options: Some(vec![Opt { label: "Todo로 보내고 대기".into(), kind: None, member_sn: None, task_sn: None }]),
            streak_count: 3, deadline_at: None, event_sn: None, guard_code: None, status: None };
        ask::suggest(&db, pn("assign")).await.unwrap();
        ask::suggest(&db, pn("next_issue")).await.unwrap();
        assert!(ask::suggest(&db, pn("x")).await.is_err());
        let v = call(&app, "GET", "/asks?kind=proposal&project_sn=1&status=proposed", None).await.1;
        assert_eq!((v.as_array().unwrap().len(), v[1]["option"][0]["label"].as_str()), (2, Some("Todo로 보내고 대기")));
        assert_eq!(call(&app, "POST", "/asks/1/edit", Some(json!({}))).await.0, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(call(&app, "POST", "/asks/1/edit", Some(json!({"option": "Todo로 보내고 대기"}))).await.1["status"], "changed");
        assert_eq!(call(&app, "POST", "/asks/2/proceed", None).await.1["status"], "user_done");
        assert_eq!(call(&app, "POST", "/asks/2/cancel", None).await.0, StatusCode::CONFLICT);
        assert_eq!(call(&app, "POST", "/asks/2/zzz", None).await.0, StatusCode::NOT_FOUND);
        assert_eq!(call(&app, "GET", "/asks?kind=proposal&status=proposed", None).await.1.as_array().unwrap().len(), 0);
        assert_eq!(events(&db, "ask", 2).await, [("OrchProposed".into(), 1), ("OrchProposalResolved".into(), 2)]); // 대상 종류 proposal
        assert_eq!(actor(&db, "OrchProposed", 2).await.0, "orch");

        // 실행 중 지시: 활동 기록 + 이벤트, 끝난 Run 409 · 빈 지시 422
        let rs = call(&app, "POST", &format!("/tasks/{ts}/runs"), None).await.1["sn"].as_i64().unwrap();
        let (st, i) = call(&app, "POST", &format!("/runs/{rs}/instruct"), Some(json!({"text": "테스트 먼저 돌려"}))).await;
        assert_eq!((st, i["member_sn"].as_i64(), i["text"].as_str()), (StatusCode::CREATED, Some(1), Some("테스트 먼저 돌려")));
        assert_eq!(events(&db, "run", rs).await.last().unwrap().0, "InstructionSent");
        assert_eq!(call(&app, "POST", &format!("/runs/{rs}/instruct"), Some(json!({"text": ""}))).await.0, StatusCode::UNPROCESSABLE_ENTITY);
        call(&app, "POST", &format!("/runs/{rs}/stop"), None).await;
        assert_eq!(call(&app, "POST", &format!("/runs/{rs}/instruct"), Some(json!({"text": "x"}))).await.0, StatusCode::CONFLICT);
    }

    /// B-12: Run 토큰 합 · 리드 합계 = 자기 + runner 하위 Run(재시도 전 Run 포함 · sub 제외 · 모름은 unknown_count), 하위 Run 목록 · children=0,
    /// 팀 통계 · 팀 한도 요약 · 월 비용(종류별)
    #[tokio::test]
    async fn stat() {
        let db = mem().await;
        let app = app(db.clone());
        let ts = task_of(&app, &db, true).await;
        let lead = call(&app, "POST", &format!("/tasks/{ts}/runs"), None).await.1["sn"].as_i64().unwrap();
        // 하위: 2 runner 실패 → 3 runner 재시도(2의 재시도) · 4 runner 토큰 기록 없음 · 5 sub(runner 합계에서 빠짐)
        db.execute_unprepared(&format!(
            "INSERT INTO tbl_connection (sn, workspace_sn, kind, provider_code, provider_name, name) VALUES (1, 1, 'api_key', 'openai', 'OpenAI', 'work'), (2, 1, 'subscription', 'anthropic', 'Anthropic', 'max'); \
             INSERT INTO tbl_run (sn, project_sn, task_sn, member_sn, num, status, start_by, parent_run_sn, spawn_mode, tier, kind, child_seq, paths, retry_run_sn) VALUES \
               (2, 1, {ts}, 1, 2, 'failed', 'lead', {lead}, 'runner', 'S', 'test', 1, '[{{\"path\":\"src/a\",\"source\":\"brief\",\"at\":null}}]', NULL), \
               (3, 1, {ts}, 1, 3, 'completed', 'retry', {lead}, 'runner', 'S', 'test', 2, NULL, 2), \
               (4, 1, {ts}, 1, 4, 'running', 'lead', {lead}, 'runner', 'M', 'fix', 3, NULL, NULL), \
               (5, 1, {ts}, 1, 5, 'completed', 'lead', {lead}, 'sub', NULL, 'explore', 4, NULL, NULL); \
             INSERT INTO tbl_log_token (run_sn, connection_sn, token_input, token_output, cost_usd_micro, usage_source) VALUES \
               ({lead}, 1, 60, 40, 500, 'provider'), (2, 1, 30, 20, 0, 'estimated'), (3, 2, 20, 10, 0, 'provider'), (5, 2, 900, 100, 0, 'provider'); \
             INSERT INTO tbl_log_token (run_sn, token_input, create_at) VALUES (3, 7, '2000-01-05 10:00:00');"
        )).await.unwrap();

        // DoD: 리드 = 100 + 50 + 37(3의 지난 기록 7 포함) — 4는 모름 · 5는 sub
        let r = call(&app, "GET", &format!("/runs/{lead}"), None).await.1;
        assert_eq!((r["tokens"]["total"].as_i64(), r["tokens"]["sources"][0].as_str()), (Some(100), Some("provider")));
        let t = &r["runner_total"];
        assert_eq!((t["value"].as_i64(), t["run_count"].as_i64(), t["unknown_count"].as_i64(), t["cost_usd_micro"].as_i64()), (Some(187), Some(4), Some(1), Some(500)));

        // 하위 Run: 순번순 · 하위 필드 · 토큰 없으면 null · 하위에는 runner_total 없음
        let v = call(&app, "GET", &format!("/runs/{lead}/children"), None).await.1;
        assert_eq!(v.as_array().unwrap().len(), 4);
        assert_eq!((v[0]["spawn_mode"].as_str(), v[0]["tier"].as_str(), v[0]["paths"][0]["path"].as_str(), v[0]["tokens"]["sources"][0].as_str()), (Some("runner"), Some("S"), Some("src/a"), Some("estimated")));
        assert_eq!((v[1]["retry_run_sn"].as_i64(), v[2]["tokens"].is_null(), v[0]["runner_total"].is_null()), (Some(2), true, true));
        assert_eq!(call(&app, "GET", &format!("/tasks/{ts}/runs"), None).await.1.as_array().unwrap().len(), 5);
        assert_eq!(call(&app, "GET", &format!("/tasks/{ts}/runs?children=0"), None).await.1.as_array().unwrap().len(), 1);
        assert_eq!(call(&app, "GET", "/runs/99/children", None).await.0, StatusCode::NOT_FOUND);

        // 팀 통계: 멤버 1 · 열린 태스크 1 · 진행 리드 Run 1 · 오늘 토큰 1180(sub 포함 · 2000년 기록 7 제외)
        let s = call(&app, "GET", "/teams/1/stats", None).await.1;
        assert_eq!((s["member_count"].as_i64(), s["open_task_count"].as_i64(), s["today_token"].as_i64(), s["done_week_count"].as_i64(), s["avg_cycle_minute"].is_null()), (Some(1), Some(1), Some(1180), Some(0), true));
        assert_eq!((s["members"][0]["name"].as_str(), s["members"][0]["active_run_count"].as_i64()), (Some("m"), Some(1)));
        assert_eq!(call(&app, "GET", "/teams/99/stats", None).await.0, StatusCode::NOT_FOUND);

        // 팀 한도: 프로필 연결(1) + 폴백(2) → 연결별 한도 · 가장 적게 남은 비율
        db.execute_unprepared("UPDATE tbl_agent_profile SET connection_sn = 1 WHERE sn = 1; INSERT INTO tbl_runtime (sn, workspace_sn, code, name) VALUES (1, 1, 'codex', 'Codex'); \
            UPDATE tbl_agent_profile SET fallback_json = '[{\"runtime_sn\":1,\"connection_sn\":2}]' WHERE sn = 1; \
            UPDATE tbl_connection SET quota_json = '[{\"period\":\"5h\",\"unit\":\"percent\",\"used_value\":70,\"remain_percent\":30},{\"period\":\"week\",\"unit\":\"percent\",\"used_value\":88,\"remain_percent\":12}]' WHERE sn = 2;").await.unwrap();
        let v = call(&app, "GET", "/teams/1/quota", None).await.1;
        assert_eq!((v.as_array().unwrap().len(), v[1]["kind"].as_str(), v[1]["min_remain_percent"].as_i64(), v[1]["quotas"].as_array().unwrap().len(), v[0]["min_remain_percent"].is_null()), (2, Some("subscription"), Some(12), 2, true));

        // 월 비용: 이번 달 종류별(구독 · API 키) · 2000-01은 연결 없는 기록(unknown) · 형식 422
        let c = call(&app, "GET", "/workspace/cost", None).await.1;
        assert_eq!((c["items"].as_array().unwrap().len(), c["items"][0]["kind"].as_str(), c["items"][0]["cost_usd_micro"].as_i64(), c["total_usd_micro"].as_i64()), (2, Some("api_key"), Some(500), Some(500)));
        assert_eq!(c["items"][1]["token"].as_i64(), Some(1030)); // 구독: 3(30) + 5(1000)

        // 구독 정액: 구독 · 요금제 연결의 monthly_fee 합이 subscription_fixed로 total에 들어간다 (API 키 정액은 무시 · 그 달 이후 만든 연결 제외)
        let (st, n) = call(&app, "POST", "/connections", Some(json!({"kind": "plan", "provider_code": "cursor", "provider_name": "Cursor", "name": "pro", "monthly_fee_usd_micro": 20_000_000}))).await;
        assert_eq!((st, n["monthly_fee_usd_micro"].as_i64()), (StatusCode::CREATED, Some(20_000_000)));
        let p = call(&app, "PATCH", "/connections/2", Some(json!({"monthly_fee_usd_micro": 100_000_000}))).await.1;
        assert_eq!(p["monthly_fee_usd_micro"].as_i64(), Some(100_000_000));
        call(&app, "PATCH", "/connections/1", Some(json!({"monthly_fee_usd_micro": 7_000_000}))).await; // api_key
        let c = call(&app, "GET", "/workspace/cost", None).await.1;
        assert_eq!((c["items"].as_array().unwrap().len(), c["items"][2]["kind"].as_str(), c["items"][2]["cost_usd_micro"].as_i64(), c["items"][2]["token"].as_i64(), c["total_usd_micro"].as_i64()),
            (3, Some("subscription_fixed"), Some(120_000_000), Some(0), Some(120_000_500)));
        let c = call(&app, "GET", "/workspace/cost?month=2000-01", None).await.1;
        assert_eq!((c["items"].as_array().unwrap().len(), c["items"][0]["kind"].as_str(), c["items"][0]["token"].as_i64()), (1, Some("unknown"), Some(7)));
        assert_eq!(call(&app, "GET", "/workspace/cost?month=2000-13", None).await.0, StatusCode::UNPROCESSABLE_ENTITY);
    }

    /// B-16 도우미: 프로젝트 1(팀 1 · Orch 멤버 2) · 이슈 1 · 태스크 a(멤버 1) · b · c(담당 없음). a의 sn을 돌려준다
    async fn rig(app: &Router, db: &DatabaseConnection) -> i64 {
        let a = task_of(app, db, true).await;
        db.execute_unprepared("UPDATE tbl_project SET team_sn = 1; INSERT INTO tbl_member (sn, team_sn, profile_sn, name, role_name, is_orch) VALUES (2, 1, 1, 'Orch', 'PM', 1);").await.unwrap();
        for t in ["B", "C"] {
            call(app, "POST", "/issues/1/tasks", Some(json!({"title": t}))).await;
        }
        a
    }

    /// 가장 최근 이벤트(kind)
    async fn last(db: &DatabaseConnection, kind: &str) -> crate::entity::tbl_log_event::Model {
        use crate::entity::tbl_log_event as ev;
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
        ev::Entity::find().filter(ev::Column::EventType.eq(kind)).order_by_desc(ev::Column::Sn).one(db).await.unwrap().unwrap()
    }

    /// 팀 1의 Orch 정책(모드 · 타이머 · 레벨 · 가드)을 읽어 고친 뒤 PATCH한다
    async fn set_policy(app: &Router, edit: impl FnOnce(&mut Value)) -> (StatusCode, Value) {
        let t = call(app, "GET", "/teams/1", None).await.1;
        let mut p = json!({"orch_mode": t["orch_mode"], "timer_sec": t["timer_sec"], "is_pause_on_view": t["is_pause_on_view"], "levels": t["levels"], "guards": t["guards"]});
        edit(&mut p);
        call(app, "PATCH", "/teams/1", Some(p)).await
    }

    /// 태스크를 in_progress → done으로 옮기고 그 TaskMoved 이벤트를 돌려준다
    async fn finish(app: &Router, db: &DatabaseConnection, sn: i64) -> crate::entity::tbl_log_event::Model {
        for to in ["in_progress", "done"] {
            call(app, "POST", &format!("/tasks/{sn}/move"), Some(json!({"status": to}))).await;
        }
        last(db, "TaskMoved").await
    }

    /// B-16 DoD: 태스크 done → 다음 태스크가 같은 멤버에게 assign 제안(기한 있음) → 타이머가 지나면 auto_done · 배정(orch_auto) · 모델 호출 0,
    /// 같은 이벤트를 다시 받아도 중복 제안 없음, manual 모드는 기한 없음 · 타이머가 건드리지 않음 · proceed로만 진행
    #[tokio::test]
    async fn orch_rule() {
        use crate::orch_rule as rule;
        let db = mem().await;
        let app = app(db.clone());
        let a = rig(&app, &db).await;
        let calls = || crate::exec::CALLS.with(|c| c.get());
        let ev = finish(&app, &db, a).await;

        // 알림 task_done (사용자 이벤트 → 알림)
        let n = call(&app, "GET", "/notifications", None).await.1;
        assert_eq!((n[0]["event_code"].as_str(), n[0]["ref_type"].as_str(), n[0]["ref_sn"].as_i64(), n[0]["title"].as_str()), (Some("task_done"), Some("task"), Some(a), Some("T")));

        // 기본 정책(auto · 5초): 다음 태스크(b = 2)를 끝낸 멤버(1)에게 assign 제안, 아직 배정 전
        let sn = rule::on(&db, &ev).await.unwrap().unwrap();
        let v = call(&app, "GET", "/asks?kind=proposal&status=proposed", None).await.1;
        assert_eq!((v.as_array().unwrap().len(), v[0]["sn"].as_i64(), v[0]["action"].as_str(), v[0]["task_sn"].as_i64(), v[0]["member_sn"].as_i64()), (1, Some(sn), Some("assign"), Some(2), Some(1)));
        assert_eq!((v[0]["event_sn"].as_i64(), v[0]["deadline_at"].is_string(), v[0]["level"].as_i64()), (Some(ev.sn), true, Some(1)));
        assert!(call(&app, "GET", "/tasks/2", None).await.1["member_sn"].is_null());
        assert_eq!(actor(&db, "OrchProposed", sn).await.0, "orch");
        assert_eq!(rule::on(&db, &ev).await.unwrap(), None); // 같은 제안이 대기 중

        // 타이머: 기한 전 0건 · 기한이 지나면 auto_done + 배정(orch_auto) + 행위자 orch
        assert_eq!(ask::tick(&db).await.unwrap(), 0);
        db.execute_unprepared("UPDATE tbl_ask SET deadline_at = datetime('now', '-1 seconds');").await.unwrap();
        assert_eq!(ask::tick(&db).await.unwrap(), 1);
        let v = call(&app, "GET", "/asks?kind=proposal&status=auto_done", None).await.1;
        assert_eq!((v.as_array().unwrap().len(), v[0]["streak_count"].as_i64()), (1, Some(1)));
        let t = call(&app, "GET", "/tasks/2", None).await.1;
        assert_eq!((t["member_sn"].as_i64(), t["assign_by"].as_str()), (Some(1), Some("orch_auto")));
        assert_eq!(actor(&db, "AgentAssigned", 2).await, ("orch".into(), Some(2), None));
        assert_eq!(events(&db, "ask", sn).await, [("OrchProposed".into(), 1), ("OrchProposalResolved".into(), 2)]);
        assert_eq!(calls(), 0); // 모델 · 실행기 호출 0

        // manual: 기한 없음 · 타이머가 실행하지 않음 · proceed로만 (다음 태스크 c = 3)
        assert_eq!(set_policy(&app, |p| p["orch_mode"] = json!("manual")).await.0, StatusCode::OK);
        let sn = rule::on(&db, &ev).await.unwrap().unwrap();
        let v = call(&app, "GET", "/asks?kind=proposal&status=proposed", None).await.1;
        assert_eq!((v[0]["task_sn"].as_i64(), v[0]["deadline_at"].is_null()), (Some(3), true));
        assert_eq!(ask::tick(&db).await.unwrap(), 0);
        assert!(call(&app, "GET", "/tasks/3", None).await.1["member_sn"].is_null());
        let (st, p) = call(&app, "POST", &format!("/asks/{sn}/proceed"), None).await;
        assert_eq!((st, p["status"].as_str(), p["streak_count"].as_i64()), (StatusCode::OK, Some("user_done"), Some(0)));
        let t = call(&app, "GET", "/tasks/3", None).await.1;
        assert_eq!((t["member_sn"].as_i64(), t["assign_by"].as_str()), (Some(1), Some("orch_auto")));
        assert_eq!(rule::on(&db, &ev).await.unwrap(), None); // 남은 후보 없음
        assert_eq!(calls(), 0);
    }

    /// B-16 가드: auto_streak threshold 2 → 세 번째 제안은 guard_stop(stopped · guard_code · trigger_at) + 알림 guard_stop(actor orch),
    /// 멈춘 뒤에는 기한 없이 대기 · 사용자가 처리하면 재개, on_trigger = to_manual이면 정책 mode가 manual
    #[tokio::test]
    async fn orch_guard() {
        use crate::orch_rule as rule;
        let db = mem().await;
        let app = app(db.clone());
        let a = rig(&app, &db).await;
        call(&app, "POST", "/issues/1/tasks", Some(json!({"title": "D"}))).await; // d = 4
        let ev = finish(&app, &db, a).await;
        let on = |db: DatabaseConnection, ev: crate::entity::tbl_log_event::Model| async move { rule::on(&db, &ev).await.unwrap() };
        assert_eq!(set_policy(&app, |p| { p["orch_mode"] = json!("full_auto"); p["guards"][0]["threshold"] = json!(2); }).await.0, StatusCode::OK);

        // full_auto: 바로 실행 → auto_done 2번 (streak 1 · 2)
        for (k, streak) in [(2, 1), (3, 2)] {
            assert!(on(db.clone(), ev.clone()).await.is_some());
            let t = call(&app, "GET", &format!("/tasks/{k}"), None).await.1;
            assert_eq!(t["assign_by"].as_str(), Some("orch_auto"));
            assert_eq!(call(&app, "GET", "/asks?kind=proposal&status=auto_done", None).await.1[0]["streak_count"].as_i64(), Some(streak));
        }
        // 세 번째: 제안 대신 guard_stop. d(4)는 배정되지 않는다
        let sn = on(db.clone(), ev.clone()).await.unwrap();
        let v = call(&app, "GET", "/asks?kind=proposal&status=stopped", None).await.1;
        assert_eq!((v.as_array().unwrap().len(), v[0]["sn"].as_i64(), v[0]["action"].as_str(), v[0]["guard_code"].as_str(), v[0]["task_sn"].as_i64()), (1, Some(sn), Some("guard_stop"), Some("auto_streak"), Some(a)));
        assert!(call(&app, "GET", "/tasks/4", None).await.1["member_sn"].is_null());
        let n = call(&app, "GET", "/notifications", None).await.1;
        assert_eq!((n[0]["event_code"].as_str(), n[0]["actor_type"].as_str(), n[0]["member_sn"].as_i64(), n[0]["ref_type"].as_str(), n[0]["ref_sn"].as_i64(), n[0]["is_action"].as_i64()),
            (Some("guard_stop"), Some("orch"), Some(2), Some("ask"), Some(sn), Some(1)));
        assert!(call(&app, "GET", "/teams/1", None).await.1["guards"][0]["trigger_at"].is_string());

        // 멈춘 뒤: 제안은 만들되 기한 없이 대기 (full_auto여도 실행하지 않음) → 사용자가 처리하면 재개
        let sn = on(db.clone(), ev.clone()).await.unwrap();
        let v = call(&app, "GET", "/asks?kind=proposal&status=proposed", None).await.1;
        assert_eq!((v[0]["sn"].as_i64(), v[0]["task_sn"].as_i64(), v[0]["deadline_at"].is_null()), (Some(sn), Some(4), true));
        assert_eq!(call(&app, "POST", &format!("/asks/{sn}/proceed"), None).await.1["status"], "user_done");
        assert_eq!(call(&app, "GET", "/tasks/4", None).await.1["member_sn"].as_i64(), Some(1));

        // to_manual: 걸리면 정책 mode가 manual로 바뀐다 (이벤트 OrchPolicyUpdated)
        call(&app, "POST", "/issues/1/tasks", Some(json!({"title": "E"}))).await;
        call(&app, "POST", "/issues/1/tasks", Some(json!({"title": "F"}))).await;
        assert_eq!(set_policy(&app, |p| { p["guards"][0]["threshold"] = json!(1); p["guards"][0]["on_trigger"] = json!("to_manual"); }).await.0, StatusCode::OK);
        assert!(on(db.clone(), ev.clone()).await.is_some()); // e 자동 배정
        let sn = on(db.clone(), ev.clone()).await.unwrap(); // 걸림
        assert_eq!(call(&app, "GET", "/asks?kind=proposal&status=stopped", None).await.1[0]["sn"].as_i64(), Some(sn));
        assert_eq!(call(&app, "GET", "/teams/1", None).await.1["orch_mode"], "manual");
        assert_eq!(events(&db, "team", 1).await.last().unwrap().0, "OrchPolicyUpdated");
    }

    /// B-16 정책: 기본값 응답(저장 전) → 저장 → 유지 · 프로젝트별 정책 분리 · L4 block 고정 등 422 · 팀 대상 이벤트
    #[tokio::test]
    async fn orch_policy() {
        let db = mem().await;
        let app = app(db.clone());
        rig(&app, &db).await;
        let t = call(&app, "GET", "/teams/1", None).await.1;
        assert_eq!((t["orch_mode"].as_str(), t["timer_sec"].as_i64(), t["is_pause_on_view"].as_i64(), t["levels"].as_array().unwrap().len(), t["guards"].as_array().unwrap().len()), (Some("auto"), Some(5), Some(1), 5, 4));
        assert_eq!((t["levels"][1]["handle"].as_str(), t["levels"][2]["wait_min"].as_i64(), t["levels"][2]["no_reply"].as_str(), t["levels"][4]["is_locked"].as_i64()), (Some("timer"), Some(10), Some("orch_decide"), Some(1)));
        assert_eq!(scalar(&db, "SELECT COUNT(*) FROM tbl_team WHERE level_json IS NOT NULL OR guard_json IS NOT NULL").await, 0); // 기본값은 응답으로만 · 저장은 PATCH 때
        let p = call(&app, "GET", "/projects/1", None).await.1;
        assert_eq!((p["orch_mode"].is_null(), p["levels"].is_null(), p["guards"].is_null()), (true, true, true)); // 프로젝트 값 없음 = 팀 값

        // 저장 → 다시 읽기 유지 (레벨 · 가드 전체 교체)
        let (st, v) = set_policy(&app, |p| { p["orch_mode"] = json!("full_auto"); p["timer_sec"] = json!(10); p["levels"][1]["handle"] = json!("auto"); p["guards"].as_array_mut().unwrap().remove(1); }).await;
        assert_eq!((st, v["orch_mode"].as_str()), (StatusCode::OK, Some("full_auto")));
        let v = call(&app, "GET", "/teams/1", None).await.1;
        assert_eq!((v["orch_mode"].as_str(), v["timer_sec"].as_i64(), v["levels"][1]["handle"].as_str(), v["guards"].as_array().unwrap().len(), v["guards"][1]["code"].as_str()), (Some("full_auto"), Some(10), Some("auto"), 3, Some("same_failure")));
        assert_eq!(scalar(&db, "SELECT COUNT(*) FROM tbl_team WHERE level_json IS NOT NULL AND guard_json IS NOT NULL").await, 1);

        // 프로젝트별: 따로 정한 값만 팀 값을 덮는다 · 팀은 그대로 · 효과는 policy::load가 합친다
        let (st, p) = call(&app, "PATCH", "/projects/1", Some(json!({"orch_mode": "manual", "timer_sec": 30}))).await;
        assert_eq!((st, p["orch_mode"].as_str(), p["timer_sec"].as_i64(), p["levels"].is_null()), (StatusCode::OK, Some("manual"), Some(30), true));
        assert_eq!(call(&app, "GET", "/teams/1", None).await.1["orch_mode"], "full_auto");
        let pol = crate::policy::load(&db, 1).await.unwrap().unwrap();
        assert_eq!((pol.mode.as_str(), pol.timer_sec, pol.levels[1].handle.as_str(), pol.guards.len()), ("manual", 30, "auto", 3));
        let mut lv = v["levels"].clone();
        lv[2]["handle"] = json!("timer");
        let p = call(&app, "PATCH", "/projects/1", Some(json!({"levels": lv, "guards": []}))).await.1;
        assert_eq!((p["levels"][2]["handle"].as_str(), p["guards"].as_array().unwrap().len()), (Some("timer"), 0));
        let pol = crate::policy::load(&db, 1).await.unwrap().unwrap();
        assert_eq!((pol.levels[2].handle.as_str(), pol.guards.len()), ("timer", 0));
        assert_eq!(call(&app, "GET", "/teams/1", None).await.1["levels"][2]["handle"], "wait"); // 팀 값은 그대로

        // 422: L4 handle 바꿈 · 모르는 mode · 레벨 모자람 · 가드 중복 · 모르는 가드 코드 · timer 0 (프로젝트도 같은 검사) / 404: 없는 팀 · 프로젝트
        for edit in [
            (|p: &mut Value| p["levels"][4]["handle"] = json!("wait")) as fn(&mut Value),
            |p| p["orch_mode"] = json!("x"),
            |p| { p["levels"].as_array_mut().unwrap().pop(); },
            |p| { let g = p["guards"][0].clone(); p["guards"].as_array_mut().unwrap().push(g); },
            |p| p["guards"][0]["code"] = json!("nope"),
            |p| p["timer_sec"] = json!(0),
        ] {
            assert_eq!(set_policy(&app, edit).await.0, StatusCode::UNPROCESSABLE_ENTITY);
        }
        assert_eq!(call(&app, "PATCH", "/projects/1", Some(json!({"orch_mode": "x"}))).await.0, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(call(&app, "PATCH", "/projects/1", Some(json!({"timer_sec": 0}))).await.0, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(call(&app, "PATCH", "/teams/99", Some(json!({"orch_mode": "manual"}))).await.0, StatusCode::NOT_FOUND);
        assert_eq!(events(&db, "team", 1).await, [("TeamUpdated".into(), 1)]);
    }

    /// 테이블 행 수
    async fn db_count(db: &DatabaseConnection, table: &str) -> i64 {
        db.query_one_raw(Statement::from_string(DbBackend::Sqlite, format!("SELECT COUNT(*) FROM {table}"))).await.unwrap().unwrap().try_get_by_index(0).unwrap()
    }

    /// B-16 제안 처리: 선택지 edit(kind가 있으면 그 동작 실행) · hold(기한 연장) · dismiss · cancel · 자동 실행 실패는 dismissed + 사유
    #[tokio::test]
    async fn orch_pick() {
        use crate::orch_rule as rule;
        let db = mem().await;
        let app = app(db.clone());
        let a = rig(&app, &db).await;
        db.execute_unprepared("INSERT INTO tbl_member (sn, team_sn, profile_sn, name, role_name) VALUES (3, 1, 1, 'm3', 'Dev');").await.unwrap();
        let ev = finish(&app, &db, a).await;

        // 같은 역할의 노는 멤버가 선택지로 붙는다
        let sn = rule::on(&db, &ev).await.unwrap().unwrap();
        let v = call(&app, "GET", "/asks?kind=proposal&project_sn=1&status=proposed", None).await.1;
        assert_eq!((v[0]["option"][0]["label"].as_str(), v[0]["option"][0]["kind"].as_str(), v[0]["option"][0]["member_sn"].as_i64(), v[0]["option"][0]["task_sn"].as_i64()), (Some("m3"), Some("assign"), Some(3), Some(2)));

        // hold: 기한이 뒤로 밀린다 (본문 없이도 · 정책 timer_sec) · 모르는 선택지 422(제안은 그대로)
        let before = v[0]["deadline_at"].as_str().unwrap().to_owned();
        let (st, h) = call(&app, "POST", &format!("/asks/{sn}/hold"), Some(json!({"sec": 60}))).await;
        assert_eq!((st, h["status"].as_str(), h["deadline_at"].as_str().unwrap() > before.as_str()), (StatusCode::OK, Some("proposed"), true));
        assert_eq!(call(&app, "POST", &format!("/asks/{sn}/hold"), None).await.0, StatusCode::OK);
        assert_eq!(call(&app, "POST", &format!("/asks/{sn}/hold"), Some(json!({"sec": 0}))).await.0, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(call(&app, "POST", &format!("/asks/{sn}/edit"), Some(json!({"option": "zzz"}))).await.0, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(call(&app, "GET", "/asks?kind=proposal&status=proposed", None).await.1.as_array().unwrap().len(), 1);

        // edit(m3) → changed + 그 멤버에게 배정
        let (st, p) = call(&app, "POST", &format!("/asks/{sn}/edit"), Some(json!({"option": "m3"}))).await;
        assert_eq!((st, p["status"].as_str()), (StatusCode::OK, Some("changed")));
        let t = call(&app, "GET", "/tasks/2", None).await.1;
        assert_eq!((t["member_sn"].as_i64(), t["assign_by"].as_str()), (Some(3), Some("orch_auto")));
        assert_eq!(call(&app, "POST", &format!("/asks/{sn}/hold"), None).await.0, StatusCode::CONFLICT); // 이미 처리됨

        // dismiss · cancel (다음 태스크 c = 3 제안을 두 번)
        let s1 = rule::on(&db, &ev).await.unwrap().unwrap();
        assert_eq!(call(&app, "POST", &format!("/asks/{s1}/dismiss"), None).await.1["status"], "dismissed");
        let s2 = rule::on(&db, &ev).await.unwrap().unwrap();
        assert_ne!(s1, s2);
        assert_eq!(call(&app, "POST", &format!("/asks/{s2}/cancel"), None).await.1["status"], "stopped");
        assert!(call(&app, "GET", "/tasks/3", None).await.1["member_sn"].is_null());

        // 자동 실행 실패(그 사이 c가 다른 멤버에게 배정됨) → dismissed + 사유, 배정은 그대로 · 사용자 proceed는 409 + proposed 유지
        let s3 = rule::on(&db, &ev).await.unwrap().unwrap();
        db.execute_unprepared("UPDATE tbl_task SET member_sn = 3 WHERE sn = 3; UPDATE tbl_ask SET deadline_at = datetime('now', '-1 seconds');").await.unwrap();
        assert_eq!(ask::tick(&db).await.unwrap(), 1);
        let v = call(&app, "GET", "/asks?kind=proposal&status=dismissed", None).await.1;
        assert_eq!((v[0]["sn"].as_i64(), v[0]["reason"].as_str().unwrap().contains("failed")), (Some(s3), true));
        assert_eq!(call(&app, "GET", "/tasks/3", None).await.1["member_sn"].as_i64(), Some(3));
        assert_eq!(rule::on(&db, &ev).await.unwrap(), None); // 후보 없음
    }

    /// B-16 Run 실패: auto_retry_max(기본 1) 안이면 retry 제안(실행하면 새 Run) · 넘으면 제안 없음 · 연결 오류면 fallback 제안(실행하면 run.connection_sn 교체 + 알림 fallback_used)
    #[tokio::test]
    async fn orch_fail() {
        use crate::{entity::tbl_run as r, orch_rule as rule};
        use sea_orm::EntityTrait;
        let db = mem().await;
        let app = app(db.clone());
        let a = rig(&app, &db).await;
        let fail = |rs: i64| { let db = db.clone(); async move {
            run::run_to(&db, rs, "starting").await.unwrap();
            run::run_to(&db, rs, "failed").await.unwrap();
            last(&db, "RunFailed").await
        }};

        // retry: 첫 실패 → 제안 → 실행(새 Run · retry_run_sn) → 두 번째 실패 → 제안 없음
        let rs = call(&app, "POST", &format!("/tasks/{a}/runs"), None).await.1["sn"].as_i64().unwrap();
        let sn = rule::on(&db, &fail(rs).await).await.unwrap().unwrap();
        let v = call(&app, "GET", "/asks?kind=proposal&status=proposed", None).await.1;
        assert_eq!((v[0]["action"].as_str(), v[0]["run_sn"].as_i64(), v[0]["task_sn"].as_i64(), v[0]["member_sn"].as_i64()), (Some("retry"), Some(rs), Some(a), Some(1)));
        assert_eq!(call(&app, "POST", &format!("/asks/{sn}/proceed"), None).await.1["status"], "user_done");
        let runs = call(&app, "GET", &format!("/tasks/{a}/runs"), None).await.1;
        assert_eq!((runs.as_array().unwrap().len(), runs[1]["retry_run_sn"].as_i64(), runs[1]["status"].as_str()), (2, Some(rs), Some("queued")));
        let rs2 = runs[1]["sn"].as_i64().unwrap();
        assert_eq!(rule::on(&db, &fail(rs2).await).await.unwrap(), None);

        // fallback: 연결 1(error) → 체인의 연결 2. 제안 → 실행 → 연결 교체 + FallbackUsed + 알림
        db.execute_unprepared(
            "INSERT INTO tbl_connection (sn, workspace_sn, kind, provider_code, provider_name, name, status) VALUES (1, 1, 'api_key', 'openai', 'OpenAI', 'bad', 'error'), (2, 1, 'api_key', 'openai', 'OpenAI', 'good', 'connected'); \
             INSERT INTO tbl_runtime (sn, workspace_sn, code, name) VALUES (1, 1, 'codex', 'Codex'); \
             UPDATE tbl_agent_profile SET fallback_json = '[{\"runtime_sn\":1,\"connection_sn\":1},{\"runtime_sn\":1,\"connection_sn\":2}]' WHERE sn = 1;",
        ).await.unwrap();
        let b = 2;
        call(&app, "POST", &format!("/tasks/{b}/assign"), Some(json!({"member_sn": 1}))).await;
        let rs3 = call(&app, "POST", &format!("/tasks/{b}/runs"), None).await.1["sn"].as_i64().unwrap();
        db.execute_unprepared(&format!("UPDATE tbl_run SET connection_sn = 1 WHERE sn = {rs3};")).await.unwrap();
        let sn = rule::on(&db, &fail(rs3).await).await.unwrap().unwrap();
        let v = call(&app, "GET", "/asks?kind=proposal&status=proposed", None).await.1;
        assert_eq!((v[0]["action"].as_str(), v[0]["title"].as_str().unwrap().contains("good")), (Some("fallback"), true));
        assert_eq!(call(&app, "POST", &format!("/asks/{sn}/proceed"), None).await.1["status"], "user_done");
        assert_eq!(r::Entity::find_by_id(rs3).one(&db).await.unwrap().unwrap().connection_sn, Some(2));
        assert_eq!(events(&db, "run", rs3).await.last().unwrap().0, "FallbackUsed");
        let n = call(&app, "GET", "/notifications", None).await.1;
        assert_eq!((n[0]["event_code"].as_str(), n[0]["title"].as_str(), n[0]["ref_type"].as_str(), n[0]["ref_sn"].as_i64()), (Some("fallback_used"), Some("good"), Some("run"), Some(rs3)));
    }

    /// B-16 레벨 처리: L1 handle이 wait면 제안 대신 판단 요청(L2 · Orch가 묻는다 · 알림 actor orch)
    #[tokio::test]
    async fn orch_ask() {
        use crate::orch_rule as rule;
        let db = mem().await;
        let app = app(db.clone());
        let a = rig(&app, &db).await;
        let ev = finish(&app, &db, a).await;
        assert_eq!(set_policy(&app, |p| p["levels"][1]["handle"] = json!("wait")).await.0, StatusCode::OK);
        assert_eq!(rule::on(&db, &ev).await.unwrap(), None);
        assert_eq!(call(&app, "GET", "/asks?kind=proposal", None).await.1.as_array().unwrap().len(), 0);
        let d = call(&app, "GET", "/asks?kind=decision", None).await.1;
        assert_eq!((d.as_array().unwrap().len(), d[0]["level"].as_i64(), d[0]["member_sn"].as_i64(), d[0]["deadline_at"].is_null()), (1, Some(2), Some(2), true));
        let n = call(&app, "GET", "/notifications", None).await.1;
        assert_eq!((n[0]["event_code"].as_str(), n[0]["actor_type"].as_str()), (Some("decision_request"), Some("orch")));
    }

    /// B-16 대기열: NOW = 진행 중 Run의 태스크 · NEXT = 담당 todo (queue_sort → priority → num) · 의존 대기 표시 · 없는 멤버 404
    #[tokio::test]
    async fn orch_queue() {
        let db = mem().await;
        let app = app(db.clone());
        let a = rig(&app, &db).await; // a(1) · b(2) · c(3)
        for sn in [2, 3] {
            call(&app, "POST", &format!("/tasks/{sn}/assign"), Some(json!({"member_sn": 1}))).await;
        }
        let q = call(&app, "GET", "/members/1/queue", None).await.1;
        assert_eq!((q["now"].is_null(), q["next"].as_array().unwrap().len()), (true, 3));
        call(&app, "PATCH", "/tasks/3", Some(json!({"queue_sort": 1}))).await;
        call(&app, "PATCH", "/tasks/2", Some(json!({"priority": 0}))).await;
        call(&app, "POST", &format!("/tasks/{a}/deps"), Some(json!({"depend_task_sn": 3}))).await; // a는 c를 기다린다
        let q = call(&app, "GET", "/members/1/queue", None).await.1;
        let order: Vec<i64> = q["next"].as_array().unwrap().iter().map(|x| x["task_sn"].as_i64().unwrap()).collect();
        assert_eq!(order, [3, 2, a]); // c(queue_sort) → b(P0) → a(P2)
        assert_eq!((q["next"][0]["title"].as_str(), q["next"][1]["priority"].as_i64(), q["next"][2]["waiting"].as_bool(), q["next"][0]["waiting"].as_bool()), (Some("C"), Some(0), Some(true), Some(false)));
        assert!(q["next"][0]["num"].is_i64());

        // Run을 시작하면 NOW로 올라가고 NEXT에서 빠진다
        call(&app, "POST", &format!("/tasks/{a}/runs"), None).await;
        let q = call(&app, "GET", "/members/1/queue", None).await.1;
        assert_eq!((q["now"]["task_sn"].as_i64(), q["now"]["waiting"].as_bool(), q["next"].as_array().unwrap().len()), (Some(a), Some(true), 2));
        assert_eq!(call(&app, "GET", "/members/99/queue", None).await.0, StatusCode::NOT_FOUND);
    }

    /// 값 하나를 돌려주는 SQL (테스트용)
    async fn scalar(db: &DatabaseConnection, sql: &str) -> i64 {
        db.query_one_raw(Statement::from_string(DbBackend::Sqlite, sql.to_owned())).await.unwrap().unwrap().try_get_by_index::<Option<i64>>(0).unwrap().unwrap_or(-1)
    }

    /// B-17 도우미: 가짜 Claude CLI(받은 프롬프트를 prompt.txt · 인자를 args.txt에 남기고 호출 횟수에 따라 usage를 다르게 낸다)와 DB.
    /// 멤버 1(프로필 1 = 실행기 1 · 연결 1) · 프로젝트 1(repo = dir) · 태스크 1 Login(#2 · 완료 조건 2 · 라벨 auth · 의존 → 태스크 2 Logout #3) ·
    /// 프리셋 3개(role · protocol · rule 순으로 연결 → 조립은 protocol → rule → role) · 프로필 파일 1 · 저장소 CLAUDE.md
    async fn ctx_rig(name: &str) -> (DatabaseConnection, std::path::PathBuf) {
        use std::os::unix::fs::PermissionsExt;
        let dir = std::env::temp_dir().join(format!("orch-ctx-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("CLAUDE.md"), "REPO-RULE").unwrap();
        let bin = dir.join("claude");
        std::fs::write(&bin, format!(r#"#!/bin/sh
D={0}
cat > $D/prompt.txt
echo "$@" > $D/args.txt
n=$(cat $D/n 2>/dev/null || echo 0); echo $((n+1)) > $D/n
if [ "$n" = 0 ]; then CR=0; else CR=300; fi
echo '{{"type":"system","subtype":"init","session_id":"s9"}}'
printf '{{"type":"result","is_error":false,"result":"all done","usage":{{"input_tokens":100,"output_tokens":3,"cache_read_input_tokens":%s,"cache_creation_input_tokens":0}}}}\n' "$CR"
"#, dir.display())).unwrap();
        std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755)).unwrap();
        let db = mem().await;
        db.execute_unprepared(&format!("
            INSERT INTO tbl_team (sn, workspace_sn, name) VALUES (1, 1, 'T');
            INSERT INTO tbl_runtime (sn, workspace_sn, code, name, bin_path) VALUES (1, 1, 'claude_code', 'Claude Code', '{1}');
            INSERT INTO tbl_connection (sn, workspace_sn, kind, provider_code, provider_name, name) VALUES (1, 1, 'subscription', 'anthropic', 'Anthropic', 'c');
            INSERT INTO tbl_agent_profile (sn, workspace_sn, kind, runtime_sn, connection_sn) VALUES (1, 1, 'member', 1, 1);
            INSERT INTO tbl_member (sn, team_sn, profile_sn, name, role_name) VALUES (1, 1, 1, 'm', 'dev');
            INSERT INTO tbl_project (sn, workspace_sn, team_sn, name, repo_path) VALUES (1, 1, 1, 'p', '{0}');
            INSERT INTO tbl_issue (sn, project_sn, num, title) VALUES (1, 1, 1, 'I');
            INSERT INTO tbl_task (sn, project_sn, issue_sn, num, title, description, member_sn) VALUES (1, 1, 1, 2, 'Login', 'desc a', 1), (2, 1, 1, 3, 'Logout', NULL, 1);
            INSERT INTO tbl_task_criterion (task_sn, content, is_done, sort) VALUES (1, 'works', 0, 1), (1, 'errors', 1, 2);
            UPDATE tbl_task SET label_json = '[\"auth\"]' WHERE sn = 1;
            INSERT INTO tbl_map_task_dependency (task_sn, depend_task_sn) VALUES (1, 2);
            UPDATE tbl_agent_profile SET path_json = '[{{\"kind\":\"include\",\"pattern\":\"src/**\"}},{{\"kind\":\"exclude\",\"pattern\":\"**/.env*\"}}]' WHERE sn = 1;
            INSERT INTO tbl_profile_file (profile_sn, path, content, sort) VALUES (1, 'rules/a11y.md', 'A11Y-FILE', 1);
            INSERT INTO tbl_instruction_preset (sn, workspace_sn, kind, preset_key, name, limit_tok, content, token_count) VALUES (1, 1, 'protocol', 'p', 'P', 500, 'P-PROTO v1', 5), (2, 1, 'rule', 'r', 'R', 500, '- keep small', 0), (3, 1, 'role', 'd', 'D', 500, '# Role: Dev', 5);
            INSERT INTO tbl_map_profile_preset (profile_sn, preset_sn, sort) VALUES (1, 3, 0), (1, 1, 1), (1, 2, 2);
            INSERT INTO tbl_run (sn, project_sn, task_sn, member_sn, num, status) VALUES (1, 1, 1, 1, 1, 'queued'), (2, 1, 2, 1, 2, 'queued');",
            dir.display(), bin.display())).await.unwrap();
        (db, dir)
    }

    /// 조립 결과를 manifest(세션 sn)로 저장한다 (runner가 하는 것과 같다)
    async fn ctx_save(db: &DatabaseConnection, run: i64, session: Option<i64>, b: &crate::context::Built) -> i64 {
        use crate::entity::tbl_context_manifest as cm;
        use sea_orm::ActiveModelTrait;
        let tx = db.begin().await.unwrap();
        let m = cm::ActiveModel { run_sn: sea_orm::Set(run), session_sn: sea_orm::Set(session), budget_token: sea_orm::Set(40_000), ..Default::default() }.insert(&tx).await.unwrap();
        crate::context::record(&tx, b, m.sn).await.unwrap();
        tx.commit().await.unwrap();
        m.sn
    }

    /// (kind, ref_label) 목록
    fn labels(b: &crate::context::Built) -> Vec<(&str, &str)> {
        b.sources.iter().map(|s| (s.kind.as_str(), s.ref_label.as_str())).collect()
    }

    /// B-17 DoD 1: 조립 순서(protocol → rule → role → repo_rule → 프로필 파일 → @TASK) · 같은 입력 = 같은 바이트 · 접두는 태스크가 달라도 같다 ·
    /// 같은 세션 두 번째 호출은 반복 항목을 프롬프트에서 빼고 새 세션이면 보내되 repeat 표시 · new_run 프로필은 빼지 않는다 · 토큰 0인 프리셋 버전은 채워 저장
    #[tokio::test]
    async fn context_order() {
        use crate::context;
        let (db, _dir) = ctx_rig("order").await;
        let b = context::assemble(&db, 1, None).await.unwrap();
        assert_eq!(labels(&b), [("preset", "protocol/p"), ("preset", "rule/r"), ("preset", "role/d"), ("repo_rule", "CLAUDE.md"), ("instruction", "rules/a11y.md"), ("task", "@TASK #2")]);
        assert!(b.sources.iter().all(|s| s.content_hash.len() == 16 && s.is_repeat == 0));
        assert_eq!((b.sources[0].ref_sn, b.sources[0].ref_version, b.sources[5].ref_sn), (Some(1), Some(1), Some(1)));
        assert_eq!(b.prompt, "P-PROTO v1\n\n- keep small\n\n# Role: Dev\n\nREPO-RULE\n\nA11Y-FILE\n\n@TASK v1\nid: #2  title: Login\ndesc: desc a\nac 1 todo: works\nac 2 done: errors\ndep #3 todo: Logout\nlabels: auth\npaths: src/**  !**/.env*\n\n");
        assert_eq!(b.estimate, b.sources.iter().map(|s| s.token_count).sum::<i64>());

        // 같은 입력 → 같은 바이트 · 해시 / 다른 태스크도 @TASK 앞 접두는 같다
        let again = context::assemble(&db, 1, None).await.unwrap();
        assert_eq!((again.prompt.as_str(), again.sources.iter().map(|s| s.content_hash.clone()).collect::<Vec<_>>()), (b.prompt.as_str(), b.sources.iter().map(|s| s.content_hash.clone()).collect::<Vec<_>>()));
        let other = context::assemble(&db, 2, None).await.unwrap();
        let cut = b.prompt.find("@TASK v1").unwrap();
        assert_eq!(&other.prompt[..cut], &b.prompt[..cut]);
        assert_eq!((other.sources[..5].iter().map(|s| &s.content_hash).collect::<Vec<_>>(), other.sources[5].content_hash == b.sources[5].content_hash),
            (b.sources[..5].iter().map(|s| &s.content_hash).collect::<Vec<_>>(), false));

        // 저장: 출처 6건 · 토큰 0이던 프리셋 버전(rule)만 추정값으로 채워진다
        db.execute_unprepared("INSERT INTO tbl_session (sn, run_sn, member_sn, num) VALUES (1, 1, 1, 1), (2, 1, 1, 2);").await.unwrap();
        ctx_save(&db, 1, Some(1), &b).await;
        assert_eq!(scalar(&db, "SELECT json_array_length(source_json) FROM tbl_context_manifest WHERE sn = 1").await, 6);
        assert_eq!((scalar(&db, "SELECT token_count FROM tbl_instruction_preset WHERE sn = 2").await > 0, scalar(&db, "SELECT token_count FROM tbl_instruction_preset WHERE sn = 3").await), (true, 5));

        // 같은 세션 두 번째 호출: 전부 반복 · 프롬프트에서 빠진다 (토큰 0) → 태스크 설명이 바뀌면 태스크 블록만 보낸다
        let two = context::assemble(&db, 1, Some(1)).await.unwrap();
        assert_eq!((two.sources.iter().all(|s| s.is_repeat == 1), two.prompt.as_str(), two.estimate), (true, "", 0));
        db.execute_unprepared("UPDATE tbl_task SET description = 'desc b' WHERE sn = 1;").await.unwrap();
        let two = context::assemble(&db, 1, Some(1)).await.unwrap();
        assert_eq!(two.sources.iter().map(|s| s.is_repeat).collect::<Vec<_>>(), [1, 1, 1, 1, 1, 0]);
        assert!(two.prompt.starts_with("@TASK v1") && two.prompt.contains("desc: desc b") && two.sources[..5].iter().all(|s| s.token_count == 0));
        assert_eq!(two.estimate, two.sources[5].token_count);
        // 새 세션(session 없음 · 다른 세션): 모두 보내되 repeat 표시
        let fresh = context::assemble(&db, 1, Some(2)).await.unwrap();
        assert_eq!((fresh.sources.iter().map(|s| s.is_repeat).collect::<Vec<_>>(), fresh.prompt.starts_with("P-PROTO v1")), ([1, 1, 1, 1, 1, 0].to_vec(), true));
        // new_run 프로필은 같은 세션이어도 빼지 않는다
        db.execute_unprepared("UPDATE tbl_agent_profile SET session_mode = 'new_run';").await.unwrap();
        let nr = context::assemble(&db, 1, Some(1)).await.unwrap();
        assert!(nr.prompt.starts_with("P-PROTO v1") && nr.sources[0].is_repeat == 1 && nr.sources[0].token_count > 0);
        // repo_rule_mode = ignore → 저장소 규칙을 넣지 않는다
        db.execute_unprepared("UPDATE tbl_agent_profile SET repo_rule_mode = 'ignore';").await.unwrap();
        assert!(!labels(&context::assemble(&db, 1, None).await.unwrap()).contains(&("repo_rule", "CLAUDE.md")));
    }

    /// B-17 DoD 2: 상한 초과 → 422 context_over(상위 출처 3개) · Run 시작 · 다시 시도 모두 Run · 세션 · manifest가 생기지 않는다 ·
    /// 견적은 200 + over = true · 하위 Run도 spawn이 만들지 않고 리드에게 돌려준다
    #[tokio::test]
    async fn context_over() {
        use crate::{rule::Stop, runner::{self, Spawn}};
        let (db, dir) = ctx_rig("over").await;
        let app = app(db.clone());
        let before = (db_count(&db, "tbl_run").await, db_count(&db, "tbl_session").await, db_count(&db, "tbl_context_manifest").await);
        // 큰 프로필 파일 (ASCII 170K자 ≈ 42.5K 토큰) + 작은 파일
        db.execute_unprepared(&format!("INSERT INTO tbl_profile_file (profile_sn, path, content, sort) VALUES (1, 'rules/big.md', '{}', 2);", "x".repeat(170_000))).await.unwrap();
        let (st, e) = call(&app, "POST", "/tasks/1/runs", None).await;
        let msg = e["message"].as_str().unwrap_or_default();
        assert_eq!((st, e["error"].as_str()), (StatusCode::UNPROCESSABLE_ENTITY, Some("context_over")), "{e}");
        assert!(msg.contains("top: instruction rules/big.md 42.5K") && msg.contains("> cap 40.0K") && msg.matches(", ").count() == 2, "{msg}");
        db.execute_unprepared("UPDATE tbl_run SET status = 'failed' WHERE sn = 1;").await.unwrap();
        assert_eq!(call(&app, "POST", "/runs/1/retry", None).await.1["error"], "context_over");
        assert_eq!((db_count(&db, "tbl_run").await, db_count(&db, "tbl_session").await, db_count(&db, "tbl_context_manifest").await), before);
        assert_eq!(call(&app, "GET", "/tasks/1", None).await.1["status"], "todo"); // 태스크도 그대로

        // 견적: 200 + over
        let (st, v) = call(&app, "POST", "/tasks/1/estimate", None).await;
        assert_eq!((st, v["over"].as_bool(), v["cap"].as_i64(), v["estimate"].as_i64().unwrap() > 40_000), (StatusCode::OK, Some(true), Some(40_000), true));

        // 하위 Run: 64K 파일 3개 ≈ 48K 토큰 → Run을 만들지 않고 리드에게 context_over
        for f in ["a", "b", "c"] {
            std::fs::write(dir.join(format!("{f}.txt")), "y".repeat(70_000)).unwrap();
        }
        db.execute_unprepared("UPDATE tbl_agent_profile SET fallback_json = '[{\"runtime_sn\":1,\"connection_sn\":1,\"tier\":\"L\"},{\"runtime_sn\":1,\"connection_sn\":1,\"tier\":\"M\"}]' WHERE sn = 1; UPDATE tbl_run SET status = 'running' WHERE sn = 1;").await.unwrap();
        let brief = "@TASK v1\nid: T1.1  parent: R1  mode: runner  kind: fix\ngoal: g\nac: [1 a]\npaths: [a.txt, b.txt, c.txt]";
        let runs = db_count(&db, "tbl_run").await;
        let Spawn::Stop(Stop::Lead(why)) = runner::spawn(&db, 1, brief).await.unwrap() else { panic!("not stopped") };
        assert!(why.starts_with("context_over: input") && why.contains("top: file "), "{why}");
        assert_eq!(db_count(&db, "tbl_run").await, runs);
        // 파일 하나면 통과
        assert!(matches!(runner::spawn(&db, 1, &brief.replace("[a.txt, b.txt, c.txt]", "[a.txt]")).await.unwrap(), Spawn::Run(_)));
    }

    /// B-17 DoD 3: 견적 — Run이 없으면 cached 0 · 직전 Run과 접두가 같으면 그 접두 토큰이 cached_estimate · 담당 없음 409 · 없는 태스크 404
    #[tokio::test]
    async fn context_estimate() {
        let (db, _dir) = ctx_rig("estimate").await;
        let app = app(db.clone());
        let (st, v) = call(&app, "POST", "/tasks/1/estimate", None).await;
        assert_eq!((st, v["cached_estimate"].as_i64(), v["over"].as_bool(), v["sources"].as_array().unwrap().len()), (StatusCode::OK, Some(0), Some(false), 6));
        assert_eq!((v["sources"][0]["kind"].as_str(), v["sources"][0]["ref_label"].as_str(), v["sources"][5]["ref_label"].as_str()), (Some("preset"), Some("protocol/p"), Some("@TASK #2")));
        let est = v["estimate"].as_i64().unwrap();
        assert_eq!(est, v["sources"].as_array().unwrap().iter().map(|s| s["token_count"].as_i64().unwrap()).sum::<i64>());

        // 직전 Run(태스크 1)이 보낸 manifest → 태스크 2 견적의 접두 5개(@TASK 앞)가 캐시에 맞는다
        let prev = crate::context::assemble(&db, 1, None).await.unwrap();
        ctx_save(&db, 1, None, &prev).await;
        let prefix: i64 = prev.sources[..5].iter().map(|s| s.token_count).sum();
        let (_, v) = call(&app, "POST", "/tasks/2/estimate", None).await;
        assert_eq!((v["cached_estimate"].as_i64(), v["estimate"].as_i64().unwrap() > prefix), (Some(prefix), true));
        // 같은 태스크는 @TASK까지 전부 맞는다
        assert_eq!(call(&app, "POST", "/tasks/1/estimate", None).await.1["cached_estimate"].as_i64(), Some(est));
        // 접두가 중간에 달라지면 그 앞까지만 (프로필 파일 수정 → 그 앞 4개)
        db.execute_unprepared("UPDATE tbl_profile_file SET content = 'A11Y-CHANGED';").await.unwrap();
        let four: i64 = prev.sources[..4].iter().map(|s| s.token_count).sum();
        assert_eq!(call(&app, "POST", "/tasks/2/estimate", None).await.1["cached_estimate"].as_i64(), Some(four));
        db.execute_unprepared("UPDATE tbl_task SET member_sn = NULL WHERE sn = 2;").await.unwrap();
        assert_eq!(call(&app, "POST", "/tasks/2/estimate", None).await.0, StatusCode::CONFLICT);
        assert_eq!(call(&app, "POST", "/tasks/99/estimate", None).await.0, StatusCode::NOT_FOUND);
    }

    /// B-17 DoD 4 + 리드 실행: 리드 Run을 가짜 CLI로 두 번 실행 → 프롬프트 = 조립 결과 · 호출마다 manifest(상한 예산) · /runs/{sn}/context ·
    /// tbl_log_token 2건 뒤 연결 cache_hit_percent(300 / 500 = 60) · sync_at 갱신 · 팀 통계 avg_input_token · cache_hit_percent · 모델 호출은 실행기 1번씩
    #[tokio::test]
    async fn context_cache() {
        use crate::runner;
        let (db, dir) = ctx_rig("cache").await;
        let app = app(db.clone());
        assert_eq!(call(&app, "GET", "/connections/1", None).await.1["cache_hit_percent"], Value::Null);
        let want = crate::context::assemble(&db, 1, None).await.unwrap();
        assert_eq!(want.cached_estimate, 0);

        let said = runner::go(&db, 1, tokio::sync::watch::channel(false).1).await.unwrap();
        assert_eq!(said, "all done");
        assert_eq!(std::fs::read_to_string(dir.join("prompt.txt")).unwrap(), want.prompt); // 실행기가 받은 것 = 조립 결과
        assert!(std::fs::read_to_string(dir.join("args.txt")).unwrap().contains("--strict-mcp-config"));
        let r1 = call(&app, "GET", "/runs/1", None).await.1;
        assert_eq!((r1["status"].as_str(), r1["tokens"]["total"].as_i64()), (Some("completed"), Some(103)));
        assert_eq!(scalar(&db, "SELECT connection_sn FROM tbl_run WHERE sn = 1").await, 1); // 리드 Run에 연결 · 실행기를 남긴다
        // 첫 호출만으로 적중률이 생긴다 (0 / 100 = 0%)
        let c = call(&app, "GET", "/connections/1", None).await.1;
        assert_eq!((c["cache_hit_percent"].as_i64(), c["sync_at"].is_string()), (Some(0), true));

        // 두 번째 Run(같은 멤버): 직전 Run의 접두가 캐시 예상으로 잡힌다 → 실행 → 300 / (200 + 300) = 60%
        let pre = crate::context::assemble(&db, 2, None).await.unwrap();
        assert_eq!(pre.cached_estimate, pre.sources[..5].iter().map(|s| s.token_count).sum::<i64>());
        runner::go(&db, 2, tokio::sync::watch::channel(false).1).await.unwrap();
        assert_eq!(std::fs::read_to_string(dir.join("n")).unwrap().trim(), "2"); // 실행기 호출 = Run마다 1번 · 요약 · 압축 호출 없음
        let c = call(&app, "GET", "/connections/1", None).await.1;
        assert_eq!(c["cache_hit_percent"].as_i64(), Some(60));
        assert_eq!(db_count(&db, "tbl_log_token").await, 2);

        // /runs/{sn}/context: 호출마다 1개 · 출처 순서 · 예산 · 실측 캐시율
        let v = call(&app, "GET", "/runs/2/context", None).await.1;
        assert_eq!((v["manifests"].as_array().unwrap().len(), v["manifests"][0]["budget_token"].as_i64(), v["manifests"][0]["sources"].as_array().unwrap().len(), v["manifests"][0]["cache_percent"].as_i64()), (1, Some(40_000), 6, Some(75)));
        assert_eq!((v["manifests"][0]["sources"][0]["ref_label"].as_str(), v["manifests"][0]["sources"][5]["kind"].as_str(), v["manifests"][0]["total"].as_i64(), v["total"].as_i64()), (Some("protocol/p"), Some("task"), Some(pre.estimate), Some(pre.estimate)));
        assert_eq!(call(&app, "GET", "/runs/1/context", None).await.1["manifests"][0]["cache_percent"].as_i64(), Some(0));
        assert_eq!(call(&app, "GET", "/runs/99/context", None).await.0, StatusCode::NOT_FOUND);

        // 팀 통계: 호출당 평균 입력 (100 + 400) / 2 · 연결 캐시율
        let s = call(&app, "GET", "/teams/1/stats", None).await.1;
        assert_eq!((s["avg_input_token"].as_i64(), s["cache_hit_percent"].as_i64()), (Some(250), Some(60)));
    }

    /// B-13: 완료 조건 교체(sn 유지 · 체크 이벤트) · 라벨(PATCH 이름 → 생성 · 목록) · 의존(대기 계산 · 순환 · 중복 · 삭제) · 저장 보기 · 전체 목록 필터 · Diagram 배치
    #[tokio::test]
    async fn meta() {
        let db = mem().await;
        let app = app(db.clone());
        let a = task_of(&app, &db, true).await; // 프로젝트 1 · 이슈 1 · 태스크 a(member 1)
        let (_, b) = call(&app, "POST", "/issues/1/tasks", Some(json!({"title": "Login API", "priority": 0}))).await;
        let b = b["sn"].as_i64().unwrap();

        // 완료 조건: 만들기 → sn 유지한 채 체크 · 순서 변경 · 하나 삭제 → CriterionChecked 1번
        let v = call(&app, "PUT", &format!("/tasks/{a}/criteria"), Some(json!([{"content": "로그인 성공", "is_done": 0}, {"content": "에러 표시", "is_done": 0}]))).await.1;
        let (c1, c2) = (v[0]["sn"].as_i64().unwrap(), v[1]["sn"].as_i64().unwrap());
        let v = call(&app, "PUT", &format!("/tasks/{a}/criteria"), Some(json!([{"sn": c2, "content": "에러 표시", "is_done": 1}, {"content": "새 조건", "is_done": 0}]))).await.1;
        assert_eq!((v.as_array().unwrap().len(), v[0]["sn"].as_i64(), v[0]["is_done"].as_i64(), v[1]["content"].as_str()), (2, Some(c2), Some(1), Some("새 조건")));
        assert_eq!(call(&app, "PUT", &format!("/tasks/{a}/criteria"), Some(json!([{"sn": c1, "content": "x", "is_done": 0}]))).await.0, StatusCode::UNPROCESSABLE_ENTITY); // 지운 sn
        assert_eq!(call(&app, "PUT", &format!("/tasks/{a}/criteria"), Some(json!([{"content": " ", "is_done": 0}]))).await.0, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(call(&app, "GET", &format!("/tasks/{a}/criteria"), None).await.1[0]["sn"].as_i64(), Some(c2));
        let kinds: Vec<String> = events(&db, "task", a).await.into_iter().map(|e| e.0).collect();
        assert_eq!(kinds, ["TaskCreated", "CriteriaUpdated", "CriteriaUpdated", "CriterionChecked"]);

        // 라벨: 이름으로 붙이면 프로젝트 라벨이 생기고 중복 · 공백은 정리 · 빈 이름 422
        let t = call(&app, "PATCH", &format!("/tasks/{a}"), Some(json!({"labels": ["ui", "auth", " ui "]}))).await.1;
        assert_eq!(t["labels"], json!(["auth", "ui"]));
        call(&app, "PATCH", &format!("/tasks/{b}"), Some(json!({"labels": ["auth"]}))).await;
        assert_eq!(call(&app, "GET", "/projects/1/labels", None).await.1.as_array().unwrap().len(), 2);
        assert_eq!(call(&app, "PATCH", &format!("/tasks/{a}"), Some(json!({"labels": [""]}))).await.0, StatusCode::UNPROCESSABLE_ENTITY);

        // 의존: a가 b를 기다림 → waiting · b 완료 후 waiting 아님 · 순환 · 자기 자신 · 중복 · 다른 프로젝트
        let (st, d) = call(&app, "POST", &format!("/tasks/{a}/deps"), Some(json!({"depend_task_sn": b}))).await;
        assert_eq!((st, d), (StatusCode::CREATED, json!([b])));
        let t = call(&app, "GET", &format!("/tasks/{a}"), None).await.1;
        assert_eq!((t["deps"].clone(), t["waiting"].as_bool()), (json!([b]), Some(true)));
        assert_eq!(call(&app, "GET", "/projects/1/snapshot", None).await.1["tasks"][0]["waiting"], true);
        assert_eq!(call(&app, "POST", &format!("/tasks/{b}/deps"), Some(json!({"depend_task_sn": a}))).await.0, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(call(&app, "POST", &format!("/tasks/{a}/deps"), Some(json!({"depend_task_sn": a}))).await.0, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(call(&app, "POST", &format!("/tasks/{a}/deps"), Some(json!({"depend_task_sn": b}))).await.0, StatusCode::CONFLICT);
        let other = task_of(&app, &db, false).await;
        assert_eq!(call(&app, "POST", &format!("/tasks/{a}/deps"), Some(json!({"depend_task_sn": other}))).await.0, StatusCode::UNPROCESSABLE_ENTITY);
        // 이동 API 대신 SQL로 완료 — move_task가 프로세스 전역 버스의 TaskMoved 수를 센다
        db.execute_unprepared(&format!("UPDATE tbl_task SET status = 'done' WHERE sn = {b};")).await.unwrap();
        assert_eq!(call(&app, "GET", &format!("/tasks/{a}"), None).await.1["waiting"], false);
        assert_eq!(call(&app, "DELETE", &format!("/tasks/{a}/deps/{b}"), None).await.0, StatusCode::NO_CONTENT);
        assert_eq!(call(&app, "DELETE", &format!("/tasks/{a}/deps/{b}"), None).await.0, StatusCode::NOT_FOUND);

        // 전체 목록: 프로젝트 · 상태 · 우선순위 · 제목 검색 · #번호
        assert_eq!(call(&app, "GET", "/tasks", None).await.1.as_array().unwrap().len(), 3);
        assert_eq!(call(&app, "GET", "/tasks?project=1&status=done", None).await.1[0]["sn"].as_i64(), Some(b));
        assert_eq!(call(&app, "GET", "/tasks?priority=0", None).await.1.as_array().unwrap().len(), 1);
        assert_eq!(call(&app, "GET", "/tasks?q=login", None).await.1[0]["sn"].as_i64(), Some(b));
        assert_eq!(call(&app, "GET", "/tasks?project=1&q=%232", None).await.1[0]["sn"].as_i64(), Some(a));
        assert_eq!(call(&app, "GET", "/tasks?member=1", None).await.1.as_array().unwrap().len(), 1);

        // 저장 보기: 만들기 · 목록 · 필터는 객체만 · 삭제 후 404
        let (st, v) = call(&app, "POST", "/task-views", Some(json!({"name": "P0–P1", "filter": {"priority": [0, 1]}}))).await;
        assert_eq!((st, v["filter"]["priority"][1].as_i64()), (StatusCode::CREATED, Some(1)));
        assert_eq!(call(&app, "GET", "/task-views", None).await.1[0]["name"], "P0–P1");
        assert_eq!(call(&app, "POST", "/task-views", Some(json!({"name": "x", "filter": []}))).await.0, StatusCode::UNPROCESSABLE_ENTITY);
        let vs = v["sn"].as_i64().unwrap();
        assert_eq!(call(&app, "DELETE", &format!("/task-views/{vs}"), None).await.0, StatusCode::NO_CONTENT);
        assert_eq!(call(&app, "DELETE", &format!("/task-views/{vs}"), None).await.0, StatusCode::NOT_FOUND);

        // Diagram: 기본값 → 저장 → 다시 읽으면 유지 · 다시 저장하면 교체 · 잘못된 값 422
        assert_eq!(call(&app, "GET", "/projects/1/diagram", None).await.1["view"]["layout_mode"], "auto");
        let body = json!({"view": {"layout_mode": "manual", "zoom_percent": 80, "is_show_capability": 0, "is_show_done": 1},
            "nodes": [{"node_type": "task", "node_sn": a, "pos_x": 10, "pos_y": 20}, {"node_type": "member", "node_sn": 1, "pos_x": -5, "pos_y": 0, "is_collapsed": 1}]});
        assert_eq!(call(&app, "PUT", "/projects/1/diagram", Some(body)).await.0, StatusCode::OK);
        let d = call(&app, "GET", "/projects/1/diagram", None).await.1;
        assert_eq!((d["view"]["zoom_percent"].as_i64(), d["nodes"].as_array().unwrap().len(), d["nodes"][1]["is_collapsed"].as_i64()), (Some(80), 2, Some(1)));
        call(&app, "PUT", "/projects/1/diagram", Some(json!({"view": d["view"], "nodes": [{"node_type": "task", "node_sn": a, "pos_x": 1, "pos_y": 1}]}))).await;
        assert_eq!(call(&app, "GET", "/projects/1/diagram", None).await.1["nodes"].as_array().unwrap().len(), 1);
        for bad in [json!({"view": {"layout_mode": "x", "zoom_percent": 100, "is_show_capability": 1, "is_show_done": 0}, "nodes": []}),
                    json!({"view": d["view"], "nodes": [{"node_type": "task", "node_sn": 1, "pos_x": 0, "pos_y": 0}, {"node_type": "task", "node_sn": 1, "pos_x": 1, "pos_y": 1}]})] {
            assert_eq!(call(&app, "PUT", "/projects/1/diagram", Some(bad)).await.0, StatusCode::UNPROCESSABLE_ENTITY);
        }
        assert_eq!(call(&app, "GET", "/projects/99/diagram", None).await.0, StatusCode::NOT_FOUND);
    }

    /// B-14: 프리셋 만들기 · 복제 · 새 버전(DoD: 연결 고정 버전 유지) · 검사 422(상한 · 비밀키 · 겹치는 규칙 줄) · 기본 제공 409 · 사용처 · .md 가져오기,
    /// 보고서 양식 수정(잠긴 칸 422) · 미리보기 조립, 워크스페이스 보안 기본값
    #[tokio::test]
    async fn preset() {
        let db = mem().await;
        let app = app(db.clone());
        let ts = task_of(&app, &db, true).await; // 프로필 1 · 멤버 1(m)

        // 만들기 → 새 버전: 버전 2, 연결(고정 v1)은 그대로
        let (st, p) = call(&app, "POST", "/presets", Some(json!({"kind": "role", "preset_key": "qa", "name": "QA", "limit_tok": 100, "content": "# Role: QA\n- Test every endpoint."}))).await;
        assert_eq!((st, p["version"].as_i64(), p["is_builtin"].as_i64()), (StatusCode::CREATED, Some(1), Some(0)));
        let ps = p["sn"].as_i64().unwrap();
        db.execute_unprepared(&format!("INSERT INTO tbl_map_profile_preset (profile_sn, preset_sn) VALUES (1, {ps});")).await.unwrap();
        let (st, p) = call(&app, "PUT", &format!("/presets/{ps}"), Some(json!({"content": "# Role: QA\n- Test every endpoint.\n- Report flaky tests.", "change_note": "flaky"}))).await;
        assert_eq!((st, p["version"].as_i64()), (StatusCode::OK, Some(2)));
        let v2 = p["sn"].as_i64().unwrap();
        assert_ne!(v2, ps); // 새 버전 = 새 행
        assert_eq!(scalar(&db, "SELECT preset_sn FROM tbl_map_profile_preset").await, ps); // 연결은 고정한 v1 행 그대로
        assert_eq!(scalar(&db, &format!("SELECT is_latest FROM tbl_instruction_preset WHERE sn = {ps}")).await, 0);
        let v = call(&app, "GET", &format!("/presets/{ps}/versions"), None).await.1;
        assert_eq!((v[0]["version"].as_i64(), v[0]["change_note"].as_str(), v[1]["version"].as_i64()), (Some(2), Some("flaky"), Some(1)));
        assert_eq!((events(&db, "preset", ps).await.iter().map(|e| e.0.as_str()).collect::<Vec<_>>(), events(&db, "preset", v2).await.iter().map(|e| e.0.as_str()).collect::<Vec<_>>()), (vec!["PresetCreated"], vec!["PresetVersioned"]));

        // 사용처: 멤버 m · 고정 v1
        let u = call(&app, "GET", &format!("/presets/{ps}/usage"), None).await.1;
        assert_eq!((u[0]["owner"].as_str(), u[0]["name"].as_str(), u[0]["detail"].as_str()), (Some("member"), Some("m"), Some("v1")));

        // 검사: 상한 · 비밀키 · 본문 안 겹치는 줄 · 다른 rule 프리셋과 겹치는 줄 → 422 사유
        let long = "x".repeat(500);
        let (st, e) = call(&app, "PUT", &format!("/presets/{ps}"), Some(json!({"content": long}))).await;
        assert!(st == StatusCode::UNPROCESSABLE_ENTITY && e["message"].as_str().unwrap().contains("token limit"), "{e}");
        // 한글은 글자당 1.5토큰: 70자 = 105 > 100 (글자/4면 18이라 통과했을 길이)
        let (st, e) = call(&app, "PUT", &format!("/presets/{ps}"), Some(json!({"content": "가".repeat(70)}))).await;
        assert!(st == StatusCode::UNPROCESSABLE_ENTITY && e["message"].as_str().unwrap().contains("token limit: 105 > 100"), "{e}");
        let (_, e) = call(&app, "PUT", &format!("/presets/{ps}"), Some(json!({"content": "key sk-abcdefghijklmnop1234"}))).await;
        assert!(e["message"].as_str().unwrap().contains("secret pattern: sk-"), "{e}");
        let (_, e) = call(&app, "PUT", &format!("/presets/{ps}"), Some(json!({"content": "- a\n- a"}))).await;
        assert!(e["message"].as_str().unwrap().contains("duplicate rule line"), "{e}");
        call(&app, "POST", "/presets", Some(json!({"kind": "rule", "preset_key": "base", "name": "Base", "limit_tok": 100, "content": "- Keep diffs small."}))).await;
        let (st, e) = call(&app, "POST", "/presets", Some(json!({"kind": "rule", "preset_key": "more", "name": "More", "limit_tok": 100, "content": "- Keep diffs small.\n- Ask first."}))).await;
        assert!(st == StatusCode::UNPROCESSABLE_ENTITY && e["message"].as_str().unwrap().contains("also in base"), "{e}");
        assert_eq!(call(&app, "POST", "/presets", Some(json!({"kind": "protocol", "preset_key": "p", "name": "P", "limit_tok": 9, "content": "x"}))).await.0, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(call(&app, "POST", "/presets", Some(json!({"kind": "role", "preset_key": "qa", "name": "QA2", "limit_tok": 100, "content": "x"}))).await.0, StatusCode::CONFLICT);

        // 기본 제공은 수정 409 → 복제하면 새 프리셋(본문 · 상한 이어받음)
        db.execute_unprepared("INSERT INTO tbl_instruction_preset (sn, workspace_sn, kind, preset_key, name, version, content, limit_tok, is_builtin) VALUES (90, 1, 'style', 'terse', 'Terse', 1, 'Be terse.', 50, 1);").await.unwrap();
        assert_eq!(call(&app, "PUT", "/presets/90", Some(json!({"content": "x"}))).await.0, StatusCode::CONFLICT);
        let (st, c) = call(&app, "POST", "/presets", Some(json!({"preset_key": "terse2", "name": "Terse 2", "copy_from_sn": 90}))).await;
        assert_eq!((st, c["kind"].as_str(), c["limit_tok"].as_i64(), c["copy_from_sn"].as_i64()), (StatusCode::CREATED, Some("style"), Some(50), Some(90)));
        assert_eq!(call(&app, "GET", &format!("/presets/{}/versions", c["sn"]), None).await.1[0]["content"], "Be terse.");

        // .md 가져오기: 새 프리셋(import) · 같은 키면 새 버전 · 앞머리 없음 / 새인데 limit 없음 422
        let md = "---\nkind: role\nkey: backend\nname: Backend\nlimit_tok: 600\n---\n# Role: Backend\n- Validate input.";
        let (st, b) = call(&app, "POST", "/presets/import", Some(json!({"markdown": md}))).await;
        assert_eq!((st, b["preset_key"].as_str(), b["version"].as_i64()), (StatusCode::OK, Some("backend"), Some(1)));
        assert_eq!(call(&app, "GET", &format!("/presets/{}/versions", b["sn"]), None).await.1[0]["source"], "import");
        assert_eq!(call(&app, "POST", "/presets/import", Some(json!({"markdown": md.replace("Validate", "Check")}))).await.1["version"], 2);
        assert_eq!(call(&app, "POST", "/presets/import", Some(json!({"markdown": "# no head"}))).await.0, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(call(&app, "POST", "/presets/import", Some(json!({"markdown": "---\nkind: role\nkey: z\n---\nbody"}))).await.0, StatusCode::UNPROCESSABLE_ENTITY);

        // 보고서 양식: 잠긴 칸을 지우면 422 · 고치면 버전 +1
        db.execute_unprepared("INSERT INTO tbl_report_form (workspace_sn, kind, form_key, name, body, locked_json, is_default) VALUES (1, 'task_report', 'task-report', '태스크 보고서', \
            '#{{task.num}} {{task.title}} [{{status.label}}]\n[[result]]\n{{#each area}}- {{key}}: {{value}}\n{{/each}}통과: {{tests.passed_summary}} / 미확인: [[unverified]]{{#if pr}} · PR #{{pr.num}}{{/if}} / {{run.tokens}} tok', \
            '[\"unverified\",\"tests.passed_summary\"]', 1);").await.unwrap();
        assert_eq!(call(&app, "GET", "/report-forms", None).await.1.as_array().unwrap().len(), 1);
        let f = call(&app, "GET", "/report-forms/task-report", None).await.1;
        assert_eq!(f["locked"], json!(["unverified", "tests.passed_summary"]));
        let (st, e) = call(&app, "PUT", "/report-forms/task-report", Some(json!({"body": "[[result]]"}))).await;
        assert!(st == StatusCode::UNPROCESSABLE_ENTITY && e["message"].as_str().unwrap().contains("unverified"), "{e}");
        let body = f["body"].as_str().unwrap().to_owned();
        assert_eq!(call(&app, "PUT", "/report-forms/task-report", Some(json!({"body": body}))).await.1["version"], 2);
        assert_eq!(call(&app, "GET", "/report-forms/nope", None).await.0, StatusCode::NOT_FOUND);

        // 미리보기: Run 보고 항목 · 결과 요약 · 토큰으로 조립, PR 없으면 if 블록 빠짐
        let rs = call(&app, "POST", &format!("/tasks/{ts}/runs"), None).await.1["sn"].as_i64().unwrap();
        db.execute_unprepared(&format!("UPDATE tbl_run SET result_summary = 'tests 12/12' WHERE sn = {rs}; \
            INSERT INTO tbl_log_token (run_sn, token_input) VALUES ({rs}, 420); \
            INSERT INTO tbl_report_item (run_sn, task_sn, kind, ref_key, code, value) VALUES ({rs}, {ts}, 'result', NULL, 'ok', '로그인 완료'), ({rs}, {ts}, 'area', 'auth', 'ok', '토큰 갱신'), ({rs}, {ts}, 'unverified', NULL, 'ok', '없음');")).await.unwrap();
        let (st, pv) = call(&app, "POST", &format!("/report-forms/task-report/preview?task={ts}"), None).await;
        let text = pv["body"].as_str().unwrap();
        assert_eq!(st, StatusCode::OK);
        assert!(text.starts_with("#2 T [in_progress]\n로그인 완료\n- auth: 토큰 갱신\n통과: tests 12/12 / 미확인: 없음 / 420 tok"), "{text}");
        assert_eq!(pv["missing"], json!([]));

        // 워크스페이스 보안 기본값: 기본 → 저장(프로필 생성 · 규칙 · 가드 · GitHub) → 다시 읽기 · 잘못된 값 422
        let w = call(&app, "GET", "/workspace/profile", None).await.1;
        assert_eq!((w["trust_level"].as_i64(), w["github_mode"].as_str(), w["rules"].as_array().unwrap().len()), (Some(3), Some("bot"), 0));
        let body = json!({"trust_level": 2, "github_mode": "personal", "github_account": "me", "github_repo_scope": "orch/*",
            "rules": [{"action_code": "pr_create", "title": "PR 생성", "policy": "approval", "approver": "user"}, {"action_code": "command", "title": "git push --force", "pattern": "git push --force", "policy": "block"}],
            "guards": [{"name": "비밀키 출력", "stage": "output", "pattern": "sk-"}]});
        let w = call(&app, "PUT", "/workspace/profile", Some(body.clone())).await.1;
        assert_eq!((w["trust_level"].as_i64(), w["rules"][1]["policy"].as_str(), w["guards"][0]["is_enabled"].as_i64(), w["github_account"].as_str()), (Some(2), Some("block"), Some(1), Some("me")));
        assert_eq!(call(&app, "GET", "/workspace/profile", None).await.1["rules"].as_array().unwrap().len(), 2);
        for bad in [json!({"trust_level": 5}), json!({"rules": [{"action_code": "command", "title": "x", "policy": "block"}]}), json!({"guards": [{"name": "g", "stage": "x"}]})] {
            let mut b = body.clone();
            b.as_object_mut().unwrap().extend(bad.as_object().unwrap().clone());
            assert_eq!(call(&app, "PUT", "/workspace/profile", Some(b)).await.0, StatusCode::UNPROCESSABLE_ENTITY);
        }
    }

    #[tokio::test]
    async fn openapi() {
        let (st, v) = call(&setup().await, "GET", "/openapi.json", None).await;
        assert_eq!(st, StatusCode::OK);
        for path in ["/health", "/projects", "/projects/{sn}", "/projects/{sn}/issues", "/issues/{sn}", "/issues/{sn}/tasks", "/projects/{sn}/tasks", "/tasks/{sn}", "/tasks/{sn}/move", "/tasks/{sn}/runs", "/runs/{sn}", "/runs/{sn}/sessions", "/runs/{sn}/stop", "/runs/{sn}/retry", "/runs/{sn}/review", "/runs/{sn}/approve", "/runs/{sn}/reject", "/tasks/{sn}/assign", "/profiles", "/profiles/{sn}", "/templates", "/templates/{sn}", "/teams", "/teams/{sn}", "/teams/{sn}/members", "/members/{sn}", "/connections", "/connections/{sn}", "/runtimes", "/workspace", "/presets", "/presets/{sn}/versions", "/skill-sources", "/skills", "/skills/{sn}", "/skills/{sn}/usage", "/mcps", "/mcps/{sn}", "/mcps/{sn}/usage", "/profiles/{sn}/skills", "/notifications", "/notifications/read", "/notify/rules", "/notify/test", "/audit", "/projects/{sn}/conversation", "/projects/{sn}/messages", "/messages/{sn}", "/messages/{sn}/proceed", "/messages/{sn}/cancel", "/asks/{sn}/{action}", "/runs/{sn}/instruct", "/runs/{sn}/children", "/teams/{sn}/stats", "/teams/{sn}/quota", "/workspace/cost", "/tasks", "/tasks/{sn}/criteria", "/tasks/{sn}/deps", "/tasks/{sn}/deps/{dep}", "/projects/{sn}/labels", "/task-views", "/task-views/{sn}", "/projects/{sn}/diagram", "/presets/{sn}", "/presets/{sn}/usage", "/presets/import", "/report-forms", "/report-forms/{key}", "/report-forms/{key}/preview", "/workspace/profile", "/asks", "/asks/{sn}", "/asks/{sn}/answer", "/asks/{sn}/writing", "/asks/{sn}/approve", "/asks/{sn}/deny", "/asks/{sn}/hold", "/asks/{sn}/{action}"] {
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
