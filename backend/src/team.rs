//! tbl_team · tbl_member CRUD. 쓰기는 event::run 경유 (Team* · Member* 이벤트). 멤버는 템플릿의 live 버전 프로필을 복사해 만든다
use crate::{agent, policy::{self, Guard, Level}, run, task, entity::{tbl_agent_profile as ap, tbl_ask as ak, tbl_member as mb, tbl_run as rn, tbl_task as tk, tbl_team::{self as tm, Entity as Tbl}, tbl_template as tp},
    error::{Body, Error, ErrorBody, Res, Sn, in_use}, event::{self, Ev}};
use axum::{Json, extract::State, http::StatusCode};
use sea_orm::{ActiveModelTrait, ActiveValue::{NotSet, Set}, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect, sea_query::Expr};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use serde_json::json;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

/// 허용되는 팀 종류
const KINDS: [&str; 2] = ["orch", "project"];
/// 허용되는 멤버 상태
const STATUS: [&str; 3] = ["active", "paused", "archived"];
/// 허용되는 하위 작업 방식 (#67)
pub(crate) const SPAWN: [&str; 3] = ["sub", "fork", "runner"];

/// team · member 관련 경로 묶음
pub fn routes() -> OpenApiRouter<DatabaseConnection> {
    OpenApiRouter::new()
        .routes(routes!(list, create))
        .routes(routes!(read, update, remove))
        .routes(routes!(members, add))
        .routes(routes!(member, edit, delete))
}

/// 팀 (API 응답 형태)
#[derive(Serialize, ToSchema)]
pub struct Team {
    sn: i64,
    name: String,
    /// orch | project
    kind: String,
    daily_token_budget: Option<i64>,
    context_warn_percent: i64,
    max_concurrent_run: i64,
    /// 하위 작업 기본 방식: sub | fork | runner
    spawn_mode: String,
    /// 허용 방식 (쉼표 구분)
    spawn_allow: String,
    /// 리드 Run 1개당 동시 하위 작업 수
    max_child_run: i64,
    is_review_required: i64,
    /// before_merge | before_done
    review_stage: String,
    repo_scope: Option<String>,
    /// read | branch | push
    repo_permission: String,
    /// Orch 진행 모드: manual | auto | full_auto (프로젝트가 따로 정하면 프로젝트 값이 이긴다)
    orch_mode: String,
    /// 자동 진행 전 대기(초)
    timer_sec: i64,
    /// 카드를 보고 있으면 타이머 멈춤
    is_pause_on_view: i64,
    /// 작업 레벨별 처리 L0 ~ L4 (저장 전이면 기본값)
    levels: Vec<Level>,
    /// 루프 가드 (저장 전이면 기본값)
    guards: Vec<Guard>,
    sort: i64,
    create_at: String,
    update_at: String,
}

impl From<tm::Model> for Team {
    fn from(m: tm::Model) -> Self {
        Self {
            levels: policy::team_levels(&m), guards: policy::team_guards(&m), orch_mode: m.orch_mode, timer_sec: m.timer_sec, is_pause_on_view: m.is_pause_on_view,
            sn: m.sn, name: m.name, kind: m.kind, daily_token_budget: m.daily_token_budget, context_warn_percent: m.context_warn_percent,
            max_concurrent_run: m.max_concurrent_run, spawn_mode: m.spawn_mode, spawn_allow: m.spawn_allow, max_child_run: m.max_child_run,
            is_review_required: m.is_review_required, review_stage: m.review_stage,
            repo_scope: m.repo_scope, repo_permission: m.repo_permission, sort: m.sort, create_at: m.create_at, update_at: m.update_at,
        }
    }
}

/// 멤버 (API 응답 형태)
#[derive(Serialize, ToSchema)]
pub struct Member {
    sn: i64,
    team_sn: i64,
    profile_sn: i64,
    template_sn: Option<i64>,
    template_version: Option<i64>,
    name: String,
    role_name: String,
    icon: Option<String>,
    color: Option<String>,
    is_orch: i64,
    /// active | paused | archived (저장값)
    status: String,
    /// 작업 상태 (계산값): running | waiting | idle | paused | archived — `work_of` 기준
    work: String,
    /// orch | task | wait
    first_task_mode: String,
    sort: i64,
    create_at: String,
    update_at: String,
}

