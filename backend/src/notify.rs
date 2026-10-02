//! 알림(tbl_notification) 목록 · 읽음 + 이벤트 → 알림 projection, 알림 규칙(워크스페이스 · 연결의 notify_json) · 채널 테스트(workspace.channel_json), 감사 로그(tbl_log_event 중 키 · 정책 · 연결 · 업데이트 · 차단 이벤트) 조회
use crate::{entity::{tbl_ask as ak, tbl_connection as cn, tbl_log_event as e, tbl_notification as n, tbl_run as r, tbl_task as t, tbl_workspace as ws},
    error::{Body, Error, ErrorBody, Res}, event::{self, Ev}};
use axum::{Json, extract::{Query, State}};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection, DatabaseTransaction, DbBackend, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Statement, sea_query::Expr};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};

/// 알림 이벤트 코드 (notify_json · tbl_notification CHECK와 같다)
const CODES: [&str; 14] = ["decision_request", "approval_request", "orch_decided", "run_failed", "guard_stop", "context_warn", "quota_low",
    "budget_80", "budget_over", "connection_error", "fallback_used", "pr", "task_done", "daily_summary"];
/// 알림 채널
const CHANNELS: [&str; 4] = ["app", "desktop", "telegram", "email"];
/// 탭 `run`에 들어가는 이벤트
const RUN_TAB: [&str; 6] = ["run_failed", "guard_stop", "context_warn", "orch_decided", "pr", "task_done"];
/// 탭 `quota`에 들어가는 이벤트
const QUOTA_TAB: [&str; 5] = ["quota_low", "budget_80", "budget_over", "connection_error", "fallback_used"];
/// 감사 로그 종류
const AUDITS: [&str; 6] = ["KEY", "POLICY", "CONNECTION", "INSTALL", "UPDATE", "BLOCK"];
/// 감사 로그가 되는 이벤트 → 종류 (별도 감사 테이블 없이 tbl_log_event를 조회한다 · 차단 이벤트는 차단을 만드는 쪽이 ActionBlocked로 남긴다)
const AUDIT: [(&str, &str); 7] = [
    ("ConnectionCreated", "KEY"), ("ConnectionDeleted", "KEY"), ("ConnectionUpdated", "CONNECTION"),
    ("ProfileUpdated", "POLICY"), ("OrchPolicyUpdated", "POLICY"), ("SkillUpdated", "UPDATE"), ("ActionBlocked", "BLOCK"),
];

/// notify 관련 경로 묶음
pub fn routes() -> OpenApiRouter<DatabaseConnection> {
    OpenApiRouter::new()
        .routes(routes!(list))
        .routes(routes!(read))
        .routes(routes!(rules, set_rules))
        .routes(routes!(test))
        .routes(routes!(audit))
}

