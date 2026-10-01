//! runner 실행 (#67): 리드 @TASK → 규칙 검사 → 하위 Run 생성 → 최소 입력 · 지침 차단으로 실행 → 리드에게는 @REPORT 기계 칸만 돌려준다.
//! 입력은 고정 규칙 + @TASK + paths 파일뿐이다 (저장소 지침 md · 스킬 · MCP · checkpoint · 이슈 전문 없음 · ContextManifest에 남긴다)
#![allow(dead_code)] // 리드 Run 디스패처가 호출한다

use crate::{
    entity::{tbl_context_manifest as cm, tbl_context_source as cs, tbl_log_token as lt, tbl_map_fallback as fb, tbl_member, tbl_model, tbl_project, tbl_run as r, tbl_runtime, tbl_session as s, tbl_task, tbl_team, tbl_workspace},
    error::{Error, Res},
    event::{self, Ev},
    exec::{self, Job, Status},
    rule::{self, Brief, Ctx, Stop, Wait},
    run,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection, DatabaseTransaction, DbBackend, EntityTrait, QueryFilter, QueryOrder, Statement, sea_query::Expr,
};
use serde_json::json;
use std::{path::Path, time::Duration};
use tokio::sync::{mpsc, oneshot};

/// 하위 작업자 고정 규칙 (≈200 tok). 항상 프롬프트 맨 앞에 같은 글자로 둔다 (캐시 접두어)
const RULES: &str = "\
You are a sub-task worker. Rules:
- Do only the @TASK below. Edit only files in `paths`. Read nothing else.
- Follow the existing code style of the given files. Smallest change that meets every `ac`.
- No git commands (commit, stash, checkout, reset). No new dependencies.
- If you cannot finish without another file or a decision, stop and reply with one `@ASK v1` block (`need: [path, ...]` or `q: ...`).
- When done, reply with only this block (English, no prose, no diff):
@REPORT v1
id: <task id>  run: <parent>  status: done|partial|failed|blocked
ac 1 ok | ac 2 fail \"short reason\"
left todo \"short\"   (optional, one per line; also risk perf|security|data|compat \"short\")
";
/// paths 파일 하나에 넣는 최대 바이트 (넘으면 앞부분만)
const FILE_MAX: usize = 64 * 1024;

/// 실행기별 지침 차단 인자 (Job.args). 저장소 · 사용자 지침 md · 메모리 · 설정 파일 hooks · MCP · 스킬을 끈다.
/// Claude는 canary(CLAUDE.md "reply BANANA")로 확인 (2.1.286). Codex는 미검증 · CODEX_HOME 아래 사용자 AGENTS.md는 막지 못한다 (#60)
pub fn block(code: &str) -> Vec<String> {
    let a: &[&str] = match code {
        "claude_code" => &[
            "--setting-sources", "",
            "--settings", r#"{"claudeMdExcludes":["**/CLAUDE.md","**/CLAUDE.local.md","**/.claude/**"],"autoMemoryEnabled":false}"#,
            "--strict-mcp-config", "--disable-slash-commands", "--disallowedTools", "mcp__*",
            "--permission-mode", "acceptEdits", "--permission-prompts", "none",
        ],
        "codex" => &["--ignore-user-config", "--ignore-rules", "--ephemeral", "--disable", "plugins", "-c", "project_doc_max_bytes=0", "--sandbox", "workspace-write"],
        _ => &[],
    };
    a.iter().map(|s| s.to_string()).collect()
}

/// 최소 입력: 고정 규칙 → @TASK 원문 → paths 파일 (경로, 내용 · None = 아직 없는 파일 · glob)
pub fn prompt(brief: &str, files: &[(String, Option<String>)]) -> String {
    let mut p = format!("{RULES}\n{}\n", brief.trim());
    for (path, body) in files {
        match body {
            Some(b) => p += &format!("\n--- {path}\n{b}\n"),
            None => p += &format!("\n--- {path} (not read: new file or glob)\n"),
        }
    }
    p
}

