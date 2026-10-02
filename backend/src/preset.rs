//! 프리셋 편집(새로 · 복제 · 새 버전 · .md 가져오기 · 사용처)과 보고서 양식(tbl_report_form 조회 · 수정 · 미리보기).
//! 조회(목록 · 버전)는 setting.rs. 저장 검사(토큰 상한 · 비밀키 · 겹치는 규칙 줄)는 서버가 판정해 422 사유로 돌려준다
use crate::{entity::{tbl_instruction_preset as ip, tbl_instruction_preset_version as iv, tbl_map_profile_preset as pp, tbl_report_form as rf,
    tbl_report_item as ri, tbl_run as r, tbl_run_file as fl, tbl_task as t},
    error::{Body, Error, ErrorBody, Res, Sn}, event::{self, Ev}, run, setting::Preset, skill::{Usage, owners}};
use axum::{Json, extract::{Path, Query, State}, http::StatusCode};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder, sea_query::Expr};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};

/// 사용자가 만들 수 있는 프리셋 종류 (protocol은 시스템 소유)
const KINDS: [&str; 4] = ["role", "style", "rule", "report"];
/// 비밀키로 보는 접두어 (뒤에 키 글자가 16자 이상 이어질 때)
const SECRETS: [&str; 7] = ["sk-", "ghp_", "github_pat_", "AKIA", "xoxb-", "AIza", "-----BEGIN"];

/// preset · report form 관련 경로 묶음
pub fn routes() -> OpenApiRouter<DatabaseConnection> {
    OpenApiRouter::new()
        .routes(routes!(create))
        .routes(routes!(edit))
        .routes(routes!(usage))
        .routes(routes!(import))
        .routes(routes!(forms))
        .routes(routes!(form, set_form))
        .routes(routes!(preview))
}

/// 프리셋 만들기 요청 본문. copy_from_sn이 있으면 그 프리셋의 최신 본문 · 종류 · 상한을 기본값으로 복제한다
#[derive(Deserialize, ToSchema)]
struct PresetNew {
    preset_key: String,
    name: String,
    /// role | style | rule | report (복제면 생략 가능)
    kind: Option<String>,
    description: Option<String>,
    limit_tok: Option<i64>,
    content: Option<String>,
    /// 본문 언어 (기본 en)
    language: Option<String>,
    copy_from_sn: Option<i64>,
}

/// 새 버전 요청 본문
#[derive(Deserialize, ToSchema)]
struct PresetEdit {
    content: String,
    change_note: Option<String>,
    name: Option<String>,
    description: Option<String>,
    limit_tok: Option<i64>,
    language: Option<String>,
}

/// .md 가져오기 요청 본문. 앞머리(--- kind · key · name · limit_tok ---) + 본문
#[derive(Deserialize, ToSchema)]
struct ImportBody {
    markdown: String,
}

/// 보고서 양식 (API 응답 형태)
#[derive(Serialize, ToSchema)]
struct Form {
    sn: i64,
    /// task_report | issue_report | pr_body | daily_summary
    kind: String,
    form_key: String,
    name: String,
    version: i64,
    /// {{시스템 값}} · [[Agent 사람 칸]] 자리표시
    body: String,
    /// 지울 수 없는 칸 (예: unverified, tests.passed_summary)
    locked: Vec<String>,
    is_builtin: i64,
    is_default: i64,
    update_at: String,
}

impl From<rf::Model> for Form {
    fn from(m: rf::Model) -> Self {
        Self {
            locked: m.locked_json.as_deref().and_then(|j| serde_json::from_str(j).ok()).unwrap_or_default(),
            sn: m.sn, kind: m.kind, form_key: m.form_key, name: m.name, version: m.version, body: m.body, is_builtin: m.is_builtin, is_default: m.is_default, update_at: m.update_at,
        }
    }
}

/// 양식 수정 요청 본문
#[derive(Deserialize, ToSchema)]
struct FormEdit {
    body: String,
    name: Option<String>,
}

/// 미리보기 쿼리
#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct PreviewQuery {
    /// 조립에 쓸 태스크 (마지막 리드 Run의 보고 · 사실을 쓴다)
    task: i64,
}

/// 미리보기 결과
#[derive(Serialize, ToSchema)]
struct Preview {
    body: String,
    /// 값이 없어 자리표시 그대로 남은 칸
    missing: Vec<String>,
}

