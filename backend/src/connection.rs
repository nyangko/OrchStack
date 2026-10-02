//! tbl_connection CRUD + 한도(tbl_connection_quota) 조회. 쓰기는 event::run 경유 (Connection* 이벤트). key_ref는 응답에 내보내지 않는다
use crate::{entity::{tbl_connection::{self as c, Entity as Tbl}, tbl_connection_quota as q},
    error::{Body, Error, ErrorBody, Res, Sn, in_use}, event::{self, Ev}};
use axum::{Json, extract::State, http::StatusCode};
use sea_orm::{ActiveModelTrait, ActiveValue::{NotSet, Set}, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, sea_query::Expr};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

/// 허용되는 연결 종류
const KINDS: [&str; 5] = ["subscription", "plan", "api_key", "gateway", "local"];
/// 허용되는 구독 로그인 방식
const LOGINS: [&str; 3] = ["browser", "device_code", "terminal"];
/// 허용되는 사용 범위
const SCOPES: [&str; 3] = ["workspace", "team", "me"];

/// connection 관련 경로 묶음
pub fn routes() -> OpenApiRouter<DatabaseConnection> {
    OpenApiRouter::new()
        .routes(routes!(list, create))
        .routes(routes!(read, update, remove))
        .routes(routes!(quotas))
}

/// 연결 (API 응답 형태). 키체인 항목 이름(key_ref)은 빼고 끝 4자리(key_hint)만 준다
#[derive(Serialize, ToSchema)]
pub struct Connection {
    sn: i64,
    /// subscription | plan | api_key | gateway | local
    kind: String,
    provider_code: String,
    provider_name: String,
    name: String,
    account_label: Option<String>,
    plan_name: Option<String>,
    runtime_sn: Option<i64>,
    /// browser | device_code | terminal
    login_method: Option<String>,
    base_url: Option<String>,
    key_hint: Option<String>,
    /// connected | checking | login_required | expired | error | available
    status: String,
    status_message: Option<String>,
    monthly_budget_usd_micro: Option<i64>,
    budget_warn_percent: i64,
    is_budget_exclude: i64,
    /// workspace | team | me
    scope: String,
    report_language: Option<String>,
    commit_language: Option<String>,
    latency_ms: Option<i64>,
    cache_hit_percent: Option<i64>,
    test_at: Option<String>,
    sync_at: Option<String>,
    create_at: String,
    update_at: String,
}

impl From<c::Model> for Connection {
    fn from(m: c::Model) -> Self {
        Self {
            sn: m.sn, kind: m.kind, provider_code: m.provider_code, provider_name: m.provider_name, name: m.name,
            account_label: m.account_label, plan_name: m.plan_name, runtime_sn: m.runtime_sn, login_method: m.login_method,
            base_url: m.base_url, key_hint: m.key_hint, status: m.status, status_message: m.status_message,
            monthly_budget_usd_micro: m.monthly_budget_usd_micro, budget_warn_percent: m.budget_warn_percent, is_budget_exclude: m.is_budget_exclude,
            scope: m.scope, report_language: m.report_language, commit_language: m.commit_language, latency_ms: m.latency_ms,
            cache_hit_percent: m.cache_hit_percent, test_at: m.test_at, sync_at: m.sync_at, create_at: m.create_at, update_at: m.update_at,
        }
    }
}

/// 연결 한도 1건 (API 응답 형태). 갱신은 실행기가 한다
#[derive(Serialize, ToSchema)]
pub struct Quota {
    sn: i64,
    /// minute | 5h | day | week | month
    period: String,
    /// percent | usd | request | token
    unit: String,
    used_value: f64,
    limit_value: Option<f64>,
    remain_percent: Option<i64>,
    reset_at: Option<String>,
    update_at: String,
}

impl From<q::Model> for Quota {
    fn from(m: q::Model) -> Self {
        Self { sn: m.sn, period: m.period, unit: m.unit, used_value: m.used_value, limit_value: m.limit_value, remain_percent: m.remain_percent, reset_at: m.reset_at, update_at: m.update_at }
    }
}

