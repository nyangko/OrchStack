//! 팀 Orch 진행 정책 (tbl_orch_policy · _level · _guard) 조회 · 저장. 행이 없으면 기본값을 응답으로만 주고, 저장은 PUT 때 한다.
//! 프로젝트별 정책이 있으면 그것을, 없으면 팀 기본, 그것도 없으면 기본값을 쓴다
use crate::{entity::{tbl_orch_guard as g, tbl_orch_policy as p, tbl_orch_policy_level as l, tbl_project as pj, tbl_team as tm},
    error::{Body, Error, ErrorBody, Res, Sn}, event::{self, Ev}};
use axum::{Json, extract::{Query, State}};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder, sea_query::Expr};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};

const MODES: [&str; 3] = ["manual", "auto", "full_auto"];
const HANDLES: [&str; 4] = ["auto", "timer", "wait", "block"];
const NO_REPLY: [&str; 4] = ["proceed", "orch_decide", "keep_wait", "none"];
const CODES: [&str; 6] = ["auto_streak", "reject_loop", "same_failure", "issue_budget", "orch_new_task", "user_absent"];
const UNITS: [&str; 3] = ["count", "token", "minute"];
const SCOPES: [&str; 3] = ["issue", "task", "team"];
const TRIGGERS: [&str; 2] = ["stop", "to_manual"];

/// policy 관련 경로 묶음
pub fn routes() -> OpenApiRouter<DatabaseConnection> {
    OpenApiRouter::new().routes(routes!(read, save))
}

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
    /// 마지막으로 걸린 시각 (읽기 전용)
    #[serde(default)]
    pub trigger_at: Option<String>,
    #[serde(skip)]
    pub sn: Option<i64>,
}

/// 진행 정책. PUT에서는 team_sn · project_sn · is_saved를 무시한다
#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct Policy {
    #[serde(default)]
    pub team_sn: i64,
    /// 이 정책이 속한 프로젝트 (null = 팀 기본)
    #[serde(default)]
    pub project_sn: Option<i64>,
    /// manual | auto | full_auto
    pub mode: String,
    /// 자동 진행 전 대기(초)
    pub timer_sec: i64,
    pub is_pause_on_view: i64,
    pub levels: Vec<Level>,
    pub guards: Vec<Guard>,
    /// 저장된 행이면 1, 기본값 응답이면 0
    #[serde(default)]
    pub is_saved: i64,
}

/// 정책 쿼리
#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct Scope {
    /// 프로젝트 번호 (없으면 팀 기본)
    project: Option<i64>,
}

/// 기본값: auto · 5초 · L0 auto · L1 timer · L2 wait 10분(Orch가 대신 결정) · L3 wait(계속 대기) · L4 block 잠금 · 가드 4종
pub fn defaults(team_sn: i64) -> Policy {
    let lv = |level, name: &str, example: &str, handle: &str, wait_min, no_reply: Option<&str>| Level {
        level, name: name.into(), example: Some(example.into()), handle: handle.into(), wait_min, no_reply: no_reply.map(Into::into), is_locked: (level == 4) as i64,
    };
    let gd = |code: &str, name: &str, threshold, scope: &str| Guard {
        code: code.into(), name: name.into(), threshold, threshold_unit: "count".into(), scope: scope.into(), sub_threshold: None,
        on_trigger: "stop".into(), is_enabled: 1, trigger_at: None, sn: None,
    };
    Policy {
        team_sn, project_sn: None, mode: "auto".into(), timer_sec: 5, is_pause_on_view: 1, is_saved: 0,
        levels: vec![
            lv(0, "내부 작업", "파일 읽기 · 테스트 실행", "auto", None, None),
            lv(1, "일반 작업", "코드 수정 · 커밋 · 다음 태스크 배정", "timer", None, None),
            lv(2, "모호한 판단", "요구사항 해석 · 설계 선택지", "wait", Some(10), Some("orch_decide")),
            lv(3, "중요한 변경", "의존성 · 스키마 변경", "wait", None, Some("keep_wait")),
            lv(4, "위험 작업", "force push · 배포 · 삭제", "block", None, Some("none")),
        ],
        guards: vec![
            gd("auto_streak", "연속 자동 진행", 10, "team"), gd("reject_loop", "반려 → 재작업 반복", 3, "task"),
            gd("same_failure", "같은 실패 반복", 3, "task"), gd("orch_new_task", "Orch가 만든 새 태스크", 5, "issue"),
        ],
    }
}