/// 실행기 응답에서 찾은 @REPORT
#[derive(Debug, PartialEq)]
pub struct Report {
    /// done | partial | failed | blocked
    pub status: String,
    /// 리드에게 줄 기계 칸 (`--- human` 아래 · 다른 글은 뺀다)
    pub lead: String,
}

/// 응답에서 마지막 `@REPORT v1` 블록을 찾는다. 헤더 다음 줄에 status가 없거나 값이 틀리면 None
pub fn report(text: &str) -> Option<Report> {
    let block = &text[text.rfind("@REPORT v1")?..];
    let lines: Vec<&str> = block.lines().map(str::trim_end).take_while(|l| !l.starts_with("--- human") && !l.starts_with("```")).filter(|l| !l.trim().is_empty()).collect();
    let status = lines.get(1)?.split("  ").find_map(|kv| kv.trim().strip_prefix("status:")).map(str::trim)?;
    ["done", "partial", "failed", "blocked"].contains(&status).then(|| Report { status: status.into(), lead: lines.join("\n") })
}

/// 실패를 리드에게 같은 @REPORT 모양으로 (리드는 @REPORT만 받는다)
fn failed(id: &str, parent: &str, code: &str) -> String {
    format!("@REPORT v1\nid: {id}  run: {parent}  status: failed\nleft blocked \"{code}\"")
}

/// 하위 Run 생성 결과
#[derive(Debug)]
pub enum Spawn {
    /// 만든 하위 Run (wait_run_sn · 상한이면 queued로 기다린다)
    Run(r::Model),
    /// 만들지 않음 (리드 반려 · 사람 판단)
    Stop(Stop),
}

/// 숫자 하나를 돌려주는 SQL
async fn count(tx: &DatabaseTransaction, sql: &str, v: i64) -> Res<i64> {
    let row = tx.query_one_raw(Statement::from_sql_and_values(DbBackend::Sqlite, sql, [v.into()])).await?;
    Ok(row.map(|r| r.try_get_by_index::<i64>(0)).transpose()?.unwrap_or(0))
}

