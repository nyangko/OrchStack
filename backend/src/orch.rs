//! PM Dock: 프로젝트 Orch 대화(tbl_conversation · tbl_message · 첨부 조회) · 작업 제안(WorkProposal 메시지) 진행 → 이슈 · 태스크 생성,
//! Orch 제안(tbl_orch_proposal) 처리, 실행 중 지시(tbl_log_activity). Orch가 답 · 제안을 만드는 것은 실행기 Task(#13 #14) — 여기서는 저장 · 조회 · 진행만
use crate::{entity::{tbl_attachment as at, tbl_conversation as cv, tbl_issue as i, tbl_log_activity as la, tbl_member as mb, tbl_message as ms,
    tbl_orch_proposal as op, tbl_project as pj, tbl_run as r, tbl_task as t},
    error::{Body, Error, ErrorBody, Res, Sn}, event::{self, Ev}, issue::{Issue, next_num}, orch_rule::{self, By}, run, task::Task};
use axum::{Json, extract::{Query, State}, http::StatusCode};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder, sea_query::Expr};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};

/// Orch 제안 처리 경로 → 바뀌는 상태 (proposed에서만)
const RESOLVES: [(&str, &str); 4] = [("proceed", "user_done"), ("edit", "changed"), ("cancel", "stopped"), ("dismiss", "dismissed")];

/// orch 관련 경로 묶음
pub fn routes() -> OpenApiRouter<DatabaseConnection> {
    OpenApiRouter::new()
        .routes(routes!(conversation))
        .routes(routes!(post))
        .routes(routes!(edit))
        .routes(routes!(proceed))
        .routes(routes!(cancel))
        .routes(routes!(proposals))
        .routes(routes!(resolve))
        .routes(routes!(instruct))
}

/// 첨부 (API 응답 형태 · 조회만. 업로드는 별도 Task)
#[derive(Serialize, ToSchema)]
pub struct Attachment {
    sn: i64,
    file_name: String,
    mime_type: Option<String>,
    file_size: i64,
    create_at: String,
}

/// 메시지 (API 응답 형태)
#[derive(Serialize, ToSchema)]
pub struct Message {
    sn: i64,
    conversation_sn: i64,
    /// user | orch | member
    sender_type: String,
    member_sn: Option<i64>,
    /// text | work_proposal | command_result | runtime_instruction
    kind: String,
    content: Option<String>,
    /// 카드 내용 JSON (작업 제안이면 Plan)
    payload: Option<serde_json::Value>,
    /// draft | proceeded | cancelled (작업 제안만)
    proposal_status: Option<String>,
    task_sn: Option<i64>,
    run_sn: Option<i64>,
    attachments: Vec<Attachment>,
    create_at: String,
}

impl From<ms::Model> for Message {
    fn from(m: ms::Model) -> Self {
        Self {
            payload: m.payload_json.as_deref().and_then(|p| serde_json::from_str(p).ok()),
            sn: m.sn, conversation_sn: m.conversation_sn, sender_type: m.sender_type, member_sn: m.member_sn, kind: m.kind, content: m.content,
            proposal_status: m.proposal_status, task_sn: m.task_sn, run_sn: m.run_sn, attachments: Vec::new(), create_at: m.create_at,
        }
    }
}

/// 프로젝트 Orch 대화 (API 응답 형태). 대화가 아직 없으면 conversation_sn은 null · 메시지 0개
#[derive(Serialize, ToSchema)]
pub struct Conversation {
    conversation_sn: Option<i64>,
    /// 대화 상대 Orch 멤버 (팀에 Orch가 없으면 null)
    member_sn: Option<i64>,
    messages: Vec<Message>,
}

/// 메시지 보내기 요청 본문
#[derive(Deserialize, ToSchema)]
struct MessageNew {
    content: String,
    /// @Task 언급
    task_sn: Option<i64>,
    /// @Run 언급
    run_sn: Option<i64>,
}