/// 본문 토큰 수 추정: ASCII 글자 / 4 + CJK(한글 · 한자 · 가나) 글자 × 1.5, 올림
// ponytail: 토크나이저 없는 근사. 실행기 실측(tbl_context_manifest.token_count)이 오면 교체
fn tokens(s: &str) -> i64 {
    // n/4 + 1.5k = (n + 6k)/4 — 정수로 올림
    let cjk = |c: char| matches!(c, '\u{AC00}'..='\u{D7A3}' | '\u{1100}'..='\u{11FF}' | '\u{3130}'..='\u{318F}' | '\u{4E00}'..='\u{9FFF}' | '\u{3400}'..='\u{4DBF}' | '\u{3040}'..='\u{30FF}');
    let (n, k) = s.chars().fold((0i64, 0i64), |(n, k), c| if cjk(c) { (n, k + 1) } else { (n + 1, k) });
    (n + 6 * k + 3) / 4
}

/// 저장 검사. 사유 목록이 비어 있으면 통과. 겹치는 규칙 줄 = 본문 안 중복 "- " 줄 + (rule이면) 다른 활성 rule 프리셋 최신 본문과 같은 줄
async fn verdict(db: &impl ConnectionTrait, kind: &str, me: Option<i64>, content: &str, limit: i64) -> Res<Vec<String>> {
    let mut why = Vec::new();
    let n = tokens(content);
    if n > limit {
        why.push(format!("token limit: {n} > {limit}"));
    }
    for p in SECRETS {
        let hit = content.match_indices(p).any(|(i, _)| {
            p == "-----BEGIN" || content[i + p.len()..].chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-').count() >= 16
        });
        if hit {
            why.push(format!("secret pattern: {p}"));
        }
    }
    let lines: Vec<&str> = content.lines().map(str::trim).filter(|l| l.starts_with("- ")).collect();
    let mut seen = std::collections::HashSet::new();
    for l in &lines {
        if !seen.insert(*l) {
            why.push(format!("duplicate rule line: {l}"));
        }
    }
    if kind == "rule" {
        let others = ip::Entity::find().filter(ip::Column::Kind.eq("rule")).filter(ip::Column::Status.eq("active")).all(db).await?;
        for o in others.into_iter().filter(|o| Some(o.sn) != me) {
            let Some(v) = iv::Entity::find().filter(iv::Column::PresetSn.eq(o.sn)).filter(iv::Column::Version.eq(o.version)).one(db).await? else { continue };
            for l in v.content.lines().map(str::trim).filter(|l| lines.contains(l)) {
                why.push(format!("rule line also in {}: {l}", o.preset_key));
            }
        }
    }
    Ok(why)
}

/// 검사 사유가 있으면 422
fn reject(why: Vec<String>) -> Res<()> {
    if why.is_empty() { Ok(()) } else { Err(Error::invalid(why.join("; "))) }
}

/// 프리셋 1건. 없으면 404
async fn preset(db: &impl ConnectionTrait, sn: i64) -> Res<ip::Model> {
    ip::Entity::find_by_id(sn).one(db).await?.ok_or_else(Error::not_found)
}

/// 새 프리셋 + 버전 1 (PresetCreated)
#[allow(clippy::too_many_arguments)]
async fn insert(tx: &DatabaseTransaction, kind: &str, key: &str, name: &str, description: Option<String>, limit: i64, content: &str, language: &str, source: &str, copy_from: Option<i64>) -> Res<(Preset, Ev)> {
    let m = ip::ActiveModel {
        wid: Set(crate::WID), kind: Set(kind.into()), preset_key: Set(key.into()), name: Set(name.into()), description: Set(description), limit_tok: Set(limit),
        copy_from_sn: Set(copy_from), uid: Set(Some(crate::UID)), ..Default::default()
    }.insert(tx).await?;
    iv::ActiveModel {
        preset_sn: Set(m.sn), version: Set(1), content: Set(content.into()), token_count: Set(tokens(content)), language: Set(language.into()), source: Set(source.into()),
        uid: Set(Some(crate::UID)), ..Default::default()
    }.insert(tx).await?;
    let ev = Ev::new(None, "preset", m.sn, "PresetCreated", &json!({ "kind": kind, "preset_key": key, "copy_from_sn": copy_from }));
    Ok((Preset::from(m), ev))
}

