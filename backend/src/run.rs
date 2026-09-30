//! tbl_run · tbl_session 조회 + Run 명령 (Start · Stop · Retry · Review · Approve · Reject) + 실행기(#13)용 전이 함수
use crate::{entity::{tbl_review as rv, tbl_run as r, tbl_session as s, tbl_task}, error::{Body, Error, ErrorBody, Res, Sn}, event::{self, Ev}, task};
use axum::{Json, extract::State, http::StatusCode};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, sea_query::Expr};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

/// 허용되는 Run 상태 전이 (from, to) · 원본 상태값은 #9. completed · failed · cancelled는 끝 상태 (재시도는 새 Run)
const RUN_MOVES: &[(&str, &str)] = &[
    ("queued", "starting"), ("queued", "cancelled"),
    ("starting", "running"), ("starting", "failed"), ("starting", "cancelled"),
    ("running", "waiting"), ("running", "review"), ("running", "completed"), ("running", "failed"), ("running", "cancelled"),
    ("waiting", "running"), ("waiting", "failed"), ("waiting", "cancelled"),
    ("review", "completed"), ("review", "failed"), ("review", "cancelled"), // completed = 승인, failed = 반려
];

/// 허용되는 Session 상태 전이. stopped · failed는 끝 상태 (교체는 같은 Run 안의 새 Session)
const SESSION_MOVES: &[(&str, &str)] = &[("starting", "active"), ("starting", "failed"), ("active", "stopped"), ("active", "failed")];

/// 아직 끝나지 않은 Run 상태 (태스크당 1개만 허용)
const ACTIVE: [&str; 5] = ["queued", "starting", "running", "waiting", "review"];

/// run 관련 경로 묶음
pub fn routes() -> OpenApiRouter<DatabaseConnection> {
    OpenApiRouter::new()
        .routes(routes!(list, start))
        .routes(routes!(read))
        .routes(routes!(sessions))
        .routes(routes!(stop))
        .routes(routes!(retry))
        .routes(routes!(review))
        .routes(routes!(approve))
        .routes(routes!(reject))
}

/// Run (API 응답 형태)
#[derive(Serialize, ToSchema)]
pub struct Run {
    sn: i64,
    project_sn: i64,
    task_sn: i64,
    member_sn: i64,
    /// 화면 표시 번호 (프로젝트 안에서 1부터)
    num: i64,
    /// queued | starting | running | waiting | review | completed | failed | cancelled
    status: String,
    /// orch | user | retry
    start_by: String,
    /// 재시도 대상인 이전 Run
    retry_run_sn: Option<i64>,
    result_summary: Option<String>,
    fail_code: Option<String>,
    fail_detail: Option<String>,
    branch: Option<String>,
    start_at: Option<String>,
    end_at: Option<String>,
    create_at: String,
}

impl From<r::Model> for Run {
    fn from(m: r::Model) -> Self {
        Self {
            sn: m.sn, project_sn: m.project_sn, task_sn: m.task_sn, member_sn: m.member_sn, num: m.num, status: m.status,
            start_by: m.start_by, retry_run_sn: m.retry_run_sn, result_summary: m.result_summary, fail_code: m.fail_code,
            fail_detail: m.fail_detail, branch: m.branch, start_at: m.start_at, end_at: m.end_at, create_at: m.create_at,
        }
    }
}

/// Session (API 응답 형태)
#[derive(Serialize, ToSchema)]
pub struct Session {
    sn: i64,
    run_sn: i64,
    member_sn: i64,
    /// 멤버별 표시 번호
    num: i64,
    provider_session_id: Option<String>,
    /// starting | active | stopped | failed
    status: String,
    is_resumed: i64,
    rotate_reason: Option<String>,
    start_at: String,
    end_at: Option<String>,
}

impl From<s::Model> for Session {
    fn from(m: s::Model) -> Self {
        Self {
            sn: m.sn, run_sn: m.run_sn, member_sn: m.member_sn, num: m.num, provider_session_id: m.provider_session_id,
            status: m.status, is_resumed: m.is_resumed, rotate_reason: m.rotate_reason, start_at: m.start_at, end_at: m.end_at,
        }
    }
}