/// 이벤트 1건 → 알림 0~1건. event::run 트랜잭션 안에서 불린다 (이벤트와 알림이 함께 커밋 · 롤백).
/// 워크스페이스 기본 규칙에서 그 이벤트의 앱 알림을 끈 경우는 만들지 않는다
pub async fn project(tx: &DatabaseTransaction, ev: &e::Model) -> Res<()> {
    // 알린 쪽 · 멤버는 이벤트 행의 행위자에서 가져온다 (알림에는 user가 없어 사용자가 낸 이벤트는 system)
    // (이벤트 코드, 제목, 설명, 바로가기 종류, 바로가기 번호, 확인 필요)
    let sn = ev.aggregate_sn;
    let (code, title, body, ref_type, ref_sn, action) = match ev.event_type.as_str() {
        "DecisionRequested" => {
            let Some(m) = ak::Entity::find_by_id(sn).one(tx).await? else { return Ok(()) };
            ("decision_request", m.title, None, "ask", sn, 1)
        }
        "ApprovalRequested" => {
            let Some(m) = ak::Entity::find_by_id(sn).one(tx).await? else { return Ok(()) };
            let detail = crate::ask::detail(&m);
            ("approval_request", m.title, detail, "ask", sn, 1)
        }
        "RunFailed" => {
            let Some(m) = r::Entity::find_by_id(ev.aggregate_sn).one(tx).await? else { return Ok(()) };
            let title = t::Entity::find_by_id(m.task_sn).one(tx).await?.map_or_else(String::new, |x| x.title);
            ("run_failed", title, m.fail_detail, "run", sn, 0)
        }
        // 태스크 done (TaskMoved · RunApproved)
        "TaskMoved" | "RunApproved" => {
            let Some(ts) = crate::orch_rule::done_task(ev) else { return Ok(()) };
            let Some(m) = t::Entity::find_by_id(ts).one(tx).await? else { return Ok(()) };
            ("task_done", m.title, None, "task", ts, 0)
        }
        // 가드 정지 (OrchProposed kind=guard_stop)
        "OrchProposed" => {
            let Some(m) = ak::Entity::find_by_id(sn).one(tx).await?.filter(|x| x.action.as_deref() == Some("guard_stop")) else { return Ok(()) };
            ("guard_stop", m.title, m.reason, "ask", sn, 1)
        }
        // 폴백 연결로 바꿈 (FallbackUsed): 제목 = 새 연결 이름
        "FallbackUsed" => {
            let p: serde_json::Value = serde_json::from_str(&ev.payload_json).unwrap_or_default();
            let Some(m) = cn::Entity::find_by_id(p["to"].as_i64().unwrap_or_default()).one(tx).await? else { return Ok(()) };
            ("fallback_used", m.name, None, "run", sn, 0)
        }
        _ => return Ok(()),
    };
    let w = ws::Entity::find_by_id(crate::WORKSPACE).one(tx).await?;
    let off = w.is_some_and(|w| cells(&w.notify_json).iter().any(|c| c.event_code == code && c.channel_kind == "app" && c.is_enabled == 0));
    if off {
        return Ok(());
    }
    n::ActiveModel {
        workspace_sn: Set(crate::WORKSPACE), user_sn: Set(crate::USER), event_code: Set(code.into()), actor_type: Set(if ev.actor_type == "user" { "system".into() } else { ev.actor_type.clone() }), member_sn: Set(ev.member_sn),
        title: Set(title), body: Set(body), ref_type: Set(Some(ref_type.into())), ref_sn: Set(Some(ref_sn)), is_action: Set(action),
        event_sn: Set(Some(ev.sn)), ..Default::default()
    }.insert(tx).await?;
    Ok(())
}

/// 알림 (API 응답 형태). project_sn은 원본 이벤트에서 — 프론트가 `/p/{project}?decide=`로 이동
#[derive(Serialize, ToSchema)]
pub struct Notice {
    sn: i64,
    event_code: String,
    /// orch | member | system
    actor_type: String,
    member_sn: Option<i64>,
    title: String,
    body: Option<String>,
    /// ask | run | task | connection
    ref_type: Option<String>,
    ref_sn: Option<i64>,
    project_sn: Option<i64>,
    is_action: i64,
    is_read: i64,
    /// 묶음: need(확인 필요 · 안 읽은 확인 알림) | today | yesterday | week | older (UTC 날짜 기준)
    group: String,
    create_at: String,
    read_at: Option<String>,
}

/// 알림 목록 쿼리
#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct ListQuery {
    /// all(기본) | need | run | quota
    tab: Option<String>,
    /// 이 sn보다 새 알림만
    after: Option<i64>,
}

/// 읽음 요청 본문. sns가 없으면 전체
#[derive(Deserialize, ToSchema)]
struct ReadBody {
    sns: Option<Vec<i64>>,
}

/// 읽음 처리 결과
#[derive(Serialize, ToSchema)]
struct ReadOut {
    /// 새로 읽음 처리된 수
    updated: u64,
}

/// 알림 규칙 1줄: 이벤트 × 채널 켬/끔. connection_sn이 없으면 워크스페이스 기본
#[derive(Serialize, Deserialize, ToSchema)]
struct Rule {
    connection_sn: Option<i64>,
    event_code: String,
    /// app | desktop | telegram | email
    channel_kind: String,
    is_enabled: i64,
}

/// 알림 규칙 1칸 (notify_json 원소 · 워크스페이스 기본 또는 연결별)
#[derive(Serialize, Deserialize, Clone)]
struct Cell {
    event_code: String,
    channel_kind: String,
    is_enabled: i64,
}

