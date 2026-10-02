//! 워크스페이스 라이브러리 조회: tbl_skill_source · tbl_skill · tbl_mcp + 사용처(프로필 → 멤버 · 템플릿). 쓰기는 스킬 허용/차단만 (워크스페이스 SkillUpdated). 설치 · 업데이트 · 보안 검사는 실행기/Ops
use crate::{entity::{tbl_map_profile_mcp as pm, tbl_map_profile_skill as ps, tbl_mcp as mc, tbl_member as mb, tbl_skill as sk, tbl_skill_source as src,
    tbl_template as tp, tbl_template_revision as tr},
    error::{Body, Error, ErrorBody, Res, Sn}, event::{self, Ev}};
use axum::{Json, extract::State};
use sea_orm::{ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, sea_query::Expr};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

/// skill · mcp 관련 경로 묶음
pub fn routes() -> OpenApiRouter<DatabaseConnection> {
    OpenApiRouter::new()
        .routes(routes!(sources))
        .routes(routes!(skills))
        .routes(routes!(skill, update))
        .routes(routes!(skill_usage))
        .routes(routes!(mcps))
        .routes(routes!(mcp))
        .routes(routes!(mcp_usage))
}

/// 스킬 소스 (API 응답 형태)
#[derive(Serialize, ToSchema)]
pub struct Source {
    sn: i64,
    /// skills_sh | github | local | marketplace | builtin
    kind: String,
    name: String,
    location: Option<String>,
    branch: Option<String>,
    /// connected | error | off
    status: String,
    sort: i64,
    sync_at: Option<String>,
    create_at: String,
}

/// 스킬 (API 응답 형태)
#[derive(Serialize, ToSchema)]
pub struct Skill {
    sn: i64,
    source_sn: Option<i64>,
    name: String,
    description: Option<String>,
    version: Option<String>,
    /// 업데이트 가능한 버전 (없으면 null)
    latest_version: Option<String>,
    sha256: Option<String>,
    token_cost: i64,
    /// pending | passed | failed
    scan_status: String,
    scan_message: Option<String>,
    is_enabled: i64,
    is_blocked: i64,
    install_at: String,
    update_at: String,
}

impl From<sk::Model> for Skill {
    fn from(m: sk::Model) -> Self {
        Self {
            sn: m.sn, source_sn: m.source_sn, name: m.name, description: m.description, version: m.version, latest_version: m.latest_version,
            sha256: m.sha256, token_cost: m.token_cost, scan_status: m.scan_status, scan_message: m.scan_message, is_enabled: m.is_enabled,
            is_blocked: m.is_blocked, install_at: m.install_at, update_at: m.update_at,
        }
    }
}

/// MCP 서버 (API 응답 형태)
#[derive(Serialize, ToSchema)]
pub struct Mcp {
    sn: i64,
    name: String,
    description: Option<String>,
    tool_count: i64,
    token_cost: i64,
    /// installed | available
    install_status: String,
    is_auth_required: i64,
    /// ok | required | none
    auth_status: String,
    /// ok | error | unknown
    health: String,
    /// 실행 설정 JSON 문자열 (command · args · env 이름 — 값은 없다)
    config_json: Option<String>,
    check_at: Option<String>,
    create_at: String,
    update_at: String,
}

impl From<mc::Model> for Mcp {
    fn from(m: mc::Model) -> Self {
        Self {
            sn: m.sn, name: m.name, description: m.description, tool_count: m.tool_count, token_cost: m.token_cost, install_status: m.install_status,
            is_auth_required: m.is_auth_required, auth_status: m.auth_status, health: m.health, config_json: m.config_json, check_at: m.check_at,
            create_at: m.create_at, update_at: m.update_at,
        }
    }
}

/// 사용처 1건: 이 프로필을 가진 멤버 또는 템플릿
#[derive(Serialize, ToSchema)]
pub struct Usage {
    profile_sn: i64,
    /// member | template
    owner: &'static str,
    /// 멤버 번호 또는 템플릿 번호
    owner_sn: i64,
    name: String,
    /// 스킬: 프로필에서 켜짐 여부 · MCP: installed | accessible
    detail: String,
}

