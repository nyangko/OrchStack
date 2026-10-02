//! 컨텍스트 조립기 (#120): 모델에 보내는 입력을 한 곳에서 고정 순서로 만들고(`assemble`), 보낸 출처 · 토큰을 manifest에 남긴다(`record`).
//! 리드 Run 순서: protocol → rule → role → style → report → repo_rule → 프로필 파일 → @TASK → (파일). 접두(@TASK 앞)는 같은 프로필이면 바이트까지 같다.
//! 하위 Run은 최소 입력(고정 규칙 → @TASK → paths 파일)이다. 토큰은 `preset::tokens` 추정이고 실측은 tbl_log_token에만 있다. manifest · source는 이벤트를 남기지 않는다
use crate::{entity::{tbl_agent_profile as ap, tbl_context_manifest as cm, tbl_context_source as cs, tbl_instruction_preset as ip, tbl_instruction_preset_version as iv,
    tbl_label as lb, tbl_log_token as lt, tbl_map_profile_preset as mp, tbl_map_task_dependency as dp, tbl_map_task_label as tl, tbl_member as mb, tbl_profile_file as pf,
    tbl_profile_path as pp, tbl_project as pj, tbl_run as r, tbl_task as t, tbl_task_criterion as tc, tbl_connection as cn},
    error::{Error, ErrorBody, Res, Sn}, event, preset::tokens, rule, runner::{FILE_MAX, RULES, granted}};
use axum::{Json, extract::State};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder, QuerySelect, sea_query::Expr};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{collections::HashSet, path::Path};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

/// 호출 1번의 입력 상한(토큰)
// ponytail: 상수 40K (#14 목표). 워크스페이스 · 팀별로 바꿀 칸이 필요해지면 설정 칸으로
pub const INPUT_CAP: i64 = 40_000;
/// 블록 끝 구분자. 본문을 다듬지 않고 그대로 붙여 접두 바이트를 고정한다
const END: &str = "\n\n";
/// 프리셋 종류 조합 순서
const KINDS: [&str; 5] = ["protocol", "rule", "role", "style", "report"];
/// 저장소 규칙 파일 (cwd 기준)
const REPO_FILES: [&str; 3] = ["AGENTS.md", "CLAUDE.md", ".claude/CLAUDE.md"];

/// context 관련 경로 묶음
pub fn routes() -> OpenApiRouter<DatabaseConnection> {
    OpenApiRouter::new().routes(routes!(estimate)).routes(routes!(run_context))
}

/// 컨텍스트 출처 1건
#[derive(Serialize, ToSchema, Clone)]
pub struct Source {
    /// preset | instruction | repo_rule | task | file
    pub kind: String,
    /// 표시 이름 (프리셋 = 종류/키 · 파일 = 경로)
    pub ref_label: String,
    pub ref_sn: Option<i64>,
    /// 프리셋이면 고정 버전
    pub ref_version: Option<i64>,
    /// 보낸 내용의 sha256 앞 16자
    pub content_hash: String,
    /// 이 호출에 보낸 추정 토큰 (세션을 이어 쓰며 뺀 반복 항목은 0)
    pub token_count: i64,
    /// 같은 Run의 이전 호출에 같은 (kind, ref_label, content_hash)가 있었음
    pub is_repeat: i64,
    pub sort: i64,
}

impl From<cs::Model> for Source {
    fn from(m: cs::Model) -> Self {
        Self { kind: m.kind, ref_label: m.ref_label, ref_sn: m.ref_sn, ref_version: m.ref_version, content_hash: m.content_hash.unwrap_or_default(),
            token_count: m.token_count, is_repeat: m.is_repeat, sort: m.sort }
    }
}

/// 조립 결과
pub struct Built {
    pub prompt: String,
    pub sources: Vec<Source>,
    /// 보내는 추정 토큰 합
    pub estimate: i64,
    /// 그 중 직전 호출과 접두가 같아 캐시에 맞을 것으로 보는 토큰
    pub cached_estimate: i64,
    /// token_count가 0인 프리셋 버전 (번호, 추정값) — `record`가 채워 저장한다
    fill: Vec<(i64, i64)>,
}

/// 조립 대상: 리드 Run(프로필 전체) | 하위 Run(최소 입력)
enum Spec {
    Lead { task: Box<t::Model>, profile: Box<ap::Model>, cwd: Option<String> },
    Sub { id: String, brief: String, paths: Vec<String>, cwd: Option<String> },
}