/// notify_json → 규칙 칸 목록
fn cells(j: &Option<String>) -> Vec<Cell> {
    j.as_deref().and_then(|s| serde_json::from_str(s).ok()).unwrap_or_default()
}

/// 알림 채널 (channel_json 원소). 봇 토큰 위치(key_ref)는 응답에 내보내지 않는다
#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct Channel {
    /// app | desktop | telegram | email
    pub kind: String,
    /// on | allowed | connected | off
    pub status: String,
    pub target: Option<String>,
    pub target_label: Option<String>,
    pub update_at: Option<String>,
    /// 봇 토큰의 키체인 항목 이름 (저장 전용 · 응답에는 없다)
    #[serde(skip_serializing, default)]
    #[allow(dead_code)] // 저장 전용 — 봇 발송(Ops Task)이 읽는다
    key_ref: Option<String>,
}

/// 워크스페이스의 알림 채널 (channel_json)
pub fn channels_of(w: &ws::Model) -> Vec<Channel> {
    w.channel_json.as_deref().and_then(|s| serde_json::from_str(s).ok()).unwrap_or_default()
}

/// 채널 테스트 요청 본문
#[derive(Deserialize, ToSchema)]
struct TestBody {
    /// app | desktop | telegram | email
    kind: String,
}

/// 채널 테스트 결과. 실제 외부 발송(Telegram · 이메일)은 Ops Task — 지금은 채널 준비 여부만
#[derive(Serialize, ToSchema)]
struct TestOut {
    kind: String,
    /// 채널이 설정되어 보낼 수 있는 상태
    ready: bool,
    /// 외부로 실제 보냈는지 (지금은 항상 false)
    delivered: bool,
}

/// 감사 로그 쿼리
#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct AuditQuery {
    /// KEY | POLICY | CONNECTION | INSTALL | UPDATE | BLOCK
    kind: Option<String>,
    /// 최대 건수 (기본 100 · 최대 1000)
    limit: Option<u64>,
}

/// 감사 로그 1건 (API 응답 형태)
#[derive(Serialize, ToSchema)]
struct Audit {
    sn: i64,
    /// user | orch | member | system
    actor_type: String,
    user_sn: Option<i64>,
    member_sn: Option<i64>,
    run_sn: Option<i64>,
    kind: String,
    title: String,
    detail: Option<String>,
    create_at: String,
}

/// 알림 목록 (최신순 · 최대 200). 탭 · after로 거르고 묶음은 서버가 정한다
#[utoipa::path(operation_id = "notify_list", get, path = "/notifications", params(ListQuery), responses((status = 200, body = Vec<Notice>), (status = "default", body = ErrorBody)))]
async fn list(State(db): State<DatabaseConnection>, Query(q): Query<ListQuery>) -> Res<Json<Vec<Notice>>> {
    let mut f = n::Entity::find().filter(n::Column::UserSn.eq(crate::USER));
    f = match q.tab.as_deref().unwrap_or("all") {
        "all" => f,
        "need" => f.filter(n::Column::IsAction.eq(1)),
        "run" => f.filter(n::Column::EventCode.is_in(RUN_TAB)),
        "quota" => f.filter(n::Column::EventCode.is_in(QUOTA_TAB)),
        _ => return Err(Error::invalid("tab must be all | need | run | quota".into())),
    };
    if let Some(a) = q.after { f = f.filter(n::Column::Sn.gt(a)); }
    // 묶음 경계: 오늘 · 어제 · 6일 전 (SQLite UTC 날짜 문자열 — YYYY-MM-DD라 문자열 비교가 날짜 비교)
    let row = db.query_one_raw(Statement::from_string(DbBackend::Sqlite, "SELECT date('now') a, date('now','-1 day') b, date('now','-6 day') c")).await?;
    let day = |c: &str| row.as_ref().and_then(|r| r.try_get::<String>("", c).ok()).unwrap_or_default();
    let (today, yesterday, week) = (day("a"), day("b"), day("c"));
    let rows = f.order_by_desc(n::Column::Sn).limit(200).all(&db).await?;
    let evs = e::Entity::find().filter(e::Column::Sn.is_in(rows.iter().filter_map(|m| m.event_sn))).all(&db).await?;
    Ok(Json(rows.into_iter().map(|m| {
        let d = m.create_at.get(..10).unwrap_or_default();
        let group = match () {
            _ if m.is_action == 1 && m.is_read == 0 => "need",
            _ if d >= today.as_str() => "today",
            _ if d >= yesterday.as_str() => "yesterday",
            _ if d >= week.as_str() => "week",
            _ => "older",
        }.into();
        Notice {
            project_sn: m.event_sn.and_then(|s| evs.iter().find(|x| x.sn == s)).and_then(|x| x.project_sn),
            sn: m.sn, event_code: m.event_code, actor_type: m.actor_type, member_sn: m.member_sn, title: m.title, body: m.body,
            ref_type: m.ref_type, ref_sn: m.ref_sn, is_action: m.is_action, is_read: m.is_read, group, create_at: m.create_at, read_at: m.read_at,
        }
    }).collect()))
}