/// 스킬 허용/차단 요청 본문. 보낸 필드만 바꾼다
#[derive(Serialize, Deserialize, ToSchema)]
struct SkillPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    is_enabled: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_blocked: Option<i64>,
}

/// 스킬 1건 읽기. 없으면 404
async fn get(db: &impl ConnectionTrait, sn: i64) -> Res<sk::Model> {
    sk::Entity::find_by_id(sn).one(db).await?.ok_or_else(Error::not_found)
}

/// (프로필 번호, detail) 목록 → 멤버 · 템플릿 사용처. 보관된 멤버 · 지난 템플릿 버전은 뺀다
pub(crate) async fn owners(db: &impl ConnectionTrait, links: Vec<(i64, String)>) -> Res<Vec<Usage>> {
    let sns = links.iter().map(|l| l.0).collect::<Vec<_>>();
    let detail = |p: i64| links.iter().find(|l| l.0 == p).map(|l| l.1.clone()).unwrap_or_default();
    let mut out: Vec<Usage> = mb::Entity::find().filter(mb::Column::ProfileSn.is_in(sns.clone())).filter(mb::Column::Status.ne("archived"))
        .order_by_asc(mb::Column::Sn).all(db).await?.into_iter()
        .map(|m| Usage { profile_sn: m.profile_sn, owner: "member", owner_sn: m.sn, detail: detail(m.profile_sn), name: m.name }).collect();
    for r in tr::Entity::find().filter(tr::Column::ProfileSn.is_in(sns)).filter(tr::Column::Status.ne("archived")).order_by_asc(tr::Column::Sn).all(db).await? {
        if let Some(t) = tp::Entity::find_by_id(r.template_sn).one(db).await? {
            out.push(Usage { profile_sn: r.profile_sn, owner: "template", owner_sn: t.sn, detail: detail(r.profile_sn), name: t.name });
        }
    }
    Ok(out)
}

/// 스킬 소스 목록 (우선순위순)
#[utoipa::path(operation_id = "skill_sources", get, path = "/skill-sources", responses((status = 200, body = Vec<Source>), (status = "default", body = ErrorBody)))]
async fn sources(State(db): State<DatabaseConnection>) -> Res<Json<Vec<Source>>> {
    Ok(Json(src::Entity::find().order_by_asc(src::Column::Sort).order_by_asc(src::Column::Sn).all(&db).await?.into_iter().map(|m| Source {
        sn: m.sn, kind: m.kind, name: m.name, location: m.location, branch: m.branch, status: m.status, sort: m.sort, sync_at: m.sync_at, create_at: m.create_at,
    }).collect()))
}

/// 스킬 목록 (이름순)
#[utoipa::path(operation_id = "skill_list", get, path = "/skills", responses((status = 200, body = Vec<Skill>), (status = "default", body = ErrorBody)))]
async fn skills(State(db): State<DatabaseConnection>) -> Res<Json<Vec<Skill>>> {
    Ok(Json(sk::Entity::find().order_by_asc(sk::Column::Name).all(&db).await?.into_iter().map(Skill::from).collect()))
}