/// 기존 프리셋에 새 버전 (PresetVersioned). 기존 연결의 고정 버전은 건드리지 않는다. 잠김 · 기본 제공은 409
#[allow(clippy::too_many_arguments)]
async fn bump(tx: &DatabaseTransaction, cur: &ip::Model, content: &str, note: Option<String>, language: &str, source: &str, name: Option<String>, description: Option<String>, limit: i64) -> Res<(Preset, Ev)> {
    if cur.is_locked == 1 || cur.is_builtin == 1 {
        return Err(Error::conflict(format!("preset {} is builtin or locked — copy it to edit", cur.preset_key)));
    }
    let version = cur.version + 1;
    iv::ActiveModel {
        preset_sn: Set(cur.sn), version: Set(version), content: Set(content.into()), token_count: Set(tokens(content)), language: Set(language.into()), source: Set(source.into()),
        change_note: Set(note.clone()), uid: Set(Some(crate::UID)), ..Default::default()
    }.insert(tx).await?;
    let mut u = ip::Entity::update_many().filter(ip::Column::Sn.eq(cur.sn)).col_expr(ip::Column::Version, version.into()).col_expr(ip::Column::LimitTok, limit.into())
        .col_expr(ip::Column::UpdateAt, Expr::cust("datetime('now')"));
    if let Some(v) = name { u = u.col_expr(ip::Column::Name, v.into()); }
    if let Some(v) = description { u = u.col_expr(ip::Column::Description, v.into()); }
    u.exec(tx).await?;
    let ev = Ev::new(None, "preset", cur.sn, "PresetVersioned", &json!({ "version": version, "source": source, "change_note": note }));
    Ok((Preset::from(preset(tx, cur.sn).await?), ev))
}

/// 프리셋 만들기 · 복제 (PresetCreated). 모르는 종류(protocol 포함) · 빈 키 · 이름 · 본문, 검사 실패는 422. 같은 종류 · 키가 있으면 409
#[utoipa::path(operation_id = "preset_create", post, path = "/presets", request_body = PresetNew, responses((status = 201, body = Preset), (status = "default", body = ErrorBody)))]
async fn create(State(db): State<DatabaseConnection>, Body(b): Body<PresetNew>) -> Res<(StatusCode, Json<Preset>)> {
    let out = event::run(&db, async |tx| {
        let src = match b.copy_from_sn {
            Some(s) => {
                let p = preset(tx, s).await?;
                let v = iv::Entity::find().filter(iv::Column::PresetSn.eq(s)).filter(iv::Column::Version.eq(p.version)).one(tx).await?.ok_or_else(Error::not_found)?;
                Some((p, v))
            }
            None => None,
        };
        let kind = b.kind.clone().or_else(|| src.as_ref().map(|s| s.0.kind.clone())).unwrap_or_default();
        let content = b.content.clone().or_else(|| src.as_ref().map(|s| s.1.content.clone())).unwrap_or_default();
        let limit = b.limit_tok.or_else(|| src.as_ref().map(|s| s.0.limit_tok)).unwrap_or(0);
        let language = b.language.clone().or_else(|| src.as_ref().map(|s| s.1.language.clone())).unwrap_or_else(|| "en".into());
        if !KINDS.contains(&kind.as_str()) || b.preset_key.trim().is_empty() || b.name.trim().is_empty() || content.trim().is_empty() || limit <= 0 {
            return Err(Error::invalid(format!("kind in {KINDS:?}, preset_key · name · content non-empty, limit_tok > 0")));
        }
        if ip::Entity::find().filter(ip::Column::Kind.eq(kind.as_str())).filter(ip::Column::PresetKey.eq(b.preset_key.trim())).one(tx).await?.is_some() {
            return Err(Error::conflict(format!("preset {kind}/{} exists", b.preset_key.trim())));
        }
        reject(verdict(tx, &kind, None, &content, limit).await?)?;
        let (out, ev) = insert(tx, &kind, b.preset_key.trim(), b.name.trim(), b.description.clone(), limit, &content, &language, "user", b.copy_from_sn).await?;
        Ok((out, vec![ev]))
    }).await?;
    Ok((StatusCode::CREATED, Json(out)))
}

