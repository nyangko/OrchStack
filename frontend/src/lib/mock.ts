/// 서버 API(#59 W-8) 연결 전 목데이터. 타입은 OpenAPI 스키마에서 가져와 교체 시 호출부가 바뀌지 않게 한다.
import type { ApiProject, ApiIssue, ApiTask, ApiTeam, ApiMember, ApiTemplate } from "$lib/api/types";

/// 프로젝트 탭 표시용. dot은 진행 상태(실행 중 · 대기 · 쉼)를 나타내는 계산값이라 스키마에 없다.
export type ProjectTab = Pick<ApiProject, "sn" | "name" | "status"> & { dot: string };

export const projects: ProjectTab[] = [
	{ sn: 1, name: "OrchStack", status: "active", dot: "bg-success" },
	{ sn: 2, name: "Web Platform", status: "active", dot: "bg-status-waiting" },
	{ sn: 3, name: "Mobile App", status: "active", dot: "bg-subtle-foreground" },
];

/// 새 프로젝트에 연결할 수 있는 저장소 (GitHub App이 설치된 곳). issues = 열린 이슈 수.
export const repos = [
	{ name: "orchstack/checkout", meta: "main · 비공개", issues: 12 },
	{ name: "orchstack/app", meta: "main · 비공개", issues: 12 },
	{ name: "orchstack/web", meta: "main · 공개", issues: 4 },
	{ name: "orchstack/mobile", meta: "develop · 비공개", issues: 7 },
];

import type { TaskStatus } from "$lib/status";
import type { Role } from "$lib/roles";
import type { Runtime } from "$lib/components/orch/runtime-logo";

/// 에이전트(멤버). 스키마 tbl_member · tbl_agent_profile 요약. OpenAPI(#45) 전 임시 타입.
export type Agent = {
	sn: number;
	name: string;
	role: Role;
	runtime: Runtime;
	/** 지금 하는 일 한 줄. */
	activity: string;
	online: boolean;
	tokens: string;
};

/// 태스크. 서버 Task와 겹치는 필드(num · title)는 스키마에서, 나머지는 화면 계산값. 페이지별 교체(A-1~)에서 sn · member_sn 등으로 맞춘다.
export type Task = Pick<ApiTask, "num" | "title"> & Partial<Pick<ApiTask, "sn" | "description">> & {
	/** 프로젝트 sn. 없으면 1 (OrchStack). */
	project?: number;
	status: TaskStatus;
	priority: "P0" | "P1" | "P2" | "P3";
	/** 하위 작업 방식 (tbl_task.spawn_mode). 없으면 팀 기본값 (#67). */
	spawnMode?: SpawnMode;
	agent?: number;
	effort?: "High" | "Medium" | "Low";
	/** 상위 이슈 번호. */
	issue: number;
	/** 완료 조건(Criterion) 완료 수 / 전체. */
	steps: [number, number];
	tokens?: string;
	/** 토큰 예산 초과 경고. */
	over?: boolean;
	messages: number;
	model?: string;
	/** 진행 중 Run 경과. */
	run?: string;
	/** 마지막 변경. */
	updated: string;
};

/// 이슈. 스키마 tbl_issue 요약 (상태: open · in_progress · done · closed). parent가 있으면 하위 이슈.
export type Issue = Pick<ApiIssue, "num" | "title"> & {
	status: "open" | "in_progress" | "done" | "closed";
	parent?: number;
	updated: string;
	description: string;
	labels: string[];
};

export const issues: Issue[] = [
	{ num: 51, title: "Authentication Flow 개선", status: "in_progress", updated: "1m ago", description: "로그인 · 토큰 갱신 · 세션 만료까지 인증 흐름 전체를 개선한다. refresh token을 도입하고 QA · 리뷰까지 마친다.", labels: ["auth", "frontend", "backend"] },
	{ num: 52, title: "Auth Backend · Token 갱신", status: "in_progress", parent: 51, updated: "1m ago", description: "access token 만료 시 refresh token으로 재발급한다. 토큰 회전 방식과 저장 위치를 정한다.", labels: ["auth", "backend"] },
	{ num: 53, title: "Login 화면", status: "in_progress", parent: 51, updated: "4m ago", description: "로그인 화면과 오류 · 재시도 상태를 구현한다. 백엔드 #52 API에 맞춘다.", labels: ["auth", "frontend"] },
	{ num: 48, title: "Onboarding 개선", status: "in_progress", updated: "18m ago", description: "첫 실행 온보딩 화면과 오류 상태 일러스트를 정리한다.", labels: ["design", "onboarding"] },
	{ num: 45, title: "Rate limiting", status: "in_progress", updated: "1h ago", description: "API rate limiting 미들웨어를 추가한다. IP · 사용자 단위 제한을 둔다.", labels: ["backend", "security"] },
	{ num: 42, title: "Design system 정리", status: "open", updated: "3h ago", description: "색 · 타이포 · 간격 토큰을 정리하고 컴포넌트에 반영한다.", labels: ["design"] },
	{ num: 39, title: "Auth DB 설계", status: "done", updated: "1d ago", description: "인증 테이블(사용자 · 세션 · 토큰) 스키마를 설계한다.", labels: ["backend", "db"] },
	{ num: 40, title: "Legacy cleanup", status: "closed", updated: "2d ago", description: "사용하지 않는 레거시 로그인 코드를 제거한다.", labels: ["cleanup"] },
];

/// Ops 패널 한 줄. 스키마 tbl_log_run 요약.
export type LogLine = { time: string; source: string; role: Role; message: string };

export const agents: Agent[] = [
	{ sn: 1, name: "진", role: "frontend", runtime: "codex", activity: "Running · #129 Login UI", online: true, tokens: "24.7K" },
	{ sn: 2, name: "민수", role: "backend", runtime: "claude", activity: "Running · #128 Auth API", online: true, tokens: "118K" },
	{ sn: 3, name: "하린", role: "qa", runtime: "claude", activity: "Waiting · #130 검증 요청 대기", online: true, tokens: "1.2K" },
	{ sn: 4, name: "소라", role: "reviewer", runtime: "claude", activity: "Idle", online: false, tokens: "2.1K" },
	{ sn: 5, name: "유나", role: "designer", runtime: "codex", activity: "Running · #135 Error state", online: true, tokens: "9.8K" },
];

// .pen #26 Workbench / Kanban Board · Issue Board 내용과 맞춘다.
export const tasks: Task[] = [
	{ num: 131, title: "Review · Auth", status: "backlog", priority: "P2", agent: 4, effort: "Medium", issue: 53, steps: [0, 2], tokens: "2.1K", messages: 1, model: "claude", updated: "12m ago" },
	{ num: 133, title: "Password reset flow", status: "backlog", priority: "P3", issue: 51, steps: [0, 0], messages: 0, updated: "2h ago" },
	{ num: 127, title: "Design token 정리", status: "todo", priority: "P2", agent: 5, effort: "Low", issue: 42, steps: [0, 3], messages: 2, model: "gpt-5", updated: "3h ago" },
	{ num: 132, title: "Session expiry toast", status: "todo", priority: "P3", issue: 51, steps: [0, 0], messages: 0, updated: "2h ago" },
	{ num: 129, title: "Login UI 구현", status: "in_progress", priority: "P1", agent: 1, effort: "High", issue: 53, steps: [3, 5], tokens: "24.7K", messages: 8, model: "gpt-5", run: "15m", updated: "now" },
	{ num: 128, title: "Auth API (refresh token)", status: "in_progress", priority: "P1", agent: 2, effort: "High", issue: 52, steps: [4, 6], tokens: "118K", over: true, messages: 12, model: "claude", run: "42m", updated: "1m ago" },
	{ num: 135, title: "Error state 일러스트", status: "in_progress", priority: "P2", agent: 5, effort: "Medium", issue: 48, steps: [1, 3], tokens: "9.8K", messages: 3, model: "gpt-5", run: "6m", updated: "6m ago" },
	{ num: 130, title: "QA · Authentication flow", status: "waiting", priority: "P1", agent: 3, effort: "Medium", issue: 53, steps: [0, 4], tokens: "1.2K", messages: 2, model: "claude", updated: "4m ago" },
	{ num: 124, title: "Rate limit middleware", status: "blocked", priority: "P0", agent: 2, effort: "High", issue: 45, steps: [1, 4], tokens: "22.5K", messages: 6, model: "claude", updated: "1h ago" },
	{ num: 126, title: "Onboarding 화면 시안", status: "review", priority: "P2", agent: 5, effort: "Low", issue: 48, steps: [5, 5], tokens: "15.0K", messages: 9, model: "gpt-5", updated: "18m ago" },
	{ num: 122, title: "Auth DB schema", status: "done", priority: "P1", agent: 2, effort: "Medium", issue: 39, steps: [6, 6], tokens: "33.4K", messages: 14, model: "claude", updated: "1d ago" },
	{ num: 119, title: "Legacy login 제거", status: "cancelled", priority: "P3", issue: 40, steps: [0, 0], messages: 1, updated: "2d ago" },
];

export const logs: LogLine[] = [
	{ time: "22:54:12", source: "민수", role: "backend", message: "read src/core/token_ledger.rs" },
	{ time: "22:54:15", source: "민수", role: "backend", message: "edit src/core/token_ledger.rs (+42 −8)" },
	{ time: "22:54:21", source: "진", role: "frontend", message: "run pnpm check → 0 errors" },
	{ time: "22:54:30", source: "Orch", role: "orch", message: "ASSIGN #130 → 하린 (검증 요청)" },
	{ time: "22:54:41", source: "진", role: "frontend", message: "edit src/routes/login/+page.svelte (+88)" },
	{ time: "22:54:52", source: "민수", role: "backend", message: "run cargo test → 12 passed" },
];

/// 태스크 상세 (.pen TaskDetailView). 스키마 tbl_task · tbl_task_criterion · tbl_map_task_dependency · tbl_run · tbl_log_activity 요약.
export type TaskDetail = {
	description: string;
	criteria: { text: string; done: boolean }[];
	deps: { kind: "depends" | "blocks"; num: number }[];
	runs: { num: number; note: string; time: string; tokens: string; live?: boolean }[];
	activity: { type: string; who: string; time: string; text: string }[];
	eta: string;
	labels: string[];
	/** 현재 Run 컨텍스트 사용 / 한도 (K 토큰). */
	context?: [number, number];
	git?: { branch: string; commits: string; pr?: string };
	/** 판단 대기 (L2+). */
	decision?: { question: string; note: string; level: string };
};

export const taskDetails: Record<number, TaskDetail> = {
	// .pen KanbanCard/HoverPreview 예시 (#128)
	128: {
		description: "access token 만료 시 refresh token으로 재발급하는 Auth API. 토큰 회전 · 재사용 감지 · 마이그레이션을 포함합니다.",
		criteria: [
			{ text: "refresh 엔드포인트", done: true },
			{ text: "토큰 저장 테이블 마이그레이션", done: true },
			{ text: "만료 · 서명 검증", done: true },
			{ text: "재사용 감지 시 세션 폐기", done: true },
			{ text: "token rotation 구현", done: false },
			{ text: "통합 테스트", done: false },
		],
		deps: [{ kind: "blocks", num: 130 }],
		runs: [{ num: 79, note: "진행 중 · 4/6 단계", time: "42m", tokens: "118K", live: true }],
		activity: [
			{ type: "ASSIGN", who: "Orch → 민수", time: "13:40", text: "#128 배정" },
			{ type: "TOOL_CALL", who: "민수", time: "14:16", text: "run_migration · 1m" },
		],
		eta: "~1h 20m",
		labels: ["auth", "backend"],
		context: [118, 128],
	},
	129: {
		description:
			"Issue #51 Authentication Flow 개선의 로그인 화면을 구현합니다. #128 Auth API(refresh token) 스펙을 기준으로 이메일/비밀번호 로그인, 에러·로딩 상태, 세션 만료 처리를 포함합니다. 시안: 유나의 Login v2 (첨부).",
		criteria: [
			{ text: "로그인 폼 · 유효성 검사", done: true },
			{ text: "에러 / 로딩 상태", done: true },
			{ text: "refresh token 만료 시 재로그인 유도", done: false },
		],
		deps: [
			{ kind: "depends", num: 128 },
			{ kind: "blocks", num: 130 },
		],
		runs: [
			{ num: 81, note: "진행 중 · 3/5 단계", time: "15m", tokens: "18.4K", live: true },
			{ num: 77, note: "lint error 3건 → Run #81 재시도", time: "4m", tokens: "6.3K" },
		],
		activity: [
			{ type: "TASK_INSTRUCTION", who: "Orch → 진", time: "13:58", text: "Task #129 생성 · WorkProposal 'Authentication 개선'에서 Proceed" },
			{ type: "ASSIGN", who: "Orch → 진", time: "14:01", text: "#129 배정 · 유나의 Login 시안 v2 첨부" },
			{ type: "DECISION", who: "나", time: "14:12", text: "Q 로그인 실패 5회 시 잠금? → 15분 잠금 + 안내" },
			{ type: "TEST", who: "pnpm test auth", time: "14:16", text: "✓ 12 passed · 0 failed" },
			{ type: "REQUEST_VERIFICATION", who: "진 → 하린 · live", time: "14:17", text: "Task #130 QA 대기 해제 요청" },
		],
		eta: "~2h (14:15 → 16:30)",
		labels: ["auth", "ui", "frontend"],
		context: [41.2, 128],
		git: { branch: "feat/login-ui", commits: "3 · +310 −15", pr: "#88 · Draft" },
		decision: { question: "진이 묻고 있어요 · 로그인 실패 에러 문구 톤", note: "판단 대기 1 · L2 · 9:42 후 Orch가 추천안(A)으로 결정", level: "L2" },
	},
};