/// 리드 Run이 보낸 @TASK로 하위 Run을 만든다: 파싱 · 규칙 검사 → (runner면) 등급에 맞는 폴백 단계 → tbl_run 행 + RunStarted
pub async fn spawn(db: &DatabaseConnection, lead_sn: i64, text: &str) -> Res<Spawn> {
    let b = match rule::parse(text) {
        Ok(b) => b,
        Err(s) => return Ok(Spawn::Stop(s)),
    };
    event::run(db, async |tx| {
        let lead = r::Entity::find_by_id(lead_sn).one(tx).await?.ok_or_else(Error::not_found)?;
        if lead.parent_run_sn.is_some() {
            return Ok((Spawn::Stop(Stop::Lead("sub-task runs cannot spawn".into())), vec![]));
        }
        let task = tbl_task::Entity::find_by_id(lead.task_sn).one(tx).await?.ok_or_else(Error::not_found)?;
        let member = tbl_member::Entity::find_by_id(lead.member_sn).one(tx).await?.ok_or_else(Error::not_found)?;
        let team = tbl_team::Entity::find_by_id(member.team_sn).one(tx).await?.ok_or_else(Error::not_found)?;
        let kids = r::Entity::find().filter(r::Column::ParentRunSn.eq(lead_sn)).all(tx).await?;
        let live: Vec<&r::Model> = kids.iter().filter(|k| run::ACTIVE.contains(&k.status.as_str())).collect();
        let siblings: Vec<(i64, Vec<String>)> = live.iter().map(|k| (k.sn, granted(k))).collect();
        // 같은 id의 앞선 실패 수 (등급 +1 · 2번째부터 판단 요청)
        let retry = kids.iter().filter(|k| k.status == "failed" && k.brief.as_deref().and_then(|t| rule::parse(t).ok()).is_some_and(|o| o.id == b.id)).count() as u8;
        let team_run = count(tx, "SELECT COUNT(*) FROM tbl_run r JOIN tbl_member m ON m.sn = r.member_sn WHERE m.team_sn = ? AND r.status IN ('starting','running','waiting')", team.sn).await?;
        // ponytail: 오늘(UTC) 쓴 토큰 = 새 입력 + 캐시 쓰기 + 출력 (캐시 읽기 제외 · #60). 워크스페이스 시간대 기준은 일일 리셋 구현 때
        let used = count(tx, "SELECT COALESCE(SUM(t.token_input + t.token_cache_write + t.token_output), 0) FROM tbl_log_token t JOIN tbl_run r ON r.sn = t.run_sn JOIN tbl_member m ON m.sn = r.member_sn WHERE m.team_sn = ? AND date(t.create_at) = date('now')", team.sn).await?;
        let ctx = Ctx {
            lead_num: lead.num, team_mode: &team.spawn_mode, allow: &team.spawn_allow, task_mode: task.spawn_mode.as_deref(), retry,
            budget: team.daily_token_budget, used, children: live.len() as i64, max_child: team.max_child_run, team_run, team_max: team.max_concurrent_run,
            siblings: &siblings,
        };
        let plan = match rule::check(&b, &ctx) {
            Ok(p) => p,
            Err(s) => return Ok((Spawn::Stop(s), vec![])),
        };
        // runner: 리드 프로필 폴백 체인에서 등급이 맞는 첫 단계 (리드는 모델을 고르지 않는다)
        let mut step = None;
        if let Some(t) = plan.tier {
            let chain = fb::Entity::find().filter(fb::Column::ProfileSn.eq(member.profile_sn)).order_by_asc(fb::Column::Sort).all(tx).await?;
            let Some(m) = rule::models(&chain, t).next().cloned() else {
                return Ok((Spawn::Stop(Stop::Judge(format!("no fallback step for tier {t}"))), vec![]));
            };
            let code = match m.model_sn { Some(sn) => tbl_model::Entity::find_by_id(sn).one(tx).await?.map(|x| x.code), None => None };
            step = Some((m, code));
        }
        let (wait_run_sn, wait_glob) = match &plan.wait { Some(Wait::Paths(sn, p)) => (Some(*sn), Some(p.clone())), _ => (None, None) };
        let at = tx.query_one_raw(Statement::from_string(DbBackend::Sqlite, "SELECT datetime('now')")).await?.map(|r| r.try_get_by_index::<String>(0)).transpose()?;
        let paths = json!(b.paths.iter().map(|p| json!({"path": p, "source": "brief", "at": at})).collect::<Vec<_>>()).to_string();
        let num = r::Entity::find().filter(r::Column::ProjectSn.eq(lead.project_sn)).order_by_desc(r::Column::Num).one(tx).await?.map_or(1, |m| m.num + 1);
        let seq = kids.iter().filter_map(|k| k.child_seq).max().unwrap_or(0) + 1;
        let m = r::ActiveModel {
            project_sn: Set(lead.project_sn), task_sn: Set(lead.task_sn), member_sn: Set(lead.member_sn), num: Set(num),
            start_by: Set("lead".into()), parent_run_sn: Set(Some(lead_sn)), spawn_mode: Set(Some(plan.mode.clone())),
            tier: Set(plan.tier.map(str::to_owned)), brief: Set(Some(text.trim().to_owned())), kind: Set(Some(b.kind.clone())),
            child_seq: Set(Some(seq)), paths: Set(Some(paths)), wait_run_sn: Set(wait_run_sn), wait_glob: Set(wait_glob),
            runtime_sn: Set(step.as_ref().map(|(m, _)| m.runtime_sn)), connection_sn: Set(step.as_ref().map(|(m, _)| m.connection_sn)),
            model_code: Set(step.and_then(|(_, c)| c)),
            ..Default::default()
        }.insert(tx).await?;
        let ev = Ev::new(Some(m.project_sn), "run", m.sn, "RunStarted", &json!({
            "task_sn": m.task_sn, "member_sn": m.member_sn, "num": num, "parent_run_sn": lead_sn, "spawn_mode": plan.mode,
            "tier": plan.tier, "child_seq": seq, "wait_run_sn": wait_run_sn, "cap": plan.wait == Some(Wait::Cap),
        }));
        Ok((Spawn::Run(m), vec![ev]))
    }).await
}

