//! 실행기 (#13): Claude Code · Codex CLI를 같은 계약으로 비대화형 실행한다.
//! CLI별 인자 · 출력 해석만 `Executor` 구현에 두고, 프로세스 · 환경 · 스트림 · 제한 시간 · 취소 · 결과 정리는 `run` 하나가 맡는다
#![allow(dead_code)] // Run 디스패처(#67 runner)가 호출한다

use serde_json::Value;
use std::{path::PathBuf, process::Stdio, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    process::{Child, Command},
    sync::{mpsc, oneshot},
};

#[cfg(test)]
thread_local!(
    /// 테스트용: 이 스레드에서 `run`이 불린 횟수 (Orch 규칙 엔진이 모델을 부르지 않는다는 단언용)
    pub static CALLS: std::cell::Cell<u32> = const { std::cell::Cell::new(0) }
);

/// 자식 프로세스에 넘기는 환경 변수 (나머지는 지운다). CLI 로그인 상태는 HOME 아래 설정 · 키체인에서 읽는다
const ENV_KEEP: [&str; 9] = ["PATH", "HOME", "USER", "LOGNAME", "SHELL", "LANG", "LC_ALL", "TERM", "TMPDIR"];
/// stderr는 끝부분만 남긴다
const ERR_TAIL: usize = 4096;
/// 취소 · 제한 시간 때 TERM 후 KILL까지 기다리는 시간
const GRACE: Duration = Duration::from_secs(3);

/// CLI 하나의 차이 (인자 · 로그인 확인 · 출력 한 줄 해석)
pub trait Executor: Send + Sync {
    /// tbl_runtime.code
    fn code(&self) -> &'static str;
    /// PATH에서 찾을 실행 파일 이름
    fn bin(&self) -> &'static str;
    /// 비대화형 실행 인자. 프롬프트는 stdin으로 보낸다
    fn args(&self, job: &Job) -> Vec<String>;
    /// 로그인 확인 인자
    fn login_args(&self) -> &'static [&'static str];
    /// 로그인 확인 결과 해석 (성공 여부 · stdout)
    fn logged_in(&self, ok: bool, out: &str) -> bool;
    /// 출력 JSON 한 줄 → 공통 이벤트 (모르는 줄은 버린다)
    fn parse(&self, v: &Value) -> Vec<Event>;
}

/// 실행 요청
#[derive(Default)]
pub struct Job {
    pub prompt: String,
    pub cwd: PathBuf,
    /// tbl_runtime.bin_path (없으면 PATH)
    pub bin: Option<PathBuf>,
    /// 연결 모델 이름 (없으면 CLI 기본)
    pub model: Option<String>,
    /// 실행기 뒤에 붙일 인자 (지침 차단 등 · runner가 채운다)
    pub args: Vec<String>,
    /// 추가 환경 변수 (API 키 · CODEX_HOME 등). 값은 결과 문구에서 가린다
    pub env: Vec<(String, String)>,
    pub timeout: Duration,
}

/// CLI 출력에서 뽑은 공통 이벤트 (UI · DB는 이것만 본다)
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// 실행기 세션 id (tbl_session.provider_session_id)
    Session(String),
    /// 모델 응답 텍스트
    Text(String),
    Usage(Usage),
    /// 실행기가 알린 끝 (성공 여부 · 최종 응답)
    Done { ok: bool, text: Option<String> },
    Error(String),
}

/// 토큰 사용량. input은 캐시를 뺀 입력
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Usage {
    pub input: i64,
    pub output: i64,
    pub cache_read: i64,
    pub cache_write: i64,
}

/// 결과 상태
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Status {
    Done,
    Failed,
    Timeout,
    Cancelled,
    /// 실행 파일 없음
    Missing,
    /// 로그인 필요 · 인증 만료
    Login,
}

/// 정리된 실행 결과
#[derive(Debug)]
pub struct Outcome {
    pub status: Status,
    pub exit: Option<i32>,
    pub session: Option<String>,
    /// 최종 응답 (없으면 마지막 Text)
    pub text: Option<String>,
    pub usage: Usage,
    /// stderr 끝부분 · 실행기 오류 (Job.env 값은 가림)
    pub err: Option<String>,
}

/// 설치 · 버전 · 로그인 확인 결과
#[derive(Debug, PartialEq)]
pub struct Probe {
    pub installed: bool,
    pub version: Option<String>,
    pub logged_in: bool,
}

/// Claude Code (`claude -p --output-format stream-json`)
pub struct Claude;
/// Codex CLI (`codex exec --json`)
pub struct Codex;

