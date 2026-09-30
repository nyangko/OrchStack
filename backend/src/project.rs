//! tbl_project 기본 CRUD (첫 리소스 · 나머지 테이블도 같은 모양으로 추가)
use crate::{entity::tbl_project::{self as p, Entity as Project}, error::{Body, Error, Res, Sn}};
use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use sea_orm::{ColumnTrait, QueryFilter, ActiveValue::{NotSet, Set}, DatabaseConnection, EntityTrait, QueryOrder, sea_query::Expr};
use serde::Deserialize;

/// project 관련 경로 묶음
pub fn routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/projects", get(list).post(create))
        .route("/projects/{sn}", get(read).patch(update).delete(remove))
}

/// 생성 요청 본문. name만 필수 (워크스페이스는 Alpha에서 1개 고정), 나머지는 생략하면 DB 기본값을 쓴다
#[derive(Deserialize)]
struct New {
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

/// 목록 (탭 순서 sort, 같으면 번호순)
async fn list(State(db): State<DatabaseConnection>) -> Res<Json<Vec<p::Model>>> {
    Ok(Project::find().order_by_asc(p::Column::Sort).order_by_asc(p::Column::Sn).all(&db).await.map(Json)?)
}

/// 1건 조회. 없으면 404
async fn read(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<p::Model>> {
    Project::find_by_id(sn).one(&db).await?.map(Json).ok_or_else(Error::not_found)
}

/// 생성 후 DB 기본값까지 채운 행을 201로 돌려준다
async fn create(State(db): State<DatabaseConnection>, Body(b): Body<New>) -> Res<(StatusCode, Json<p::Model>)> {
    let m = p::ActiveModel {
        wid: Set(crate::WID),
        name: Set(b.name),
        team_sn: Set(b.team_sn),
        repo_name: Set(b.repo_name),
        repo_path: Set(b.repo_path),
        default_branch: b.default_branch.map_or(NotSet, Set), // NotSet → DB 기본값 'main'
        ..Default::default()
    };
    // 삽입 후 DB 기본값(status, 시각 등)까지 채워서 돌려받는다
    let sn = Project::insert(m).exec(&db).await?.last_insert_id;
    read(State(db), Sn(sn)).await.map(|j| (StatusCode::CREATED, j))
}

/// 부분 수정 후 최신 행을 돌려준다. 없으면 404
async fn update(State(db): State<DatabaseConnection>, Sn(sn): Sn, Body(b): Body<Patch>) -> Res<Json<p::Model>> {
    // update_at은 항상 갱신하고, 나머지는 요청에 있는 필드만 SET에 추가한다
    let mut q = Project::update_many().filter(p::Column::Sn.eq(sn)).col_expr(p::Column::UpdateAt, Expr::cust("datetime('now')"));
    if let Some(v) = b.name { q = q.col_expr(p::Column::Name, v.into()); }
    if let Some(v) = b.repo_name { q = q.col_expr(p::Column::RepoName, v.into()); }
    if let Some(v) = b.repo_path { q = q.col_expr(p::Column::RepoPath, v.into()); }
    if let Some(v) = b.default_branch { q = q.col_expr(p::Column::DefaultBranch, v.into()); }
    if let Some(v) = b.status { q = q.col_expr(p::Column::Status, v.into()); }
    if let Some(v) = b.sort { q = q.col_expr(p::Column::Sort, v.into()); }
    // 바뀐 행이 없으면 대상이 없는 것
    if q.exec(&db).await?.rows_affected == 0 {
        return Err(Error::not_found());
    }
    read(State(db), Sn(sn)).await
}

/// 삭제. 성공 204, 없으면 404 (하위 데이터는 스키마의 ON DELETE 규칙을 따른다)
async fn remove(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<StatusCode> {
    match Project::delete_by_id(sn).exec(&db).await?.rows_affected {
        0 => Err(Error::not_found()),
        _ => Ok(StatusCode::NO_CONTENT),
    }
}