/// 에이전트 활동 기록 (.pen Agent Inspector · Activity). 스키마 tbl_log_activity · tbl_interaction 요약.
export const agentActivity: Record<number, { type: string; who: string; time: string; text: string }[]> = {
	1: [
		{ type: "TASK_INSTRUCTION", who: "Orch → 진", time: "13:58", text: "Task #129 Login UI 구현. #128 Auth API 스펙 기준, 에러/로딩 상태 포함." },
		{ type: "ASSIGN", who: "Orch → 진", time: "14:01", text: "#129 배정 · 유나의 Login 시안 v2 첨부" },
		{ type: "RUN", who: "Run #81 · Session #12", time: "14:02", text: "Run started · Codex CLI · gpt-5" },
		{ type: "TOOL_CALL", who: "edit_file", time: "14:09", text: "src/routes/login/+page.svelte  +142 −8" },
		{ type: "MESSAGE", who: "진", time: "14:15", text: "로그인 폼, 에러/로딩 상태 구현 완료. 검증 요청 보냅니다." },
		{ type: "TEST", who: "pnpm test auth", time: "14:16", text: "✓ 12 passed · 0 failed" },
		{ type: "REQUEST_VERIFICATION", who: "진 → 하린 · live", time: "14:17", text: "Task #130 QA 대기 해제 요청" },
	],
};

/// 판단 대기 (.pen DecisionPanel). 스키마 tbl_decision · tbl_decision_question · tbl_decision_option 요약.
export type Decision = {
	id: number;
	agent: number;
	task: number;
	title: string;
	/** 대기 목록 한 줄 요약. */
	topic: string;
	sub: string;
	/** 남은 시간 (없으면 결정됨). 시간이 지나면 Orch가 추천안으로 결정. */
	left?: string;
	decided?: string;
	questions: {
		q: string;
		context?: string;
		options?: { key: string; title: string; desc?: string; rec?: boolean }[];
		answer?: string;
	}[];
};

export const decisions: Decision[] = [
	{
		id: 1, agent: 1, task: 129, title: "Login UI 구현", topic: "에러 문구 톤 · 재시도 제한 · 안내 문구", sub: "질문 3개 · Run #81 대기 중 · 그동안 #135 참고", left: "9:42",
		questions: [
			{ q: "로그인 실패 에러 문구 톤", answer: "친근하게 · 원인 + 다음 행동" },
			{
				q: "재시도 제한",
				context: "현재 구현은 로그인 재시도에 제한이 없어요. 무차별 대입 위험이 있고, 백엔드(#128)에는 rate limit이 IP 단위로만 있어요. 사용자 단위 제한을 프론트에서 둘지 정해 주세요.",
				options: [
					{ key: "A", title: "제한 없음", desc: "구현 간단 · 무차별 대입 위험" },
					{ key: "B", title: "60초 쿨다운 표시", desc: "버튼 비활성 + 남은 시간 · 백엔드 변경 없음", rec: true },
					{ key: "C", title: "시간당 5회 제한", desc: "민수(#128)와 API 변경 필요" },
				],
			},
			{ q: "제한 안내 문구" },
		],
	},
	{
		id: 2, agent: 2, task: 128, title: "Auth API (refresh token)", topic: "refresh 토큰 회전 방식", sub: "질문 1개 · Run #80 진행 중", left: "23:10",
		questions: [{ q: "refresh 토큰 회전 방식", options: [{ key: "A", title: "매 요청 회전", desc: "보안 ↑ · 동시 요청 충돌 처리 필요" }, { key: "B", title: "만료 임박 시 회전", desc: "구현 간단", rec: true }] }],
	},
	{
		id: 3, agent: 3, task: 130, title: "QA · Authentication flow", topic: "E2E 브라우저 범위", sub: "질문 1개 · #128 #129 대기 중", left: "41:02",
		questions: [{ q: "E2E 브라우저 범위", options: [{ key: "A", title: "Chromium만" }, { key: "B", title: "Chromium + WebKit", rec: true }, { key: "C", title: "3종 전부", desc: "시간 2배" }] }],
	},
	{
		id: 4, agent: 4, task: 126, title: "Onboarding 화면 시안", topic: "a11y 기준 확정", sub: "질문 1개", decided: "12분 전",
		questions: [{ q: "a11y 기준", answer: "WCAG 2.2 AA" }],
	},
];

/// PM Dock 대화 (.pen ProjectPMChatDock). 종류별로 말풍선 · 작업 제안 · 명령 결과를 그린다.
export type Chat =
	| { kind: "user"; text: string; time: string }
	| { kind: "orch"; text: string; time: string }
	| { kind: "proposal"; title: string; issue: string; tasks: { title: string; agent: number }[]; deps: string; status: "draft" | "proceeded" | "cancelled" }
	| { kind: "result"; time: string; lines: string[] };

export const thread: Chat[] = [
	{ kind: "user", text: "로그인 쪽 인증 흐름 전체 개선해줘. refresh token 도입하고 QA · 리뷰까지.", time: "13:52" },
	{ kind: "orch", text: "이슈 1개 · 태스크 4개로 나눠봤어요. 확인 후 진행을 눌러 주세요.", time: "13:53" },
	{
		kind: "proposal", title: "Authentication 개선", issue: "Authentication Flow 개선",
		tasks: [{ title: "Auth API", agent: 2 }, { title: "Login UI", agent: 1 }, { title: "QA", agent: 3 }, { title: "Review", agent: 4 }],
		deps: "QA ← Auth API + Login UI · Review ← QA", status: "draft",
	},
];

/// 팀 멤버 한 줄 (.pen Teams / MemberRow · WorkloadRow). 스키마 tbl_member · tbl_team_member · tbl_run 요약. OpenAPI(#45) 전 임시 타입.
export type TeamMember = Pick<ApiMember, "sn" | "name"> & {
	role: Role;
	/** 역할 표시 이름 (예: Backend Developer). */
	title: string;
	runtime: Runtime;
	model: string;
	status: "running" | "waiting" | "idle";
	/** 현재 작업 한 줄. next면 다음 배정 예정. */
	work: string;
	next?: boolean;
	/** 컨텍스트 사용률 %. */
	context: number;
	/** 오늘 토큰 (K). */
	tokens: number;
	/** 작업량 막대: 배정된 태스크 상태 순서. */
	load: ("in_progress" | "waiting" | "blocked" | "todo")[];
	loadNote: string;
	/** 고른 아바타 아이콘 순번 (없으면 역할 기본). */
	glyph?: number;
	/** 이 멤버의 지침 파일. 없으면 원본 템플릿과 같다. */
	files?: { name: string; body: string }[];
	/** 이 멤버의 스킬 · 도구 · 권한. 없으면 원본 템플릿과 같다. */
	config?: AgentConfig;
};

/// 팀 (.pen Teams / TeamListItem · Team Header). 스키마 tbl_team 요약. orch 팀은 모든 프로젝트를 조정한다.
export type Team = Pick<ApiTeam, "sn" | "name"> & {
	orch?: boolean;
	/** 연결 프로젝트 이름. orch 팀은 없음. */
	project?: string;
	desc: string;
	members: TeamMember[];
	/** 서버 집계값 (태스크 · 사이클). trend는 최근 7일 막대 높이 비율. */
	stats: {
		open: number;
		openNote: string;
		done: number;
		doneDelta: string;
		doneNote: string;
		doneTrend: number[];
		tokenTrend: number[];
		cycle: string;
		cycleDelta: string;
		cycleTrend: number[];
	};
};

const coreMembers: TeamMember[] = [
	{ sn: 2, name: "민수", role: "backend", title: "Backend Developer", runtime: "claude", model: "Claude Code · claude-sonnet-5", status: "running", work: "#128 Auth API (refresh token)", context: 92, tokens: 18.2, load: ["in_progress", "blocked"], loadNote: "#45 Rate limiting 막힘" },
	{ sn: 1, name: "진", role: "frontend", title: "Frontend Developer", runtime: "codex", model: "Codex CLI · gpt-5", status: "running", work: "#129 Login UI 구현", context: 32, tokens: 24.7, load: ["in_progress", "todo", "todo"], loadNote: "#129 진행 · 대기열 2" },
	{ sn: 3, name: "하린", role: "qa", title: "QA Engineer", runtime: "claude", model: "Claude Code · claude-sonnet-5", status: "waiting", work: "#130 ← #128 · #129 대기", context: 12, tokens: 5.0, load: ["waiting"], loadNote: "병목 · #128 #129 대기" },
	{ sn: 4, name: "소라", role: "reviewer", title: "Reviewer", runtime: "claude", model: "Claude Code · claude-opus-5.5", status: "idle", work: "다음: #131 Review · Auth", next: true, context: 6, tokens: 2.2, load: ["todo"], loadNote: "#131 Review 대기" },
	{ sn: 5, name: "유나", role: "designer", title: "Product Designer", runtime: "codex", model: "Codex CLI · gpt-5", status: "idle", work: "#126 Review 대기", context: 20, tokens: 8.4, load: [], loadNote: "여유 · 다음 배정 추천" },
];

const trend = { doneTrend: [8, 12, 10, 15, 18, 14, 22], tokenTrend: [14, 20, 11, 18, 16, 9, 21], cycleTrend: [22, 18, 20, 15, 16, 13, 11] };

export const teams: Team[] = [
	{
		sn: 1, name: "Orch", orch: true, desc: "모든 프로젝트의 Issue를 분해해 팀에 배정하고 진행을 조정하는 PM.",
		members: [{ sn: 0, name: "Orch", role: "orch", title: "Project Manager", runtime: "claude", model: "Claude Code · claude-opus-5.5", status: "idle", work: "판단 대기 3건 · PM Dock", context: 18, tokens: 6.1, load: [], loadNote: "대기" }],
		stats: { open: 3, openNote: "판단 대기 3", done: 21, doneDelta: "+6", doneNote: "지난주 15", ...trend, cycle: "12m", cycleDelta: "−3m" },
	},
	{
		sn: 2, name: "Core Team", project: "OrchStack", desc: "OrchStack 백엔드 · 프론트엔드 · QA · 리뷰를 담당하는 기본 팀. Orch가 Issue를 분해해 이 팀에 배정합니다.",
		members: coreMembers,
		stats: { open: 7, openNote: "Issue 2 · Sub-issue 2", done: 12, doneDelta: "+4", doneNote: "지난주 8", ...trend, cycle: "3h 12m", cycleDelta: "−18m" },
	},
	{
		sn: 3, name: "Web Squad", project: "Web Platform", desc: "Web Platform 화면과 디자인 시스템을 맡는 팀.",
		members: [
			{ sn: 6, name: "태오", role: "frontend", title: "Frontend Developer", runtime: "codex", model: "Codex CLI · gpt-5", status: "running", work: "#212 Settings 레이아웃", context: 41, tokens: 12.3, load: ["in_progress", "todo"], loadNote: "#212 진행 · 대기열 1" },
			{ sn: 7, name: "나래", role: "designer", title: "Product Designer", runtime: "claude", model: "Claude Code · claude-sonnet-5", status: "running", work: "#208 Token 문서화", context: 27, tokens: 7.9, load: ["in_progress"], loadNote: "#208 진행" },
			{ sn: 8, name: "지우", role: "qa", title: "QA Engineer", runtime: "claude", model: "Claude Code · claude-sonnet-5", status: "idle", work: "다음: #213 회귀 테스트", next: true, context: 4, tokens: 0.8, load: ["todo"], loadNote: "#213 대기" },
		],
		stats: { open: 4, openNote: "Issue 1 · Sub-issue 1", done: 6, doneDelta: "+1", doneNote: "지난주 5", ...trend, cycle: "4h 05m", cycleDelta: "+12m" },
	},
	{
		sn: 4, name: "Mobile Crew", project: "Mobile App", desc: "Mobile App 기능 개발과 검증을 맡는 팀.",
		members: [
			{ sn: 9, name: "도윤", role: "frontend", title: "Mobile Developer", runtime: "codex", model: "Codex CLI · gpt-5", status: "idle", work: "다음: #301 푸시 알림", next: true, context: 9, tokens: 1.4, load: ["todo"], loadNote: "#301 대기" },
			{ sn: 10, name: "서아", role: "qa", title: "QA Engineer", runtime: "claude", model: "Claude Code · claude-sonnet-5", status: "idle", work: "배정 없음", context: 0, tokens: 0, load: [], loadNote: "여유" },
		],
		stats: { open: 1, openNote: "Issue 1", done: 2, doneDelta: "+0", doneNote: "지난주 2", ...trend, cycle: "5h 40m", cycleDelta: "−5m" },
	},
];

