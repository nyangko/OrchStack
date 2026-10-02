//! tbl_task CRUD + 전체 목록(/tasks) + MoveTask + 배정 + 라벨(PATCH labels). 응답에 의존 · 의존 대기 · 라벨을 채운다. 쓰기는 event::run 경유 (TaskCreated · TaskUpdated · TaskMoved · TaskDeleted · AgentAssigned · AgentUnassigned)
use crate::{entity::{tbl_label as lb, tbl_map_task_dependency as dp, tbl_map_task_label as tl, tbl_task::{self as t, Entity as Tbl}}, error::{Body, Error, ErrorBody, Res, Sn}, event::{self, Ev}, issue::next_num};
use axum::{Json, extract::{Query, State}, http::StatusCode};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder, sea_query::Expr};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

/// 허용되는 상태 전이 (from, to) · 원본 상태값은 #9. done · cancelled는 끝 상태라 나가는 전이가 없다.
/// waiting(의존 대기)은 저장하지 않으므로 여기 없다
const MOVES: &[(&str, &str)] = &[
    ("backlog", "todo"), ("backlog", "cancelled"),
    ("todo", "backlog"), ("todo", "in_progress"), ("todo", "blocked"), ("todo", "cancelled"),
    ("in_progress", "todo"), ("in_progress", "blocked"), ("in_progress", "review"), ("in_progress", "done"),
    ("in_progress", "failed"), ("in_progress", "cancelled"),
    ("blocked", "todo"), ("blocked", "in_progress"), ("blocked", "cancelled"),
    ("review", "in_progress"), ("review", "done"), ("review", "cancelled"), // 반려 = 재작업으로 되돌림
    ("failed", "todo"), ("failed", "cancelled"),
];

/// task 관련 경로 묶음
pub fn routes() -> OpenApiRouter<DatabaseConnection> {
    OpenApiRouter::new()
        .routes(routes!(board))
        .routes(routes!(all))
        .routes(routes!(list, create))
        .routes(routes!(read, update, remove))
        .routes(routes!(mv))
        .routes(routes!(assign, unassign))
}

/// 태스크 (API 응답 형태)
#[derive(Serialize, ToSchema)]
pub struct Task {
    sn: i64,
    project_sn: i64,
    issue_sn: Option<i64>,
    member_sn: Option<i64>,
    /// 화면 표시 번호 (이슈와 공용)
    num: i64,
    title: String,
    description: Option<String>,
    /// backlog | todo | in_progress | blocked | review | done | failed | cancelled
    status: String,
    /// 0(P0) ~ 3(P3)
    priority: i64,
    assign_by: Option<String>,
    queue_sort: Option<i64>,
    estimate_min: Option<i64>,
    eta_at: Option<String>,
    block_reason: Option<String>,
    branch: Option<String>,
    commit_count: i64,
    pr_number: Option<i64>,
    pr_status: Option<String>,
    create_by: String,
    create_at: String,
    update_at: String,
    start_at: Option<String>,
    done_at: Option<String>,
    /// 먼저 끝나야 하는 태스크들 (tbl_map_task_dependency)
    deps: Vec<i64>,
    /// 의존 대기: 끝나지 않은 의존이 있다 (저장하지 않고 계산 · #9)
    waiting: bool,
    /// 라벨 이름 (프로젝트 라벨)
    labels: Vec<String>,
}

impl From<t::Model> for Task {
    fn from(m: t::Model) -> Self {
        Self {
            sn: m.sn, project_sn: m.project_sn, issue_sn: m.issue_sn, member_sn: m.member_sn, num: m.num, title: m.title,
            description: m.description, status: m.status, priority: m.priority, assign_by: m.assign_by, queue_sort: m.queue_sort,
            estimate_min: m.estimate_min, eta_at: m.eta_at, block_reason: m.block_reason, branch: m.branch,
            commit_count: m.commit_count, pr_number: m.pr_number, pr_status: m.pr_status, create_by: m.create_by,
            create_at: m.create_at, update_at: m.update_at, start_at: m.start_at, done_at: m.done_at,
            deps: Vec::new(), waiting: false, labels: Vec::new(),
        }
    }
}

/// 생성 요청 본문. 상태는 항상 todo로 시작한다
#[derive(Deserialize, ToSchema)]
struct TaskNew {
    title: String,
    description: Option<String>,
    /// 0(P0) ~ 3(P3), 생략하면 2
    priority: Option<i64>,
}

