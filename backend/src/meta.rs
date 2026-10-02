//! 태스크 부가 데이터: 완료 조건(tbl_task_criterion) · 의존(tbl_map_task_dependency) · 프로젝트 라벨 목록(tbl_project.label_json) · 저장 보기(tbl_user.task_view_json) · Diagram 배치(tbl_diagram).
//! 쓰기는 event::run 경유. Diagram 배치는 사용자별 화면 상태라 이벤트를 남기지 않는다
use crate::{entity::{tbl_diagram as dg, tbl_map_task_dependency as dp, tbl_project as pj, tbl_task as t, tbl_task_criterion as cr, tbl_user as us},
    error::{Body, Error, ErrorBody, Res, Sn}, event::{self, Ev}};
use axum::{Json, extract::{Path, State}, http::StatusCode};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

/// 허용되는 Diagram 배치
const LAYOUTS: [&str; 3] = ["auto", "grid", "manual"];
/// 허용되는 Diagram 노드 종류
const NODES: [&str; 7] = ["project", "issue", "task", "member", "skill", "mcp", "tools"];

/// meta 관련 경로 묶음
pub fn routes() -> OpenApiRouter<DatabaseConnection> {
    OpenApiRouter::new()
        .routes(routes!(criteria, set_criteria))
        .routes(routes!(add_dep))
        .routes(routes!(remove_dep))
        .routes(routes!(labels))
        .routes(routes!(views, save_view))
        .routes(routes!(remove_view))
        .routes(routes!(diagram, set_diagram))
}

/// 완료 조건 1줄. 저장 시 sn이 있으면 그 줄을 고치고(보고서 연결 유지 · #100), 없으면 새로 만든다
#[derive(Serialize, Deserialize, ToSchema)]
struct Criterion {
    sn: Option<i64>,
    content: String,
    is_done: i64,
}

/// 의존 추가 요청 본문
#[derive(Deserialize, ToSchema)]
struct DepNew {
    /// 먼저 끝나야 하는 태스크
    depend_task_sn: i64,
}

/// 프로젝트 라벨 (API 응답 형태)
#[derive(Serialize, ToSchema)]
struct Label {
    name: String,
    color: Option<String>,
}

/// 저장 보기 (API 응답 형태 · 생성 요청 본문 겸용)
#[derive(Serialize, Deserialize, ToSchema, Clone)]
struct View {
    /// 응답에서만
    sn: Option<i64>,
    name: String,
    /// 필터 조건 JSON 객체 (프로젝트 · 담당 · 상태 · 우선순위 · 정렬 — 형식은 프론트가 정한다)
    filter: serde_json::Value,
    #[serde(default)]
    sort: i64,
}

/// Diagram 보기 설정
#[derive(Serialize, Deserialize, ToSchema)]
struct DiagramView {
    /// auto | grid | manual
    layout_mode: String,
    zoom_percent: i64,
    is_show_capability: i64,
    is_show_done: i64,
}

/// Diagram 노드 위치
#[derive(Serialize, Deserialize, ToSchema)]
struct Node {
    /// project | issue | task | member | skill | mcp | tools
    node_type: String,
    node_sn: i64,
    pos_x: i64,
    pos_y: i64,
    #[serde(default)]
    is_collapsed: i64,
}

/// Diagram 배치 (보기 설정 + 수동 위치) — 조회 응답 · 저장 본문 겸용
#[derive(Serialize, Deserialize, ToSchema)]
struct Diagram {
    view: DiagramView,
    nodes: Vec<Node>,
}

/// 태스크 1건. 없으면 404
async fn task(db: &impl ConnectionTrait, sn: i64) -> Res<t::Model> {
    t::Entity::find_by_id(sn).one(db).await?.ok_or_else(Error::not_found)
}

/// 완료 조건 목록 (순서대로)
async fn rows(db: &impl ConnectionTrait, sn: i64) -> Res<Vec<Criterion>> {
    Ok(cr::Entity::find().filter(cr::Column::TaskSn.eq(sn)).order_by_asc(cr::Column::Sort).order_by_asc(cr::Column::Sn).all(db).await?
        .into_iter().map(|m| Criterion { sn: Some(m.sn), content: m.content, is_done: m.is_done }).collect())
}