/// 작업 제안 Plan: 이슈 1개 + 태스크 목록 (payload_json 형식)
#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct Plan {
    pub issue: PlanIssue,
    pub tasks: Vec<PlanTask>,
}

/// Plan의 이슈
#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct PlanIssue {
    pub title: String,
    pub body: Option<String>,
}

/// Plan의 태스크
#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct PlanTask {
    pub title: String,
    pub description: Option<String>,
    /// 배정할 멤버
    pub member_sn: Option<i64>,
}

/// 제안 진행 결과: 만든 이슈 · 태스크 + 결과 카드 메시지
#[derive(Serialize, ToSchema)]
struct ProceedOut {
    issue: Issue,
    tasks: Vec<Task>,
    result: Message,
}

/// Orch 제안 (API 응답 형태)
#[derive(Serialize, ToSchema)]
pub struct Proposal {
    pub(crate) sn: i64,
    project_sn: i64,
    issue_sn: Option<i64>,
    task_sn: Option<i64>,
    run_sn: Option<i64>,
    member_sn: Option<i64>,
    guard_sn: Option<i64>,
    /// assign | retry | close_issue | next_issue | fallback | guard_stop
    kind: String,
    level: i64,
    title: String,
    reason: Option<String>,
    /// 다른 선택지 JSON 배열
    options: Option<serde_json::Value>,
    /// proposed | auto_done | user_done | changed | stopped | dismissed
    status: String,
    streak_count: i64,
    deadline_at: Option<String>,
    event_sn: Option<i64>,
    create_at: String,
    resolve_at: Option<String>,
}

impl From<op::Model> for Proposal {
    fn from(m: op::Model) -> Self {
        Self {
            options: m.option_json.as_deref().and_then(|p| serde_json::from_str(p).ok()),
            sn: m.sn, project_sn: m.project_sn, issue_sn: m.issue_sn, task_sn: m.task_sn, run_sn: m.run_sn, member_sn: m.member_sn, guard_sn: m.guard_sn,
            kind: m.kind, level: m.level, title: m.title, reason: m.reason, status: m.status, streak_count: m.streak_count, deadline_at: m.deadline_at,
            event_sn: m.event_sn, create_at: m.create_at, resolve_at: m.resolve_at,
        }
    }
}

/// 제안의 다른 선택지 (option_json 배열의 원소). kind가 있으면 edit 때 그 동작을 실행하고, 없으면 고른 기록만 남긴다
#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct Opt {
    pub label: String,
    /// assign | retry | close_issue | fallback
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member_sn: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_sn: Option<i64>,
}

/// 생성할 Orch 제안 (Orch/실행기 입력)
#[derive(Deserialize, Default)]
pub struct ProposalNew {
    pub project_sn: i64,
    pub issue_sn: Option<i64>,
    pub task_sn: Option<i64>,
    pub run_sn: Option<i64>,
    pub member_sn: Option<i64>,
    pub kind: String,
    pub level: i64,
    pub title: String,
    pub reason: Option<String>,
    pub options: Option<Vec<Opt>>,
    pub streak_count: i64,
    pub deadline_at: Option<String>,
    /// 이 제안을 만든 이벤트 (#100)
    pub event_sn: Option<i64>,
    /// 걸린 루프 가드 (guard_stop일 때)
    pub guard_sn: Option<i64>,
    /// 처음 상태 (없으면 proposed · guard_stop은 stopped)
    pub status: Option<String>,
}

/// Orch 제안 목록 쿼리
#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct ProposalQuery {
    project_sn: Option<i64>,
    /// 이 상태만 (예: proposed)
    status: Option<String>,
}

/// Orch 제안 처리 요청 본문 (edit일 때 고른 다른 선택지)
#[derive(Serialize, Deserialize, ToSchema)]
struct ResolveBody {
    option: Option<String>,
}