/// 모델 연결 (.pen AccountQuotaCard). 스키마 tbl_connection 요약. 한도는 5시간 · 주간 잔량 %.
export type Account = {
	runtime: Runtime;
	name: string;
	/** 제공자 · 요금제 (예: Anthropic · Max). */
	plan: string;
	login: string;
	h5: number;
	h5Reset: string;
	week: number;
	weekReset: string;
	/** 소진 예상 (잔량 경고 문구에 쓴다). */
	left?: string;
	forecast?: string;
};

export const accounts: Account[] = [
	{ runtime: "claude", name: "Claude", plan: "Anthropic · Max", login: "claude-team@orch", h5: 38, h5Reset: "3h 후", week: 64, weekReset: "월 09:00" },
	{ runtime: "codex", name: "Codex", plan: "Codex · ChatGPT Pro", login: "codex-pro@orch", h5: 71, h5Reset: "5h 후", week: 18, weekReset: "화 09:00", left: "약 1.5일", forecast: "현재 속도면 목요일 오후에 소진돼요" },
];

/// 연결 추가에서 고르는 제공자 (.pen Settings · 연결 추가 1 · 제공자 선택).
/// state — ok: 연결됨 · detected: 이 기기에서 감지 · login: 로그인 필요 · open: 새로 연결 가능.
export type ProviderKind = "구독" | "플랜" | "API 키" | "게이트웨이" | "로컬";
export type Provider = {
	key: string;
	name: string;
	kind: ProviderKind;
	state: "ok" | "detected" | "login" | "open";
	note: string;
	/** 구독 · 플랜 — 로그인을 보관하는 실행기와 로그인하면 감지되는 요금제. */
	cli?: string;
	plan?: string;
	/** 게이트웨이 · 로컬 — 기본 엔드포인트. */
	baseUrl?: string;
	/** 연결 테스트로 불러오는 모델 수. */
	models: number;
};
export const providers: Provider[] = [
	{ key: "anthropic", name: "Anthropic · Claude", kind: "구독", state: "ok", note: "연결됨 · 계정 추가", cli: "Claude Code", plan: "Max", models: 4 },
	{ key: "chatgpt", name: "OpenAI · ChatGPT", kind: "구독", state: "ok", note: "연결됨 · 계정 추가", cli: "Codex CLI", plan: "ChatGPT Pro", models: 5 },
	{ key: "gemini", name: "Google · Gemini", kind: "구독", state: "detected", note: "Gemini CLI 설치됨 · 로그인 필요", cli: "Gemini CLI", plan: "Google AI Pro", models: 3 },
	{ key: "copilot", name: "GitHub Copilot", kind: "플랜", state: "ok", note: "연결됨 · 모델 9", cli: "OpenCode", plan: "Copilot Pro+", models: 9 },
	{ key: "cursor", name: "Cursor", kind: "플랜", state: "login", note: "Cursor Agent 로그인 필요", cli: "Cursor Agent", plan: "Cursor Pro", models: 6 },
	{ key: "zai", name: "Z.AI Coding Plan", kind: "플랜", state: "ok", note: "연결됨 · GLM 4.6", cli: "OpenCode", plan: "Z.AI Coding Plan", models: 2 },
	{ key: "kiro", name: "Amazon Q / Kiro", kind: "플랜", state: "login", note: "Kiro CLI 로그인 필요", cli: "Kiro CLI", plan: "Kiro Pro", models: 3 },
	{ key: "kimi", name: "Kimi Code", kind: "플랜", state: "open", note: "연결 가능", cli: "OpenCode", plan: "Kimi Moderato", models: 2 },
	{ key: "openai-api", name: "OpenAI API", kind: "API 키", state: "open", note: "키 입력", models: 31 },
	{ key: "anthropic-api", name: "Anthropic API", kind: "API 키", state: "ok", note: "prod 연결됨 · 키 추가", models: 8 },
	{ key: "ai-studio", name: "Google AI Studio", kind: "API 키", state: "open", note: "키 입력", models: 12 },
	{ key: "omniroute", name: "OmniRoute", kind: "게이트웨이", state: "ok", note: "localhost:20128 연결됨", baseUrl: "http://localhost:20128/v1", models: 22 },
	{ key: "openrouter", name: "OpenRouter", kind: "게이트웨이", state: "ok", note: "연결됨 · 크레딧 $41.2", baseUrl: "https://openrouter.ai/api/v1", models: 312 },
	{ key: "vercel", name: "Vercel AI Gateway", kind: "게이트웨이", state: "open", note: "키 + 엔드포인트", baseUrl: "https://ai-gateway.vercel.sh/v1", models: 48 },
	{ key: "ollama", name: "Ollama", kind: "로컬", state: "ok", note: "localhost:11434 · 모델 3", baseUrl: "http://localhost:11434/v1", models: 3 },
	{ key: "lmstudio", name: "LM Studio", kind: "로컬", state: "detected", note: "localhost:1234 감지됨", baseUrl: "http://localhost:1234/v1", models: 2 },
	{ key: "compat", name: "OpenAI 호환 (직접 입력)", kind: "로컬", state: "open", note: "Base URL 입력", baseUrl: "", models: 1 },
];

/// 설정 › 모델 연결 목록 (.pen Settings · 모델 연결). key는 providers의 key.
/// 한도 칩 — pct: 남은 % (구독 · 플랜) · used/limit: 이번 달 $ (API 키 · 게이트웨이).
/// state — low: 잔량 20% 미만 · exhausted: 한도 소진 · expired: 키 만료 (폴백에서 빠짐).
export type Quota = { label: string; pct?: number; used?: number; limit?: number };
export type Connection = {
	key: string;
	name: string;
	kind: ProviderKind;
	state: "ok" | "low" | "exhausted" | "expired" | "login" | "open";
	note: string;
	quotas: Quota[];
	/** 연결 가능 제공자의 버튼 문구 (기본 "연결"). */
	action?: string;
};
export const connections: Connection[] = [
	{ key: "chatgpt", name: "Codex · ChatGPT Pro", kind: "구독", state: "low", note: "주간 거의 소진 · 진 · 유나 사용", quotas: [{ label: "5H", pct: 71 }, { label: "주간", pct: 18 }] },
	{ key: "anthropic", name: "Anthropic · Max", kind: "구독", state: "ok", note: "Max 플랜 · 모델 17", quotas: [{ label: "5H", pct: 38 }, { label: "주간", pct: 64 }] },
	{ key: "copilot", name: "GitHub Copilot", kind: "플랜", state: "ok", note: "플랜 연결됨 · 모델 9", quotas: [{ label: "월", pct: 62 }] },
	{ key: "zai", name: "Z.AI Coding Plan", kind: "플랜", state: "ok", note: "플랜 연결됨 · GLM 4.6", quotas: [{ label: "5H", pct: 88 }] },
	{ key: "omniroute", name: "OmniRoute", kind: "게이트웨이", state: "ok", note: "localhost:20128 · 512 모델", quotas: [{ label: "월", used: 12, limit: 30 }] },
	{ key: "openrouter", name: "OpenRouter", kind: "게이트웨이", state: "ok", note: "크레딧 $41.2 남음", quotas: [{ label: "월", used: 4, limit: 20 }] },
	{ key: "anthropic-api", name: "Anthropic API · prod", kind: "API 키", state: "ok", note: "키체인 저장 · 폴백 전용", quotas: [{ label: "월", used: 38, limit: 100 }] },
	{ key: "ollama", name: "Ollama", kind: "로컬", state: "ok", note: "localhost:11434 · 모델 3", quotas: [] },
	{ key: "cursor", name: "Cursor", kind: "플랜", state: "login", note: "Cursor Agent 로그인 필요", quotas: [] },
	{ key: "kiro", name: "Amazon Q / Kiro", kind: "플랜", state: "login", note: "Kiro CLI 로그인 필요", quotas: [] },
	{ key: "kimi", name: "Kimi Code", kind: "플랜", state: "open", note: "연결 가능", quotas: [] },
	{ key: "vercel", name: "Vercel AI Gateway", kind: "게이트웨이", state: "open", note: "연결 가능", quotas: [], action: "확인 후 연결" },
];

/// 실행기별 폴백 체인 (.pen FallbackStep). 위에서부터 시도. key는 connections의 key.
/// tier — 이 단계가 맡는 하위 작업 등급 (tbl_map_fallback.tier). 없으면 모든 등급. 같은 연결을 등급만 다르게 여러 번 넣을 수 있다 (#67).
export type ChainStep = { key: string; name: string; kind: ProviderKind; cond: string; cost: string; tier?: SubRunTier };
export const fallbackChains: Record<Runtime, ChainStep[]> = {
	claude: [
		{ key: "anthropic", name: "Anthropic · Max · Haiku", kind: "구독", cond: "S 작업(조사 · 요약 · 린트 수정) · 주간 잔량 20% 미만이면 다음", cost: "—", tier: "S" },
		{ key: "anthropic", name: "Anthropic · Max · Opus", kind: "구독", cond: "기본 · 주간 잔량 20% 미만이면 다음으로", cost: "—" },
		{ key: "anthropic-api", name: "Anthropic API · prod", kind: "API 키", cond: "429 · 한도 초과 시 · 월 예산 $100 안에서", cost: "$100/월" },
		{ key: "omniroute", name: "OmniRoute", kind: "게이트웨이", cond: "API 예산 소진 시 · 저렴한 제공자로 자동 라우팅", cost: "$30/월" },
		{ key: "ollama", name: "Ollama · qwen3-coder", kind: "로컬", cond: "모두 실패 시 · L0 내부 작업만", cost: "무료", tier: "S" },
	],
	codex: [
		{ key: "chatgpt", name: "Codex · ChatGPT Pro", kind: "구독", cond: "기본 · 주간 잔량 20% 미만이면 다음으로", cost: "—" },
		{ key: "omniroute", name: "OmniRoute", kind: "게이트웨이", cond: "한도 초과 · 오류 시 · OpenAI 호환 모델로 라우팅", cost: "$30/월" },
		{ key: "ollama", name: "Ollama · qwen3-coder", kind: "로컬", cond: "모두 실패 시 · L0 내부 작업만", cost: "무료" },
	],
};

/// 설정 › 실행기 (CLI) — 이 기기에서 감지한 CLI (.pen Settings · 실행기). provider는 로그인에 쓰는 providers key.
/// state — ok: 로그인됨 · update: 업데이트 가능 · gateway: API 키 연결만 · login: 로그인 필요 · missing: 설치 안 됨.
export type RuntimeCli = {
	key: string;
	name: string;
	version: string;
	state: "ok" | "update" | "gateway" | "login" | "missing";
	note: string;
	provider?: string;
	/** 이 실행기로 쓸 수 있는 연결 (.pen 실행기 ↔ 연결). */
	conns?: string;
};
export const runtimeClis: RuntimeCli[] = [
	{ key: "codex", name: "Codex CLI", version: "v0.41", state: "ok", note: "로그인됨 · ChatGPT Pro · 멤버 2", conns: "ChatGPT Pro · OmniRoute" },
	{ key: "claude", name: "Claude Code", version: "v2.3", state: "update", note: "v2.4 업데이트 가능 · Anthropic Max · 멤버 3", conns: "Anthropic Max · API · OmniRoute" },
	{ key: "gemini", name: "Gemini CLI", version: "v0.9", state: "ok", note: "로그인됨 · Google AI Pro · 멤버 0", conns: "Google AI Pro" },
	{ key: "opencode", name: "OpenCode", version: "v0.14", state: "gateway", note: "게이트웨이 · API 키 연결만 사용", conns: "OpenRouter · Ollama" },
	{ key: "cursor", name: "Cursor Agent", version: "v1.2", state: "login", note: "설치됨 · Cursor 로그인 필요", provider: "cursor" },
	{ key: "kiro", name: "Kiro CLI", version: "—", state: "missing", note: "설치 안 됨 · Amazon Q / Kiro 플랜용", provider: "kiro" },
];

/// 공통 실행 설정 (.pen 실행기 · 공통 실행 설정). 모든 실행기에 적용 — 팀 정책 · 멤버 권한이 더 좁으면 그쪽 우선.
export const runSettings = { maxRuns: "3 Run", timeout: "20분", workspace: "git worktree", sandbox: true, autoUpdate: true };