/// 새 버전 저장 (PresetVersioned). 연결된 프로필의 고정 버전은 그대로(업데이트는 사용자가 가져올 때). 잠김 · 기본 제공 409, 검사 실패 422, 없으면 404
#[utoipa::path(operation_id = "preset_edit", put, path = "/presets/{sn}", params(("sn" = i64, Path, description = "프리셋 번호")), request_body = PresetEdit, responses((status = 200, body = Preset), (status = "default", body = ErrorBody)))]
async fn edit(State(db): State<DatabaseConnection>, Sn(sn): Sn, Body(b): Body<PresetEdit>) -> Res<Json<Preset>> {
    let out = event::run(&db, async |tx| {
        let cur = preset(tx, sn).await?;
        let limit = b.limit_tok.unwrap_or(cur.limit_tok);
        if b.content.trim().is_empty() || limit <= 0 {
            return Err(Error::invalid("content non-empty, limit_tok > 0".into()));
        }
        reject(verdict(tx, &cur.kind, Some(sn), &b.content, limit).await?)?;
        let (out, ev) = bump(tx, &cur, &b.content, b.change_note.clone(), b.language.as_deref().unwrap_or("en"), "user", b.name.clone(), b.description.clone(), limit).await?;
        Ok((out, vec![ev]))
    }).await?;
    Ok(Json(out))
}

/// 프리셋 사용처: 연결한 프로필의 멤버 · 템플릿 (detail = 고정 버전 "v3" · 꺼져 있으면 "v3 off"). 없으면 404
#[utoipa::path(operation_id = "preset_usage", get, path = "/presets/{sn}/usage", params(("sn" = i64, Path, description = "프리셋 번호")), responses((status = 200, body = Vec<Usage>), (status = "default", body = ErrorBody)))]
async fn usage(State(db): State<DatabaseConnection>, Sn(sn): Sn) -> Res<Json<Vec<Usage>>> {
    preset(&db, sn).await?;
    let links = pp::Entity::find().filter(pp::Column::PresetSn.eq(sn)).all(&db).await?.into_iter()
        .map(|m| (m.profile_sn, format!("v{}{}", m.pinned_version, if m.is_enabled == 1 { "" } else { " off" }))).collect();
    owners(&db, links).await.map(Json)
}

/// .md 가져오기: 앞머리 kind · key(필수) · name · limit_tok. 같은 종류 · 키가 있으면 새 버전(source import), 없으면 새 프리셋(limit_tok 필수). 형식 · 검사 실패 422
#[utoipa::path(operation_id = "preset_import", post, path = "/presets/import", request_body = ImportBody, responses((status = 200, body = Preset), (status = "default", body = ErrorBody)))]
async fn import(State(db): State<DatabaseConnection>, Body(b): Body<ImportBody>) -> Res<Json<Preset>> {
    let md = b.markdown.replace("\r\n", "\n");
    let (head, body) = md.strip_prefix("---\n").and_then(|r| r.split_once("\n---\n")).ok_or_else(|| Error::invalid("front matter (--- … ---) required".into()))?;
    let meta: HashMap<&str, &str> = head.lines().filter_map(|l| l.split_once(':')).map(|(k, v)| (k.trim(), v.trim())).collect();
    let (kind, key) = (meta.get("kind").copied().unwrap_or_default(), meta.get("key").copied().unwrap_or_default());
    let body = body.trim();
    if !KINDS.contains(&kind) || key.is_empty() || body.is_empty() {
        return Err(Error::invalid(format!("front matter kind in {KINDS:?} and key required, body non-empty")));
    }
    let limit = meta.get("limit_tok").map(|v| v.parse::<i64>().map_err(|_| Error::invalid("limit_tok must be a number".into()))).transpose()?;
    let out = event::run(&db, async |tx| {
        match ip::Entity::find().filter(ip::Column::Kind.eq(kind)).filter(ip::Column::PresetKey.eq(key)).one(tx).await? {
            Some(cur) => {
                let limit = limit.unwrap_or(cur.limit_tok);
                reject(verdict(tx, kind, Some(cur.sn), body, limit).await?)?;
                let (out, ev) = bump(tx, &cur, body, Some("import".into()), "en", "import", None, None, limit).await?;
                Ok((out, vec![ev]))
            }
            None => {
                let limit = limit.filter(|l| *l > 0).ok_or_else(|| Error::invalid("limit_tok required for a new preset".into()))?;
                reject(verdict(tx, kind, None, body, limit).await?)?;
                let name = meta.get("name").copied().unwrap_or(key);
                let (out, ev) = insert(tx, kind, key, name, None, limit, body, "en", "import", None).await?;
                Ok((out, vec![ev]))
            }
        }
    }).await?;
    Ok(Json(out))
}