/// 실행 중 지시 요청 본문
#[derive(Deserialize, ToSchema)]
struct InstructBody {
    text: String,
}

/// 실행 중 지시 결과 (활동 기록 1건)
#[derive(Serialize, ToSchema)]
struct Instruction {
    activity_sn: i64,
    run_sn: i64,
    member_sn: i64,
    text: String,
    create_at: String,
}

/// 프로젝트 팀의 Orch 멤버 번호. 프로젝트가 없으면 404
pub(crate) async fn orch_of(db: &impl ConnectionTrait, project_sn: i64) -> Res<Option<i64>> {
    let p = pj::Entity::find_by_id(project_sn).one(db).await?.ok_or_else(Error::not_found)?;
    let Some(team) = p.team_sn else { return Ok(None) };
    Ok(mb::Entity::find().filter(mb::Column::TeamSn.eq(team)).filter(mb::Column::IsOrch.eq(1)).filter(mb::Column::Status.ne("archived"))
        .order_by_asc(mb::Column::Sn).one(db).await?.map(|m| m.sn))
}

/// 프로젝트의 열린 Orch 대화 (가장 최근)
async fn open_of(db: &impl ConnectionTrait, project_sn: i64) -> Res<Option<cv::Model>> {
    Ok(cv::Entity::find().filter(cv::Column::ProjectSn.eq(project_sn)).filter(cv::Column::Status.eq("open")).order_by_desc(cv::Column::Sn).one(db).await?)
}

/// 대화에 메시지 1건을 넣고 대화 update_at을 갱신한다. 대화가 없으면 Orch 멤버와 새로 연다 (Orch가 없으면 409)
async fn append(tx: &DatabaseTransaction, project_sn: i64, m: ms::ActiveModel) -> Res<Message> {
    let conv = match open_of(tx, project_sn).await? {
        Some(c) => c.sn,
        None => {
            let orch = orch_of(tx, project_sn).await?.ok_or_else(|| Error::conflict("project team has no Orch member".into()))?;
            cv::ActiveModel { project_sn: Set(project_sn), member_sn: Set(orch), user_sn: Set(crate::USER), ..Default::default() }.insert(tx).await?.sn
        }
    };
    cv::Entity::update_many().filter(cv::Column::Sn.eq(conv)).col_expr(cv::Column::UpdateAt, Expr::cust("datetime('now')")).exec(tx).await?;
    Ok(ms::ActiveModel { conversation_sn: Set(conv), ..m }.insert(tx).await?.into())
}

/// 메시지의 프로젝트 번호 + 메시지. 없으면 404
async fn message(db: &impl ConnectionTrait, sn: i64) -> Res<(i64, ms::Model)> {
    let m = ms::Entity::find_by_id(sn).one(db).await?.ok_or_else(Error::not_found)?;
    let c = cv::Entity::find_by_id(m.conversation_sn).one(db).await?.ok_or_else(Error::not_found)?;
    Ok((c.project_sn, m))
}

/// 초안 작업 제안이어야 한다 (아니면 409)
fn draft(m: &ms::Model) -> Res<()> {
    if m.kind != "work_proposal" || m.proposal_status.as_deref() != Some("draft") {
        return Err(Error::conflict(format!("message {} is not a draft work proposal", m.sn)));
    }
    Ok(())
}

/// Plan 검사: 제목이 비면 422
fn check(p: &Plan) -> Res<()> {
    if p.issue.title.trim().is_empty() || p.tasks.is_empty() || p.tasks.iter().any(|t| t.title.trim().is_empty()) {
        return Err(Error::invalid("plan needs an issue title and at least one titled task".into()));
    }
    Ok(())
}

