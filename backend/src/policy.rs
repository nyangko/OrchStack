//! Orch 진행 정책: 팀(tbl_team) · 프로젝트(tbl_project)의 orch_mode · timer_sec · is_pause_on_view · level_json · guard_json.
//! 프로젝트 값이 NULL이면 팀 값, 팀 level_json · guard_json이 NULL이면 기본값을 쓴다. 저장은 팀 · 프로젝트 PATCH가 한다 (경로는 team.rs · project.rs)
use crate::{entity::{tbl_project as pj, tbl_team as tm}, error::{Error, Res}, event::Ev};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, sea_query::Expr};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;

const MODES: [&str; 3] = ["manual", "auto", "full_auto"];
const HANDLES: [&str; 4] = ["auto", "timer", "wait", "block"];
const NO_REPLY: [&str; 4] = ["proceed", "orch_decide", "keep_wait", "none"];
const CODES: [&str; 6] = ["auto_streak", "reject_loop", "same_failure", "issue_budget", "orch_new_task", "user_absent"];
const UNITS: [&str; 3] = ["count", "token", "minute"];
const SCOPES: [&str; 3] = ["issue", "task", "team"];
const TRIGGERS: [&str; 2] = ["stop", "to_manual"];

/// 작업 레벨별 처리 (L0 ~ L4)
#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct Level {
    pub level: i64,
    pub name: String,
    pub example: Option<String>,
    /// auto | timer | wait | block (L4는 block 고정)
    pub handle: String,
    /// 응답 대기(분)
    pub wait_min: Option<i64>,
    /// proceed | orch_decide | keep_wait | none
    pub no_reply: Option<String>,
    /// 변경 불가 (L4) · 저장할 때는 무시되고 L4만 잠긴다
    #[serde(default)]
    pub is_locked: i64,
}

/// 루프 가드
#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct Guard {
    /// auto_streak | reject_loop | same_failure | issue_budget | orch_new_task | user_absent
    pub code: String,
    pub name: String,
    pub threshold: i64,
    /// count | token | minute
    pub threshold_unit: String,
    /// issue | task | team
    pub scope: String,
    pub sub_threshold: Option<i64>,
    /// stop | to_manual
    pub on_trigger: String,
    pub is_enabled: i64,
    /// 마지막으로 걸린 시각 (저장할 때는 기존 값이 유지된다)
    #[serde(default)]
    pub trigger_at: Option<String>,
}

/// 프로젝트에 적용되는 진행 정책 (프로젝트 값 → 팀 값 → 기본값을 합친 것)
#[derive(Clone)]
pub struct Policy {
    pub mode: String,
    pub timer_sec: i64,
    pub levels: Vec<Level>,
    pub guards: Vec<Guard>,
}

/// 기본 레벨: L0 auto · L1 timer · L2 wait 10분(Orch가 대신 결정) · L3 wait(계속 대기) · L4 block 잠금
pub fn default_levels() -> Vec<Level> {
    let lv = |level, name: &str, example: &str, handle: &str, wait_min, no_reply: Option<&str>| Level {
        level, name: name.into(), example: Some(example.into()), handle: handle.into(), wait_min, no_reply: no_reply.map(Into::into), is_locked: (level == 4) as i64,
    };
    vec![
        lv(0, "내부 작업", "파일 읽기 · 테스트 실행", "auto", None, None),
        lv(1, "일반 작업", "코드 수정 · 커밋 · 다음 태스크 배정", "timer", None, None),
        lv(2, "모호한 판단", "요구사항 해석 · 설계 선택지", "wait", Some(10), Some("orch_decide")),
        lv(3, "중요한 변경", "의존성 · 스키마 변경", "wait", None, Some("keep_wait")),
        lv(4, "위험 작업", "force push · 배포 · 삭제", "block", None, Some("none")),
    ]
}