/// tbl_runtime.code → 실행기
pub fn pick(code: &str) -> Option<&'static dyn Executor> {
    match code {
        "claude_code" => Some(&Claude),
        "codex" => Some(&Codex),
        _ => None,
    }
}

/// 숫자 필드 (없으면 0)
fn int(v: &Value, k: &str) -> i64 {
    v[k].as_i64().unwrap_or(0)
}

/// 문자열 필드
fn text(v: &Value, k: &str) -> Option<String> {
    v[k].as_str().map(str::to_owned)
}

impl Executor for Claude {
    fn code(&self) -> &'static str { "claude_code" }
    fn bin(&self) -> &'static str { "claude" }
    fn args(&self, job: &Job) -> Vec<String> {
        // stream-json은 -p에서 --verbose가 있어야 한다
        let mut a: Vec<String> = ["-p", "--output-format", "stream-json", "--verbose"].map(String::from).into();
        if let Some(m) = &job.model { a.extend(["--model".into(), m.clone()]); }
        a.extend(job.args.iter().cloned());
        a
    }
    fn login_args(&self) -> &'static [&'static str] { &["auth", "status", "--json"] }
    fn logged_in(&self, ok: bool, out: &str) -> bool {
        // 출력에 계정 정보가 있으므로 loggedIn만 읽고 버린다
        ok && serde_json::from_str::<Value>(out).is_ok_and(|v| v["loggedIn"] == true)
    }
    fn parse(&self, v: &Value) -> Vec<Event> {
        match (v["type"].as_str(), v["subtype"].as_str()) {
            (Some("system"), Some("init")) => text(v, "session_id").map(Event::Session).into_iter().collect(),
            (Some("assistant"), _) => v["message"]["content"].as_array().into_iter().flatten()
                .filter(|c| c["type"] == "text").filter_map(|c| text(c, "text")).map(Event::Text).collect(),
            // result에 Run 전체 사용량이 있다 (assistant 줄의 usage는 메시지별이라 쓰지 않는다)
            (Some("result"), _) => {
                let u = &v["usage"];
                vec![
                    Event::Usage(Usage { input: int(u, "input_tokens"), output: int(u, "output_tokens"), cache_read: int(u, "cache_read_input_tokens"), cache_write: int(u, "cache_creation_input_tokens") }),
                    Event::Done { ok: v["is_error"] == false, text: text(v, "result") },
                ]
            }
            _ => vec![],
        }
    }
}

impl Executor for Codex {
    fn code(&self) -> &'static str { "codex" }
    fn bin(&self) -> &'static str { "codex" }
    fn args(&self, job: &Job) -> Vec<String> {
        let mut a: Vec<String> = ["exec", "--json", "--skip-git-repo-check"].map(String::from).into();
        if let Some(m) = &job.model { a.extend(["-m".into(), m.clone()]); }
        a.extend(job.args.iter().cloned());
        a.push("-".into()); // 프롬프트는 stdin
        a
    }
    fn login_args(&self) -> &'static [&'static str] { &["login", "status"] }
    fn logged_in(&self, ok: bool, _: &str) -> bool { ok }
    fn parse(&self, v: &Value) -> Vec<Event> {
        match v["type"].as_str() {
            Some("thread.started") => text(v, "thread_id").map(Event::Session).into_iter().collect(),
            // item.type error는 설정 경고라 버린다
            Some("item.completed") if v["item"]["type"] == "agent_message" => text(&v["item"], "text").map(Event::Text).into_iter().collect(),
            Some("turn.completed") => {
                let u = &v["usage"];
                // Codex input_tokens는 캐시를 포함한다 → 빼서 Claude와 같은 뜻으로
                let read = int(u, "cached_input_tokens");
                vec![
                    Event::Usage(Usage { input: int(u, "input_tokens") - read, output: int(u, "output_tokens"), cache_read: read, cache_write: int(u, "cache_write_input_tokens") }),
                    Event::Done { ok: true, text: None },
                ]
            }
            Some("turn.failed") => vec![Event::Error(text(&v["error"], "message").unwrap_or_default()), Event::Done { ok: false, text: None }],
            Some("error") => text(v, "message").map(Event::Error).into_iter().collect(),
            _ => vec![],
        }
    }
}