/// 완료 조건 목록. 태스크가 없으면 404
#[utoipa::path(operation_id = "meta_criteria", get, path = "/tasks/{sn}/criteria", params(("sn" = i64, Path, description = "태스크 번호")), responses((status = 200, body = Vec<Criterion>), (status = "default", body = ErrorBody)))]
async fn criteria(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Vec<Criterion>>> {
    task(&db, sn).await?;
    rows(&db, sn).await.map(Json)
}

/// 완료 조건 전체 교체 (CriteriaUpdated + 체크가 바뀐 줄마다 CriterionChecked). 배열 순서 = 표시 순서.
/// 빈 내용 · 0/1 밖 · 다른 태스크의 sn은 422, 태스크가 없으면 404
#[utoipa::path(operation_id = "meta_set_criteria", put, path = "/tasks/{sn}/criteria", params(("sn" = i64, Path, description = "태스크 번호")), request_body = Vec<Criterion>, responses((status = 200, body = Vec<Criterion>), (status = "default", body = ErrorBody)))]
async fn set_criteria(State(db): State<DatabaseConnection>, Sn(sn): Sn, Body(b): Body<Vec<Criterion>>) -> Res<Json<Vec<Criterion>>> {
    if b.iter().any(|c| c.content.trim().is_empty() || !(0..=1).contains(&c.is_done)) {
        return Err(Error::invalid("content must not be empty, is_done 0/1".into()));
    }
    let out = event::run(&db, async |tx| {
        let ps = task(tx, sn).await?.project_sn;
        let old = cr::Entity::find().filter(cr::Column::TaskSn.eq(sn)).all(tx).await?;
        if b.iter().filter_map(|c| c.sn).any(|s| !old.iter().any(|o| o.sn == s)) {
            return Err(Error::invalid("criterion sn is not in this task".into()));
        }
        let keep: Vec<i64> = b.iter().filter_map(|c| c.sn).collect();
        cr::Entity::delete_many().filter(cr::Column::TaskSn.eq(sn)).filter(cr::Column::Sn.is_not_in(keep)).exec(tx).await?;
        let mut evs = Vec::new();
        for (i, c) in b.iter().enumerate() {
            let sort = i as i64 + 1;
            match c.sn {
                Some(s) => {
                    if old.iter().any(|o| o.sn == s && o.is_done != c.is_done) {
                        evs.push(Ev::new(Some(ps), "task", sn, "CriterionChecked", &json!({ "criterion_sn": s, "is_done": c.is_done })));
                    }
                    cr::Entity::update_many().filter(cr::Column::Sn.eq(s)).col_expr(cr::Column::Content, c.content.clone().into())
                        .col_expr(cr::Column::IsDone, c.is_done.into()).col_expr(cr::Column::Sort, sort.into()).exec(tx).await?;
                }
                None => {
                    cr::ActiveModel { task_sn: Set(sn), content: Set(c.content.clone()), is_done: Set(c.is_done), sort: Set(sort), ..Default::default() }.insert(tx).await?;
                }
            }
        }
        let out = rows(tx, sn).await?;
        evs.insert(0, Ev::new(Some(ps), "task", sn, "CriteriaUpdated", &json!({ "count": out.len() })));
        Ok((out, evs))
    }).await?;
    Ok(Json(out))
}

/// 의존 추가 (DependencyAdded). 같은 프로젝트 태스크만 · 자기 자신 · 순환은 422, 이미 있으면 409, 태스크가 없으면 404
#[utoipa::path(operation_id = "meta_add_dep", post, path = "/tasks/{sn}/deps", params(("sn" = i64, Path, description = "기다리는 태스크 번호")), request_body = DepNew, responses((status = 201, body = Vec<i64>), (status = "default", body = ErrorBody)))]
async fn add_dep(State(db): State<DatabaseConnection>, Sn(sn): Sn, Body(b): Body<DepNew>) -> Res<(StatusCode, Json<Vec<i64>>)> {
    let to = b.depend_task_sn;
    let out = event::run(&db, async |tx| {
        let (me, other) = (task(tx, sn).await?, task(tx, to).await?);
        if me.project_sn != other.project_sn || sn == to {
            return Err(Error::invalid("dependency must be another task in the same project".into()));
        }
        // 순환 검사: to에서 의존을 따라가 sn에 닿으면 순환
        let (mut stack, mut seen) = (vec![to], std::collections::HashSet::new());
        while let Some(x) = stack.pop() {
            if x == sn {
                return Err(Error::invalid("dependency would create a cycle".into()));
            }
            if seen.insert(x) {
                stack.extend(dp::Entity::find().filter(dp::Column::TaskSn.eq(x)).all(tx).await?.into_iter().map(|d| d.depend_task_sn));
            }
        }
        if dp::Entity::find().filter(dp::Column::TaskSn.eq(sn)).filter(dp::Column::DependTaskSn.eq(to)).one(tx).await?.is_some() {
            return Err(Error::conflict("dependency exists".into()));
        }
        dp::ActiveModel { task_sn: Set(sn), depend_task_sn: Set(to), ..Default::default() }.insert(tx).await?;
        let deps = dp::Entity::find().filter(dp::Column::TaskSn.eq(sn)).order_by_asc(dp::Column::DependTaskSn).all(tx).await?.into_iter().map(|d| d.depend_task_sn).collect();
        Ok((deps, vec![Ev::new(Some(me.project_sn), "task", sn, "DependencyAdded", &json!({ "depend_task_sn": to }))]))
    }).await?;
    Ok((StatusCode::CREATED, Json(out)))
}

/// 의존 삭제 (DependencyRemoved). 없으면 404
#[utoipa::path(operation_id = "meta_remove_dep", delete, path = "/tasks/{sn}/deps/{dep}", params(("sn" = i64, Path, description = "기다리는 태스크 번호"), ("dep" = i64, Path, description = "먼저 끝나야 하는 태스크 번호")), responses((status = 204, description = "삭제됨"), (status = "default", body = ErrorBody)))]
async fn remove_dep(State(db): State<DatabaseConnection>, Path((sn, dep)): Path<(i64, i64)>) -> Res<StatusCode> {
    event::run(&db, async |tx| {
        let ps = task(tx, sn).await?.project_sn;
        if dp::Entity::delete_many().filter(dp::Column::TaskSn.eq(sn)).filter(dp::Column::DependTaskSn.eq(dep)).exec(tx).await?.rows_affected == 0 {
            return Err(Error::not_found());
        }
        Ok(((), vec![Ev::new(Some(ps), "task", sn, "DependencyRemoved", &json!({ "depend_task_sn": dep }))]))
    }).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// 프로젝트 라벨 목록 (이름순). 라벨은 태스크 PATCH labels로 만든다. 프로젝트가 없으면 404
#[utoipa::path(operation_id = "meta_labels", get, path = "/projects/{sn}/labels", params(("sn" = i64, Path, description = "프로젝트 번호")), responses((status = 200, body = Vec<Label>), (status = "default", body = ErrorBody)))]
async fn labels(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Vec<Label>>> {
    let p = pj::Entity::find_by_id(sn).one(&db).await?.ok_or_else(Error::not_found)?;
    let mut out: Vec<Label> = p.label_json.as_deref().and_then(|j| serde_json::from_str::<Vec<serde_json::Value>>(j).ok()).unwrap_or_default().into_iter()
        .filter_map(|l| Some(Label { name: l["name"].as_str()?.to_owned(), color: l["color"].as_str().map(str::to_owned) })).collect();
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(Json(out))
}

/// 사용자의 저장 보기 (task_view_json · sort → 번호순)
async fn saved(db: &impl ConnectionTrait) -> Res<Vec<View>> {
    let u = us::Entity::find_by_id(crate::USER).one(db).await?.ok_or_else(Error::not_found)?;
    let mut v: Vec<View> = u.task_view_json.as_deref().and_then(|j| serde_json::from_str(j).ok()).unwrap_or_default();
    v.sort_by_key(|x| (x.sort, x.sn));
    Ok(v)
}

/// 저장 보기 전체를 사용자 행에 쓴다
async fn put_views(tx: &impl ConnectionTrait, v: &[View]) -> Res<()> {
    us::Entity::update_many().filter(us::Column::Sn.eq(crate::USER)).col_expr(us::Column::TaskViewJson, json!(v).to_string().into()).exec(tx).await?;
    Ok(())
}

/// 저장 보기 목록 (sort → 번호순)
#[utoipa::path(operation_id = "meta_views", get, path = "/task-views", responses((status = 200, body = Vec<View>), (status = "default", body = ErrorBody)))]
async fn views(State(db): State<DatabaseConnection>) -> Res<Json<Vec<View>>> {
    saved(&db).await.map(Json)
}

/// 저장 보기 만들기 (워크스페이스 TaskViewSaved). 빈 이름 · 객체가 아닌 필터는 422
#[utoipa::path(operation_id = "meta_save_view", post, path = "/task-views", request_body = View, responses((status = 201, body = View), (status = "default", body = ErrorBody)))]
async fn save_view(State(db): State<DatabaseConnection>, Body(b): Body<View>) -> Res<(StatusCode, Json<View>)> {
    if b.name.trim().is_empty() || !b.filter.is_object() {
        return Err(Error::invalid("name must not be empty, filter must be an object".into()));
    }
    let out = event::run(&db, async |tx| {
        let mut all = saved(tx).await?;
        let sn = all.iter().filter_map(|x| x.sn).max().unwrap_or(0) + 1;
        let out = View { sn: Some(sn), name: b.name.clone(), filter: b.filter.clone(), sort: b.sort };
        all.push(View { sn: Some(sn), name: b.name.clone(), filter: b.filter.clone(), sort: b.sort });
        put_views(tx, &all).await?;
        Ok((out, vec![Ev::new(None, "workspace", crate::WORKSPACE, "TaskViewSaved", &json!({ "view_sn": sn, "name": b.name }))]))
    }).await?;
    Ok((StatusCode::CREATED, Json(out)))
}

/// 저장 보기 삭제 (워크스페이스 TaskViewDeleted). 없으면 404
#[utoipa::path(operation_id = "meta_remove_view", delete, path = "/task-views/{sn}", params(("sn" = i64, Path, description = "보기 번호")), responses((status = 204, description = "삭제됨"), (status = "default", body = ErrorBody)))]
async fn remove_view(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<StatusCode> {
    event::run(&db, async |tx| {
        let all = saved(tx).await?;
        let keep: Vec<View> = all.iter().filter(|x| x.sn != Some(sn)).cloned().collect();
        if keep.len() == all.len() {
            return Err(Error::not_found());
        }
        put_views(tx, &keep).await?;
        Ok(((), vec![Ev::new(None, "workspace", crate::WORKSPACE, "TaskViewDeleted", &json!({ "view_sn": sn }))]))
    }).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Diagram 배치 조회 (사용자별). 저장한 적 없으면 기본 보기 + 빈 위치. 프로젝트가 없으면 404
#[utoipa::path(operation_id = "meta_diagram", get, path = "/projects/{sn}/diagram", params(("sn" = i64, Path, description = "프로젝트 번호")), responses((status = 200, body = Diagram), (status = "default", body = ErrorBody)))]
async fn diagram(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Diagram>> {
    pj::Entity::find_by_id(sn).one(&db).await?.ok_or_else(Error::not_found)?;
    let row = dg::Entity::find().filter(dg::Column::ProjectSn.eq(sn)).filter(dg::Column::UserSn.eq(crate::USER)).one(&db).await?;
    let nodes = row.as_ref().and_then(|m| m.node_json.as_deref()).and_then(|j| serde_json::from_str(j).ok()).unwrap_or_default();
    let view = row.map_or(DiagramView { layout_mode: "auto".into(), zoom_percent: 100, is_show_capability: 1, is_show_done: 0 },
        |m| DiagramView { layout_mode: m.layout_mode, zoom_percent: m.zoom_percent, is_show_capability: m.is_show_capability, is_show_done: m.is_show_done });
    Ok(Json(Diagram { view, nodes }))
}

/// Diagram 배치 저장 (보기 설정 + 위치 전체 교체 · 사용자별 화면 상태라 이벤트 없음).
/// 모르는 배치 · 노드 종류, 확대 10~400 밖, 같은 노드 중복은 422. 프로젝트가 없으면 404
#[utoipa::path(operation_id = "meta_set_diagram", put, path = "/projects/{sn}/diagram", params(("sn" = i64, Path, description = "프로젝트 번호")), request_body = Diagram, responses((status = 200, body = Diagram), (status = "default", body = ErrorBody)))]
async fn set_diagram(State(db): State<DatabaseConnection>, Sn(sn): Sn, Body(b): Body<Diagram>) -> Res<Json<Diagram>> {
    let mut keys: Vec<_> = b.nodes.iter().map(|n| (n.node_type.as_str(), n.node_sn)).collect();
    keys.sort();
    keys.dedup();
    let v = &b.view;
    if !LAYOUTS.contains(&v.layout_mode.as_str()) || !(10..=400).contains(&v.zoom_percent) || ![v.is_show_capability, v.is_show_done].iter().all(|x| (0..=1).contains(x))
        || keys.len() != b.nodes.len() || b.nodes.iter().any(|n| !NODES.contains(&n.node_type.as_str())) {
        return Err(Error::invalid(format!("layout_mode in {LAYOUTS:?}, zoom 10..=400, flags 0/1, node_type in {NODES:?}, no duplicate node")));
    }
    pj::Entity::find_by_id(sn).one(&db).await?.ok_or_else(Error::not_found)?;
    // 이벤트 0개 — 쓰기 직렬화 락 · 트랜잭션만 쓴다
    event::run(&db, async |tx| {
        dg::Entity::delete_many().filter(dg::Column::ProjectSn.eq(sn)).filter(dg::Column::UserSn.eq(crate::USER)).exec(tx).await?;
        dg::ActiveModel {
            project_sn: Set(sn), user_sn: Set(crate::USER), layout_mode: Set(v.layout_mode.clone()), zoom_percent: Set(v.zoom_percent),
            is_show_capability: Set(v.is_show_capability), is_show_done: Set(v.is_show_done), node_json: Set(Some(json!(b.nodes).to_string())), ..Default::default()
        }.insert(tx).await?;
        Ok(((), vec![]))
    }).await?;
    Ok(Json(b))
}
