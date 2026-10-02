//! 규칙 엔진 (#67 · LLM 0): 리드가 보낸 하위 작업 @TASK 검사 · 모델 등급 · paths 겹침 · @ASK 경로 추가 판정.
//! DB를 읽지 않는다. 호출하는 쪽(runner 실행)이 현재 상태를 `Ctx`로 모아 넘긴다
#![allow(dead_code)] // runner 실행(#67 다음 Task)이 호출한다

use crate::{agent::{Fallback, TIERS}, team::SPAWN};

/// 작업 종류 → 기본 등급 (TIERS 위치: 0 = S, 1 = M, 2 = L)
const KINDS: [(&str, usize); 9] = [
    ("explore", 0), ("search", 0), ("format", 0), ("test", 0),
    ("implement", 1), ("fix", 1),
    ("design", 2), ("review", 2), ("debug", 2),
];
/// paths · ac 수가 이보다 많으면 등급 +1
const MANY: usize = 5;
/// 팀 일일 예산 잔량이 이 % 미만이면 L 금지
const LOW_PCT: i64 = 20;
/// 하위 Run 하나가 @ASK로 paths를 늘릴 수 있는 횟수
const ASK_MAX: u8 = 2;

/// 리드가 보낸 하위 작업 @TASK 중 규칙에 쓰는 키
#[derive(Debug)]
pub struct Brief {
    pub id: String,
    /// 리드 Run 표시 번호 (R81)
    pub parent: String,
    /// 비면 태스크 · 팀 기본 방식
    pub mode: Option<String>,
    pub kind: String,
    pub goal: String,
    pub ac: Vec<String>,
    pub paths: Vec<String>,
    /// limits.tok (예상 토큰). 없으면 예산 사전 검사 안 함
    pub tok: Option<i64>,
}

/// 판정이 멈춘 이유
#[derive(Debug, PartialEq)]
pub enum Stop {
    /// 리드에게 1줄로 돌려준다 (반려 · @ASK 넘김)
    Lead(String),
    /// 사람 판단 요청 (재시도 반복 · 예산)
    Judge(String),
}

/// 검사에 쓰는 현재 상태. 호출하는 쪽이 DB에서 모은다
pub struct Ctx<'a> {
    /// 리드 Run 표시 번호
    pub lead_num: i64,
    /// tbl_team.spawn_mode · spawn_allow
    pub team_mode: &'a str,
    pub allow: &'a str,
    /// tbl_task.spawn_mode (NULL = 팀 기본)
    pub task_mode: Option<&'a str>,
    /// 같은 id의 앞선 실패 횟수
    pub retry: u8,
    /// tbl_team.daily_token_budget · 오늘 쓴 토큰
    pub budget: Option<i64>,
    pub used: i64,
    /// 이 리드의 진행 중 하위 Run 수 · tbl_team.max_child_run
    pub children: i64,
    pub max_child: i64,
    /// 팀 진행 중 Run 수 · tbl_team.max_concurrent_run
    pub team_run: i64,
    pub team_max: i64,
    /// 같은 리드의 진행 중 하위 Run (run sn, paths)
    pub siblings: &'a [(i64, Vec<String>)],
}

/// 통과한 하위 작업의 실행 계획
#[derive(Debug, PartialEq)]
pub struct Plan {
    pub mode: String,
    /// runner만 (sub · fork는 리드 실행기 모델)
    pub tier: Option<&'static str>,
    /// 지금 시작 못 하면 기다릴 이유. None = 바로 시작
    pub wait: Option<Wait>,
}

/// 대기 이유
#[derive(Debug, PartialEq)]
pub enum Wait {
    /// paths가 겹친 하위 Run과 그 경로 → 끝날 때까지 순차
    Paths(i64, String),
    /// 리드당 · 팀 동시 실행 상한
    Cap,
}

