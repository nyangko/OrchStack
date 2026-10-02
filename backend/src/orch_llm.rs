//! Orch가 모델을 부르는 두 곳 (#122): ① 사용자 자유 글(PM Dock 메시지) → WorkProposal 1개, ② 기한이 지난 판단 요청 → Orch가 근거와 함께 대신 결정.
//! 나머지 PM 판단은 orch_rule(LLM 0)이 한다. 입력은 protocol + Orch role 프리셋(같은 바이트) + 프로젝트 요약 + 팀 멤버 + 사용자 원문뿐이다
//! (대화 기록 · 코드 파일 · 이전 제안은 보내지 않는다 · 요약을 위한 모델 호출도 없다). 호출은 tbl_log_token(run_sn NULL) + OrchCalled 이벤트로 남는다
use crate::{agent::ToolRule, ask::{self, Ask, Choice, DecisionNew, Question}, context::END, entity::{tbl_agent_profile as ap, tbl_ask as ak, tbl_instruction_preset as ip, tbl_issue as i, tbl_log_event as e, tbl_log_token as lt,
    tbl_map_profile_preset as mp, tbl_member as mb, tbl_message as ms, tbl_model, tbl_runtime as rt, tbl_task as t},
    error::{Error, ErrorBody, Res, Sn}, event::{self, Ev}, exec::{self, Executor, Job, Status}, orch::{self, Message, Plan}, orch_rule::at, policy, preset::tokens, runner, team};
use axum::{Json, extract::State};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, sea_query::Expr};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{collections::HashSet, path::PathBuf, sync::{LazyLock, Mutex}, time::Duration};
use tokio::sync::oneshot;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

/// 호출당 입력 상한(토큰). 넘으면 프로젝트 요약 줄 수를 줄인다 (모델로 요약하지 않는다)
const CAP: i64 = 8_000;
/// 프로젝트 요약 최대 줄 수
const LINES: usize = 50;
/// 호출 제한 시간
const TIMEOUT: Duration = Duration::from_secs(180);
/// 판단 대행 중인 ask (같은 ask를 두 번 부르지 않는다)
static BUSY: LazyLock<Mutex<HashSet<i64>>> = LazyLock::new(Mutex::default);

/// 출력 형식 지침 (입력 맨 끝)
const PLAN_FMT: &str = "Reply with `@PLAN v1` and then one JSON object only, no other text: {\"issue\":{\"title\":\"..\",\"body\":\"..\"},\"tasks\":[{\"title\":\"..\",\"description\":\"..\",\"member_sn\":<sn from @TEAM or null>}]}";
const DECIDE_FMT: &str = "Reply with `@DECIDE v1` and then one JSON object only, no other text: {\"choice\":[{\"question\":<index>,\"option\":\"<code>\"}],\"reason\":\"<one line>\"}";

/// 프리셋 시드 (파일이 비어 있을 때만 쓰는 최소 지침 · 영어 10줄 이내)
const PROTOCOL_MIN: &str = "# OrchStack protocol\nReply only through the requested block (`@PLAN v1` or `@DECIDE v1`) followed by one JSON object.\nNo chat, no greetings.";
const ROLE_MIN: &str = "# Role: Orch (PM)\nSplit work into small tasks one agent can finish in one run. Prefer fewer tasks.\nFor decisions pick the safest reversible option and give a one-line reason.";
/// 시드할 프리셋 (종류, 키, 이름, 파일 본문, 비었을 때 상수)
const SEEDS: [(&str, &str, &str, &str, &str); 2] = [
    ("protocol", "orch-protocol", "OrchStack protocol", include_str!("../../data/presets/protocol/orch-protocol.md"), PROTOCOL_MIN),
    ("role", "orch", "Orch (PM)", include_str!("../../data/presets/role/orch.md"), ROLE_MIN),
];

/// orch_llm 관련 경로 묶음
pub fn routes() -> OpenApiRouter<DatabaseConnection> {
    OpenApiRouter::new().routes(routes!(replan))
}

/// Orch 호출에 쓰는 설정: Orch 멤버 · 실행기 · 연결 · 모델 · 고정 접두
struct Orch {
    member: mb::Model,
    ex: &'static dyn Executor,
    bin: Option<PathBuf>,
    conn: Option<i64>,
    model: Option<String>,
    args: Vec<String>,
    /// protocol → role 프리셋 본문 (바이트 고정 · 캐시 접두)
    prefix: String,
}