/// 읽음 처리 (전체 또는 sns). UI 상태라 이벤트를 남기지 않는다 (#86)
#[utoipa::path(operation_id = "notify_read", post, path = "/notifications/read", request_body = ReadBody, responses((status = 200, body = ReadOut), (status = "default", body = ErrorBody)))]
async fn read(State(db): State<DatabaseConnection>, Body(b): Body<ReadBody>) -> Res<Json<ReadOut>> {
    let mut u = n::Entity::update_many().filter(n::Column::UserSn.eq(crate::USER)).filter(n::Column::IsRead.eq(0))
        .col_expr(n::Column::IsRead, 1.into()).col_expr(n::Column::ReadAt, Expr::cust("datetime('now')"));
    if let Some(sns) = b.sns { u = u.filter(n::Column::Sn.is_in(sns)); }
    Ok(Json(ReadOut { updated: u.exec(&db).await?.rows_affected }))
}

/// 알림 규칙 목록 (워크스페이스 기본 → 연결별, 번호순). 행이 없는 이벤트 × 채널은 켜짐으로 본다
#[utoipa::path(operation_id = "notify_rules", get, path = "/notify/rules", responses((status = 200, body = Vec<Rule>), (status = "default", body = ErrorBody)))]
async fn rules(State(db): State<DatabaseConnection>) -> Res<Json<Vec<Rule>>> {
    let w = ws::Entity::find_by_id(crate::WORKSPACE).one(&db).await?.ok_or_else(Error::not_found)?;
    let rule = |connection_sn: Option<i64>, c: Cell| Rule { connection_sn, event_code: c.event_code, channel_kind: c.channel_kind, is_enabled: c.is_enabled };
    let mut out: Vec<Rule> = cells(&w.notify_json).into_iter().map(|c| rule(None, c)).collect();
    for m in cn::Entity::find().order_by_asc(cn::Column::Sn).all(&db).await? {
        out.extend(cells(&m.notify_json).into_iter().map(|c| rule(Some(m.sn), c)));
    }
    Ok(Json(out))
}

