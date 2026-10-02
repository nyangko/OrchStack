//! Orch 규칙 엔진 (#119 · LLM 0): 이벤트 1건 → 제안 0~1건 (`on`), 제안 실행 (`run` · 기존 command 재사용), 타이머 (`tick`), 멤버 대기열.
//! 일상 PM 판단(다음 태스크 배정 · 재시도 · 이슈 닫기 · 폴백 · 가드 정지)을 모델 없이 처리한다. 실행기 · 모델 호출은 여기서 하지 않는다 (B-18)
use crate::{ask::{self, Choice, DecisionNew, Opt, ProposalNew, Question}, entity::{tbl_agent_profile as ap, tbl_connection as cn, tbl_connection_quota as qt, tbl_issue as i, tbl_log_event as e, tbl_member as mb,
    tbl_ask as ak, tbl_project as pj, tbl_review as rv, tbl_run as r, tbl_task as t},
    error::{Error, ErrorBody, Res, Sn}, event::self,
    orch, policy::{self, Guard, Level, Policy}, run as runs, stat, task};
use axum::{Json, extract::State};
use sea_orm::{ColumnTrait, ConnectionTrait, DatabaseConnection, DbBackend, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Statement, Value,
    sea_query::{Expr, SimpleExpr}};
use serde::Serialize;
use serde_json::Value as Json_;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

/// 규칙 엔진이 만드는 제안의 레벨 (kind별 고정 · #119) / 가드 정지 레벨
const LEVEL: i64 = 1;
const STOP_LEVEL: i64 = 2;

/// orch_rule 관련 경로 묶음
pub fn routes() -> OpenApiRouter<DatabaseConnection> {
    OpenApiRouter::new().routes(routes!(queue))
}

/// 지금(UTC) + shift("+5 seconds")의 SQLite datetime 문자열
pub(crate) async fn at(db: &impl ConnectionTrait, shift: &str) -> Res<String> {
    let row = db.query_one_raw(Statement::from_sql_and_values(DbBackend::Sqlite, "SELECT datetime('now', ?)", [Value::from(shift)])).await?;
    Ok(row.and_then(|x| x.try_get_by_index::<String>(0).ok()).unwrap_or_default())
}

/// 태스크를 done으로 만든 이벤트면 그 태스크 번호 (TaskMoved to=done · RunApproved)
pub(crate) fn done_task(ev: &e::Model) -> Option<i64> {
    let p: Json_ = serde_json::from_str(&ev.payload_json).unwrap_or_default();
    match ev.event_type.as_str() {
        "TaskMoved" if p["to"] == "done" => Some(ev.aggregate_sn),
        "RunApproved" => p["task_sn"].as_i64(),
        _ => None,
    }
}

/// 규칙이 보는 이벤트 종류
enum Flow {
    Done(i64),
    Failed(i64),
    Rejected(i64),
    Made(i64),
}

/// 이벤트 → 규칙이 볼 흐름 (관심 없는 이벤트는 None)
fn flow(ev: &e::Model) -> Option<Flow> {
    let p: Json_ = serde_json::from_str(&ev.payload_json).unwrap_or_default();
    match ev.event_type.as_str() {
        "RunFailed" => Some(Flow::Failed(ev.aggregate_sn)),
        "RunRejected" => p["task_sn"].as_i64().map(Flow::Rejected),
        "TaskCreated" => Some(Flow::Made(ev.aggregate_sn)),
        _ => done_task(ev).map(Flow::Done),
    }
}

/// 제안 후보 + 가드 검사에 쓸 문맥
struct Pick {
    nb: ProposalNew,
    task: t::Model,
    fail: Option<String>,
}

/// `on` 한 번의 문맥
struct Cx<'a> {
    db: &'a DatabaseConnection,
    ev: &'a e::Model,
    ps: i64,
    orch: i64,
    pol: &'a Policy,
}

