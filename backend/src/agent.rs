//! tbl_agent_profile CRUD + 하위 매핑 조회 + tbl_template 조회. 쓰기는 event::run 경유 (ProfileCreated · ProfileUpdated · ProfileDeleted)
use crate::{entity::{tbl_agent_profile::{self as p, Entity as Tbl}, tbl_map_fallback as fb, tbl_map_profile_mcp as pm, tbl_map_profile_skill as ps, tbl_profile_tool as pt, tbl_template as tp},
    error::{Body, Error, ErrorBody, Res, Sn, in_use}, event::{self, Ev}};
use axum::{Json, extract::{Query, State}, http::StatusCode};
use sea_orm::{ActiveModelTrait, ActiveValue::{NotSet, Set}, ColumnTrait, ConnectionTrait, DatabaseConnection, DatabaseTransaction, DbBackend, EntityTrait,
    IntoActiveModel, QueryFilter, QueryOrder, Statement, sea_query::Expr};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

/// 허용되는 하위 작업 모델 등급 (#67)
const TIERS: [&str; 3] = ["S", "M", "L"];

/// 허용되는 프로필 소유 종류
const KINDS: [&str; 3] = ["workspace", "template", "member"];

/// 프로필을 복사할 때 함께 복사하는 하위 설정 테이블 (tbl_template_revision · tbl_member는 소유자라 제외)
const CHILDREN: [&str; 9] = [
    "tbl_profile_file", "tbl_map_profile_skill", "tbl_map_profile_mcp", "tbl_profile_tool", "tbl_profile_path",
    "tbl_profile_rule", "tbl_profile_guard", "tbl_map_fallback", "tbl_map_profile_preset",
];

/// profile · template 관련 경로 묶음
pub fn routes() -> OpenApiRouter<DatabaseConnection> {
    OpenApiRouter::new()
        .routes(routes!(list, create))
        .routes(routes!(read, update, remove))
        .routes(routes!(caps))
        .routes(routes!(fallbacks, chain))
        .routes(routes!(templates))
        .routes(routes!(template))
}

/// 에이전트 프로필 (API 응답 형태)
#[derive(Serialize, ToSchema)]
pub struct Profile {
    sn: i64,
    wid: i64,
    /// workspace | template | member
    kind: String,
    runtime_sn: Option<i64>,
    connection_sn: Option<i64>,
    model_sn: Option<i64>,
    /// auto | low | medium | high
    effort: String,
    /// resume_task | new_run
    session_mode: String,
    /// use | ignore
    repo_rule_mode: String,
    workdir: Option<String>,
    /// 1(읽기 전용) ~ 4(자율)
    trust_level: i64,
    /// bot | personal · null = 워크스페이스 기본
    github_mode: Option<String>,
    /// allowlist | open | off
    network_mode: String,
    run_token_limit: Option<i64>,
    context_warn_percent: i64,
    run_time_limit_min: Option<i64>,
    auto_retry_max: i64,
    create_at: String,
    update_at: String,
}

impl From<p::Model> for Profile {
    fn from(m: p::Model) -> Self {
        Self {
            sn: m.sn, wid: m.wid, kind: m.kind, runtime_sn: m.runtime_sn, connection_sn: m.connection_sn, model_sn: m.model_sn,
            effort: m.effort, session_mode: m.session_mode, repo_rule_mode: m.repo_rule_mode, workdir: m.workdir,
            trust_level: m.trust_level, github_mode: m.github_mode, network_mode: m.network_mode, run_token_limit: m.run_token_limit,
            context_warn_percent: m.context_warn_percent, run_time_limit_min: m.run_time_limit_min, auto_retry_max: m.auto_retry_max,
            create_at: m.create_at, update_at: m.update_at,
        }
    }
}

/// 생성 요청 본문. 설정값은 DB 기본값으로 시작하고 PATCH로 바꾼다
#[derive(Deserialize, ToSchema)]
struct ProfileNew {
    /// workspace | template | member, 생략하면 workspace
    kind: Option<String>,
}

/// 수정 요청 본문. 보낸 필드만 바꾼다 (이벤트 payload로도 그대로 저장된다)
#[derive(Serialize, Deserialize, ToSchema)]
struct ProfilePatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    runtime_sn: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    connection_sn: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    model_sn: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    effort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    session_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    repo_rule_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    workdir: Option<String>,
    /// 1 ~ 4
    #[serde(skip_serializing_if = "Option::is_none")]
    trust_level: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    github_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    network_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    run_token_limit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    context_warn_percent: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    run_time_limit_min: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    auto_retry_max: Option<i64>,
}

/// 프로필의 스킬 연결
#[derive(Serialize, ToSchema)]
struct SkillLink {
    skill_sn: i64,
    is_enabled: i64,
}