/// 설정 › 알림 (.pen Settings · 알림). 이벤트 × 채널 켜짐 [앱, 데스크톱, Telegram, 이메일].
export type NotifyChannel = "app" | "desktop" | "telegram" | "email";
export const notifyEvents: { group: string; items: { name: string; desc?: string; on: Record<NotifyChannel, boolean> }[] }[] = [
	{ group: "결정 · 승인", items: [
		{ name: "결정 요청 (L2 이상)", desc: "Orch가 사람 판단을 기다릴 때", on: { app: true, desktop: true, telegram: true, email: true } },
		{ name: "승인 요청", desc: "PR 생성 · 새 의존성 · Run 연장", on: { app: true, desktop: true, telegram: true, email: false } },
		{ name: "Orch가 대신 결정", desc: "대기 시간 초과로 Orch가 결정했을 때", on: { app: true, desktop: false, telegram: false, email: false } },
	] },
	{ group: "Run", items: [
		{ name: "Run 실패", desc: "lint · test 실패, 오류 종료", on: { app: true, desktop: true, telegram: true, email: false } },
		{ name: "루프 가드 정지", desc: "반려 3회 · 연속 실행 한도", on: { app: true, desktop: true, telegram: true, email: true } },
		{ name: "컨텍스트 경고", desc: "80% 이상", on: { app: true, desktop: false, telegram: false, email: false } },
	] },
	{ group: "한도 · 연결", items: [
		{ name: "구독 잔량 20% 미만", desc: "5시간 · 주간 창", on: { app: true, desktop: true, telegram: true, email: false } },
		{ name: "예산 80% 도달", desc: "API 키 · 게이트웨이", on: { app: true, desktop: false, telegram: true, email: true } },
		{ name: "연결 오류 · 키 만료", desc: "테스트 실패 3회 · 401", on: { app: true, desktop: true, telegram: true, email: true } },
	] },
	{ group: "결과", items: [
		{ name: "PR 생성 · 병합", on: { app: true, desktop: false, telegram: true, email: false } },
		{ name: "태스크 완료", on: { app: true, desktop: false, telegram: false, email: false } },
		{ name: "일일 요약", desc: "매일 18:00", on: { app: false, desktop: false, telegram: true, email: true } },
	] },
];
/// 채널 상태 — ok: 연결됨 · off: 설정 안 됨.
export const notifyChannels: { key: NotifyChannel; name: string; state: string; ok: boolean; desc: string }[] = [
	{ key: "app", name: "앱", state: "항상", ok: true, desc: "화면 오른쪽 위 · PM Dock" },
	{ key: "desktop", name: "데스크톱", state: "허용됨", ok: true, desc: "macOS 알림 센터" },
	{ key: "telegram", name: "Telegram", state: "연결됨", ok: true, desc: "@orchstack_bot · 알림방" },
	{ key: "email", name: "이메일", state: "설정 안 됨", ok: false, desc: "일일 요약 · 중요 알림만 권장" },
];
export const notifyQuiet = { hours: "22:00 – 08:00", weekend: true, urgentBypass: true, digest: "매일 18:00" };

/// 설정 › 권한 · 보안 (.pen Settings · 권한 · 보안). 워크스페이스 기본값 — 팀 · 멤버는 더 엄격하게만 바꾼다.
export const security = {
	trust: 3 as AgentConfig["trust"],
	approvals: [
		{ action: "PR 생성", policy: "승인 필요", who: "나", notify: true },
		{ action: "새 의존성 추가", policy: "승인 필요", who: "Orch → 나", notify: true },
		{ action: "팀 외부로 메시지", policy: "자동", who: "—", notify: false },
		{ action: ".env 파일 접근", policy: "차단", who: "—", notify: true },
		{ action: "20분 이상 Run 연장", policy: "승인 필요", who: "나", notify: true },
	] as Approval[],
	/** scope — 명령이 막히는 조건 (없으면 어디서나). */
	blocked: [
		{ pattern: "git push --force", desc: "원격 기록 덮어쓰기" },
		{ pattern: "git reset --hard origin/*", desc: "로컬 변경 삭제" },
		{ pattern: "rm -rf", scope: "작업 공간 밖", desc: "worktree 밖 파일 삭제" },
		{ pattern: "curl … | sh", desc: "원격 스크립트 실행" },
		{ pattern: "git push origin --delete", desc: "원격 브랜치 삭제" },
	] as { pattern: string; scope?: string; desc: string }[],
	guards: [
		{ name: ".env 파일 접근 경고", desc: "Tool use · 내용 패턴", on: true },
		{ name: "도구 실행 오류 알림", desc: "Tool result", on: true },
		{ name: "비밀키 패턴 출력 감지", desc: "Output · sk-… / ghp_…", on: true },
	],
	secrets: [["키 저장 위치", "macOS 키체인"], [".env 파일", "읽기 차단 (모든 멤버)"], ["출력 마스킹", "sk-… · ghp_… · AKIA…"]] as [string, string][],
	github: { name: "orch-bot", note: "사람이 한 작업과 구분돼요 · Repo 권한 orchstack/*" },
};
/// 감사 로그 — 권한 · 연결 · 설정 변경 기록 (최근이 앞).
export type AuditLog = { kind: "KEY" | "POLICY" | "BLOCK"; who: string; when: string; text: string };
export const auditLog: AuditLog[] = [
	{ kind: "KEY", who: "나", when: "오늘 10:40", text: "Vercel AI Gateway 키 추가 · 키체인" },
	{ kind: "POLICY", who: "나", when: "어제 19:02", text: "승인 규칙: 새 의존성 → Orch → 나" },
	{ kind: "BLOCK", who: "시스템", when: "어제 18:10", text: "git reset --hard origin/main 차단 · 진" },
];

/// 설정 › Instruction presets (.pen Settings · Instruction presets). 원문은 data/presets — 기본 제공(OrchStack v1)은 잠겨 있고 복제해서 고친다.
/// tok — o200k 기준 측정값 · limit — 종류별 상한 · locked — 시스템 소유(protocol) · default — 새 역할 조합에 기본 포함.
export type PresetKind = "protocol" | "role" | "rule" | "style" | "report";
export type Preset = { key: string; kind: PresetKind; limit: number; tok: number; locked?: boolean; default?: boolean; body: string };
export const presets: Preset[] = [
	{
		"key": "orch-protocol",
		"kind": "protocol",
		"limit": 400,
		"tok": 332,
		"locked": true,
		"body": "# OrchStack protocol\nYou receive work as an `@TASK` block. Reply only through the blocks below. No chat, no greetings.\n\n- Finish every run with exactly one `@REPORT` block as the last message.\n- If a human decision is required, output one `@ASK` block and stop. Do not guess.\n- Keep machine lines in English. Values <=120 chars.\n- Do not list files, test output, tokens or time. The system collects them.\n\n@REPORT v1\nid: <task>  run: <run>  status: done|partial|failed|blocked\nac <n> ok|fail|skip [\"reason\"]          # one line per acceptance item\nchg <contract-key> add|modify|remove|breaking \"<new shape>\"   # only if another task uses it\nimpact <task> retest|rework|review|ctx_update \"<why>\"\nverify <n> step \"<what QA should check>\"\nrisk perf|security|data|compat \"<what>\"\n--- human\nresult: <1-2 sentences, report style>\narea: <area> | <what changed>             # max 6\nreview: <function/table/file> | <why>      # max 5\nunverified: <what was not checked, or \"none\">   # required\nleft: <item> | <when/condition>\n\n@ASK v1\nid: <task>  level: L2|L3  q: \"<one line>\"  opts: [A <opt>, B <opt>*]  refs: [<path:line>]"
	},
	{
		"key": "frontend",
		"kind": "role",
		"limit": 600,
		"tok": 106,
		"body": "# Role: Frontend developer\nOwn UI in the task `paths`. Do not change backend, schema or build config unless the task says so.\n- Reuse existing components and tokens before adding new ones.\n- Handle loading, empty and error states for every async view.\n- Labels, focus and keyboard access on every control.\n- Consume API contracts exactly as given in `uses`; if a contract is wrong, report `risk compat`, do not work around it.\n- Done = all ac ok, lint and tests pass."
	},
	{
		"key": "backend",
		"kind": "role",
		"limit": 600,
		"tok": 102,
		"body": "# Role: Backend developer\nOwn APIs, services and data in the task `paths`.\n- Any change to an API, schema, env or config that another task uses is a contract change: report it as `chg`, mark `breaking` if callers must change.\n- Schema changes need a migration that can run twice safely.\n- Validate input at the boundary; never log secrets or tokens.\n- Add or update tests for every changed endpoint.\n- Done = all ac ok, tests pass, migrations apply."
	},
	{
		"key": "qa",
		"kind": "role",
		"limit": 500,
		"tok": 83,
		"body": "# Role: QA engineer\nVerify, do not implement features.\n- Start from the `verify` steps and ac of the tasks under test.\n- Write or run the smallest tests that prove each ac; report each as `ac n ok|fail`.\n- A failure must include how to reproduce in one line.\n- You may fix test code only. Product bugs go to `impact <task> rework`."
	},
	{
		"key": "reviewer",
		"kind": "role",
		"limit": 500,
		"tok": 92,
		"body": "# Role: Reviewer\nReview the diff against the task ac and the contracts. Do not rewrite the code.\n- Approve only if every ac is met and no contract is broken silently.\n- Report findings as `risk` or `impact <task> rework` with file:line.\n- Ignore style issues that lint already covers.\n- Same rejection reason twice means the ac is unclear: ask with `@ASK`, do not reject a third time."
	},
	{
		"key": "designer",
		"kind": "role",
		"limit": 450,
		"tok": 69,
		"body": "# Role: Product designer\nProduce UI specs and assets the frontend role can build without questions.\n- Reuse the existing design system; list any new token or component explicitly as `chg`.\n- Specify states: default, hover, focus, disabled, loading, empty, error.\n- Keep copy short; mark every user-facing string for translation."
	},
	{
		"key": "generalist",
		"kind": "role",
		"limit": 300,
		"tok": 32,
		"body": "# Role: Generalist\nDo the task exactly as scoped. Touch only the task `paths`. Anything outside scope goes to `left` or `impact`."
	},
	{
		"key": "orch",
		"kind": "role",
		"limit": 350,
		"tok": 108,
		"body": "# Role: Orch (PM)\nYou are called only for task breakdown or L2+ judgment. Output a WorkProposal or a decision, nothing else.\n- Split work into tasks a single agent can finish in one run (<=20 min).\n- Each task: goal, ac (testable, numbered), paths, uses/provides contracts, role, deps.\n- Prefer fewer tasks. Never assign two tasks that write the same paths in parallel.\n- For decisions: pick the safest reversible option, state the reason in one line."
	},
	{
		"key": "token-economy",
		"kind": "rule",
		"limit": 300,
		"tok": 92,
		"default": true,
		"body": "# Rule: token economy\n- Open only files in `paths`, `ctx` or directly imported by them. Search before opening.\n- Read a file once per run; do not re-read unless it changed.\n- Never print whole files, logs or diffs. Quote at most the lines you need.\n- Do not run commands whose output you will not use. Pipe long output through head/grep.\n- No explanations outside the final `@REPORT`."
	},
	{
		"key": "git-safety",
		"kind": "rule",
		"limit": 200,
		"tok": 56,
		"default": true,
		"body": "# Rule: git safety\n- Work on the task branch only. Never force-push, reset --hard, or delete remote branches.\n- Small commits with messages `<type>: <what>` in English.\n- Do not commit secrets, .env files or generated build output."
	},
	{
		"key": "testing",
		"kind": "rule",
		"limit": 200,
		"tok": 53,
		"default": true,
		"body": "# Rule: test before report\nRun the project's lint and the tests for changed code before `@REPORT`. If you cannot run them, set the ac to `skip` and put the reason in `unverified`. Never report `done` with failing tests."
	},
	{
		"key": "secrets",
		"kind": "rule",
		"limit": 150,
		"tok": 35,
		"default": true,
		"body": "# Rule: secrets\nNever read .env files or key stores, and never echo tokens, keys or passwords. If a secret is needed, stop and `@ASK`."
	},
	{
		"key": "language",
		"kind": "rule",
		"limit": 120,
		"tok": 29,
		"default": true,
		"body": "# Rule: language\nMachine lines, code, commits and comments in English. Lines under `--- human` in {{workspace.report_language}}."
	},
	{
		"key": "concise",
		"kind": "style",
		"limit": 150,
		"tok": 37,
		"default": true,
		"body": "# Style: concise\nWrite in report style. No greetings, apologies or restating the task. State facts, not intentions. If unsure, say what is unknown instead of guessing."
	},
	{
		"key": "careful",
		"kind": "style",
		"limit": 150,
		"tok": 40,
		"body": "# Style: careful\nBefore changing shared code, check its callers. Prefer the smaller, reversible change. When two options are close, choose the one easier to undo and say why in one line."
	},
	{
		"key": "task-report",
		"kind": "report",
		"limit": 200,
		"tok": 102,
		"default": true,
		"body": "# Report: task\nFill the `--- human` part so a person can decide without reading the diff.\n- result: what now works, and what does not, in 1-2 sentences.\n- area: meaningful areas (screen, module, API), not file names.\n- review: the exact places a reviewer should look first, with why.\n- unverified: be honest; list anything you did not run or could not check.\n- left: remaining work with the condition to finish it."
	}
];