/// 프리셋 .md → (상한, 본문). 앞머리(--- … ---)를 떼고 limit_tok를 읽는다. 본문이 비면 상수
fn split(md: &str, min: &str) -> (i64, String) {
    let md = md.replace("\r\n", "\n");
    let (head, body) = md.strip_prefix("---\n").and_then(|r| r.split_once("\n---\n")).unwrap_or(("", &md));
    let limit = head.lines().find_map(|l| l.strip_prefix("limit_tok:")).and_then(|v| v.trim().parse().ok()).unwrap_or(400);
    let body = body.trim();
    (limit, if body.is_empty() { min.to_owned() } else { body.to_owned() })
}

/// protocol · Orch role 프리셋이 DB에 없으면 data/presets의 .md로 넣는다 (최신 행이 있으면 건드리지 않는다)
async fn seed(db: &DatabaseConnection) -> Res<()> {
    event::run_as(db, "system", None, async |tx| {
        for (kind, key, name, md, min) in SEEDS {
            if ip::Entity::find().filter(ip::Column::Kind.eq(kind)).filter(ip::Column::PresetKey.eq(key)).one(tx).await?.is_some() {
                continue;
            }
            let (limit, body) = split(md, min);
            ip::ActiveModel {
                workspace_sn: Set(crate::WORKSPACE), kind: Set(kind.into()), preset_key: Set(key.into()), name: Set(name.into()), limit_tok: Set(limit), is_builtin: Set(1),
                is_locked: Set((kind == "protocol") as i64), token_count: Set(tokens(&body)), content: Set(body), source: Set("builtin".into()), ..Default::default()
            }.insert(tx).await?;
        }
        Ok(((), vec![]))
    }).await
}

/// Orch 멤버의 프리셋 접두: 프로필에 연결한(고정 버전) protocol · role 프리셋, 없으면 시드한 최신 행. protocol → role 순 · 본문 그대로 + 구분자
async fn prefix(db: &DatabaseConnection, profile_sn: i64) -> Res<String> {
    seed(db).await?;
    let linked: Vec<i64> = mp::Entity::find().filter(mp::Column::ProfileSn.eq(profile_sn)).filter(mp::Column::IsEnabled.eq(1)).all(db).await?.into_iter().map(|m| m.preset_sn).collect();
    let mut rows = ip::Entity::find().filter(ip::Column::Sn.is_in(linked)).filter(ip::Column::Kind.is_in(["protocol", "role"])).all(db).await?;
    if rows.is_empty() {
        rows = ip::Entity::find().filter(ip::Column::IsLatest.eq(1)).filter(ip::Column::PresetKey.is_in(["orch-protocol", "orch"])).filter(ip::Column::Kind.is_in(["protocol", "role"])).all(db).await?;
    }
    rows.sort_by_key(|r| (r.kind != "protocol", r.sn));
    Ok(rows.into_iter().map(|r| format!("{}{END}", r.content)).collect())
}

/// 프로젝트 팀의 Orch 설정. Orch 멤버 · 실행기가 없거나 실행기를 모르면 409 no_orch
async fn setup(db: &DatabaseConnection, project_sn: i64) -> Res<Orch> {
    let sn = orch::orch_of(db, project_sn).await?.ok_or_else(Error::no_orch)?;
    let member = mb::Entity::find_by_id(sn).one(db).await?.ok_or_else(Error::no_orch)?;
    let prof = ap::Entity::find_by_id(member.profile_sn).one(db).await?.ok_or_else(Error::no_orch)?;
    let rt = match prof.runtime_sn { Some(x) => rt::Entity::find_by_id(x).one(db).await?, None => None }.ok_or_else(Error::no_orch)?;
    let ex = exec::pick(&rt.code).ok_or_else(Error::no_orch)?;
    let model = match prof.model_sn { Some(x) => tbl_model::Entity::find_by_id(x).one(db).await?.map(|m| m.code), None => None };
    // 지침 · MCP · 스킬을 막고 도구도 쓰지 않는다 (Orch는 글만 낸다)
    let tools = [ToolRule { tool_code: "shell".into(), scope_text: None, policy: "block".into() }];
    let args = runner::block(&rt.code).into_iter().chain(runner::perms(&rt.code, &tools, &[])).collect();
    Ok(Orch { member, ex, bin: rt.bin_path.map(Into::into), conn: prof.connection_sn, model, args, prefix: prefix(db, prof.sn).await? })
}

/// 제목 한 줄 (80자까지)
fn one(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ").chars().take(80).collect()
}