/// 블록 하나 (조립 중)
struct Item {
    kind: &'static str,
    label: String,
    ref_sn: Option<i64>,
    ref_version: Option<i64>,
    text: String,
}

/// sha256 앞 16자
fn hash(s: &str) -> String {
    Sha256::digest(s.as_bytes()).iter().take(8).map(|b| format!("{b:02x}")).collect()
}

/// 파일 읽기 (최대 FILE_MAX 바이트). 없으면 None
fn read(cwd: &str, p: &str) -> Option<String> {
    let v = std::fs::read(Path::new(cwd).join(p)).ok()?;
    Some(String::from_utf8_lossy(&v[..v.len().min(FILE_MAX)]).into_owned())
}

/// 리드용 @TASK 블록: 제목 · 설명 · 완료 조건 · 의존 · 라벨 · 허용 경로 (서버가 만든다)
async fn task_block(db: &impl ConnectionTrait, task: &t::Model, profile_sn: i64) -> Res<String> {
    let mut l = vec!["@TASK v1".to_owned(), format!("id: #{}  title: {}", task.num, task.title)];
    if let Some(d) = task.description.as_deref().filter(|d| !d.trim().is_empty()) {
        l.push(format!("desc: {d}"));
    }
    for (i, c) in tc::Entity::find().filter(tc::Column::TaskSn.eq(task.sn)).order_by_asc(tc::Column::Sort).order_by_asc(tc::Column::Sn).all(db).await?.iter().enumerate() {
        l.push(format!("ac {} {}: {}", i + 1, if c.is_done == 1 { "done" } else { "todo" }, c.content));
    }
    let deps = dp::Entity::find().filter(dp::Column::TaskSn.eq(task.sn)).all(db).await?;
    for d in t::Entity::find().filter(t::Column::Sn.is_in(deps.iter().map(|d| d.depend_task_sn))).order_by_asc(t::Column::Num).all(db).await? {
        l.push(format!("dep #{} {}: {}", d.num, d.status, d.title));
    }
    let maps = tl::Entity::find().filter(tl::Column::TaskSn.eq(task.sn)).all(db).await?;
    let names: Vec<String> = lb::Entity::find().filter(lb::Column::Sn.is_in(maps.iter().map(|m| m.label_sn))).order_by_asc(lb::Column::Name).all(db).await?.into_iter().map(|x| x.name).collect();
    if !names.is_empty() {
        l.push(format!("labels: {}", names.join(", ")));
    }
    let paths: Vec<String> = pp::Entity::find().filter(pp::Column::ProfileSn.eq(profile_sn)).order_by_asc(pp::Column::Sort).order_by_asc(pp::Column::Sn).all(db).await?
        .into_iter().map(|p| if p.kind == "exclude" { format!("!{}", p.pattern) } else { p.pattern }).collect();
    if !paths.is_empty() {
        l.push(format!("paths: {}", paths.join("  ")));
    }
    Ok(l.join("\n"))
}

/// 같은 Run의 이전 호출 출처 키 (kind, 이름, 해시) 전체 / 같은 세션에서 보낸 것만
async fn seen(db: &impl ConnectionTrait, run: Option<&r::Model>, session: Option<i64>) -> Res<(HashSet<(String, String, String)>, HashSet<(String, String, String)>)> {
    let (mut any, mut same) = (HashSet::new(), HashSet::new());
    let Some(run) = run else { return Ok((any, same)) };
    for m in cm::Entity::find().filter(cm::Column::RunSn.eq(run.sn)).all(db).await? {
        for s in cs::Entity::find().filter(cs::Column::ManifestSn.eq(m.sn)).all(db).await? {
            let k = (s.kind, s.ref_label, s.content_hash.unwrap_or_default());
            if session.is_some() && m.session_sn == session { same.insert(k.clone()); }
            any.insert(k);
        }
    }
    Ok((any, same))
}

