//! tbl_run · tbl_session 조회 + Run 명령 (Start · Stop · Retry · Review · Approve · Reject) + 실행기(#13)용 전이 함수
use crate::{entity::{tbl_run as r, tbl_session as s, tbl_task}, error::{Body, Error, ErrorBody, Res, Sn}, event::{self, Ev}, task};
use axum::{Json, extract::{Query, State}, http::StatusCode};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection, DatabaseTransaction, DbBackend, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Statement, sea_query::Expr};
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
pub(crate) const ACTIVE: [&str; 5] = ["queued", "starting", "running", "waiting", "review"];

/// run 관련 경로 묶음
pub fn routes() -> OpenApiRouter<DatabaseConnection> {
    OpenApiRouter::new()
        .routes(routes!(list, start))
        .routes(routes!(read))
        .routes(routes!(children))
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
    /// 상위(리드) Run · 하위 작업일 때 (#67)
    parent_run_sn: Option<i64>,
    /// sub | fork | runner · 일반 Run이면 null
    spawn_mode: Option<String>,
    /// S | M | L
    tier: Option<String>,
    /// 하위 작업 종류 (explore · search · format · test · implement · fix · design · review · debug)
    kind: Option<String>,
    /// 리드 Run 안의 하위 순번 (화면 T{task.num}.{child_seq})
    child_seq: Option<i64>,
    /// 받은 @TASK 원문
    brief: Option<String>,
    /// 허용 경로 [{path, source, at}]
    paths: Option<serde_json::Value>,
    /// paths가 겹쳐 기다리는 하위 Run
    wait_run_sn: Option<i64>,
    /// 겹친 경로
    wait_glob: Option<String>,
    start_at: Option<String>,
    end_at: Option<String>,
    create_at: String,
    /// tbl_log_token 합 · 기록이 없으면 null(모름). 조회 응답에서만 채운다
    tokens: Option<Tokens>,
    /// 리드 Run만: 자기 + runner 하위 Run(재시도 전 Run 포함) 토큰 합
    runner_total: Option<Total>,
}

impl From<r::Model> for Run {
    fn from(m: r::Model) -> Self {
        Self {
            paths: m.paths.as_deref().and_then(|p| serde_json::from_str(p).ok()),
            sn: m.sn, project_sn: m.project_sn, task_sn: m.task_sn, member_sn: m.member_sn, num: m.num, status: m.status,
            start_by: m.start_by, retry_run_sn: m.retry_run_sn, result_summary: m.result_summary, fail_code: m.fail_code,
            fail_detail: m.fail_detail, branch: m.branch, parent_run_sn: m.parent_run_sn, spawn_mode: m.spawn_mode, tier: m.tier, kind: m.kind,
            child_seq: m.child_seq, brief: m.brief, wait_run_sn: m.wait_run_sn, wait_glob: m.wait_glob,
            start_at: m.start_at, end_at: m.end_at, create_at: m.create_at, tokens: None, runner_total: None,
        }
    }
}

/// Run 1개의 토큰 합 (tbl_log_token)
#[derive(Serialize, ToSchema, Clone, Default)]
pub struct Tokens {
    input: i64,
    cache_read: i64,
    cache_write: i64,
    output: i64,
    /// input + cache_read + cache_write + output
    total: i64,
    cost_usd_micro: i64,
    /// 값 출처 목록: provider(실측) | estimated(추정)
    sources: Vec<String>,
}

/// 여러 Run 합계. 토큰 기록이 없는 Run은 더하지 않고 unknown_count로 센다 (#100)
#[derive(Serialize, ToSchema)]
pub struct Total {
    value: i64,
    cost_usd_micro: i64,
    /// 더한 Run 수 (자기 포함)
    run_count: i64,
    /// 토큰 기록이 없어 모르는 Run 수
    unknown_count: i64,
}