impl From<mb::Model> for Member {
    fn from(m: mb::Model) -> Self {
        Self {
            sn: m.sn, team_sn: m.team_sn, profile_sn: m.profile_sn, template_sn: m.template_sn, template_version: m.template_version,
            name: m.name, role_name: m.role_name, icon: m.icon, color: m.color, is_orch: m.is_orch, work: if m.status == "active" { "idle".into() } else { m.status.clone() }, status: m.status,
            first_task_mode: m.first_task_mode, sort: m.sort, create_at: m.create_at, update_at: m.update_at,
        }
    }
}

/// 팀 생성 요청 본문. 나머지 설정은 DB 기본값으로 시작한다
#[derive(Deserialize, ToSchema)]
struct TeamNew {
    name: String,
    /// orch | project, 생략하면 project
    kind: Option<String>,
}

/// 팀 수정 요청 본문. 보낸 필드만 바꾼다
#[derive(Serialize, Deserialize, ToSchema)]
struct TeamPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    daily_token_budget: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    context_warn_percent: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_concurrent_run: Option<i64>,
    /// sub | fork | runner
    #[serde(skip_serializing_if = "Option::is_none")]
    spawn_mode: Option<String>,
    /// 쉼표 구분 (예: "sub,runner")
    #[serde(skip_serializing_if = "Option::is_none")]
    spawn_allow: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_child_run: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_review_required: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    review_stage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    repo_scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    repo_permission: Option<String>,
    /// manual | auto | full_auto
    #[serde(skip_serializing_if = "Option::is_none")]
    orch_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timer_sec: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_pause_on_view: Option<i64>,
    /// 작업 레벨별 처리 전체 교체 (L0 ~ L4 모두 · L4는 block 고정)
    #[serde(skip_serializing_if = "Option::is_none")]
    levels: Option<Vec<Level>>,
    /// 루프 가드 전체 교체 (마지막으로 걸린 시각은 유지)
    #[serde(skip_serializing_if = "Option::is_none")]
    guards: Option<Vec<Guard>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sort: Option<i64>,
}

/// 멤버 추가 요청 본문. template_sn이 있으면 live 버전 프로필을 복사하고 역할 · 아이콘 · 색을 템플릿에서 가져온다
#[derive(Deserialize, ToSchema)]
struct MemberNew {
    name: String,
    template_sn: Option<i64>,
    /// 템플릿 없이 만들 때 필수 (있으면 템플릿 값을 덮어쓴다)
    role_name: Option<String>,
    icon: Option<String>,
    color: Option<String>,
    /// orch | task | wait, 생략하면 orch
    first_task_mode: Option<String>,
}

/// 멤버 수정 요청 본문. 보낸 필드만 바꾼다
#[derive(Serialize, Deserialize, ToSchema)]
struct MemberPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    role_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<String>,
    /// active | paused | archived (일하는 중 · 대기 중은 Run으로 계산)
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    first_task_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sort: Option<i64>,
}

/// 팀 1건 읽기. 없으면 404
async fn get(db: &impl ConnectionTrait, sn: i64) -> Res<Team> {
    Tbl::find_by_id(sn).one(db).await?.map(Team::from).ok_or_else(Error::not_found)
}

