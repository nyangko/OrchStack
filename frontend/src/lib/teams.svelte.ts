/// Teams 목상태 — /teams(팀 · 멤버)와 /teams/agents(템플릿)가 같은 데이터를 본다.
/// 서버 API(#45) 연결 전 임시 저장소. 연결 후에는 load 결과로 바뀐다.
import type { Component } from "svelte";
import Sparkles from "@lucide/svelte/icons/sparkles";
import Workflow from "@lucide/svelte/icons/workflow";
import Bot from "@lucide/svelte/icons/bot";
import Server from "@lucide/svelte/icons/server";
import Database from "@lucide/svelte/icons/database";
import Terminal from "@lucide/svelte/icons/terminal";
import Monitor from "@lucide/svelte/icons/monitor";
import Code from "@lucide/svelte/icons/code";
import LayoutGrid from "@lucide/svelte/icons/layout-grid";
import FlaskConical from "@lucide/svelte/icons/flask-conical";
import Bug from "@lucide/svelte/icons/bug";
import ListChecks from "@lucide/svelte/icons/list-checks";
import ShieldCheck from "@lucide/svelte/icons/shield-check";
import Globe from "@lucide/svelte/icons/globe";
import GitFork from "@lucide/svelte/icons/git-fork";
import Folder from "@lucide/svelte/icons/folder";
import Store from "@lucide/svelte/icons/store";
import Package from "@lucide/svelte/icons/package";
import Eye from "@lucide/svelte/icons/eye";
import GitPullRequest from "@lucide/svelte/icons/git-pull-request";
import Palette from "@lucide/svelte/icons/palette";
import PenTool from "@lucide/svelte/icons/pen-tool";
import Shapes from "@lucide/svelte/icons/shapes";
import type { Role } from "$lib/roles";
import { useMock } from "$lib/api/env";
import { api, failureOf } from "$lib/api/client";
import { roleOf } from "$lib/project.svelte";
import type { ApiMember, ApiProject, ApiTask } from "$lib/api/types";
import { ago } from "$lib/time";
import { statuses, type TaskStatus } from "$lib/status";
import type { Runtime } from "$lib/components/orch/runtime-logo";
import type { MdFile } from "$lib/components/orch/md-editor";
import { projects, teams, templates, orchPolicy, skillLibrary, skillSources, skillLog, accounts, mcpServers, teamPolicy, type AgentConfig, type OrchPolicy, type TeamMember, type Team, type SpawnMode, type Task, type ProjectTab, type Template, type Skill, type SkillHit } from "$lib/mock";

export const store = $state({
	/// 프로젝트 탭 (새 프로젝트를 만들면 늘어난다).
	projects: useMock ? structuredClone(projects) : ([] as typeof projects),
	crew: structuredClone(teams),
	templates: structuredClone(templates),
	/// 팀별 Orch 진행 정책.
	policies: Object.fromEntries(teams.map((t) => [t.sn, structuredClone(orchPolicy)])) as Record<number, OrchPolicy>,
	/// 워크스페이스 스킬 라이브러리 (skills.sh에서 추가하면 늘어난다).
	library: structuredClone(skillLibrary),
	sources: structuredClone(skillSources),
	/// 스킬 · MCP 변경 기록 (최근이 앞).
	log: structuredClone(skillLog),
	/// 워크스페이스 MCP 서버 상태 — installed: 설치됨, auth: 인증 필요.
	mcp: Object.fromEntries(mcpServers.map((m) => [m.name, { installed: ["playwright-mcp", "github-mcp"].includes(m.name), auth: !!m.auth }])) as Record<string, { installed: boolean; auth: boolean }>,
});

/// 스킬 출처 표시 (.pen 소스 목록).
export const sourceMeta: Record<Skill["source"], { label: string; icon: Component }> = {
	Team: { label: "orchstack/team-skills", icon: GitFork },
	"skills.sh": { label: "skills.sh", icon: Globe },
	Local: { label: "~/.orch/skills", icon: Folder },
	Marketplace: { label: "Marketplace", icon: Store },
	"Built-in": { label: "Built-in", icon: Package },
};

/// skills.sh에서 워크스페이스 라이브러리로 설치한다 (이미 있으면 그대로).
export function installSkill(h: SkillHit) {
	if (store.library.some((x) => x.name === h.name)) return;
	store.library.push({ name: h.name, desc: h.desc, source: h.repo.startsWith("orchstack/") ? "Team" : "skills.sh", version: h.version.split(" ")[0], tok: h.tok });
	store.log.unshift({ kind: "INSTALL", who: "나", when: "방금", text: `${h.name} ${h.version.split(" ")[0]} 설치 · ${h.repo} · 보안 검사 ${h.audit[0]}/${h.audit[1]}` });
}