/// 이벤트 1건 → 제안 0~1건. 만든 제안 번호를 돌려준다 (판단 요청으로 돌린 것은 None).
/// 정책 모드대로: manual = 기한 없이 대기 · auto = timer_sec 뒤 자동 · full_auto = 바로 실행. 가드에 걸리면 제안 대신 guard_stop
pub async fn on(db: &DatabaseConnection, ev: &e::Model) -> Res<Option<i64>> {
    let Some(flow) = flow(ev) else { return Ok(None) };
    let Some(ps) = ev.project_sn else { return Ok(None) };
    let Some(orch) = orch::orch_of(db, ps).await? else { return Ok(None) };
    let Some(pol) = policy::load(db, ps).await? else { return Ok(None) };
    let cx = Cx { db, ev, ps, orch, pol: &pol };
    let (codes, cand): (&[&str], Option<Pick>) = match flow {
        Flow::Done(tsn) => (&["auto_streak", "issue_budget"], cx.done(tsn).await?),
        Flow::Failed(rsn) => (&["auto_streak", "same_failure", "issue_budget"], cx.failed(rsn).await?),
        // 가드만 보는 이벤트: 반려 반복 · Orch가 만든 태스크 수
        Flow::Rejected(tsn) | Flow::Made(tsn) => {
            let Some(tk) = t::Entity::find_by_id(tsn).one(db).await? else { return Ok(None) };
            if matches!(flow, Flow::Made(_)) && tk.create_by != "orch" {
                return Ok(None);
            }
            return cx.trip(if matches!(flow, Flow::Made(_)) { &["orch_new_task"] } else { &["reject_loop"] }, &tk, None).await;
        }
    };
    let Some(c) = cand else { return Ok(None) };
    if let Some(sn) = cx.trip(codes, &c.task, c.fail.as_deref()).await? {
        return Ok(Some(sn));
    }
    cx.offer(c.nb).await
}

