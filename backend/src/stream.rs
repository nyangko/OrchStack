//! 실시간 전달 (B-5 #46): snapshot(현재 상태 + 마지막 이벤트 sn) → stream(SSE) → 끊긴 사이는 events?after= 로 보충.
//! 이벤트 저장과 발행은 event::run이 하고, 여기서는 읽기와 전송만 한다
use crate::{entity::{tbl_issue as i, tbl_log_event as e, tbl_member as mb, tbl_project as p, tbl_task as t}, error::{ErrorBody, Res, Sn}, event, issue::Issue, task::Task, team::Member};
use axum::{Json, extract::{Query, State}, http::HeaderMap, response::sse::{Event, KeepAlive, Sse}};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect, sea_query::ExprTrait};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{convert::Infallible, time::Duration};
use tokio_stream::{Stream, StreamExt, wrappers::BroadcastStream};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

/// 한 번에 보충하는 이벤트 수 상한 (더 있으면 클라이언트가 마지막 sn으로 다시 부른다)
const PAGE: u64 = 500;
/// heartbeat 간격 (프록시 · 브라우저가 연결을 끊지 않게)
const HEARTBEAT: Duration = Duration::from_secs(15);

/// 실시간 관련 경로 묶음
pub fn routes() -> OpenApiRouter<DatabaseConnection> {
    OpenApiRouter::new().routes(routes!(snapshot)).routes(routes!(events)).routes(routes!(stream))
}

/// 프로젝트 현재 상태 묶음. 이걸 받은 뒤 `last_event_sn` 이후를 stream · events로 받으면 빠짐이 없다
#[derive(Serialize, ToSchema)]
pub struct Snapshot {
    issues: Vec<Issue>,
    tasks: Vec<Task>,
    /// 프로젝트 팀의 멤버 (팀이 없으면 빈 목록)
    members: Vec<Member>,
    /// 이 snapshot이 반영한 마지막 이벤트 sn (워크스페이스 전체 기준). 0이면 이벤트 없음
    last_event_sn: i64,
}

/// 이벤트 1건 (API 응답 · SSE data). payload는 JSON 그대로
#[derive(Serialize, ToSchema)]
pub struct EventOut {
    /// 전체 순번 — SSE `id` · `events?after=`의 기준
    sn: i64,
    project_sn: Option<i64>,
    /// 대상 종류 (project · issue · task · run · member …)
    aggregate_type: String,
    aggregate_sn: i64,
    /// 대상 안에서의 순번
    seq: i64,
    /// 예: TaskMoved. 모르는 종류는 클라이언트가 무시한다
    event_type: String,
    payload: Value,
    create_at: String,
}

impl From<e::Model> for EventOut {
    fn from(m: e::Model) -> Self {
        Self {
            sn: m.sn, project_sn: m.project_sn, aggregate_type: m.aggregate_type, aggregate_sn: m.aggregate_sn, seq: m.seq,
            event_type: m.event_type, payload: serde_json::from_str(&m.payload_json).unwrap_or(Value::Null), create_at: m.create_at,
        }
    }
}

/// 이 프로젝트에 해당하는 이벤트인가. project_sn이 비어 있는 이벤트(워크스페이스 공통 · 팀 · 멤버)는 모든 프로젝트에 보낸다
fn belongs(m: &e::Model, sn: i64) -> bool {
    m.project_sn.is_none_or(|p| p == sn)
}

/// 이벤트 sn 기준 after 이후를 순서대로 (프로젝트 것 + 공통 것), 최대 PAGE건
async fn after(db: &DatabaseConnection, sn: i64, after: i64) -> Res<Vec<e::Model>> {
    Ok(e::Entity::find().filter(e::Column::Sn.gt(after)).filter(e::Column::ProjectSn.eq(sn).or(e::Column::ProjectSn.is_null()))
        .order_by_asc(e::Column::Sn).limit(PAGE).all(db).await?)
}