/// 기본 팀 — 첫 프로젝트 팀 (Orch 팀 제외).
export const defaultTeam = () => store.crew.find((t) => !t.orch)!;

export const runtimeName = (r: Runtime) => (r === "claude" ? "Claude Code" : "Codex CLI");
export const accountOf = (r: Runtime) => accounts.find((a) => a.runtime === r)!;
/// 잔량 경고 기준 미만인지.
export const low = (pct: number) => pct < teamPolicy.quotaWarn;
export const scopeText = (c: AgentConfig) => c.include.join(", ") || "읽기 전용";
export const liveMap = (files: MdFile[]) => Object.fromEntries(files.map((f) => [f.name, f.body]));
export const k = (n: number) => `${n.toFixed(1)}K`;

/// 이 템플릿으로 만든 멤버 (모든 팀).
export const membersOf = (name: string) => store.crew.flatMap((t) => t.members.filter((m) => m.title === name).map((m) => ({ m, team: t.name })));

/// 멤버가 고를 수 있는 아바타 아이콘 (역할 색은 유지). 0번이 역할 기본.
export const glyphs: Record<Role, Component[]> = {
	orch: [Sparkles, Workflow, Bot],
	backend: [Server, Database, Terminal],
	frontend: [Monitor, Code, LayoutGrid],
	qa: [FlaskConical, Bug, ListChecks],
	reviewer: [ShieldCheck, Eye, GitPullRequest],
	designer: [Palette, PenTool, Shapes],
	agent: [Bot, Sparkles, Code],
};
export const glyphOf = (m: TeamMember) => (m.glyph ? glyphs[m.role][m.glyph] : undefined);

const skillTok = (name: string) => store.library.find((x) => x.name === name)?.tok ?? mcpServers.find((m) => m.name === name)?.tok ?? 0;
/// 켜진 스킬 · MCP가 Run마다 더하는 토큰.
export const cfgTok = (c: AgentConfig) => [...c.skills, ...c.mcp].reduce((n, x) => n + skillTok(x), 0);

/// 템플릿 대비 바뀐 설정 항목 수.
export const cfgDiff = (c?: AgentConfig, b?: AgentConfig) =>
	!c || !b
		? 0
		: [
				c.skills.join() !== b.skills.join(),
				c.mcp.join() !== b.mcp.join(),
				c.trust !== b.trust,
				c.include.join() !== b.include.join() || c.exclude.join() !== b.exclude.join(),
				JSON.stringify(c.approvals) !== JSON.stringify(b.approvals),
				c.github !== b.github,
			].filter(Boolean).length;

export function toggleIn(list: string[], name: string) {
	const i = list.indexOf(name);
	if (i >= 0) list.splice(i, 1);
	else list.push(name);
}

/// 템플릿 다음 버전 초안에 파일을 넣는다 (템플릿 편집 저장 · 멤버의 제안).
export function putDraft(t: Template, files: MdFile[], who: string, note: string) {
	t.draft = structuredClone(files);
	if (!t.revisions.some((r) => r.state === "draft")) t.revisions.unshift({ v: Math.max(...t.revisions.map((r) => r.v)) + 1, state: "draft", who, when: "방금", note });
}

// ---- 서버 데이터 (A-2 #93). 화면은 목데이터 모양을 그대로 쓰고, 서버 행을 여기서 맞춘다.
// 서버에 아직 없는 값(작업량 · 컨텍스트 · 토큰 · KPI #88, 런타임 이름 #47, 지침 파일 · 리비전 #45)은 빈 값으로 둔다 (#60).

/// 불러오기 상태 — 목데이터 모드면 처음부터 ready.
export const teamsLoad = $state({ state: (useMock ? "ready" : "idle") as "idle" | "loading" | "ready" | "error" });

const emptyStats: Team["stats"] = { open: 0, openNote: "—", done: 0, doneDelta: "", doneNote: "—", doneTrend: [], tokenTrend: [], cycle: "—", cycleDelta: "", cycleTrend: [] };
const memberStatus = (s: string): TeamMember["status"] => (s === "running" || s === "waiting" ? s : "idle");

function memberView(m: ApiMember): TeamMember {
	return {
		sn: m.sn, name: m.name, role: roleOf(m), title: m.role_name, runtime: "claude", model: "—", status: memberStatus(m.status),
		work: "—", context: 0, tokens: 0, load: [], loadNote: "—"
	};
}

async function membersOfTeam(sn: number): Promise<TeamMember[]> {
	const { data } = await api.GET("/teams/{sn}/members", { params: { path: { sn } } });
	return (data ?? []).filter((m) => m.status !== "archived").map(memberView);
}