/// 환경을 지운 명령. ENV_KEEP + 추가 변수만, 새 프로세스 그룹으로 (취소 때 자식까지 한 번에 끈다)
fn command(bin: &PathBuf, args: &[impl AsRef<std::ffi::OsStr>], cwd: &PathBuf, env: &[(String, String)]) -> Command {
    let mut c = Command::new(bin);
    c.args(args).current_dir(cwd).env_clear().kill_on_drop(true);
    for k in ENV_KEEP {
        if let Ok(v) = std::env::var(k) { c.env(k, v); }
    }
    c.envs(env.iter().map(|(k, v)| (k, v)));
    #[cfg(unix)]
    c.process_group(0);
    c
}

/// 설치 · 버전 · 로그인 확인. 실행 파일이 없으면 installed = false
pub async fn probe(ex: &dyn Executor, bin: Option<PathBuf>) -> Probe {
    let bin = bin.unwrap_or_else(|| ex.bin().into());
    let cwd = std::env::temp_dir();
    let out = |args: &[&str]| command(&bin, args, &cwd, &[]).stdin(Stdio::null()).output();
    let Ok(v) = out(&["--version"]).await else { return Probe { installed: false, version: None, logged_in: false } };
    let version = String::from_utf8_lossy(&v.stdout).split_whitespace().find(|w| w.starts_with(|c: char| c.is_ascii_digit())).map(str::to_owned);
    let logged_in = out(ex.login_args()).await.is_ok_and(|o| ex.logged_in(o.status.success(), &String::from_utf8_lossy(&o.stdout)));
    Probe { installed: true, version, logged_in }
}

/// 프로세스 그룹 전체에 TERM → GRACE 뒤에도 살아 있으면 KILL
async fn kill_tree(child: &mut Child) {
    #[cfg(unix)]
    if let Some(pid) = child.id() {
        // ponytail: libc 없이 kill 명령으로 그룹 신호. Windows는 kill_on_drop(자식만)
        let sig = |s: &str| std::process::Command::new("kill").args([s, "--", &format!("-{pid}")]).status();
        let _ = sig("-TERM");
        if tokio::time::timeout(GRACE, child.wait()).await.is_ok() { return; }
        let _ = sig("-KILL");
    }
    let _ = child.kill().await;
}

/// Job.env 값을 가린다 (키 원문이 결과 · 로그로 새지 않게)
fn mask(s: String, env: &[(String, String)]) -> String {
    env.iter().filter(|(_, v)| v.len() >= 8).fold(s, |s, (_, v)| s.replace(v.as_str(), "***"))
}

/// 인증 실패로 보이는 문구
// ponytail: 문구 검사. 실행기가 오류 코드를 주면 그걸로
fn auth_fail(s: &str) -> bool {
    let s = s.to_ascii_lowercase();
    ["not logged in", "please run /login", "invalid api key", "unauthorized", "401", "login required", "authentication"].iter().any(|k| s.contains(k))
}