/// 스킬 1건. 없으면 404
#[utoipa::path(operation_id = "skill_read", get, path = "/skills/{sn}", params(("sn" = i64, Path, description = "스킬 번호")), responses((status = 200, body = Skill), (status = "default", body = ErrorBody)))]
async fn skill(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Skill>> {
    get(&db, sn).await.map(|m| Json(m.into()))
}

/// 스킬 허용/차단 (SkillUpdated). 값은 0 · 1만 (422). 보안 검사 실패 스킬은 차단을 풀 수 없다 (409). 없으면 404
#[utoipa::path(operation_id = "skill_update", patch, path = "/skills/{sn}", params(("sn" = i64, Path, description = "스킬 번호")), request_body = SkillPatch, responses((status = 200, body = Skill), (status = "default", body = ErrorBody)))]
async fn update(State(db): State<DatabaseConnection>, Sn(sn): Sn, Body(b): Body<SkillPatch>) -> Res<Json<Skill>> {
    if [b.is_enabled, b.is_blocked].iter().flatten().any(|v| !(0..=1).contains(v)) {
        return Err(Error::invalid("is_enabled · is_blocked must be 0 or 1".into()));
    }
    let out = event::run(&db, async |tx| {
        let cur = get(tx, sn).await?;
        if b.is_blocked == Some(0) && cur.scan_status == "failed" {
            return Err(Error::conflict(format!("skill {} failed scan", cur.name)));
        }
        let mut u = sk::Entity::update_many().filter(sk::Column::Sn.eq(sn)).col_expr(sk::Column::UpdateAt, Expr::cust("datetime('now')"));
        if let Some(v) = b.is_enabled { u = u.col_expr(sk::Column::IsEnabled, v.into()); }
        if let Some(v) = b.is_blocked { u = u.col_expr(sk::Column::IsBlocked, v.into()); }
        u.exec(tx).await?;
        // 이벤트 대상 종류에 skill이 없어(스키마 CHECK) 워크스페이스 라이브러리 변경으로 남긴다
        let ev = Ev::new(None, "workspace", crate::WID, "SkillUpdated", &json!({ "skill_sn": sn, "name": cur.name, "patch": &b }));
        Ok((Skill::from(get(tx, sn).await?), vec![ev]))
    }).await?;
    Ok(Json(out))
}

/// 스킬 사용처: 이 스킬을 연결한 프로필의 멤버 · 템플릿. 없으면 404
#[utoipa::path(operation_id = "skill_usage", get, path = "/skills/{sn}/usage", params(("sn" = i64, Path, description = "스킬 번호")), responses((status = 200, body = Vec<Usage>), (status = "default", body = ErrorBody)))]
async fn skill_usage(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Vec<Usage>>> {
    get(&db, sn).await?;
    let links = ps::Entity::find().filter(ps::Column::SkillSn.eq(sn)).all(&db).await?.into_iter()
        .map(|m| (m.profile_sn, if m.is_enabled == 1 { "on" } else { "off" }.to_owned())).collect();
    owners(&db, links).await.map(Json)
}

/// MCP 서버 목록 (이름순)
#[utoipa::path(operation_id = "mcp_list", get, path = "/mcps", responses((status = 200, body = Vec<Mcp>), (status = "default", body = ErrorBody)))]
async fn mcps(State(db): State<DatabaseConnection>) -> Res<Json<Vec<Mcp>>> {
    Ok(Json(mc::Entity::find().order_by_asc(mc::Column::Name).all(&db).await?.into_iter().map(Mcp::from).collect()))
}

/// MCP 서버 1건. 없으면 404
#[utoipa::path(operation_id = "mcp_read", get, path = "/mcps/{sn}", params(("sn" = i64, Path, description = "MCP 번호")), responses((status = 200, body = Mcp), (status = "default", body = ErrorBody)))]
async fn mcp(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Mcp>> {
    mc::Entity::find_by_id(sn).one(&db).await?.map(|m| Json(m.into())).ok_or_else(Error::not_found)
}

/// MCP 사용처: 이 서버를 연결한 프로필의 멤버 · 템플릿 (detail = installed | accessible). 없으면 404
#[utoipa::path(operation_id = "mcp_usage", get, path = "/mcps/{sn}/usage", params(("sn" = i64, Path, description = "MCP 번호")), responses((status = 200, body = Vec<Usage>), (status = "default", body = ErrorBody)))]
async fn mcp_usage(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Vec<Usage>>> {
    mc::Entity::find_by_id(sn).one(&db).await?.ok_or_else(Error::not_found)?;
    let links = pm::Entity::find().filter(pm::Column::McpSn.eq(sn)).all(&db).await?.into_iter().map(|m| (m.profile_sn, m.access_mode)).collect();
    owners(&db, links).await.map(Json)
}
