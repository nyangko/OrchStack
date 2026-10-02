//! tbl_ask: Orch가 사람에게 묻거나 알리는 요청 하나. 판단 요청(decision · 질문 · 선택지 · 답) · 승인 요청(approval) · Orch 제안(proposal)을
//! 같은 테이블 · 같은 경로(/asks)로 다룬다. kind에 안 맞는 동작은 409. 쓰기는 event::run_as 경유 (Decision* · Approval* · OrchPropos* 이벤트 · 대상 ask).
//! 제안 실행(`run`)은 기존 command(배정 · 재시도 · 이슈 닫기 · 폴백 표시)를 그대로 부른다 — 새 UPDATE 경로 없음
use crate::{policy, entity::{tbl_ask as a, tbl_run as r, tbl_task as t},
    error::{Body, Error, ErrorBody, Res, Sn}, event::{self, Ev}, issue::{self, IssuePatch}, orch, orch_rule::{self, at}, run as runs, task};
use axum::{Json, body::Bytes, extract::{Query, State}};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder, sea_query::Expr};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashSet;
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};

/// 허용되는 상태 전이 (kind, from, to). 판단 · 승인은 끝 상태에서 나가지 않는다. orch_decided · cancelled · expired는 실행기 Task가 쓴다 (제안은 `settle`)
const MOVES: &[(&str, &str, &str)] = &[
    ("decision", "pending", "writing"), ("decision", "pending", "answered"), ("decision", "pending", "orch_decided"), ("decision", "pending", "cancelled"),
    ("decision", "writing", "answered"), ("decision", "writing", "orch_decided"), ("decision", "writing", "cancelled"),
    ("approval", "pending", "approved"), ("approval", "pending", "denied"), ("approval", "pending", "expired"),
];
/// 승인 대상 동작
const ACTIONS: [&str; 5] = ["pr_create", "pr_merge", "dependency_add", "run_extend", "external_message"];
/// 제안 종류
const PROPOSALS: [&str; 6] = ["assign", "retry", "close_issue", "next_issue", "fallback", "guard_stop"];
/// 제안 처리 경로 → 바뀌는 상태 (proposed에서만)
const RESOLVES: [(&str, &str); 4] = [("proceed", "user_done"), ("edit", "changed"), ("cancel", "stopped"), ("dismiss", "dismissed")];

/// ask 관련 경로 묶음
pub fn routes() -> OpenApiRouter<DatabaseConnection> {
    OpenApiRouter::new()
        .routes(routes!(list))
        .routes(routes!(read))
        .routes(routes!(answer))
        .routes(routes!(writing))
        .routes(routes!(approve))
        .routes(routes!(deny))
        .routes(routes!(hold))
        .routes(routes!(resolve))
}

/// 선택지 (판단 요청 · 질문 안)
#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct Choice {
    /// 표시 기호 (A, B, C) · 질문 안에서 선택지를 가리키는 키
    pub code: String,
    pub label: String,
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default)]
    pub is_recommended: bool,
    /// 사용자가 고른 선택지 (질문당 1개)
    #[serde(default)]
    pub is_selected: bool,
}

/// 질문 (판단 요청 안 · 순서가 번호다)
#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct Question {
    pub title: String,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub code_snippet: Option<String>,
    /// 참고 자료 (파일 · 줄 · 태스크)
    #[serde(default, rename = "ref")]
    pub reference: Option<Value>,
    pub options: Vec<Choice>,
    /// 덧붙인 말 또는 직접 쓴 답
    #[serde(default)]
    pub answer_text: Option<String>,
    /// 이 질문은 Orch에게 맡김
    #[serde(default)]
    pub is_delegate: bool,
    #[serde(default)]
    pub answer_at: Option<String>,
}

/// 제안의 다른 선택지 (제안의 option_json 배열 원소). kind가 있으면 edit 때 그 동작을 실행하고, 없으면 고른 기록만 남긴다
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