/// 프로젝트 요약 줄: 열린 이슈 · 안 끝난 태스크 한 줄씩 (이슈 → 태스크 · 번호순)
async fn lines(db: &DatabaseConnection, project_sn: i64) -> Res<Vec<String>> {
    let mut out: Vec<String> = i::Entity::find().filter(i::Column::ProjectSn.eq(project_sn)).filter(i::Column::Status.is_in(["open", "in_progress"])).order_by_asc(i::Column::Num).all(db).await?
        .into_iter().map(|x| format!("issue #{} {} [{}]", x.num, one(&x.title), x.status)).collect();
    out.extend(t::Entity::find().filter(t::Column::ProjectSn.eq(project_sn)).filter(t::Column::Status.is_not_in(["done", "cancelled"])).order_by_asc(t::Column::Num).all(db).await?
        .into_iter().map(|x| format!("task #{} {} [{}] @{}", x.num, one(&x.title), x.status, x.member_sn.map_or("-".into(), |m| m.to_string()))));
    Ok(out)
}

/// 팀 멤버 한 줄씩: sn · 이름 · 역할 · 작업 상태 (Orch · 보관 멤버 제외)
async fn team_lines(db: &DatabaseConnection, project_sn: i64) -> Res<Vec<(i64, String)>> {
    let Some(ts) = crate::entity::tbl_project::Entity::find_by_id(project_sn).one(db).await?.and_then(|p| p.team_sn) else { return Ok(vec![]) };
    let ms: Vec<mb::Model> = mb::Entity::find().filter(mb::Column::TeamSn.eq(ts)).filter(mb::Column::IsOrch.eq(0)).filter(mb::Column::Status.ne("archived")).order_by_asc(mb::Column::Sn).all(db).await?;
    let work = team::work_of(db, &ms, true).await?;
    Ok(ms.into_iter().map(|m| (m.sn, format!("{} {} / {} / {}", m.sn, one(&m.name), one(&m.role_name), work[&m.sn]))).collect())
}

/// 자유 글 → Plan 입력과 추정 토큰. 추정 > 8K면 프로젝트 요약 줄을 절반씩 줄인다 (원문만으로 넘으면 422 context_over)
pub(crate) async fn input_plan(db: &DatabaseConnection, project_sn: i64, text: &str, prefix: &str) -> Res<(String, i64)> {
    let all = lines(db, project_sn).await?;
    let team = team_lines(db, project_sn).await?.into_iter().map(|x| x.1).collect::<Vec<_>>().join("\n");
    let mut n = all.len().min(LINES);
    loop {
        let body = format!("{prefix}@PROJECT\n{}\n\n@TEAM\n{team}\n\n@USER\n{text}\n\n{PLAN_FMT}\n", all[..n].join("\n"));
        let est = tokens(&body);
        if est <= CAP {
            return Ok((body, est));
        }
        if n == 0 {
            return Err(Error::over(format!("orch input {est} > cap {CAP}")));
        }
        n /= 2;
    }
}

/// 마지막 `tag` 블록 뒤의 JSON 객체 하나 → T. 없거나 틀리면 한 줄 오류
fn block<T: serde::de::DeserializeOwned>(text: &str, tag: &str) -> Result<T, String> {
    let rest = &text[text.rfind(tag).ok_or_else(|| format!("{tag} block missing"))? + tag.len()..];
    let json = &rest[rest.find('{').ok_or("json object missing")?..];
    serde_json::Deserializer::from_str(json).into_iter::<T>().next().ok_or("json object missing")?.map_err(|e| e.to_string().replace('\n', " ").chars().take(160).collect())
}