/// Orch/실행기용: Orch가 작업 제안(초안)을 대화에 올린다 (MessagePosted)
#[allow(dead_code)] // Orch · 실행기(#13 #14)가 호출한다
pub async fn propose(db: &DatabaseConnection, project_sn: i64, content: Option<String>, plan: Plan) -> Res<Message> {
    check(&plan)?;
    let orch = orch_of(db, project_sn).await?;
    event::run_as(db, "orch", orch, async |tx| {
        let out = append(tx, project_sn, ms::ActiveModel {
            sender_type: Set("orch".into()), member_sn: Set(orch), kind: Set("work_proposal".into()), content: Set(content),
            payload_json: Set(Some(json!(plan).to_string())), proposal_status: Set(Some("draft".into())), ..Default::default()
        }).await?;
        let ev = Ev::new(Some(project_sn), "message", out.sn, "MessagePosted", &json!({ "message_sn": out.sn, "kind": out.kind }));
        Ok((out, vec![ev]))
    }).await
}

/// Orch/실행기용: Orch 제안 생성 (OrchProposed). 모르는 종류 · 레벨 0~4 밖은 422
#[allow(dead_code)] // Orch · 실행기(#14)가 호출한다
pub async fn suggest(db: &DatabaseConnection, b: ProposalNew) -> Res<Proposal> {
    let orch = orch_of(db, b.project_sn).await?;
    event::run_as(db, "orch", orch, async |tx| add(tx, b).await.map(|(out, ev)| (out, vec![ev]))).await
}

/// Orch 제안 행 + OrchProposed 이벤트를 만든다 (suggest · 규칙 엔진 공용). 모르는 종류 · 레벨 0~4 밖 · 모르는 처음 상태는 422
pub(crate) async fn add(tx: &DatabaseTransaction, b: ProposalNew) -> Res<(Proposal, Ev)> {
    let status = b.status.unwrap_or_else(|| "proposed".into());
    if !["assign", "retry", "close_issue", "next_issue", "fallback", "guard_stop"].contains(&b.kind.as_str()) || !(0..=4).contains(&b.level)
        || !["proposed", "auto_done", "user_done", "changed", "stopped", "dismissed"].contains(&status.as_str()) {
        return Err(Error::invalid("unknown kind · status or level out of 0..=4".into()));
    }
    // 바로 끝난 상태로 만들면 처리 시각도 남긴다
    let done = if status == "proposed" { None } else { Some(crate::orch_rule::at(tx, "+0 seconds").await?) };
    let out = Proposal::from(op::ActiveModel {
        project_sn: Set(b.project_sn), issue_sn: Set(b.issue_sn), task_sn: Set(b.task_sn), run_sn: Set(b.run_sn), member_sn: Set(b.member_sn), guard_sn: Set(b.guard_sn),
        kind: Set(b.kind), level: Set(b.level), title: Set(b.title), reason: Set(b.reason), option_json: Set(b.options.map(|o| json!(o).to_string())),
        status: Set(status), streak_count: Set(b.streak_count), deadline_at: Set(b.deadline_at), event_sn: Set(b.event_sn), resolve_at: Set(done), ..Default::default()
    }.insert(tx).await?);
    let ev = Ev::new(Some(out.project_sn), "proposal", out.sn, "OrchProposed", &json!({ "proposal_sn": out.sn, "kind": out.kind, "title": out.title }));
    Ok((out, ev))
}

