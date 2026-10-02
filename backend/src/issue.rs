//! tbl_issue CRUD. 쓰기는 event::run 경유 (IssueCreated · IssueUpdated · IssueDeleted)
use crate::{entity::{tbl_issue::{self as i, Entity as Tbl}, tbl_project}, error::{Body, Error, ErrorBody, Res, Sn}, event::{self, Ev}};
use axum::{Json, extract::State, http::StatusCode};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder, sea_query::Expr};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

/// 허용되는 이슈 상태
const STATUS: [&str; 4] = ["open", "in_progress", "done", "closed"];

/// issue 관련 경로 묶음
pub fn routes() -> OpenApiRouter<DatabaseConnection> {
    OpenApiRouter::new()
        .routes(routes!(list, create))
        .routes(routes!(read, update, remove))
}

/// 이슈 (API 응답 형태)
#[derive(Serialize, ToSchema)]
pub struct Issue {
    sn: i64,
    project_sn: i64,
    parent_sn: Option<i64>,
    /// 화면 표시 번호 (태스크와 공용)
    num: i64,
    title: String,
    body: Option<String>,
    /// open | in_progress | done | closed
    status: String,
    source: String,
    github_number: Option<i64>,
    github_url: Option<String>,
    create_at: String,
    update_at: String,
    close_at: Option<String>,
}

impl From<i::Model> for Issue {
    fn from(m: i::Model) -> Self {
        Self {
            sn: m.sn, project_sn: m.project_sn, parent_sn: m.parent_sn, num: m.num, title: m.title, body: m.body,
            status: m.status, source: m.source, github_number: m.github_number, github_url: m.github_url,
            create_at: m.create_at, update_at: m.update_at, close_at: m.close_at,
        }
    }
}

/// 생성 요청 본문
#[derive(Deserialize, ToSchema)]
struct IssueNew {
    title: String,
    body: Option<String>,
    parent_sn: Option<i64>,
}

/// 수정 요청 본문. 보낸 필드만 바꾼다 (이벤트 payload로도 그대로 저장된다)
#[derive(Serialize, Deserialize, ToSchema)]
pub(crate) struct IssuePatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// open | in_progress | done | closed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// 프로젝트의 다음 표시 번호를 발급한다 (이슈 · 태스크 공용 · event::run 락 안에서만 부른다). 프로젝트가 없으면 404
pub async fn next_num(tx: &DatabaseTransaction, project_sn: i64) -> Res<i64> {
    let p = tbl_project::Entity::find_by_id(project_sn).one(tx).await?.ok_or_else(Error::not_found)?;
    tbl_project::Entity::update_many().filter(tbl_project::Column::Sn.eq(project_sn))
        .col_expr(tbl_project::Column::NextNum, Expr::cust("next_num + 1")).exec(tx).await?;
    Ok(p.next_num)
}

/// 1건 읽기. 없으면 404
async fn get(db: &impl ConnectionTrait, sn: i64) -> Res<Issue> {
    Tbl::find_by_id(sn).one(db).await?.map(Issue::from).ok_or_else(Error::not_found)
}