/// 설정 › 보고서 양식 (.pen Settings · 보고서 양식). 원문은 data/report-forms. {{…}} 시스템 값 · [[…]] Agent 사람 칸 · 모델 호출 없음.
export type ReportForm = { key: string; name: string; note: string; body: string };
export const reportForms: ReportForm[] = [
	{
		"key": "task-report",
		"name": "태스크 보고서",
		"note": "시스템이 조립 (LLM 호출 없음). {{…}} 는 시스템 값, [[…]] 는 Agent 사람 칸",
		"body": "#{{task.num}} — {{task.title}} [{{status.label}}]\n## 결과\n[[result]]\n## 주요 변경\n{{#each area}}- {{key}}: {{value}}\n{{/each}}\n## 검토할 지점\n{{#each review}}- {{key}}: {{value}}\n{{/each}}\n## 검증\n- 통과: {{tests.passed_summary}}\n- 미확인: [[unverified]]\n## 남은 문제\n{{#each left}}- {{key}}: {{value}}\n{{/each}}\n범위: {{scope.files}}개 파일 +{{scope.add}} −{{scope.del}} / 커밋 {{scope.commits}}{{#if pr}} · PR #{{pr.num}} ({{pr.status}}){{/if}} / {{run.tokens}} tok · {{run.duration}}"
	},
	{
		"key": "issue-report",
		"name": "이슈 보고서",
		"note": "하위 태스크 보고서 합산 (LLM 호출 없음)",
		"body": "#{{issue.num}} — {{issue.title}} [{{status.label}}]\n## 결과\n{{#each tasks}}- #{{num}} {{title}}: {{result}}\n{{/each}}\n## 계약 변경\n{{#each contracts}}- {{key}} v{{from}}→v{{to}}: {{delta}}\n{{/each}}\n## 미확인 (합산)\n{{#each unverified}}- #{{task}}: {{value}}\n{{/each}}\n## 남은 문제\n{{#each left}}- #{{task}} {{key}}: {{value}}\n{{/each}}\n범위: 태스크 {{tasks.count}} · Run {{runs.count}} · 재작업 {{rework.count}} / {{tokens.total}} tok · {{duration}}"
	},
	{
		"key": "pr-body",
		"name": "PR 본문",
		"note": "태스크 보고서를 PR 본문으로 복사. 재작업 시 갱신, 이전 판은 태스크 기록에 보관",
		"body": "{{> task-report}}\n\n---\nOrchStack task #{{task.num}} · run {{run.num}} · {{member.name}} ({{runtime}} · {{model}})"
	},
	{
		"key": "daily-summary",
		"name": "일일 요약",
		"note": "알림 › 일일 요약 (Telegram · 이메일)",
		"body": "{{date}} 일일 요약 — {{project.name}}\n- 완료 {{done.count}} · 진행 {{running.count}} · 막힘 {{blocked.count}} · 판단 대기 {{decision.count}}\n{{#each done}}- 완료 #{{num}} {{title}}{{#if unverified}} (미확인 있음){{/if}}\n{{/each}}{{#each blocked}}- 막힘 #{{num}} {{title}}: {{reason}}\n{{/each}}\n토큰 {{tokens.today}} (어제 대비 {{tokens.delta}}) · 한도 경고 {{quota.warnings}}"
	}
];

/// 보고서 양식 미리보기 데이터 — #129 (Login UI 구현). human은 Agent가 채운 [[…]] 칸, 나머지는 시스템 값.
export const reportSample = {
	date: "9/30",
	project: { name: "OrchStack" },
	task: { num: 129, title: "Login UI 구현" },
	issue: { num: 51, title: "로그인 · 인증" },
	status: { label: "일부 완료" },
	human: {
		result: "로그인 폼과 에러·로딩 상태가 동작함. 세션 만료 처리는 #128 응답 확정 전이라 보류.",
		unverified: "실제 백엔드(#128) 연동, 스크린리더 확인",
	},
	area: [
		{ key: "로그인 화면", value: "이메일·비밀번호 폼, 401/429/5xx 에러 문구, 중복 제출 방지" },
		{ key: "인증 모듈", value: "로그인 요청·토큰 저장 추가" },
	],
	review: [
		{ key: "LoginForm.svelte handleSubmit()", value: "에러 코드별 분기" },
		{ key: "auth.ts saveTokens()", value: "토큰 저장 위치" },
	],
	tests: { passed_summary: "단위 테스트 12개, lint" },
	left: [{ task: 129, key: "세션 만료 리다이렉트", value: "#128 refresh 응답 확정 후" }],
	scope: { files: 4, add: 310, del: 15, commits: 3 },
	pr: { num: 88, status: "draft" },
	run: { num: 85, tokens: "24.6K", duration: "15m" },
	member: { name: "진" },
	runtime: "Codex CLI",
	model: "gpt-5",
	tasks: [
		{ num: 128, title: "Auth API", result: "로그인 · refresh 엔드포인트 완료" },
		{ num: 129, title: "Login UI 구현", result: "폼 · 에러 상태 완료, 세션 만료 보류" },
	],
	contracts: [{ key: "auth.login", from: 1, to: 2, delta: "응답에 refresh_token 추가" }],
	unverified: [{ task: 129, value: "실제 백엔드(#128) 연동, 스크린리더 확인" }],
	runs: { count: 5 },
	rework: { count: 1 },
	tokens: { total: "142K", today: "86K", delta: "+12%" },
	duration: "3h 10m",
	done: [
		{ num: 128, title: "Auth API", unverified: false },
		{ num: 129, title: "Login UI 구현", unverified: true },
	],
	running: { count: 2 },
	blocked: [{ num: 131, title: "세션 만료 처리", reason: "#128 refresh 응답 대기" }],
	decision: { count: 3 },
	quota: { warnings: 1 },
};

/// 상단 벨 알림 (.pen Workbench · 알림 목록). kind — decision: 결정 요청 · approval: 승인 요청 · fail: Run 실패 · quota: 한도 · guard: 루프 가드 · pr: PR.
/// need — 사람이 확인해야 하는 알림 (확인 필요 묶음). who가 멤버면 member sn.
export type Notice = {
	id: number;
	kind: "decision" | "approval" | "fail" | "quota" | "guard" | "pr";
	who: string;
	member?: number;
	title: string;
	desc: string;
	when: string;
	group: "확인 필요" | "오늘" | "어제" | "이번 주";
	task?: number;
	unread?: boolean;
};
export const notices: Notice[] = [
	{ id: 1, kind: "decision", who: "Orch", title: "#130 E2E 브라우저 선택 (L2)", desc: "Chromium만 vs Chromium + WebKit · 10분 뒤 Orch가 결정", when: "2분 전", group: "확인 필요", task: 130, unread: true },
	{ id: 2, kind: "approval", who: "진", member: 1, title: "Run 연장 요청 · #129 Run #81", desc: "20분 초과 예상 · 남은 단계 2/5", when: "3분 전", group: "확인 필요", task: 129, unread: true },
	{ id: 3, kind: "fail", who: "진", title: "Run #77 실패 · lint 3건", desc: "Orch가 lint 수정 지시와 함께 Run #81로 재시도", when: "13:56", group: "오늘", task: 129, unread: true },
	{ id: 4, kind: "quota", who: "시스템", title: "Codex 계정 주간 잔량 18%", desc: "현재 속도면 목요일 오후 소진 · 폴백: Anthropic Max", when: "13:40", group: "오늘", unread: true },
	{ id: 5, kind: "guard", who: "소라", title: "#121 리뷰 3회 반려 → 정지", desc: "완료 조건에 접근성 기준 추가 후 통과", when: "17:40", group: "어제", task: 121 },
	{ id: 6, kind: "pr", who: "Orch", title: "PR #86 병합 · #121 Signup UI", desc: "tests 12/12 · 하린에게 QA 배정", when: "9/26", group: "이번 주", task: 121 },
];

/// 하위 작업 (#67 · .pen Diagram · 하위 작업). 리드 Run 안에서 나눈 작업 — runner는 별도 Run, sub · fork는 리드 Run 안.
/// tier는 규칙 엔진이 kind로 정한다 (리드가 고르지 않음). paths from — orig: 처음 받음 · ask: @ASK로 추가 · bad: 범위 밖 변경.
/// runs — runner의 Run들 (재시도 전 Run 포함), 토큰 K. 토큰은 Run별로 저장하고 화면에서만 합친다.
export type SubRunTier = "S" | "M" | "L";
export type SubRun = {
	id: string;
	task: number;
	/** 리드 멤버 sn · 리드 Run 번호. */
	lead: number;
	leadRun: number;
	mode: "sub" | "fork" | "runner";
	kind: "explore" | "search" | "format" | "test" | "implement" | "fix" | "design" | "review" | "debug";
	tier?: SubRunTier;
	goal: string;
	status: "running" | "done" | "failed" | "queued";
	model?: string;
	minutes?: number;
	runs: { num: number; tokens: number; tier?: SubRunTier }[];
	/** 재시도로 등급이 오른 경우. */
	retry?: { from: SubRunTier; to: SubRunTier };
	paths: { path: string; from: "orig" | "ask" | "bad"; at?: string }[];
	ac: { text: string; ok: boolean }[];
	report?: string;
	/** paths가 겹쳐 순서를 기다리는 하위 작업과 겹친 경로. */
	waits?: { id: string; glob: string };
	workdir: { mode: "repo" | "worktree"; path?: string; branch: string };
};
/// 리드 Run의 자기 사용량 (K) — 하위 runner 합계는 subRuns에서 더한다.
export const leadRuns: Record<number, { run: number; self: number }> = { 129: { run: 81, self: 10 } };
export const subRuns: SubRun[] = [
	{ id: "T129.1", task: 129, lead: 1, leadRun: 81, mode: "runner", kind: "implement", tier: "M", goal: "LoginForm 컴포넌트 구현", status: "running", model: "gpt-5 · Effort Auto", minutes: 6,
		runs: [{ num: 82, tokens: 18.4 }],
		paths: [{ path: "login/LoginForm.svelte", from: "orig" }, { path: "lib/ui/Input.svelte", from: "orig" }, { path: "src/api/auth.ts", from: "ask", at: "14:02" }],
		ac: [{ text: "이메일 · 비밀번호 입력에 label 연결", ok: true }, { text: "제출 중 버튼 비활성 + 스피너", ok: false }, { text: "pnpm lint · check 통과", ok: false }],
		report: "@REPORT T129.1 · 진행 중 · 폼 마크업 완료, 제출 상태 작업 중", workdir: { mode: "repo", branch: "feat/129-login-form" } },
	{ id: "T129.2", task: 129, lead: 1, leadRun: 81, mode: "runner", kind: "search", tier: "S", goal: "에러 코드별 문구 정리", status: "done", model: "gpt-5-mini", minutes: 4,
		runs: [{ num: 83, tokens: 9.1 }],
		paths: [{ path: "lib/auth/errors.ts", from: "orig" }],
		ac: [{ text: "401 · 429 · 5xx 문구", ok: true }, { text: "i18n 키로 분리", ok: true }],
		report: "@REPORT T129.2 · 완료 · 문구 5개 · 키 lib/auth/errors.ts", workdir: { mode: "repo", branch: "feat/129-login-form" } },
	{ id: "T129.4", task: 129, lead: 1, leadRun: 81, mode: "runner", kind: "implement", tier: "M", goal: "Input 에러 상태 추가", status: "queued", model: "gpt-5",
		runs: [],
		paths: [{ path: "lib/ui/Input.svelte", from: "orig" }, { path: "src/api/errors.ts", from: "orig" }],
		ac: [{ text: "aria-invalid · 설명 연결", ok: false }, { text: "에러 색 토큰 사용", ok: false }],
		waits: { id: "T129.1", glob: "src/api/*" }, workdir: { mode: "repo", branch: "feat/129-login-form" } },
	{ id: "T129.3", task: 129, lead: 1, leadRun: 81, mode: "sub", kind: "search", goal: "기존 Input 사용처 조사", status: "done", minutes: 2,
		runs: [],
		paths: [],
		ac: [{ text: "사용처 목록", ok: true }, { text: "props 차이 정리", ok: true }],
		report: "@REPORT T129.3 · 완료 · 사용처 7곳", workdir: { mode: "repo", branch: "feat/129-login-form" } },
	{ id: "T129.6", task: 129, lead: 1, leadRun: 81, mode: "runner", kind: "test", tier: "M", goal: "E2E 로그인 시나리오", status: "failed", model: "gpt-5 · Effort Auto", minutes: 11,
		runs: [{ num: 84, tokens: 6.0, tier: "S" }, { num: 85, tokens: 22.0, tier: "M" }], retry: { from: "S", to: "M" },
		paths: [{ path: "e2e/login.spec.ts", from: "orig" }, { path: "e2e/fixtures/user.ts", from: "orig" }, { path: "src/x.ts", from: "bad" }],
		ac: [{ text: "로그인 성공 시나리오 통과", ok: true }, { text: "잘못된 비밀번호 안내 문구 확인", ok: false }, { text: "pnpm test:e2e 통과", ok: false }],
		report: "@REPORT T129.6 · 실패 · paths 밖 파일(src/x.ts)을 고쳐 규칙 엔진이 중단", workdir: { mode: "worktree", path: ".orch/wt/T129.6", branch: "feat/129-e2e-login" } },
	{ id: "T129.5", task: 129, lead: 1, leadRun: 81, mode: "fork", kind: "design", goal: "에러 문구 톤 가이드 초안", status: "running", minutes: 5,
		runs: [],
		paths: [{ path: "docs/tone.md", from: "orig" }, { path: "lib/auth/errors.ts", from: "orig" }],
		ac: [{ text: "톤 규칙 5줄", ok: false }],
		workdir: { mode: "repo", branch: "feat/129-login-form" } },
];