/// Run 번호들 → 토큰 합 (기록 없는 Run은 빠짐)
pub(crate) async fn tokens_of(db: &impl ConnectionTrait, sns: &[i64]) -> Res<std::collections::HashMap<i64, Tokens>> {
    if sns.is_empty() {
        return Ok(Default::default());
    }
    let ids = sns.iter().map(i64::to_string).collect::<Vec<_>>().join(","); // 정수만이라 그대로 넣는다
    let rows = db.query_all_raw(Statement::from_string(DbBackend::Sqlite, format!(
        "SELECT run_sn, SUM(token_input) i, SUM(token_cache_read) cr, SUM(token_cache_write) cw, SUM(token_output) o, SUM(cost_usd_micro) c, \
         GROUP_CONCAT(DISTINCT usage_source) src FROM tbl_log_token WHERE run_sn IN ({ids}) GROUP BY run_sn"))).await?;
    let mut out = std::collections::HashMap::new();
    for row in rows {
        let g = |c: &str| row.try_get::<i64>("", c).unwrap_or(0);
        let (input, cache_read, cache_write, output) = (g("i"), g("cr"), g("cw"), g("o"));
        let mut sources: Vec<String> = row.try_get::<String>("", "src").unwrap_or_default().split(',').filter(|s| !s.is_empty()).map(str::to_owned).collect();
        sources.sort();
        out.insert(g("run_sn"), Tokens { input, cache_read, cache_write, output, total: input + cache_read + cache_write + output, cost_usd_micro: g("c"), sources });
    }
    Ok(out)
}