/// 연결 생성 요청 본문. 상태는 available로 시작하고 실행기가 확인해 바꾼다
#[derive(Deserialize, ToSchema)]
struct ConnectionNew {
    /// subscription | plan | api_key | gateway | local
    kind: String,
    provider_code: String,
    provider_name: String,
    name: String,
    account_label: Option<String>,
    plan_name: Option<String>,
    runtime_sn: Option<i64>,
    login_method: Option<String>,
    base_url: Option<String>,
    /// 키체인 항목 이름 (키 원문이 아니다 · 응답에는 나오지 않는다)
    key_ref: Option<String>,
    key_hint: Option<String>,
    monthly_budget_usd_micro: Option<i64>,
    budget_warn_percent: Option<i64>,
    is_budget_exclude: Option<i64>,
    scope: Option<String>,
    report_language: Option<String>,
    commit_language: Option<String>,
}

/// 연결 수정 요청 본문. 보낸 필드만 바꾼다
#[derive(Serialize, Deserialize, ToSchema)]
struct ConnectionPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    account_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    plan_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    runtime_sn: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    login_method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    base_url: Option<String>,
    // 이벤트 payload(이 본문 직렬화)에 키 위치를 남기지 않는다
    #[serde(skip_serializing)]
    key_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    key_hint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    monthly_budget_usd_micro: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    budget_warn_percent: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_budget_exclude: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    report_language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    commit_language: Option<String>,
}

/// 종류 · 로그인 방식 · 범위 · 경고 기준(0~100) 검사. 틀리면 422
fn check(kind: Option<&str>, login: Option<&str>, scope: Option<&str>, warn: Option<i64>) -> Res<()> {
    if kind.is_some_and(|k| !KINDS.contains(&k)) || login.is_some_and(|l| !LOGINS.contains(&l)) || scope.is_some_and(|s| !SCOPES.contains(&s))
        || warn.is_some_and(|w| !(0..=100).contains(&w)) {
        return Err(Error::invalid(format!("kind in {KINDS:?}, login_method in {LOGINS:?}, scope in {SCOPES:?}, budget_warn_percent 0..=100")));
    }
    Ok(())
}

/// 연결 1건 읽기. 없으면 404
async fn get(db: &impl ConnectionTrait, sn: i64) -> Res<Connection> {
    Tbl::find_by_id(sn).one(db).await?.map(Connection::from).ok_or_else(Error::not_found)
}

/// 연결 목록 (번호순)
#[utoipa::path(operation_id = "connection_list", get, path = "/connections", responses((status = 200, body = Vec<Connection>), (status = "default", body = ErrorBody)))]
async fn list(State(db): State<DatabaseConnection>) -> Res<Json<Vec<Connection>>> {
    Ok(Json(Tbl::find().order_by_asc(c::Column::Sn).all(&db).await?.into_iter().map(Connection::from).collect()))
}

/// 연결 생성 (ConnectionCreated). 모르는 종류 · 로그인 방식 · 범위는 422, 없는 실행기는 422(invalid_ref)
#[utoipa::path(operation_id = "connection_create", post, path = "/connections", request_body = ConnectionNew, responses((status = 201, body = Connection), (status = "default", body = ErrorBody)))]
async fn create(State(db): State<DatabaseConnection>, Body(b): Body<ConnectionNew>) -> Res<(StatusCode, Json<Connection>)> {
    check(Some(&b.kind), b.login_method.as_deref(), b.scope.as_deref(), b.budget_warn_percent)?;
    let out = event::run(&db, async |tx| {
        let m = c::ActiveModel {
            wid: Set(crate::WID), uid: Set(Some(crate::UID)), kind: Set(b.kind), provider_code: Set(b.provider_code), provider_name: Set(b.provider_name),
            name: Set(b.name), account_label: Set(b.account_label), plan_name: Set(b.plan_name), runtime_sn: Set(b.runtime_sn),
            login_method: Set(b.login_method), base_url: Set(b.base_url), key_ref: Set(b.key_ref), key_hint: Set(b.key_hint),
            monthly_budget_usd_micro: Set(b.monthly_budget_usd_micro), budget_warn_percent: b.budget_warn_percent.map_or(NotSet, Set),
            is_budget_exclude: b.is_budget_exclude.map_or(NotSet, Set), scope: b.scope.map_or(NotSet, Set),
            report_language: Set(b.report_language), commit_language: Set(b.commit_language), ..Default::default()
        };
        let out = Connection::from(m.insert(tx).await?);
        let ev = Ev::new(None, "connection", out.sn, "ConnectionCreated", &out);
        Ok((out, vec![ev]))
    }).await?;
    Ok((StatusCode::CREATED, Json(out)))
}