/// 요청 (API 응답 형태). option = option_json을 파싱한 값 (판단 = 질문 배열 · 승인 = {rule_title, detail} · 제안 = 다른 선택지 배열)
#[derive(Serialize, ToSchema)]
pub struct Ask {
    pub sn: i64,
    pub project_sn: i64,
    pub issue_sn: Option<i64>,
    pub task_sn: Option<i64>,
    pub run_sn: Option<i64>,
    pub member_sn: Option<i64>,
    /// decision | approval | proposal
    pub kind: String,
    /// 승인 = pr_create | pr_merge | dependency_add | run_extend | external_message · 제안 = assign | retry | close_issue | next_issue | fallback | guard_stop
    pub action: Option<String>,
    pub level: i64,
    pub title: String,
    pub reason: Option<String>,
    pub option: Option<Value>,
    /// 판단 = pending | writing | answered | orch_decided | cancelled · 승인 = pending | approved | denied | expired · 제안 = proposed | auto_done | user_done | changed | stopped | dismissed
    pub status: String,
    pub is_timer_pause: i64,
    pub is_review_needed: i64,
    pub deadline_at: Option<String>,
    /// user | orch
    pub decide_by: Option<String>,
    pub user_sn: Option<i64>,
    pub streak_count: i64,
    pub guard_code: Option<String>,
    pub event_sn: Option<i64>,
    pub create_at: String,
    pub decide_at: Option<String>,
}

impl From<a::Model> for Ask {
    fn from(m: a::Model) -> Self {
        Self {
            option: m.option_json.as_deref().and_then(|p| serde_json::from_str(p).ok()),
            sn: m.sn, project_sn: m.project_sn, issue_sn: m.issue_sn, task_sn: m.task_sn, run_sn: m.run_sn, member_sn: m.member_sn, kind: m.kind, action: m.action,
            level: m.level, title: m.title, reason: m.reason, status: m.status, is_timer_pause: m.is_timer_pause, is_review_needed: m.is_review_needed,
            deadline_at: m.deadline_at, decide_by: m.decide_by, user_sn: m.user_sn, streak_count: m.streak_count, guard_code: m.guard_code, event_sn: m.event_sn,
            create_at: m.create_at, decide_at: m.decide_at,
        }
    }
}

/// 목록 쿼리. 범위는 전체(없음) · 프로젝트 · 태스크 — DecisionPanel scope
#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct Filter {
    /// decision | approval | proposal
    pub kind: Option<String>,
    /// 이 상태만 (예: pending · proposed)
    pub status: Option<String>,
    pub project_sn: Option<i64>,
    pub task_sn: Option<i64>,
}

/// 생성할 판단 요청 (Orch/실행기 입력)
#[derive(Deserialize)]
pub struct DecisionNew {
    pub project_sn: i64,
    pub task_sn: Option<i64>,
    pub run_sn: Option<i64>,
    pub member_sn: i64,
    /// 작업 레벨 2~4 (L2 이상만 사람에게 묻는다)
    pub level: i64,
    pub title: String,
    pub deadline_at: Option<String>,
    pub questions: Vec<Question>,
}

/// 생성할 승인 요청 (Orch/실행기 입력)
#[derive(Deserialize)]
pub struct ApprovalNew {
    pub project_sn: i64,
    pub task_sn: Option<i64>,
    pub run_sn: Option<i64>,
    pub member_sn: i64,
    /// 걸린 승인 규칙 이름
    pub rule_title: Option<String>,
    pub action_code: String,
    pub title: String,
    pub detail: Option<String>,
    pub deadline_at: Option<String>,
}

/// 생성할 Orch 제안 (Orch/실행기 · 규칙 엔진 입력)
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
    /// 걸린 루프 가드 코드 (guard_stop일 때)
    pub guard_code: Option<String>,
    /// 처음 상태 (없으면 proposed · guard_stop은 stopped)
    pub status: Option<String>,
}

/// 질문 하나의 답. 선택지 · 직접 쓴 말 · Orch에게 맡김 중 하나 이상
#[derive(Serialize, Deserialize, ToSchema)]
struct Answer {
    /// 질문 순번 (0부터 · option 배열 index)
    question: usize,
    /// 고른 선택지 기호 (code)
    option: Option<String>,
    /// 덧붙인 말 또는 직접 쓴 답
    text: Option<String>,
    /// 이 질문은 Orch에게 맡김
    #[serde(default)]
    delegate: bool,
}