impl Cx<'_> {
    /// 걸린 가드가 있으면 guard_stop 제안을 만들고 그 번호를 돌려준다 (없으면 None)
    // ponytail: 가드의 scope는 코드별로 고정(auto_streak = 팀 · 나머지 = 태스크/이슈), minute 단위 예산 · user_absent는 아직 안 본다
    async fn trip(&self, codes: &[&str], tk: &t::Model, fail: Option<&str>) -> Res<Option<i64>> {
        for g in self.pol.guards.iter().filter(|g| g.is_enabled == 1 && codes.contains(&g.code.as_str())) {
            let n = match (g.code.as_str(), tk.issue_sn) {
                ("auto_streak", _) => streak(self.db, &projects_of(self.db, self.ps).await?).await?,
                ("reject_loop", _) => rv::Entity::find().filter(rv::Column::TaskSn.eq(tk.sn)).filter(rv::Column::Result.eq("rejected")).count(self.db).await? as i64,
                ("same_failure", _) => match fail {
                    Some(f) => r::Entity::find().filter(r::Column::TaskSn.eq(tk.sn)).filter(r::Column::ParentRunSn.is_null()).filter(r::Column::Status.eq("failed"))
                        .filter(r::Column::FailCode.eq(f)).count(self.db).await? as i64,
                    None => 0,
                },
                ("orch_new_task", Some(is)) => t::Entity::find().filter(t::Column::IssueSn.eq(is)).filter(t::Column::CreateBy.eq("orch")).count(self.db).await? as i64,
                ("issue_budget", Some(is)) if g.threshold_unit == "token" => stat::one(self.db,
                    "SELECT SUM(l.token_input + l.token_cache_read + l.token_cache_write + l.token_output) FROM tbl_log_token l JOIN tbl_run x ON x.sn = l.run_sn JOIN tbl_task k ON k.sn = x.task_sn WHERE k.issue_sn = ?",
                    vec![is.into()]).await?.unwrap_or(0),
                _ => 0,
            };
            if n >= g.threshold {
                return self.stop(g, n, tk).await.map(Some);
            }
        }
        Ok(None)
    }

    /// guard_stop 제안(바로 stopped) + 가드 trigger_at (+ on_trigger = to_manual이면 정책 모드 manual). 한 트랜잭션
    async fn stop(&self, g: &Guard, n: i64, tk: &t::Model) -> Res<i64> {
        let nb = ProposalNew {
            project_sn: self.ps, issue_sn: tk.issue_sn, task_sn: Some(tk.sn), kind: "guard_stop".into(), level: STOP_LEVEL, title: g.name.clone(),
            reason: Some(format!("{} {n} >= {}", g.code, g.threshold)), guard_code: Some(g.code.clone()), status: Some("stopped".into()), event_sn: Some(self.ev.sn), ..Default::default()
        };
        event::run_as(self.db, "orch", Some(self.orch), async |tx| {
            let (p, ev) = ask::add(tx, nb).await?;
            let mut evs = vec![ev];
            policy::guard_hit(tx, self.ps, &g.code).await?;
            if g.on_trigger == "to_manual" {
                evs.extend(policy::to_manual(tx, self.ps).await?);
            }
            Ok((p.sn, evs))
        }).await
    }

    /// 태스크 done → 같은 이슈의 다음 태스크 배정 후보, 이슈 태스크가 모두 done이면 이슈 닫기 후보
    async fn done(&self, tsn: i64) -> Res<Option<Pick>> {
        let Some(tk) = t::Entity::find_by_id(tsn).one(self.db).await? else { return Ok(None) };
        let Some(isn) = tk.issue_sn else { return Ok(None) };
        let Some(issue) = i::Entity::find_by_id(isn).one(self.db).await? else { return Ok(None) };
        let sibs = t::Entity::find().filter(t::Column::IssueSn.eq(isn)).filter(t::Column::Status.ne("cancelled")).order_by_asc(t::Column::Num).all(self.db).await?;
        if sibs.iter().all(|x| x.status == "done") {
            if issue.status == "closed" {
                return Ok(None);
            }
            let nb = ProposalNew { issue_sn: Some(isn), kind: "close_issue".into(), level: LEVEL, title: format!("#{} {}", issue.num, issue.title), reason: Some("all tasks done".into()), ..Default::default() };
            return Ok(Some(Pick { nb, task: tk, fail: None }));
        }
        // 다음 태스크: 담당 없는 todo · 의존 해소 · priority → queue_sort → num
        let todo: Vec<&t::Model> = sibs.iter().filter(|x| x.status == "todo" && x.member_sn.is_none()).collect();
        let wait = task::waiting(self.db, todo.iter().map(|x| x.sn).collect()).await?;
        let Some(next) = todo.into_iter().filter(|x| !wait.contains(&x.sn)).min_by_key(|x| (x.priority, x.queue_sort.unwrap_or(i64::MAX), x.num)) else { return Ok(None) };
        // 멤버: 끝낸 멤버가 놀고 있으면 그 멤버, 아니면 역할이 같은 노는 멤버
        let Some(dm) = (match tk.member_sn { Some(m) => mb::Entity::find_by_id(m).one(self.db).await?, None => None }) else { return Ok(None) };
        let mut pool = vec![dm.clone()];
        pool.extend(mb::Entity::find().filter(mb::Column::TeamSn.eq(dm.team_sn)).filter(mb::Column::RoleName.eq(dm.role_name.as_str())).filter(mb::Column::IsOrch.eq(0))
            .filter(mb::Column::Sn.ne(dm.sn)).order_by_asc(mb::Column::Sn).all(self.db).await?);
        let mut free = Vec::new();
        for m in pool {
            if idle(self.db, &m).await? {
                free.push(m);
            }
        }
        let Some(first) = free.first() else { return Ok(None) };
        let opts: Vec<Opt> = free.iter().skip(1).take(3).map(|m| Opt { label: m.name.clone(), kind: Some("assign".into()), member_sn: Some(m.sn), task_sn: Some(next.sn) }).collect();
        let nb = ProposalNew {
            issue_sn: Some(isn), task_sn: Some(next.sn), member_sn: Some(first.sn), kind: "assign".into(), level: LEVEL, title: format!("#{} {} → {}", next.num, next.title, first.name),
            reason: Some(format!("next after #{}", tk.num)), options: (!opts.is_empty()).then_some(opts), ..Default::default()
        };
        Ok(Some(Pick { nb, task: tk, fail: None }))
    }

    /// Run 실패 → 연결이 문제면 폴백 후보, 아니면 프로필 auto_retry_max 안에서 재시도 후보
    async fn failed(&self, rsn: i64) -> Res<Option<Pick>> {
        let Some(rn) = r::Entity::find_by_id(rsn).one(self.db).await? else { return Ok(None) };
        if rn.parent_run_sn.is_some() || matches!(rn.fail_code.as_deref(), Some("rejected" | "cancelled")) {
            return Ok(None);
        }
        let Some(tk) = t::Entity::find_by_id(rn.task_sn).one(self.db).await? else { return Ok(None) };
        let fail = rn.fail_code.clone();
        let base = ProposalNew { issue_sn: tk.issue_sn, task_sn: Some(tk.sn), run_sn: Some(rsn), member_sn: Some(rn.member_sn), level: LEVEL, ..Default::default() };
        if let Some(cs) = rn.connection_sn
            && !healthy(self.db, cs).await?
        {
            let Some(next) = next_conn(self.db, &rn).await? else { return Ok(None) };
            let nb = ProposalNew { kind: "fallback".into(), title: format!("#{} → {}", tk.num, next.name), reason: Some(format!("connection {cs} unavailable")), ..base };
            return Ok(Some(Pick { nb, task: tk, fail }));
        }
        let Some(m) = mb::Entity::find_by_id(rn.member_sn).one(self.db).await? else { return Ok(None) };
        let max = ap::Entity::find_by_id(m.profile_sn).one(self.db).await?.map_or(0, |p| p.auto_retry_max);
        // 이번 실패까지 센다 (반려 · 취소는 품질 실패가 아니라 뺀다)
        let n = r::Entity::find().filter(r::Column::TaskSn.eq(tk.sn)).filter(r::Column::ParentRunSn.is_null()).filter(r::Column::Status.eq("failed"))
            .filter(Expr::cust("COALESCE(fail_code, '') NOT IN ('rejected', 'cancelled')")).count(self.db).await? as i64;
        if n > max {
            return Ok(None);
        }
        let nb = ProposalNew { kind: "retry".into(), title: format!("#{} {}", tk.num, tk.title), reason: Some(format!("failed {n}/{}", max + 1)), ..base };
        Ok(Some(Pick { nb, task: tk, fail }))
    }

    /// 제안 만들기: 같은 제안이 대기 중이면 건너뜀 · 레벨 처리가 wait/block이면 판단 요청 · 아니면 모드대로 기한 정해 저장 (바로 실행이면 곧바로 run)
    async fn offer(&self, mut nb: ProposalNew) -> Res<Option<i64>> {
        let dup = ak::Entity::find().filter(ak::Column::ProjectSn.eq(self.ps)).filter(ak::Column::Kind.eq("proposal")).filter(ak::Column::Action.eq(nb.kind.as_str())).filter(ak::Column::Status.eq("proposed"))
            .filter(opt(ak::Column::TaskSn, nb.task_sn)).filter(opt(ak::Column::RunSn, nb.run_sn)).filter(opt(ak::Column::IssueSn, nb.issue_sn)).one(self.db).await?;
        if dup.is_some() {
            return Ok(None);
        }
        nb.project_sn = self.ps;
        nb.event_sn = Some(self.ev.sn);
        let lv = self.pol.levels.iter().find(|x| x.level == nb.level);
        let handle = lv.map_or("timer", |x| x.handle.as_str());
        if matches!(handle, "wait" | "block") {
            self.wait(&nb, lv).await?;
            return Ok(None);
        }
        // 가드로 멈춘 뒤(가장 최근에 처리된 제안이 guard_stop)에는 사용자가 하나 처리할 때까지 자동 진행하지 않는다
        let last = ak::Entity::find().filter(ak::Column::ProjectSn.is_in(projects_of(self.db, self.ps).await?)).filter(ak::Column::Kind.eq("proposal")).filter(ak::Column::Status.ne("proposed"))
            .order_by_desc(ak::Column::DecideAt).order_by_desc(ak::Column::Sn).one(self.db).await?;
        let held = last.is_some_and(|x| x.action.as_deref() == Some("guard_stop"));
        let now = !held && self.pol.mode != "manual" && (self.pol.mode == "full_auto" || handle == "auto");
        if !held && !now && self.pol.mode == "auto" {
            nb.deadline_at = Some(at(self.db, &format!("+{} seconds", self.pol.timer_sec)).await?);
        }
        let sn = event::run_as(self.db, "orch", Some(self.orch), async |tx| {
            let (p, ev) = ask::add(tx, nb).await?;
            Ok((p.sn, vec![ev]))
        }).await?;
        if now {
            ask::run(self.db, sn, ask::By::Auto).await?;
        }
        Ok(Some(sn))
    }

    /// 레벨 처리가 wait/block이면 제안 대신 사용자에게 묻는다 (판단 요청 · 응답 반영은 B-18)
    async fn wait(&self, nb: &ProposalNew, lv: Option<&Level>) -> Res<()> {
        let c = |code: &str| Choice { code: code.into(), label: code.into(), note: None, is_recommended: code == "proceed", is_selected: false };
        let deadline = match lv.and_then(|x| x.wait_min) { Some(m) => Some(at(self.db, &format!("+{m} minutes")).await?), None => None };
        let q = Question { title: nb.title.clone(), body: nb.reason.clone(), code_snippet: None, reference: None, options: vec![c("proceed"), c("stop")], answer_text: None, is_delegate: false, answer_at: None };
        ask::decision(self.db, DecisionNew {
            project_sn: self.ps, task_sn: nb.task_sn, run_sn: nb.run_sn, member_sn: self.orch, level: nb.level.max(2), title: nb.title.clone(), deadline_at: deadline, questions: vec![q],
        }).await?;
        Ok(())
    }
}