/// 멤버들의 작업 상태 (저장하지 않고 계산 · §19). 멤버 수와 상관없이 Run · 요청 · 태스크를 IN 조건으로 한 번씩만 읽는다.
/// archived · paused = 저장값 · running = 리드 Run이 queued · starting · running · review ·
/// waiting = 리드 Run이 waiting이거나, 열린 판단 · 승인 요청이 있거나, 담당 todo가 전부 의존 대기 · idle = 나머지
/// deps = false면 의존 대기만인 멤버를 idle로 본다 (배정 후보 — 의존이 풀릴 때까지 다른 태스크를 맡을 수 있다)
pub(crate) async fn work_of(db: &impl ConnectionTrait, ms: &[mb::Model], deps: bool) -> Res<HashMap<i64, &'static str>> {
    let sns: Vec<i64> = ms.iter().filter(|m| m.status == "active").map(|m| m.sn).collect();
    let runs = rn::Entity::find().filter(rn::Column::MemberSn.is_in(sns.clone())).filter(rn::Column::ParentRunSn.is_null()).filter(rn::Column::Status.is_in(run::ACTIVE)).all(db).await?;
    let asks = ak::Entity::find().filter(ak::Column::MemberSn.is_in(sns.clone())).filter(ak::Column::Kind.is_in(["decision", "approval"])).filter(ak::Column::Status.is_in(["pending", "writing"])).all(db).await?;
    let todo = tk::Entity::find().filter(tk::Column::MemberSn.is_in(sns)).filter(tk::Column::Status.eq("todo")).all(db).await?;
    let blocked = task::waiting(db, todo.iter().map(|t| t.sn).collect()).await?;
    Ok(ms.iter().map(|m| {
        let mine = || runs.iter().filter(|r| r.member_sn == m.sn);
        let work = match m.status.as_str() {
            "active" if mine().any(|r| r.status != "waiting") => "running",
            "active" if mine().next().is_some() || asks.iter().any(|a| a.member_sn == Some(m.sn))
                || deps && { let t: Vec<_> = todo.iter().filter(|t| t.member_sn == Some(m.sn)).collect(); !t.is_empty() && t.iter().all(|t| blocked.contains(&t.sn)) } => "waiting",
            "active" => "idle",
            "paused" => "paused",
            _ => "archived",
        };
        (m.sn, work)
    }).collect())
}

/// 멤버 행들 → 응답 (작업 상태 포함 · 입력 순서 유지)
pub(crate) async fn with_work(db: &impl ConnectionTrait, ms: Vec<mb::Model>) -> Res<Vec<Member>> {
    let work = work_of(db, &ms, true).await?;
    Ok(ms.into_iter().map(|m| { let w = work[&m.sn]; Member { work: w.into(), ..Member::from(m) } }).collect())
}

/// 멤버 1건 읽기. 없으면 404
async fn one(db: &impl ConnectionTrait, sn: i64) -> Res<Member> {
    let m = mb::Entity::find_by_id(sn).one(db).await?.ok_or_else(Error::not_found)?;
    with_work(db, vec![m]).await?.pop().ok_or_else(Error::not_found)
}

/// 팀 목록 (sort → 번호순)
#[utoipa::path(operation_id = "team_list", get, path = "/teams", responses((status = 200, body = Vec<Team>), (status = "default", body = ErrorBody)))]
async fn list(State(db): State<DatabaseConnection>) -> Res<Json<Vec<Team>>> {
    Ok(Json(Tbl::find().order_by_asc(tm::Column::Sort).order_by_asc(tm::Column::Sn).all(&db).await?.into_iter().map(Team::from).collect()))
}

/// 팀 생성 (TeamCreated)
#[utoipa::path(operation_id = "team_create", post, path = "/teams", request_body = TeamNew, responses((status = 201, body = Team), (status = "default", body = ErrorBody)))]
async fn create(State(db): State<DatabaseConnection>, Body(b): Body<TeamNew>) -> Res<(StatusCode, Json<Team>)> {
    if b.kind.as_deref().is_some_and(|k| !KINDS.contains(&k)) {
        return Err(Error::invalid(format!("kind must be one of {KINDS:?}")));
    }
    let out = event::run(&db, async |tx| {
        let m = tm::ActiveModel { workspace_sn: Set(crate::WORKSPACE), name: Set(b.name), kind: b.kind.map_or(NotSet, Set), ..Default::default() };
        let out = Team::from(m.insert(tx).await?);
        let ev = Ev::new(None, "team", out.sn, "TeamCreated", &out);
        Ok((out, vec![ev]))
    }).await?;
    Ok((StatusCode::CREATED, Json(out)))
}