/// 답변 요청 본문. 판단 요청의 모든 질문에 답한다
#[derive(Serialize, Deserialize, ToSchema)]
struct AnswerBody {
    answers: Vec<Answer>,
    /// 나중에 다시 보기로 표시
    #[serde(default)]
    review_needed: bool,
}

/// 제안 처리 요청 본문 (edit일 때 고른 다른 선택지)
#[derive(Serialize, Deserialize, ToSchema)]
struct ResolveBody {
    /// 고른 선택지의 label
    option: Option<String>,
}

/// 제안 대기 연장 요청 본문
#[derive(Deserialize, ToSchema)]
struct HoldBody {
    /// 새 대기(초) · 없으면 정책의 timer_sec
    sec: Option<i64>,
}

/// 제안을 처리하는 쪽: 사용자 proceed · 타이머 · 사용자가 고른 다른 선택지(label)
pub enum By {
    User,
    Auto,
    Pick(String),
}

/// 행 1건 읽기. 없으면 404
async fn row(db: &impl ConnectionTrait, sn: i64) -> Res<a::Model> {
    a::Entity::find_by_id(sn).one(db).await?.ok_or_else(Error::not_found)
}

/// 이 종류의 행 1건 읽기. 없으면 404, 다른 종류면 409
async fn of(db: &impl ConnectionTrait, sn: i64, kind: &str) -> Res<a::Model> {
    let m = row(db, sn).await?;
    if m.kind != kind {
        return Err(Error::conflict(format!("ask {sn} is a {}, not a {kind}", m.kind)));
    }
    Ok(m)
}

/// 상태를 to로 옮긴다 (kind · 표에 없는 전이면 409). 바뀌기 전 행을 돌려준다
async fn step(tx: &DatabaseTransaction, sn: i64, kind: &str, to: &str) -> Res<a::Model> {
    let cur = of(tx, sn, kind).await?;
    if !MOVES.contains(&(kind, cur.status.as_str(), to)) {
        return Err(Error::conflict(format!("cannot move {kind} {} -> {to}", cur.status)));
    }
    a::Entity::update_many().filter(a::Column::Sn.eq(sn)).col_expr(a::Column::Status, to.to_owned().into()).exec(tx).await?;
    Ok(cur)
}

/// 요청 1건 (응답 모양). 없으면 404
async fn get(db: &impl ConnectionTrait, sn: i64) -> Res<Ask> {
    row(db, sn).await.map(Ask::from)
}

/// 판단 요청의 질문 배열 (저장된 JSON)
fn questions(m: &a::Model) -> Vec<Question> {
    m.option_json.as_deref().and_then(|j| serde_json::from_str(j).ok()).unwrap_or_default()
}

/// 승인 요청의 상세 설명 (option_json.detail · 알림 본문)
pub fn detail(m: &a::Model) -> Option<String> {
    let v: Value = serde_json::from_str(m.option_json.as_deref()?).ok()?;
    v["detail"].as_str().map(str::to_owned)
}

/// Orch/실행기용: 판단 요청 생성 (DecisionRequested). 레벨 2~4 · 질문 1개 이상 아니면 422, 없는 프로젝트 · 멤버는 422(invalid_ref)
#[allow(dead_code)] // Orch · 실행기(#13)가 호출한다
pub async fn decision(db: &DatabaseConnection, b: DecisionNew) -> Res<Ask> {
    if !(2..=4).contains(&b.level) || b.questions.is_empty() || b.questions.iter().any(|q| q.options.is_empty()) {
        return Err(Error::invalid("level must be 2..=4 and every question needs options".into()));
    }
    let actor = event::who(db, b.member_sn).await?;
    event::run_as(db, actor, Some(b.member_sn), async |tx| {
        let qs: Vec<Question> = b.questions.into_iter().map(|q| Question {
            answer_text: None, is_delegate: false, answer_at: None,
            options: q.options.into_iter().map(|c| Choice { is_selected: false, ..c }).collect(), ..q
        }).collect();
        let m = a::ActiveModel {
            project_sn: Set(b.project_sn), task_sn: Set(b.task_sn), run_sn: Set(b.run_sn), member_sn: Set(Some(b.member_sn)), kind: Set("decision".into()),
            level: Set(b.level), title: Set(b.title), option_json: Set(Some(json!(qs).to_string())), deadline_at: Set(b.deadline_at), ..Default::default()
        }.insert(tx).await?;
        let out = Ask::from(m);
        let ev = Ev::new(Some(out.project_sn), "ask", out.sn, "DecisionRequested", &json!({ "task_sn": out.task_sn, "level": out.level, "title": out.title }));
        Ok((out, vec![ev]))
    }).await
}

