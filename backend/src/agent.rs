//! tbl_agent_profile CRUD + 하위 매핑 조회 + tbl_template 조회. 쓰기는 event::run 경유 (ProfileCreated · ProfileUpdated · ProfileDeleted)
use crate::{entity::{tbl_agent_profile::{self as p, Entity as Tbl}, tbl_connection as cn, tbl_map_profile_mcp as pm, tbl_map_profile_skill as ps, tbl_model as md, tbl_runtime as rt, tbl_skill as sk, tbl_template as tp},
    error::{Body, Error, ErrorBody, Res, Sn, in_use}, event::{self, Ev}};
use axum::{Json, extract::{Query, State}, http::StatusCode};
use sea_orm::{ActiveModelTrait, ActiveValue::{NotSet, Set}, ColumnTrait, ConnectionTrait, DatabaseConnection, DatabaseTransaction, DbBackend, EntityTrait,
    IntoActiveModel, QueryFilter, QueryOrder, Statement, sea_query::Expr};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

/// 허용되는 하위 작업 모델 등급 (#67)
pub(crate) const TIERS: [&str; 3] = ["S", "M", "L"];

/// 기본 차단 명령 (하위 Run · #67). 프로필 rule_json의 command 규칙(pattern = 명령 글자)이 같은 명령이면 그 정책이 이긴다.
/// 명령 글자 → 실행기 인자 맵핑은 runner::perms
pub(crate) const CMDS: [&str; 3] = ["git stash", "git checkout", "git reset"];

/// 허용되는 프로필 소유 종류
const KINDS: [&str; 3] = ["workspace", "template", "member"];

/// 프로필을 복사할 때 함께 복사하는 하위 설정 테이블 (tbl_member는 소유자라 제외 · 작은 목록은 *_json 컬럼이라 프로필 행과 함께 복사된다)
const CHILDREN: [&str; 4] = ["tbl_profile_file", "tbl_map_profile_skill", "tbl_map_profile_mcp", "tbl_map_profile_preset"];

/// 허용되는 도구 · 도구 정책 · 파일 범위 종류 · 규칙 동작 · 규칙 정책 · 승인자 · 가드 시점
const TOOLS: [&str; 6] = ["read", "edit", "shell", "git_push", "web_fetch", "git_destructive"];
const TOOL_POLICIES: [&str; 4] = ["allow", "allowlist", "approval", "block"];
const PATHS: [&str; 2] = ["include", "exclude"];
const ACTIONS: [&str; 6] = ["pr_create", "dependency_add", "external_message", "env_access", "run_extend", "command"];
const POLICIES: [&str; 3] = ["auto", "approval", "block"];
const APPROVERS: [&str; 2] = ["user", "orch_then_user"];
const STAGES: [&str; 3] = ["tool_use", "tool_result", "output"];

/// profile · template 관련 경로 묶음
pub fn routes() -> OpenApiRouter<DatabaseConnection> {
    OpenApiRouter::new()
        .routes(routes!(list, create))
        .routes(routes!(read, update, remove))
        .routes(routes!(set_skills))
        .routes(routes!(templates))
        .routes(routes!(template))
}

/// 에이전트 프로필 (API 응답 형태)
#[derive(Serialize, ToSchema)]
pub struct Profile {
    sn: i64,
    workspace_sn: i64,
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
    /// 파일 범위 (path_json)
    paths: Vec<PathRule>,
    /// CLI 기본 도구 정책 (tool_json)
    tools: Vec<ToolRule>,
    /// 승인 규칙 · 항상 차단 (rule_json)
    rules: Vec<Rule>,
    /// 가드 트리거 (guard_json)
    guards: Vec<Guard>,
    /// 폴백 체인 (fallback_json · 위에서부터 시도)
    fallbacks: Vec<Fallback>,
    /// 명령 사용 여부 (기본 목록 + rules의 command 규칙 · 읽기 전용)
    commands: Vec<Cmd>,
    /// 켠 스킬 (tbl_map_profile_skill · 편집은 PUT /profiles/{sn}/skills)
    skills: Vec<SkillLink>,
    /// 쓰는 MCP (tbl_map_profile_mcp)
    mcps: Vec<McpLink>,
    create_at: String,
    update_at: String,
}

/// JSON 배열 컬럼 → 목록 (비었거나 깨졌으면 빈 목록)
fn arr<T: serde::de::DeserializeOwned>(j: &Option<String>) -> Vec<T> {
    j.as_deref().and_then(|s| serde_json::from_str(s).ok()).unwrap_or_default()
}

