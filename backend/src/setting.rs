//! 설정: 워크스페이스(tbl_workspace sn=WORKSPACE) 조회 · 수정, 보안 기본값(workspace 프로필), 실행기 목록, Instruction preset 목록 · 버전 조회 (편집은 preset.rs)
use crate::{entity::{tbl_agent_profile as ap, tbl_instruction_preset as ip, tbl_instruction_preset_version as iv, tbl_profile_guard as pg, tbl_profile_rule as pr, tbl_runtime as rt, tbl_workspace as ws},
    error::{Body, Error, ErrorBody, Res, Sn}, event::{self, Ev}};
use axum::{Json, extract::{Query, State}};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, sea_query::Expr};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

/// 허용되는 테마
const THEMES: [&str; 3] = ["light", "dark", "system"];

/// 설정 관련 경로 묶음
pub fn routes() -> OpenApiRouter<DatabaseConnection> {
    OpenApiRouter::new()
        .routes(routes!(workspace, edit))
        .routes(routes!(runtimes))
        .routes(routes!(presets))
        .routes(routes!(versions))
        .routes(routes!(profile, set_profile))
}

/// 허용되는 규칙 동작 · 정책 · 승인자, 가드 시점
const ACTIONS: [&str; 6] = ["pr_create", "dependency_add", "external_message", "env_access", "run_extend", "command"];
const POLICIES: [&str; 3] = ["auto", "approval", "block"];
const APPROVERS: [&str; 2] = ["user", "orch_then_user"];
const STAGES: [&str; 3] = ["tool_use", "tool_result", "output"];

/// 승인 규칙 · 차단 패턴 1줄 (tbl_profile_rule)
#[derive(Serialize, Deserialize, ToSchema)]
struct Rule {
    /// pr_create | dependency_add | external_message | env_access | run_extend | command
    action_code: String,
    title: String,
    /// 명령 패턴 (command일 때)
    pattern: Option<String>,
    description: Option<String>,
    /// auto | approval | block
    policy: String,
    /// user | orch_then_user (approval일 때)
    approver: Option<String>,
    #[serde(default = "one")]
    is_notify: i64,
}

/// 가드 트리거 1줄 (tbl_profile_guard)
#[derive(Serialize, Deserialize, ToSchema)]
struct Guard {
    name: String,
    /// tool_use | tool_result | output
    stage: String,
    pattern: Option<String>,
    #[serde(default = "one")]
    is_enabled: i64,
}

/// 기본값 1
fn one() -> i64 { 1 }

/// 워크스페이스 보안 기본값: kind=workspace 프로필의 Trust · 규칙 · 가드 + 워크스페이스 GitHub 계정 (조회 응답 · 저장 본문 겸용)
#[derive(Serialize, Deserialize, ToSchema)]
struct SecurityProfile {
    /// 1(읽기 전용) ~ 4(자율)
    trust_level: i64,
    rules: Vec<Rule>,
    guards: Vec<Guard>,
    /// bot | personal
    github_mode: String,
    github_account: Option<String>,
    github_repo_scope: Option<String>,
}

/// 워크스페이스 일반 설정 (API 응답 형태)
#[derive(Serialize, ToSchema)]
pub struct Workspace {
    name: String,
    default_repo: Option<String>,
    timezone: String,
    ui_language: String,
    report_language: String,
    commit_language: String,
    date_format: String,
    /// light | dark | system
    theme: String,
    /// 인트로(연결 → 프로젝트 → 기본 팀) 완료 여부
    is_onboarded: i64,
    /// 방해 금지 시작 · 끝 (HH:MM)
    dnd_start: Option<String>,
    dnd_end: Option<String>,
    is_dnd_weekend: i64,
    /// 이 레벨 이상은 방해 금지 무시 (0~4)
    dnd_bypass_level: i64,
    /// 일일 요약 시각 (HH:MM)
    daily_summary_time: Option<String>,
    update_at: String,
}