/// 이번 달 비용 (.pen 이번 달 비용). limit이 없으면 고정 요금.
export const monthCost = [
	{ label: "구독 · 플랜 (고정)", used: 239, note: "Max $100 · ChatGPT Pro $100 · Copilot $19 · Z.AI $20" },
	{ label: "API 키", used: 38.2, limit: 100 },
	{ label: "게이트웨이", used: 16.4, limit: 50, note: "OmniRoute $12.4 · OpenRouter $4.0" },
];

/// 종량제 연결 (폴백 · 게이트웨이). 이번 달 사용액 $.
export const meters = [
	{ name: "Anthropic API · 폴백", kind: "key", used: 38.2, limit: 100, note: "이번 달 · 폴백 12회" },
	{ name: "OmniRoute · 게이트웨이", kind: "route", used: 12.4, limit: 30, note: "폴백 7회 · 캐시 적중 31%" },
] as const;

/// Orch 진행 정책 (.pen Orch 진행 정책 · 자동 진행 / 사용자 판단 / 루프 가드). 레벨 L0–L4별 처리 방식.
export type SpawnMode = "sub" | "fork" | "runner";
export type LevelAction = "auto" | "timer" | "wait" | "block";
export type OrchPolicy = {
	mode: "manual" | "timer" | "full";
	/** Auto · 타이머 대기 초. */
	timer: number;
	/** 연속 자동 진행 현재 횟수. */
	streak: number;
	levels: { name: string; example: string; action: LevelAction; timeout: string; locked?: boolean }[];
	guards: { key: "streak" | "reject" | "repeat" | "budget" | "spawn" | "away"; name: string; value: string; on: boolean }[];
	lastStop: string;
	/** 하위 작업 정책 (tbl_team.spawn_mode · spawn_allow · max_child_run · #67). 기본 방식은 허용 안에 있어야 하고 동시 수는 1 이상. */
	spawn: { mode: SpawnMode; allow: SpawnMode[]; maxChild: number };
};

export const orchPolicy: OrchPolicy = {
	mode: "timer",
	timer: 5,
	streak: 3,
	levels: [
		{ name: "내부 작업", example: "파일 수정 · 테스트 · 로컬 커밋 · 브랜치 생성", action: "auto", timeout: "" },
		{ name: "흐름 결정", example: "다음 태스크 배정 · Todo/Backlog 분배 · 태스크 분할", action: "timer", timeout: "" },
		{ name: "모호한 판단", example: "요구사항 해석 · 설계 선택지 · 에이전트의 질문", action: "wait", timeout: "10분" },
		{ name: "외부 영향", example: "PR 생성 · 의존성 추가 · DB 스키마 · 외부 메시지", action: "wait", timeout: "" },
		{ name: "위험", example: "Destructive git · 비밀키 접근 · 범위 밖 파일", action: "block", timeout: "", locked: true },
	],
	guards: [
		{ key: "streak", name: "연속 자동 진행", value: "10회 / 이슈", on: true },
		{ key: "reject", name: "반려 → 재작업 반복", value: "3회 / 태스크", on: true },
		{ key: "repeat", name: "같은 실패 반복", value: "2회 (같은 에러 · 같은 diff)", on: true },
		{ key: "budget", name: "이슈 예산", value: "300K tok · 4h", on: true },
		{ key: "spawn", name: "Orch가 만든 새 태스크", value: "5개 / 이슈", on: true },
		{ key: "away", name: "사용자 부재 감지", value: "30분 무응답 → Manual로 전환", on: true },
	],
	lastStop: "어제 #121 반려 3회",
	spawn: { mode: "runner", allow: ["sub", "runner"], maxChild: 3 },
};

/// 팀 정책 (.pen 팀 정책 카드). 사용량은 멤버 목록에서 계산한다.
export const teamPolicy = {
	runtime: "Claude Code · Codex CLI 혼합",
	tokenBudget: 200,
	contextWarn: 80,
	maxRuns: 3,
	review: "PR 병합 전",
	repo: "orchstack/* · branch",
	/** 주간 잔량 경고 기준 %. */
	quotaWarn: 20,
};

/// 에이전트 설정 (.pen Skills · Tools & MCP · 권한). 템플릿 기본값을 멤버가 복사해 고친다. 스키마 tbl_agent_profile · tbl_map_agent_skill 요약.
export type Approval = { action: string; policy: "승인 필요" | "자동" | "차단"; who: string; notify: boolean };
export type AgentConfig = {
	skills: string[];
	mcp: string[];
	/** 1 읽기 전용 · 2 제안만 · 3 워크스페이스 쓰기 · 4 자율. */
	trust: 1 | 2 | 3 | 4;
	include: string[];
	exclude: string[];
	approvals: Approval[];
	/** 커밋 · PR 작성 계정. */
	github: "bot" | "mine";
	/** 실행 방식 (.pen Harness). 실행기 · 모델은 템플릿 · 멤버 쪽 값을 쓴다. */
	harness: {
		effort: "Auto" | "Low" | "Medium" | "High";
		/** 폴백 순서 (fallbackSteps 키). */
		fallback: ("sub" | "omni" | "local")[];
		maxTime: string;
		maxTurns: number;
		resume: boolean;
		autoResume: boolean;
	};
};

const cfg = (skills: string[], mcp: string[], trust: number, include: string[]): AgentConfig => ({
	skills,
	mcp,
	trust: trust as AgentConfig["trust"],
	include,
	exclude: ["**/.env*", "**/secrets/**", ...include.map((g) => g.replace("**", "node_modules/**"))],
	approvals: [
		{ action: "PR 생성", policy: "승인 필요", who: "나", notify: true },
		{ action: "새 의존성 추가", policy: "승인 필요", who: "Orch → 나", notify: true },
		{ action: "팀 외부로 메시지", policy: "자동", who: "—", notify: true },
		{ action: ".env 파일 접근", policy: "차단", who: "—", notify: true },
		{ action: "20분 이상 Run 연장", policy: "승인 필요", who: "나", notify: true },
	],
	github: "bot",
	harness: { effort: "Auto", fallback: ["sub", "omni", "local"], maxTime: "2h", maxTurns: 80, resume: true, autoResume: true },
});

/// 폴백 단계 (.pen FallbackStep). sub는 실행기의 구독 연결.
export const fallbackSteps = {
	sub: { kind: "구독", desc: "기본 · 주간 잔량 20% 미만이면 다음으로", cost: "—" },
	omni: { name: "OmniRoute", kind: "게이트웨이", desc: "한도 초과 · 오류 시 · OpenAI 호환 모델로 라우팅", cost: "$30/월" },
	local: { name: "Ollama · qwen3-coder", kind: "로컬", desc: "모두 실패 시 · L0 내부 작업만", cost: "무료" },
} as const;

/// 모델 목록 (.pen 모델 선택 다이얼로그). 연결별로 묶는다. runtime은 그 모델을 돌리는 실행기.
export type ModelInfo = { name: string; vendor: string; price: string; ctx: string; speed: string; tags: ("추천" | "Tested" | "NEW")[]; cost: number; ctxK: number };
export const modelCatalog: { key: string; label: string; runtime: Runtime; kind: "구독" | "게이트웨이"; models: ModelInfo[] }[] = [
	{ key: "codex", label: "Codex · ChatGPT Pro", runtime: "codex", kind: "구독", models: [
		{ name: "gpt-5", vendor: "OpenAI · 코딩 · 도구 사용", price: "$1.25 / $10", ctx: "400K", speed: "보통", tags: ["추천", "Tested"], cost: 1.25, ctxK: 400 },
		{ name: "gpt-5-codex", vendor: "OpenAI · 코딩 특화", price: "$1.25 / $10", ctx: "400K", speed: "보통", tags: ["NEW", "Tested"], cost: 1.25, ctxK: 400 },
		{ name: "gpt-5-mini", vendor: "OpenAI · 가벼운 작업", price: "$0.25 / $2", ctx: "400K", speed: "빠름", tags: [], cost: 0.25, ctxK: 400 },
	] },
	{ key: "claude", label: "Anthropic · Max", runtime: "claude", kind: "구독", models: [
		{ name: "claude-sonnet-5", vendor: "Anthropic · 코딩 · 도구 사용", price: "$3 / $15", ctx: "1M", speed: "보통", tags: ["추천", "Tested"], cost: 3, ctxK: 1000 },
		{ name: "claude-opus-5.5", vendor: "Anthropic · 어려운 설계 · 리뷰", price: "$15 / $75", ctx: "1M", speed: "느림", tags: ["Tested"], cost: 15, ctxK: 1000 },
		{ name: "claude-haiku-4.5", vendor: "Anthropic · 가벼운 작업", price: "$1 / $5", ctx: "200K", speed: "빠름", tags: [], cost: 1, ctxK: 200 },
	] },
	{ key: "omni", label: "OmniRoute", runtime: "codex", kind: "게이트웨이", models: [
		{ name: "deepseek-v3.1", vendor: "DeepSeek · OpenAI 호환", price: "$0.27 / $1.1", ctx: "128K", speed: "빠름", tags: ["NEW"], cost: 0.27, ctxK: 128 },
		{ name: "qwen3-coder", vendor: "Qwen · 코딩 특화", price: "$0.4 / $1.6", ctx: "256K", speed: "보통", tags: [], cost: 0.4, ctxK: 256 },
	] },
];

/// 워크스페이스 스킬 라이브러리 (설치된 스킬). source는 가져온 곳, tok은 Run당 컨텍스트 추가량.
export type Skill = {
	name: string;
	desc: string;
	/** Team = orchstack/team-skills, Local = ~/.orch/skills. */
	source: "Team" | "skills.sh" | "Local" | "Marketplace" | "Built-in";
	version?: string;
	tok: number;
	/** false면 워크스페이스에서 꺼짐 — 모든 멤버에서 빠진다. */
	on?: boolean;
	/** 새 버전 (업데이트 가능). */
	update?: string;
	/** 차단 사유 (보안 검사 실패 등). 차단되면 켤 수 없다. */
	blocked?: string;
};
export const skillLibrary: Skill[] = [
	{ name: "svelte-ui", desc: "SvelteKit + shadcn-svelte 컴포넌트 규칙과 예제. UI 태스크에서 자동 사용.", source: "Team", version: "v0.8", update: "v0.9", tok: 420 },
	{ name: "git-workflow", desc: "브랜치 · 커밋 · PR 규칙. feat/<issue>-<slug> 브랜치 생성.", source: "Built-in", tok: 180 },
	{ name: "a11y-check", desc: "입력 라벨 · 대비 · 키보드 접근성 점검 체크리스트.", source: "skills.sh", version: "v1.2", tok: 260 },
	{ name: "storybook-writer", desc: "컴포넌트 스토리 파일을 자동 작성.", source: "Marketplace", version: "v0.4", tok: 310, on: false },
	{ name: "i18n-sync", desc: "번역 키 누락을 찾아 ko/en 파일을 동기화.", source: "Local", tok: 150 },
	{ name: "rust-api", desc: "Axum 핸들러 · 에러 · 테스트 규칙.", source: "Team", tok: 380 },
	{ name: "sql-review", desc: "마이그레이션 · 인덱스 · 쿼리 점검.", source: "Local", tok: 240 },
	{ name: "playwright", desc: "E2E 시나리오 작성 · 실행 규칙.", source: "Built-in", tok: 300 },
	{ name: "test-plan", desc: "완료 조건에서 테스트 계획을 만든다.", source: "Team", tok: 200 },
	{ name: "code-review", desc: "리뷰 체크리스트 · 반려 사유 형식.", source: "Built-in", tok: 260 },
	{ name: "design-tokens", desc: "색 · 타이포 · 간격 토큰 규칙.", source: "Team", tok: 220 },
	{ name: "figma-export", desc: "Figma 컴포넌트를 코드 스펙으로 옮긴다.", source: "Marketplace", version: "v1.1", tok: 280 },
	{ name: "dep-audit", desc: "의존성 취약점 점검.", source: "Built-in", tok: 190 },
	{ name: "secret-scan", desc: "비밀키 노출 점검.", source: "Built-in", tok: 160 },
	{ name: "prompt-injector", desc: "외부 프롬프트를 지침에 삽입.", source: "skills.sh", version: "v0.3", tok: 0, on: false, blocked: "보안 검사 실패 · 외부 URL 실행" },
];

