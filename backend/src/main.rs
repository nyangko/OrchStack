#[allow(unused_imports, dead_code)] // sea-orm-cli 생성 코드
// OrchStack 백엔드 진입점: DB 준비 → 라우터 구성 → HTTP 서버 실행

mod entity; // 테이블별 SeaORM entity (sea-orm-cli 생성물 · 직접 수정하지 않는다)
mod project; // /projects CRUD

use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr, Statement};
use serde_json::{Value, json};

// 스키마 원본. 빌드할 때 파일 내용을 바이너리에 포함한다
const SCHEMA: &str = include_str!("../../data/sqlite.sql");

/// 연결 후 테이블이 없으면 data/sqlite.sql로 만든다 (스키마 원본).
async fn connect(opt: ConnectOptions) -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect(opt).await?;
    let probe = Statement::from_string(DbBackend::Sqlite, "SELECT 1 FROM sqlite_master WHERE name = 'tbl_user'");
    if db.query_one_raw(probe).await?.is_none() {
        db.execute_unprepared(SCHEMA).await?;
    }
    Ok(db)
}

/// 전체 라우터. 테스트에서도 같은 라우터를 쓰려고 main에서 분리했다
fn app(db: DatabaseConnection) -> Router {
    Router::new().route("/health", get(health)).merge(project::routes()).with_state(db)
}

/// 서버와 DB가 살아 있는지 확인한다 (DB 연결이 끊기면 503)
async fn health(State(db): State<DatabaseConnection>) -> (StatusCode, Json<Value>) {
    match db.ping().await {
        Ok(()) => (StatusCode::OK, Json(json!({ "status": "ok", "db": "ok" }))),
        Err(_) => (StatusCode::SERVICE_UNAVAILABLE, Json(json!({ "status": "error", "db": "down" }))),
    }
}

/// 환경변수로 설정을 읽고 서버를 시작한다
#[tokio::main]
async fn main() {
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

    #[tokio::test]
    async fn health() {
        // 메모리 DB는 연결마다 따로 생기므로 연결 1개로 고정
        let mut opt = ConnectOptions::new("sqlite::memory:");
        opt.max_connections(1);
        let db = connect(opt).await.unwrap();
        // project가 참조하는 사용자 · 워크스페이스를 미리 넣는다 (스키마에 기본 데이터 없음)
        db.execute_unprepared("INSERT INTO tbl_user (name) VALUES ('t'); INSERT INTO tbl_workspace (uid, name) VALUES (1, 'w');").await.unwrap();
        let app = app(db);

        assert_eq!(call(&app, "GET", "/health", None).await.0, StatusCode::OK);

        // FK 위반(없는 워크스페이스)은 422
        assert_eq!(call(&app, "POST", "/projects", Some(json!({"wid": 99, "name": "x"}))).await.0, StatusCode::UNPROCESSABLE_ENTITY);

        // 생성: DB 기본값(default_branch = 'main')이 채워져야 한다
        let (st, p) = call(&app, "POST", "/projects", Some(json!({"wid": 1, "name": "OrchStack"}))).await;
        assert_eq!((st, p["default_branch"].as_str()), (StatusCode::CREATED, Some("main")));
        let sn = p["sn"].as_i64().unwrap();

        // 목록 · 수정 · 조회 · 삭제 후 404
        assert_eq!(call(&app, "GET", "/projects", None).await.1.as_array().unwrap().len(), 1);
        let (_, p) = call(&app, "PATCH", &format!("/projects/{sn}"), Some(json!({"name": "Renamed", "status": "archived"}))).await;
        assert_eq!((p["name"].as_str(), p["status"].as_str()), (Some("Renamed"), Some("archived")));
        assert_eq!(call(&app, "GET", &format!("/projects/{sn}"), None).await.1["name"], "Renamed");
        assert_eq!(call(&app, "DELETE", &format!("/projects/{sn}"), None).await.0, StatusCode::NO_CONTENT);
        assert_eq!(call(&app, "GET", &format!("/projects/{sn}"), None).await.0, StatusCode::NOT_FOUND);
    }
}