/// 팀 · 멤버 · 프로젝트 연결을 서버에서 읽는다. 하위 작업 정책(spawn)은 팀 행에서, Orch 진행 레벨 · 가드는 아직 서버에 없어 화면 기본값.
export async function loadTeams() {
	if (useMock || teamsLoad.state === "loading") return;
	teamsLoad.state = "loading";
	const [{ data: rows }, { data: projs }] = await Promise.all([api.GET("/teams"), api.GET("/projects")]);
	if (!rows) return void (teamsLoad.state = "error");
	const members = await Promise.all(rows.map((t) => membersOfTeam(t.sn)));
	store.crew = rows.map((t, i) => ({
		sn: t.sn, name: t.name, orch: t.kind === "orch", project: projs?.find((p) => p.team_sn === t.sn)?.name, desc: "", members: members[i], stats: emptyStats
	}));
	for (const t of rows) {
		store.policies[t.sn] ??= structuredClone(orchPolicy);
		store.policies[t.sn].spawn = { mode: t.spawn_mode as SpawnMode, allow: t.spawn_allow.split(",") as SpawnMode[], maxChild: t.max_child_run };
	}
	teamsLoad.state = "ready";
}

/// 멤버 추가 — 성공하면 그 팀 멤버를 다시 읽고 새 멤버 sn을 돌려준다. 실패 문구는 클라이언트가 토스트로 띄운다.
export async function addMember(teamSn: number, body: { name: string; template_sn?: number | null; role_name?: string; icon?: string; first_task_mode?: string }): Promise<number | undefined> {
	// 연결 실패는 클라이언트가 토스트로 알린다 — 여기서는 저장 안 됨(undefined)으로만 돌려준다.
	const res = await api.POST("/teams/{sn}/members", { params: { path: { sn: teamSn } }, body }).catch(() => undefined);
	if (!res || res.error) return;
	const data = res.data;
	const team = store.crew.find((t) => t.sn === teamSn);
	const before = new Set(team?.members.map((m) => m.sn));
	if (team) team.members = await membersOfTeam(teamSn);
	return (data as { sn?: number } | undefined)?.sn ?? team?.members.find((m) => !before.has(m.sn))?.sn;
}

/// 팀 하위 작업 정책 저장 (PATCH /teams/{sn}). 서버 검증(기본 ⊂ 허용 · 동시 ≥ 1)에 걸리면 그 문구를 돌려준다.
export async function saveTeamSpawn(teamSn: number, spawn: OrchPolicy["spawn"]): Promise<string | undefined> {
	const res = await api.PATCH("/teams/{sn}", { params: { path: { sn: teamSn } }, body: { spawn_mode: spawn.mode, spawn_allow: spawn.allow.join(","), max_child_run: Number(spawn.maxChild) } }).catch(() => undefined);
	if (!res) return "서버에 연결할 수 없어요";
	return res.error ? failureOf(res.error).message : undefined;
}

/// 프로젝트 행 → 탭. 탭 · All Projects · Workbench · Tasks가 같은 목록(store.projects)을 본다.
export const projectTab = (p: ApiProject): ProjectTab => ({ sn: p.sn, name: p.name, status: p.status, dot: p.status === "active" ? "bg-success" : "bg-subtle-foreground" });

/// 서버 모드 프로젝트 목록. 실패하면 false (토스트는 클라이언트).
export async function loadProjects(): Promise<boolean> {
	if (useMock) return true;
	const res = await api.GET("/projects").catch(() => undefined);
	if (!res?.data) return false;
	store.projects = res.data.map(projectTab);
	return true;
}

/// 모든 프로젝트의 태스크 (A-3 #94) — 전체 목록 API(#89)가 없어 프로젝트별 목록을 합친다. 이슈 번호 · 진행 · 메시지 수는 아직 빈 값.
export async function loadAllTasks(): Promise<(Task & { project: number })[] | undefined> {
	const view = (t: ApiTask): Task & { project: number } => ({
		sn: t.sn, num: t.num, project: t.project_sn, title: t.title, description: t.description,
		status: (t.status in statuses ? t.status : "todo") as TaskStatus,
		priority: `P${Math.min(3, Math.max(0, t.priority))}` as Task["priority"],
		agent: t.member_sn ?? undefined, issue: 0, steps: [0, 0], messages: 0, updated: ago(t.update_at)
	});
	const lists = await Promise.all(store.projects.map((p) => api.GET("/projects/{sn}/tasks", { params: { path: { sn: p.sn } } }).then((r) => r.data).catch(() => undefined)));
	if (lists.some((l) => !l)) return;
	return lists.flatMap((l) => (l ?? []).map(view));
}