/// 연결 1건 조회. 없으면 404
#[utoipa::path(operation_id = "connection_read", get, path = "/connections/{sn}", params(("sn" = i64, Path, description = "연결 번호")), responses((status = 200, body = Connection), (status = "default", body = ErrorBody)))]
async fn read(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Connection>> {
    get(&db, sn).await.map(Json)
}

/// 연결 부분 수정 (ConnectionUpdated). 없으면 404, 모르는 로그인 방식 · 범위는 422
#[utoipa::path(operation_id = "connection_update", patch, path = "/connections/{sn}", params(("sn" = i64, Path, description = "연결 번호")), request_body = ConnectionPatch, responses((status = 200, body = Connection), (status = "default", body = ErrorBody)))]
async fn update(State(db): State<DatabaseConnection>, Sn(sn): Sn, Body(b): Body<ConnectionPatch>) -> Res<Json<Connection>> {
    check(None, b.login_method.as_deref(), b.scope.as_deref(), b.budget_warn_percent)?;
    let out = event::run(&db, async |tx| {
        use c::Column as C;
        let mut u = Tbl::update_many().filter(C::Sn.eq(sn)).col_expr(C::UpdateAt, Expr::cust("datetime('now')"));
        if let Some(v) = &b.name { u = u.col_expr(C::Name, v.clone().into()); }
        if let Some(v) = &b.provider_name { u = u.col_expr(C::ProviderName, v.clone().into()); }
        if let Some(v) = &b.account_label { u = u.col_expr(C::AccountLabel, v.clone().into()); }
        if let Some(v) = &b.plan_name { u = u.col_expr(C::PlanName, v.clone().into()); }
        if let Some(v) = b.runtime_sn { u = u.col_expr(C::RuntimeSn, v.into()); }
        if let Some(v) = &b.login_method { u = u.col_expr(C::LoginMethod, v.clone().into()); }
        if let Some(v) = &b.base_url { u = u.col_expr(C::BaseUrl, v.clone().into()); }
        if let Some(v) = &b.key_ref { u = u.col_expr(C::KeyRef, v.clone().into()); }
        if let Some(v) = &b.key_hint { u = u.col_expr(C::KeyHint, v.clone().into()); }
        if let Some(v) = b.monthly_budget_usd_micro { u = u.col_expr(C::MonthlyBudgetUsdMicro, v.into()); }
        if let Some(v) = b.budget_warn_percent { u = u.col_expr(C::BudgetWarnPercent, v.into()); }
        if let Some(v) = b.is_budget_exclude { u = u.col_expr(C::IsBudgetExclude, v.into()); }
        if let Some(v) = &b.scope { u = u.col_expr(C::Scope, v.clone().into()); }
        if let Some(v) = &b.report_language { u = u.col_expr(C::ReportLanguage, v.clone().into()); }
        if let Some(v) = &b.commit_language { u = u.col_expr(C::CommitLanguage, v.clone().into()); }
        if u.exec(tx).await?.rows_affected == 0 {
            return Err(Error::not_found());
        }
        let out = get(tx, sn).await?;
        Ok((out, vec![Ev::new(None, "connection", sn, "ConnectionUpdated", &b)]))
    }).await?;
    Ok(Json(out))
}

/// 연결 삭제 (ConnectionDeleted). 한도 · 팀 매핑은 함께 지워진다. 폴백 체인에 쓰이고 있으면 409
#[utoipa::path(operation_id = "connection_remove", delete, path = "/connections/{sn}", params(("sn" = i64, Path, description = "연결 번호")), responses((status = 204, description = "삭제됨"), (status = "default", body = ErrorBody)))]
async fn remove(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<StatusCode> {
    event::run(&db, async |tx| {
        let m = get(tx, sn).await?;
        Tbl::delete_by_id(sn).exec(tx).await.map_err(in_use)?;
        Ok(((), vec![Ev::new(None, "connection", sn, "ConnectionDeleted", &json!({ "name": m.name }))]))
    }).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// 연결 한도 목록 (번호순). 연결이 없으면 404
#[utoipa::path(operation_id = "connection_quotas", get, path = "/connections/{sn}/quotas", params(("sn" = i64, Path, description = "연결 번호")), responses((status = 200, body = Vec<Quota>), (status = "default", body = ErrorBody)))]
async fn quotas(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Vec<Quota>>> {
    get(&db, sn).await?;
    Ok(Json(q::Entity::find().filter(q::Column::ConnectionSn.eq(sn)).order_by_asc(q::Column::Sn).all(&db).await?.into_iter().map(Quota::from).collect()))
}