/// 기본 가드: auto_streak 10 · reject_loop 3 · same_failure 3 · orch_new_task 5 (모두 켬 · 걸리면 멈춤)
pub fn default_guards() -> Vec<Guard> {
    let gd = |code: &str, name: &str, threshold, scope: &str| Guard {
        code: code.into(), name: name.into(), threshold, threshold_unit: "count".into(), scope: scope.into(), sub_threshold: None, on_trigger: "stop".into(), is_enabled: 1, trigger_at: None,
    };
    vec![gd("auto_streak", "연속 자동 진행", 10, "team"), gd("reject_loop", "반려 → 재작업 반복", 3, "task"), gd("same_failure", "같은 실패 반복", 3, "task"), gd("orch_new_task", "Orch가 만든 새 태스크", 5, "issue")]
}

/// JSON 배열 컬럼 → 목록. 비었거나 깨졌으면 None (호출하는 쪽이 기본값 · 팀 값으로)
fn arr<T: serde::de::DeserializeOwned>(j: &Option<String>) -> Option<Vec<T>> {
    j.as_deref().and_then(|s| serde_json::from_str(s).ok())
}

/// 팀의 레벨 (저장값 또는 기본값)
pub fn team_levels(t: &tm::Model) -> Vec<Level> { arr(&t.level_json).unwrap_or_else(default_levels) }
/// 팀의 가드 (저장값 또는 기본값)
pub fn team_guards(t: &tm::Model) -> Vec<Guard> { arr(&t.guard_json).unwrap_or_else(default_guards) }
/// 프로젝트가 따로 정한 레벨 (없으면 None)
pub fn project_levels(p: &pj::Model) -> Option<Vec<Level>> { arr(&p.level_json) }
/// 프로젝트가 따로 정한 가드 (없으면 None)
pub fn project_guards(p: &pj::Model) -> Option<Vec<Guard>> { arr(&p.guard_json) }

/// 프로젝트의 진행 정책 (프로젝트 값 → 팀 값 → 기본값). 팀이 없는 프로젝트는 None
pub async fn load(db: &impl ConnectionTrait, project_sn: i64) -> Res<Option<Policy>> {
    let Some(p) = pj::Entity::find_by_id(project_sn).one(db).await? else { return Ok(None) };
    let Some(t) = (match p.team_sn { Some(ts) => tm::Entity::find_by_id(ts).one(db).await?, None => None }) else { return Ok(None) };
    Ok(Some(Policy {
        mode: p.orch_mode.clone().unwrap_or_else(|| t.orch_mode.clone()), timer_sec: p.timer_sec.unwrap_or(t.timer_sec),
        levels: project_levels(&p).unwrap_or_else(|| team_levels(&t)), guards: project_guards(&p).unwrap_or_else(|| team_guards(&t)),
    }))
}

/// 모드 · 타이머 · 대기 설정 값 검사. 틀리면 422
pub fn check_basic(mode: Option<&str>, timer_sec: Option<i64>, pause: Option<i64>) -> Res<()> {
    if mode.is_some_and(|m| !MODES.contains(&m)) || timer_sec.is_some_and(|t| !(1..=86_400).contains(&t)) || pause.is_some_and(|v| !(0..=1).contains(&v)) {
        return Err(Error::invalid("orch_mode in manual | auto | full_auto, timer_sec 1..=86400, is_pause_on_view 0 | 1".into()));
    }
    Ok(())
}

/// 레벨 목록 검사: L0~L4가 정확히 하나씩 · 값 범위 · L4는 block 고정. 틀리면 422
pub fn check_levels(ls: &[Level]) -> Res<()> {
    let bad = |m: &str| Err(Error::invalid(m.into()));
    let mut ns: Vec<i64> = ls.iter().map(|x| x.level).collect();
    ns.sort_unstable();
    if ns != [0, 1, 2, 3, 4] {
        return bad("levels must be exactly L0..L4");
    }
    for x in ls {
        if !HANDLES.contains(&x.handle.as_str()) || x.no_reply.as_deref().is_some_and(|n| !NO_REPLY.contains(&n)) || x.wait_min.is_some_and(|m| m < 1) {
            return bad("level handle in auto | timer | wait | block, no_reply in proceed | orch_decide | keep_wait | none, wait_min >= 1");
        }
        if x.level == 4 && x.handle != "block" {
            return bad("L4 is locked: handle must be block");
        }
    }
    Ok(())
}