/// 팀 · 프로젝트에 맞는 정책: 프로젝트 행 → 팀 기본 행 → 기본값. 팀이 없으면 404 (호출자가 확인)
pub async fn load(db: &impl ConnectionTrait, team_sn: i64, project_sn: Option<i64>) -> Res<Policy> {
    if project_sn.is_some()
        && let Some(m) = row(db, team_sn, project_sn).await?
    {
        return fill(db, m).await;
    }
    match row(db, team_sn, None).await? {
        Some(m) => fill(db, m).await,
        None => Ok(defaults(team_sn)),
    }
}

/// (팀, 프로젝트)가 정확히 같은 정책 행
async fn row(db: &impl ConnectionTrait, team_sn: i64, project_sn: Option<i64>) -> Res<Option<p::Model>> {
    let f = p::Entity::find().filter(p::Column::TeamSn.eq(team_sn));
    let f = match project_sn { Some(v) => f.filter(p::Column::ProjectSn.eq(v)), None => f.filter(p::Column::ProjectSn.is_null()) };
    Ok(f.order_by_asc(p::Column::Sn).one(db).await?)
}

/// 정책 행 + 레벨 · 가드
async fn fill(db: &impl ConnectionTrait, m: p::Model) -> Res<Policy> {
    let levels = l::Entity::find().filter(l::Column::PolicySn.eq(m.sn)).order_by_asc(l::Column::Level).all(db).await?.into_iter().map(|x| Level {
        level: x.level, name: x.name, example: x.example, handle: x.handle, wait_min: x.wait_min, no_reply: x.no_reply, is_locked: x.is_locked,
    }).collect();
    let guards = g::Entity::find().filter(g::Column::PolicySn.eq(m.sn)).order_by_asc(g::Column::Sn).all(db).await?.into_iter().map(|x| Guard {
        code: x.code, name: x.name, threshold: x.threshold, threshold_unit: x.threshold_unit, scope: x.scope, sub_threshold: x.sub_threshold,
        on_trigger: x.on_trigger, is_enabled: x.is_enabled, trigger_at: x.trigger_at, sn: Some(x.sn),
    }).collect();
    Ok(Policy { team_sn: m.team_sn, project_sn: m.project_sn, mode: m.mode, timer_sec: m.timer_sec, is_pause_on_view: m.is_pause_on_view, levels, guards, is_saved: 1 })
}

/// 저장 검사. 틀리면 422
fn check(b: &Policy) -> Res<()> {
    let bad = |m: &str| Err(Error::invalid(m.into()));
    if !MODES.contains(&b.mode.as_str()) || !(1..=86_400).contains(&b.timer_sec) || !(0..=1).contains(&b.is_pause_on_view) {
        return bad("mode in manual | auto | full_auto, timer_sec 1..=86400, is_pause_on_view 0 | 1");
    }
    let mut ls: Vec<i64> = b.levels.iter().map(|x| x.level).collect();
    ls.sort_unstable();
    if ls != [0, 1, 2, 3, 4] {
        return bad("levels must be exactly L0..L4");
    }
    for x in &b.levels {
        if !HANDLES.contains(&x.handle.as_str()) || x.no_reply.as_deref().is_some_and(|n| !NO_REPLY.contains(&n)) || x.wait_min.is_some_and(|m| m < 1) {
            return bad("level handle in auto | timer | wait | block, no_reply in proceed | orch_decide | keep_wait | none, wait_min >= 1");
        }
        if x.level == 4 && x.handle != "block" {
            return bad("L4 is locked: handle must be block");
        }
    }
    let mut cs: Vec<&str> = b.guards.iter().map(|x| x.code.as_str()).collect();
    cs.sort_unstable();
    cs.dedup();
    if cs.len() != b.guards.len() {
        return bad("guard code must be unique");
    }
    for x in &b.guards {
        if !CODES.contains(&x.code.as_str()) || !UNITS.contains(&x.threshold_unit.as_str()) || !SCOPES.contains(&x.scope.as_str())
            || !TRIGGERS.contains(&x.on_trigger.as_str()) || x.threshold < 1 || !(0..=1).contains(&x.is_enabled) {
            return bad("guard code · threshold_unit · scope · on_trigger must be known, threshold >= 1, is_enabled 0 | 1");
        }
    }
    Ok(())
}