/// 컬럼 = 값 (값이 없으면 IS NULL)
fn opt<C: ColumnTrait>(c: C, v: Option<i64>) -> SimpleExpr {
    match v {
        Some(v) => c.eq(v),
        None => c.is_null(),
    }
}

/// 같은 팀 프로젝트 번호들 (팀이 없으면 자기 자신)
pub(crate) async fn projects_of(db: &impl ConnectionTrait, ps: i64) -> Res<Vec<i64>> {
    let Some(team) = pj::Entity::find_by_id(ps).one(db).await?.and_then(|x| x.team_sn) else { return Ok(vec![ps]) };
    Ok(pj::Entity::find().filter(pj::Column::TeamSn.eq(team)).all(db).await?.into_iter().map(|x| x.sn).collect())
}

/// 최근에 처리된 제안 중 끊김 없이 이어진 auto_done 수 (사용자가 처리하거나 멈추면 0으로 돌아간다)
pub(crate) async fn streak(db: &impl ConnectionTrait, projects: &[i64]) -> Res<i64> {
    let rows = ak::Entity::find().filter(ak::Column::ProjectSn.is_in(projects.to_vec())).filter(ak::Column::Kind.eq("proposal")).filter(ak::Column::Status.ne("proposed"))
        .order_by_desc(ak::Column::DecideAt).order_by_desc(ak::Column::Sn).limit(200).all(db).await?;
    Ok(rows.iter().take_while(|x| x.status == "auto_done").count() as i64)
}

