//! tbl_approval (위험 작업 승인 · L3/L4) 조회 · 승인 · 거부 + Orch/실행기용 생성 함수. 쓰기는 event::run 경유 (Approval* 이벤트)
use crate::{decision::Filter, entity::tbl_approval::{self as a, Entity as Tbl}, error::{Error, ErrorBody, Res, Sn}, event::{self, Ev}};
use axum::{Json, extract::{Query, State}};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, sea_query::Expr};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

/// 허용되는 승인 대상 동작
const ACTIONS: [&str; 5] = ["pr_create", "pr_merge", "dependency_add", "run_extend", "external_message"];

/// 허용되는 Approval 상태 전이 (from, to). pending에서 한 번만 끝난다. expired는 실행기 Task가 쓴다
const MOVES: &[(&str, &str)] = &[("pending", "approved"), ("pending", "denied"), ("pending", "expired")];

/// approval 관련 경로 묶음
pub fn routes() -> OpenApiRouter<DatabaseConnection> {
    OpenApiRouter::new()
        .routes(routes!(list))
        .routes(routes!(approve))
        .routes(routes!(deny))
}

/// 승인 요청 (API 응답 형태)
#[derive(Serialize, ToSchema)]
pub struct Approval {
    sn: i64,
    project_sn: i64,
    task_sn: Option<i64>,
    run_sn: Option<i64>,
    member_sn: i64,
    rule_sn: Option<i64>,
    /// pr_create | pr_merge | dependency_add | run_extend | external_message
    action_code: String,
    title: String,
    detail: Option<String>,
    /// pending | approved | denied | expired
    status: String,
    deadline_at: Option<String>,
    user_sn: Option<i64>,
    create_at: String,
    decide_at: Option<String>,
}

impl From<a::Model> for Approval {
    fn from(m: a::Model) -> Self {
        Self {
            sn: m.sn, project_sn: m.project_sn, task_sn: m.task_sn, run_sn: m.run_sn, member_sn: m.member_sn, rule_sn: m.rule_sn,
            action_code: m.action_code, title: m.title, detail: m.detail, status: m.status, deadline_at: m.deadline_at, user_sn: m.user_sn,
            create_at: m.create_at, decide_at: m.decide_at,
        }
    }
}

/// 생성할 승인 요청 (Orch/실행기 입력)
#[derive(Deserialize)]
pub struct ApprovalNew {
    pub project_sn: i64,
    pub task_sn: Option<i64>,
    pub run_sn: Option<i64>,
    pub member_sn: i64,
    pub rule_sn: Option<i64>,
    pub action_code: String,
    pub title: String,
    pub detail: Option<String>,
    pub deadline_at: Option<String>,
}

/// 1건 읽기. 없으면 404
async fn get(db: &impl ConnectionTrait, sn: i64) -> Res<Approval> {
    Tbl::find_by_id(sn).one(db).await?.map(Approval::from).ok_or_else(Error::not_found)
}

/// Orch/실행기용: 승인 요청 생성 (ApprovalRequested). 모르는 동작은 422, 없는 프로젝트 · 멤버는 422(invalid_ref)
#[allow(dead_code)] // Orch · 실행기(#13)가 호출한다
pub async fn create(db: &DatabaseConnection, b: ApprovalNew) -> Res<Approval> {
    if !ACTIONS.contains(&b.action_code.as_str()) {
        return Err(Error::invalid(format!("action_code in {ACTIONS:?}")));
    }
    let actor = event::who(db, b.member_sn).await?;
    event::run_as(db, actor, Some(b.member_sn), async |tx| {
        let m = a::ActiveModel {
            project_sn: Set(b.project_sn), task_sn: Set(b.task_sn), run_sn: Set(b.run_sn), member_sn: Set(b.member_sn), rule_sn: Set(b.rule_sn),
            action_code: Set(b.action_code), title: Set(b.title), detail: Set(b.detail), deadline_at: Set(b.deadline_at), ..Default::default()
        }.insert(tx).await?;
        let out = Approval::from(m);
        let ev = Ev::new(Some(out.project_sn), "approval", out.sn, "ApprovalRequested", &json!({ "task_sn": out.task_sn, "action_code": out.action_code, "title": out.title }));
        Ok((out, vec![ev]))
    }).await
}

/// pending → to (approved · denied). 처리자 · 시각을 남긴다. 표에 없는 전이면 409
async fn decide(db: &DatabaseConnection, sn: i64, to: &'static str, kind: &'static str) -> Res<Approval> {
    event::run(db, async |tx| {
        let cur = get(tx, sn).await?;
        if !MOVES.contains(&(cur.status.as_str(), to)) {
            return Err(Error::conflict(format!("cannot move approval {} -> {to}", cur.status)));
        }
        Tbl::update_many().filter(a::Column::Sn.eq(sn)).col_expr(a::Column::Status, to.into()).col_expr(a::Column::UserSn, crate::USER.into())
            .col_expr(a::Column::DecideAt, Expr::cust("datetime('now')")).exec(tx).await?;
        let out = get(tx, sn).await?;
        Ok((out, vec![Ev::new(Some(cur.project_sn), "approval", sn, kind, &json!({ "task_sn": cur.task_sn, "action_code": cur.action_code }))]))
    }).await
}

/// 승인 요청 목록 (최신순). status · project_sn · task_sn으로 거른다
#[utoipa::path(operation_id = "approval_list", get, path = "/approvals", params(Filter), responses((status = 200, body = Vec<Approval>), (status = "default", body = ErrorBody)))]
async fn list(State(db): State<DatabaseConnection>, Query(f): Query<Filter>) -> Res<Json<Vec<Approval>>> {
    let mut s = Tbl::find();
    if let Some(v) = f.status { s = s.filter(a::Column::Status.eq(v)); }
    if let Some(v) = f.project_sn { s = s.filter(a::Column::ProjectSn.eq(v)); }
    if let Some(v) = f.task_sn { s = s.filter(a::Column::TaskSn.eq(v)); }
    Ok(Json(s.order_by_desc(a::Column::Sn).all(&db).await?.into_iter().map(Approval::from).collect()))
}

/// 승인 (ApprovalApproved). pending이 아니면 409
#[utoipa::path(operation_id = "approval_approve", post, path = "/approvals/{sn}/approve", params(("sn" = i64, Path, description = "승인 요청 번호")), responses((status = 200, body = Approval), (status = "default", body = ErrorBody)))]
async fn approve(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Approval>> {
    decide(&db, sn, "approved", "ApprovalApproved").await.map(Json)
}

/// 거부 (ApprovalDenied). pending이 아니면 409
#[utoipa::path(operation_id = "approval_deny", post, path = "/approvals/{sn}/deny", params(("sn" = i64, Path, description = "승인 요청 번호")), responses((status = 200, body = Approval), (status = "default", body = ErrorBody)))]
async fn deny(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Approval>> {
    decide(&db, sn, "denied", "ApprovalDenied").await.map(Json)
}