/// 프로필의 MCP 연결
#[derive(Serialize, ToSchema)]
struct McpLink {
    mcp_sn: i64,
    /// installed | accessible
    access_mode: String,
}

/// 프로필의 도구 정책
#[derive(Serialize, ToSchema)]
struct ToolRule {
    /// read | edit | shell | git_push | web_fetch | git_destructive
    tool_code: String,
    scope_text: Option<String>,
    /// allow | allowlist | approval | block
    policy: String,
}

/// 프로필 하위 매핑 (조회 전용)
#[derive(Serialize, ToSchema)]
struct Caps {
    skills: Vec<SkillLink>,
    mcps: Vec<McpLink>,
    tools: Vec<ToolRule>,
}

/// 역할 템플릿 (API 응답 형태)
#[derive(Serialize, ToSchema)]
pub struct Template {
    sn: i64,
    name: String,
    role_name: Option<String>,
    /// dev | verify | design_pm | orch
    category: String,
    icon: Option<String>,
    color: Option<String>,
    description: Option<String>,
    /// 태그 JSON 배열 문자열
    tag_json: Option<String>,
    is_orch: i64,
    /// draft | active | archived
    status: String,
    sort: i64,
    create_at: String,
    update_at: String,
}

impl From<tp::Model> for Template {
    fn from(m: tp::Model) -> Self {
        Self {
            sn: m.sn, name: m.name, role_name: m.role_name, category: m.category, icon: m.icon, color: m.color,
            description: m.description, tag_json: m.tag_json, is_orch: m.is_orch, status: m.status, sort: m.sort,
            create_at: m.create_at, update_at: m.update_at,
        }
    }
}

/// 1건 읽기. 없으면 404
async fn get(db: &impl ConnectionTrait, sn: i64) -> Res<Profile> {
    Tbl::find_by_id(sn).one(db).await?.map(Profile::from).ok_or_else(Error::not_found)
}

/// 프로필과 하위 설정을 `kind` 소유로 복사하고 새 번호를 돌려준다 (템플릿 → 멤버)
pub async fn copy(tx: &DatabaseTransaction, sn: i64, kind: &str) -> Res<i64> {
    let mut m = Tbl::find_by_id(sn).one(tx).await?.ok_or_else(Error::not_found)?.into_active_model();
    (m.sn, m.create_at, m.update_at) = (NotSet, NotSet, NotSet);
    m.kind = Set(kind.into());
    let new = m.insert(tx).await?.sn;
    for t in CHILDREN {
        // sn · profile_sn · 시각을 뺀 컬럼을 그대로 옮긴다 (테이블마다 컬럼이 달라 스키마에서 읽는다)
        let cols = tx.query_all_raw(Statement::from_string(DbBackend::Sqlite, format!(
            "SELECT name FROM pragma_table_info('{t}') WHERE name NOT IN ('sn', 'profile_sn', 'create_at', 'update_at')"
        ))).await?.into_iter().map(|r| r.try_get::<String>("", "name")).collect::<Result<Vec<_>, _>>()?.join(", ");
        tx.execute_raw(Statement::from_sql_and_values(DbBackend::Sqlite,
            format!("INSERT INTO {t} (profile_sn, {cols}) SELECT ?, {cols} FROM {t} WHERE profile_sn = ?"), [new.into(), sn.into()],
        )).await?;
    }
    Ok(new)
}

/// 값 범위 검사: 종류 · Trust 레벨
fn check(kind: Option<&str>, trust: Option<i64>) -> Res<()> {
    if kind.is_some_and(|k| !KINDS.contains(&k)) {
        return Err(Error::invalid(format!("kind must be one of {KINDS:?}")));
    }
    if trust.is_some_and(|v| !(1..=4).contains(&v)) {
        return Err(Error::invalid("trust_level must be 1..=4".into()));
    }
    Ok(())
}

/// 프로필 목록 (번호순). `kind`로 거른다
#[utoipa::path(get, path = "/profiles", params(("kind" = Option<String>, Query, description = "이 종류만")), responses((status = 200, body = Vec<Profile>), (status = "default", body = ErrorBody)))]
async fn list(State(db): State<DatabaseConnection>, Query(q): Query<std::collections::HashMap<String, String>>) -> Res<Json<Vec<Profile>>> {
    let mut f = Tbl::find();
    if let Some(k) = q.get("kind") { f = f.filter(p::Column::Kind.eq(k.as_str())); }
    Ok(Json(f.order_by_asc(p::Column::Sn).all(&db).await?.into_iter().map(Profile::from).collect()))
}

