//! tbl_project 기본 CRUD (첫 리소스 · 나머지 테이블도 같은 모양으로 추가)
use crate::entity::tbl_project::{self as p, Entity as Project};
use axum::{Json, Router, extract::{Path, State}, http::StatusCode, routing::get};
use sea_orm::{ColumnTrait, QueryFilter, ActiveValue::{NotSet, Set}, DatabaseConnection, DbErr, EntityTrait, QueryOrder, SqlErr, sea_query::Expr};
use serde::Deserialize;

/// project 관련 경로 묶음
pub fn routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/projects", get(list).post(create))
        .route("/projects/{sn}", get(read).patch(update).delete(remove))
}

/// 생성 요청 본문. wid · name만 필수, 나머지는 생략하면 DB 기본값을 쓴다
#[derive(Deserialize)]
struct New {
    wid: i64,
    name: String,
    team_sn: Option<i64>,
    repo_name: Option<String>,
    repo_path: Option<String>,
    default_branch: Option<String>,
}

/// 수정 요청 본문. 보낸 필드만 바꾼다 (생략 = 그대로 둠)
#[derive(Deserialize)]
struct Patch {
    name: Option<String>,
    repo_name: Option<String>,
    repo_path: Option<String>,
    default_branch: Option<String>,
    status: Option<String>,
    sort: Option<i64>,
}

/// 핸들러 반환 형태. 실패는 상태 코드만 돌려준다
type Res<T> = Result<T, StatusCode>;

/// DB 오류 → HTTP 상태 코드. 외래키 위반은 잘못된 입력이므로 422, 나머지는 500
fn error(e: DbErr) -> StatusCode {
    match e.sql_err() {
        Some(SqlErr::ForeignKeyConstraintViolation(_)) => StatusCode::UNPROCESSABLE_ENTITY,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

/// 목록 (탭 순서 sort, 같으면 번호순)
async fn list(State(db): State<DatabaseConnection>) -> Res<Json<Vec<p::Model>>> {
    Project::find().order_by_asc(p::Column::Sort).order_by_asc(p::Column::Sn).all(&db).await.map(Json).map_err(error)
}

/// 1건 조회. 없으면 404
async fn read(State(db): State<DatabaseConnection>, Path(sn): Path<i64>) -> Res<Json<p::Model>> {
    Project::find_by_id(sn).one(&db).await.map_err(error)?.map(Json).ok_or(StatusCode::NOT_FOUND)
}

/// 생성 후 DB 기본값까지 채운 행을 201로 돌려준다
async fn create(State(db): State<DatabaseConnection>, Json(b): Json<New>) -> Res<(StatusCode, Json<p::Model>)> {
    let m = p::ActiveModel {
        wid: Set(b.wid),
        name: Set(b.name),
        team_sn: Set(b.team_sn),
        repo_name: Set(b.repo_name),
        repo_path: Set(b.repo_path),
        default_branch: b.default_branch.map_or(NotSet, Set), // NotSet → DB 기본값 'main'
        ..Default::default()
    };
    // 삽입 후 DB 기본값(status, 시각 등)까지 채워서 돌려받는다
    let sn = Project::insert(m).exec(&db).await.map_err(error)?.last_insert_id;
    read(State(db), Path(sn)).await.map(|j| (StatusCode::CREATED, j))
}

/// 부분 수정 후 최신 행을 돌려준다. 없으면 404
async fn update(State(db): State<DatabaseConnection>, Path(sn): Path<i64>, Json(b): Json<Patch>) -> Res<Json<p::Model>> {
    // update_at은 항상 갱신하고, 나머지는 요청에 있는 필드만 SET에 추가한다
    let mut q = Project::update_many().filter(p::Column::Sn.eq(sn)).col_expr(p::Column::UpdateAt, Expr::cust("datetime('now')"));
    if let Some(v) = b.name { q = q.col_expr(p::Column::Name, v.into()); }
    if let Some(v) = b.repo_name { q = q.col_expr(p::Column::RepoName, v.into()); }
    if let Some(v) = b.repo_path { q = q.col_expr(p::Column::RepoPath, v.into()); }
    if let Some(v) = b.default_branch { q = q.col_expr(p::Column::DefaultBranch, v.into()); }
    if let Some(v) = b.status { q = q.col_expr(p::Column::Status, v.into()); }
    if let Some(v) = b.sort { q = q.col_expr(p::Column::Sort, v.into()); }
    // 바뀐 행이 없으면 대상이 없는 것
    if q.exec(&db).await.map_err(error)?.rows_affected == 0 {
        return Err(StatusCode::NOT_FOUND);
    }
    read(State(db), Path(sn)).await
}

/// 삭제. 성공 204, 없으면 404 (하위 데이터는 스키마의 ON DELETE 규칙을 따른다)
async fn remove(State(db): State<DatabaseConnection>, Path(sn): Path<i64>) -> Res<StatusCode> {
    match Project::delete_by_id(sn).exec(&db).await.map_err(error)?.rows_affected {
        0 => Err(StatusCode::NOT_FOUND),
        _ => Ok(StatusCode::NO_CONTENT),
    }
}