/// 모델 호출 1번: 실행 → 토큰 기록(실행기가 usage를 안 주면 추정) · 연결 갱신 · OrchCalled 이벤트. 실행기를 못 찾은 경우(Missing)는 기록하지 않는다
async fn call(db: &DatabaseConnection, o: &Orch, project_sn: i64, prompt: &str, est: i64, at: (&'static str, i64, &'static str)) -> Res<exec::Outcome> {
    let (agg, agg_sn, kind) = at; // 이벤트 대상 종류 · 번호 · 호출 종류(plan | decide)
    let job = Job { prompt: prompt.into(), cwd: std::env::temp_dir(), bin: o.bin.clone(), model: o.model.clone(), args: o.args.clone(), timeout: TIMEOUT, ..Default::default() };
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let (_stop, stop_rx) = oneshot::channel::<()>(); // 보내는 쪽이 사라져도 취소하지 않는다
    let out = exec::run(o.ex, &job, &tx, stop_rx).await;
    if out.status == Status::Missing {
        return Ok(out);
    }
    let u = out.usage;
    let real = u.input + u.output + u.cache_read + u.cache_write > 0;
    event::run_as(db, "orch", Some(o.member.sn), async |tx| {
        lt::ActiveModel {
            connection_sn: Set(o.conn), model_code: Set(o.model.clone()), context_token: Set(Some(est)), usage_source: Set(if real { "provider" } else { "estimated" }.into()),
            token_input: Set(if real { u.input } else { est }), token_cache_read: Set(u.cache_read), token_cache_write: Set(u.cache_write),
            token_output: Set(if real { u.output } else { out.text.as_deref().map_or(0, tokens) }), ..Default::default()
        }.insert(tx).await?;
        crate::context::touch(tx, o.conn).await?;
        Ok(((), vec![Ev::new(Some(project_sn), agg, agg_sn, "OrchCalled", &json!({ "kind": kind, "est_input": est, "ok": out.status == Status::Done }))]))
    }).await?;
    Ok(out)
}

/// 자유 글 처리 결과: 만든 작업 제안(초안) 또는 실패해서 만든 판단 요청
#[derive(Serialize, ToSchema)]
pub struct Planned {
    pub(crate) proposal: Option<Message>,
    /// 두 번 다 형식이 틀렸을 때 사용자에게 묻는 판단 요청 (질문 plan_failed · 선택지 retry · manual)
    pub(crate) ask_sn: Option<i64>,
}

/// 사용자 메시지 하나를 작업 제안 1개로: 조립 → 호출 → 마지막 `@PLAN v1` JSON → `orch::propose`(행위자 orch).
/// 형식 오류면 같은 입력 + 오류 한 줄로 1회만 재시도하고, 또 틀리거나 실행기가 실패하면 판단 요청(plan_failed)을 만든다.
/// 메시지가 사용자 글(text)이 아니면 409 · 빈 글 422 · Orch 없음 409 no_orch
pub async fn plan(db: &DatabaseConnection, message_sn: i64) -> Res<Planned> {
    let m = ms::Entity::find_by_id(message_sn).one(db).await?.ok_or_else(Error::not_found)?;
    if m.sender_type != "user" || m.kind != "text" {
        return Err(Error::conflict(format!("message {message_sn} is not a user text")));
    }
    let text = m.content.clone().filter(|c| !c.trim().is_empty()).ok_or_else(|| Error::invalid("message is empty".into()))?;
    let ps = m.project_sn;
    let o = setup(db, ps).await?;
    let (prompt, est) = input_plan(db, ps, &text, &o.prefix).await?;
    let mut why = String::new();
    for attempt in 0..2 {
        let input = if attempt == 0 { prompt.clone() } else { format!("{prompt}@ERROR {why}\n") };
        let out = call(db, &o, ps, &input, est, ("message", message_sn, "plan")).await?;
        if out.status != Status::Done {
            why = "exec".into();
            break; // 실행기 실패는 같은 입력으로 다시 해도 같다 — 재시도 없이 사용자에게
        }
        match block::<Plan>(out.text.as_deref().unwrap_or_default(), "@PLAN v1").and_then(|p| orch::check(&p).map(|()| p).map_err(|e| e.message().to_owned())) {
            Ok(mut p) => {
                // 팀에 없는 멤버 번호는 배정 없음으로
                let members: HashSet<i64> = team_lines(db, ps).await?.into_iter().map(|x| x.0).collect();
                p.tasks.iter_mut().for_each(|x| x.member_sn = x.member_sn.filter(|s| members.contains(s)));
                return Ok(Planned { proposal: Some(orch::propose(db, ps, None, p).await?), ask_sn: None });
            }
            Err(e) => why = e,
        }
    }
    // 사람용 문구는 만들지 않는다: 질문 코드 plan_failed · 선택지 retry | manual · 원인 코드는 body
    let ch = |code: &str| Choice { code: code.into(), label: code.into(), note: None, is_recommended: code == "retry", is_selected: false };
    let q = Question { title: "plan_failed".into(), body: Some(why), code_snippet: None, reference: Some(json!({ "message_sn": message_sn })), options: vec![ch("retry"), ch("manual")], answer_text: None, is_delegate: false, answer_at: None };
    let a = ask::decision(db, DecisionNew { project_sn: ps, task_sn: None, run_sn: None, member_sn: o.member.sn, level: 2, title: "plan_failed".into(), deadline_at: None, questions: vec![q] }).await?;
    Ok(Planned { proposal: None, ask_sn: Some(a.sn) })
}