/// 수정 요청 본문. 보낸 필드만 바꾼다 (이벤트 payload로도 그대로 저장된다). 상태는 `/tasks/{sn}/move`로만 바꾼다
#[derive(Serialize, Deserialize, ToSchema)]
struct TaskPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    /// 0(P0) ~ 3(P3)
    #[serde(skip_serializing_if = "Option::is_none")]
    priority: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    queue_sort: Option<i64>,
    /// 라벨 이름 전체 교체. 프로젝트에 없는 이름은 새로 만든다
    #[serde(skip_serializing_if = "Option::is_none")]
    labels: Option<Vec<String>>,
}

/// 전체 태스크 목록 쿼리 (Tasks 페이지 · 검색 ⌘K)
#[derive(Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
struct Search {
    project: Option<i64>,
    status: Option<String>,
    member: Option<i64>,
    priority: Option<i64>,
    /// 제목 포함 검색 · `#12`처럼 쓰면 표시 번호
    q: Option<String>,
}

/// MoveTask 요청 본문
#[derive(Deserialize, ToSchema)]
struct MoveBody {
    /// 옮겨갈 상태
    status: String,
}

/// AssignAgent 요청 본문
#[derive(Deserialize, ToSchema)]
struct AssignBody {
    /// 배정할 멤버 (Agent profile 아님)
    member_sn: i64,
}

/// 우선순위 범위 검사 (0~3)
fn prio(v: i64) -> Res<i64> {
    if (0..=3).contains(&v) { Ok(v) } else { Err(Error::invalid("priority must be 0..=3".into())) }
}

/// 1건 읽기 (의존 · 라벨 포함). 없으면 404
async fn get(db: &impl ConnectionTrait, sn: i64) -> Res<Task> {
    let one = Tbl::find_by_id(sn).one(db).await?.map(Task::from).ok_or_else(Error::not_found)?;
    enrich(db, vec![one]).await?.pop().ok_or_else(Error::not_found)
}

/// 태스크 목록에 의존 · 의존 대기 · 라벨 이름을 채운다
pub(crate) async fn enrich(db: &impl ConnectionTrait, mut ts: Vec<Task>) -> Res<Vec<Task>> {
    let sns: Vec<i64> = ts.iter().map(|x| x.sn).collect();
    let deps = dp::Entity::find().filter(dp::Column::TaskSn.is_in(sns.clone())).order_by_asc(dp::Column::DependTaskSn).all(db).await?;
    let open: Vec<i64> = Tbl::find().filter(t::Column::Sn.is_in(deps.iter().map(|d| d.depend_task_sn))).filter(t::Column::Status.ne("done")).all(db).await?
        .into_iter().map(|m| m.sn).collect();
    let maps = tl::Entity::find().filter(tl::Column::TaskSn.is_in(sns)).all(db).await?;
    let names = lb::Entity::find().filter(lb::Column::Sn.is_in(maps.iter().map(|m| m.label_sn))).order_by_asc(lb::Column::Name).all(db).await?;
    for x in &mut ts {
        x.deps = deps.iter().filter(|d| d.task_sn == x.sn).map(|d| d.depend_task_sn).collect();
        x.waiting = x.deps.iter().any(|d| open.contains(d));
        x.labels = names.iter().filter(|l| maps.iter().any(|m| m.task_sn == x.sn && m.label_sn == l.sn)).map(|l| l.name.clone()).collect();
    }
    Ok(ts)
}

/// 라벨 이름 → 태스크 라벨 전체 교체. 프로젝트에 없는 이름은 만든다. 빈 이름은 422
async fn relabel(tx: &DatabaseTransaction, sn: i64, project_sn: i64, names: &[String]) -> Res<()> {
    if names.iter().any(|n| n.trim().is_empty()) {
        return Err(Error::invalid("label name is empty".into()));
    }
    tl::Entity::delete_many().filter(tl::Column::TaskSn.eq(sn)).exec(tx).await?;
    for n in names.iter().map(|n| n.trim()).collect::<std::collections::BTreeSet<_>>() {
        let label = match lb::Entity::find().filter(lb::Column::ProjectSn.eq(project_sn)).filter(lb::Column::Name.eq(n)).one(tx).await? {
            Some(l) => l.sn,
            None => lb::ActiveModel { project_sn: Set(project_sn), name: Set(n.to_owned()), ..Default::default() }.insert(tx).await?.sn,
        };
        tl::ActiveModel { task_sn: Set(sn), label_sn: Set(label), ..Default::default() }.insert(tx).await?;
    }
    Ok(())
}