/// Orch/실행기용: 승인 요청 생성 (ApprovalRequested). 모르는 동작은 422, 없는 프로젝트 · 멤버는 422(invalid_ref)
#[allow(dead_code)] // Orch · 실행기(#13)가 호출한다
pub async fn approval(db: &DatabaseConnection, b: ApprovalNew) -> Res<Ask> {
    if !ACTIONS.contains(&b.action_code.as_str()) {
        return Err(Error::invalid(format!("action_code in {ACTIONS:?}")));
    }
    let actor = event::who(db, b.member_sn).await?;
    event::run_as(db, actor, Some(b.member_sn), async |tx| {
        let m = a::ActiveModel {
            project_sn: Set(b.project_sn), task_sn: Set(b.task_sn), run_sn: Set(b.run_sn), member_sn: Set(Some(b.member_sn)), kind: Set("approval".into()),
            action: Set(Some(b.action_code)), level: Set(3), title: Set(b.title), option_json: Set(Some(json!({ "rule_title": b.rule_title, "detail": b.detail }).to_string())),
            deadline_at: Set(b.deadline_at), ..Default::default()
        }.insert(tx).await?;
        let out = Ask::from(m);
        let ev = Ev::new(Some(out.project_sn), "ask", out.sn, "ApprovalRequested", &json!({ "task_sn": out.task_sn, "action_code": out.action, "title": out.title }));
        Ok((out, vec![ev]))
    }).await
}

/// Orch/실행기용: Orch 제안 생성 (OrchProposed). 모르는 종류 · 레벨 0~4 밖은 422
#[allow(dead_code)] // Orch · 실행기(#14)가 호출한다
pub async fn suggest(db: &DatabaseConnection, b: ProposalNew) -> Res<Ask> {
    let orch = orch::orch_of(db, b.project_sn).await?;
    event::run_as(db, "orch", orch, async |tx| add(tx, b).await.map(|(out, ev)| (out, vec![ev]))).await
}

/// Orch 제안 행 + OrchProposed 이벤트를 만든다 (suggest · 규칙 엔진 공용). 모르는 종류 · 레벨 0~4 밖 · 모르는 처음 상태는 422
pub(crate) async fn add(tx: &DatabaseTransaction, b: ProposalNew) -> Res<(Ask, Ev)> {
    let status = b.status.unwrap_or_else(|| "proposed".into());
    if !PROPOSALS.contains(&b.kind.as_str()) || !(0..=4).contains(&b.level) || !["proposed", "auto_done", "user_done", "changed", "stopped", "dismissed"].contains(&status.as_str()) {
        return Err(Error::invalid("unknown kind · status or level out of 0..=4".into()));
    }
    // 바로 끝난 상태로 만들면 처리 시각도 남긴다
    let done = if status == "proposed" { None } else { Some(at(tx, "+0 seconds").await?) };
    let out = Ask::from(a::ActiveModel {
        project_sn: Set(b.project_sn), issue_sn: Set(b.issue_sn), task_sn: Set(b.task_sn), run_sn: Set(b.run_sn), member_sn: Set(b.member_sn), guard_code: Set(b.guard_code),
        kind: Set("proposal".into()), action: Set(Some(b.kind)), level: Set(b.level), title: Set(b.title), reason: Set(b.reason), option_json: Set(b.options.map(|o| json!(o).to_string())),
        status: Set(status), streak_count: Set(b.streak_count), deadline_at: Set(b.deadline_at), event_sn: Set(b.event_sn), decide_at: Set(done), ..Default::default()
    }.insert(tx).await?);
    let ev = Ev::new(Some(out.project_sn), "ask", out.sn, "OrchProposed", &json!({ "proposal_sn": out.sn, "kind": out.action, "title": out.title }));
    Ok((out, ev))
}

