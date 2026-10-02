//! tbl_decision (판단 요청 · 질문 · 선택지) 조회 · 응답 + Orch/실행기용 생성 함수. 쓰기는 event::run 경유 (Decision* 이벤트)
use crate::{entity::{tbl_decision::{self as d, Entity as Tbl}, tbl_decision_option as o, tbl_decision_question as q},
    error::{Body, Error, ErrorBody, Res, Sn}, event::{self, Ev}};
use axum::{Json, extract::{Query, State}};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder, sea_query::Expr};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashSet;
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};

/// 허용되는 Decision 상태 전이 (from, to). answered · orch_decided · cancelled는 끝 상태. orch_decided · cancelled는 실행기 Task가 쓴다
const MOVES: &[(&str, &str)] = &[
    ("pending", "writing"), ("pending", "answered"), ("pending", "orch_decided"), ("pending", "cancelled"),
    ("writing", "answered"), ("writing", "orch_decided"), ("writing", "cancelled"),
];

/// decision 관련 경로 묶음
pub fn routes() -> OpenApiRouter<DatabaseConnection> {
    OpenApiRouter::new()
        .routes(routes!(list))
        .routes(routes!(read))
        .routes(routes!(answer))
        .routes(routes!(writing))
}

/// 선택지 (API 응답 형태)
#[derive(Serialize, ToSchema)]
pub struct Choice {
    sn: i64,
    sort: i64,
    /// 표시 기호 (A, B, C)
    code: String,
    label: String,
    note: Option<String>,
    is_recommended: i64,
    is_selected: i64,
}

/// 질문 (API 응답 형태). 선택지를 순서대로 담는다
#[derive(Serialize, ToSchema)]
pub struct Question {
    sn: i64,
    sort: i64,
    title: String,
    body: Option<String>,
    code_snippet: Option<String>,
    /// 참고 자료 JSON 배열 문자열 (파일 · 줄 · 태스크)
    ref_json: Option<String>,
    answer_text: Option<String>,
    is_delegate: i64,
    answer_at: Option<String>,
    options: Vec<Choice>,
}

/// 판단 요청 (API 응답 형태). 질문 · 선택지를 함께 준다 (DecisionPanel 한 번에 그리기)
#[derive(Serialize, ToSchema)]
pub struct Decision {
    sn: i64,
    project_sn: i64,
    task_sn: Option<i64>,
    run_sn: Option<i64>,
    member_sn: i64,
    level: i64,
    title: String,
    /// pending | writing | answered | orch_decided | cancelled
    status: String,
    /// user | orch
    decide_by: Option<String>,
    orch_reason: Option<String>,
    is_timer_pause: i64,
    deadline_at: Option<String>,
    is_review_needed: i64,
    create_at: String,
    decide_at: Option<String>,
    questions: Vec<Question>,
}

/// 목록 쿼리. 범위는 전체(없음) · 프로젝트 · 태스크 — DecisionPanel scope
#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct Filter {
    /// 이 상태만 (예: pending)
    pub status: Option<String>,
    pub project_sn: Option<i64>,
    pub task_sn: Option<i64>,
}

/// 생성할 선택지 (Orch/실행기 입력)
#[derive(Deserialize)]
pub struct ChoiceNew {
    pub code: String,
    pub label: String,
    pub note: Option<String>,
    #[serde(default)]
    pub is_recommended: bool,
}

/// 생성할 질문 (Orch/실행기 입력)
#[derive(Deserialize)]
pub struct QuestionNew {
    pub title: String,
    pub body: Option<String>,
    pub code_snippet: Option<String>,
    pub ref_json: Option<String>,
    pub options: Vec<ChoiceNew>,
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
    pub questions: Vec<QuestionNew>,
}