/// 가드 목록 검사: 코드 중복 없음 · 값 범위. 틀리면 422
pub fn check_guards(gs: &[Guard]) -> Res<()> {
    if gs.iter().enumerate().any(|(i, g)| gs[..i].iter().any(|o| o.code == g.code)) {
        return Err(Error::invalid("guard code must be unique".into()));
    }
    if gs.iter().any(|x| !CODES.contains(&x.code.as_str()) || !UNITS.contains(&x.threshold_unit.as_str()) || !SCOPES.contains(&x.scope.as_str())
        || !TRIGGERS.contains(&x.on_trigger.as_str()) || x.threshold < 1 || !(0..=1).contains(&x.is_enabled)) {
        return Err(Error::invalid("guard code · threshold_unit · scope · on_trigger must be known, threshold >= 1, is_enabled 0 | 1".into()));
    }
    Ok(())
}

/// 저장할 레벨: L0~L4 순서로 정렬하고 L4만 잠근다
pub fn save_levels(mut ls: Vec<Level>) -> String {
    ls.sort_by_key(|x| x.level);
    for x in &mut ls { x.is_locked = (x.level == 4) as i64; }
    json!(ls).to_string()
}

/// 저장할 가드: 같은 코드의 기존 trigger_at은 유지한다
pub fn save_guards(gs: Vec<Guard>, old: &[Guard]) -> String {
    let gs: Vec<Guard> = gs.into_iter().map(|g| Guard { trigger_at: old.iter().find(|o| o.code == g.code).and_then(|o| o.trigger_at.clone()), ..g }).collect();
    json!(gs).to_string()
}

/// 가드가 걸려 모드를 manual로 바꾼다 (프로젝트가 모드를 따로 정했으면 프로젝트, 아니면 팀). OrchPolicyUpdated 이벤트 (대상 team)
pub async fn to_manual(tx: &impl ConnectionTrait, project_sn: i64) -> Res<Option<Ev>> {
    let Some(p) = pj::Entity::find_by_id(project_sn).one(tx).await? else { return Ok(None) };
    let Some(ts) = p.team_sn else { return Ok(None) };
    if p.orch_mode.is_some() {
        pj::Entity::update_many().filter(pj::Column::Sn.eq(project_sn)).col_expr(pj::Column::OrchMode, "manual".into()).exec(tx).await?;
    } else {
        tm::Entity::update_many().filter(tm::Column::Sn.eq(ts)).col_expr(tm::Column::OrchMode, "manual".into()).exec(tx).await?;
    }
    Ok(Some(Ev::new(Some(project_sn), "team", ts, "OrchPolicyUpdated", &json!({ "project_sn": project_sn, "orch_mode": "manual" }))))
}

/// 가드가 걸린 시각(trigger_at)을 남긴다 (가드를 저장한 쪽 JSON에만 · 기본값을 쓰는 중이면 남길 곳이 없어 건너뛴다)
pub async fn guard_hit(tx: &impl ConnectionTrait, project_sn: i64, code: &str) -> Res<()> {
    let Some(p) = pj::Entity::find_by_id(project_sn).one(tx).await? else { return Ok(()) };
    let Some(ts) = p.team_sn else { return Ok(()) };
    let now = crate::orch_rule::at(tx, "+0 seconds").await?;
    let stamp = |mut gs: Vec<Guard>| -> String {
        for g in gs.iter_mut().filter(|g| g.code == code) { g.trigger_at = Some(now.clone()); }
        json!(gs).to_string()
    };
    if let Some(gs) = project_guards(&p) {
        pj::Entity::update_many().filter(pj::Column::Sn.eq(project_sn)).col_expr(pj::Column::GuardJson, stamp(gs).into()).exec(tx).await?;
    } else if let Some(t) = tm::Entity::find_by_id(ts).one(tx).await?.filter(|t| t.guard_json.is_some()) {
        tm::Entity::update_many().filter(tm::Column::Sn.eq(ts)).col_expr(tm::Column::GuardJson, stamp(team_guards(&t)).into()).col_expr(tm::Column::UpdateAt, Expr::cust("datetime('now')")).exec(tx).await?;
    }
    Ok(())
}
