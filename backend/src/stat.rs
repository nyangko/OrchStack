//! 화면 집계: 팀 통계(Teams KPI · 작업량) · 팀 한도 요약 · 워크스페이스 월 비용. 원천은 tbl_log_token · tbl_task · tbl_run · tbl_connection_quota — 저장하지 않고 응답에서 합친다
use crate::{connection::Quota, entity::{tbl_agent_profile as ap, tbl_connection as c, tbl_connection_quota as q, tbl_map_fallback as fb, tbl_member as mb, tbl_team as tm},
    error::{Error, ErrorBody, Res, Sn}};
use axum::{Json, extract::{Query, State}};
use sea_orm::{ColumnTrait, ConnectionTrait, DatabaseConnection, DbBackend, EntityTrait, QueryFilter, QueryOrder, Statement, Value};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};

/// stat 관련 경로 묶음
pub fn routes() -> OpenApiRouter<DatabaseConnection> {
    OpenApiRouter::new()
        .routes(routes!(team))
        .routes(routes!(quota))
        .routes(routes!(cost))
}

/// 멤버 1명의 작업량
#[derive(Serialize, ToSchema)]
struct Load {
    member_sn: i64,
    name: String,
    /// running | waiting | idle | paused
    status: String,
    /// 담당 중인 열린 태스크 (done · cancelled 아님)
    open_task_count: i64,
    /// 진행 중 Run (리드 Run만)
    active_run_count: i64,
    /// 오늘(UTC) 토큰
    today_token: i64,
}

/// 팀 통계 (Teams KPI)
#[derive(Serialize, ToSchema)]
struct TeamStat {
    member_count: i64,
    open_task_count: i64,
    /// 최근 7일(UTC) 완료 태스크
    done_week_count: i64,
    /// 오늘(UTC) 팀 토큰 합 · 하루 예산
    today_token: i64,
    daily_token_budget: Option<i64>,
    /// 최근 7일 완료 태스크의 평균 사이클(분, 시작 → 완료). 없으면 null
    avg_cycle_minute: Option<i64>,
    members: Vec<Load>,
}

/// 연결 1개의 한도 요약
#[derive(Serialize, ToSchema)]
struct ConnQuota {
    connection_sn: i64,
    name: String,
    /// subscription | plan | api_key | gateway | local
    kind: String,
    status: String,
    /// 한도 중 가장 적게 남은 비율 (한도 행이 없으면 null)
    min_remain_percent: Option<i64>,
    quotas: Vec<Quota>,
}

/// 월 비용 쿼리
#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct Month {
    /// YYYY-MM (기본: 이번 달 UTC)
    month: Option<String>,
}

/// 연결 종류별 비용 1줄
#[derive(Serialize, ToSchema)]
struct CostLine {
    /// subscription(구독 · plan 포함) | api_key | gateway | local | unknown(연결 기록 없음)
    kind: String,
    cost_usd_micro: i64,
    token: i64,
}

/// 워크스페이스 월 비용 (Settings 이번 달 비용)
#[derive(Serialize, ToSchema)]
struct Cost {
    month: String,
    items: Vec<CostLine>,
    total_usd_micro: i64,
}

/// 숫자 1개를 돌려주는 SQL (NULL이면 None)
async fn one(db: &DatabaseConnection, sql: &str, vals: Vec<Value>) -> Res<Option<i64>> {
    Ok(db.query_one_raw(Statement::from_sql_and_values(DbBackend::Sqlite, sql, vals)).await?.and_then(|r| r.try_get_by_index::<Option<i64>>(0).ok().flatten()))
}

/// 팀 멤버(보관 제외) 목록. 팀이 없으면 404
async fn members(db: &DatabaseConnection, sn: i64) -> Res<(tm::Model, Vec<mb::Model>)> {
    let team = tm::Entity::find_by_id(sn).one(db).await?.ok_or_else(Error::not_found)?;
    let ms = mb::Entity::find().filter(mb::Column::TeamSn.eq(sn)).filter(mb::Column::Status.ne("archived")).order_by_asc(mb::Column::Sort).order_by_asc(mb::Column::Sn).all(db).await?;
    Ok((team, ms))
}