impl From<ws::Model> for Workspace {
    fn from(m: ws::Model) -> Self {
        Self {
            name: m.name, default_repo: m.default_repo, timezone: m.timezone, ui_language: m.ui_language, report_language: m.report_language,
            commit_language: m.commit_language, date_format: m.date_format, theme: m.theme, is_onboarded: m.is_onboarded, dnd_start: m.dnd_start,
            dnd_end: m.dnd_end, is_dnd_weekend: m.is_dnd_weekend, dnd_bypass_level: m.dnd_bypass_level, daily_summary_time: m.daily_summary_time, update_at: m.update_at,
        }
    }
}

/// 워크스페이스 수정 요청 본문. 보낸 필드만 바꾼다
#[derive(Serialize, Deserialize, ToSchema)]
struct WorkspacePatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default_repo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timezone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ui_language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    report_language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    commit_language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    date_format: Option<String>,
    /// light | dark | system
    #[serde(skip_serializing_if = "Option::is_none")]
    theme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_onboarded: Option<i64>,
    /// HH:MM
    #[serde(skip_serializing_if = "Option::is_none")]
    dnd_start: Option<String>,
    /// HH:MM
    #[serde(skip_serializing_if = "Option::is_none")]
    dnd_end: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_dnd_weekend: Option<i64>,
    /// 0~4
    #[serde(skip_serializing_if = "Option::is_none")]
    dnd_bypass_level: Option<i64>,
    /// HH:MM
    #[serde(skip_serializing_if = "Option::is_none")]
    daily_summary_time: Option<String>,
}

/// HH:MM (00:00 ~ 23:59) 인지
fn hhmm(v: &str) -> bool {
    matches!(v.split_once(':'), Some((h, m)) if h.len() == 2 && m.len() == 2 && h.parse::<u8>().is_ok_and(|h| h < 24) && m.parse::<u8>().is_ok_and(|m| m < 60))
}

/// 실행기 (API 응답 형태). 감지 · 설치 · 로그인 상태 갱신은 실행기 Task가 한다
#[derive(Serialize, ToSchema)]
pub struct Runtime {
    sn: i64,
    /// codex | claude_code | gemini | opencode | cursor | kiro
    code: String,
    name: String,
    version: Option<String>,
    latest_version: Option<String>,
    bin_path: Option<String>,
    /// installed | missing
    install_status: String,
    /// logged_in | login_required | none
    login_status: String,
    sort: i64,
    detect_at: Option<String>,
}

impl From<rt::Model> for Runtime {
    fn from(m: rt::Model) -> Self {
        Self {
            sn: m.sn, code: m.code, name: m.name, version: m.version, latest_version: m.latest_version, bin_path: m.bin_path,
            install_status: m.install_status, login_status: m.login_status, sort: m.sort, detect_at: m.detect_at,
        }
    }
}

/// Instruction preset (API 응답 형태)
#[derive(Serialize, ToSchema)]
pub struct Preset {
    sn: i64,
    project_sn: Option<i64>,
    /// protocol | role | style | rule | report
    kind: String,
    preset_key: String,
    name: String,
    description: Option<String>,
    /// 현재 최신 버전
    version: i64,
    limit_tok: i64,
    is_builtin: i64,
    is_locked: i64,
    is_default: i64,
    copy_from_sn: Option<i64>,
    /// active | archived
    status: String,
    update_at: String,
}

impl From<ip::Model> for Preset {
    fn from(m: ip::Model) -> Self {
        Self {
            sn: m.sn, project_sn: m.project_sn, kind: m.kind, preset_key: m.preset_key, name: m.name, description: m.description, version: m.version,
            limit_tok: m.limit_tok, is_builtin: m.is_builtin, is_locked: m.is_locked, is_default: m.is_default, copy_from_sn: m.copy_from_sn,
            status: m.status, update_at: m.update_at,
        }
    }
}

/// 프리셋 버전 1건 (API 응답 형태)
#[derive(Serialize, ToSchema)]
pub struct PresetVersion {
    version: i64,
    content: String,
    token_count: i64,
    language: String,
    /// builtin | user | import | translated
    source: String,
    change_note: Option<String>,
    create_at: String,
}

impl From<iv::Model> for PresetVersion {
    fn from(m: iv::Model) -> Self {
        Self { version: m.version, content: m.content, token_count: m.token_count, language: m.language, source: m.source, change_note: m.change_note, create_at: m.create_at }
    }
}