/// 스킬 · MCP 변경 기록 (.pen Skills & MCP · 최근 변경). 설치 · 업데이트 · 차단.
export type SkillLog = { kind: "INSTALL" | "UPDATE" | "BLOCK" | "TOGGLE" | "AUTH"; who: string; when: string; text: string };
export const skillLog: SkillLog[] = [
	{ kind: "INSTALL", who: "나", when: "오늘 10:12", text: "a11y-check v1.2 설치 · skills.sh · 보안 검사 통과" },
	{ kind: "UPDATE", who: "Orch", when: "어제 21:00", text: "rust-api 0.4 → 0.5 · team-skills 동기화" },
	{ kind: "BLOCK", who: "시스템", when: "9/27 15:40", text: "prompt-injector 추가 차단 · 보안 검사 실패 (외부 URL 실행)" },
];

/// skills.sh 검색 결과 (.pen 스킬 추가 · skills.sh 탐색). installs · 지원 에이전트 수 · 보안 검사.
export type SkillHit = { name: string; repo: string; desc: string; installs: string; agents: number; audit: [number, number]; tok: number; tags: ("trending" | "official" | "test" | "a11y")[]; version: string; license: string; skillMd: string[] };
export const skillsSh: SkillHit[] = [
	{ name: "svelte-best-practices", repo: "sveltejs/ai-tools", desc: "Svelte 5 runes · SvelteKit 라우팅 규칙과 흔한 실수를 정리한 스킬", installs: "24.1K", agents: 8, audit: [3, 3], tok: 900, tags: ["trending", "official"], version: "v1.4.2 · 3일 전", license: "MIT",
		skillMd: ["---", "name: svelte-best-practices", "description: Svelte 5 · SvelteKit 코드를 쓰거나 고칠 때 사용", "---", "", "# Svelte 5 best practices", "", "## 언제 쓰나요", "- .svelte · +page.ts · +layout.svelte 파일을 만들거나 고칠 때", "- stores를 runes로 옮길 때", "", "## 규칙", "- 상태는 $state, 계산 값은 $derived — writable store 대신", "- load에서는 event.fetch 사용 · 서버 전용 코드는 +page.server.ts", "- 이벤트는 onclick 속성 — on:click 쓰지 않기"] },
	{ name: "svelte-ui", repo: "orchstack/team-skills", desc: "shadcn-svelte 컴포넌트 재사용 규칙 · 이 팀 표준", installs: "팀", agents: 4, audit: [3, 3], tok: 400, tags: ["official"], version: "v2.0 · 1주 전", license: "Private", skillMd: ["# svelte-ui", "- 기존 ui 컴포넌트를 먼저 찾는다"] },
	{ name: "web-accessibility", repo: "anthropics/skills", desc: "WCAG 2.2 기준 점검 · aria/포커스/대비 체크리스트", installs: "61.3K", agents: 8, audit: [3, 3], tok: 1100, tags: ["trending", "official", "a11y"], version: "v3.1.0 · 1주 전", license: "Apache-2.0", skillMd: ["# Web accessibility", "- WCAG 2.2 AA 기준으로 점검한다"] },
	{ name: "playwright-testing", repo: "microsoft/playwright-skills", desc: "Playwright 테스트 작성 · 로케이터 · 흔한 flaky 원인", installs: "18.7K", agents: 6, audit: [3, 3], tok: 700, tags: ["test", "official"], version: "v0.9.0 · 2주 전", license: "MIT", skillMd: ["# Playwright testing", "- getByRole 로케이터를 우선한다"] },
	{ name: "svelte-motion-tricks", repo: "someone/svelte-motion", desc: "Svelte transition · spring 예제 모음", installs: "2.3K", agents: 2, audit: [2, 3], tok: 1400, tags: [], version: "v0.2.1 · 5개월 전", license: "MIT", skillMd: ["# svelte-motion-tricks"] },
];

/// MCP 서버 (.pen Tools & MCP). 설치 = 매 Run 컨텍스트에 도구 추가, 접근 가능 = 허용만.
export const mcpServers = [
	{ name: "playwright-mcp", desc: "브라우저 조작 · 스크린샷", tools: 12, tok: 220 },
	{ name: "github-mcp", desc: "이슈 · PR 조회와 코멘트", tools: 9, tok: 140 },
	{ name: "figma-mcp", desc: "디자인 파일 읽기", tools: 6, tok: 180 },
	{ name: "postgres-mcp", desc: "DB 스키마 조회 (읽기 전용)", tools: 4, tok: 160, auth: true },
] as { name: string; desc: string; tools: number; tok: number; auth?: boolean }[];

/// 스킬 소스 (.pen 스킬 소스 연동). 설치 정책은 워크스페이스 전체에 적용.
export const skillSources = {
	sources: [
		{ kind: "web", name: "skills.sh", state: "연결됨", desc: "공개 스킬 디렉터리 · 검색 · 보안 검사 결과 조회", sync: "API · 3분 전" },
		{ kind: "github", name: "orchstack/team-skills", state: "연결됨", desc: "팀 스킬 저장소 · main 브랜치 · 팀 스킬 2", sync: "동기화 12분 전" },
		{ kind: "folder", name: "~/.orch/skills", state: "로컬", desc: "로컬 폴더 · 이 기기에서만 · 새 스킬 작성 시 저장", sync: "스킬 1" },
	],
	tool: { name: "npx skills", version: "v1.3.0" },
	policy: [
		{ name: "보안 검사 통과한 스킬만 추가", desc: "3곳 중 경고가 있으면 추가 버튼 대신 ‘검토 후 추가’로 표시", on: true },
		{ name: "추가 전 승인 받기", desc: "멤버 · Orch가 스킬을 제안하면 나에게 승인 요청 (L3)", on: true },
		{ name: "버전 고정 (sha256)", desc: "설치 시점 버전을 고정 · 새 버전은 알림으로만", on: true },
		{ name: "업데이트 알림", desc: "주 1회 · 사용 중인 스킬에 새 버전이 있으면 PM Dock 카드로", on: true },
	],
};

/// 에이전트 템플릿 (.pen TemplateListItem · 멤버 추가 1단계). 스키마 tbl_agent_profile 요약. 멤버는 템플릿을 복사해 만든다.
export type Template = Pick<ApiTemplate, "sn" | "name"> & {
	role: Role;
	version: number;
	/** 한 줄 요약 (역할 · 전문). */
	focus: string;
	desc: string;
	runtime: Runtime;
	model: string;
	members: number;
	/** 스킬 · MCP · 권한 기본값. 멤버는 추가 시 복사한다. */
	config: AgentConfig;
	success?: number;
	/** 복사되는 Instructions 파일 (배포 중인 버전). */
	files: { name: string; body: string }[];
	/** 목록 묶음 (개발 · 검증 · 디자인 · PM). */
	group: string;
	tags: string[];
	created: string;
	/** 게시 전 새 버전 초안. */
	draft?: { name: string; body: string }[];
	revisions: { v: number; state: "draft" | "live" | "old"; who: string; when: string; note: string }[];
};

const agentMd = (title: string, lines: string[]) =>
	[`# ${title}`, "", `당신은 {{team.name}} 의 ${title}입니다.`, ...lines, "", "## 완료 보고", "완료 시 Orch에게 아래 형식으로 요약을 남깁니다.", "@include templates/report.md"].join("\n");

const feAgent = [
	"# Frontend Developer", "", "당신은 {{team.name}} 의 프론트엔드 개발자입니다.", "SvelteKit + shadcn-svelte 기반 UI를 구현합니다.", "",
	"## 작업 원칙", "- 태스크의 **완료 조건**을 모두 만족할 때만 Done 처리", "- 컴포넌트는 `src/lib/ui` 의 기존 것을 우선 재사용", "- 모든 입력에 `<label>` 연결", "- lint · test 통과 전 PR 생성 금지", "",
	"## 완료 보고", "완료 시 Orch에게 아래 형식으로 요약을 남깁니다.", "@include templates/report.md", "@include rules/accessibility.md", "",
	"## 금지", "- `frontend/**` 밖 파일 수정", "- 새 의존성 추가 (Orch 승인 필요)",
].join("\n");
const feSoul = ["# Soul", "", "## 말투", "- 간결하고 차분하게, 근거와 함께 보고", "- 모르면 추측하지 않고 질문한다", "", "## 보고", "- 완료 보고는 “무엇을 · 왜 · 남은 것” 3줄", "", "## 협업", "- 완료 보고 시 테스트 방법을 함께 적는다"].join("\n");
const feA11y = ["# 접근성 규칙", "- 색 대비 4.5:1 이상", "- 키보드만으로 모든 동작 가능", "- 아이콘 버튼에 aria-label"].join("\n");
const feFiles = [
	{ name: "AGENT.md", body: feAgent },
	{ name: "SOUL.md", body: feSoul },
	{ name: "rules/code-style.md", body: "# 코드 스타일\n- Svelte 5 runes 사용\n- 컴포넌트 props는 타입을 명시\n- Tailwind 임의 값 대신 토큰 사용" },
	{ name: "rules/accessibility.md", body: feA11y },
	{ name: "templates/report.md", body: "## 완료 보고\n- 무엇을:\n- 왜:\n- 남은 것:" },
	{ name: "templates/pr-description.md", body: "## 변경\n\n## 확인 방법\n\n## 스크린샷" },
];
const withLines = (files: typeof feFiles, name: string, edit: (lines: string[]) => string[]) =>
	files.map((f) => (f.name === name ? { ...f, body: edit(f.body.split("\n")).join("\n") } : f));
const labelRules = (l: string[]) => [...l, "- 입력마다 보이는 라벨 · for/id 연결", "- 오류 문구는 aria-describedby로 연결"];
const feDraft = withLines(withLines(feFiles, "SOUL.md", (l) => [...l.slice(0, 5), "- 디자인 의도가 모호하면 유나에게 먼저 확인한다", ...l.slice(5)]), "rules/accessibility.md", labelRules);
/// 진의 지침 — 템플릿 v3 복사 후 SOUL.md · 접근성 규칙을 고쳤다.
export const jinFiles = withLines(
	withLines(feFiles, "SOUL.md", (l) => [...l.slice(0, 5), "- 디자인 의도가 모호하면 유나에게 먼저 확인한다", ...l.slice(5, 7), "- 완료 보고는 “무엇을 · 왜 · 남은 것 · 스크린샷”", ...l.slice(8), "- 완료 보고에 a11y 확인 항목을 체크리스트로 남긴다"]),
	"rules/accessibility.md",
	labelRules
);
coreMembers.find((m) => m.sn === 1)!.files = jinFiles;