/// 캐시 비교 기준: 이 Run의 마지막 manifest, 없으면 같은 멤버의 이전 Run(같은 종류 · 번호가 앞선 것) 중 manifest가 있는 가장 최근 Run의 마지막 manifest
async fn baseline(db: &impl ConnectionTrait, run: Option<&r::Model>, member: Option<i64>, lead: bool) -> Res<Vec<cs::Model>> {
    let mut runs = Vec::new();
    if let Some(rn) = run { runs.push(rn.sn); }
    if let Some(ms) = member {
        let mut q = r::Entity::find().filter(r::Column::MemberSn.eq(ms)).filter(if lead { r::Column::ParentRunSn.is_null() } else { r::Column::ParentRunSn.is_not_null() });
        if let Some(rn) = run { q = q.filter(r::Column::Sn.lt(rn.sn)); }
        runs.extend(q.order_by_desc(r::Column::Sn).limit(20).all(db).await?.into_iter().map(|x| x.sn));
    }
    for sn in runs {
        if let Some(m) = cm::Entity::find().filter(cm::Column::RunSn.eq(sn)).order_by_desc(cm::Column::Sn).one(db).await? {
            return Ok(cs::Entity::find().filter(cs::Column::ManifestSn.eq(m.sn)).order_by_asc(cs::Column::Sort).all(db).await?);
        }
    }
    Ok(Vec::new())
}

/// 조립. run은 반복 · 캐시 비교 기준(없으면 비교 없음), member는 캐시 비교 멤버, resume = 세션을 이어 쓰는 호출(반복 항목을 프롬프트에서 뺀다)
async fn build(db: &impl ConnectionTrait, sp: &Spec, run: Option<&r::Model>, member: Option<i64>, session: Option<i64>, resume: bool) -> Res<Built> {
    let mut items: Vec<Item> = Vec::new();
    let mut fill = Vec::new();
    let it = |kind, label: String, ref_sn, ref_version, text: String| Item { kind, label, ref_sn, ref_version, text };
    match sp {
        Spec::Lead { task, profile, cwd } => {
            // 프리셋: 종류 순서 → 연결 순서. 고정 버전 본문 그대로
            let maps = mp::Entity::find().filter(mp::Column::ProfileSn.eq(profile.sn)).filter(mp::Column::IsEnabled.eq(1)).order_by_asc(mp::Column::Sort).order_by_asc(mp::Column::Sn).all(db).await?;
            let presets = ip::Entity::find().filter(ip::Column::Sn.is_in(maps.iter().map(|m| m.preset_sn))).all(db).await?;
            let mut rows = Vec::new();
            for m in &maps {
                let Some(p) = presets.iter().find(|p| p.sn == m.preset_sn) else { continue };
                let Some(k) = KINDS.iter().position(|k| *k == p.kind) else { continue };
                let Some(v) = iv::Entity::find().filter(iv::Column::PresetSn.eq(p.sn)).filter(iv::Column::Version.eq(m.pinned_version)).one(db).await? else { continue };
                rows.push((k, p, v));
            }
            rows.sort_by_key(|x| x.0);
            for (_, p, v) in rows {
                if v.token_count == 0 { fill.push((v.sn, tokens(&v.content))); }
                items.push(it("preset", format!("{}/{}", p.kind, p.preset_key), Some(p.sn), Some(v.version), format!("{}{END}", v.content)));
            }
            // 저장소 규칙 (use일 때만 · 각 FILE_MAX)
            if profile.repo_rule_mode == "use" && let Some(cwd) = cwd {
                for f in REPO_FILES {
                    if let Some(body) = read(cwd, f) { items.push(it("repo_rule", f.into(), None, None, format!("{body}{END}"))); }
                }
            }
            for f in pf::Entity::find().filter(pf::Column::ProfileSn.eq(profile.sn)).order_by_asc(pf::Column::Sort).order_by_asc(pf::Column::Sn).all(db).await? {
                items.push(it("instruction", f.path, Some(f.sn), None, format!("{}{END}", f.content)));
            }
            items.push(it("task", format!("@TASK #{}", task.num), Some(task.sn), None, format!("{}{END}", task_block(db, task, profile.sn).await?)));
        }
        Spec::Sub { id, brief, paths, cwd } => {
            items.push(it("instruction", "runner-rules".into(), None, None, format!("{RULES}{END}")));
            items.push(it("task", format!("@TASK {id}"), None, None, format!("{brief}{END}")));
            for p in paths {
                let body = cwd.as_deref().filter(|_| !p.contains('*')).and_then(|c| read(c, p));
                let text = match body { Some(b) => format!("--- {p}\n{b}{END}"), None => format!("--- {p} (not read: new file or glob){END}") };
                items.push(it("file", p.clone(), None, None, text));
            }
        }
    }
    let (any, same) = seen(db, run, session).await?;
    let mut sources = Vec::new();
    let mut prompt = String::new();
    for (i, x) in items.into_iter().enumerate() {
        let h = hash(&x.text);
        let key = (x.kind.to_owned(), x.label.clone(), h.clone());
        let rep = any.contains(&key);
        // 세션을 이어 쓰면 같은 세션에서 이미 보낸 항목은 다시 보내지 않는다. 새 세션이면 보내되 repeat로 표시
        let omit = rep && resume && same.contains(&key);
        let token_count = if omit { 0 } else { tokens(&x.text) };
        if !omit { prompt += &x.text; }
        sources.push(Source { kind: x.kind.into(), ref_label: x.label, ref_sn: x.ref_sn, ref_version: x.ref_version, content_hash: h, token_count, is_repeat: rep as i64,
            sort: i as i64 });
    }
    // 접두가 이어지는 동안만 캐시에 맞는다고 본다 (제공자 프롬프트 캐시는 접두 일치)
    let base = baseline(db, run, member, matches!(sp, Spec::Lead { .. })).await?;
    let cached = sources.iter().zip(&base).take_while(|(s, b)| s.kind == b.kind && s.ref_label == b.ref_label && Some(&s.content_hash) == b.content_hash.as_ref()).map(|(s, _)| s.token_count).sum();
    Ok(Built { estimate: sources.iter().map(|s| s.token_count).sum(), cached_estimate: cached, prompt, sources, fill })
}