/// 양식 1건 (form_key). 없으면 404
async fn form_of(db: &impl ConnectionTrait, key: &str) -> Res<rf::Model> {
    rf::Entity::find().filter(rf::Column::FormKey.eq(key)).order_by_asc(rf::Column::Sn).one(db).await?.ok_or_else(Error::not_found)
}

/// 보고서 양식 목록 (종류 → 번호순)
#[utoipa::path(operation_id = "preset_forms", get, path = "/report-forms", responses((status = 200, body = Vec<Form>), (status = "default", body = ErrorBody)))]
async fn forms(State(db): State<DatabaseConnection>) -> Res<Json<Vec<Form>>> {
    Ok(Json(rf::Entity::find().order_by_asc(rf::Column::Kind).order_by_asc(rf::Column::Sn).all(&db).await?.into_iter().map(Form::from).collect()))
}

/// 보고서 양식 1건. 없으면 404
#[utoipa::path(operation_id = "preset_form", get, path = "/report-forms/{key}", params(("key" = String, Path, description = "양식 키 (예: task-report)")), responses((status = 200, body = Form), (status = "default", body = ErrorBody)))]
async fn form(State(db): State<DatabaseConnection>, Path(key): Path<String>) -> Res<Json<Form>> {
    form_of(&db, &key).await.map(|m| Json(m.into()))
}

/// 보고서 양식 수정 (ReportFormUpdated · 버전 +1). 잠긴 칸이 본문에서 빠지면 422(빠진 칸 목록). 빈 본문 422, 없으면 404
#[utoipa::path(operation_id = "preset_set_form", put, path = "/report-forms/{key}", params(("key" = String, Path, description = "양식 키")), request_body = FormEdit, responses((status = 200, body = Form), (status = "default", body = ErrorBody)))]
async fn set_form(State(db): State<DatabaseConnection>, Path(key): Path<String>, Body(b): Body<FormEdit>) -> Res<Json<Form>> {
    if b.body.trim().is_empty() {
        return Err(Error::invalid("body is empty".into()));
    }
    let out = event::run(&db, async |tx| {
        let cur = form_of(tx, &key).await?;
        let gone: Vec<String> = Form::from(cur.clone()).locked.into_iter()
            .filter(|f| !b.body.contains(&format!("{{{{{f}}}}}")) && !b.body.contains(&format!("[[{f}]]"))).collect();
        if !gone.is_empty() {
            return Err(Error::invalid(format!("locked fields removed: {}", gone.join(", "))));
        }
        let mut u = rf::Entity::update_many().filter(rf::Column::Sn.eq(cur.sn)).col_expr(rf::Column::Body, b.body.clone().into())
            .col_expr(rf::Column::Version, (cur.version + 1).into()).col_expr(rf::Column::UpdateAt, Expr::cust("datetime('now')"));
        if let Some(v) = &b.name { u = u.col_expr(rf::Column::Name, v.clone().into()); }
        u.exec(tx).await?;
        let out = Form::from(form_of(tx, &key).await?);
        Ok((out, vec![Ev::new(None, "workspace", crate::WID, "ReportFormUpdated", &json!({ "form_key": key, "version": cur.version + 1 }))]))
    }).await?;
    Ok(Json(out))
}

/// 양식 자리표시 채우기: {{a.b}} · [[x]] 값, {{#each n}}…{{/each}}(항목마다 {{key}} · {{value}}), {{#if n}}…{{/if}}. 값이 없는 칸은 그대로 두고 missing에 넣는다
fn fill(tpl: &str, vals: &HashMap<String, String>, lists: &HashMap<String, Vec<(String, String)>>, missing: &mut Vec<String>) -> String {
    let mut out = String::new();
    let mut rest = tpl;
    while let Some(i) = rest.find(['{', '[']) {
        out.push_str(&rest[..i]);
        rest = &rest[i..];
        let (open, close) = if rest.starts_with("{{") { ("{{", "}}") } else if rest.starts_with("[[") { ("[[", "]]") } else {
            out.push_str(&rest[..1]);
            rest = &rest[1..];
            continue;
        };
        let Some(end) = rest.find(close) else { break };
        let tag = rest[open.len()..end].trim().to_owned();
        rest = &rest[end + close.len()..];
        if let Some(name) = tag.strip_prefix("#each ").or(tag.strip_prefix("#if ")).map(str::trim) {
            let each = tag.starts_with("#each");
            let stop = if each { "{{/each}}" } else { "{{/if}}" };
            let (inner, after) = rest.split_once(stop).unwrap_or((rest, ""));
            rest = after;
            if each {
                for (k, v) in lists.get(name).into_iter().flatten() {
                    let mut item = vals.clone();
                    item.insert("key".into(), k.clone());
                    item.insert("value".into(), v.clone());
                    out.push_str(&fill(inner, &item, lists, missing));
                }
            } else if vals.get(name).is_some_and(|v| !v.is_empty()) {
                out.push_str(&fill(inner, vals, lists, missing));
            }
            continue;
        }
        match vals.get(&tag) {
            Some(v) => out.push_str(v),
            None => {
                out.push_str(&format!("{open}{tag}{close}"));
                if !missing.contains(&tag) { missing.push(tag); }
            }
        }
    }
    out.push_str(rest);
    out
}