/// 워크스페이스 읽기 (시드가 없으면 404)
async fn get(db: &impl ConnectionTrait) -> Res<Workspace> {
    ws::Entity::find_by_id(crate::WORKSPACE).one(db).await?.map(Workspace::from).ok_or_else(Error::not_found)
}

/// 워크스페이스 일반 설정 조회
#[utoipa::path(operation_id = "setting_workspace", get, path = "/workspace", responses((status = 200, body = Workspace), (status = "default", body = ErrorBody)))]
async fn workspace(State(db): State<DatabaseConnection>) -> Res<Json<Workspace>> {
    get(&db).await.map(Json)
}

/// 워크스페이스 부분 수정 (WorkspaceUpdated). 모르는 테마 · HH:MM 아닌 시각 · 레벨 0~4 밖은 422
#[utoipa::path(operation_id = "setting_edit", patch, path = "/workspace", request_body = WorkspacePatch, responses((status = 200, body = Workspace), (status = "default", body = ErrorBody)))]
async fn edit(State(db): State<DatabaseConnection>, Body(b): Body<WorkspacePatch>) -> Res<Json<Workspace>> {
    if b.theme.as_deref().is_some_and(|t| !THEMES.contains(&t)) {
        return Err(Error::invalid(format!("theme must be one of {THEMES:?}")));
    }
    if [&b.dnd_start, &b.dnd_end, &b.daily_summary_time].iter().any(|v| v.as_deref().is_some_and(|v| !hhmm(v)))
        || b.dnd_bypass_level.is_some_and(|v| !(0..=4).contains(&v)) || b.is_dnd_weekend.is_some_and(|v| !(0..=1).contains(&v)) {
        return Err(Error::invalid("times must be HH:MM, dnd_bypass_level 0..=4, is_dnd_weekend 0/1".into()));
    }
    let out = event::run(&db, async |tx| {
        use ws::Column as C;
        let mut u = ws::Entity::update_many().filter(C::Sn.eq(crate::WORKSPACE)).col_expr(C::UpdateAt, Expr::cust("datetime('now')"));
        if let Some(v) = &b.name { u = u.col_expr(C::Name, v.clone().into()); }
        if let Some(v) = &b.default_repo { u = u.col_expr(C::DefaultRepo, v.clone().into()); }
        if let Some(v) = &b.timezone { u = u.col_expr(C::Timezone, v.clone().into()); }
        if let Some(v) = &b.ui_language { u = u.col_expr(C::UiLanguage, v.clone().into()); }
        if let Some(v) = &b.report_language { u = u.col_expr(C::ReportLanguage, v.clone().into()); }
        if let Some(v) = &b.commit_language { u = u.col_expr(C::CommitLanguage, v.clone().into()); }
        if let Some(v) = &b.date_format { u = u.col_expr(C::DateFormat, v.clone().into()); }
        if let Some(v) = &b.theme { u = u.col_expr(C::Theme, v.clone().into()); }
        if let Some(v) = b.is_onboarded { u = u.col_expr(C::IsOnboarded, v.into()); }
        if let Some(v) = &b.dnd_start { u = u.col_expr(C::DndStart, v.clone().into()); }
        if let Some(v) = &b.dnd_end { u = u.col_expr(C::DndEnd, v.clone().into()); }
        if let Some(v) = b.is_dnd_weekend { u = u.col_expr(C::IsDndWeekend, v.into()); }
        if let Some(v) = b.dnd_bypass_level { u = u.col_expr(C::DndBypassLevel, v.into()); }
        if let Some(v) = &b.daily_summary_time { u = u.col_expr(C::DailySummaryTime, v.clone().into()); }
        if u.exec(tx).await?.rows_affected == 0 {
            return Err(Error::not_found());
        }
        let out = get(tx).await?;
        Ok((out, vec![Ev::new(None, "workspace", crate::WORKSPACE, "WorkspaceUpdated", &b)]))
    }).await?;
    Ok(Json(out))
}