/// 프로필의 파일 범위
pub(crate) fn paths_of(m: &p::Model) -> Vec<PathRule> { arr(&m.path_json) }
/// 프로필의 도구 정책
pub(crate) fn tools_of(m: &p::Model) -> Vec<ToolRule> { arr(&m.tool_json) }
/// 프로필의 승인 규칙
pub(crate) fn rules_of(m: &p::Model) -> Vec<Rule> { arr(&m.rule_json) }
/// 프로필의 가드
pub(crate) fn guards_of(m: &p::Model) -> Vec<Guard> { arr(&m.guard_json) }
/// 프로필의 폴백 체인 (위에서부터)
pub(crate) fn fallbacks_of(m: &p::Model) -> Vec<Fallback> { arr(&m.fallback_json) }

impl From<p::Model> for Profile {
    fn from(m: p::Model) -> Self {
        Self {
            paths: paths_of(&m), tools: tools_of(&m), guards: guards_of(&m), fallbacks: fallbacks_of(&m), commands: effective(&rules_of(&m)), rules: rules_of(&m),
            skills: Vec::new(), mcps: Vec::new(),
            sn: m.sn, workspace_sn: m.workspace_sn, kind: m.kind, runtime_sn: m.runtime_sn, connection_sn: m.connection_sn, model_sn: m.model_sn,
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
    /// 파일 범위 전체 교체
    #[serde(skip_serializing_if = "Option::is_none")]
    paths: Option<Vec<PathRule>>,
    /// 도구 정책 전체 교체
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<ToolRule>>,
    /// 승인 규칙 전체 교체
    #[serde(skip_serializing_if = "Option::is_none")]
    rules: Option<Vec<Rule>>,
    /// 가드 전체 교체
    #[serde(skip_serializing_if = "Option::is_none")]
    guards: Option<Vec<Guard>>,
    /// 폴백 체인 전체 교체 (배열 순서 = 시도 순서)
    #[serde(skip_serializing_if = "Option::is_none")]
    fallbacks: Option<Vec<Fallback>>,
}

/// 프로필의 스킬 연결
#[derive(Serialize, Deserialize, ToSchema)]
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

/// 파일 범위 1줄 (path_json)
#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct PathRule {
    /// include | exclude (제외가 포함보다 우선)
    pub kind: String,
    /// glob 패턴 (예: frontend/**, **/.env*)
    pub pattern: String,
}

/// 도구 정책 1줄 (tool_json). 행이 없는 도구는 허용
#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct ToolRule {
    /// read | edit | shell | git_push | web_fetch | git_destructive
    pub tool_code: String,
    /// 허용 범위 설명 (예: pnpm lint · test · dev)
    pub scope_text: Option<String>,
    /// allow | allowlist | approval | block
    pub policy: String,
}

/// 승인 규칙 · 차단 패턴 1줄 (rule_json)
#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct Rule {
    /// pr_create | dependency_add | external_message | env_access | run_extend | command
    pub action_code: String,
    pub title: String,
    /// 명령 패턴 (command일 때)
    pub pattern: Option<String>,
    pub description: Option<String>,
    /// auto | approval | block
    pub policy: String,
    /// user | orch_then_user (approval일 때)
    pub approver: Option<String>,
    #[serde(default = "one")]
    pub is_notify: i64,
}

/// 가드 트리거 1줄 (guard_json)
#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct Guard {
    pub name: String,
    /// tool_use | tool_result | output
    pub stage: String,
    pub pattern: Option<String>,
    #[serde(default = "one")]
    pub is_enabled: i64,
}

/// 기본값 1
fn one() -> i64 { 1 }

/// 폴백 체인 1단계 (fallback_json). 배열 순서대로 시도하고, tier가 있으면 그 등급의 하위 작업만 쓴다 (NULL = 모든 등급)
#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct Fallback {
    pub runtime_sn: i64,
    pub connection_sn: i64,
    /// NULL = 연결 기본 모델
    pub model_sn: Option<i64>,
    /// 다음 단계로 넘어가는 조건 (예: 429)
    pub switch_rule: Option<String>,
    pub max_level: Option<i64>,
    /// S | M | L | NULL
    pub tier: Option<String>,
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