/// tbl_run.paths JSON → 경로 목록
fn granted(m: &r::Model) -> Vec<String> {
    let v: serde_json::Value = m.paths.as_deref().and_then(|p| serde_json::from_str(p).ok()).unwrap_or_default();
    v.as_array().into_iter().flatten().filter_map(|x| x["path"].as_str().map(str::to_owned)).collect()
}

/// queued runner 하위 Run 하나를 실행하고 리드에게 줄 @REPORT를 돌려준다.
/// 세션 · ContextManifest · 토큰(tbl_log_token · 하위 Run 행에만)을 남기고, 끝나면 이 Run을 기다리던 형제의 대기를 푼다
pub async fn go(db: &DatabaseConnection, sn: i64, cancel: oneshot::Receiver<()>) -> Res<String> {
    let m = r::Entity::find_by_id(sn).one(db).await?.ok_or_else(Error::not_found)?;
    let (Some("runner"), Some(text), "queued", None) = (m.spawn_mode.as_deref(), m.brief.as_deref(), m.status.as_str(), m.wait_run_sn) else {
        return Err(Error::conflict(format!("run {sn} is not a ready runner")));
    };
    let b: Brief = rule::parse(text).map_err(|_| Error::conflict("stored brief is invalid".into()))?;
    let project = tbl_project::Entity::find_by_id(m.project_sn).one(db).await?.ok_or_else(Error::not_found)?;
    let ws = tbl_workspace::Entity::find_by_id(project.wid).one(db).await?.ok_or_else(Error::not_found)?;
    let rt = match m.runtime_sn { Some(x) => tbl_runtime::Entity::find_by_id(x).one(db).await?, None => None };
    let (Some(rt), Some(cwd)) = (rt, project.repo_path.clone()) else {
        return finish(db, &m, &b, None, "no_runtime", None).await;
    };
    let Some(ex) = exec::pick(&rt.code) else { return finish(db, &m, &b, None, "no_runtime", None).await };
    // ponytail: repo 모드만 (Alpha 기본). worktree 모드는 #21 worktree 생성 때
    let files: Vec<(String, Option<String>)> = granted(&m).into_iter().map(|p| {
        let body = (!p.contains('*')).then(|| std::fs::read(Path::new(&cwd).join(&p)).ok()).flatten()
            .map(|v| String::from_utf8_lossy(&v[..v.len().min(FILE_MAX)]).into_owned());
        (p, body)
    }).collect();
    let input = prompt(text, &files);

    // 세션 · 컨텍스트 목록 (보낸 항목만 · 토큰은 글자 수 / 4 추정)
    let (session, manifest) = event::run(db, async |tx| {
        let num = s::Entity::find().filter(s::Column::MemberSn.eq(m.member_sn)).order_by_desc(s::Column::Num).one(tx).await?.map_or(1, |x| x.num + 1);
        let ses = s::ActiveModel { run_sn: Set(sn), member_sn: Set(m.member_sn), num: Set(num), ..Default::default() }.insert(tx).await?;
        let man = cm::ActiveModel { run_sn: Set(sn), session_sn: Set(Some(ses.sn)), budget_token: Set(b.tok.unwrap_or(0)), ..Default::default() }.insert(tx).await?;
        let mut items = vec![("instruction", "runner-rules".to_owned(), RULES.len()), ("task", format!("@TASK {}", b.id), text.len())];
        items.extend(files.iter().map(|(p, body)| ("file", p.clone(), body.as_ref().map_or(0, String::len))));
        for (i, (kind, label, len)) in items.into_iter().enumerate() {
            cs::ActiveModel { manifest_sn: Set(man.sn), kind: Set(kind.into()), ref_label: Set(label), token_count: Set((len / 4) as i64), sort: Set(i as i64), ..Default::default() }.insert(tx).await?;
        }
        Ok(((ses.sn, man.sn), vec![]))
    }).await?;
    run::run_to(db, sn, "starting").await?;
    run::run_to(db, sn, "running").await?;
    run::session_to(db, session, "active").await?;

    let job = Job {
        prompt: input, cwd: cwd.into(), bin: rt.bin_path.map(Into::into), model: m.model_code.clone(), args: block(&rt.code),
        timeout: Duration::from_secs(ws.run_timeout_min.max(1) as u64 * 60), ..Default::default()
    };
    let (tx, _rx) = mpsc::unbounded_channel(); // ponytail: 실시간 스트림은 B-5 구독이 붙을 때
    let out = exec::run(ex, &job, &tx, cancel).await;

    // 토큰은 하위 Run에만 쌓는다 (리드 합계와 분리)
    let u = out.usage;
    event::run(db, async |tx| {
        lt::ActiveModel {
            run_sn: Set(sn), session_sn: Set(Some(session)), connection_sn: Set(m.connection_sn), manifest_sn: Set(Some(manifest)), model_code: Set(m.model_code.clone()),
            token_input: Set(u.input), token_cache_read: Set(u.cache_read), token_cache_write: Set(u.cache_write), token_output: Set(u.output), ..Default::default()
        }.insert(tx).await?;
        // 하위 Run은 한 번만 실행하므로 합계 = 이번 사용량
        r::Entity::update_many().filter(r::Column::Sn.eq(sn))
            .col_expr(r::Column::TokenInput, u.input.into()).col_expr(r::Column::TokenCacheRead, u.cache_read.into())
            .col_expr(r::Column::TokenCacheWrite, u.cache_write.into()).col_expr(r::Column::TokenOutput, u.output.into())
            .exec(tx).await?;
        s::Entity::update_many().filter(s::Column::Sn.eq(session)).col_expr(s::Column::ProviderSessionId, out.session.clone().into()).exec(tx).await?;
        Ok(((), vec![]))
    }).await?;
    run::session_to(db, session, if out.status == Status::Done { "stopped" } else { "failed" }).await?;

    let fail = match out.status {
        Status::Done => None,
        Status::Timeout => Some("timeout"),
        Status::Cancelled => Some("cancelled"),
        Status::Missing => Some("runtime_missing"),
        Status::Login => Some("login_required"),
        Status::Failed => Some("exec_failed"),
    };
    let rep = out.text.as_deref().and_then(report);
    match (fail, rep) {
        (None, Some(rep)) if matches!(rep.status.as_str(), "done" | "partial") => finish(db, &m, &b, Some(rep), "", out.err).await,
        (None, Some(rep)) => { let code = format!("report_{}", rep.status); finish(db, &m, &b, Some(rep), &code, out.err).await }
        (None, None) => finish(db, &m, &b, None, "no_report", out.text).await,
        (Some(code), _) => finish(db, &m, &b, None, code, out.err).await,
    }
}