/// 반려 요청 본문
#[derive(Deserialize, ToSchema)]
struct RejectBody {
    /// 리뷰한 멤버 (tbl_review.member_sn이 필수라 받는다)
    member_sn: i64,
    /// 반려 사유
    reason: Option<String>,
}

/// Run 1건 읽기. 없으면 404
async fn get(db: &impl ConnectionTrait, sn: i64) -> Res<r::Model> {
    r::Entity::find_by_id(sn).one(db).await?.ok_or_else(Error::not_found)
}

/// Run 상태를 표대로 옮기고 시각을 남긴다 (표에 없으면 409). 태스크는 건드리지 않는다
async fn step(tx: &DatabaseTransaction, sn: i64, to: &str) -> Res<r::Model> {
    let cur = get(tx, sn).await?;
    if !RUN_MOVES.contains(&(cur.status.as_str(), to)) {
        return Err(Error::conflict(format!("cannot move run {} -> {to}", cur.status)));
    }
    let mut q = r::Entity::update_many().filter(r::Column::Sn.eq(sn)).col_expr(r::Column::Status, to.to_owned().into());
    if to == "running" && cur.start_at.is_none() { q = q.col_expr(r::Column::StartAt, Expr::cust("datetime('now')")); }
    if matches!(to, "completed" | "failed" | "cancelled") { q = q.col_expr(r::Column::EndAt, Expr::cust("datetime('now')")); }
    q.exec(tx).await?;
    get(tx, sn).await
}

/// 승인 · 반려는 리뷰 중인 Run에만 (running → completed는 실행기 전이라 표만으로는 리뷰 없이 승인되어 버린다)
async fn in_review(tx: &DatabaseTransaction, sn: i64) -> Res<()> {
    match get(tx, sn).await?.status.as_str() {
        "review" => Ok(()),
        st => Err(Error::conflict(format!("run is {st}, not review"))),
    }
}

/// 새 Run(queued)을 만들고 태스크를 in_progress로 올린다. StartRun · RetryRun 공용. 담당 멤버가 없거나 진행 중 Run(하위 Run 제외 · #67)이 있으면 409
async fn begin(tx: &DatabaseTransaction, task_sn: i64, retry: Option<i64>) -> Res<(Run, Ev)> {
    let t = tbl_task::Entity::find_by_id(task_sn).one(tx).await?.ok_or_else(Error::not_found)?;
    let member = t.member_sn.ok_or_else(|| Error::conflict("task has no member".into()))?;
    if r::Entity::find().filter(r::Column::TaskSn.eq(task_sn)).filter(r::Column::ParentRunSn.is_null()).filter(r::Column::Status.is_in(ACTIVE)).one(tx).await?.is_some() {
        return Err(Error::conflict("task already has an active run".into()));
    }
    if t.status != "in_progress" {
        task::shift(tx, task_sn, "in_progress").await?;
    }
    // 표시 번호: 프로젝트 안 마지막 번호 + 1 (event::run 락 안이라 경합 없음)
    let num = r::Entity::find().filter(r::Column::ProjectSn.eq(t.project_sn)).order_by_desc(r::Column::Num).one(tx).await?.map_or(1, |m| m.num + 1);
    let m = r::ActiveModel {
        project_sn: Set(t.project_sn), task_sn: Set(task_sn), member_sn: Set(member), num: Set(num),
        start_by: Set("user".into()), retry_run_sn: Set(retry), ..Default::default()
    }.insert(tx).await?;
    let out = Run::from(m);
    let ev = Ev::new(Some(out.project_sn), "run", out.sn, "RunStarted", &json!({
        "task_sn": task_sn, "member_sn": member, "num": num, "retry_run_sn": retry, "task_status": "in_progress",
    }));
    Ok((out, ev))
}