/// 조회 응답용: 토큰 합을 채우고, 리드 Run(parent 없음)에는 runner 하위 Run 포함 합계를 붙인다
pub(crate) async fn enrich(db: &impl ConnectionTrait, mut runs: Vec<Run>) -> Res<Vec<Run>> {
    let leads: Vec<i64> = runs.iter().filter(|x| x.parent_run_sn.is_none()).map(|x| x.sn).collect();
    let kids = r::Entity::find().filter(r::Column::ParentRunSn.is_in(leads.clone())).filter(r::Column::SpawnMode.eq("runner")).all(db).await?;
    let all: Vec<i64> = runs.iter().map(|x| x.sn).chain(kids.iter().map(|k| k.sn)).collect();
    let tok = tokens_of(db, &all).await?;
    for x in &mut runs {
        x.tokens = tok.get(&x.sn).cloned();
        if x.parent_run_sn.is_none() {
            let group: Vec<i64> = std::iter::once(x.sn).chain(kids.iter().filter(|k| k.parent_run_sn == Some(x.sn)).map(|k| k.sn)).collect();
            let known: Vec<&Tokens> = group.iter().filter_map(|s| tok.get(s)).collect();
            x.runner_total = Some(Total {
                value: known.iter().map(|t| t.total).sum(), cost_usd_micro: known.iter().map(|t| t.cost_usd_micro).sum(),
                run_count: group.len() as i64, unknown_count: (group.len() - known.len()) as i64,
            });
        }
    }
    Ok(runs)
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

/// 태스크의 반려 횟수 = fail_code가 rejected인 Run 수 (반려 반복 가드 · round)
pub(crate) async fn rejects(db: &impl ConnectionTrait, task_sn: i64) -> Res<i64> {
    Ok(r::Entity::find().filter(r::Column::TaskSn.eq(task_sn)).filter(r::Column::FailCode.eq("rejected")).count(db).await? as i64)
}

/// 반려 요청 본문
#[derive(Deserialize, ToSchema)]
struct RejectBody {
    /// 반려한 멤버 (tbl_run.review_member_sn)
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

/// 태스크의 Run 목록 (번호순 · 토큰 합 포함). `children=0`이면 하위 Run을 뺀다 (기본: 포함)
#[utoipa::path(operation_id = "run_list", get, path = "/tasks/{sn}/runs", params(("sn" = i64, Path, description = "태스크 번호"), ("children" = Option<i64>, Query, description = "0 = 리드 Run만")), responses((status = 200, body = Vec<Run>), (status = "default", body = ErrorBody)))]
async fn list(State(db): State<DatabaseConnection>, Sn(sn): Sn, Query(q): Query<std::collections::HashMap<String, String>>) -> Res<Json<Vec<Run>>> {
    let mut f = r::Entity::find().filter(r::Column::TaskSn.eq(sn));
    if q.get("children").map(String::as_str) == Some("0") { f = f.filter(r::Column::ParentRunSn.is_null()); }
    let runs = f.order_by_asc(r::Column::Num).all(&db).await?.into_iter().map(Run::from).collect();
    enrich(&db, runs).await.map(Json)
}

/// 하위 Run 목록 (child_seq → 번호순 · 토큰 합 포함). 리드 Run이 없으면 404
#[utoipa::path(operation_id = "run_children", get, path = "/runs/{sn}/children", params(("sn" = i64, Path, description = "리드 Run 번호")), responses((status = 200, body = Vec<Run>), (status = "default", body = ErrorBody)))]
async fn children(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Vec<Run>>> {
    get(&db, sn).await?;
    let runs = r::Entity::find().filter(r::Column::ParentRunSn.eq(sn)).order_by_asc(r::Column::ChildSeq).order_by_asc(r::Column::Sn).all(&db).await?.into_iter().map(Run::from).collect();
    enrich(&db, runs).await.map(Json)
}

/// Run 시작 (StartRun → RunStarted). Run은 queued로 만들고 태스크는 in_progress. 담당 멤버가 없거나 진행 중 Run(하위 Run 제외 · #67)이 있으면 409
#[utoipa::path(operation_id = "run_start", post, path = "/tasks/{sn}/runs", params(("sn" = i64, Path, description = "태스크 번호")), responses((status = 201, body = Run), (status = "default", body = ErrorBody)))]
async fn start(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<(StatusCode, Json<Run>)> {
    crate::context::guard(&db, sn).await?; // 입력 상한을 넘으면 422 context_over · Run을 만들지 않는다
    let out = event::run(&db, async |tx| begin(tx, sn, None).await.map(|(out, ev)| (out, vec![ev]))).await?;
    Ok((StatusCode::CREATED, Json(out)))
}

/// 1건 조회 (토큰 합 포함). 없으면 404
#[utoipa::path(operation_id = "run_read", get, path = "/runs/{sn}", params(("sn" = i64, Path, description = "Run 번호")), responses((status = 200, body = Run), (status = "default", body = ErrorBody)))]
async fn read(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Run>> {
    let run = Run::from(get(&db, sn).await?);
    enrich(&db, vec![run]).await?.pop().map(Json).ok_or_else(Error::not_found)
}

/// Run의 Session 목록 (번호순)
#[utoipa::path(operation_id = "run_sessions", get, path = "/runs/{sn}/sessions", params(("sn" = i64, Path, description = "Run 번호")), responses((status = 200, body = Vec<Session>), (status = "default", body = ErrorBody)))]
async fn sessions(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Vec<Session>>> {
    Ok(Json(s::Entity::find().filter(s::Column::RunSn.eq(sn)).order_by_asc(s::Column::Sn).all(&db).await?.into_iter().map(Session::from).collect()))
}

/// Run 중지 (StopRun → RunCancelled). 끝난 Run이면 409. 태스크 상태는 그대로 둔다
#[utoipa::path(operation_id = "run_stop", post, path = "/runs/{sn}/stop", params(("sn" = i64, Path, description = "Run 번호")), responses((status = 200, body = Run), (status = "default", body = ErrorBody)))]
async fn stop(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Run>> {
    let out = event::run(&db, async |tx| {
        let out = Run::from(step(tx, sn, "cancelled").await?);
        let ev = Ev::new(Some(out.project_sn), "run", sn, "RunCancelled", &json!({ "task_sn": out.task_sn }));
        Ok((out, vec![ev]))
    }).await?;
    Ok(Json(out))
}

/// 재시도 (RetryRun → RunStarted). failed · cancelled Run에서만, 기존 Run은 그대로 두고 새 Run을 만든다
#[utoipa::path(operation_id = "run_retry", post, path = "/runs/{sn}/retry", params(("sn" = i64, Path, description = "재시도할 Run 번호")), responses((status = 201, body = Run), (status = "default", body = ErrorBody)))]
async fn retry(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<(StatusCode, Json<Run>)> {
    crate::context::guard(&db, get(&db, sn).await?.task_sn).await?; // 입력 상한을 넘으면 422 context_over
    let out = event::run(&db, async |tx| redo(tx, sn).await.map(|(out, ev)| (out, vec![ev]))).await?;
    Ok((StatusCode::CREATED, Json(out)))
}

/// 재시도 Run을 만든다 (RunStarted). failed · cancelled Run에서만 (아니면 409). 재시도 API와 Orch 제안 실행이 함께 쓴다
pub(crate) async fn redo(tx: &DatabaseTransaction, sn: i64) -> Res<(Run, Ev)> {
    let old = get(tx, sn).await?;
    if !matches!(old.status.as_str(), "failed" | "cancelled") {
        return Err(Error::conflict(format!("cannot retry run {}", old.status)));
    }
    begin(tx, old.task_sn, Some(sn)).await
}

/// Run의 연결을 바꿔 표시한다 (FallbackUsed). 실제 전환은 실행기(#13)가 한다
pub(crate) async fn switch(tx: &DatabaseTransaction, sn: i64, connection_sn: i64) -> Res<Ev> {
    let cur = get(tx, sn).await?;
    r::Entity::update_many().filter(r::Column::Sn.eq(sn)).col_expr(r::Column::ConnectionSn, connection_sn.into()).exec(tx).await?;
    Ok(Ev::new(Some(cur.project_sn), "run", sn, "FallbackUsed", &json!({ "from": cur.connection_sn, "to": connection_sn })))
}

/// 리뷰 요청 (RequestReview → ReviewRequested). running Run과 in_progress 태스크가 함께 review로 간다
#[utoipa::path(operation_id = "run_review", post, path = "/runs/{sn}/review", params(("sn" = i64, Path, description = "Run 번호")), responses((status = 200, body = Run), (status = "default", body = ErrorBody)))]
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
#[utoipa::path(operation_id = "run_approve", post, path = "/runs/{sn}/approve", params(("sn" = i64, Path, description = "Run 번호")), responses((status = 200, body = Run), (status = "default", body = ErrorBody)))]
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

/// 반려 (RejectRun → RunRejected). Run은 failed(rejected), 태스크는 in_progress로 돌아가고 반려 멤버 · 사유가 Run에 남는다 (round = 그 태스크의 반려 Run 수). 재실행은 retry
#[utoipa::path(operation_id = "run_reject", post, path = "/runs/{sn}/reject", params(("sn" = i64, Path, description = "Run 번호")), request_body = RejectBody, responses((status = 200, body = Run), (status = "default", body = ErrorBody)))]
async fn reject(State(db): State<DatabaseConnection>, Sn(sn): Sn, Body(b): Body<RejectBody>) -> Res<Json<Run>> {
    let out = event::run(&db, async |tx| {
        in_review(tx, sn).await?;
        step(tx, sn, "failed").await?;
        r::Entity::update_many().filter(r::Column::Sn.eq(sn)).col_expr(r::Column::FailCode, "rejected".into())
            .col_expr(r::Column::FailDetail, b.reason.clone().into()).col_expr(r::Column::ReviewMemberSn, b.member_sn.into())
            .col_expr(r::Column::ReviewReason, b.reason.clone().into()).exec(tx).await?;
        let out = Run::from(get(tx, sn).await?);
        task::shift(tx, out.task_sn, "in_progress").await?;
        // 몇 번째 반려인지 = 그 태스크의 반려 Run 수 (방금 것 포함)
        let round = rejects(tx, out.task_sn).await?;
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
    let member = get(db, sn).await?.member_sn;
    event::run_as(db, "member", Some(member), async |tx| {
        let from = get(tx, sn).await?.status;
        let m = step(tx, sn, to).await?;
        Ok(((), vec![Ev::new(Some(m.project_sn), "run", sn, kind, &json!({ "from": from, "to": to }))]))
    }).await
}

/// 실행기용: Session 상태를 옮긴다 (starting → active → stopped · failed). 표에 없으면 409
#[allow(dead_code)] // #13 실행기가 호출한다
pub async fn session_to(db: &DatabaseConnection, sn: i64, to: &str) -> Res<()> {
    event::run_as(db, "system", None, async |tx| {
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
