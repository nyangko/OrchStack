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
import type { Runtime } from "$lib/components/ui/runtime-logo";
import type { MdFile } from "$lib/components/ui/md-editor";
import { projects, teams, templates, orchPolicy, skillLibrary, skillSources, skillLog, accounts, mcpServers, teamPolicy, type AgentConfig, type OrchPolicy, type TeamMember, type Template, type Skill, type SkillHit } from "$lib/mock";

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