/// 실행: stdin으로 프롬프트 → stdout JSON 줄마다 이벤트를 `tx`로 보낸다 → 끝 · 제한 시간 · 취소 중 먼저 오는 것으로 정리.
/// `cancel`은 값이 오면 취소, 보내는 쪽이 사라지면 무시
pub async fn run(ex: &dyn Executor, job: &Job, tx: &mpsc::UnboundedSender<Event>, cancel: oneshot::Receiver<()>) -> Outcome {
    #[cfg(test)]
    CALLS.with(|c| c.set(c.get() + 1));
    let bin = job.bin.clone().unwrap_or_else(|| ex.bin().into());
    let mut out = Outcome { status: Status::Failed, exit: None, session: None, text: None, usage: Usage::default(), err: None };
    let mut child = match command(&bin, &ex.args(job), &job.cwd, &job.env).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn() {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Outcome { status: Status::Missing, err: Some(format!("{} not found", bin.display())), ..out },
        Err(e) => return Outcome { err: Some(e.to_string()), ..out },
    };
    // stdin은 따로 써서 닫는다 (CLI가 출력을 먼저 내도 막히지 않게)
    let mut stdin = child.stdin.take().expect("piped");
    let prompt = job.prompt.clone();
    tokio::spawn(async move { let _ = stdin.write_all(prompt.as_bytes()).await; });
    let mut stderr = child.stderr.take().expect("piped");
    let err_task = tokio::spawn(async move {
        let mut buf = Vec::new();
        let _ = stderr.read_to_end(&mut buf).await;
        String::from_utf8_lossy(&buf[buf.len().saturating_sub(ERR_TAIL)..]).into_owned()
    });
    let mut lines = BufReader::new(child.stdout.take().expect("piped")).lines();
    let cancel = async { if cancel.await.is_err() { std::future::pending::<()>().await } };
    let timer = tokio::time::sleep(job.timeout);
    tokio::pin!(cancel, timer);
    let (mut done, mut last, mut errs) = (None, None, Vec::new());
    let stopped = loop {
        tokio::select! {
            l = lines.next_line() => match l {
                Ok(Some(l)) => {
                    let Ok(v) = serde_json::from_str::<Value>(&l) else { continue }; // JSON이 아닌 줄은 버린다
                    for ev in ex.parse(&v) {
                        match &ev {
                            Event::Session(s) => out.session = Some(s.clone()),
                            Event::Text(t) => last = Some(t.clone()),
                            Event::Usage(u) => { out.usage.input += u.input; out.usage.output += u.output; out.usage.cache_read += u.cache_read; out.usage.cache_write += u.cache_write; }
                            Event::Done { ok, text } => { done = Some(*ok); if text.is_some() { out.text = text.clone(); } }
                            Event::Error(e) => errs.push(e.clone()),
                        }
                        let _ = tx.send(ev); // 받는 쪽이 없으면 무시
                    }
                }
                _ => break None,
            },
            _ = &mut timer => break Some(Status::Timeout),
            _ = &mut cancel => break Some(Status::Cancelled),
        }
    };
    if stopped.is_some() { kill_tree(&mut child).await; }
    out.exit = child.wait().await.ok().and_then(|s| s.code());
    let stderr = err_task.await.unwrap_or_default();
    errs.extend((!stderr.trim().is_empty()).then(|| stderr.trim().to_owned()));
    out.text = out.text.or(last).map(|t| mask(t, &job.env));
    out.err = (!errs.is_empty()).then(|| mask(errs.join("\n"), &job.env));
    out.status = match stopped {
        Some(s) => s,
        None if done == Some(true) && out.exit == Some(0) => Status::Done,
        None if [out.err.as_deref(), out.text.as_deref()].into_iter().flatten().any(auth_fail) => Status::Login,
        None => Status::Failed,
    };
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    /// 가짜 CLI: sh 스크립트를 임시 폴더에 쓰고 경로를 돌려준다
    fn fake(name: &str, body: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("orch-exec-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join(name);
        std::fs::write(&p, format!("#!/bin/sh\n{body}\n")).unwrap();
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
        p
    }

    fn job(bin: PathBuf, secs: u64) -> Job {
        Job { prompt: "hi".into(), cwd: std::env::temp_dir(), bin: Some(bin), timeout: Duration::from_secs(secs), ..Default::default() }
    }

    /// 끝까지 실행하고 (결과, 받은 이벤트)
    async fn go(ex: &dyn Executor, j: &Job) -> (Outcome, Vec<Event>) {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let (_keep, cancel) = oneshot::channel();
        let out = run(ex, j, &tx, cancel).await;
        drop(tx);
        let mut evs = vec![];
        while let Some(e) = rx.recv().await { evs.push(e); }
        (out, evs)
    }

    /// Claude stream-json: 세션 · 텍스트 · 사용량 · 결과. stdin 프롬프트를 받는다
    #[tokio::test]
    async fn claude() {
        let bin = fake("claude-ok", r#"read p
echo '{"type":"system","subtype":"init","session_id":"s1"}'
echo "not json"
echo '{"type":"assistant","message":{"content":[{"type":"thinking","thinking":"x"},{"type":"text","text":"'$p'"}]}}'
echo '{"type":"result","is_error":false,"result":"done","usage":{"input_tokens":9,"output_tokens":6,"cache_read_input_tokens":100,"cache_creation_input_tokens":50}}'"#);
        let (o, evs) = go(&Claude, &job(bin, 10)).await;
        assert_eq!((o.status, o.exit, o.session.as_deref(), o.text.as_deref()), (Status::Done, Some(0), Some("s1"), Some("done")));
        assert_eq!(o.usage, Usage { input: 9, output: 6, cache_read: 100, cache_write: 50 });
        assert!(evs.contains(&Event::Text("hi".into())));
    }

    /// Codex JSONL: 캐시를 뺀 input · 마지막 agent_message가 응답
    #[tokio::test]
    async fn codex() {
        let bin = fake("codex-ok", r#"cat >/dev/null
echo '{"type":"thread.started","thread_id":"t1"}'
echo '{"type":"item.completed","item":{"type":"error","message":"config warn"}}'
echo '{"type":"item.completed","item":{"type":"agent_message","text":"ok"}}'
echo '{"type":"turn.completed","usage":{"input_tokens":100,"cached_input_tokens":40,"output_tokens":5}}'"#);
        let (o, _) = go(&Codex, &job(bin, 10)).await;
        assert_eq!((o.status, o.session.as_deref(), o.text.as_deref(), o.err), (Status::Done, Some("t1"), Some("ok"), None));
        assert_eq!(o.usage, Usage { input: 60, output: 5, cache_read: 40, cache_write: 0 });
    }

    /// 실패 · 로그인 필요 · 실행 파일 없음
    #[tokio::test]
    async fn fail() {
        let bin = fake("codex-fail", r#"echo '{"type":"turn.failed","error":{"message":"boom"}}'; exit 1"#);
        let (o, _) = go(&Codex, &job(bin, 10)).await;
        assert_eq!((o.status, o.exit, o.err.as_deref()), (Status::Failed, Some(1), Some("boom")));
        let bin = fake("claude-auth", r#"echo '{"type":"result","is_error":true,"result":"Not logged in · Please run /login"}'; exit 1"#);
        assert_eq!(go(&Claude, &job(bin, 10)).await.0.status, Status::Login);
        assert_eq!(go(&Claude, &job("/nonexistent/claude".into(), 10)).await.0.status, Status::Missing);
    }

    /// 제한 시간 · 취소 → 손자 프로세스까지 끝난다
    #[tokio::test]
    async fn stop() {
        let pid = std::env::temp_dir().join(format!("orch-exec-{}-grand", std::process::id()));
        let bin = fake("hang", &format!("sleep 30 & echo $! > {}\nwait", pid.display()));
        let t = std::time::Instant::now();
        assert_eq!(go(&Claude, &job(bin.clone(), 1)).await.0.status, Status::Timeout);
        assert!(t.elapsed() < Duration::from_secs(5));
        let grand = std::fs::read_to_string(&pid).unwrap();
        let alive = std::process::Command::new("kill").args(["-0", grand.trim()]).status().unwrap().success();
        assert!(!alive, "grandchild survived");

        let (tx, _rx) = mpsc::unbounded_channel();
        let (stop, cancel) = oneshot::channel();
        let j = job(bin, 30);
        let h = tokio::spawn(async move { run(&Claude, &j, &tx, cancel).await.status });
        tokio::time::sleep(Duration::from_millis(300)).await;
        stop.send(()).unwrap();
        assert_eq!(h.await.unwrap(), Status::Cancelled);
    }

    /// 환경은 허용 목록 + Job.env만, Job.env 값은 결과에서 가린다
    #[tokio::test]
    async fn env() {
        // SAFETY: 이 테스트만 쓰는 변수
        unsafe { std::env::set_var("ORCH_LEAK", "secret-leak") };
        let bin = fake("env", r#"cat >/dev/null
echo "{\"type\":\"result\",\"is_error\":false,\"result\":\"leak=$ORCH_LEAK key=$MY_KEY home=${HOME:+y}\"}""#);
        let mut j = job(bin, 10);
        j.env = vec![("MY_KEY".into(), "sk-1234567890".into())];
        let (o, _) = go(&Claude, &j).await;
        assert_eq!(o.text.as_deref(), Some("leak= key=*** home=y"));
    }

    /// 설치 · 로그인 확인 (가짜 CLI)
    #[tokio::test]
    async fn probes() {
        let bin = fake("claude-probe", r#"[ "$1" = "--version" ] && echo "2.1.286 (Claude Code)" && exit 0
echo '{"loggedIn": true, "email": "x@y"}'"#);
        assert_eq!(probe(&Claude, Some(bin)).await, Probe { installed: true, version: Some("2.1.286".into()), logged_in: true });
        let bin = fake("codex-probe", r#"[ "$1" = "--version" ] && echo "codex-cli 0.157.1" && exit 0
echo "Not logged in"; exit 1"#);
        assert_eq!(probe(&Codex, Some(bin)).await, Probe { installed: true, version: Some("0.157.1".into()), logged_in: false });
        assert!(!probe(&Codex, Some("/nonexistent/codex".into())).await.installed);
        assert!(pick("claude_code").is_some() && pick("codex").is_some() && pick("gemini").is_none());
    }

    /// 실제 CLI 확인 (토큰 쓰지 않음 · 설치된 기기에서만): cargo test real -- --ignored
    #[tokio::test]
    #[ignore]
    async fn real() {
        for ex in [&Claude as &dyn Executor, &Codex] {
            println!("{} {:?}", ex.code(), probe(ex, None).await);
        }
    }
}