/// 팀 1건 조회. 없으면 404
#[utoipa::path(operation_id = "team_read", get, path = "/teams/{sn}", params(("sn" = i64, Path, description = "팀 번호")), responses((status = 200, body = Team), (status = "default", body = ErrorBody)))]
async fn read(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Team>> {
    get(&db, sn).await.map(Json)
}

/// 팀 부분 수정 (TeamUpdated). 없으면 404, 모르는 하위 작업 방식 · 허용 밖 기본 방식은 422
#[utoipa::path(operation_id = "team_update", patch, path = "/teams/{sn}", params(("sn" = i64, Path, description = "팀 번호")), request_body = TeamPatch, responses((status = 200, body = Team), (status = "default", body = ErrorBody)))]
async fn update(State(db): State<DatabaseConnection>, Sn(sn): Sn, Body(b): Body<TeamPatch>) -> Res<Json<Team>> {
    if b.spawn_mode.as_deref().is_some_and(|m| !SPAWN.contains(&m))
        || b.spawn_allow.as_deref().is_some_and(|a| a.split(',').any(|m| !SPAWN.contains(&m)))
        || b.max_child_run.is_some_and(|n| n < 1) {
        return Err(Error::invalid(format!("spawn_mode · spawn_allow must be in {SPAWN:?}, max_child_run >= 1")));
    }
    policy::check_basic(b.orch_mode.as_deref(), b.timer_sec, b.is_pause_on_view)?;
    if let Some(ls) = &b.levels { policy::check_levels(ls)?; }
    if let Some(gs) = &b.guards { policy::check_guards(gs)?; }
    let out = event::run(&db, async |tx| {
        let t = get(tx, sn).await?;
        // 기본 방식은 허용 방식 안에 있어야 한다
        let (mode, allow) = (b.spawn_mode.as_deref().unwrap_or(&t.spawn_mode), b.spawn_allow.as_deref().unwrap_or(&t.spawn_allow));
        if !allow.split(',').any(|m| m == mode) {
            return Err(Error::invalid(format!("spawn_mode {mode} is not in spawn_allow {allow}")));
        }
        use tm::Column as C;
        let mut q = Tbl::update_many().filter(C::Sn.eq(sn)).col_expr(C::UpdateAt, Expr::cust("datetime('now')"));
        if let Some(v) = &b.name { q = q.col_expr(C::Name, v.clone().into()); }
        if let Some(v) = b.daily_token_budget { q = q.col_expr(C::DailyTokenBudget, v.into()); }
        if let Some(v) = b.context_warn_percent { q = q.col_expr(C::ContextWarnPercent, v.into()); }
        if let Some(v) = b.max_concurrent_run { q = q.col_expr(C::MaxConcurrentRun, v.into()); }
        if let Some(v) = &b.spawn_mode { q = q.col_expr(C::SpawnMode, v.clone().into()); }
        if let Some(v) = &b.spawn_allow { q = q.col_expr(C::SpawnAllow, v.clone().into()); }
        if let Some(v) = b.max_child_run { q = q.col_expr(C::MaxChildRun, v.into()); }
        if let Some(v) = b.is_review_required { q = q.col_expr(C::IsReviewRequired, v.into()); }
        if let Some(v) = &b.review_stage { q = q.col_expr(C::ReviewStage, v.clone().into()); }
        if let Some(v) = &b.repo_scope { q = q.col_expr(C::RepoScope, v.clone().into()); }
        if let Some(v) = &b.repo_permission { q = q.col_expr(C::RepoPermission, v.clone().into()); }
        if let Some(v) = &b.orch_mode { q = q.col_expr(C::OrchMode, v.clone().into()); }
        if let Some(v) = b.timer_sec { q = q.col_expr(C::TimerSec, v.into()); }
        if let Some(v) = b.is_pause_on_view { q = q.col_expr(C::IsPauseOnView, v.into()); }
        if let Some(v) = &b.levels { q = q.col_expr(C::LevelJson, policy::save_levels(v.clone()).into()); }
        if let Some(v) = &b.guards { q = q.col_expr(C::GuardJson, policy::save_guards(v.clone(), &t.guards).into()); }
        if let Some(v) = b.sort { q = q.col_expr(C::Sort, v.into()); }
        if q.exec(tx).await?.rows_affected == 0 {
            return Err(Error::not_found());
        }
        let out = get(tx, sn).await?;
        Ok((out, vec![Ev::new(None, "team", sn, "TeamUpdated", &b)]))
    }).await?;
    Ok(Json(out))
}

/// 팀 삭제 (TeamDeleted). 멤버와 멤버 프로필도 지운다. Run 기록이 있는 멤버가 있으면 409
#[utoipa::path(operation_id = "team_remove", delete, path = "/teams/{sn}", params(("sn" = i64, Path, description = "팀 번호")), responses((status = 204, description = "삭제됨"), (status = "default", body = ErrorBody)))]
async fn remove(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<StatusCode> {
    event::run(&db, async |tx| {
        let t = get(tx, sn).await?;
        let profiles: Vec<i64> = mb::Entity::find().select_only().column(mb::Column::ProfileSn).filter(mb::Column::TeamSn.eq(sn)).into_tuple().all(tx).await?;
        Tbl::delete_by_id(sn).exec(tx).await.map_err(in_use)?; // 멤버는 CASCADE
        ap::Entity::delete_many().filter(ap::Column::Sn.is_in(profiles)).exec(tx).await?;
        Ok(((), vec![Ev::new(None, "team", sn, "TeamDeleted", &json!({ "name": t.name }))]))
    }).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// 팀 멤버 목록 (sort → 번호순). 팀이 없으면 404
#[utoipa::path(operation_id = "team_members", get, path = "/teams/{sn}/members", params(("sn" = i64, Path, description = "팀 번호")), responses((status = 200, body = Vec<Member>), (status = "default", body = ErrorBody)))]
async fn members(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Vec<Member>>> {
    get(&db, sn).await?;
    with_work(&db, mb::Entity::find().filter(mb::Column::TeamSn.eq(sn)).order_by_asc(mb::Column::Sort).order_by_asc(mb::Column::Sn).all(&db).await?).await.map(Json)
}

/// 멤버 추가 (MemberCreated). 템플릿이 없으면 422, draft · 보관 · live 버전 없음은 409, 템플릿 없이 role_name도 없으면 422
#[utoipa::path(operation_id = "team_add", post, path = "/teams/{sn}/members", params(("sn" = i64, Path, description = "팀 번호")), request_body = MemberNew, responses((status = 201, body = Member), (status = "default", body = ErrorBody)))]
async fn add(State(db): State<DatabaseConnection>, Sn(sn): Sn, Body(b): Body<MemberNew>) -> Res<(StatusCode, Json<Member>)> {
    let out = event::run(&db, async |tx| {
        get(tx, sn).await?;
        let mut m = mb::ActiveModel {
            team_sn: Set(sn), name: Set(b.name), icon: Set(b.icon), color: Set(b.color),
            first_task_mode: b.first_task_mode.map_or(NotSet, Set), ..Default::default()
        };
        let role = match b.template_sn {
            Some(ts) => {
                let t = tp::Entity::find_by_id(ts).one(tx).await?.ok_or_else(|| Error::invalid("template not found".into()))?;
                if t.status != "active" {
                    return Err(Error::conflict(format!("template is {}", t.status)));
                }
                let rev = ap::Entity::find().filter(ap::Column::TemplateSn.eq(ts)).filter(ap::Column::RevStatus.eq("live")).one(tx).await?
                    .ok_or_else(|| Error::conflict("template has no live version".into()))?;
                m.profile_sn = Set(agent::copy(tx, rev.sn, "member").await?);
                (m.template_sn, m.template_version, m.is_orch) = (Set(Some(ts)), Set(rev.version), Set(t.is_orch));
                // 요청에 없는 표시값은 템플릿 값
                if let Set(None) = m.icon { m.icon = Set(t.icon); }
                if let Set(None) = m.color { m.color = Set(t.color); }
                b.role_name.or(t.role_name).unwrap_or(t.name)
            }
            None => {
                let role = b.role_name.ok_or_else(|| Error::invalid("role_name is required without template_sn".into()))?;
                // 빈 캐릭터: DB 기본값 프로필
                m.profile_sn = Set(ap::ActiveModel { workspace_sn: Set(crate::WORKSPACE), kind: Set("member".into()), ..Default::default() }.insert(tx).await?.sn);
                role
            }
        };
        m.role_name = Set(role);
        let out = Member::from(m.insert(tx).await?);
        let ev = Ev::new(None, "member", out.sn, "MemberCreated", &out);
        Ok((out, vec![ev]))
    }).await?;
    Ok((StatusCode::CREATED, Json(out)))
}

/// 멤버 1건 조회. 없으면 404
#[utoipa::path(operation_id = "team_member", get, path = "/members/{sn}", params(("sn" = i64, Path, description = "멤버 번호")), responses((status = 200, body = Member), (status = "default", body = ErrorBody)))]
async fn member(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Member>> {
    one(&db, sn).await.map(Json)
}

/// 멤버 부분 수정 (MemberUpdated). 모르는 상태는 422, 없으면 404
#[utoipa::path(operation_id = "team_edit", patch, path = "/members/{sn}", params(("sn" = i64, Path, description = "멤버 번호")), request_body = MemberPatch, responses((status = 200, body = Member), (status = "default", body = ErrorBody)))]
async fn edit(State(db): State<DatabaseConnection>, Sn(sn): Sn, Body(b): Body<MemberPatch>) -> Res<Json<Member>> {
    if b.status.as_deref().is_some_and(|s| !STATUS.contains(&s)) {
        return Err(Error::invalid(format!("status must be one of {STATUS:?}")));
    }
    let out = event::run(&db, async |tx| {
        use mb::Column as C;
        let mut q = mb::Entity::update_many().filter(C::Sn.eq(sn)).col_expr(C::UpdateAt, Expr::cust("datetime('now')"));
        if let Some(v) = &b.name { q = q.col_expr(C::Name, v.clone().into()); }
        if let Some(v) = &b.role_name { q = q.col_expr(C::RoleName, v.clone().into()); }
        if let Some(v) = &b.icon { q = q.col_expr(C::Icon, v.clone().into()); }
        if let Some(v) = &b.color { q = q.col_expr(C::Color, v.clone().into()); }
        if let Some(v) = &b.status { q = q.col_expr(C::Status, v.clone().into()); }
        if let Some(v) = &b.first_task_mode { q = q.col_expr(C::FirstTaskMode, v.clone().into()); }
        if let Some(v) = b.sort { q = q.col_expr(C::Sort, v.into()); }
        if q.exec(tx).await?.rows_affected == 0 {
            return Err(Error::not_found());
        }
        let out = one(tx, sn).await?;
        Ok((out, vec![Ev::new(None, "member", sn, "MemberUpdated", &b)]))
    }).await?;
    Ok(Json(out))
}

/// 멤버 삭제 (MemberDeleted). 멤버 프로필도 지운다. Run · 리뷰 등 기록이 있으면 409 (보관 = status archived)
#[utoipa::path(operation_id = "team_delete", delete, path = "/members/{sn}", params(("sn" = i64, Path, description = "멤버 번호")), responses((status = 204, description = "삭제됨"), (status = "default", body = ErrorBody)))]
async fn delete(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<StatusCode> {
    event::run(&db, async |tx| {
        let m = one(tx, sn).await?;
        mb::Entity::delete_by_id(sn).exec(tx).await.map_err(in_use)?;
        ap::Entity::delete_by_id(m.profile_sn).exec(tx).await?;
        Ok(((), vec![Ev::new(None, "member", sn, "MemberDeleted", &json!({ "team_sn": m.team_sn, "name": m.name }))]))
    }).await?;
    Ok(StatusCode::NO_CONTENT)
}