/// 프로필 생성 (ProfileCreated)
#[utoipa::path(post, path = "/profiles", request_body = ProfileNew, responses((status = 201, body = Profile), (status = "default", body = ErrorBody)))]
async fn create(State(db): State<DatabaseConnection>, Body(b): Body<ProfileNew>) -> Res<(StatusCode, Json<Profile>)> {
    let kind = b.kind.unwrap_or_else(|| "workspace".into());
    check(Some(&kind), None)?;
    let out = event::run(&db, async |tx| {
        let out = Profile::from(p::ActiveModel { wid: Set(crate::WID), kind: Set(kind), ..Default::default() }.insert(tx).await?);
        let ev = Ev::new(None, "profile", out.sn, "ProfileCreated", &out);
        Ok((out, vec![ev]))
    }).await?;
    Ok((StatusCode::CREATED, Json(out)))
}

/// 1건 조회. 없으면 404
#[utoipa::path(get, path = "/profiles/{sn}", params(("sn" = i64, Path, description = "프로필 번호")), responses((status = 200, body = Profile), (status = "default", body = ErrorBody)))]
async fn read(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Profile>> {
    get(&db, sn).await.map(Json)
}

/// 부분 수정 (ProfileUpdated). 없으면 404
#[utoipa::path(patch, path = "/profiles/{sn}", params(("sn" = i64, Path, description = "프로필 번호")), request_body = ProfilePatch, responses((status = 200, body = Profile), (status = "default", body = ErrorBody)))]
async fn update(State(db): State<DatabaseConnection>, Sn(sn): Sn, Body(b): Body<ProfilePatch>) -> Res<Json<Profile>> {
    check(None, b.trust_level)?;
    let out = event::run(&db, async |tx| {
        use p::Column as C;
        let mut q = Tbl::update_many().filter(C::Sn.eq(sn)).col_expr(C::UpdateAt, Expr::cust("datetime('now')"));
        if let Some(v) = b.runtime_sn { q = q.col_expr(C::RuntimeSn, v.into()); }
        if let Some(v) = b.connection_sn { q = q.col_expr(C::ConnectionSn, v.into()); }
        if let Some(v) = b.model_sn { q = q.col_expr(C::ModelSn, v.into()); }
        if let Some(v) = &b.effort { q = q.col_expr(C::Effort, v.clone().into()); }
        if let Some(v) = &b.session_mode { q = q.col_expr(C::SessionMode, v.clone().into()); }
        if let Some(v) = &b.repo_rule_mode { q = q.col_expr(C::RepoRuleMode, v.clone().into()); }
        if let Some(v) = &b.workdir { q = q.col_expr(C::Workdir, v.clone().into()); }
        if let Some(v) = b.trust_level { q = q.col_expr(C::TrustLevel, v.into()); }
        if let Some(v) = &b.github_mode { q = q.col_expr(C::GithubMode, v.clone().into()); }
        if let Some(v) = &b.network_mode { q = q.col_expr(C::NetworkMode, v.clone().into()); }
        if let Some(v) = b.run_token_limit { q = q.col_expr(C::RunTokenLimit, v.into()); }
        if let Some(v) = b.context_warn_percent { q = q.col_expr(C::ContextWarnPercent, v.into()); }
        if let Some(v) = b.run_time_limit_min { q = q.col_expr(C::RunTimeLimitMin, v.into()); }
        if let Some(v) = b.auto_retry_max { q = q.col_expr(C::AutoRetryMax, v.into()); }
        if q.exec(tx).await?.rows_affected == 0 {
            return Err(Error::not_found());
        }
        let out = get(tx, sn).await?;
        Ok((out, vec![Ev::new(None, "profile", sn, "ProfileUpdated", &b)]))
    }).await?;
    Ok(Json(out))
}

/// 삭제 (ProfileDeleted). 성공 204, 없으면 404, 멤버가 쓰는 중이면 409 (하위 설정은 CASCADE)
#[utoipa::path(delete, path = "/profiles/{sn}", params(("sn" = i64, Path, description = "프로필 번호")), responses((status = 204, description = "삭제됨"), (status = "default", body = ErrorBody)))]
async fn remove(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<StatusCode> {
    event::run(&db, async |tx| {
        let m = get(tx, sn).await?;
        Tbl::delete_by_id(sn).exec(tx).await.map_err(in_use)?;
        Ok(((), vec![Ev::new(None, "profile", sn, "ProfileDeleted", &json!({ "kind": m.kind }))]))
    }).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// 하위 매핑 조회: 스킬 · MCP · 도구 정책 (편집은 별도 Task). 프로필이 없으면 404
#[utoipa::path(get, path = "/profiles/{sn}/caps", params(("sn" = i64, Path, description = "프로필 번호")), responses((status = 200, body = Caps), (status = "default", body = ErrorBody)))]
async fn caps(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Caps>> {
    get(&db, sn).await?;
    let skills = ps::Entity::find().filter(ps::Column::ProfileSn.eq(sn)).order_by_asc(ps::Column::Sn).all(&db).await?
        .into_iter().map(|m| SkillLink { skill_sn: m.skill_sn, is_enabled: m.is_enabled }).collect();
    let mcps = pm::Entity::find().filter(pm::Column::ProfileSn.eq(sn)).order_by_asc(pm::Column::Sn).all(&db).await?
        .into_iter().map(|m| McpLink { mcp_sn: m.mcp_sn, access_mode: m.access_mode }).collect();
    let tools = pt::Entity::find().filter(pt::Column::ProfileSn.eq(sn)).order_by_asc(pt::Column::Sort).all(&db).await?
        .into_iter().map(|m| ToolRule { tool_code: m.tool_code, scope_text: m.scope_text, policy: m.policy }).collect();
    Ok(Json(Caps { skills, mcps, tools }))
}

/// 폴백 체인 1단계. sort 순으로 시도하고, tier가 있으면 그 등급의 하위 작업만 쓴다 (NULL = 모든 등급)
#[derive(Serialize, Deserialize, ToSchema)]
struct Fallback {
    runtime_sn: i64,
    connection_sn: i64,
    /// NULL = 연결 기본 모델
    model_sn: Option<i64>,
    /// 다음 단계로 넘어가는 조건 (예: 429)
    switch_rule: Option<String>,
    max_level: Option<i64>,
    /// S | M | L | NULL
    tier: Option<String>,
}

/// 폴백 체인 조회 (sort 순). 프로필이 없으면 404
#[utoipa::path(get, path = "/profiles/{sn}/fallbacks", params(("sn" = i64, Path, description = "프로필 번호")), responses((status = 200, body = Vec<Fallback>), (status = "default", body = ErrorBody)))]
async fn fallbacks(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Vec<Fallback>>> {
    get(&db, sn).await?;
    Ok(Json(fb::Entity::find().filter(fb::Column::ProfileSn.eq(sn)).order_by_asc(fb::Column::Sort).all(&db).await?.into_iter()
        .map(|m| Fallback { runtime_sn: m.runtime_sn, connection_sn: m.connection_sn, model_sn: m.model_sn, switch_rule: m.switch_rule, max_level: m.max_level, tier: m.tier })
        .collect()))
}

/// 폴백 체인 전체 교체 (ProfileUpdated). 배열 순서 = sort. 모르는 tier · 없는 실행기 · 연결 · 모델은 422, 프로필이 없으면 404
#[utoipa::path(put, path = "/profiles/{sn}/fallbacks", params(("sn" = i64, Path, description = "프로필 번호")), request_body = Vec<Fallback>, responses((status = 200, body = Vec<Fallback>), (status = "default", body = ErrorBody)))]
async fn chain(State(db): State<DatabaseConnection>, Sn(sn): Sn, Body(b): Body<Vec<Fallback>>) -> Res<Json<Vec<Fallback>>> {
    if b.iter().any(|f| f.tier.as_deref().is_some_and(|t| !TIERS.contains(&t))) {
        return Err(Error::invalid(format!("tier must be one of {TIERS:?} or null")));
    }
    event::run(&db, async |tx| {
        get(tx, sn).await?;
        fb::Entity::delete_many().filter(fb::Column::ProfileSn.eq(sn)).exec(tx).await?;
        for (i, f) in b.iter().enumerate() {
            fb::ActiveModel {
                profile_sn: Set(sn), runtime_sn: Set(f.runtime_sn), connection_sn: Set(f.connection_sn), model_sn: Set(f.model_sn),
                sort: Set(i as i64 + 1), switch_rule: Set(f.switch_rule.clone()), max_level: Set(f.max_level), tier: Set(f.tier.clone()), ..Default::default()
            }.insert(tx).await?;
        }
        Ok(((), vec![Ev::new(None, "profile", sn, "ProfileUpdated", &json!({ "fallbacks": &b }))]))
    }).await?;
    Ok(Json(b))
}

/// 템플릿 목록 (보관 제외 · sort → 번호순)
#[utoipa::path(get, path = "/templates", responses((status = 200, body = Vec<Template>), (status = "default", body = ErrorBody)))]
async fn templates(State(db): State<DatabaseConnection>) -> Res<Json<Vec<Template>>> {
    Ok(Json(tp::Entity::find().filter(tp::Column::Status.ne("archived")).order_by_asc(tp::Column::Sort).order_by_asc(tp::Column::Sn)
        .all(&db).await?.into_iter().map(Template::from).collect()))
}

/// 템플릿 1건 조회. 없으면 404
#[utoipa::path(get, path = "/templates/{sn}", params(("sn" = i64, Path, description = "템플릿 번호")), responses((status = 200, body = Template), (status = "default", body = ErrorBody)))]
async fn template(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Template>> {
    tp::Entity::find_by_id(sn).one(&db).await?.map(|m| Json(m.into())).ok_or_else(Error::not_found)
}