/// 프로젝트 태스크 목록 (Kanban용). `status`로 거르고 queue_sort → 번호순
#[utoipa::path(operation_id = "task_board", get, path = "/projects/{sn}/tasks", params(("sn" = i64, Path, description = "프로젝트 번호"), ("status" = Option<String>, Query, description = "이 상태만")), responses((status = 200, body = Vec<Task>), (status = "default", body = ErrorBody)))]
async fn board(State(db): State<DatabaseConnection>, Sn(sn): Sn, Query(q): Query<std::collections::HashMap<String, String>>) -> Res<Json<Vec<Task>>> {
    let mut f = Tbl::find().filter(t::Column::ProjectSn.eq(sn));
    if let Some(s) = q.get("status") { f = f.filter(t::Column::Status.eq(s.as_str())); }
    let ts = f.order_by_asc(t::Column::QueueSort).order_by_asc(t::Column::Sn).all(&db).await?.into_iter().map(Task::from).collect();
    enrich(&db, ts).await.map(Json)
}

/// 전체 태스크 목록 (프로젝트 → 번호순 · 최대 500). project · status · member · priority · q로 거른다
#[utoipa::path(operation_id = "task_all", get, path = "/tasks", params(Search), responses((status = 200, body = Vec<Task>), (status = "default", body = ErrorBody)))]
async fn all(State(db): State<DatabaseConnection>, Query(q): Query<Search>) -> Res<Json<Vec<Task>>> {
    use sea_orm::QuerySelect;
    let mut f = Tbl::find();
    if let Some(v) = q.project { f = f.filter(t::Column::ProjectSn.eq(v)); }
    if let Some(v) = q.status { f = f.filter(t::Column::Status.eq(v)); }
    if let Some(v) = q.member { f = f.filter(t::Column::MemberSn.eq(v)); }
    if let Some(v) = q.priority { f = f.filter(t::Column::Priority.eq(v)); }
    if let Some(v) = q.q.as_deref().map(str::trim).filter(|v| !v.is_empty()) {
        f = match v.strip_prefix('#').and_then(|n| n.parse::<i64>().ok()) {
            Some(n) => f.filter(t::Column::Num.eq(n)),
            None => f.filter(t::Column::Title.contains(v)),
        };
    }
    let ts = f.order_by_asc(t::Column::ProjectSn).order_by_asc(t::Column::Num).limit(500).all(&db).await?.into_iter().map(Task::from).collect();
    enrich(&db, ts).await.map(Json)
}