/// 현재 상태 묶음. 프로젝트가 없으면 404
#[utoipa::path(operation_id = "stream_snapshot", get, path = "/projects/{sn}/snapshot", params(("sn" = i64, Path, description = "프로젝트 번호")), responses((status = 200, body = Snapshot), (status = "default", body = ErrorBody)))]
async fn snapshot(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Snapshot>> {
    let project = p::Entity::find_by_id(sn).one(&db).await?.ok_or_else(crate::error::Error::not_found)?;
    // 마지막 이벤트 sn을 먼저 읽는다: 그 뒤에 생긴 변경은 stream · events로 다시 오므로 중복은 있어도 빠짐은 없다
    let last = e::Entity::find().order_by_desc(e::Column::Sn).one(&db).await?.map_or(0, |m| m.sn);
    let issues = i::Entity::find().filter(i::Column::ProjectSn.eq(sn)).order_by_asc(i::Column::Num).all(&db).await?.into_iter().map(Issue::from).collect();
    let tasks = crate::task::enrich(&db, t::Entity::find().filter(t::Column::ProjectSn.eq(sn)).order_by_asc(t::Column::Num).all(&db).await?.into_iter().map(Task::from).collect()).await?;
    let members = match project.team_sn {
        Some(team) => mb::Entity::find().filter(mb::Column::TeamSn.eq(team)).order_by_asc(mb::Column::Sort).order_by_asc(mb::Column::Sn).all(&db).await?.into_iter().map(Member::from).collect(),
        None => Vec::new(),
    };
    Ok(Json(Snapshot { issues, tasks, members, last_event_sn: last }))
}

/// `events?after=` 쿼리
#[derive(Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
struct After {
    /// 이 sn 다음부터 (snapshot의 last_event_sn 또는 마지막으로 받은 SSE id)
    after: i64,
}

/// `stream?after=` 쿼리 — 브라우저가 Last-Event-ID를 못 보내는 경우(클라이언트가 직접 다시 연 연결)를 위한 대안. 둘 다 있으면 헤더가 우선
#[derive(Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
struct StreamQuery {
    after: Option<i64>,
}

/// 놓친 이벤트 재수신. after 다음부터 순서대로 최대 500건 — 500건이면 마지막 sn으로 다시 부른다
#[utoipa::path(operation_id = "stream_events", get, path = "/projects/{sn}/events", params(("sn" = i64, Path, description = "프로젝트 번호"), After), responses((status = 200, body = Vec<EventOut>), (status = "default", body = ErrorBody)))]
async fn events(State(db): State<DatabaseConnection>, Sn(sn): Sn, Query(q): Query<After>) -> Res<Json<Vec<EventOut>>> {
    Ok(Json(after(&db, sn, q.after).await?.into_iter().map(EventOut::from).collect()))
}

/// SSE 이벤트 1건: id = sn, event = event_type, data = EventOut JSON
fn sse(m: e::Model) -> Event {
    let out = EventOut::from(m);
    Event::default().id(out.sn.to_string()).event(out.event_type.clone()).data(serde_json::to_string(&out).unwrap_or_default())
}

/// SSE 스트림. 구독을 먼저 걸고, `Last-Event-ID`(또는 `?after=`)가 있으면 그 다음 이벤트를 DB에서 먼저 보낸 뒤 실시간으로 잇는다.
/// 따라서 재연결해도 이벤트를 새로 만들지 않고 빠짐도 없다. 15초마다 `ping` 이벤트(주석이 아니라 이벤트 — 클라이언트가 끊김을 감지하는 데 쓴다).
/// 수신이 밀려 버린 이벤트(lagged)는 건너뛴다 — 클라이언트가 events?after=로 메운다
#[utoipa::path(operation_id = "stream_stream", get, path = "/projects/{sn}/stream", params(("sn" = i64, Path, description = "프로젝트 번호"), ("Last-Event-ID" = Option<i64>, Header, description = "재연결 시 마지막으로 받은 이벤트 sn"), StreamQuery), responses((status = 200, description = "text/event-stream · id=이벤트 sn · event=event_type(+ ping) · data=EventOut", content_type = "text/event-stream"), (status = "default", body = ErrorBody)))]
async fn stream(State(db): State<DatabaseConnection>, Sn(sn): Sn, headers: HeaderMap, Query(q): Query<StreamQuery>) -> Res<Sse<impl Stream<Item = Result<Event, Infallible>>>> {
    let rx = event::subscribe();
    let last = headers.get("last-event-id").and_then(|v| v.to_str().ok()).and_then(|v| v.trim().parse::<i64>().ok()).or(q.after);
    let missed = match last {
        Some(id) => after(&db, sn, id).await?,
        None => Vec::new(),
    };
    // 보충분과 실시간이 겹칠 수 있어 마지막 보충 sn 이하는 실시간에서 버린다
    let floor = missed.last().map_or(last.unwrap_or(0), |m| m.sn);
    let live = BroadcastStream::new(rx).filter_map(move |r| r.ok().filter(|m| belongs(m, sn) && m.sn > floor));
    // 첫 바이트를 바로 보내야 프록시 · 브라우저가 응답 머리를 넘기고 onopen이 뜬다 (heartbeat 15초를 기다리지 않게)
    let hello = tokio_stream::once(Event::default().comment("ok"));
    let all = hello.chain(tokio_stream::iter(missed).chain(live).map(sse)).map(Ok);
    Ok(Sse::new(all).keep_alive(KeepAlive::new().interval(HEARTBEAT).event(Event::default().event("ping"))))
}