/// 끝 상태 기록 → 기다리던 형제 대기 해제 → 리드에게 줄 @REPORT. code가 비면 completed, 아니면 failed(code)
async fn finish(db: &DatabaseConnection, m: &r::Model, b: &Brief, rep: Option<Report>, code: &str, detail: Option<String>) -> Res<String> {
    let lead = rep.as_ref().map_or_else(|| failed(&b.id, &b.parent, code), |r| r.lead.clone());
    let summary = rep.as_ref().map(|r| r.status.clone());
    let fail = (!code.is_empty()).then(|| code.to_owned());
    event::run(db, async |tx| {
        r::Entity::update_many().filter(r::Column::Sn.eq(m.sn))
            .col_expr(r::Column::ResultSummary, summary.into()).col_expr(r::Column::FailCode, fail.clone().into()).col_expr(r::Column::FailDetail, detail.into())
            .exec(tx).await?;
        r::Entity::update_many().filter(r::Column::WaitRunSn.eq(m.sn))
            .col_expr(r::Column::WaitRunSn, Expr::value(Option::<i64>::None)).col_expr(r::Column::WaitGlob, Expr::value(Option::<String>::None)).exec(tx).await?;
        Ok(((), vec![]))
    }).await?;
    let now = r::Entity::find_by_id(m.sn).one(db).await?.ok_or_else(Error::not_found)?.status;
    // queued(실행 전 실패)는 starting을 거쳐 failed로
    if now == "queued" { run::run_to(db, m.sn, "starting").await?; }
    run::run_to(db, m.sn, if fail.is_some() { "failed" } else { "completed" }).await?;
    Ok(lead)
}