/// 이슈의 태스크 목록 (번호순)
#[utoipa::path(operation_id = "task_list", get, path = "/issues/{sn}/tasks", params(("sn" = i64, Path, description = "이슈 번호")), responses((status = 200, body = Vec<Task>), (status = "default", body = ErrorBody)))]
async fn list(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Vec<Task>>> {
    let ts = Tbl::find().filter(t::Column::IssueSn.eq(sn)).order_by_asc(t::Column::Num).all(&db).await?.into_iter().map(Task::from).collect();
    enrich(&db, ts).await.map(Json)
}

/// 태스크 생성 (CreateTask → TaskCreated). 이슈가 없으면 404
#[utoipa::path(operation_id = "task_create", post, path = "/issues/{sn}/tasks", params(("sn" = i64, Path, description = "이슈 번호")), request_body = TaskNew, responses((status = 201, body = Task), (status = "default", body = ErrorBody)))]
async fn create(State(db): State<DatabaseConnection>, Sn(sn): Sn, Body(b): Body<TaskNew>) -> Res<(StatusCode, Json<Task>)> {
    let priority = b.priority.map(prio).transpose()?;
    let out = event::run(&db, async |tx| {
        let issue = crate::entity::tbl_issue::Entity::find_by_id(sn).one(tx).await?.ok_or_else(Error::not_found)?;
        let num = next_num(tx, issue.project_sn).await?;
        let m = t::ActiveModel {
            project_sn: Set(issue.project_sn), issue_sn: Set(Some(sn)), num: Set(num), title: Set(b.title), description: Set(b.description),
            priority: priority.map_or(sea_orm::ActiveValue::NotSet, Set), user_sn: Set(Some(crate::USER)), ..Default::default()
        }.insert(tx).await?;
        let out = Task::from(m);
        let ev = Ev::new(Some(out.project_sn), "task", out.sn, "TaskCreated", &out);
        Ok((out, vec![ev]))
    }).await?;
    Ok((StatusCode::CREATED, Json(out)))
}

/// 1건 조회. 없으면 404
#[utoipa::path(operation_id = "task_read", get, path = "/tasks/{sn}", params(("sn" = i64, Path, description = "태스크 번호")), responses((status = 200, body = Task), (status = "default", body = ErrorBody)))]
async fn read(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Task>> {
    get(&db, sn).await.map(Json)
}

/// 부분 수정 (TaskUpdated). 상태는 못 바꾼다. labels는 이름 전체 교체(없는 라벨은 만든다). 없으면 404
#[utoipa::path(operation_id = "task_update", patch, path = "/tasks/{sn}", params(("sn" = i64, Path, description = "태스크 번호")), request_body = TaskPatch, responses((status = 200, body = Task), (status = "default", body = ErrorBody)))]
async fn update(State(db): State<DatabaseConnection>, Sn(sn): Sn, Body(b): Body<TaskPatch>) -> Res<Json<Task>> {
    b.priority.map(prio).transpose()?;
    let out = event::run(&db, async |tx| {
        let mut q = Tbl::update_many().filter(t::Column::Sn.eq(sn)).col_expr(t::Column::UpdateAt, Expr::cust("datetime('now')"));
        if let Some(v) = &b.title { q = q.col_expr(t::Column::Title, v.clone().into()); }
        if let Some(v) = &b.description { q = q.col_expr(t::Column::Description, v.clone().into()); }
        if let Some(v) = b.priority { q = q.col_expr(t::Column::Priority, v.into()); }
        if let Some(v) = b.queue_sort { q = q.col_expr(t::Column::QueueSort, v.into()); }
        if q.exec(tx).await?.rows_affected == 0 {
            return Err(Error::not_found());
        }
        if let Some(names) = &b.labels {
            let project_sn = Tbl::find_by_id(sn).one(tx).await?.ok_or_else(Error::not_found)?.project_sn;
            relabel(tx, sn, project_sn, names).await?;
        }
        let out = get(tx, sn).await?;
        let ev = Ev::new(Some(out.project_sn), "task", sn, "TaskUpdated", &b);
        Ok((out, vec![ev]))
    }).await?;
    Ok(Json(out))
}

/// 상태를 옮긴다 (표에 없는 전이는 409). MoveTask와 Run 명령이 함께 쓴다. (바뀐 태스크, 이전 상태)를 돌려준다
pub async fn shift(tx: &DatabaseTransaction, sn: i64, to: &str) -> Res<(Task, String)> {
    let cur = get(tx, sn).await?;
    if !MOVES.contains(&(cur.status.as_str(), to)) {
        return Err(Error::conflict(format!("cannot move {} -> {to}", cur.status)));
    }
    let mut q = Tbl::update_many().filter(t::Column::Sn.eq(sn))
        .col_expr(t::Column::Status, to.to_owned().into()).col_expr(t::Column::UpdateAt, Expr::cust("datetime('now')"));
    // 처음 시작한 시각 · 완료한 시각을 남긴다
    if to == "in_progress" && cur.start_at.is_none() { q = q.col_expr(t::Column::StartAt, Expr::cust("datetime('now')")); }
    if to == "done" { q = q.col_expr(t::Column::DoneAt, Expr::cust("datetime('now')")); }
    q.exec(tx).await?;
    Ok((get(tx, sn).await?, cur.status))
}

/// 상태 이동 (MoveTask → TaskMoved). 표에 없는 전이는 409, 없는 태스크는 404
#[utoipa::path(operation_id = "task_mv", post, path = "/tasks/{sn}/move", params(("sn" = i64, Path, description = "태스크 번호")), request_body = MoveBody, responses((status = 200, body = Task), (status = "default", body = ErrorBody)))]
async fn mv(State(db): State<DatabaseConnection>, Sn(sn): Sn, Body(b): Body<MoveBody>) -> Res<Json<Task>> {
    let out = event::run(&db, async |tx| {
        let (out, from) = shift(tx, sn, &b.status).await?;
        let ev = Ev::new(Some(out.project_sn), "task", sn, "TaskMoved", &json!({ "from": from, "to": b.status }));
        Ok((out, vec![ev]))
    }).await?;
    Ok(Json(out))
}

/// 삭제 (TaskDeleted). 성공 204, 없으면 404
#[utoipa::path(operation_id = "task_remove", delete, path = "/tasks/{sn}", params(("sn" = i64, Path, description = "태스크 번호")), responses((status = 204, description = "삭제됨"), (status = "default", body = ErrorBody)))]
async fn remove(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<StatusCode> {
    event::run(&db, async |tx| {
        let m = get(tx, sn).await?;
        Tbl::delete_by_id(sn).exec(tx).await?;
        Ok(((), vec![Ev::new(Some(m.project_sn), "task", sn, "TaskDeleted", &json!({ "num": m.num }))]))
    }).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// 담당 멤버 설정 (없으면 해제)과 배정 이벤트를 남긴다. 태스크가 없으면 404
async fn set_member(db: &DatabaseConnection, sn: i64, member: Option<i64>) -> Res<Task> {
    event::run(db, async |tx| put_member(tx, sn, member, "user").await.map(|(out, ev)| (out, vec![ev]))).await
}

/// 담당 멤버를 바꾸고 배정 이벤트를 만든다 (by = assign_by: user | orch_auto | orch_move). 사용자 배정과 Orch 제안 실행이 함께 쓴다.
/// 없는 태스크 404 · 없는 멤버 422 · 보관 멤버 409
pub(crate) async fn put_member(tx: &DatabaseTransaction, sn: i64, member: Option<i64>, by: &str) -> Res<(Task, Ev)> {
    get(tx, sn).await?;
    if let Some(ms) = member {
        let m = crate::entity::tbl_member::Entity::find_by_id(ms).one(tx).await?.ok_or_else(|| Error::invalid("member not found".into()))?;
        if m.status == "archived" {
            return Err(Error::conflict("member is archived".into()));
        }
    }
    // 해제는 member_sn · assign_by 모두 NULL
    let (to, by_col) = member.map_or((Expr::cust("NULL"), Expr::cust("NULL")), |v| (Expr::value(v), Expr::value(by.to_owned())));
    let q = Tbl::update_many().filter(t::Column::Sn.eq(sn)).col_expr(t::Column::MemberSn, to).col_expr(t::Column::AssignBy, by_col)
        .col_expr(t::Column::UpdateAt, Expr::cust("datetime('now')"));
    q.exec(tx).await?;
    let out = get(tx, sn).await?;
    let ev = match member {
        Some(ms) => Ev::new(Some(out.project_sn), "task", sn, "AgentAssigned", &json!({ "member_sn": ms, "assign_by": by })),
        None => Ev::new(Some(out.project_sn), "task", sn, "AgentUnassigned", &json!({})),
    };
    Ok((out, ev))
}

/// 의존 대기(먼저 끝나야 하는 태스크가 done이 아님)인 태스크 번호 집합 (sns 중에서)
pub(crate) async fn waiting(db: &impl ConnectionTrait, sns: Vec<i64>) -> Res<std::collections::HashSet<i64>> {
    let deps = dp::Entity::find().filter(dp::Column::TaskSn.is_in(sns)).all(db).await?;
    let open: Vec<i64> = Tbl::find().filter(t::Column::Sn.is_in(deps.iter().map(|d| d.depend_task_sn))).filter(t::Column::Status.ne("done")).all(db).await?
        .into_iter().map(|m| m.sn).collect();
    Ok(deps.into_iter().filter(|d| open.contains(&d.depend_task_sn)).map(|d| d.task_sn).collect())
}

/// 멤버 배정 (AssignAgent → AgentAssigned · assign_by = user). 없는 멤버 422, 보관된 멤버 409. Run은 만들지 않는다
#[utoipa::path(operation_id = "task_assign", post, path = "/tasks/{sn}/assign", params(("sn" = i64, Path, description = "태스크 번호")), request_body = AssignBody, responses((status = 200, body = Task), (status = "default", body = ErrorBody)))]
async fn assign(State(db): State<DatabaseConnection>, Sn(sn): Sn, Body(b): Body<AssignBody>) -> Res<Json<Task>> {
    set_member(&db, sn, Some(b.member_sn)).await.map(Json)
}

/// 배정 해제 (AgentUnassigned · member_sn = NULL)
#[utoipa::path(operation_id = "task_unassign", delete, path = "/tasks/{sn}/assign", params(("sn" = i64, Path, description = "태스크 번호")), responses((status = 200, body = Task), (status = "default", body = ErrorBody)))]
async fn unassign(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Task>> {
    set_member(&db, sn, None).await.map(Json)
}