/// 질문 하나의 답. 선택지 · 직접 쓴 말 · Orch에게 맡김 중 하나 이상
#[derive(Serialize, Deserialize, ToSchema)]
struct Answer {
    question_sn: i64,
    option_sn: Option<i64>,
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

/// 행 1건 읽기. 없으면 404
async fn row(db: &impl ConnectionTrait, sn: i64) -> Res<d::Model> {
    Tbl::find_by_id(sn).one(db).await?.ok_or_else(Error::not_found)
}

/// 판단 요청 행들에 질문 · 선택지를 붙인다 (질문 · 선택지는 sort → 번호순)
async fn full(db: &impl ConnectionTrait, rows: Vec<d::Model>) -> Res<Vec<Decision>> {
    let qs = q::Entity::find().filter(q::Column::DecisionSn.is_in(rows.iter().map(|m| m.sn))).order_by_asc(q::Column::Sort).order_by_asc(q::Column::Sn).all(db).await?;
    let mut os = o::Entity::find().filter(o::Column::QuestionSn.is_in(qs.iter().map(|m| m.sn))).order_by_asc(o::Column::Sort).order_by_asc(o::Column::Sn).all(db).await?;
    let mut qs: Vec<(i64, Question)> = qs.into_iter().map(|m| {
        let options = os.extract_if(.., |c| c.question_sn == m.sn).map(|c| Choice {
            sn: c.sn, sort: c.sort, code: c.code, label: c.label, note: c.note, is_recommended: c.is_recommended, is_selected: c.is_selected,
        }).collect();
        (m.decision_sn, Question {
            sn: m.sn, sort: m.sort, title: m.title, body: m.body, code_snippet: m.code_snippet, ref_json: m.ref_json,
            answer_text: m.answer_text, is_delegate: m.is_delegate, answer_at: m.answer_at, options,
        })
    }).collect();
    Ok(rows.into_iter().map(|m| Decision {
        questions: qs.extract_if(.., |(ds, _)| *ds == m.sn).map(|(_, x)| x).collect(),
        sn: m.sn, project_sn: m.project_sn, task_sn: m.task_sn, run_sn: m.run_sn, member_sn: m.member_sn, level: m.level, title: m.title,
        status: m.status, decide_by: m.decide_by, orch_reason: m.orch_reason, is_timer_pause: m.is_timer_pause, deadline_at: m.deadline_at,
        is_review_needed: m.is_review_needed, create_at: m.create_at, decide_at: m.decide_at,
    }).collect())
}

/// 판단 요청 1건 (질문 · 선택지 포함). 없으면 404
async fn get(db: &impl ConnectionTrait, sn: i64) -> Res<Decision> {
    full(db, vec![row(db, sn).await?]).await?.pop().ok_or_else(Error::not_found)
}

/// 상태를 to로 옮긴다. 표에 없는 전이면 409
async fn step(tx: &DatabaseTransaction, sn: i64, to: &str) -> Res<d::Model> {
    let cur = row(tx, sn).await?;
    if !MOVES.contains(&(cur.status.as_str(), to)) {
        return Err(Error::conflict(format!("cannot move decision {} -> {to}", cur.status)));
    }
    Tbl::update_many().filter(d::Column::Sn.eq(sn)).col_expr(d::Column::Status, to.to_owned().into()).exec(tx).await?;
    Ok(cur)
}

/// Orch/실행기용: 판단 요청 생성 (DecisionRequested). 레벨 2~4 · 질문 1개 이상 아니면 422, 없는 프로젝트 · 멤버는 422(invalid_ref)
#[allow(dead_code)] // Orch · 실행기(#13)가 호출한다
pub async fn create(db: &DatabaseConnection, b: DecisionNew) -> Res<Decision> {
    if !(2..=4).contains(&b.level) || b.questions.is_empty() {
        return Err(Error::invalid("level must be 2..=4 and questions non-empty".into()));
    }
    event::run(db, async |tx| {
        let m = d::ActiveModel {
            project_sn: Set(b.project_sn), task_sn: Set(b.task_sn), run_sn: Set(b.run_sn), member_sn: Set(b.member_sn),
            level: Set(b.level), title: Set(b.title), deadline_at: Set(b.deadline_at), ..Default::default()
        }.insert(tx).await?;
        for (qi, x) in b.questions.into_iter().enumerate() {
            let qm = q::ActiveModel {
                decision_sn: Set(m.sn), sort: Set(qi as i64 + 1), title: Set(x.title), body: Set(x.body), code_snippet: Set(x.code_snippet),
                ref_json: Set(x.ref_json), ..Default::default()
            }.insert(tx).await?;
            for (oi, c) in x.options.into_iter().enumerate() {
                o::ActiveModel {
                    question_sn: Set(qm.sn), sort: Set(oi as i64 + 1), code: Set(c.code), label: Set(c.label), note: Set(c.note),
                    is_recommended: Set(c.is_recommended.into()), ..Default::default()
                }.insert(tx).await?;
            }
        }
        let out = get(tx, m.sn).await?;
        let ev = Ev::new(Some(out.project_sn), "decision", out.sn, "DecisionRequested", &json!({ "task_sn": out.task_sn, "level": out.level, "title": out.title }));
        Ok((out, vec![ev]))
    }).await
}

/// 판단 요청 목록 (최신순). status · project_sn · task_sn으로 거른다
#[utoipa::path(operation_id = "decision_list", get, path = "/decisions", params(Filter), responses((status = 200, body = Vec<Decision>), (status = "default", body = ErrorBody)))]
async fn list(State(db): State<DatabaseConnection>, Query(f): Query<Filter>) -> Res<Json<Vec<Decision>>> {
    let mut s = Tbl::find();
    if let Some(v) = f.status { s = s.filter(d::Column::Status.eq(v)); }
    if let Some(v) = f.project_sn { s = s.filter(d::Column::ProjectSn.eq(v)); }
    if let Some(v) = f.task_sn { s = s.filter(d::Column::TaskSn.eq(v)); }
    Ok(Json(full(&db, s.order_by_desc(d::Column::Sn).all(&db).await?).await?))
}

/// 판단 요청 1건 (질문 · 선택지 포함). 없으면 404
#[utoipa::path(operation_id = "decision_read", get, path = "/decisions/{sn}", params(("sn" = i64, Path, description = "판단 요청 번호")), responses((status = 200, body = Decision), (status = "default", body = ErrorBody)))]
async fn read(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Decision>> {
    get(&db, sn).await.map(Json)
}

/// 답변 (AnswerDecision → DecisionAnswered). pending · writing에서만 (아니면 409).
/// 모든 질문에 정확히 한 번씩, 각 답은 선택지 · 말 · 맡김 중 하나 이상, 선택지는 그 질문의 것 — 아니면 422
#[utoipa::path(operation_id = "decision_answer", post, path = "/decisions/{sn}/answer", params(("sn" = i64, Path, description = "판단 요청 번호")), request_body = AnswerBody, responses((status = 200, body = Decision), (status = "default", body = ErrorBody)))]
async fn answer(State(db): State<DatabaseConnection>, Sn(sn): Sn, Body(b): Body<AnswerBody>) -> Res<Json<Decision>> {
    let out = event::run(&db, async |tx| {
        let cur = step(tx, sn, "answered").await?;
        let qs: HashSet<i64> = q::Entity::find().filter(q::Column::DecisionSn.eq(sn)).all(tx).await?.into_iter().map(|m| m.sn).collect();
        let got: HashSet<i64> = b.answers.iter().map(|a| a.question_sn).collect();
        if got != qs || got.len() != b.answers.len() || b.answers.iter().any(|a| a.option_sn.is_none() && a.text.is_none() && !a.delegate) {
            return Err(Error::invalid("answer every question once with option_sn, text or delegate".into()));
        }
        for a in &b.answers {
            if let Some(os) = a.option_sn && o::Entity::find_by_id(os).filter(o::Column::QuestionSn.eq(a.question_sn)).one(tx).await?.is_none() {
                return Err(Error::invalid(format!("option {os} is not in question {}", a.question_sn)));
            }
            // 질문당 선택 1개: 전부 끄고 고른 것만 켠다
            o::Entity::update_many().filter(o::Column::QuestionSn.eq(a.question_sn)).col_expr(o::Column::IsSelected, 0.into()).exec(tx).await?;
            if let Some(os) = a.option_sn {
                o::Entity::update_many().filter(o::Column::Sn.eq(os)).col_expr(o::Column::IsSelected, 1.into()).exec(tx).await?;
            }
            q::Entity::update_many().filter(q::Column::Sn.eq(a.question_sn)).col_expr(q::Column::AnswerText, a.text.clone().into())
                .col_expr(q::Column::IsDelegate, i64::from(a.delegate).into()).col_expr(q::Column::AnswerAt, Expr::cust("datetime('now')")).exec(tx).await?;
        }
        Tbl::update_many().filter(d::Column::Sn.eq(sn)).col_expr(d::Column::DecideBy, "user".into()).col_expr(d::Column::Uid, crate::UID.into())
            .col_expr(d::Column::IsTimerPause, 0.into()).col_expr(d::Column::IsReviewNeeded, i64::from(b.review_needed).into())
            .col_expr(d::Column::DecideAt, Expr::cust("datetime('now')")).exec(tx).await?;
        let out = get(tx, sn).await?;
        Ok((out, vec![Ev::new(Some(cur.project_sn), "decision", sn, "DecisionAnswered", &json!({ "task_sn": cur.task_sn, "from": cur.status, "answers": b.answers, "review_needed": b.review_needed }))]))
    }).await?;
    Ok(Json(out))
}

/// 작성 중 (DecisionWriting). pending → writing, 타이머를 멈춘다. pending이 아니면 409
#[utoipa::path(operation_id = "decision_writing", post, path = "/decisions/{sn}/writing", params(("sn" = i64, Path, description = "판단 요청 번호")), responses((status = 200, body = Decision), (status = "default", body = ErrorBody)))]
async fn writing(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Decision>> {
    let out = event::run(&db, async |tx| {
        let cur = step(tx, sn, "writing").await?;
        Tbl::update_many().filter(d::Column::Sn.eq(sn)).col_expr(d::Column::IsTimerPause, 1.into()).exec(tx).await?;
        let out = get(tx, sn).await?;
        Ok((out, vec![Ev::new(Some(cur.project_sn), "decision", sn, "DecisionWriting", &json!({ "task_sn": cur.task_sn }))]))
    }).await?;
    Ok(Json(out))
}