/// 알림 규칙 전체 교체 (워크스페이스 NotifyRulesUpdated). 모르는 이벤트 · 채널 · 0/1 밖 · 같은 칸 중복 · 없는 연결은 422
#[utoipa::path(operation_id = "notify_set_rules", put, path = "/notify/rules", request_body = Vec<Rule>, responses((status = 200, body = Vec<Rule>), (status = "default", body = ErrorBody)))]
async fn set_rules(State(db): State<DatabaseConnection>, Body(b): Body<Vec<Rule>>) -> Res<Json<Vec<Rule>>> {
    let mut keys: Vec<_> = b.iter().map(|x| (x.connection_sn, x.event_code.as_str(), x.channel_kind.as_str())).collect();
    keys.sort();
    keys.dedup();
    if keys.len() != b.len() || b.iter().any(|x| !CODES.contains(&x.event_code.as_str()) || !CHANNELS.contains(&x.channel_kind.as_str()) || !(0..=1).contains(&x.is_enabled)) {
        return Err(Error::invalid(format!("event_code in {CODES:?}, channel_kind in {CHANNELS:?}, is_enabled 0/1, no duplicate cell")));
    }
    let of = |conn: Option<i64>| -> String {
        json!(b.iter().filter(|x| x.connection_sn == conn).map(|x| json!({ "event_code": x.event_code, "channel_kind": x.channel_kind, "is_enabled": x.is_enabled })).collect::<Vec<_>>()).to_string()
    };
    event::run(&db, async |tx| {
        // 연결별 규칙은 전부 비우고 보낸 연결 것만 다시 쓴다
        cn::Entity::update_many().col_expr(cn::Column::NotifyJson, Expr::value(Option::<String>::None)).exec(tx).await?;
        let mut conns: Vec<i64> = b.iter().filter_map(|x| x.connection_sn).collect();
        conns.sort_unstable();
        conns.dedup();
        for sn in conns {
            if cn::Entity::update_many().filter(cn::Column::Sn.eq(sn)).col_expr(cn::Column::NotifyJson, of(Some(sn)).into()).exec(tx).await?.rows_affected == 0 {
                return Err(Error::invalid(format!("connection {sn} not found")));
            }
        }
        ws::Entity::update_many().filter(ws::Column::Sn.eq(crate::WORKSPACE)).col_expr(ws::Column::NotifyJson, of(None).into()).exec(tx).await?;
        Ok(((), vec![Ev::new(None, "workspace", crate::WORKSPACE, "NotifyRulesUpdated", &json!({ "count": b.len() }))]))
    }).await?;
    Ok(Json(b))
}

/// 채널 테스트. app은 항상 준비됨, 나머지는 채널 행이 off가 아니면 준비됨. 모르는 채널은 422
#[utoipa::path(operation_id = "notify_test", post, path = "/notify/test", request_body = TestBody, responses((status = 200, body = TestOut), (status = "default", body = ErrorBody)))]
async fn test(State(db): State<DatabaseConnection>, Body(b): Body<TestBody>) -> Res<Json<TestOut>> {
    if !CHANNELS.contains(&b.kind.as_str()) {
        return Err(Error::invalid(format!("kind in {CHANNELS:?}")));
    }
    let w = ws::Entity::find_by_id(crate::WORKSPACE).one(&db).await?.ok_or_else(Error::not_found)?;
    let ready = b.kind == "app" || channels_of(&w).iter().any(|c| c.kind == b.kind && c.status != "off");
    Ok(Json(TestOut { kind: b.kind, ready, delivered: false }))
}

/// 감사 로그 (최신순). 키 · 정책 · 연결 · 업데이트 · 차단 이벤트(tbl_log_event)를 종류로 거르고 limit 기본 100 · 최대 1000. 모르는 종류는 422.
/// title = payload의 title · name(없으면 이벤트 이름), detail = payload
#[utoipa::path(operation_id = "notify_audit", get, path = "/audit", params(AuditQuery), responses((status = 200, body = Vec<Audit>), (status = "default", body = ErrorBody)))]
async fn audit(State(db): State<DatabaseConnection>, Query(q): Query<AuditQuery>) -> Res<Json<Vec<Audit>>> {
    let kind = q.kind.as_deref();
    if kind.is_some_and(|k| !AUDITS.contains(&k)) {
        return Err(Error::invalid(format!("kind in {AUDITS:?}")));
    }
    let names: Vec<&str> = AUDIT.iter().filter(|(_, k)| kind.is_none_or(|w| w == *k)).map(|(n, _)| *n).collect();
    let rows = e::Entity::find().filter(e::Column::EventType.is_in(names)).order_by_desc(e::Column::Sn).limit(q.limit.unwrap_or(100).min(1000)).all(&db).await?;
    Ok(Json(rows.into_iter().map(|m| {
        let p: serde_json::Value = serde_json::from_str(&m.payload_json).unwrap_or_default();
        Audit {
            kind: AUDIT.iter().find(|(n, _)| *n == m.event_type).map_or_else(String::new, |(_, k)| (*k).into()),
            title: p["title"].as_str().or(p["name"].as_str()).map_or_else(|| m.event_type.clone(), str::to_owned),
            detail: Some(m.payload_json), sn: m.sn, actor_type: m.actor_type, user_sn: m.user_sn, member_sn: m.member_sn, run_sn: m.run_sn, create_at: m.create_at,
        }
    }).collect()))
}