/// 프로젝트의 이슈 목록 (번호순)
#[utoipa::path(operation_id = "issue_list", get, path = "/projects/{sn}/issues", params(("sn" = i64, Path, description = "프로젝트 번호")), responses((status = 200, body = Vec<Issue>), (status = "default", body = ErrorBody)))]
async fn list(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Vec<Issue>>> {
    Ok(Json(Tbl::find().filter(i::Column::ProjectSn.eq(sn)).order_by_asc(i::Column::Num).all(&db).await?.into_iter().map(Issue::from).collect()))
}

/// 이슈 생성 (CreateIssue → IssueCreated). 프로젝트가 없으면 404
#[utoipa::path(operation_id = "issue_create", post, path = "/projects/{sn}/issues", params(("sn" = i64, Path, description = "프로젝트 번호")), request_body = IssueNew, responses((status = 201, body = Issue), (status = "default", body = ErrorBody)))]
async fn create(State(db): State<DatabaseConnection>, Sn(sn): Sn, Body(b): Body<IssueNew>) -> Res<(StatusCode, Json<Issue>)> {
    let out = event::run(&db, async |tx| {
        let num = next_num(tx, sn).await?;
        let m = i::ActiveModel {
            project_sn: Set(sn), parent_sn: Set(b.parent_sn), num: Set(num), title: Set(b.title), body: Set(b.body),
            uid: Set(Some(crate::UID)), ..Default::default()
        }.insert(tx).await?;
        let out = Issue::from(m);
        let ev = Ev::new(Some(sn), "issue", out.sn, "IssueCreated", &out);
        Ok((out, vec![ev]))
    }).await?;
    Ok((StatusCode::CREATED, Json(out)))
}

/// 1건 조회. 없으면 404
#[utoipa::path(operation_id = "issue_read", get, path = "/issues/{sn}", params(("sn" = i64, Path, description = "이슈 번호")), responses((status = 200, body = Issue), (status = "default", body = ErrorBody)))]
async fn read(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Issue>> {
    get(&db, sn).await.map(Json)
}

/// 부분 수정 (UpdateIssue → IssueUpdated). 없으면 404, 모르는 status면 422
#[utoipa::path(operation_id = "issue_update", patch, path = "/issues/{sn}", params(("sn" = i64, Path, description = "이슈 번호")), request_body = IssuePatch, responses((status = 200, body = Issue), (status = "default", body = ErrorBody)))]
async fn update(State(db): State<DatabaseConnection>, Sn(sn): Sn, Body(b): Body<IssuePatch>) -> Res<Json<Issue>> {
    if b.status.as_deref().is_some_and(|s| !STATUS.contains(&s)) {
        return Err(Error::invalid(format!("status must be one of {STATUS:?}")));
    }
    let out = event::run(&db, async |tx| patch(tx, sn, &b).await.map(|(out, ev)| (out, vec![ev]))).await?;
    Ok(Json(out))
}

/// 이슈 부분 수정 + IssueUpdated. 수정 API와 Orch 제안 실행(이슈 닫기)이 함께 쓴다. 없으면 404
pub(crate) async fn patch(tx: &DatabaseTransaction, sn: i64, b: &IssuePatch) -> Res<(Issue, Ev)> {
    let mut q = Tbl::update_many().filter(i::Column::Sn.eq(sn)).col_expr(i::Column::UpdateAt, Expr::cust("datetime('now')"));
    if let Some(v) = &b.title { q = q.col_expr(i::Column::Title, v.clone().into()); }
    if let Some(v) = &b.body { q = q.col_expr(i::Column::Body, v.clone().into()); }
    if let Some(v) = &b.status {
        // 닫히면 닫은 시각을 남기고, 다시 열리면 지운다
        q = q.col_expr(i::Column::Status, v.clone().into())
            .col_expr(i::Column::CloseAt, Expr::cust(if v == "closed" { "datetime('now')" } else { "NULL" }));
    }
    if q.exec(tx).await?.rows_affected == 0 {
        return Err(Error::not_found());
    }
    let out = get(tx, sn).await?;
    let ev = Ev::new(Some(out.project_sn), "issue", sn, "IssueUpdated", b);
    Ok((out, ev))
}

/// 삭제 (IssueDeleted). 성공 204, 없으면 404. 속한 태스크는 남고 issue_sn만 비워진다
#[utoipa::path(operation_id = "issue_remove", delete, path = "/issues/{sn}", params(("sn" = i64, Path, description = "이슈 번호")), responses((status = 204, description = "삭제됨"), (status = "default", body = ErrorBody)))]
async fn remove(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<StatusCode> {
    event::run(&db, async |tx| {
        let m = get(tx, sn).await?;
        Tbl::delete_by_id(sn).exec(tx).await?;
        Ok(((), vec![Ev::new(Some(m.project_sn), "issue", sn, "IssueDeleted", &json!({ "num": m.num }))]))
    }).await?;
    Ok(StatusCode::NO_CONTENT)
}