/// `@TASK v1` 블록 파싱. 형식 오류 · goal · ac · paths 중 빈 것이 있으면 Lead
pub fn parse(text: &str) -> Result<Brief, Stop> {
    let bad = |why: &str| Stop::Lead(format!("bad @TASK: {why}"));
    let mut lines = text.lines().map(str::trim).filter(|l| !l.is_empty());
    if lines.next() != Some("@TASK v1") {
        return Err(bad("header"));
    }
    // 한 줄에 여러 키가 올 수 있다 (`id: T129.1  parent: R81`) → 공백 2칸으로 나눈다. `#` 이후는 주석
    let kv: Vec<(&str, &str)> = lines.flat_map(|l| l.split("  ")).map(str::trim)
        .take_while(|p| !p.starts_with('#')).filter_map(|p| p.split_once(':')).map(|(k, v)| (k.trim(), v.trim())).collect();
    let get = |k: &str| kv.iter().find(|(key, _)| *key == k).map(|(_, v)| *v).filter(|v| !v.is_empty());
    let need = |k: &str| get(k).map(str::to_owned).ok_or_else(|| bad(&format!("{k} missing")));
    let list = |k: &str| -> Result<Vec<String>, Stop> {
        let v = get(k).and_then(|v| v.strip_prefix('[')?.strip_suffix(']')).ok_or_else(|| bad(&format!("{k} must be [..]")))?;
        let out: Vec<String> = v.split(',').map(str::trim).filter(|s| !s.is_empty()).map(str::to_owned).collect();
        if out.is_empty() { Err(bad(&format!("{k} empty"))) } else { Ok(out) }
    };
    let tok = get("limits").and_then(|v| v.trim_matches(['{', '}']).split(',').find_map(|p| p.trim().strip_prefix("tok:")))
        .and_then(|t| num(t.trim()));
    Ok(Brief {
        id: need("id")?, parent: need("parent")?, mode: get("mode").map(str::to_owned), kind: need("kind")?,
        goal: need("goal")?, ac: list("ac")?, paths: list("paths")?, tok,
    })
}

/// `30k` · `1.2m` · `500` → 토큰 수
fn num(t: &str) -> Option<i64> {
    let (n, mul) = match t.to_ascii_lowercase() {
        s if s.ends_with('k') => (s[..s.len() - 1].to_owned(), 1e3),
        s if s.ends_with('m') => (s[..s.len() - 1].to_owned(), 1e6),
        s => (s, 1.0),
    };
    n.parse::<f64>().ok().map(|n| (n * mul) as i64)
}

/// 하위 작업 생성 판정: parent · 방식 허용 · kind · 예산 · 등급 · 동시 상한 · paths 겹침
pub fn check(b: &Brief, c: &Ctx) -> Result<Plan, Stop> {
    if b.parent != format!("R{}", c.lead_num) {
        return Err(Stop::Lead(format!("parent {} is not R{}", b.parent, c.lead_num)));
    }
    // 리드가 방식을 비우면 태스크 → 팀 기본. 적었으면 팀 허용 방식 안이어야 한다
    let mode = b.mode.as_deref().or(c.task_mode).unwrap_or(c.team_mode);
    if !SPAWN.contains(&mode) || !c.allow.split(',').any(|m| m == mode) {
        return Err(Stop::Lead(format!("mode {mode} not allowed ({})", c.allow)));
    }
    let left = c.budget.map(|b| b - c.used);
    if let (Some(left), Some(tok)) = (left, b.tok) && tok > left {
        return Err(Stop::Judge(format!("team budget left {left} < limits.tok {tok}")));
    }
    let low = c.budget.zip(left).is_some_and(|(b, l)| l * 100 < b * LOW_PCT);
    let t = tier(&b.kind, b.paths.len(), b.ac.len(), c.retry, low)?; // runner가 아니어도 kind 검사는 한다
    let wait = if c.children >= c.max_child || c.team_run >= c.team_max { Some(Wait::Cap) } else { waits(&b.paths, c.siblings).map(|(sn, p)| Wait::Paths(sn, p)) };
    Ok(Plan { mode: mode.into(), tier: (mode == "runner").then_some(t), wait })
}

/// 모델 등급: kind 기본 → paths · ac가 많으면 +1 → 재시도 1회 +1 (최대 L). 2번째 재시도 · 예산 부족 중 L은 Judge
pub fn tier(kind: &str, paths: usize, ac: usize, retry: u8, low: bool) -> Result<&'static str, Stop> {
    let base = KINDS.iter().find(|(k, _)| *k == kind).map(|(_, i)| *i).ok_or_else(|| Stop::Lead(format!("unknown kind {kind}")))?;
    if retry >= 2 {
        return Err(Stop::Judge(format!("failed {retry} times")));
    }
    let i = (base + usize::from(paths > MANY || ac > MANY) + usize::from(retry)).min(2);
    if i == 2 && low {
        return Err(Stop::Judge(format!("team budget under {LOW_PCT}%, tier L blocked")));
    }
    Ok(TIERS[i])
}