/// 프로필 행 + 켠 스킬 · MCP 연결 (응답 모양)
async fn full(db: &impl ConnectionTrait, m: p::Model) -> Res<Profile> {
    let sn = m.sn;
    let mut out = Profile::from(m);
    out.skills = ps::Entity::find().filter(ps::Column::ProfileSn.eq(sn)).order_by_asc(ps::Column::Sn).all(db).await?
        .into_iter().map(|m| SkillLink { skill_sn: m.skill_sn, is_enabled: m.is_enabled }).collect();
    out.mcps = pm::Entity::find().filter(pm::Column::ProfileSn.eq(sn)).order_by_asc(pm::Column::Sn).all(db).await?
        .into_iter().map(|m| McpLink { mcp_sn: m.mcp_sn, access_mode: m.access_mode }).collect();
    Ok(out)
}

/// 1건 읽기. 없으면 404
async fn get(db: &impl ConnectionTrait, sn: i64) -> Res<Profile> {
    full(db, Tbl::find_by_id(sn).one(db).await?.ok_or_else(Error::not_found)?).await
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

/// 규칙 · 가드 · 도구 · 파일 범위 · 폴백 값 검사 (저장하는 쪽이 부른다). 틀리면 422
pub(crate) fn check_rules(rules: &[Rule]) -> Res<()> {
    let bad = rules.iter().any(|r| !ACTIONS.contains(&r.action_code.as_str()) || !POLICIES.contains(&r.policy.as_str()) || r.title.trim().is_empty()
        || r.approver.as_deref().is_some_and(|a| !APPROVERS.contains(&a)) || (r.action_code == "command" && r.pattern.as_deref().is_none_or(|p| p.trim().is_empty())));
    if bad {
        return Err(Error::invalid(format!("rule: action in {ACTIONS:?}, policy in {POLICIES:?}, approver in {APPROVERS:?}, title not empty, command needs pattern")));
    }
    Ok(())
}

/// 가드 값 검사 (시점 · 이름). 틀리면 422
pub(crate) fn check_guards(guards: &[Guard]) -> Res<()> {
    if guards.iter().any(|g| !STAGES.contains(&g.stage.as_str()) || g.name.trim().is_empty()) {
        return Err(Error::invalid(format!("guard: stage in {STAGES:?}, name not empty")));
    }
    Ok(())
}

/// PATCH 본문의 목록 값 검사 (도구 · 파일 범위 · 규칙 · 가드 · 폴백 등급). 틀리면 422
fn check_lists(b: &ProfilePatch) -> Res<()> {
    if let Some(ts) = &b.tools {
        let dup = ts.iter().enumerate().any(|(i, t)| ts[..i].iter().any(|o| o.tool_code == t.tool_code));
        if dup || ts.iter().any(|t| !TOOLS.contains(&t.tool_code.as_str()) || !TOOL_POLICIES.contains(&t.policy.as_str())) {
            return Err(Error::invalid(format!("tool: tool_code in {TOOLS:?} (once each), policy in {TOOL_POLICIES:?}")));
        }
    }
    if b.paths.as_ref().is_some_and(|ps| ps.iter().any(|p| !PATHS.contains(&p.kind.as_str()) || p.pattern.trim().is_empty())) {
        return Err(Error::invalid(format!("path: kind in {PATHS:?}, pattern not empty")));
    }
    if let Some(rs) = &b.rules { check_rules(rs)?; }
    if let Some(gs) = &b.guards { check_guards(gs)?; }
    if b.fallbacks.as_ref().is_some_and(|fs| fs.iter().any(|f| f.tier.as_deref().is_some_and(|t| !TIERS.contains(&t)))) {
        return Err(Error::invalid(format!("tier must be one of {TIERS:?} or null")));
    }
    Ok(())
}

/// 폴백 단계가 가리키는 실행기 · 연결 · 모델이 있는지 (없으면 422 · 예전 FK 검사)
async fn check_refs(db: &impl ConnectionTrait, fs: &[Fallback]) -> Res<()> {
    for f in fs {
        let ok = rt::Entity::find_by_id(f.runtime_sn).one(db).await?.is_some() && cn::Entity::find_by_id(f.connection_sn).one(db).await?.is_some()
            && match f.model_sn { Some(m) => md::Entity::find_by_id(m).one(db).await?.is_some(), None => true };
        if !ok {
            return Err(Error::invalid(format!("fallback: runtime {} · connection {} · model not found", f.runtime_sn, f.connection_sn)));
        }
    }
    Ok(())
}

/// 이 연결을 폴백 체인에 쓰는 프로필 수 (연결 삭제 409용)
pub(crate) async fn fallback_uses(db: &impl ConnectionTrait, connection_sn: i64) -> Res<i64> {
    let row = db.query_one_raw(Statement::from_sql_and_values(DbBackend::Sqlite,
        "SELECT COUNT(*) FROM tbl_agent_profile p, json_each(p.fallback_json) j WHERE json_extract(j.value, '$.connection_sn') = ?", [connection_sn.into()])).await?;
    Ok(row.map(|r| r.try_get_by_index::<i64>(0)).transpose()?.unwrap_or(0))
}

/// 프로필 목록 (번호순). `kind`로 거른다
#[utoipa::path(operation_id = "agent_list", get, path = "/profiles", params(("kind" = Option<String>, Query, description = "이 종류만")), responses((status = 200, body = Vec<Profile>), (status = "default", body = ErrorBody)))]
async fn list(State(db): State<DatabaseConnection>, Query(q): Query<std::collections::HashMap<String, String>>) -> Res<Json<Vec<Profile>>> {
    let mut f = Tbl::find();
    if let Some(k) = q.get("kind") { f = f.filter(p::Column::Kind.eq(k.as_str())); }
    let mut out = Vec::new();
    for m in f.order_by_asc(p::Column::Sn).all(&db).await? {
        out.push(full(&db, m).await?);
    }
    Ok(Json(out))
}

/// 프로필 생성 (ProfileCreated)
#[utoipa::path(operation_id = "agent_create", post, path = "/profiles", request_body = ProfileNew, responses((status = 201, body = Profile), (status = "default", body = ErrorBody)))]
async fn create(State(db): State<DatabaseConnection>, Body(b): Body<ProfileNew>) -> Res<(StatusCode, Json<Profile>)> {
    let kind = b.kind.unwrap_or_else(|| "workspace".into());
    check(Some(&kind), None)?;
    let out = event::run(&db, async |tx| {
        let out = full(tx, p::ActiveModel { workspace_sn: Set(crate::WORKSPACE), kind: Set(kind), ..Default::default() }.insert(tx).await?).await?;
        let ev = Ev::new(None, "profile", out.sn, "ProfileCreated", &out);
        Ok((out, vec![ev]))
    }).await?;
    Ok((StatusCode::CREATED, Json(out)))
}

/// 1건 조회. 없으면 404
#[utoipa::path(operation_id = "agent_read", get, path = "/profiles/{sn}", params(("sn" = i64, Path, description = "프로필 번호")), responses((status = 200, body = Profile), (status = "default", body = ErrorBody)))]
async fn read(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Profile>> {
    get(&db, sn).await.map(Json)
}

/// 부분 수정 (ProfileUpdated). 없으면 404
#[utoipa::path(operation_id = "agent_update", patch, path = "/profiles/{sn}", params(("sn" = i64, Path, description = "프로필 번호")), request_body = ProfilePatch, responses((status = 200, body = Profile), (status = "default", body = ErrorBody)))]
async fn update(State(db): State<DatabaseConnection>, Sn(sn): Sn, Body(b): Body<ProfilePatch>) -> Res<Json<Profile>> {
    check(None, b.trust_level)?;
    check_lists(&b)?;
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
        if let Some(v) = &b.paths { q = q.col_expr(C::PathJson, json!(v).to_string().into()); }
        if let Some(v) = &b.tools { q = q.col_expr(C::ToolJson, json!(v).to_string().into()); }
        if let Some(v) = &b.rules { q = q.col_expr(C::RuleJson, json!(v).to_string().into()); }
        if let Some(v) = &b.guards { q = q.col_expr(C::GuardJson, json!(v).to_string().into()); }
        if let Some(v) = &b.fallbacks { q = q.col_expr(C::FallbackJson, json!(v).to_string().into()); }
        if q.exec(tx).await?.rows_affected == 0 {
            return Err(Error::not_found());
        }
        if let Some(fs) = &b.fallbacks { check_refs(tx, fs).await?; }
        let out = get(tx, sn).await?;
        Ok((out, vec![Ev::new(None, "profile", sn, "ProfileUpdated", &b)]))
    }).await?;
    Ok(Json(out))
}