/// 멤버가 노는 중: 보관 · 멈춤이 아니고 진행 중 리드 Run이 없다
async fn idle(db: &impl ConnectionTrait, m: &mb::Model) -> Res<bool> {
    if matches!(m.status.as_str(), "archived" | "paused") {
        return Ok(false);
    }
    let busy = r::Entity::find().filter(r::Column::MemberSn.eq(m.sn)).filter(r::Column::ParentRunSn.is_null()).filter(r::Column::Status.is_in(runs::ACTIVE)).count(db).await?;
    Ok(busy == 0)
}

/// 연결이 쓸 만하다: 상태가 error · expired · login_required가 아니고 남은 한도 0인 칸이 없다
async fn healthy(db: &impl ConnectionTrait, cs: i64) -> Res<bool> {
    let Some(c) = cn::Entity::find_by_id(cs).one(db).await? else { return Ok(false) };
    if matches!(c.status.as_str(), "error" | "expired" | "login_required") {
        return Ok(false);
    }
    Ok(qt::Entity::find().filter(qt::Column::ConnectionSn.eq(cs)).filter(qt::Column::RemainPercent.eq(0)).count(db).await? == 0)
}

/// Run 멤버 프로필의 폴백 체인에서 지금 연결 다음의 쓸 만한 연결 (체인에 없으면 처음부터)
pub(crate) async fn next_conn(db: &impl ConnectionTrait, rn: &r::Model) -> Res<Option<cn::Model>> {
    let Some(m) = mb::Entity::find_by_id(rn.member_sn).one(db).await? else { return Ok(None) };
    let Some(prof) = ap::Entity::find_by_id(m.profile_sn).one(db).await? else { return Ok(None) };
    let chain = crate::agent::fallbacks_of(&prof);
    let from = rn.connection_sn.and_then(|cur| chain.iter().position(|f| f.connection_sn == cur)).map_or(0, |p| p + 1);
    for f in &chain[from..] {
        if Some(f.connection_sn) != rn.connection_sn && healthy(db, f.connection_sn).await? {
            return Ok(cn::Entity::find_by_id(f.connection_sn).one(db).await?);
        }
    }
    Ok(None)
}