/// 실행기 목록 (sort → 번호순). 연결 생성 폼의 선택지
#[utoipa::path(operation_id = "setting_runtimes", get, path = "/runtimes", responses((status = 200, body = Vec<Runtime>), (status = "default", body = ErrorBody)))]
async fn runtimes(State(db): State<DatabaseConnection>) -> Res<Json<Vec<Runtime>>> {
    Ok(Json(rt::Entity::find().order_by_asc(rt::Column::Sort).order_by_asc(rt::Column::Sn).all(&db).await?.into_iter().map(Runtime::from).collect()))
}

/// Instruction preset 목록 (종류 → 번호순). `kind`로 거른다
#[utoipa::path(operation_id = "setting_presets", get, path = "/presets", params(("kind" = Option<String>, Query, description = "이 종류만")), responses((status = 200, body = Vec<Preset>), (status = "default", body = ErrorBody)))]
async fn presets(State(db): State<DatabaseConnection>, Query(q): Query<std::collections::HashMap<String, String>>) -> Res<Json<Vec<Preset>>> {
    let mut f = ip::Entity::find();
    if let Some(k) = q.get("kind") { f = f.filter(ip::Column::Kind.eq(k.as_str())); }
    Ok(Json(f.order_by_asc(ip::Column::Kind).order_by_asc(ip::Column::Sn).all(&db).await?.into_iter().map(Preset::from).collect()))
}

