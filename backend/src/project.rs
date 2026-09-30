//! tbl_project 기본 CRUD (첫 리소스 · 나머지 테이블도 같은 모양으로 추가)
use crate::{entity::tbl_project::{self as p, Entity as Tbl}, error::{Body, Error, ErrorBody, Res, Sn}};
use axum::{Json, extract::State, http::StatusCode};
use sea_orm::{ColumnTrait, QueryFilter, ActiveValue::{NotSet, Set}, DatabaseConnection, EntityTrait, QueryOrder, sea_query::Expr};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

/// project 관련 경로 묶음 (OpenAPI 문서도 함께 모은다)
pub fn routes() -> OpenApiRouter<DatabaseConnection> {
    OpenApiRouter::new()
        .routes(routes!(list, create))
        .routes(routes!(read, update, remove))
}

/// 프로젝트 (API 응답 형태)
#[derive(Serialize, ToSchema)]
pub struct Project {
    sn: i64,
    wid: i64,
    team_sn: Option<i64>,
    name: String,
    repo_name: Option<String>,
    repo_path: Option<String>,
    default_branch: String,
    next_num: i64,
    is_github_import: i64,
    import_label: Option<String>,
    /// active(진행 중) | archived(보관 · 읽기 전용)
    status: String,
    sort: i64,
    create_at: String,
    update_at: String,
}

impl From<p::Model> for Project {
    fn from(m: p::Model) -> Self {
        Self {
            sn: m.sn, wid: m.wid, team_sn: m.team_sn, name: m.name, repo_name: m.repo_name, repo_path: m.repo_path,
            default_branch: m.default_branch, next_num: m.next_num, is_github_import: m.is_github_import,
            import_label: m.import_label, status: m.status, sort: m.sort, create_at: m.create_at, update_at: m.update_at,
        }
    }
}

/// 생성 요청 본문. name만 필수 (워크스페이스는 Alpha에서 1개 고정), 나머지는 생략하면 DB 기본값을 쓴다
#[derive(Deserialize, ToSchema)]
struct ProjectNew {
    name: String,
    team_sn: Option<i64>,
    repo_name: Option<String>,
    repo_path: Option<String>,
    default_branch: Option<String>,
}

/// 수정 요청 본문. 보낸 필드만 바꾼다 (생략 = 그대로 둠)
#[derive(Deserialize, ToSchema)]
struct ProjectPatch {
    name: Option<String>,
    repo_name: Option<String>,
    repo_path: Option<String>,
    default_branch: Option<String>,
    status: Option<String>,
    sort: Option<i64>,
}

/// 목록 (탭 순서 sort, 같으면 번호순)
#[utoipa::path(get, path = "/projects", responses((status = 200, body = Vec<Project>), (status = "default", body = ErrorBody)))]
async fn list(State(db): State<DatabaseConnection>) -> Res<Json<Vec<Project>>> {
    Ok(Json(Tbl::find().order_by_asc(p::Column::Sort).order_by_asc(p::Column::Sn).all(&db).await?.into_iter().map(Project::from).collect()))
}

/// 1건 조회. 없으면 404
#[utoipa::path(get, path = "/projects/{sn}", params(("sn" = i64, Path, description = "프로젝트 번호")), responses((status = 200, body = Project), (status = "default", body = ErrorBody)))]
async fn read(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Project>> {
    Tbl::find_by_id(sn).one(&db).await?.map(|m| Json(m.into())).ok_or_else(Error::not_found)
}

/// 생성 후 DB 기본값까지 채운 행을 201로 돌려준다
#[utoipa::path(post, path = "/projects", request_body = ProjectNew, responses((status = 201, body = Project), (status = "default", body = ErrorBody)))]
async fn create(State(db): State<DatabaseConnection>, Body(b): Body<ProjectNew>) -> Res<(StatusCode, Json<Project>)> {
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
    let sn = Tbl::insert(m).exec(&db).await?.last_insert_id;
    read(State(db), Sn(sn)).await.map(|j| (StatusCode::CREATED, j))
}

/// 부분 수정 후 최신 행을 돌려준다. 없으면 404
#[utoipa::path(patch, path = "/projects/{sn}", params(("sn" = i64, Path, description = "프로젝트 번호")), request_body = ProjectPatch, responses((status = 200, body = Project), (status = "default", body = ErrorBody)))]
async fn update(State(db): State<DatabaseConnection>, Sn(sn): Sn, Body(b): Body<ProjectPatch>) -> Res<Json<Project>> {
    // update_at은 항상 갱신하고, 나머지는 요청에 있는 필드만 SET에 추가한다
    let mut q = Tbl::update_many().filter(p::Column::Sn.eq(sn)).col_expr(p::Column::UpdateAt, Expr::cust("datetime('now')"));
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
#[utoipa::path(delete, path = "/projects/{sn}", params(("sn" = i64, Path, description = "프로젝트 번호")), responses((status = 204, description = "삭제됨"), (status = "default", body = ErrorBody)))]
async fn remove(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<StatusCode> {
    match Tbl::delete_by_id(sn).exec(&db).await?.rows_affected {
        0 => Err(Error::not_found()),
        _ => Ok(StatusCode::NO_CONTENT),
    }
}