/// 1초마다 `tick` (main이 spawn)
pub async fn ticker(db: DatabaseConnection) {
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        if let Err(err) = ask::tick(&db).await {
            eprintln!("orch timer: {}", err.message());
        }
    }
}

/// 커밋된 이벤트를 받아 `on`에 넘긴다 (main이 spawn). 놓친(lagged) 이벤트는 건너뛴다 — 다음 이벤트가 같은 제안을 다시 만든다
pub async fn listen(db: DatabaseConnection) {
    use tokio::sync::broadcast::error::RecvError;
    let mut rx = event::subscribe();
    loop {
        match rx.recv().await {
            Ok(ev) => {
                if let Err(err) = on(&db, &ev).await {
                    eprintln!("orch rule: {}", err.message());
                }
            }
            Err(RecvError::Lagged(_)) => {}
            Err(RecvError::Closed) => break,
        }
    }
}

/// 대기열 항목
#[derive(Serialize, ToSchema)]
struct QItem {
    task_sn: i64,
    /// 화면 표시 번호
    num: i64,
    title: String,
    priority: i64,
    /// 의존 대기
    waiting: bool,
}

/// 멤버 대기열 (Agent 카드 NOW · NEXT)
#[derive(Serialize, ToSchema)]
struct Queue {
    /// 진행 중 Run의 태스크
    now: Option<QItem>,
    /// 그 멤버의 todo (queue_sort → priority → num)
    next: Vec<QItem>,
}

/// 멤버 대기열: NOW = 진행 중 리드 Run의 태스크 1개, NEXT = 담당 todo (의존 대기 표시 포함). 멤버가 없으면 404
#[utoipa::path(operation_id = "member_queue", get, path = "/members/{sn}/queue", params(("sn" = i64, Path, description = "멤버 번호")), responses((status = 200, body = Queue), (status = "default", body = ErrorBody)))]
async fn queue(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Queue>> {
    mb::Entity::find_by_id(sn).one(&db).await?.ok_or_else(Error::not_found)?;
    let run = r::Entity::find().filter(r::Column::MemberSn.eq(sn)).filter(r::Column::ParentRunSn.is_null()).filter(r::Column::Status.is_in(runs::ACTIVE))
        .order_by_desc(r::Column::Sn).one(&db).await?;
    let now = match run { Some(x) => t::Entity::find_by_id(x.task_sn).one(&db).await?, None => None };
    let next = t::Entity::find().filter(t::Column::MemberSn.eq(sn)).filter(t::Column::Status.eq("todo"))
        .order_by_asc(Expr::cust("queue_sort IS NULL")).order_by_asc(t::Column::QueueSort).order_by_asc(t::Column::Priority).order_by_asc(t::Column::Num).all(&db).await?;
    let wait = task::waiting(&db, now.iter().chain(&next).map(|x| x.sn).collect()).await?;
    let item = |x: &t::Model| QItem { task_sn: x.sn, num: x.num, title: x.title.clone(), priority: x.priority, waiting: wait.contains(&x.sn) };
    Ok(Json(Queue { now: now.as_ref().map(item), next: next.iter().map(item).collect() }))
}