/// Run의 컨텍스트 조립. 리드 Run = 프로필 프리셋 · 저장소 규칙 · 프로필 파일 · 서버가 만든 @TASK, 하위 Run = 고정 규칙 · brief · paths 파일.
/// session_sn이 있고 프로필이 resume_task면 그 세션에서 이미 보낸 항목을 프롬프트에서 뺀다 (실행기가 같은 세션을 이어 쓸 때만 넘긴다)
pub async fn assemble(db: &impl ConnectionTrait, run_sn: i64, session_sn: Option<i64>) -> Res<Built> {
    let run = r::Entity::find_by_id(run_sn).one(db).await?.ok_or_else(Error::not_found)?;
    let member = mb::Entity::find_by_id(run.member_sn).one(db).await?.ok_or_else(Error::not_found)?;
    let profile = ap::Entity::find_by_id(member.profile_sn).one(db).await?.ok_or_else(Error::not_found)?;
    let cwd = pj::Entity::find_by_id(run.project_sn).one(db).await?.and_then(|p| p.repo_path);
    let resume = session_sn.is_some() && profile.session_mode == "resume_task";
    let sp = match (&run.brief, run.parent_run_sn) {
        (Some(brief), Some(_)) => {
            let b = rule::parse(brief).map_err(|_| Error::conflict("stored brief is invalid".into()))?;
            Spec::Sub { id: b.id, brief: brief.clone(), paths: granted(&run), cwd }
        }
        _ => Spec::Lead { task: Box::new(t::Entity::find_by_id(run.task_sn).one(db).await?.ok_or_else(Error::not_found)?), profile: Box::new(profile), cwd },
    };
    build(db, &sp, Some(&run), Some(run.member_sn), session_sn, resume).await
}

/// Run을 만들지 않고 태스크의 리드 컨텍스트를 조립한다 (견적 · 시작 전 검사). 담당 멤버가 없으면 409
pub async fn preview(db: &impl ConnectionTrait, task_sn: i64) -> Res<Built> {
    let task = t::Entity::find_by_id(task_sn).one(db).await?.ok_or_else(Error::not_found)?;
    let ms = task.member_sn.ok_or_else(|| Error::conflict("task has no member".into()))?;
    let member = mb::Entity::find_by_id(ms).one(db).await?.ok_or_else(Error::not_found)?;
    let profile = ap::Entity::find_by_id(member.profile_sn).one(db).await?.ok_or_else(Error::not_found)?;
    let cwd = pj::Entity::find_by_id(task.project_sn).one(db).await?.and_then(|p| p.repo_path);
    build(db, &Spec::Lead { task: Box::new(task), profile: Box::new(profile), cwd }, None, Some(ms), None, false).await
}