export const templates: Template[] = [
	{
		sn: 1, name: "Frontend Developer", role: "frontend", version: 3, focus: "UI 구현 · 접근성", desc: "SvelteKit UI 구현 · 접근성 · 컴포넌트 재사용",
		runtime: "codex", model: "gpt-5", members: 3, config: cfg(["svelte-ui", "git-workflow", "a11y-check"], ["playwright-mcp", "github-mcp"], 3, ["frontend/**"]), success: 78,
		files: feFiles,
		group: "개발", tags: ["frontend", "svelte", "a11y"], created: "나 · 9/12", draft: feDraft,
		revisions: [
			{ v: 4, state: "draft", who: "나", when: "오늘 10:20", note: "SOUL: 디자인 의도 확인 · rules: 라벨 규칙 2줄 (진의 변경 제안 채택)" },
			{ v: 3, state: "live", who: "나", when: "9/20", note: "Harness 기본값 Codex CLI · gpt-5 · 스킬 a11y-check 추가" },
			{ v: 2, state: "old", who: "Orch", when: "9/12", note: "report.md 템플릿 분리 · 금지 규칙 정리" },
			{ v: 1, state: "old", who: "나", when: "9/02", note: "최초 작성" },
		],
	},
	{
		sn: 2, name: "Backend Developer", role: "backend", version: 4, focus: "API · DB · 성능", desc: "Rust API · SeaORM · 성능 개선",
		runtime: "claude", model: "claude-sonnet-5", members: 2, config: cfg(["rust-api", "git-workflow", "sql-review"], ["github-mcp"], 3, ["backend/**"]), success: 82,
		files: [
			{ name: "AGENT.md", body: agentMd("Backend Developer", ["Axum · SeaORM 기반 API를 구현합니다.", "", "## 작업 원칙", "- 마이그레이션은 되돌릴 수 있게 작성", "- cargo test 통과 전 PR 생성 금지"]) },
			{ name: "SOUL.md", body: "# Soul\n- 수치와 로그로 근거를 남긴다\n- 위험한 변경은 먼저 묻는다" },
		],
		group: "개발", tags: ["backend", "rust", "db"], created: "나 · 9/18",
		revisions: [{ v: 4, state: "live", who: "나", when: "9/18", note: "현재 버전" }, { v: 3, state: "old", who: "나", when: "9/03", note: "지침 정리" }, { v: 2, state: "old", who: "나", when: "9/02", note: "지침 정리" }, { v: 1, state: "old", who: "나", when: "9/01", note: "최초 작성" }],
	},
	{
		sn: 3, name: "QA Engineer", role: "qa", version: 2, focus: "테스트 · E2E · 회귀", desc: "Playwright E2E · 회귀 테스트",
		runtime: "claude", model: "claude-sonnet-5", members: 1, config: cfg(["playwright", "test-plan"], ["playwright-mcp"], 2, ["tests/**"]), success: 91,
		files: [
			{ name: "AGENT.md", body: agentMd("QA Engineer", ["완료 조건을 기준으로 E2E · 회귀 테스트를 작성하고 실행합니다."]) },
			{ name: "SOUL.md", body: "# Soul\n- 재현 절차를 먼저 적는다" },
		],
		group: "검증", tags: ["qa", "e2e"], created: "나 · 9/15",
		revisions: [{ v: 2, state: "live", who: "나", when: "9/15", note: "현재 버전" }, { v: 1, state: "old", who: "나", when: "9/01", note: "최초 작성" }],
	},
	{
		sn: 4, name: "Reviewer", role: "reviewer", version: 2, focus: "코드 리뷰 · 규칙", desc: "PR 리뷰 · 규칙 점검",
		runtime: "claude", model: "claude-opus-5.5", members: 2, config: cfg(["code-review", "git-workflow"], ["github-mcp"], 1, []), success: 88,
		files: [
			{ name: "AGENT.md", body: agentMd("Reviewer", ["PR을 검토하고 승인 · 반려 사유를 남깁니다."]) },
			{ name: "SOUL.md", body: "# Soul\n- 근거 없는 지적은 하지 않는다" },
		],
		group: "검증", tags: ["review"], created: "나 · 9/10",
		revisions: [{ v: 2, state: "live", who: "나", when: "9/10", note: "현재 버전" }, { v: 1, state: "old", who: "나", when: "9/01", note: "최초 작성" }],
	},
	{
		sn: 5, name: "Product Designer", role: "designer", version: 1, focus: "화면 설계 · 토큰", desc: "화면 설계 · 디자인 토큰 관리",
		runtime: "codex", model: "gpt-5", members: 1, config: cfg(["design-tokens", "figma-export"], [], 2, ["design/**"]), success: 74,
		files: [
			{ name: "AGENT.md", body: agentMd("Product Designer", ["화면 설계와 디자인 토큰을 관리합니다."]) },
			{ name: "SOUL.md", body: "# Soul\n- 사용자 흐름부터 설명한다" },
		],
		group: "디자인 · PM", tags: ["design", "tokens"], created: "나 · 9/08",
		revisions: [{ v: 1, state: "live", who: "나", when: "9/08", note: "현재 버전" }],
	},
	{
		sn: 6, name: "Security Reviewer", role: "reviewer", version: 1, focus: "초안 · 의존성 · 비밀키 점검", desc: "의존성 · 비밀키 · 권한 점검",
		runtime: "claude", model: "claude-opus-5.5", members: 0, config: cfg(["dep-audit", "secret-scan"], ["github-mcp"], 1, []),
		files: [
			{ name: "AGENT.md", body: agentMd("Security Reviewer", ["의존성 취약점과 비밀키 노출을 점검합니다."]) },
			{ name: "SOUL.md", body: "# Soul\n- 위험도를 먼저 말한다" },
		],
		group: "검증", tags: ["security"], created: "나 · 9/25",
		revisions: [{ v: 1, state: "live", who: "나", when: "9/25", note: "현재 버전" }],
	},
];

/// Orch 추천 (멤버 추가 1단계 상단). 대기열을 보고 부족한 역할을 권한다.
export const recommend = { template: 1, reason: "Core Team 대기열에 UI 태스크 3개, 담당 1명" };

/// 멤버 상세 (.pen Teams · 멤버 상세). 스키마 tbl_task · tbl_run · tbl_log_activity · tbl_decision 요약. 상세가 없는 멤버는 요약만 보인다.
export type MemberRun = {
	num: number;
	task: string;
	status: "running" | "failed" | "done" | "cancelled";
	start: string;
	dur: string;
	/** 토큰 (K). */
	tokens: number;
	result: string;
};

export type MemberActivity =
	| { type: "decision"; who: string; orch?: boolean; kind: string; text: string; time: string }
	| { type: "event"; who: string; role: Role; kind: "지시" | "실패" | "메시지" | "배정" | "반려" | "리뷰" | "시스템"; title: string; time: string; note?: string };

export type MemberDetail = {
	/** 원본 템플릿 번호. */
	template: number;
	week: { done: number; doneDelta: string; cycle: string; tokens: number; days: number[] };
	now?: { task: number; pct: number; run: number; elapsed: string; branch: string; commits: number; eta: string };
	attention: { kind: "quota" | "wait" | "review"; title: string; sub: string; link: string }[];
	queue: { num: number; title: string; note: string; blocked?: boolean; issue: string; est: string; priority: string }[];
	done: { num: number; title: string; tokens: string; when: string }[];
	diff: { file: string; note: string }[];
	runs: MemberRun[];
	/** 최근 7일 Run 수 (성공 · 실패). */
	runDays: [number, number][];
	failure?: {
		run: number;
		meta: string;
		steps: { name: string; time: string; state: "done" | "failed" | "skipped" }[];
		errors: string[];
		retry: string;
		files: { kind: "M" | "A"; path: string; diff: string }[];
	};
	activity: { day: string; items: MemberActivity[] }[];
	relations: { name: string; role: Role; label: string; note: string; value: number }[];
	rules: { sort: string; autostart: boolean; rebalance: boolean; runs: string };
	source: { orch: number; manual: number };
};

export const memberDetails: Record<number, MemberDetail> = {
	1: {
		template: 1,
		week: { done: 3, doneDelta: "+1", cycle: "2h 40m", tokens: 175, days: [0, 42, 0, 0, 88, 26, 19] },
		now: { task: 129, pct: 62, run: 81, elapsed: "15m", branch: "feat/login-ui", commits: 3, eta: "~2h" },
		attention: [
			{ kind: "quota", title: "Codex 계정 주간 잔량 18%", sub: "현재 속도면 목요일 오후 소진 · 진의 몫 62%", link: "정책 보기" },
			{ kind: "wait", title: "#133 이 #128 완료를 기다림", sub: "민수 · Auth API 92% 컨텍스트 경고 중", link: "#128 보기" },
			{ kind: "review", title: "소라가 #121 리뷰 3회 반려 → 루프 가드 정지", sub: "완료 조건에 접근성 기준 추가 후 통과", link: "리뷰 보기" },
		],
		queue: [
			{ num: 133, title: "Signup 폼 접근성 개선", note: "#128 Auth API 완료 대기 — 민수 진행 중", blocked: true, issue: "#51", est: "est 1h", priority: "P2" },
			{ num: 136, title: "Password reset 화면", note: "바로 시작 가능", issue: "#48", est: "est 2h", priority: "P3" },
		],
		done: [
			{ num: 121, title: "Signup UI", tokens: "61.2K tok", when: "3일 전 · 42m" },
			{ num: 118, title: "Nav 리팩토링", tokens: "88.0K tok", when: "4일 전 · 1h 5m" },
			{ num: 116, title: "Toast 컴포넌트", tokens: "22.4K tok", when: "5일 전 · 28m" },
		],
		diff: [
			{ file: "SOUL.md", note: "+ 디자인 의도가 모호하면 유나에게 먼저 확인" },
			{ file: "rules/accessibility.md", note: "라벨 규칙 2줄 추가 (#121 반려 이후)" },
		],
		runs: [
			{ num: 81, task: "#129 Login UI 구현", status: "running", start: "오늘 14:15", dur: "15m", tokens: 18.4, result: "진행 중 · 62%" },
			{ num: 77, task: "#129 Login UI 구현", status: "failed", start: "오늘 13:52", dur: "4m", tokens: 6.3, result: "lint error 3건" },
			{ num: 74, task: "#121 Signup UI", status: "done", start: "9/26 16:10", dur: "42m", tokens: 61.2, result: "tests 12/12 · PR #86" },
			{ num: 70, task: "#118 Nav 리팩토링", status: "done", start: "9/25 10:02", dur: "1h 5m", tokens: 88.0, result: "PR #84" },
			{ num: 66, task: "#118 Nav 리팩토링", status: "cancelled", start: "9/25 09:40", dur: "2m", tokens: 1.2, result: "사용자가 중지" },
		],
		runDays: [[0, 0], [0, 0], [0, 0], [0, 0], [2, 1], [1, 0], [0, 1]],
		failure: {
			run: 77,
			meta: "#129 Login UI 구현 · 오늘 13:52 · 4m · 6.3K tok · gpt-5",
			steps: [
				{ name: "Checkout feat/login-ui", time: "2s", state: "done" },
				{ name: "컨텍스트 로드 · 14 files", time: "11s", state: "done" },
				{ name: "구현 · 4 files 변경", time: "3m 20s", state: "done" },
				{ name: "pnpm lint", time: "18s", state: "failed" },
				{ name: "pnpm test", time: "건너뜀", state: "skipped" },
			],
			errors: ["LoginForm.svelte:42  a11y-label-has-associated-control", "LoginForm.svelte:58  no-unused-vars 'isLoading'", "auth.ts:17  @typescript-eslint/no-explicit-any"],
			retry: "Orch가 lint 수정 지시와 함께 Run #81로 재시도",
			files: [
				{ kind: "M", path: "src/routes/login/LoginForm.svelte", diff: "+184 −12" },
				{ kind: "A", path: "src/lib/auth/auth.ts", diff: "+96" },
				{ kind: "M", path: "src/lib/ui/Input.svelte", diff: "+8 −3" },
				{ kind: "A", path: "src/routes/login/+page.ts", diff: "+22" },
			],
		},
		activity: [
			{
				day: "오늘",
				items: [
					{ type: "decision", who: "나", kind: "결정", text: "#133 재전송 → 30초 쿨다운", time: "14:12" },
					{ type: "decision", who: "Orch", orch: true, kind: "대신 결정", text: "#130 E2E 브라우저 → Chromium + WebKit", time: "13:58" },
					{ type: "event", who: "Orch", role: "orch", kind: "지시", title: "#129 재시도 지시", time: "14:15", note: "“lint 규칙 준수 후 다시 진행. a11y 라벨 필수”" },
					{ type: "event", who: "진", role: "frontend", kind: "실패", title: "Run #77 실패 · lint 3건", time: "13:56" },
					{ type: "event", who: "진", role: "frontend", kind: "메시지", title: "민수에게 질문", time: "13:30", note: "“refresh 응답의 expires_in 단위가 초 맞나요?” → 민수: “네, 초 단위”" },
					{ type: "event", who: "Orch", role: "orch", kind: "배정", title: "Orch가 #129를 진에게 배정 (디자인 승인 후)", time: "11:02", note: "Figma 링크 · 컴포넌트 3개" },
				],
			},
			{
				day: "어제",
				items: [
					{ type: "event", who: "소라", role: "reviewer", kind: "반려", title: "#121 리뷰 반려", time: "17:40", note: "접근성 라벨 누락 2곳" },
					{ type: "event", who: "Orch", role: "orch", kind: "배정", title: "#121 완료 → Orch가 하린에게 QA 배정", time: "18:25" },
					{ type: "event", who: "나", role: "agent", kind: "지시", title: "#133 우선순위 P3 → P2", time: "16:02" },
				],
			},
		],
		relations: [
			{ name: "하린", role: "qa", label: "QA", note: "같은 이슈 3", value: 80 },
			{ name: "민수", role: "backend", label: "Backend", note: "메시지 6 · 병렬 작업", value: 100 },
			{ name: "소라", role: "reviewer", label: "Reviewer", note: "리뷰 2 · 반려 1", value: 55 },
			{ name: "유나", role: "designer", label: "Design", note: "같은 이슈 2", value: 40 },
		],
		rules: { sort: "우선순위 → 의존 해소", autostart: true, rebalance: true, runs: "1 Run (개별)" },
		source: { orch: 18, manual: 4 },
	},
};

/// 다른 프로젝트의 태스크 (.pen Tasks · 전체 태스크의 Project B · C 묶음). 담당은 해당 팀 멤버 sn.
export const projectTasks: Task[] = [
	{ num: 212, project: 2, title: "Settings 레이아웃", status: "in_progress", priority: "P1", agent: 6, issue: 0, steps: [2, 4], tokens: "12.3K", messages: 4, model: "gpt-5", run: "12m", updated: "20m ago" },
	{ num: 208, project: 2, title: "Token 문서화", status: "review", priority: "P2", agent: 7, issue: 0, steps: [3, 3], tokens: "7.9K", messages: 2, model: "claude-sonnet-5", updated: "3h ago" },
	{ num: 213, project: 2, title: "회귀 테스트", status: "todo", priority: "P2", agent: 8, issue: 0, steps: [0, 5], messages: 0, model: "claude-sonnet-5", updated: "1h ago" },
	{ num: 301, project: 3, title: "푸시 알림 권한 화면", status: "todo", priority: "P1", agent: 9, issue: 0, steps: [0, 4], messages: 1, model: "gpt-5", updated: "45m ago" },
	{ num: 302, project: 3, title: "오프라인 캐시", status: "backlog", priority: "P3", issue: 0, steps: [0, 0], messages: 0, updated: "2d ago" },
];