/// 프리셋 버전 목록 (최신순). 프리셋이 없으면 404
#[utoipa::path(operation_id = "setting_versions", get, path = "/presets/{sn}/versions", params(("sn" = i64, Path, description = "프리셋 번호")), responses((status = 200, body = Vec<PresetVersion>), (status = "default", body = ErrorBody)))]
async fn versions(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Vec<PresetVersion>>> {
    ip::Entity::find_by_id(sn).one(&db).await?.ok_or_else(Error::not_found)?;
    Ok(Json(iv::Entity::find().filter(iv::Column::PresetSn.eq(sn)).order_by_desc(iv::Column::Version).all(&db).await?.into_iter().map(PresetVersion::from).collect()))
}

/// 워크스페이스 기본 프로필 (kind = workspace 중 첫 번째)
async fn base(db: &impl ConnectionTrait) -> Res<Option<ap::Model>> {
    Ok(ap::Entity::find().filter(ap::Column::Kind.eq("workspace")).order_by_asc(ap::Column::Sn).one(db).await?)
}

/// 보안 기본값 읽기. 기본 프로필이 아직 없으면 Trust 3 · 규칙 · 가드 없음
async fn security(db: &impl ConnectionTrait) -> Res<SecurityProfile> {
    let w = ws::Entity::find_by_id(crate::WORKSPACE).one(db).await?.ok_or_else(Error::not_found)?;
    let p = base(db).await?;
    let sn = p.as_ref().map_or(0, |p| p.sn);
    let rules = pr::Entity::find().filter(pr::Column::ProfileSn.eq(sn)).order_by_asc(pr::Column::Sort).order_by_asc(pr::Column::Sn).all(db).await?.into_iter()
        .map(|m| Rule { action_code: m.action_code, title: m.title, pattern: m.pattern, description: m.description, policy: m.policy, approver: m.approver, is_notify: m.is_notify }).collect();
    let guards = pg::Entity::find().filter(pg::Column::ProfileSn.eq(sn)).order_by_asc(pg::Column::Sort).order_by_asc(pg::Column::Sn).all(db).await?.into_iter()
        .map(|m| Guard { name: m.name, stage: m.stage, pattern: m.pattern, is_enabled: m.is_enabled }).collect();
    Ok(SecurityProfile {
        trust_level: p.map_or(3, |p| p.trust_level), rules, guards, github_mode: w.github_mode, github_account: w.github_account, github_repo_scope: w.github_repo_scope,
    })
}

/// 워크스페이스 보안 기본값 조회 (Trust · 승인 규칙 · 항상 차단 패턴 · 가드 · GitHub 계정)
#[utoipa::path(operation_id = "setting_profile", get, path = "/workspace/profile", responses((status = 200, body = SecurityProfile), (status = "default", body = ErrorBody)))]
async fn profile(State(db): State<DatabaseConnection>) -> Res<Json<SecurityProfile>> {
    security(&db).await.map(Json)
}

/// 워크스페이스 보안 기본값 저장 (ProfileUpdated · WorkspaceUpdated). 규칙 · 가드는 전체 교체(배열 순서 = 표시 순서), 기본 프로필이 없으면 만든다.
/// Trust 1~4 · 모르는 동작 · 정책 · 승인자 · 시점 · GitHub 방식, command인데 패턴 없음, 빈 이름은 422
#[utoipa::path(operation_id = "setting_set_profile", put, path = "/workspace/profile", request_body = SecurityProfile, responses((status = 200, body = SecurityProfile), (status = "default", body = ErrorBody)))]
async fn set_profile(State(db): State<DatabaseConnection>, Body(b): Body<SecurityProfile>) -> Res<Json<SecurityProfile>> {
    let bad_rule = b.rules.iter().any(|r| !ACTIONS.contains(&r.action_code.as_str()) || !POLICIES.contains(&r.policy.as_str()) || r.title.trim().is_empty()
        || r.approver.as_deref().is_some_and(|a| !APPROVERS.contains(&a)) || (r.action_code == "command" && r.pattern.as_deref().is_none_or(|p| p.trim().is_empty())));
    let bad_guard = b.guards.iter().any(|g| !STAGES.contains(&g.stage.as_str()) || g.name.trim().is_empty());
    if !(1..=4).contains(&b.trust_level) || !["bot", "personal"].contains(&b.github_mode.as_str()) || bad_rule || bad_guard {
        return Err(Error::invalid(format!("trust_level 1..=4, github_mode bot|personal, action in {ACTIONS:?}, policy in {POLICIES:?}, approver in {APPROVERS:?}, stage in {STAGES:?}, command needs pattern")));
    }
    let out = event::run(&db, async |tx| {
        let sn = match base(tx).await? {
            Some(p) => p.sn,
            None => ap::ActiveModel { workspace_sn: Set(crate::WORKSPACE), kind: Set("workspace".into()), ..Default::default() }.insert(tx).await?.sn,
        };
        ap::Entity::update_many().filter(ap::Column::Sn.eq(sn)).col_expr(ap::Column::TrustLevel, b.trust_level.into()).exec(tx).await?;
        pr::Entity::delete_many().filter(pr::Column::ProfileSn.eq(sn)).exec(tx).await?;
        for (i, r) in b.rules.iter().enumerate() {
            pr::ActiveModel {
                profile_sn: Set(sn), action_code: Set(r.action_code.clone()), title: Set(r.title.trim().into()), pattern: Set(r.pattern.clone()), description: Set(r.description.clone()),
                policy: Set(r.policy.clone()), approver: Set(r.approver.clone()), is_notify: Set(r.is_notify), sort: Set(i as i64), ..Default::default()
            }.insert(tx).await?;
        }
        pg::Entity::delete_many().filter(pg::Column::ProfileSn.eq(sn)).exec(tx).await?;
        for (i, g) in b.guards.iter().enumerate() {
            pg::ActiveModel {
                profile_sn: Set(sn), name: Set(g.name.trim().into()), stage: Set(g.stage.clone()), pattern: Set(g.pattern.clone()), is_enabled: Set(g.is_enabled), sort: Set(i as i64), ..Default::default()
            }.insert(tx).await?;
        }
        ws::Entity::update_many().filter(ws::Column::Sn.eq(crate::WORKSPACE)).col_expr(ws::Column::GithubMode, b.github_mode.clone().into())
            .col_expr(ws::Column::GithubAccount, b.github_account.clone().into()).col_expr(ws::Column::GithubRepoScope, b.github_repo_scope.clone().into())
            .col_expr(ws::Column::UpdateAt, Expr::cust("datetime('now')")).exec(tx).await?;
        let out = security(tx).await?;
        let evs = vec![
            Ev::new(None, "profile", sn, "ProfileUpdated", &serde_json::json!({ "trust_level": b.trust_level, "rules": b.rules.len(), "guards": b.guards.len() })),
            Ev::new(None, "workspace", crate::WORKSPACE, "WorkspaceUpdated", &serde_json::json!({ "github_mode": b.github_mode, "github_account": b.github_account, "github_repo_scope": b.github_repo_scope })),
        ];
        Ok((out, evs))
    }).await?;
    Ok(Json(out))
}