/// 수동 재시도: 사용자 글을 다시 작업 제안으로 (plan_failed 판단에서 retry를 고른 뒤 · 같은 입력 규칙). 사용자 글이 아니면 409 · Orch 없음 409 no_orch
#[utoipa::path(operation_id = "orch_replan", post, path = "/messages/{sn}/plan", params(("sn" = i64, Path, description = "메시지 번호")), responses((status = 200, body = Planned), (status = "default", body = ErrorBody)))]
async fn replan(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Planned>> {
    plan(&db, sn).await.map(Json)
}

/// 이벤트 하나 처리: 사용자 글 MessagePosted면 그 메시지 뒤에 Orch 메시지가 아직 없을 때만 `plan` (같은 글로 두 번 부르지 않는다)
pub async fn on(db: &DatabaseConnection, ev: &e::Model) -> Res<Option<Planned>> {
    let p: Value = serde_json::from_str(&ev.payload_json).unwrap_or_default();
    let (Some(ps), "MessagePosted", "user", Some("text")) = (ev.project_sn, ev.event_type.as_str(), ev.actor_type.as_str(), p["kind"].as_str()) else { return Ok(None) };
    let later = ms::Entity::find().filter(ms::Column::ProjectSn.eq(ps)).filter(ms::Column::Sn.gt(ev.aggregate_sn)).filter(ms::Column::SenderType.eq("orch")).one(db).await?;
    if later.is_some() {
        return Ok(None);
    }
    plan(db, ev.aggregate_sn).await.map(Some)
}

/// 커밋된 이벤트를 받아 사용자 글마다 `on`을 백그라운드로 돌린다 (main이 spawn · orch_rule::listen 옆). 실패는 로그만 (Orch가 없으면 글을 보낼 때 이미 409)
pub async fn listen(db: DatabaseConnection) {
    use tokio::sync::broadcast::error::RecvError;
    let mut rx = event::subscribe();
    loop {
        match rx.recv().await {
            Ok(ev) => {
                let db = db.clone();
                tokio::spawn(async move { if let Err(err) = on(&db, &ev).await { eprintln!("orch plan: {}", err.message()); } });
            }
            Err(RecvError::Lagged(_)) => {}
            Err(RecvError::Closed) => break,
        }
    }
}

/// BUSY 자리 반납
struct Busy(i64);
impl Drop for Busy {
    fn drop(&mut self) {
        BUSY.lock().unwrap_or_else(|p| p.into_inner()).remove(&self.0);
    }
}

/// 질문마다 추천 선택지(없으면 첫 번째) = (질문 순번, code)
fn recommended(qs: &[Question]) -> Vec<(usize, String)> {
    qs.iter().enumerate().filter_map(|(i, q)| q.options.iter().find(|c| c.is_recommended).or(q.options.first()).map(|c| (i, c.code.clone()))).collect()
}

/// 판단 대행 입력: 프리셋 접두 + 질문 · 선택지 · 추천 표시만
fn input_decide(prefix: &str, a: &ak::Model, qs: &[Question]) -> String {
    let body: String = qs.iter().enumerate().map(|(i, q)| {
        let opts: String = q.options.iter().map(|c| format!("  {} {}{}{}\n", c.code, one(&c.label), if c.is_recommended { " *recommended" } else { "" }, c.note.as_deref().map_or(String::new(), |n| format!(" | {}", one(n))))).collect();
        format!("Q{i} {}\n{}{opts}", one(&q.title), q.body.as_deref().map_or(String::new(), |b| format!("{}\n", one(b))))
    }).collect();
    format!("{prefix}@DECISION level: L{} title: {}\n{body}\n{DECIDE_FMT}\n", a.level, one(&a.title))
}

/// `@DECIDE v1` 출력 → 모든 질문에 정확히 한 번씩 고른 (순번, code) + 근거. 틀리면 한 줄 오류
fn parse_decide(text: &str, qs: &[Question]) -> Result<(Vec<(usize, String)>, String), String> {
    #[derive(Deserialize)]
    struct Pick { question: usize, option: String }
    #[derive(Deserialize)]
    struct D { choice: Vec<Pick>, reason: String }
    let d: D = block(text, "@DECIDE v1")?;
    let got: HashSet<usize> = d.choice.iter().map(|p| p.question).collect();
    if got != (0..qs.len()).collect() || got.len() != d.choice.len() || d.choice.iter().any(|p| !qs[p.question].options.iter().any(|c| c.code == p.option)) {
        return Err("choice must pick one known option for every question".into());
    }
    Ok((d.choice.into_iter().map(|p| (p.question, p.option)).collect(), d.reason.split_whitespace().collect::<Vec<_>>().join(" ").chars().take(200).collect()))
}

/// 판단 요청 하나를 대신 결정한다. 판단의 pending · 기한 경과 · 타이머 안 멈춤일 때만, 레벨 정책의 `no_reply`가
/// orch_decide면 모델 1회 호출(`@DECIDE`) · proceed면 호출 없이 추천 선택지 · keep_wait · none이면 아무것도 안 한다 (None).
/// 호출이 실패하거나 형식이 틀리면 추천 선택지로 결정하고 근거에 코드만 남긴다 (같은 판단을 다시 부르지 않는다).
/// 결정한 뒤 레벨 wait로 만든 판단이면 원래 제안을 이어 실행한다 (`ask::follow`)
pub async fn decide(db: &DatabaseConnection, ask_sn: i64) -> Res<Option<Ask>> {
    let a = ask::row(db, ask_sn).await?;
    let Some(deadline) = a.deadline_at.as_deref() else { return Ok(None) };
    if a.kind != "decision" || a.status != "pending" || a.is_timer_pause == 1 || deadline > at(db, "+0 seconds").await?.as_str() {
        return Ok(None);
    }
    let qs = ask::questions(&a);
    // 레벨 wait로 만든 판단은 원래 제안의 레벨 정책을 따른다 (판단 요청 레벨은 L2 이상으로 올려 둔 값)
    let level = qs.first().and_then(|q| q.reference.as_ref()).and_then(|r| r["proposal"]["level"].as_i64()).unwrap_or(a.level);
    let Some(pol) = policy::load(db, a.project_sn).await? else { return Ok(None) };
    let mode = pol.levels.iter().find(|l| l.level == level).and_then(|l| l.no_reply.clone());
    if !matches!(mode.as_deref(), Some("orch_decide" | "proceed")) {
        return Ok(None);
    }
    if !BUSY.lock().unwrap_or_else(|p| p.into_inner()).insert(ask_sn) {
        return Ok(None);
    }
    let _busy = Busy(ask_sn);
    let (picks, reason) = if mode.as_deref() == Some("proceed") {
        (recommended(&qs), "proceed".to_owned())
    } else {
        match ask_model(db, &a, &qs).await {
            Ok(x) => x,
            Err(code) => (recommended(&qs), format!("fallback {code}")),
        }
    };
    let out = ask::orch_decided(db, ask_sn, &picks, &reason).await?;
    if let Err(err) = ask::follow(db, ask_sn).await {
        eprintln!("ask follow: {}", err.message());
    }
    Ok(Some(out))
}

/// 판단 대행 모델 호출 1회 → (선택, 근거). 실패는 코드(no_orch · exec · db · 형식 오류 한 줄)
async fn ask_model(db: &DatabaseConnection, a: &ak::Model, qs: &[Question]) -> Result<(Vec<(usize, String)>, String), String> {
    let o = setup(db, a.project_sn).await.map_err(|_| "no_orch".to_owned())?;
    let input = input_decide(&o.prefix, a, qs);
    let out = call(db, &o, a.project_sn, &input, tokens(&input), ("ask", a.sn, "decide")).await.map_err(|_| "db".to_owned())?;
    if out.status != Status::Done {
        return Err("exec".into());
    }
    parse_decide(out.text.as_deref().unwrap_or_default(), qs)
}

/// 기한이 지난 판단 요청을 모두 `decide`한다 (1초 루프가 부른다). 결정한 수를 돌려준다. 개별 실패는 로그만
pub async fn sweep(db: &DatabaseConnection) -> Res<u32> {
    let due = ak::Entity::find().filter(ak::Column::Kind.eq("decision")).filter(ak::Column::Status.eq("pending")).filter(ak::Column::IsTimerPause.eq(0))
        .filter(Expr::cust("deadline_at IS NOT NULL AND datetime(deadline_at) <= datetime('now')")).order_by_asc(ak::Column::Sn).all(db).await?;
    let mut n = 0;
    for a in due {
        match decide(db, a.sn).await {
            Ok(Some(_)) => n += 1,
            Ok(None) => {}
            Err(err) => eprintln!("orch decide {}: {}", a.sn, err.message()),
        }
    }
    Ok(n)
}