/// 상한을 넘으면 큰 출처 상위 3개와 함께 설명 문구 (아니면 None)
pub fn over(b: &Built) -> Option<String> {
    if b.estimate <= INPUT_CAP {
        return None;
    }
    let k = |n: i64| format!("{:.1}K", n as f64 / 1000.0);
    let mut top: Vec<&Source> = b.sources.iter().collect();
    top.sort_by_key(|s| std::cmp::Reverse(s.token_count));
    let top: Vec<String> = top.into_iter().take(3).map(|s| format!("{} {} {}", s.kind, s.ref_label, k(s.token_count))).collect();
    Some(format!("input {} > cap {}. top: {}", k(b.estimate), k(INPUT_CAP), top.join(", ")))
}

/// 상한 검사: 넘으면 422 context_over
pub fn check(b: &Built) -> Res<()> {
    over(b).map_or(Ok(()), |m| Err(Error::over(m)))
}

/// 리드 Run을 만들기 전 검사 (담당 멤버가 없으면 건너뜀 — 시작 쪽이 409로 막는다). 넘으면 422 context_over
pub async fn guard(db: &impl ConnectionTrait, task_sn: i64) -> Res<()> {
    match t::Entity::find_by_id(task_sn).one(db).await? {
        Some(task) if task.member_sn.is_some() => check(&preview(db, task_sn).await?),
        _ => Ok(()),
    }
}

/// 하위 Run을 만들기 전 검사: brief · paths로 조립해 상한을 넘으면 그 설명 문구 (리드에게 돌려준다)
pub async fn over_sub(db: &impl ConnectionTrait, project_sn: i64, id: &str, brief: &str, paths: &[String]) -> Res<Option<String>> {
    let cwd = pj::Entity::find_by_id(project_sn).one(db).await?.and_then(|p| p.repo_path);
    let sp = Spec::Sub { id: id.into(), brief: brief.into(), paths: paths.to_vec(), cwd };
    Ok(over(&build(db, &sp, None, None, None, false).await?))
}

/// 조립 결과를 manifest의 출처로 저장한다 (이벤트 없음). 토큰 수가 0인 프리셋 버전은 추정값을 채워 저장한다
pub async fn record(tx: &DatabaseTransaction, b: &Built, manifest_sn: i64) -> Res<()> {
    for s in &b.sources {
        cs::ActiveModel { manifest_sn: Set(manifest_sn), kind: Set(s.kind.clone()), ref_label: Set(s.ref_label.clone()), ref_sn: Set(s.ref_sn), ref_version: Set(s.ref_version),
            content_hash: Set(Some(s.content_hash.clone())), token_count: Set(s.token_count), is_repeat: Set(s.is_repeat), sort: Set(s.sort), ..Default::default() }.insert(tx).await?;
    }
    fill(tx, &b.fill).await
}

/// 토큰 수 0인 프리셋 버전에 추정값 채우기 (한 번 · 이미 채워졌으면 건드리지 않는다)
async fn fill(tx: &DatabaseTransaction, rows: &[(i64, i64)]) -> Res<()> {
    for (sn, n) in rows {
        iv::Entity::update_many().filter(iv::Column::Sn.eq(*sn)).filter(iv::Column::TokenCount.eq(0)).col_expr(iv::Column::TokenCount, (*n).into()).exec(tx).await?;
    }
    Ok(())
}

/// 호출 뒤 연결 갱신: cache_hit_percent(최근 20건 cache_read / (input + cache_read)) · sync_at. 지연 시간 · 한도는 실행기가 주지 않아 건드리지 않는다
pub async fn touch(tx: &DatabaseTransaction, connection_sn: Option<i64>) -> Res<()> {
    let Some(sn) = connection_sn else { return Ok(()) };
    let rows = lt::Entity::find().filter(lt::Column::ConnectionSn.eq(sn)).order_by_desc(lt::Column::Sn).limit(20).all(tx).await?;
    let (inp, read): (i64, i64) = rows.iter().fold((0, 0), |(i, c), x| (i + x.token_input, c + x.token_cache_read));
    let mut q = cn::Entity::update_many().filter(cn::Column::Sn.eq(sn)).col_expr(cn::Column::SyncAt, Expr::cust("datetime('now')"));
    if inp + read > 0 {
        q = q.col_expr(cn::Column::CacheHitPercent, ((read * 100 + (inp + read) / 2) / (inp + read)).into());
    }
    q.exec(tx).await?;
    Ok(())
}