/// 프로젝트 Orch 대화 + 메시지(오래된 순 · 첨부 포함). 프로젝트가 없으면 404
#[utoipa::path(operation_id = "orch_conversation", get, path = "/projects/{sn}/conversation", params(("sn" = i64, Path, description = "프로젝트 번호")), responses((status = 200, body = Conversation), (status = "default", body = ErrorBody)))]
async fn conversation(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Conversation>> {
    let orch = orch_of(&db, sn).await?;
    let Some(c) = open_of(&db, sn).await? else { return Ok(Json(Conversation { conversation_sn: None, member_sn: orch, messages: Vec::new() })) };
    let mut messages: Vec<Message> = ms::Entity::find().filter(ms::Column::ConversationSn.eq(c.sn)).order_by_asc(ms::Column::Sn).all(&db).await?.into_iter().map(Message::from).collect();
    let files = at::Entity::find().filter(at::Column::OwnerType.eq("message")).filter(at::Column::OwnerSn.is_in(messages.iter().map(|m| m.sn))).order_by_asc(at::Column::Sn).all(&db).await?;
    for f in files {
        if let Some(m) = messages.iter_mut().find(|m| m.sn == f.owner_sn) {
            m.attachments.push(Attachment { sn: f.sn, file_name: f.file_name, mime_type: f.mime_type, file_size: f.file_size, create_at: f.create_at });
        }
    }
    Ok(Json(Conversation { conversation_sn: Some(c.sn), member_sn: Some(c.member_sn), messages }))
}

/// 사용자 메시지 보내기 (MessagePosted). 대화가 없으면 연다. 빈 본문 422 · 팀에 Orch가 없으면 409 · 프로젝트가 없으면 404
#[utoipa::path(operation_id = "orch_post", post, path = "/projects/{sn}/messages", params(("sn" = i64, Path, description = "프로젝트 번호")), request_body = MessageNew, responses((status = 201, body = Message), (status = "default", body = ErrorBody)))]
async fn post(State(db): State<DatabaseConnection>, Sn(sn): Sn, Body(b): Body<MessageNew>) -> Res<(StatusCode, Json<Message>)> {
    if b.content.trim().is_empty() {
        return Err(Error::invalid("content is empty".into()));
    }
    let out = event::run(&db, async |tx| {
        let out = append(tx, sn, ms::ActiveModel {
            sender_type: Set("user".into()), content: Set(Some(b.content)), task_sn: Set(b.task_sn), run_sn: Set(b.run_sn), ..Default::default()
        }).await?;
        let ev = Ev::new(Some(sn), "message", out.sn, "MessagePosted", &json!({ "message_sn": out.sn, "kind": out.kind }));
        Ok((out, vec![ev]))
    }).await?;
    Ok((StatusCode::CREATED, Json(out)))
}

/// 작업 제안 수정 (ProposalEdited). 초안만 (아니면 409), Plan이 비면 422
#[utoipa::path(operation_id = "orch_edit", patch, path = "/messages/{sn}", params(("sn" = i64, Path, description = "메시지 번호")), request_body = Plan, responses((status = 200, body = Message), (status = "default", body = ErrorBody)))]
async fn edit(State(db): State<DatabaseConnection>, Sn(sn): Sn, Body(b): Body<Plan>) -> Res<Json<Message>> {
    check(&b)?;
    let out = event::run(&db, async |tx| {
        let (ps, m) = message(tx, sn).await?;
        draft(&m)?;
        ms::Entity::update_many().filter(ms::Column::Sn.eq(sn)).col_expr(ms::Column::PayloadJson, json!(b).to_string().into()).exec(tx).await?;
        let out = Message::from(message(tx, sn).await?.1);
        Ok((out, vec![Ev::new(Some(ps), "message", sn, "ProposalEdited", &json!({ "message_sn": sn }))]))
    }).await?;
    Ok(Json(out))
}

/// 작업 제안 진행 (ProceedProposal → IssueCreated · TaskCreated… · ProposalProceeded). 초안만 (409).
/// Plan대로 이슈 1개 · 태스크를 만들고(배정 멤버가 있으면 orch_auto 배정), 결과 카드(command_result)를 대화에 남긴다
#[utoipa::path(operation_id = "orch_proceed", post, path = "/messages/{sn}/proceed", params(("sn" = i64, Path, description = "메시지 번호")), responses((status = 200, body = ProceedOut), (status = "default", body = ErrorBody)))]
async fn proceed(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<ProceedOut>> {
    let out = event::run(&db, async |tx| {
        let (ps, m) = message(tx, sn).await?;
        draft(&m)?;
        let plan: Plan = m.payload_json.as_deref().and_then(|p| serde_json::from_str(p).ok()).ok_or_else(|| Error::invalid("payload is not a plan".into()))?;
        let mut evs = Vec::new();
        let num = next_num(tx, ps).await?;
        let im = i::ActiveModel {
            project_sn: Set(ps), num: Set(num), title: Set(plan.issue.title.clone()), body: Set(plan.issue.body.clone()), user_sn: Set(Some(crate::USER)), ..Default::default()
        }.insert(tx).await?;
        let (issue_sn, issue) = (im.sn, Issue::from(im));
        evs.push(Ev::new(Some(ps), "issue", issue_sn, "IssueCreated", &issue));
        let (mut tasks, mut task_sns) = (Vec::new(), Vec::new());
        for x in &plan.tasks {
            let num = next_num(tx, ps).await?;
            let tm = t::ActiveModel {
                project_sn: Set(ps), issue_sn: Set(Some(issue_sn)), num: Set(num), title: Set(x.title.clone()), description: Set(x.description.clone()),
                member_sn: Set(x.member_sn), assign_by: Set(x.member_sn.map(|_| "orch_auto".into())), user_sn: Set(Some(crate::USER)), ..Default::default()
            }.insert(tx).await?;
            let (task_sn, task) = (tm.sn, Task::from(tm));
            evs.push(Ev::new(Some(ps), "task", task_sn, "TaskCreated", &task));
            tasks.push(task);
            task_sns.push(task_sn);
        }
        ms::Entity::update_many().filter(ms::Column::Sn.eq(sn)).col_expr(ms::Column::ProposalStatus, "proceeded".into()).exec(tx).await?;
        let created = json!({ "proposal_sn": sn, "issue_sn": issue_sn, "task_sns": task_sns });
        let result = append(tx, ps, ms::ActiveModel {
            sender_type: Set("orch".into()), member_sn: Set(m.member_sn), kind: Set("command_result".into()), payload_json: Set(Some(created.to_string())), ..Default::default()
        }).await?;
        evs.push(Ev::new(Some(ps), "message", sn, "ProposalProceeded", &json!({ "message_sn": sn, "result_sn": result.sn, "created": created })));
        Ok((ProceedOut { issue, tasks, result }, evs))
    }).await?;
    Ok(Json(out))
}

/// 작업 제안 취소 (ProposalCancelled). 초안만 (아니면 409)
#[utoipa::path(operation_id = "orch_cancel", post, path = "/messages/{sn}/cancel", params(("sn" = i64, Path, description = "메시지 번호")), responses((status = 200, body = Message), (status = "default", body = ErrorBody)))]
async fn cancel(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Message>> {
    let out = event::run(&db, async |tx| {
        let (ps, m) = message(tx, sn).await?;
        draft(&m)?;
        ms::Entity::update_many().filter(ms::Column::Sn.eq(sn)).col_expr(ms::Column::ProposalStatus, "cancelled".into()).exec(tx).await?;
        let out = Message::from(message(tx, sn).await?.1);
        Ok((out, vec![Ev::new(Some(ps), "message", sn, "ProposalCancelled", &json!({ "message_sn": sn }))]))
    }).await?;
    Ok(Json(out))
}

/// Orch 제안 목록 (최신순). project_sn · status로 거른다
#[utoipa::path(operation_id = "orch_proposals", get, path = "/proposals", params(ProposalQuery), responses((status = 200, body = Vec<Proposal>), (status = "default", body = ErrorBody)))]
async fn proposals(State(db): State<DatabaseConnection>, Query(q): Query<ProposalQuery>) -> Res<Json<Vec<Proposal>>> {
    let mut f = op::Entity::find();
    if let Some(v) = q.project_sn { f = f.filter(op::Column::ProjectSn.eq(v)); }
    if let Some(v) = q.status { f = f.filter(op::Column::Status.eq(v)); }
    Ok(Json(f.order_by_desc(op::Column::Sn).all(&db).await?.into_iter().map(Proposal::from).collect()))
}

/// Orch 제안 처리 (OrchProposalResolved): proceed → 실행 · user_done · edit → 고른 선택지 실행 · changed · cancel → stopped · dismiss → dismissed.
/// proposed에서만 (409). 실행은 기존 command를 그대로 쓴다 (orch_rule::run)
#[utoipa::path(operation_id = "orch_resolve", post, path = "/proposals/{sn}/{action}", params(("sn" = i64, Path, description = "제안 번호"), ("action" = String, Path, description = "proceed | edit | cancel | dismiss")), request_body = Option<ResolveBody>, responses((status = 200, body = Proposal), (status = "default", body = ErrorBody)))]
async fn resolve(State(db): State<DatabaseConnection>, axum::extract::Path((sn, action)): axum::extract::Path<(i64, String)>, raw: axum::body::Bytes) -> Res<Json<Proposal>> {
    let to = RESOLVES.iter().find(|(a, _)| *a == action).map(|(_, s)| *s).ok_or_else(Error::not_found)?;
    // 본문은 edit에만 필요 — 비어 있으면 선택지 없음
    let option = if raw.is_empty() { None } else {
        serde_json::from_slice::<ResolveBody>(&raw).map_err(|e| Error::invalid(e.to_string()))?.option
    };
    if to == "changed" && option.is_none() {
        return Err(Error::invalid("edit needs option".into()));
    }
    let out = match (to, option) {
        ("user_done", _) => orch_rule::run(&db, sn, By::User).await?,
        ("changed", Some(label)) => orch_rule::run(&db, sn, By::Pick(label)).await?,
        _ => orch_rule::end(&db, sn, to).await?,
    };
    Ok(Json(out))
}

/// 실행 중 지시 (InstructionSent). 활동 기록(TASK_INSTRUCTION · 사용자 → Run 멤버)을 남긴다. 세션 전달은 실행기(#13).
/// 끝난 Run 409 · 빈 지시 422 · 없는 Run 404
#[utoipa::path(operation_id = "orch_instruct", post, path = "/runs/{sn}/instruct", params(("sn" = i64, Path, description = "Run 번호")), request_body = InstructBody, responses((status = 201, body = Instruction), (status = "default", body = ErrorBody)))]
async fn instruct(State(db): State<DatabaseConnection>, Sn(sn): Sn, Body(b): Body<InstructBody>) -> Res<(StatusCode, Json<Instruction>)> {
    if b.text.trim().is_empty() {
        return Err(Error::invalid("text is empty".into()));
    }
    let out = event::run(&db, async |tx| {
        let run = r::Entity::find_by_id(sn).one(tx).await?.ok_or_else(Error::not_found)?;
        if !run::ACTIVE.contains(&run.status.as_str()) {
            return Err(Error::conflict(format!("run is {}", run.status)));
        }
        let a = la::ActiveModel {
            project_sn: Set(Some(run.project_sn)), task_sn: Set(Some(run.task_sn)), run_sn: Set(Some(sn)), actor_type: Set("user".into()), user_sn: Set(Some(crate::USER)),
            target_member_sn: Set(Some(run.member_sn)), kind: Set("TASK_INSTRUCTION".into()), title: Set(b.text.clone()), ..Default::default()
        }.insert(tx).await?;
        let out = Instruction { activity_sn: a.sn, run_sn: sn, member_sn: run.member_sn, text: b.text.clone(), create_at: a.create_at };
        let ev = Ev::new(Some(run.project_sn), "run", sn, "InstructionSent", &json!({ "activity_sn": a.sn, "member_sn": run.member_sn, "text": b.text }));
        Ok((out, vec![ev]))
    }).await?;
    Ok((StatusCode::CREATED, Json(out)))
}