/// 삭제 (ProfileDeleted). 성공 204, 없으면 404, 멤버가 쓰는 중이면 409 (하위 설정은 CASCADE)
#[utoipa::path(operation_id = "agent_remove", delete, path = "/profiles/{sn}", params(("sn" = i64, Path, description = "프로필 번호")), responses((status = 204, description = "삭제됨"), (status = "default", body = ErrorBody)))]
async fn remove(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<StatusCode> {
    event::run(&db, async |tx| {
        let m = get(tx, sn).await?;
        Tbl::delete_by_id(sn).exec(tx).await.map_err(in_use)?;
        Ok(((), vec![Ev::new(None, "profile", sn, "ProfileDeleted", &json!({ "kind": m.kind }))]))
    }).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// 스킬 연결 전체 교체 (ProfileUpdated). 차단 · 보안 검사 미통과 스킬은 422, 없는 스킬은 422(invalid_ref), 프로필이 없으면 404
#[utoipa::path(operation_id = "agent_set_skills", put, path = "/profiles/{sn}/skills", params(("sn" = i64, Path, description = "프로필 번호")), request_body = Vec<SkillLink>, responses((status = 200, body = Vec<SkillLink>), (status = "default", body = ErrorBody)))]
async fn set_skills(State(db): State<DatabaseConnection>, Sn(sn): Sn, Body(b): Body<Vec<SkillLink>>) -> Res<Json<Vec<SkillLink>>> {
    event::run(&db, async |tx| {
        get(tx, sn).await?;
        let bad = sk::Entity::find().filter(sk::Column::Sn.is_in(b.iter().map(|l| l.skill_sn)))
            .filter(sea_orm::Condition::any().add(sk::Column::IsBlocked.eq(1)).add(sk::Column::ScanStatus.ne("passed"))).one(tx).await?;
        if let Some(m) = bad {
            return Err(Error::invalid(format!("skill {} is blocked or not scanned", m.name)));
        }
        ps::Entity::delete_many().filter(ps::Column::ProfileSn.eq(sn)).exec(tx).await?;
        for l in &b {
            ps::ActiveModel { profile_sn: Set(sn), skill_sn: Set(l.skill_sn), is_enabled: Set(l.is_enabled), ..Default::default() }.insert(tx).await?;
        }
        Ok(((), vec![Ev::new(None, "profile", sn, "ProfileUpdated", &json!({ "skills": &b }))]))
    }).await?;
    Ok(Json(b))
}

/// 명령 사용 여부 한 줄 (응답 전용 · 편집은 rules의 command 규칙)
#[derive(Serialize, Deserialize, ToSchema, Clone, Debug, PartialEq)]
pub struct Cmd {
    /// 명령 글자 (예: git push). 이 글자로 시작하는 명령 전체에 적용
    pub cmd: String,
    /// true = 사용 · false = 차단
    pub on: bool,
    /// 서버 기본 목록(CMDS)에 있는 명령 (응답 전용 · 지우면 기본값으로 돌아간다)
    #[serde(default)]
    pub builtin: bool,
}

/// 기본 목록 + 프로필 command 규칙 → 실제 적용 목록 (기본 순서 → 추가 명령)
pub(crate) fn effective(rules: &[Rule]) -> Vec<Cmd> {
    let mine: Vec<(&str, bool)> = rules.iter().filter(|r| r.action_code == "command").filter_map(|r| Some((r.pattern.as_deref()?, r.policy == "auto"))).collect();
    let on = |c: &str| mine.iter().find(|(p, _)| *p == c).map(|(_, o)| *o);
    CMDS.iter().map(|c| Cmd { cmd: (*c).into(), on: on(c).unwrap_or(false), builtin: true })
        .chain(mine.iter().filter(|(p, _)| !CMDS.contains(p)).map(|(p, o)| Cmd { cmd: (*p).into(), on: *o, builtin: false }))
        .collect()
}

/// 템플릿 목록 (보관 제외 · sort → 번호순)
#[utoipa::path(operation_id = "agent_templates", get, path = "/templates", responses((status = 200, body = Vec<Template>), (status = "default", body = ErrorBody)))]
async fn templates(State(db): State<DatabaseConnection>) -> Res<Json<Vec<Template>>> {
    Ok(Json(tp::Entity::find().filter(tp::Column::Status.ne("archived")).order_by_asc(tp::Column::Sort).order_by_asc(tp::Column::Sn)
        .all(&db).await?.into_iter().map(Template::from).collect()))
}

/// 템플릿 1건 조회. 없으면 404
#[utoipa::path(operation_id = "agent_template", get, path = "/templates/{sn}", params(("sn" = i64, Path, description = "템플릿 번호")), responses((status = 200, body = Template), (status = "default", body = ErrorBody)))]
async fn template(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Template>> {
    tp::Entity::find_by_id(sn).one(&db).await?.map(|m| Json(m.into())).ok_or_else(Error::not_found)
}