#[cfg(test)]
mod tests {
    use super::*;

    const BRIEF: &str = "@TASK v1\nid: T1.1  parent: R1  mode: runner  kind: fix\ngoal: g\nac: [1 a]\npaths: [a.txt]";

    /// 최소 입력: 고정 규칙 → @TASK → 파일 순서, 저장소 지침이 끼지 않는다
    #[test]
    fn input() {
        let p = prompt(BRIEF, &[("a.txt".into(), Some("hello".into())), ("src/*".into(), None)]);
        assert!(p.starts_with(RULES));
        assert!(p.find("@TASK v1").unwrap() < p.find("--- a.txt\nhello").unwrap());
        assert!(p.contains("--- src/* (not read"));
        assert!(RULES.len() / 4 < 300);
        assert!(block("claude_code").contains(&"--strict-mcp-config".to_owned()));
        assert!(block("codex").contains(&"project_doc_max_bytes=0".to_owned()));
    }

    /// 마지막 @REPORT의 기계 칸만, 사람 칸 · 앞뒤 글은 버린다
    #[test]
    fn reports() {
        let t = "working...\n@REPORT v1\nid: T1.1  run: R1  status: partial\nac 1 ok | ac 2 skip \"x\"\n--- human\nresult: 사람용\n";
        assert_eq!(report(t), Some(Report { status: "partial".into(), lead: "@REPORT v1\nid: T1.1  run: R1  status: partial\nac 1 ok | ac 2 skip \"x\"".into() }));
        assert_eq!(report("```\n@REPORT v1\nid: a  run: R1  status: done\n```").unwrap().status, "done");
        assert_eq!(report("@REPORT v1\nid: a  run: R1  status: great"), None);
        assert_eq!(report("no block"), None);
    }