/// 태스크의 Run 목록 (번호순)
#[utoipa::path(get, path = "/tasks/{sn}/runs", params(("sn" = i64, Path, description = "태스크 번호")), responses((status = 200, body = Vec<Run>), (status = "default", body = ErrorBody)))]
async fn list(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Vec<Run>>> {
    Ok(Json(r::Entity::find().filter(r::Column::TaskSn.eq(sn)).order_by_asc(r::Column::Num).all(&db).await?.into_iter().map(Run::from).collect()))
}

/// Run 시작 (StartRun → RunStarted). Run은 queued로 만들고 태스크는 in_progress. 담당 멤버가 없거나 진행 중 Run(하위 Run 제외 · #67)이 있으면 409
#[utoipa::path(post, path = "/tasks/{sn}/runs", params(("sn" = i64, Path, description = "태스크 번호")), responses((status = 201, body = Run), (status = "default", body = ErrorBody)))]
async fn start(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<(StatusCode, Json<Run>)> {
    let out = event::run(&db, async |tx| begin(tx, sn, None).await.map(|(out, ev)| (out, vec![ev]))).await?;
    Ok((StatusCode::CREATED, Json(out)))
}

/// 1건 조회. 없으면 404
#[utoipa::path(get, path = "/runs/{sn}", params(("sn" = i64, Path, description = "Run 번호")), responses((status = 200, body = Run), (status = "default", body = ErrorBody)))]
async fn read(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Run>> {
    get(&db, sn).await.map(|m| Json(m.into()))
}

/// Run의 Session 목록 (번호순)
#[utoipa::path(get, path = "/runs/{sn}/sessions", params(("sn" = i64, Path, description = "Run 번호")), responses((status = 200, body = Vec<Session>), (status = "default", body = ErrorBody)))]
async fn sessions(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Vec<Session>>> {
    Ok(Json(s::Entity::find().filter(s::Column::RunSn.eq(sn)).order_by_asc(s::Column::Sn).all(&db).await?.into_iter().map(Session::from).collect()))
}

/// Run 중지 (StopRun → RunCancelled). 끝난 Run이면 409. 태스크 상태는 그대로 둔다
#[utoipa::path(post, path = "/runs/{sn}/stop", params(("sn" = i64, Path, description = "Run 번호")), responses((status = 200, body = Run), (status = "default", body = ErrorBody)))]
async fn stop(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Run>> {
    let out = event::run(&db, async |tx| {
        let out = Run::from(step(tx, sn, "cancelled").await?);
        let ev = Ev::new(Some(out.project_sn), "run", sn, "RunCancelled", &json!({ "task_sn": out.task_sn }));
        Ok((out, vec![ev]))
    }).await?;
    Ok(Json(out))
}

/// 재시도 (RetryRun → RunStarted). failed · cancelled Run에서만, 기존 Run은 그대로 두고 새 Run을 만든다
#[utoipa::path(post, path = "/runs/{sn}/retry", params(("sn" = i64, Path, description = "재시도할 Run 번호")), responses((status = 201, body = Run), (status = "default", body = ErrorBody)))]
async fn retry(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<(StatusCode, Json<Run>)> {
    let out = event::run(&db, async |tx| {
        let old = get(tx, sn).await?;
        if !matches!(old.status.as_str(), "failed" | "cancelled") {
            return Err(Error::conflict(format!("cannot retry run {}", old.status)));
        }
        begin(tx, old.task_sn, Some(sn)).await.map(|(out, ev)| (out, vec![ev]))
    }).await?;
    Ok((StatusCode::CREATED, Json(out)))
}

/// 리뷰 요청 (RequestReview → ReviewRequested). running Run과 in_progress 태스크가 함께 review로 간다
#[utoipa::path(post, path = "/runs/{sn}/review", params(("sn" = i64, Path, description = "Run 번호")), responses((status = 200, body = Run), (status = "default", body = ErrorBody)))]
async fn review(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Run>> {
    let out = event::run(&db, async |tx| {
        let out = Run::from(step(tx, sn, "review").await?);
        task::shift(tx, out.task_sn, "review").await?;
        let ev = Ev::new(Some(out.project_sn), "run", sn, "ReviewRequested", &json!({ "task_sn": out.task_sn, "task_status": "review" }));
        Ok((out, vec![ev]))
    }).await?;
    Ok(Json(out))
}

/// 승인 (ApproveRun → RunApproved). Run은 completed, 태스크는 done
#[utoipa::path(post, path = "/runs/{sn}/approve", params(("sn" = i64, Path, description = "Run 번호")), responses((status = 200, body = Run), (status = "default", body = ErrorBody)))]
async fn approve(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Run>> {
    let out = event::run(&db, async |tx| {
        in_review(tx, sn).await?;
        let out = Run::from(step(tx, sn, "completed").await?);
        task::shift(tx, out.task_sn, "done").await?;
        let ev = Ev::new(Some(out.project_sn), "run", sn, "RunApproved", &json!({ "task_sn": out.task_sn, "task_status": "done" }));
        Ok((out, vec![ev]))
    }).await?;
    Ok(Json(out))
}

/// 반려 (RejectRun → RunRejected). Run은 failed(rejected), 태스크는 in_progress로 돌아가고 tbl_review에 반려가 쌓인다 (round = 반려 횟수). 재실행은 retry
#[utoipa::path(post, path = "/runs/{sn}/reject", params(("sn" = i64, Path, description = "Run 번호")), request_body = RejectBody, responses((status = 200, body = Run), (status = "default", body = ErrorBody)))]
async fn reject(State(db): State<DatabaseConnection>, Sn(sn): Sn, Body(b): Body<RejectBody>) -> Res<Json<Run>> {
    let out = event::run(&db, async |tx| {
        in_review(tx, sn).await?;
        step(tx, sn, "failed").await?;
        r::Entity::update_many().filter(r::Column::Sn.eq(sn)).col_expr(r::Column::FailCode, "rejected".into())
            .col_expr(r::Column::FailDetail, b.reason.clone().into()).exec(tx).await?;
        let out = Run::from(get(tx, sn).await?);
        task::shift(tx, out.task_sn, "in_progress").await?;
        let round = rv::Entity::find().filter(rv::Column::TaskSn.eq(out.task_sn)).count(tx).await? as i64 + 1;
        rv::ActiveModel {
            task_sn: Set(out.task_sn), run_sn: Set(Some(sn)), member_sn: Set(b.member_sn), round: Set(round),
            result: Set("rejected".into()), reason: Set(b.reason.clone()), ..Default::default()
        }.insert(tx).await?;
        let ev = Ev::new(Some(out.project_sn), "run", sn, "RunRejected", &json!({
            "task_sn": out.task_sn, "member_sn": b.member_sn, "reason": b.reason, "round": round, "task_status": "in_progress",
        }));
        Ok((out, vec![ev]))
    }).await?;
    Ok(Json(out))
}

/// 실행기용: Run 상태를 옮긴다 (starting · running · waiting · completed · failed). review · cancelled는 태스크도 바뀌므로 HTTP 명령으로만
#[allow(dead_code)] // #13 실행기가 호출한다
pub async fn run_to(db: &DatabaseConnection, sn: i64, to: &str) -> Res<()> {
    let kind = match to {
        "starting" | "running" | "waiting" => "RunProgressed",
        "completed" => "RunCompleted",
        "failed" => "RunFailed",
        _ => return Err(Error::conflict(format!("run_to cannot target {to}"))),
    };
    event::run(db, async |tx| {
        let from = get(tx, sn).await?.status;
        let m = step(tx, sn, to).await?;
        Ok(((), vec![Ev::new(Some(m.project_sn), "run", sn, kind, &json!({ "from": from, "to": to }))]))
    }).await
}

/// 실행기용: Session 상태를 옮긴다 (starting → active → stopped · failed). 표에 없으면 409
#[allow(dead_code)] // #13 실행기가 호출한다
pub async fn session_to(db: &DatabaseConnection, sn: i64, to: &str) -> Res<()> {
    event::run(db, async |tx| {
        let cur = s::Entity::find_by_id(sn).one(tx).await?.ok_or_else(Error::not_found)?;
        if !SESSION_MOVES.contains(&(cur.status.as_str(), to)) {
            return Err(Error::conflict(format!("cannot move session {} -> {to}", cur.status)));
        }
        let mut q = s::Entity::update_many().filter(s::Column::Sn.eq(sn)).col_expr(s::Column::Status, to.to_owned().into());
        if matches!(to, "stopped" | "failed") { q = q.col_expr(s::Column::EndAt, Expr::cust("datetime('now')")); }
        q.exec(tx).await?;
        let project_sn = get(tx, cur.run_sn).await?.project_sn;
        Ok(((), vec![Ev::new(Some(project_sn), "session", sn, "SessionMoved", &json!({ "run_sn": cur.run_sn, "from": cur.status, "to": to }))]))
    }).await
}