/// 등급 → 폴백 체인에서 시도할 단계 (체인 순서, tier 일치 또는 NULL). 리드는 모델을 고르지 않는다
pub fn models<'a>(chain: &'a [Fallback], tier: &str) -> impl Iterator<Item = &'a Fallback> {
    chain.iter().filter(move |m| m.tier.as_deref().is_none_or(|t| t == tier))
}

/// 경로 두 개가 같은 파일을 가리킬 수 있나. 와일드카드는 첫 `*` 앞까지를 접두어로 본다
// ponytail: 접두어 비교라 `src/*.ts`와 `src/api/x.ts`도 겹침으로 본다 (순차가 되어 안전 쪽). 정확한 glob 교집합은 병렬이 부족할 때
pub fn overlap(a: &str, b: &str) -> bool {
    fn glob(p: &str) -> Option<&str> { p.find('*').map(|i| &p[..i]) }
    a == b || glob(a).is_some_and(|pa| b.starts_with(pa)) || glob(b).is_some_and(|pb| a.starts_with(pb))
}

/// 겹치는 진행 중 하위 Run이 있으면 (run sn, 겹친 내 경로). 첫 번째만
pub fn waits(paths: &[String], siblings: &[(i64, Vec<String>)]) -> Option<(i64, String)> {
    siblings.iter().find_map(|(sn, other)| paths.iter().find(|p| other.iter().any(|o| overlap(p, o))).map(|p| (*sn, p.clone())))
}