    /// 리드 R1 → @TASK → 하위 Run(runner · 등급 M · 폴백 단계) → 가짜 Claude 실행 → 리드는 @REPORT만, 토큰은 하위 Run에만
    #[tokio::test]
    async fn flow() {
        use std::os::unix::fs::PermissionsExt;
        let dir = std::env::temp_dir().join(format!("orch-runner-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.txt"), "hello").unwrap();
        // 가짜 CLI: 받은 프롬프트를 파일로 남기고 @REPORT로 끝낸다
        let bin = dir.join("claude");
        std::fs::write(&bin, format!(r#"#!/bin/sh
cat > {0}/prompt.txt
echo "$@" > {0}/args.txt
echo '{{"type":"system","subtype":"init","session_id":"s9"}}'
printf '%s\n' '{{"type":"result","is_error":false,"result":"done.\n@REPORT v1\nid: T1.1  run: R1  status: done\nac 1 ok\n--- human\nresult: x","usage":{{"input_tokens":7,"output_tokens":3,"cache_read_input_tokens":0,"cache_creation_input_tokens":0}}}}'"#, dir.display())).unwrap();
        std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755)).unwrap();

        let mut opt = sea_orm::ConnectOptions::new("sqlite::memory:");
        opt.max_connections(1);
        let db = crate::connect(opt).await.unwrap();
        db.execute_unprepared(&format!("
            INSERT INTO tbl_team (sn, wid, name) VALUES (1, 1, 't');
            INSERT INTO tbl_agent_profile (sn, wid, kind) VALUES (1, 1, 'member');
            INSERT INTO tbl_member (sn, team_sn, profile_sn, name, role_name) VALUES (1, 1, 1, 'm', 'dev');
            INSERT INTO tbl_project (sn, wid, team_sn, name, repo_path) VALUES (1, 1, 1, 'p', '{0}');
            INSERT INTO tbl_task (sn, project_sn, num, title, member_sn, status) VALUES (1, 1, 1, 't', 1, 'in_progress');
            INSERT INTO tbl_runtime (sn, wid, code, name, bin_path) VALUES (1, 1, 'claude_code', 'Claude Code', '{1}');
            INSERT INTO tbl_connection (sn, wid, kind, provider_code, provider_name, name) VALUES (1, 1, 'subscription', 'anthropic', 'Anthropic', 'c');
            INSERT INTO tbl_map_fallback (profile_sn, runtime_sn, connection_sn, sort, tier) VALUES (1, 1, 1, 1, 'L'), (1, 1, 1, 2, 'M');
            INSERT INTO tbl_run (sn, project_sn, task_sn, member_sn, num, status) VALUES (1, 1, 1, 1, 1, 'running');",
            dir.display(), bin.display())).await.unwrap();

        // goal 없음 → 만들지 않는다
        assert!(matches!(spawn(&db, 1, &BRIEF.replace("goal: g", "goal:")).await.unwrap(), Spawn::Stop(Stop::Lead(_))));
        let Spawn::Run(kid) = spawn(&db, 1, BRIEF).await.unwrap() else { panic!("not spawned") };
        assert_eq!((kid.start_by.as_str(), kid.parent_run_sn, kid.tier.as_deref(), kid.child_seq, kid.runtime_sn), ("lead", Some(1), Some("M"), Some(1), Some(1)));
        // 같은 paths → 순차 대기
        let Spawn::Run(next) = spawn(&db, 1, &BRIEF.replace("T1.1", "T1.2")).await.unwrap() else { panic!() };
        assert_eq!((next.wait_run_sn, next.child_seq), (Some(kid.sn), Some(2)));

        let (_keep, cancel) = oneshot::channel();
        let lead = go(&db, kid.sn, cancel).await.unwrap();
        assert_eq!(lead, "@REPORT v1\nid: T1.1  run: R1  status: done\nac 1 ok");
        let sent = std::fs::read_to_string(dir.join("prompt.txt")).unwrap();
        assert!(sent.starts_with(RULES) && sent.contains("--- a.txt\nhello"));
        assert!(std::fs::read_to_string(dir.join("args.txt")).unwrap().contains("--strict-mcp-config"));

        let done = r::Entity::find_by_id(kid.sn).one(&db).await.unwrap().unwrap();
        assert_eq!((done.status.as_str(), done.token_input, done.result_summary.as_deref()), ("completed", Some(7), Some("done")));
        let toks = lt::Entity::find().all(&db).await.unwrap();
        assert_eq!(toks.iter().map(|t| t.run_sn).collect::<Vec<_>>(), [kid.sn]);
        let kinds: Vec<String> = cs::Entity::find().all(&db).await.unwrap().into_iter().map(|c| c.kind).collect();
        assert_eq!(kinds, ["instruction", "task", "file"]);
        // 끝나면 기다리던 형제가 풀린다
        assert_eq!(r::Entity::find_by_id(next.sn).one(&db).await.unwrap().unwrap().wait_run_sn, None);
    }
}