/// 요청 목록 (최신순). kind · status · project_sn · task_sn으로 거른다
#[utoipa::path(operation_id = "ask_list", get, path = "/asks", params(Filter), responses((status = 200, body = Vec<Ask>), (status = "default", body = ErrorBody)))]
async fn list(State(db): State<DatabaseConnection>, Query(f): Query<Filter>) -> Res<Json<Vec<Ask>>> {
    let mut s = a::Entity::find();
    if let Some(v) = f.kind { s = s.filter(a::Column::Kind.eq(v)); }
    if let Some(v) = f.status { s = s.filter(a::Column::Status.eq(v)); }
    if let Some(v) = f.project_sn { s = s.filter(a::Column::ProjectSn.eq(v)); }
    if let Some(v) = f.task_sn { s = s.filter(a::Column::TaskSn.eq(v)); }
    Ok(Json(s.order_by_desc(a::Column::Sn).all(&db).await?.into_iter().map(Ask::from).collect()))
}

/// 요청 1건 (판단이면 질문 · 선택지 · 답이 option에 있다). 없으면 404
#[utoipa::path(operation_id = "ask_read", get, path = "/asks/{sn}", params(("sn" = i64, Path, description = "요청 번호")), responses((status = 200, body = Ask), (status = "default", body = ErrorBody)))]
async fn read(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Ask>> {
    get(&db, sn).await.map(Json)
}

/// 판단 답변 (AnswerDecision → DecisionAnswered). 판단의 pending · writing에서만 (아니면 409).
/// 모든 질문에 정확히 한 번씩, 각 답은 선택지 · 말 · 맡김 중 하나 이상, 선택지는 그 질문의 것 — 아니면 422
#[utoipa::path(operation_id = "ask_answer", post, path = "/asks/{sn}/answer", params(("sn" = i64, Path, description = "요청 번호")), request_body = AnswerBody, responses((status = 200, body = Ask), (status = "default", body = ErrorBody)))]
async fn answer(State(db): State<DatabaseConnection>, Sn(sn): Sn, Body(b): Body<AnswerBody>) -> Res<Json<Ask>> {
    let out = event::run(&db, async |tx| {
        let cur = step(tx, sn, "decision", "answered").await?;
        let mut qs = questions(&cur);
        let got: HashSet<usize> = b.answers.iter().map(|x| x.question).collect();
        if got != (0..qs.len()).collect() || got.len() != b.answers.len() || b.answers.iter().any(|x| x.option.is_none() && x.text.is_none() && !x.delegate) {
            return Err(Error::invalid("answer every question once with option, text or delegate".into()));
        }
        let now = at(tx, "+0 seconds").await?;
        for x in &b.answers {
            let q = &mut qs[x.question];
            if let Some(code) = &x.option && !q.options.iter().any(|c| &c.code == code) {
                return Err(Error::invalid(format!("option {code} is not in question {}", x.question)));
            }
            // 질문당 선택 1개: 전부 끄고 고른 것만 켠다
            for c in &mut q.options { c.is_selected = x.option.as_deref() == Some(c.code.as_str()); }
            (q.answer_text, q.is_delegate, q.answer_at) = (x.text.clone(), x.delegate, Some(now.clone()));
        }
        a::Entity::update_many().filter(a::Column::Sn.eq(sn)).col_expr(a::Column::OptionJson, json!(qs).to_string().into()).col_expr(a::Column::DecideBy, "user".into())
            .col_expr(a::Column::UserSn, crate::USER.into()).col_expr(a::Column::IsTimerPause, 0.into()).col_expr(a::Column::IsReviewNeeded, i64::from(b.review_needed).into())
            .col_expr(a::Column::DecideAt, Expr::cust("datetime('now')")).exec(tx).await?;
        let out = get(tx, sn).await?;
        Ok((out, vec![Ev::new(Some(cur.project_sn), "ask", sn, "DecisionAnswered", &json!({ "task_sn": cur.task_sn, "from": cur.status, "answers": b.answers, "review_needed": b.review_needed }))]))
    }).await?;
    Ok(Json(out))
}

/// 작성 중 (DecisionWriting). 판단의 pending → writing, 타이머를 멈춘다. pending이 아니면 409
#[utoipa::path(operation_id = "ask_writing", post, path = "/asks/{sn}/writing", params(("sn" = i64, Path, description = "요청 번호")), responses((status = 200, body = Ask), (status = "default", body = ErrorBody)))]
async fn writing(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Ask>> {
    let out = event::run(&db, async |tx| {
        let cur = step(tx, sn, "decision", "writing").await?;
        a::Entity::update_many().filter(a::Column::Sn.eq(sn)).col_expr(a::Column::IsTimerPause, 1.into()).exec(tx).await?;
        let out = get(tx, sn).await?;
        Ok((out, vec![Ev::new(Some(cur.project_sn), "ask", sn, "DecisionWriting", &json!({ "task_sn": cur.task_sn }))]))
    }).await?;
    Ok(Json(out))
}

/// 승인 요청 pending → to (approved · denied). 처리자 · 시각을 남긴다. 승인이 아니거나 표에 없는 전이면 409
async fn decide(db: &DatabaseConnection, sn: i64, to: &'static str, kind: &'static str) -> Res<Ask> {
    event::run(db, async |tx| {
        let cur = step(tx, sn, "approval", to).await?;
        a::Entity::update_many().filter(a::Column::Sn.eq(sn)).col_expr(a::Column::DecideBy, "user".into()).col_expr(a::Column::UserSn, crate::USER.into())
            .col_expr(a::Column::DecideAt, Expr::cust("datetime('now')")).exec(tx).await?;
        let out = get(tx, sn).await?;
        Ok((out, vec![Ev::new(Some(cur.project_sn), "ask", sn, kind, &json!({ "task_sn": cur.task_sn, "action_code": cur.action }))]))
    }).await
}

/// 승인 (ApprovalApproved). 승인 요청의 pending이 아니면 409
#[utoipa::path(operation_id = "ask_approve", post, path = "/asks/{sn}/approve", params(("sn" = i64, Path, description = "요청 번호")), responses((status = 200, body = Ask), (status = "default", body = ErrorBody)))]
async fn approve(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Ask>> {
    decide(&db, sn, "approved", "ApprovalApproved").await.map(Json)
}

/// 거부 (ApprovalDenied). 승인 요청의 pending이 아니면 409
#[utoipa::path(operation_id = "ask_deny", post, path = "/asks/{sn}/deny", params(("sn" = i64, Path, description = "요청 번호")), responses((status = 200, body = Ask), (status = "default", body = ErrorBody)))]
async fn deny(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Ask>> {
    decide(&db, sn, "denied", "ApprovalDenied").await.map(Json)
}

/// 제안 처리 (OrchProposalResolved): proceed → 실행 · user_done · edit → 고른 선택지 실행 · changed · cancel → stopped · dismiss → dismissed.
/// 제안의 proposed에서만 (아니면 409). 실행은 기존 command를 그대로 쓴다 (`run`)
#[utoipa::path(operation_id = "ask_resolve", post, path = "/asks/{sn}/{action}", params(("sn" = i64, Path, description = "요청 번호"), ("action" = String, Path, description = "proceed | edit | cancel | dismiss")), request_body = Option<ResolveBody>, responses((status = 200, body = Ask), (status = "default", body = ErrorBody)))]
async fn resolve(State(db): State<DatabaseConnection>, axum::extract::Path((sn, action)): axum::extract::Path<(i64, String)>, raw: Bytes) -> Res<Json<Ask>> {
    let to = RESOLVES.iter().find(|(x, _)| *x == action).map(|(_, s)| *s).ok_or_else(Error::not_found)?;
    // 본문은 edit에만 필요 — 비어 있으면 선택지 없음
    let option = if raw.is_empty() { None } else {
        serde_json::from_slice::<ResolveBody>(&raw).map_err(|e| Error::invalid(e.to_string()))?.option
    };
    if to == "changed" && option.is_none() {
        return Err(Error::invalid("edit needs option".into()));
    }
    of(&db, sn, "proposal").await?;
    let out = match (to, option) {
        ("user_done", _) => run(&db, sn, By::User).await?,
        ("changed", Some(label)) => run(&db, sn, By::Pick(label)).await?,
        _ => end(&db, sn, to).await?,
    };
    Ok(Json(out))
}

/// 자동 진행 대기 연장 (사용자가 카드를 보고 있을 때 · is_pause_on_view). 기한이 있는 제안의 proposed만 (아니면 409). 이벤트는 남기지 않는다 (화면 타이머 상태)
#[utoipa::path(operation_id = "ask_hold", post, path = "/asks/{sn}/hold", params(("sn" = i64, Path, description = "요청 번호")), request_body = Option<HoldBody>, responses((status = 200, body = Ask), (status = "default", body = ErrorBody)))]
async fn hold(State(db): State<DatabaseConnection>, Sn(sn): Sn, raw: Bytes) -> Res<Json<Ask>> {
    let sec = if raw.is_empty() { None } else { serde_json::from_slice::<HoldBody>(&raw).map_err(|x| Error::invalid(x.to_string()))?.sec };
    let cur = of(&db, sn, "proposal").await?;
    if cur.status != "proposed" || cur.deadline_at.is_none() {
        return Err(Error::conflict("nothing to hold".into()));
    }
    let sec = match sec {
        Some(s) => s,
        None => policy::load(&db, cur.project_sn).await?.map_or(5, |p| p.timer_sec),
    };
    if !(1..=86_400).contains(&sec) {
        return Err(Error::invalid("sec must be 1..=86400".into()));
    }
    let to = at(&db, &format!("+{sec} seconds")).await?;
    let out = event::run(&db, async |tx| {
        a::Entity::update_many().filter(a::Column::Sn.eq(sn)).filter(a::Column::Status.eq("proposed")).col_expr(a::Column::DeadlineAt, to.into()).exec(tx).await?;
        Ok((get(tx, sn).await?, vec![]))
    }).await?;
    Ok(Json(out))
}

/// 제안 실행: kind(action)별로 기존 command를 부른다 (새 UPDATE 경로 없음). next_issue · guard_stop · 선택지만 있는 것은 확인만
async fn act(tx: &DatabaseTransaction, kind: &str, tk: Option<i64>, member: Option<i64>, run_sn: Option<i64>, issue_sn: Option<i64>) -> Res<Vec<Ev>> {
    let need = |v: Option<i64>| v.ok_or_else(|| Error::invalid(format!("{kind} needs a target")));
    Ok(match kind {
        "assign" => {
            let (ts, ms) = (need(tk)?, need(member)?);
            let cur = t::Entity::find_by_id(ts).one(tx).await?.ok_or_else(Error::not_found)?;
            if cur.status != "todo" || cur.member_sn.is_some() {
                return Err(Error::conflict(format!("task {ts} is no longer an open todo")));
            }
            vec![task::put_member(tx, ts, Some(ms), "orch_auto").await?.1]
        }
        "retry" => vec![runs::redo(tx, need(run_sn)?).await?.1],
        "close_issue" => vec![issue::patch(tx, need(issue_sn)?, &IssuePatch { title: None, body: None, status: Some("closed".into()) }).await?.1],
        "fallback" => {
            let rn = r::Entity::find_by_id(need(run_sn)?).one(tx).await?.ok_or_else(Error::not_found)?;
            let next = orch_rule::next_conn(tx, &rn).await?.ok_or_else(|| Error::conflict("no fallback connection".into()))?;
            vec![runs::switch(tx, rn.sn, next.sn).await?]
        }
        _ => vec![],
    })
}

/// 제안 상태를 `to`로 마무리 (proposed에서만 · 아니면 409) + OrchProposalResolved. why가 있으면 근거에 덧붙인다
async fn settle(tx: &DatabaseTransaction, cur: &a::Model, to: &str, user: bool, streak: i64, why: Option<String>, option: Option<&str>) -> Res<(Ask, Ev)> {
    let reason = match (&cur.reason, why) { (Some(x), Some(y)) => Some(format!("{x} | {y}")), (x, y) => y.or(x.clone()) };
    let mut q = a::Entity::update_many().filter(a::Column::Sn.eq(cur.sn)).filter(a::Column::Status.eq("proposed")).col_expr(a::Column::Status, to.to_owned().into())
        .col_expr(a::Column::StreakCount, streak.into()).col_expr(a::Column::Reason, reason.into()).col_expr(a::Column::DecideAt, Expr::cust("datetime('now')"));
    if user {
        q = q.col_expr(a::Column::UserSn, crate::USER.into());
    }
    if q.exec(tx).await?.rows_affected == 0 {
        return Err(Error::conflict(format!("proposal {} is not proposed", cur.sn)));
    }
    let out = get(tx, cur.sn).await?;
    let ev = Ev::new(Some(cur.project_sn), "ask", cur.sn, "OrchProposalResolved", &json!({ "proposal_sn": cur.sn, "status": to, "option": option }));
    Ok((out, ev))
}

/// 제안 하나를 실행하고 마무리하는 트랜잭션 본문
async fn go(tx: &DatabaseTransaction, sn: i64, by: &By) -> Res<(Ask, Vec<Ev>)> {
    let cur = of(tx, sn, "proposal").await?;
    if cur.status != "proposed" {
        return Err(Error::conflict(format!("proposal is {}", cur.status)));
    }
    let (kind, tk, member, label) = match by {
        By::Pick(label) => {
            let opts: Vec<Opt> = cur.option_json.as_deref().and_then(|j| serde_json::from_str(j).ok()).unwrap_or_default();
            let o = opts.into_iter().find(|o| &o.label == label).ok_or_else(|| Error::invalid(format!("unknown option {label}")))?;
            (o.kind.unwrap_or_default(), o.task_sn.or(cur.task_sn), o.member_sn.or(cur.member_sn), Some(label.as_str()))
        }
        _ => (cur.action.clone().unwrap_or_default(), cur.task_sn, cur.member_sn, None),
    };
    let mut evs = act(tx, &kind, tk, member, cur.run_sn, cur.issue_sn).await?;
    let (to, n) = match by {
        By::User => ("user_done", 0),
        By::Pick(_) => ("changed", 0),
        By::Auto => ("auto_done", 1 + orch_rule::streak(tx, &orch_rule::projects_of(tx, cur.project_sn).await?).await?),
    };
    let (out, ev) = settle(tx, &cur, to, !matches!(by, By::Auto), n, None, label).await?;
    evs.push(ev);
    Ok((out, evs))
}

/// 제안 실행 (user_done · auto_done · changed). 자동(by = Auto)은 행위자 orch, 사용자는 user.
/// 자동 실행이 실패하면(409 등) dismissed + 사유로 닫고, 사용자 실행은 오류를 그대로 돌려주며 제안은 proposed로 남는다
pub async fn run(db: &DatabaseConnection, sn: i64, by: By) -> Res<Ask> {
    let cur = of(db, sn, "proposal").await?;
    let auto = matches!(by, By::Auto);
    let (actor, member) = if auto { ("orch", orch::orch_of(db, cur.project_sn).await?) } else { ("user", None) };
    match event::run_as(db, actor, member, async |tx| go(tx, sn, &by).await).await {
        Err(err) if auto => {
            let why = format!("failed: {}", err.message());
            event::run_as(db, "orch", member, async |tx| settle(tx, &cur, "dismissed", false, 0, Some(why), None).await.map(|(out, ev)| (out, vec![ev]))).await
        }
        res => res,
    }
}

/// 실행 없이 닫기 (cancel → stopped · dismiss → dismissed). 제안의 proposed에서만 409
pub async fn end(db: &DatabaseConnection, sn: i64, to: &str) -> Res<Ask> {
    event::run(db, async |tx| {
        let cur = of(tx, sn, "proposal").await?;
        settle(tx, &cur, to, true, 0, None, None).await.map(|(out, ev)| (out, vec![ev]))
    }).await
}

/// 기한이 지난 proposed 제안을 자동 실행한다 (manual 정책은 건너뜀). 처리한 수를 돌려준다
pub async fn tick(db: &DatabaseConnection) -> Res<u32> {
    let due = a::Entity::find().filter(a::Column::Kind.eq("proposal")).filter(a::Column::Status.eq("proposed"))
        .filter(Expr::cust("deadline_at IS NOT NULL AND datetime(deadline_at) <= datetime('now')")).order_by_asc(a::Column::Sn).all(db).await?;
    let mut n = 0;
    for p in due {
        if policy::load(db, p.project_sn).await?.is_some_and(|x| x.mode == "manual") {
            continue;
        }
        n += run(db, p.sn, By::Auto).await.is_ok() as u32;
    }
    Ok(n)
}