/// @ASK 경로 추가 판정. 전부 받은 경로와 인접(같은 폴더 · glob 안)이면 추가할 경로, 하나라도 밖이거나 횟수 초과면 Lead
pub fn ask(want: &[String], granted: &[String], asked: u8) -> Result<Vec<String>, Stop> {
    if asked >= ASK_MAX {
        return Err(Stop::Lead(format!("@ASK paths over {ASK_MAX} times")));
    }
    // 폴더: 와일드카드면 `*` 앞, 파일이면 마지막 `/`까지. 루트(빈 폴더)는 인접으로 보지 않는다
    let dir = |p: &str| p.find('*').map_or_else(|| p.rfind('/').map_or("", |i| &p[..=i]), |i| &p[..i]).to_owned();
    let near = |w: &str| granted.iter().any(|g| {
        let d = dir(g);
        !d.is_empty() && if g.contains('*') { w.starts_with(&d) } else { dir(w) == d }
    });
    match want.iter().find(|w| !near(w)) {
        Some(far) => Err(Stop::Lead(format!("@ASK path {far} outside paths"))),
        None => Ok(want.iter().filter(|w| !granted.contains(w)).cloned().collect()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BRIEF: &str = "@TASK v1
id: T129.1  parent: R81  mode: runner  kind: implement
goal: login form error states
ac: [1 err_401_inline, 2 err_429_banner]
paths: [src/routes/login/+page.svelte, src/lib/auth/errors.ts]
limits: {tok: 30k, min: 10, retry: 1}
ctx: []          # rules only
";

    fn ctx(siblings: &[(i64, Vec<String>)]) -> Ctx<'_> {
        Ctx {
            lead_num: 81, team_mode: "runner", allow: "sub,runner", task_mode: None, retry: 0, budget: Some(200_000), used: 0,
            children: 0, max_child: 3, team_run: 1, team_max: 3, siblings,
        }
    }

    /// 형식 · 필수 키
    #[test]
    fn brief() {
        let b = parse(BRIEF).unwrap();
        assert_eq!((b.id.as_str(), b.kind.as_str(), b.tok), ("T129.1", "implement", Some(30_000)));
        assert_eq!(b.ac, ["1 err_401_inline", "2 err_429_banner"]);
        assert_eq!(b.paths.len(), 2);
        assert!(matches!(parse(&BRIEF.replace("goal: login form error states", "goal:")), Err(Stop::Lead(_))));
        assert!(matches!(parse(&BRIEF.replace("paths: [src/routes/login/+page.svelte, src/lib/auth/errors.ts]", "paths: []")), Err(Stop::Lead(_))));
        assert!(matches!(parse(&BRIEF.replace("@TASK v1", "@TASK v2")), Err(Stop::Lead(_))));
    }

    /// 방식 · parent · 예산 · 대기
    #[test]
    fn verdict() {
        let b = parse(BRIEF).unwrap();
        assert_eq!(check(&b, &ctx(&[])), Ok(Plan { mode: "runner".into(), tier: Some("M"), wait: None }));
        let fork = parse(&BRIEF.replace("mode: runner", "mode: fork")).unwrap();
        assert!(matches!(check(&fork, &ctx(&[])), Err(Stop::Lead(_))));
        let other = parse(&BRIEF.replace("R81", "R80")).unwrap();
        assert!(matches!(check(&other, &ctx(&[])), Err(Stop::Lead(_))));
        // 방식을 비우면 태스크 방식 → sub는 등급 없음
        let none = parse(&BRIEF.replace("  mode: runner", "")).unwrap();
        assert_eq!(check(&none, &Ctx { task_mode: Some("sub"), ..ctx(&[]) }).unwrap().tier, None);
        assert!(matches!(check(&b, &Ctx { used: 180_000, ..ctx(&[]) }), Err(Stop::Judge(_))));
        assert_eq!(check(&b, &Ctx { children: 3, ..ctx(&[]) }).unwrap().wait, Some(Wait::Cap));
        let sib = [(90, vec!["src/lib/auth/**".to_owned()])];
        assert_eq!(check(&b, &ctx(&sib)).unwrap().wait, Some(Wait::Paths(90, "src/lib/auth/errors.ts".into())));
    }

    /// kind 기본 · 많음 +1 · 재시도 +1 · 상한 · 예산
    #[test]
    fn tiers() {
        assert_eq!(tier("search", 1, 1, 0, false), Ok("S"));
        assert_eq!(tier("search", 6, 1, 0, false), Ok("M"));
        assert_eq!(tier("fix", 1, 1, 1, false), Ok("L"));
        assert_eq!(tier("review", 9, 9, 1, false), Ok("L"));
        assert!(matches!(tier("fix", 1, 1, 2, false), Err(Stop::Judge(_))));
        assert!(matches!(tier("debug", 1, 1, 0, true), Err(Stop::Judge(_))));
        assert!(matches!(tier("chat", 1, 1, 0, false), Err(Stop::Lead(_))));
    }

    /// 등급 칩이 맞거나 비어 있는 단계만, 체인 순서
    #[test]
    fn chain() {
        let step = |sn, tier: Option<&str>| Fallback { runtime_sn: 1, connection_sn: 1, model_sn: Some(sn), switch_rule: None, max_level: None, tier: tier.map(str::to_owned) };
        let c = [step(1, Some("S")), step(2, None), step(3, Some("L"))];
        assert_eq!(models(&c, "S").filter_map(|m| m.model_sn).collect::<Vec<_>>(), [1, 2]);
        assert_eq!(models(&c, "M").filter_map(|m| m.model_sn).collect::<Vec<_>>(), [2]);
    }

    #[test]
    fn overlaps() {
        assert!(overlap("src/api/*", "src/api/user.ts"));
        assert!(overlap("src/**", "src/api/*"));
        assert!(overlap("a.ts", "a.ts"));
        assert!(!overlap("src/a*.ts", "src/b*.ts"));
        assert!(!overlap("src/api/user.ts", "src/api/team.ts"));
    }

    /// 같은 폴더 · glob 안은 추가, 밖 · 루트 · 3번째는 리드에게
    #[test]
    fn asks() {
        let g = ["src/lib/auth/errors.ts".to_owned(), "src/routes/login/**".to_owned(), "README.md".to_owned()];
        let want = |p: &[&str]| p.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(ask(&want(&["src/lib/auth/token.ts", "src/routes/login/x/y.ts"]), &g, 0), Ok(want(&["src/lib/auth/token.ts", "src/routes/login/x/y.ts"])));
        assert_eq!(ask(&want(&["src/lib/auth/errors.ts"]), &g, 1), Ok(vec![]));
        assert!(matches!(ask(&want(&["src/lib/db/pool.ts"]), &g, 0), Err(Stop::Lead(_))));
        assert!(matches!(ask(&want(&["package.json"]), &g, 0), Err(Stop::Lead(_))));
        assert!(matches!(ask(&want(&["src/lib/auth/token.ts"]), &g, 2), Err(Stop::Lead(_))));
    }
}