/// 팀 통계: 멤버 · 열린 태스크 · 최근 7일 완료 · 오늘 토큰 · 평균 사이클 · 멤버별 작업량 (날짜는 UTC). 팀이 없으면 404
#[utoipa::path(operation_id = "stat_team", get, path = "/teams/{sn}/stats", params(("sn" = i64, Path, description = "팀 번호")), responses((status = 200, body = TeamStat), (status = "default", body = ErrorBody)))]
async fn team(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<TeamStat>> {
    let (team, ms) = members(&db, sn).await?;
    let mut loads = Vec::new();
    for m in ms {
        let v = || vec![Value::from(m.sn)];
        loads.push(Load {
            open_task_count: one(&db, "SELECT COUNT(*) FROM tbl_task WHERE member_sn = ? AND status NOT IN ('done','cancelled')", v()).await?.unwrap_or(0),
            active_run_count: one(&db, "SELECT COUNT(*) FROM tbl_run WHERE member_sn = ? AND parent_run_sn IS NULL AND status IN ('queued','starting','running','waiting','review')", v()).await?.unwrap_or(0),
            today_token: one(&db, "SELECT SUM(t.token_input + t.token_cache_read + t.token_cache_write + t.token_output) FROM tbl_log_token t JOIN tbl_run r ON r.sn = t.run_sn \
                WHERE r.member_sn = ? AND date(t.create_at) = date('now')", v()).await?.unwrap_or(0),
            member_sn: m.sn, name: m.name, status: m.status,
        });
    }
    let team_q = |sql: &str| sql.replace("{team}", "SELECT sn FROM tbl_member WHERE team_sn = ?");
    let v = || vec![Value::from(sn)];
    Ok(Json(TeamStat {
        member_count: loads.len() as i64,
        open_task_count: loads.iter().map(|l| l.open_task_count).sum(),
        done_week_count: one(&db, &team_q("SELECT COUNT(*) FROM tbl_task WHERE member_sn IN ({team}) AND status = 'done' AND done_at >= date('now','-6 day')"), v()).await?.unwrap_or(0),
        today_token: loads.iter().map(|l| l.today_token).sum(),
        daily_token_budget: team.daily_token_budget,
        avg_cycle_minute: one(&db, &team_q("SELECT CAST(AVG((julianday(done_at) - julianday(start_at)) * 1440) AS INTEGER) FROM tbl_task \
            WHERE member_sn IN ({team}) AND status = 'done' AND start_at IS NOT NULL AND done_at >= date('now','-6 day')"), v()).await?,
        members: loads,
    }))
}

/// 팀 한도 요약: 팀 멤버 프로필의 연결 + 폴백 체인 연결별 한도 (번호순). 팀이 없으면 404
#[utoipa::path(operation_id = "stat_quota", get, path = "/teams/{sn}/quota", params(("sn" = i64, Path, description = "팀 번호")), responses((status = 200, body = Vec<ConnQuota>), (status = "default", body = ErrorBody)))]
async fn quota(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Vec<ConnQuota>>> {
    let (_, ms) = members(&db, sn).await?;
    let profiles: Vec<i64> = ms.iter().map(|m| m.profile_sn).collect();
    let mut conns: Vec<i64> = ap::Entity::find().filter(ap::Column::Sn.is_in(profiles.clone())).all(&db).await?.into_iter().filter_map(|p| p.connection_sn).collect();
    conns.extend(fb::Entity::find().filter(fb::Column::ProfileSn.is_in(profiles)).all(&db).await?.into_iter().map(|f| f.connection_sn));
    let mut out = Vec::new();
    for cm in c::Entity::find().filter(c::Column::Sn.is_in(conns)).order_by_asc(c::Column::Sn).all(&db).await? {
        let rows = q::Entity::find().filter(q::Column::ConnectionSn.eq(cm.sn)).order_by_asc(q::Column::Sn).all(&db).await?;
        out.push(ConnQuota {
            min_remain_percent: rows.iter().filter_map(|r| r.remain_percent).min(),
            connection_sn: cm.sn, name: cm.name, kind: cm.kind, status: cm.status, quotas: rows.into_iter().map(Quota::from).collect(),
        });
    }
    Ok(Json(out))
}

/// 워크스페이스 월 비용: tbl_log_token 비용 · 토큰을 연결 종류별로 합친다 (UTC 월). 구독 정액 요금은 스키마에 없어 사용 기록 비용만. YYYY-MM이 아니면 422
#[utoipa::path(operation_id = "stat_cost", get, path = "/workspace/cost", params(Month), responses((status = 200, body = Cost), (status = "default", body = ErrorBody)))]
async fn cost(State(db): State<DatabaseConnection>, Query(q): Query<Month>) -> Res<Json<Cost>> {
    let month = match q.month {
        Some(m) if m.len() == 7 && m.as_bytes()[4] == b'-' && m[..4].parse::<u16>().is_ok() && m[5..].parse::<u8>().is_ok_and(|x| (1..=12).contains(&x)) => m,
        Some(_) => return Err(Error::invalid("month must be YYYY-MM".into())),
        None => db.query_one_raw(Statement::from_string(DbBackend::Sqlite, "SELECT strftime('%Y-%m','now')")).await?
            .and_then(|r| r.try_get_by_index::<String>(0).ok()).unwrap_or_default(),
    };
    let rows = db.query_all_raw(Statement::from_sql_and_values(DbBackend::Sqlite,
        "SELECT CASE WHEN c.kind IN ('subscription','plan') THEN 'subscription' ELSE COALESCE(c.kind, 'unknown') END k, \
         SUM(t.cost_usd_micro) cost, SUM(t.token_input + t.token_cache_read + t.token_cache_write + t.token_output) tok \
         FROM tbl_log_token t LEFT JOIN tbl_connection c ON c.sn = t.connection_sn WHERE strftime('%Y-%m', t.create_at) = ? GROUP BY k ORDER BY k",
        vec![Value::from(month.clone())])).await?;
    let items: Vec<CostLine> = rows.into_iter().map(|r| CostLine {
        kind: r.try_get("", "k").unwrap_or_default(), cost_usd_micro: r.try_get("", "cost").unwrap_or(0), token: r.try_get("", "tok").unwrap_or(0),
    }).collect();
    Ok(Json(Cost { total_usd_micro: items.iter().map(|i| i.cost_usd_micro).sum(), month, items }))
}