/// 실행 전 견적 (Run을 만들지 않는다)
#[derive(Serialize, ToSchema)]
struct Estimate {
    /// 보낼 추정 토큰 합
    estimate: i64,
    /// 그 중 같은 멤버의 직전 호출과 접두가 같아 캐시에 맞을 것으로 보는 토큰
    cached_estimate: i64,
    /// 호출당 입력 상한
    cap: i64,
    /// estimate > cap (이 상태로는 Run을 시작할 수 없다)
    over: bool,
    sources: Vec<Source>,
}

/// 사전 견적: Run을 만들지 않고 조립만 한다. 태스크가 없으면 404 · 담당 멤버가 없으면 409.
/// 상한을 넘어도 422가 아니라 over = true로 돌려준다 (어느 출처가 큰지 화면에 보여 주려고)
#[utoipa::path(operation_id = "context_estimate", post, path = "/tasks/{sn}/estimate", params(("sn" = i64, Path, description = "태스크 번호")), responses((status = 200, body = Estimate), (status = "default", body = ErrorBody)))]
async fn estimate(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Estimate>> {
    let b = preview(&db, sn).await?;
    if !b.fill.is_empty() {
        event::run_as(&db, "system", None, async |tx| fill(tx, &b.fill).await.map(|()| ((), vec![]))).await?;
    }
    Ok(Json(Estimate { estimate: b.estimate, cached_estimate: b.cached_estimate, cap: INPUT_CAP, over: b.estimate > INPUT_CAP, sources: b.sources }))
}

/// 호출 1번의 컨텍스트 목록
#[derive(Serialize, ToSchema)]
struct Manifest {
    sn: i64,
    session_sn: Option<i64>,
    /// 허용 예산(토큰)
    budget_token: i64,
    /// 이 호출에 보낸 추정 토큰 합
    total: i64,
    /// 이전 호출과 같은 출처 수
    repeat_count: i64,
    /// 실측 캐시 적중률 (이 호출의 tbl_log_token · 없으면 null)
    cache_percent: Option<i64>,
    create_at: String,
    sources: Vec<Source>,
}

/// Run의 컨텍스트 (호출마다 1개)
#[derive(Serialize, ToSchema)]
struct RunContext {
    run_sn: i64,
    manifests: Vec<Manifest>,
    /// 모든 호출에 보낸 추정 토큰 합
    total: i64,
}

/// Run의 컨텍스트: manifest 목록(호출 순) + 출처 · is_repeat · 합계 · 호출별 실측 캐시율. Run이 없으면 404
#[utoipa::path(operation_id = "context_run", get, path = "/runs/{sn}/context", params(("sn" = i64, Path, description = "Run 번호")), responses((status = 200, body = RunContext), (status = "default", body = ErrorBody)))]
async fn run_context(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<RunContext>> {
    r::Entity::find_by_id(sn).one(&db).await?.ok_or_else(Error::not_found)?;
    let mut manifests = Vec::new();
    for m in cm::Entity::find().filter(cm::Column::RunSn.eq(sn)).order_by_asc(cm::Column::Sn).all(&db).await? {
        let sources: Vec<Source> = cs::Entity::find().filter(cs::Column::ManifestSn.eq(m.sn)).order_by_asc(cs::Column::Sort).all(&db).await?.into_iter().map(Source::from).collect();
        let (inp, read) = lt::Entity::find().filter(lt::Column::ManifestSn.eq(m.sn)).all(&db).await?.iter().fold((0, 0), |(i, c), x| (i + x.token_input, c + x.token_cache_read));
        manifests.push(Manifest {
            sn: m.sn, session_sn: m.session_sn, budget_token: m.budget_token, total: sources.iter().map(|s| s.token_count).sum(), repeat_count: sources.iter().filter(|s| s.is_repeat == 1).count() as i64,
            cache_percent: (inp + read > 0).then(|| (read * 100 + (inp + read) / 2) / (inp + read)), create_at: m.create_at, sources,
        });
    }
    Ok(Json(RunContext { run_sn: sn, total: manifests.iter().map(|m| m.total).sum(), manifests }))
}