/// 미리보기: 태스크의 마지막 리드 Run 보고 항목 · 변경 파일 · 토큰으로 양식을 조립한다 (모델 호출 없음). 양식 · 태스크가 없으면 404
#[utoipa::path(operation_id = "preset_preview", post, path = "/report-forms/{key}/preview", params(("key" = String, Path, description = "양식 키"), PreviewQuery), responses((status = 200, body = Preview), (status = "default", body = ErrorBody)))]
async fn preview(State(db): State<DatabaseConnection>, Path(key): Path<String>, Query(q): Query<PreviewQuery>) -> Res<Json<Preview>> {
    let f = form_of(&db, &key).await?;
    let task = t::Entity::find_by_id(q.task).one(&db).await?.ok_or_else(Error::not_found)?;
    let mut vals: HashMap<String, String> = [
        ("task.num", task.num.to_string()), ("task.title", task.title.clone()), ("status.label", task.status.clone()), ("scope.commits", task.commit_count.to_string()),
    ].into_iter().map(|(k, v)| (k.into(), v)).collect();
    if let Some(n) = task.pr_number {
        vals.extend([("pr".into(), "1".into()), ("pr.num".into(), n.to_string()), ("pr.status".into(), task.pr_status.clone().unwrap_or_default())]);
    }
    let mut lists: HashMap<String, Vec<(String, String)>> = HashMap::new();
    let lead = r::Entity::find().filter(r::Column::TaskSn.eq(task.sn)).filter(r::Column::ParentRunSn.is_null()).order_by_desc(r::Column::Num).one(&db).await?;
    if let Some(run) = lead {
        if let Some(s) = &run.result_summary { vals.insert("tests.passed_summary".into(), s.clone()); }
        let files = fl::Entity::find().filter(fl::Column::RunSn.eq(run.sn)).all(&db).await?;
        vals.extend([("scope.files".into(), files.len().to_string()), ("scope.add".into(), files.iter().map(|x| x.additions).sum::<i64>().to_string()),
            ("scope.del".into(), files.iter().map(|x| x.deletions).sum::<i64>().to_string())]);
        if let Some(Value::Object(tot)) = serde_json::to_value(run::enrich(&db, vec![run::Run::from(run.clone())]).await?.pop()).ok().map(|v| v["runner_total"].clone()) {
            vals.insert("run.tokens".into(), tot.get("value").map(Value::to_string).unwrap_or_default());
        }
        let dur = db.query_one_raw(sea_orm::Statement::from_sql_and_values(sea_orm::DbBackend::Sqlite,
            "SELECT CAST((julianday(end_at) - julianday(start_at)) * 1440 AS INTEGER) FROM tbl_run WHERE sn = ? AND start_at IS NOT NULL AND end_at IS NOT NULL", vec![run.sn.into()])).await?
            .and_then(|r| r.try_get_by_index::<Option<i64>>(0).ok().flatten());
        if let Some(m) = dur { vals.insert("run.duration".into(), format!("{m}m")); }
        for it in ri::Entity::find().filter(ri::Column::RunSn.eq(run.sn)).order_by_asc(ri::Column::Sn).all(&db).await? {
            let v = it.value.clone().unwrap_or_default();
            match it.kind.as_str() {
                "result" | "unverified" => { vals.insert(it.kind.clone(), v); }
                "area" | "review" | "left" => lists.entry(it.kind.clone()).or_default().push((it.ref_key.clone().unwrap_or(it.code.clone()), v)),
                _ => {}
            }
        }
    }
    let mut missing = Vec::new();
    let body = fill(&f.body, &vals, &lists, &mut missing);
    Ok(Json(Preview { body, missing }))
}
