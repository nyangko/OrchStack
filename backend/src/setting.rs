//! 설정: 워크스페이스(tbl_workspace sn=WID) 조회 · 수정, 실행기 목록, Instruction preset 목록 · 버전 조회(읽기 전용)
use crate::{entity::{tbl_instruction_preset as ip, tbl_instruction_preset_version as iv, tbl_runtime as rt, tbl_workspace as ws},
    error::{Body, Error, ErrorBody, Res, Sn}, event::{self, Ev}};
use axum::{Json, extract::{Query, State}};
use sea_orm::{ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, sea_query::Expr};
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
    ws::Entity::find_by_id(crate::WID).one(db).await?.map(Workspace::from).ok_or_else(Error::not_found)
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
        let mut u = ws::Entity::update_many().filter(C::Sn.eq(crate::WID)).col_expr(C::UpdateAt, Expr::cust("datetime('now')"));
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
        Ok((out, vec![Ev::new(None, "workspace", crate::WID, "WorkspaceUpdated", &b)]))
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