/// (팀, 프로젝트) 행을 저장한다: 없으면 만들고, 있으면 고친 뒤 레벨 · 가드를 전체 교체 (가드의 마지막으로 걸린 시각은 유지). OrchPolicyUpdated 이벤트
pub(crate) async fn set(tx: &DatabaseTransaction, team_sn: i64, project_sn: Option<i64>, b: &Policy) -> Res<(Policy, Ev)> {
    check(b)?;
    let (sn, old) = match row(tx, team_sn, project_sn).await? {
        Some(m) => {
            p::Entity::update_many().filter(p::Column::Sn.eq(m.sn)).col_expr(p::Column::Mode, b.mode.clone().into()).col_expr(p::Column::TimerSec, b.timer_sec.into())
                .col_expr(p::Column::IsPauseOnView, b.is_pause_on_view.into()).col_expr(p::Column::UpdateAt, Expr::cust("datetime('now')")).exec(tx).await?;
            let old = g::Entity::find().filter(g::Column::PolicySn.eq(m.sn)).all(tx).await?;
            l::Entity::delete_many().filter(l::Column::PolicySn.eq(m.sn)).exec(tx).await?;
            g::Entity::delete_many().filter(g::Column::PolicySn.eq(m.sn)).exec(tx).await?;
            (m.sn, old)
        }
        None => {
            let m = p::ActiveModel { team_sn: Set(team_sn), project_sn: Set(project_sn), mode: Set(b.mode.clone()), timer_sec: Set(b.timer_sec),
                is_pause_on_view: Set(b.is_pause_on_view), ..Default::default() }.insert(tx).await?;
            (m.sn, Vec::new())
        }
    };
    for x in &b.levels {
        l::ActiveModel { policy_sn: Set(sn), level: Set(x.level), name: Set(x.name.clone()), example: Set(x.example.clone()), handle: Set(x.handle.clone()),
            wait_min: Set(x.wait_min), no_reply: Set(x.no_reply.clone()), is_locked: Set((x.level == 4) as i64), ..Default::default() }.insert(tx).await?;
    }
    for x in &b.guards {
        g::ActiveModel { policy_sn: Set(sn), code: Set(x.code.clone()), name: Set(x.name.clone()), threshold: Set(x.threshold), threshold_unit: Set(x.threshold_unit.clone()),
            scope: Set(x.scope.clone()), sub_threshold: Set(x.sub_threshold), on_trigger: Set(x.on_trigger.clone()), is_enabled: Set(x.is_enabled),
            trigger_at: Set(old.iter().find(|o| o.code == x.code).and_then(|o| o.trigger_at.clone())), ..Default::default() }.insert(tx).await?;
    }
    let out = fill(tx, p::Entity::find_by_id(sn).one(tx).await?.ok_or_else(Error::not_found)?).await?;
    let ev = Ev::new(project_sn, "team", team_sn, "OrchPolicyUpdated", &json!({ "project_sn": project_sn, "mode": out.mode, "timer_sec": out.timer_sec }));
    Ok((out, ev))
}

/// 팀 · (선택) 프로젝트 확인. 없으면 404, 프로젝트가 다른 팀이면 422
async fn scope(db: &impl ConnectionTrait, team_sn: i64, project: Option<i64>) -> Res<()> {
    tm::Entity::find_by_id(team_sn).one(db).await?.ok_or_else(Error::not_found)?;
    if let Some(ps) = project {
        let pr = pj::Entity::find_by_id(ps).one(db).await?.ok_or_else(Error::not_found)?;
        if pr.team_sn != Some(team_sn) {
            return Err(Error::invalid("project is not in this team".into()));
        }
    }
    Ok(())
}

/// 진행 정책 조회. 프로젝트 → 팀 기본 → 기본값 순 (기본값이면 is_saved = 0). 팀이 없으면 404
#[utoipa::path(operation_id = "policy_read", get, path = "/teams/{sn}/policy", params(("sn" = i64, Path, description = "팀 번호"), Scope), responses((status = 200, body = Policy), (status = "default", body = ErrorBody)))]
async fn read(State(db): State<DatabaseConnection>, Sn(sn): Sn, Query(q): Query<Scope>) -> Res<Json<Policy>> {
    scope(&db, sn, q.project).await?;
    load(&db, sn, q.project).await.map(Json)
}

/// 진행 정책 저장 (OrchPolicyUpdated · 팀 대상). mode · timer_sec · levels · guards 전체 교체. L4는 block 고정 (422), 레벨은 L0~L4 모두
#[utoipa::path(operation_id = "policy_save", put, path = "/teams/{sn}/policy", params(("sn" = i64, Path, description = "팀 번호"), Scope), request_body = Policy, responses((status = 200, body = Policy), (status = "default", body = ErrorBody)))]
async fn save(State(db): State<DatabaseConnection>, Sn(sn): Sn, Query(q): Query<Scope>, Body(b): Body<Policy>) -> Res<Json<Policy>> {
    scope(&db, sn, q.project).await?;
    let out = event::run(&db, async |tx| set(tx, sn, q.project, &b).await.map(|(out, ev)| (out, vec![ev]))).await?;
    Ok(Json(out))
}
