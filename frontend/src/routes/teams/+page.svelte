<script lang="ts">
	/// Teams — 팀 헤더 · KPI · 멤버 표 · 작업량 · 우측 사용량 / Orch 진행 / 팀 정책.
	import { page } from '$app/state';
	import Plus from '@lucide/svelte/icons/plus';
	import Search from '@lucide/svelte/icons/search';
	import ChevronRight from '@lucide/svelte/icons/chevron-right';
	import Pencil from '@lucide/svelte/icons/pencil';
	import UserPlus from '@lucide/svelte/icons/user-plus';
	import LoaderCircle from '@lucide/svelte/icons/loader-circle';
	import Hourglass from '@lucide/svelte/icons/hourglass';
	import CirclePause from '@lucide/svelte/icons/circle-pause';
	import CircleDashed from '@lucide/svelte/icons/circle-dashed';
	import SquareCheck from '@lucide/svelte/icons/square-check';
	import Terminal from '@lucide/svelte/icons/terminal';
	import TriangleAlert from '@lucide/svelte/icons/triangle-alert';
	import Filter from '@lucide/svelte/icons/filter';
	import Route from '@lucide/svelte/icons/route';
	import KeyRound from '@lucide/svelte/icons/key-round';
	import Timer from '@lucide/svelte/icons/timer';
	import ShieldCheck from '@lucide/svelte/icons/shield-check';
	import Coins from '@lucide/svelte/icons/coins';
	import Layers from '@lucide/svelte/icons/layers';
	import Play from '@lucide/svelte/icons/play';
	import GitMerge from '@lucide/svelte/icons/git-merge';
	import GitBranch from '@lucide/svelte/icons/git-branch';
	import Sparkles from '@lucide/svelte/icons/sparkles';
	import CircleCheck from '@lucide/svelte/icons/circle-check';
	import Circle from '@lucide/svelte/icons/circle';
	import Info from '@lucide/svelte/icons/info';
	import ArrowRight from '@lucide/svelte/icons/arrow-right';
	import ArrowLeft from '@lucide/svelte/icons/arrow-left';
	import FileText from '@lucide/svelte/icons/file-text';
	import Cpu from '@lucide/svelte/icons/cpu';
	import Gauge from '@lucide/svelte/icons/gauge';
	import ListChecks from '@lucide/svelte/icons/list-checks';
	import Pause from '@lucide/svelte/icons/pause';
	import Copy from '@lucide/svelte/icons/copy';
	import FolderGit2 from '@lucide/svelte/icons/folder-git-2';
	import GitPullRequest from '@lucide/svelte/icons/git-pull-request';
	import Plug from '@lucide/svelte/icons/plug';
	import CirclePlay from '@lucide/svelte/icons/circle-play';
	import Activity from '@lucide/svelte/icons/activity';
	import BookOpen from '@lucide/svelte/icons/book-open';
	import Library from '@lucide/svelte/icons/library';
	import SlidersHorizontal from '@lucide/svelte/icons/sliders-horizontal';
	import Wrench from '@lucide/svelte/icons/wrench';
	import LayoutTemplate from '@lucide/svelte/icons/layout-template';
	import CircleX from '@lucide/svelte/icons/circle-x';
	import CircleSlash from '@lucide/svelte/icons/circle-slash';
	import CircleDot from '@lucide/svelte/icons/circle-dot';
	import Link2 from '@lucide/svelte/icons/link-2';
	import Undo2 from '@lucide/svelte/icons/undo-2';
	import Redo2 from '@lucide/svelte/icons/redo-2';
	import BookMarked from '@lucide/svelte/icons/book-marked';
	import GitCompare from '@lucide/svelte/icons/git-compare';
	import ArrowDownWideNarrow from '@lucide/svelte/icons/arrow-down-wide-narrow';
	import Shuffle from '@lucide/svelte/icons/shuffle';
	import MessageSquarePlus from '@lucide/svelte/icons/message-square-plus';
	import MessageCircle from '@lucide/svelte/icons/message-circle';
	import ArrowRightLeft from '@lucide/svelte/icons/arrow-right-left';
	import * as Empty from '$lib/components/ui/empty';
	import * as Dialog from '$lib/components/ui/dialog';
	import { goto } from '$app/navigation';
	import { MdEditor, estimateTokens, type MdFile } from '$lib/components/ui/md-editor';
	import Upload from '@lucide/svelte/icons/upload';
	import { Segmented } from '$lib/components/ui/segmented';
	import { Checkbox } from '$lib/components/ui/checkbox';
	import CircleAlert from '@lucide/svelte/icons/circle-alert';
	import GitFork from '@lucide/svelte/icons/git-fork';
	import { Switch } from '$lib/components/ui/switch';
	import Hand from '@lucide/svelte/icons/hand';
	import Zap from '@lucide/svelte/icons/zap';
	import Repeat from '@lucide/svelte/icons/repeat';
	import ListPlus from '@lucide/svelte/icons/list-plus';
	import Moon from '@lucide/svelte/icons/moon';
	import type { Component } from 'svelte';
	import * as Sheet from '$lib/components/ui/sheet';
	import * as Select from '$lib/components/ui/select';
	import { Steps } from '$lib/components/ui/steps';
	import { Input } from '$lib/components/ui/input';
	import { Badge } from '$lib/components/ui/badge';
	import type { Role } from '$lib/roles';
	import type { Runtime } from '$lib/components/ui/runtime-logo';
	import * as Avatar from '$lib/components/ui/avatar';
	import * as Card from '$lib/components/ui/card';
	import * as Table from '$lib/components/ui/table';
	import * as Alert from '$lib/components/ui/alert';
	import * as InputGroup from '$lib/components/ui/input-group';
	import { Button } from '$lib/components/ui/button';
	import { Toggle } from '$lib/components/ui/toggle';
	import { Pill } from '$lib/components/ui/pill';
	import { Progress } from '$lib/components/ui/progress';
	import { RoleAvatar } from '$lib/components/ui/role-avatar';
	import { RuntimeLogo } from '$lib/components/ui/runtime-logo';
	import { accounts, meters, teamPolicy, recommend, tasks, issues, memberDetails, type TeamMember, type OrchPolicy, type LevelAction, type SpawnMode } from '$lib/mock';
	import { store, defaultTeam, glyphs, glyphOf, runtimeName, accountOf, low, scopeText, liveMap, k, putDraft } from '$lib/teams.svelte';
	import { LimitRow } from '$lib/components/ui/limit-row';
	import { KeyValueRow } from '$lib/components/ui/key-value-row';
	import SkillsPanel from '$lib/components/orch/agent/skills-panel.svelte';
	import ToolsPanel from '$lib/components/orch/agent/tools-panel.svelte';
	import PermPanel from '$lib/components/orch/agent/perm-panel.svelte';
	import HarnessPanel from '$lib/components/orch/agent/harness-panel.svelte';
	import { cn } from '$lib/utils';
	import * as ChoiceCards from '$lib/components/ui/choice-cards';

	// 선택 팀은 ?team= 으로 둔다. 없으면 첫 프로젝트 팀. 팀 · 템플릿 데이터는 $lib/teams.svelte (서버 연결은 #45).
	const team = $derived(store.crew.find((t) => t.sn === Number(page.url.searchParams.get('team'))) ?? defaultTeam());

	type State = TeamMember['status'];
	const states: Record<State, { label: string; icon: Component; text: string }> = {
		running: { label: 'Running', icon: LoaderCircle, text: 'text-status-in-progress' },
		waiting: { label: 'Waiting', icon: Hourglass, text: 'text-status-waiting' },
		idle: { label: 'Idle', icon: CirclePause, text: 'text-status-backlog' },
	};

	let filter = $state<State | 'all'>('all');
	const members = $derived(team.members.filter((m) => filter === 'all' || m.status === filter));
	const count = (s: State) => team.members.filter((m) => m.status === s).length;

	// 작업량 막대 색 · 범례 (.pen Workload).
	const loads = {
		in_progress: { label: '진행', bg: 'bg-status-in-progress' },
		waiting: { label: '대기', bg: 'bg-status-waiting' },
		blocked: { label: '막힘', bg: 'bg-status-blocked' },
		todo: { label: 'Todo', bg: 'bg-status-todo' },
	};

	// L0–L4 색은 레벨 의미에 고정 (.pen LevelBadge).
	const levelTone = [
		{ bg: 'bg-status-done', text: 'text-status-done' },
		{ bg: 'bg-primary', text: 'text-primary' },
		{ bg: 'bg-status-waiting', text: 'text-status-waiting' },
		{ bg: 'bg-status-review', text: 'text-status-review' },
		{ bg: 'bg-destructive', text: 'text-destructive' },
	];

	const tokens = $derived(team.members.reduce((s, m) => s + m.tokens, 0));
	const running = $derived(count('running'));
	const hottest = $derived(team.members.reduce((a, b) => (b.context > a.context ? b : a)));
	const hot = $derived(hottest.context >= teamPolicy.contextWarn);
	const cool = $derived(team.members.filter((m) => m.context < 50).length);

	const weekOf = (m: TeamMember) => accounts.find((a) => a.runtime === m.runtime)?.week ?? 100;

	// Orch 진행 정책 (.pen Orch 진행 정책) — 팀마다 따로 둔다. 편집은 사본에서 하고 저장할 때 반영한다.
	const policy = $derived(store.policies[team.sn]);
	let draft = $state<OrchPolicy>();
	const modeLabel = (p: OrchPolicy) => (p.mode === 'manual' ? 'Manual' : p.mode === 'full' ? 'Full auto' : `Auto · ${p.timer}초 타이머`);
	const actionLabel = (p: OrchPolicy, a: LevelAction, timeout: string) =>
		a === 'auto' ? '자동' : a === 'timer' ? `타이머 ${p.timer}초` : a === 'block' ? '차단' : timeout ? `대기 · ${timeout} 후 Orch` : '대기';
	const streakLimit = (p: OrchPolicy) => Number.parseInt(p.guards.find((g) => g.key === 'streak')!.value);
	const actions = [
		{ value: 'auto', label: '자동' },
		{ value: 'timer', label: '타이머' },
		{ value: 'wait', label: '대기' },
		{ value: 'block', label: '차단' },
	];
	const waitOptions = ['', '10분', '30분', '1시간'];
	const guardIcon: Record<OrchPolicy['guards'][number]['key'], Component> = { streak: Repeat, reject: Undo2, repeat: Copy, budget: Coins, spawn: ListPlus, away: Moon };
	let customTimer = $state(false);
	// 하위 작업 정책 (.pen 4 · 하위 작업) — 기본 방식은 허용 방식 안에 있어야 하고, 리드당 동시 수는 1 이상 (#67 · tbl_team).
	const spawnModes: { value: SpawnMode; icon: typeof Cpu }[] = [{ value: 'sub', icon: GitBranch }, { value: 'fork', icon: GitFork }, { value: 'runner', icon: Cpu }];
	const spawnDesc: [SpawnMode, string][] = [['sub', '리드 Run 안에서 · 토큰은 리드 합계에 포함'], ['runner', '별도 Run · 등급은 작업 종류로 자동'], ['fork', '부모 컨텍스트 상속']];
	const spawnError = $derived(draft && !draft.spawn.allow.includes(draft.spawn.mode) ? '기본 방식은 허용 방식 안에 있어야 합니다' : '');
	const childError = $derived(draft && !(Number(draft.spawn.maxChild) >= 1) ? '리드당 동시 하위 작업은 1 이상이어야 해요' : '');
	function toggleAllow(d: OrchPolicy, m: SpawnMode, on: boolean) {
		d.spawn.allow = on ? [...d.spawn.allow, m] : d.spawn.allow.filter((x) => x !== m);
	}
	/// 정책 편집 열기 — 현재 팀 정책의 사본으로 시작한다.
	function editPolicy() {
		draft = structuredClone($state.snapshot(policy));
		customTimer = ![3, 5, 10, 30].includes(draft.timer);
	}
	function savePolicy() {
		if (draft) store.policies[team.sn] = draft;
		draft = undefined;
	}

	// 주간 잔량이 기준 미만인 연결 중 이 팀 멤버가 쓰는 것만 경고한다.
	const alerts = $derived(
		accounts
			.filter((a) => low(a.week))
			.map((a) => ({ a, hit: team.members.filter((m) => m.runtime === a.runtime) }))
			.filter((x) => x.hit.length)
	);

	// 멤버 추가 시트 (.pen Teams · 멤버 추가) — 1 템플릿 → 2 캐릭터 → 3 런타임 · 도구. 템플릿을 복사해 멤버를 만든다.
	let adding = $state(false);
	let step = $state(0);
	let tq = $state('');
	/** 고른 템플릿. null이면 빈 캐릭터. */
	let pick = $state<number | null>(recommend.template);
	const tpl = $derived(store.templates.find((t) => t.sn === pick));
	const tplShown = $derived(
		store.templates.filter((t) => `${t.name} ${t.focus} ${t.config.skills.join(' ')}`.toLowerCase().includes(tq.trim().toLowerCase()))
	);
	// 2단계에서 복사한 템플릿. 같은 템플릿으로 다시 오면 고친 내용을 유지한다.
	let copied = $state<number | null>();
	let name = $state('');
	let glyph = $state(0);
	let files = $state<{ name: string; body: string }[]>([]);
	let fileIdx = $state(0);
	let runtime = $state<Runtime>('codex');
	let model = $state('');
	let effort = $state('Auto');
	let first = $state<'orch' | 'task' | 'wait'>('orch');

	const role = $derived<Role>(tpl?.role ?? 'agent');
	const other = (r: Runtime): Runtime => (r === 'claude' ? 'codex' : 'claude');
	const models: Record<Runtime, string[]> = { claude: ['claude-sonnet-5', 'claude-opus-5.5', 'claude-haiku-4.5'], codex: ['gpt-5', 'gpt-5-mini'] };
	// 템플릿 기본 연결이 잔량 부족이면 폴백 2순위로 시작한다.
	const fellBack = $derived(!!tpl && tpl.runtime !== runtime && low(accountOf(tpl.runtime).week));
	// 폴백 순서: 템플릿 기본 연결 → 다른 구독 → 게이트웨이.
	const chain = $derived([accountOf(tpl?.runtime ?? 'claude'), accountOf(other(tpl?.runtime ?? 'claude'))]);
	const gateway = meters.find((m) => m.kind === 'route')!;
	const firstTask = tasks.find((t) => t.agent === undefined)!;
	const firstLabel = { orch: 'Orch에게 맡기기', task: `#${firstTask.num} ${firstTask.title}`, wait: '대기' };

	const tokAll = $derived(files.reduce((n, f) => n + estimateTokens(f.body), 0));
	/// 템플릿 대비 바뀐 파일 요약 (예: SOUL.md +1줄).
	const diff = $derived(
		files
			.map((f) => {
				const orig = tpl?.files.find((o) => o.name === f.name)?.body;
				if (orig === f.body) return undefined;
				const d = f.body.split('\n').length - (orig?.split('\n').length ?? 0);
				return `${f.name} ${d === 0 ? '수정' : `${d > 0 ? '+' : ''}${d}줄`}`;
			})
			.filter(Boolean)
			.join(' · ') || '같음'
	);

	/// 시트 열기. 매번 1단계 · Orch 추천 템플릿(또는 고른 템플릿)부터.
	function startAdd(from: number = recommend.template) {
		adding = true;
		step = 0;
		tq = '';
		pick = from;
		copied = undefined;
	}
	// 템플릿 화면의 "팀에 추가"는 ?add=템플릿 으로 온다 — 시트를 열고 주소에서 뗀다.
	$effect(() => {
		const from = Number(page.url.searchParams.get('add'));
		if (!from) return;
		startAdd(from);
		const url = new URL(page.url);
		url.searchParams.delete('add');
		goto(url, { replaceState: true, noScroll: true, keepFocus: true });
	});
	/// 2단계로 — 템플릿이 바뀌었으면 Instructions · 런타임을 다시 복사한다.
	function toCharacter() {
		if (copied !== pick) {
			copied = pick;
			name = '';
			glyph = 0;
			files = tpl ? tpl.files.map((f) => ({ ...f })) : [{ name: 'AGENT.md', body: '# 새 캐릭터\n\n당신은 {{team.name}} 의 멤버입니다.\n' }];
			fileIdx = 0;
			const base = tpl?.runtime ?? 'claude';
			runtime = low(accountOf(base).week) ? other(base) : base;
			model = tpl && runtime === tpl.runtime ? tpl.model : models[runtime][0];
			effort = 'Auto';
			first = 'orch';
		}
		step = 1;
	}
	/// 팀에 추가 (목데이터). 멤버 상세 이동은 T-3 (#63).
	function add() {
		const sn = Math.max(...store.crew.flatMap((t) => t.members.map((m) => m.sn))) + 1;
		team.members.push({
			sn,
			name: name.trim(),
			role,
			title: tpl?.name ?? 'Agent',
			runtime,
			model: `${runtimeName(runtime)} · ${model}`,
			status: 'idle',
			work: first === 'task' ? `다음: ${firstLabel.task}` : first === 'orch' ? 'Orch 배정 대기' : '배정 없음',
			next: first !== 'wait',
			context: 0,
			tokens: 0,
			load: first === 'task' ? ['todo'] : [],
			loadNote: first === 'task' ? `#${firstTask.num} 대기` : '여유',
			glyph,
			files: $state.snapshot(files),
		});
		adding = false;
		openMember(sn);
	}

	// 멤버 상세 시트 (.pen Teams · 멤버 상세) — 작업 · 캐릭터 · 런타임 · 팀 메뉴. 상세 기록이 없는 멤버는 요약만.
	let viewing = $state<number>();
	let tab = $state('overview');
	const viewed = $derived(team.members.find((m) => m.sn === viewing));
	const det = $derived(viewing === undefined ? undefined : memberDetails[viewing]);
	const origin = $derived(det ? store.templates.find((t) => t.sn === det.template) : store.templates.find((t) => t.name === viewed?.title));
	const memberFiles = $derived(viewed?.files ?? []);
	let memberFile = $state(0);
	let memberMode = $state<'read' | 'edit' | 'diff'>('diff');
	const memberChanged = $derived(origin ? memberFiles.filter((f) => origin.files.find((o) => o.name === f.name)?.body !== f.body) : []);
	/// 템플릿으로 되돌리기 (보고 있는 파일만).
	function revertFile() {
		const f = memberFiles[memberFile];
		const o = origin?.files.find((x) => x.name === f.name);
		if (o) f.body = o.body;
	}
	/// 템플릿에 제안 — 이 멤버의 파일을 템플릿 다음 버전 초안에 넣는다.
	function propose() {
		if (!origin) return;
		const f = memberFiles[memberFile];
		const next = $state.snapshot(origin.draft ?? origin.files).map((x) => (x.name === f.name ? { ...x, body: f.body } : x));
		putDraft(origin, next, viewed?.name ?? '멤버', `${f.name} · ${viewed?.name}의 변경 제안`);
		proposed = f.name;
	}
	let proposed = $state<string>();
	// 일시정지는 화면 상태 (서버 command는 #45).
	let paused = $state<number[]>([]);
	/// 멤버 상세 열기 (멤버 표 · 추가 직후).
	function openMember(sn: number) {
		viewing = sn;
		// 지침 파일이 없는 멤버는 원본 템플릿을 복사해 멤버 것으로 만든다.
		const m = team.members.find((x) => x.sn === sn);
		const o = store.templates.find((t) => t.sn === memberDetails[sn]?.template) ?? store.templates.find((t) => t.name === m?.title);
		if (m && !m.files) m.files = structuredClone($state.snapshot(o?.files ?? []));
		if (m && !m.config && o) m.config = structuredClone($state.snapshot(o.config));
		tab = 'overview';
		taskFilter = 'all';
		taskQuery = '';
		actFilter = 'all';
		runSel = memberDetails[sn]?.failure?.run;
		memberFile = 0;
		memberMode = 'diff';
		proposed = undefined;
	}
	function togglePause(sn: number) {
		paused = paused.includes(sn) ? paused.filter((n) => n !== sn) : [...paused, sn];
	}
	const now = $derived(det?.now ? tasks.find((t) => t.num === det.now!.task) : undefined);
	/// 이슈 경로 (예: #51 › #53).
	const issuePath = (num: number) => {
		const i = issues.find((x) => x.num === num);
		return i?.parent ? `#${i.parent} › #${num}` : `#${num}`;
	};
	const nav = $derived([
		{ group: '작업', items: [
			{ v: 'overview', label: 'Overview', icon: Sparkles },
			{ v: 'tasks', label: `Tasks · ${(det?.now ? 1 : 0) + (det?.queue.length ?? 0)}`, icon: ListChecks },
			{ v: 'runs', label: `Runs · ${det?.runs.length ?? 0}`, icon: CirclePlay },
			{ v: 'activity', label: 'Activity', icon: Activity },
		] },
		{ group: '캐릭터', items: [
			{ v: 'instructions', label: 'Instructions', icon: BookOpen, changed: memberChanged.length > 0 },
			{ v: 'skills', label: 'Skills', icon: Library, changed: !!viewed?.config && !!origin && viewed.config.skills.join() !== origin.config.skills.join() },
		] },
		{ group: '런타임', items: [
			{ v: 'harness', label: 'Harness', icon: SlidersHorizontal, changed: !!viewed?.config && !!origin && (viewed.runtime !== origin.runtime || JSON.stringify(viewed.config.harness) !== JSON.stringify(origin.config.harness)) },
			{ v: 'tools', label: 'Tools & MCP', icon: Wrench, changed: !!viewed?.config && !!origin && (viewed.config.mcp.join() !== origin.config.mcp.join() || viewed.config.github !== origin.config.github) },
			{ v: 'perm', label: '권한', icon: ShieldCheck, changed: !!viewed?.config && !!origin && (viewed.config.trust !== origin.config.trust || viewed.config.include.join() !== origin.config.include.join() || viewed.config.exclude.join() !== origin.config.exclude.join() || JSON.stringify(viewed.config.approvals) !== JSON.stringify(origin.config.approvals)) },
		] },
		{ group: '팀', items: [{ v: 'usage', label: '사용량', icon: Gauge }] },
	]);
	const days = ['월', '화', '수', '목', '금', '토', '일'];

	// Tasks 탭
	let taskFilter = $state<'all' | 'active' | 'done'>('all');
	let taskQuery = $state('');
	const hit = (text: string) => text.toLowerCase().includes(taskQuery.trim().toLowerCase());
	const blocked = $derived(viewed ? tasks.filter((t) => t.agent === viewed.sn && t.status === 'blocked') : []);

	// Runs 탭 — 고른 Run의 단계 · 원인 · 파일을 아래에 보인다.
	let runSel = $state<number>();
	const runIcon = {
		running: { icon: LoaderCircle, text: 'text-status-in-progress' },
		failed: { icon: CircleX, text: 'text-status-blocked' },
		done: { icon: CircleCheck, text: 'text-status-done' },
		cancelled: { icon: CircleSlash, text: 'text-status-cancelled' },
	};

	// Activity 탭
	const kindIcon: Record<string, Component> = { 지시: MessageSquarePlus, 실패: CircleX, 메시지: MessageCircle, 배정: ArrowRightLeft, 반려: Undo2, 리뷰: ShieldCheck, 시스템: Info };
	let actFilter = $state('all');
	const actKind = (a: { type: string; kind: string }) => (a.type === 'decision' ? '결정' : a.kind === '반려' ? '리뷰' : a.kind === '실패' ? '시스템' : a.kind);
	const actKinds = $derived(
		['결정', '배정', '지시', '리뷰', '메시지', '시스템'].map((k) => ({ k, n: det?.activity.flatMap((d) => d.items).filter((a) => actKind(a) === k).length ?? 0 }))
	);
</script>

<svelte:head><title>{team.name} · Teams · OrchStack</title></svelte:head>

{#snippet bars(values: number[])}
	<span class="mini-bars" aria-hidden="true">
		{#each values as h, i (i)}
			<span class={cn('flex-1 rounded-t-xs bg-primary', i < values.length - 1 && 'opacity-35')} style="height: {h}px"></span>
		{/each}
	</span>
{/snippet}

{#snippet kpi(label: string, value: string, sub: string, delta?: string, trend?: number[])}
	<div class="kpi-cell">
		<span class="text-xs text-muted-foreground">{label}</span>
		<span class="flex items-end gap-2">
			<span class="font-mono text-xl font-semibold whitespace-nowrap">{value}</span>
			{#if delta}<span class="kpi-delta">{delta}</span>{/if}
			<span class="flex-1"></span>
			{#if trend}{@render bars(trend)}{/if}
		</span>
		<span class="text-caption text-subtle-foreground">{sub}</span>
	</div>
{/snippet}

<!-- 역할 아바타 + 오른쪽 아래 실행기 로고 -->
{#snippet who(m: TeamMember, small = false)}
	<RoleAvatar role={m.role} icon={glyphOf(m)} size={small ? 'sm' : 'default'} class="overflow-visible">
		<RuntimeLogo runtime={m.runtime} class={cn('absolute -right-1 -bottom-1 ring-[1.5px]', small ? 'size-2.5' : 'size-3')} />
	</RoleAvatar>
{/snippet}

<!-- Team Detail -->
<main class="page-scroll">
		<header class="team-header">
			<nav aria-label="Breadcrumb" class="meta-xs gap-1.5">
				<span>Teams</span>
				<ChevronRight class="size-3" />
				<span class="font-medium text-foreground">{team.name}</span>
			</nav>
			<div class="flex items-center gap-4">
				<div class="flex flex-1 flex-col gap-1.5">
					<div class="flex items-center gap-2.5">
						<h1 class="text-2xl font-bold">{team.name}</h1>
						{#if team.project}<Pill dot="bg-success" class="rounded-sm px-2 py-0.75 text-xs text-foreground">{team.project}</Pill>{/if}
					</div>
					<p class="max-w-160 text-body text-muted-foreground">{team.desc}</p>
				</div>
				<Button variant="outline" size="sm" class="text-body"><Pencil class="size-3.5" />팀 편집</Button>
				<Button size="sm" class="text-body" onclick={() => startAdd()}><UserPlus class="size-3.5" />멤버 추가</Button>
			</div>
			<div class="flex rounded-lg border bg-card">
				{@render kpi('멤버', String(team.members.length), `${running} running · ${count('waiting')} waiting · ${count('idle')} idle`)}
				{@render kpi('열린 태스크', String(team.stats.open), team.stats.openNote)}
				{@render kpi('이번 주 완료', String(team.stats.done), team.stats.doneNote, team.stats.doneDelta, team.stats.doneTrend)}
				{@render kpi('오늘 토큰', k(tokens), `예산 ${teamPolicy.tokenBudget}K · ${Math.round((tokens / teamPolicy.tokenBudget) * 100)}%`, undefined, team.stats.tokenTrend)}
				{@render kpi('평균 사이클', team.stats.cycle, 'Task 생성 → Done', team.stats.cycleDelta, team.stats.cycleTrend)}
			</div>
		</header>

		<!-- 3xl(1760px) 미만에서는 사이드 카드를 멤버 아래로 내린다 (멤버 표 최소 폭 확보). -->
		<div class="team-body">
			<section class="col-fill gap-3" aria-label="멤버">
				<div class="flex items-center gap-2">
					<h2 class="text-sm font-semibold">멤버</h2>
					<span class="flex-1 text-body text-muted-foreground">{team.members.length}</span>
					<Toggle variant="chip" count={team.members.length} pressed={filter === 'all'} onPressedChange={() => (filter = 'all')}>All</Toggle>
					{#each Object.entries(states) as [s, v] (s)}
						<Toggle variant="chip" count={count(s as State)} pressed={filter === s} onPressedChange={() => (filter = s as State)}>{v.label}</Toggle>
					{/each}
				</div>

				{#each alerts as { a, hit } (a.runtime)}
					<Alert.Root variant="destructive" class="flex items-center gap-3">
						<TriangleAlert />
						<div class="row-text">
							<Alert.Title>{a.name} 계정 주간 잔량 {a.week}% — {a.weekReset} 리셋까지 {a.left}</Alert.Title>
							<Alert.Description>{hit.map((m) => m.name).join(' · ')} 영향. {a.forecast}. 정책: 잔량 {teamPolicy.quotaWarn}% 미만 → 확인 요청</Alert.Description>
						</div>
						<Button variant="outline" size="sm" class="h-7.5 text-xs"><Filter class="size-3.25" />P0–P1만 실행</Button>
						<Button variant="outline" size="sm" class="h-7.5 text-xs"><Route class="size-3.25" />폴백 적용 · Anthropic Max</Button>
					</Alert.Root>
				{/each}

				<div class="overflow-hidden rounded-lg border bg-card">
					<Table.Root class="table-fixed">
						<Table.Header class="bg-muted">
							<Table.Row class="hover:bg-muted">
								<Table.Head class="h-11 w-48 pl-4">멤버</Table.Head>
								<Table.Head class="h-11 w-28">상태</Table.Head>
								<Table.Head class="h-11">현재 작업</Table.Head>
								<Table.Head class="h-11 w-56">Runtime</Table.Head>
								<Table.Head class="h-11 w-28">컨텍스트</Table.Head>
								<Table.Head class="h-11 w-32 leading-tight">주간 잔량<br /><span class="text-2xs font-normal text-subtle-foreground">계정 공유</span></Table.Head>
								<Table.Head class="h-11 w-22 pr-4 text-right">오늘 토큰</Table.Head>
							</Table.Row>
						</Table.Header>
						<Table.Body>
							{#each members as m (m.sn)}
								{@const st = states[m.status]}
								{@const week = weekOf(m)}
								{@const full = m.context >= teamPolicy.contextWarn}
								<Table.Row class="h-15 cursor-pointer" onclick={() => openMember(m.sn)}>
									<Table.Cell class="pl-4">
										<span class="flex items-center gap-2.5">
											{@render who(m)}
											<span class="flex min-w-0 flex-col gap-px">
												<button type="button" class="member-name-link" onclick={(e) => { e.stopPropagation(); openMember(m.sn); }}>{m.name}</button>
												<span class="truncate text-caption text-muted-foreground">{m.title}</span>
											</span>
										</span>
									</Table.Cell>
									<Table.Cell>
										<span class={cn('label-xs', st.text)}><st.icon class="size-3.5" />{st.label}</span>
									</Table.Cell>
									<Table.Cell>
										<span class={cn('flex items-center gap-1.5 text-xs', m.status === 'idle' && 'text-muted-foreground')}>
											{#if m.next}<CircleDashed class="size-3.25 shrink-0 text-node-task" />{:else}<SquareCheck class="size-3.25 shrink-0 text-node-task" />{/if}
											<span class="truncate">{m.work}</span>
										</span>
									</Table.Cell>
									<Table.Cell>
										<span class="code-tag font-medium">
											<Terminal class="size-2.25" />{m.model}
										</span>
									</Table.Cell>
									<Table.Cell>
										<span class="flex items-center gap-2">
											<Progress value={m.context} class="h-1.25 bg-muted" indicator={full ? 'bg-status-blocked' : undefined} aria-label="{m.name} 컨텍스트" />
											<span class={cn('font-mono text-caption', full ? 'font-semibold text-status-blocked' : 'text-muted-foreground')}>{m.context}%</span>
										</span>
									</Table.Cell>
									<Table.Cell>
										<span class="flex items-center gap-1.5 text-caption">
											<span class="font-medium text-muted-foreground">주</span>
											<Progress value={week} class="h-1 w-16 bg-muted" indicator={low(week) ? 'bg-destructive' : 'bg-success'} aria-label="{m.name} 주간 잔량" />
											<span class={cn('font-mono font-medium', low(week) && 'font-semibold text-destructive')}>{week}%</span>
										</span>
									</Table.Cell>
									<Table.Cell class="pr-4 text-right font-mono text-xs">{k(m.tokens)}</Table.Cell>
								</Table.Row>
							{:else}
								<Table.Row>
									<Table.Cell colspan={7} class="h-15 text-center text-xs text-muted-foreground">해당 상태의 멤버가 없어요.</Table.Cell>
								</Table.Row>
							{/each}
						</Table.Body>
					</Table.Root>
				</div>

				<Card.Root size="sm">
					<Card.Header><Card.Title>작업량</Card.Title></Card.Header>
					<Card.Content>
						<div class="flex items-center gap-3">
							{#each Object.values(loads) as l (l.label)}
								<span class="meta-line gap-1.25"><span class={cn('size-2 rounded-xs', l.bg)}></span>{l.label}</span>
							{/each}
						</div>
						{#each team.members as m (m.sn)}
							<div class="flex items-center gap-3">
								<span class="flex w-24 items-center gap-2">
									{@render who(m, true)}
									<span class="text-xs font-medium">{m.name}</span>
								</span>
								<span class="workload-bar" aria-hidden="true">
									{#each m.load as s, i (i)}<span class={cn('w-9.5 rounded-xs', loads[s].bg)}></span>{/each}
								</span>
								<span class="workload-count">작업 {m.load.length}개</span>
								<span
									class={cn(
										'flex-1 truncate text-xs font-medium',
										m.load.includes('blocked') ? 'text-status-blocked' : m.status === 'waiting' ? 'text-status-waiting' : 'text-muted-foreground'
									)}>{m.loadNote}</span
								>
							</div>
						{/each}
					</Card.Content>
				</Card.Root>
			</section>

			<aside class="team-aside">
				<Card.Root size="sm">
					<Card.Header>
						<Card.Title>모델 연결 사용량</Card.Title>
						<Card.Action><Button variant="link" size="xs" href="/settings">모델 연결</Button></Card.Action>
					</Card.Header>
					<Card.Content>
						{#each accounts as a (a.runtime)}
							{@const users = team.members.filter((m) => m.runtime === a.runtime)}
							{@const warn = low(a.week)}
							<div class={cn('box-col gap-2.5 rounded-md p-3', warn && 'border-destructive bg-destructive-soft')}>
								<div class="flex items-center gap-2">
									<RuntimeLogo runtime={a.runtime} class="size-5 ring-0" />
									<span class="flex min-w-0 flex-1 flex-col">
										<span class="text-body font-semibold">{a.name}</span>
										<span class="font-mono text-caption text-muted-foreground">{a.login}</span>
									</span>
									<Avatar.Group class={warn ? '*:data-[slot=avatar]:ring-destructive-soft' : undefined}>
										{#each users as m (m.sn)}<RoleAvatar role={m.role} icon={glyphOf(m)} size="sm" />{/each}
									</Avatar.Group>
								</div>
								{#each [{ label: '5H', pct: a.h5, reset: a.h5Reset }, { label: '주간', pct: a.week, reset: a.weekReset }] as q (q.label)}
									<div class="flex flex-col gap-1">
										<span class="flex gap-1.5 text-caption">
											<span class="flex-1 text-muted-foreground">{q.label}</span>
											<span class={cn('font-semibold', low(q.pct) && 'text-destructive')}><span class="font-mono">{q.pct}%</span> 남음</span>
											<span class="text-subtle-foreground">· {q.reset} 리셋</span>
										</span>
										<Progress value={q.pct} class="h-1.25 bg-muted" indicator={low(q.pct) ? 'bg-destructive' : 'bg-success'} aria-label="{a.name} {q.label} 잔량" />
									</div>
								{/each}
							</div>
						{/each}
						<div class="level-list">
							{#each meters as mt (mt.name)}
								<LimitRow icon={mt.kind === 'key' ? KeyRound : Route} label={mt.name} used="${mt.used}" max="/ ${mt.limit}" value={(mt.used / mt.limit) * 100} note={mt.note} />
							{/each}
						</div>
					</Card.Content>
				</Card.Root>

				<Card.Root size="sm">
					<Card.Header>
						<Card.Title>Orch 진행</Card.Title>
						<Card.Action><Button variant="link" size="xs" onclick={editPolicy}>정책 편집</Button></Card.Action>
					</Card.Header>
					<Card.Content>
						{@const ModeIcon = policy.mode === 'manual' ? Hand : policy.mode === 'full' ? Zap : Timer}
						<div class="policy-note">
							<ModeIcon class="size-3.5 text-primary" />
							<span class="flex-1 text-xs font-semibold">{modeLabel(policy)}</span>
							{#if policy.guards.find((g) => g.key === 'streak')?.on}<span class="text-caption text-muted-foreground">연속 {policy.streak}/{streakLimit(policy)}</span>{/if}
						</div>
						<div class="flex flex-col">
							{#each policy.levels as l, i (i)}
								<div class="value-line-wide">
									<span class={cn('mono-tag py-px text-on-solid', levelTone[i].bg)}>L{i}</span>
									<span class="flex-1">{l.name}</span>
									<span class={cn('font-medium', levelTone[i].text)}>{actionLabel(policy, l.action, l.timeout)}</span>
								</div>
							{/each}
						</div>
						<div class="meta-xs gap-1.5 border-t pt-2">
							<ShieldCheck class="size-3.25 shrink-0 text-status-done" />루프 가드 {policy.guards.filter((g) => g.on).length}개 켜짐 · 마지막 정지: {policy.lastStop}
						</div>
						<div class="meta-xs gap-1.5">
							<Layers class="size-3.25 shrink-0" />하위 작업 · 기본 {policy.spawn.mode} · 허용 {policy.spawn.allow.join(' · ')} · 리드당 {policy.spawn.maxChild}
						</div>
					</Card.Content>
				</Card.Root>

				<Card.Root size="sm">
					<Card.Header>
						<Card.Title>팀 정책</Card.Title>
						<Card.Action><Button variant="link" size="xs">편집</Button></Card.Action>
					</Card.Header>
					<Card.Content class="gap-1">
						<div class="value-line">
							<Terminal class="size-3.25 text-muted-foreground" />
							<span class="flex-1 text-muted-foreground">기본 Runtime</span>
							<span class="font-medium">{teamPolicy.runtime}</span>
						</div>
						<LimitRow icon={Coins} label="오늘 토큰 예산" used={k(tokens)} max="/ {teamPolicy.tokenBudget}K" value={(tokens / teamPolicy.tokenBudget) * 100} />
						<LimitRow
							icon={Layers}
							label="Context 경고 · {teamPolicy.contextWarn}%"
							used="{hottest.context}%"
							max="/ 100%"
							value={hottest.context}
							note={hot ? `${hottest.name} ${hottest.context}% 초과 · 나머지 ${cool}명 50% 미만` : undefined}
							warn={hot}
							mark={teamPolicy.contextWarn}
						/>
						<LimitRow
							icon={Play}
							label="동시 실행"
							used={String(running)}
							max="/ {teamPolicy.maxRuns} Run"
							value={(running / teamPolicy.maxRuns) * 100}
							note={running < teamPolicy.maxRuns ? `${teamPolicy.maxRuns - running}개 여유` : '가득 참'}
						/>
						<div class="value-line">
							<ShieldCheck class="size-3.25 text-muted-foreground" />
							<span class="flex-1 text-muted-foreground">Review 필수</span>
							<Pill class="bg-review-soft text-status-review"><GitMerge />{teamPolicy.review}</Pill>
						</div>
						<div class="value-line">
							<GitBranch class="size-3.25 text-muted-foreground" />
							<span class="flex-1 text-muted-foreground">Repo 권한</span>
							<span class="font-mono font-medium">{teamPolicy.repo}</span>
						</div>
					</Card.Content>
				</Card.Root>
			</aside>
		</div>
</main>


<!-- .pen FormRow 제목 -->
{#snippet heading(title: string, desc: string)}
	<div class="flex flex-col gap-0.5">
		<h3 class="text-sm font-semibold">{title}</h3>
		<p class="text-xs text-muted-foreground">{desc}</p>
	</div>
{/snippet}

<Sheet.Root bind:open={adding}>
	<Sheet.Content side="right" size="xl">
		<Sheet.Header>
			<Sheet.Title>{team.name}에 멤버 추가</Sheet.Title>
			{#snippet sub()}<Steps steps={['템플릿 선택', '캐릭터', '런타임 · 도구']} current={step} />{/snippet}
		</Sheet.Header>

		<Sheet.Body class="flex-row gap-6">
			<div class="col-fill gap-4.5">
				{#if step === 0}
					<InputGroup.Root class="h-9">
						<InputGroup.Addon><Search /></InputGroup.Addon>
						<InputGroup.Input bind:value={tq} placeholder="템플릿 검색 · 역할, 스킬" aria-label="템플릿 검색" />
					</InputGroup.Root>
					{@const rec = store.templates.find((t) => t.sn === recommend.template)!}
					<button
						type="button"
						onclick={() => (pick = rec.sn)}
						class="suggest-banner"
					>
						<Sparkles class="size-3.5 shrink-0 text-primary" />
						<span><span class="font-semibold">Orch 추천</span> · {recommend.reason} → {rec.name}</span>
					</button>
					<ChoiceCards.Root aria-label="템플릿" class="grid-cols-2" bind:value={() => pick, (v) => (pick = v as typeof pick)}>
						{#each tplShown as t (t.sn)}
							{@const on = pick === t.sn}
							<ChoiceCards.Item value={t.sn} layout="row" ondblclick={toCharacter} class="gap-2.5 bg-card px-2 py-1.75">
								<RoleAvatar role={t.role} />
								<span class="col-fill gap-0.75">
									<span class="text-xs font-semibold">{t.name}</span>
									<span class="text-caption text-muted-foreground">v{t.version} · {t.focus}</span>
									<span class="subtle-meta gap-1.25">
										<RuntimeLogo runtime={t.runtime} class="size-3 ring-0" />
										{runtimeName(t.runtime)} · {t.model} · {t.members ? `멤버 ${t.members}` : '미사용'}
									</span>
								</span>
								{#if on}<CircleCheck class="size-4 shrink-0 text-primary" />{/if}
							</ChoiceCards.Item>
						{:else}
							<p class="grid-empty">검색 결과가 없어요.</p>
						{/each}
					</ChoiceCards.Root>
					<!-- 같은 선택(pick)의 한 칸 — 배치는 그대로 두려고 contents -->
					<ChoiceCards.Root aria-label="빈 캐릭터" class="contents" bind:value={() => pick, (v) => (pick = v as typeof pick)}>
					<ChoiceCards.Item value={null} layout="row" class="gap-2.5 p-3.5">
						<span class="add-tile"><Plus class="size-3.5" /></span>
						<span class="flex flex-1 flex-col gap-0.5">
							<span class="text-xs font-semibold">빈 캐릭터로 시작</span>
							<span class="text-caption text-muted-foreground">템플릿 없이 Instructions를 직접 작성</span>
						</span>
						{#if pick === null}<CircleCheck class="size-4 text-primary" />{/if}
					</ChoiceCards.Item>
					</ChoiceCards.Root>
				{:else}
					<!-- 고른 템플릿 (2 · 3단계 공통) -->
					<div class="soft-box items-center gap-3 p-3">
						<RoleAvatar {role} />
						<span class="row-text">
							<span class="text-xs font-semibold">{tpl ? `템플릿 · ${tpl.name} v${tpl.version}` : '빈 캐릭터'}</span>
							<span class="truncate text-caption text-muted-foreground">
								{tpl ? `${tpl.files.map((f) => f.name).join(' · ')} · 스킬 ${tpl.config.skills.length + tpl.config.mcp.length} · ${runtimeName(tpl.runtime)} · ${tpl.model} 를 복사해요` : 'Instructions를 직접 작성해요'}
							</span>
						</span>
						<Button variant="link" size="xs" onclick={() => (step = 0)}>변경</Button>
					</div>
				{/if}

				{#if step === 1}
					<section class="flex flex-col gap-2.5">
						{@render heading('캐릭터', '팀에서 일할 실제 작업자예요. 여기서 고친 내용은 템플릿과 별개로 이 멤버에게만 적용돼요.')}
						<label class="flex items-center gap-3">
							<span class="w-14 text-xs text-muted-foreground">이름</span>
							<Input bind:value={name} placeholder="예: 준" class="h-8.5" />
						</label>
						<div class="flex items-center gap-3">
							<span class="w-14 text-xs text-muted-foreground">아바타</span>
							<div role="radiogroup" aria-label="아바타" class="flex gap-2">
								{#each glyphs[role] as g, i (i)}
									<button
										type="button"
										role="radio"
										aria-checked={glyph === i}
										aria-label="아이콘 {i + 1}"
										onclick={() => (glyph = i)}
										class={cn('rounded-md outline-none focus-visible:ring-3 focus-visible:ring-ring/50', glyph === i ? 'ring-2 ring-primary ring-offset-2 ring-offset-background' : 'opacity-50 hover:opacity-100')}
									>
										<RoleAvatar {role} icon={g} />
									</button>
								{/each}
							</div>
							<span class="text-caption text-subtle-foreground">역할 색은 유지, 아이콘만 선택</span>
						</div>
					</section>
					<section class="flex flex-col gap-2">
						<div class="flex items-baseline gap-2">
							<h3 class="text-sm font-semibold">Instructions</h3>
							<span class="text-caption text-muted-foreground">{tpl ? '템플릿에서 복사됨' : '직접 작성'} · 추가 시 저장</span>
						</div>
						<MdEditor bind:files bind:active={fileIdx} base={tpl ? liveMap(tpl.files) : undefined} baseLabel="템플릿" currentLabel={name.trim() || '새 멤버'} class="h-140">
							{#snippet status()}초안 · 추가 시 저장{/snippet}
						</MdEditor>
					</section>
				{:else if step === 2}
					<section class="flex flex-col gap-3">
						{@render heading('런타임', fellBack ? '템플릿과 다름 · 실행기 · 연결을 이 멤버에게만 바꿨어요' : '템플릿 기본값 · 이 멤버에게만 바꿀 수 있어요')}
						<div class="grid grid-cols-2 gap-2.5">
							<label class="flex flex-col gap-1.5">
								<span class="text-caption font-semibold text-muted-foreground">실행기</span>
								<Select.Root type="single" bind:value={runtime} onValueChange={(v) => (model = models[v as Runtime][0])}>
									<Select.Trigger class="w-full">
										<span class="flex items-center gap-2"><RuntimeLogo runtime={runtime} class="size-4 ring-0" />{runtimeName(runtime)}</span>
									</Select.Trigger>
									<Select.Content>
										{#each ['claude', 'codex'] as const as r (r)}<Select.Item value={r} label={runtimeName(r)} />{/each}
									</Select.Content>
								</Select.Root>
							</label>
							<div class="flex flex-col gap-1.5">
								<span class="text-caption font-semibold text-muted-foreground">연결</span>
								<!-- 연결은 실행기에 딸려 정해진다 (연결 여러 개는 Settings · #31) -->
								<div class="field-box h-9">
									<RuntimeLogo runtime={runtime} class="size-4 ring-0" />
									{accountOf(runtime).plan}
									<Pill class="text-2xs">구독</Pill>
									<span class={cn('ml-auto text-caption font-normal', low(accountOf(runtime).week) ? 'text-destructive' : 'text-muted-foreground')}>주간 {accountOf(runtime).week}% 남음</span>
								</div>
							</div>
							<label class="flex flex-col gap-1.5">
								<span class="text-caption font-semibold text-muted-foreground">모델</span>
								<Select.Root type="single" bind:value={model}>
									<Select.Trigger class="w-full"><span class="flex items-center gap-2"><Cpu class="size-3.5 text-muted-foreground" />{model}</span></Select.Trigger>
									<Select.Content>
										{#each models[runtime] as m (m)}<Select.Item value={m} label={m} />{/each}
									</Select.Content>
								</Select.Root>
							</label>
							<label class="flex flex-col gap-1.5">
								<span class="text-caption font-semibold text-muted-foreground">Effort</span>
								<Select.Root type="single" bind:value={effort}>
									<Select.Trigger class="w-full"><span class="flex items-center gap-2"><Gauge class="size-3.5 text-muted-foreground" />{effort}</span></Select.Trigger>
									<Select.Content>
										{#each ['Auto', 'Low', 'Medium', 'High'] as e (e)}<Select.Item value={e} label={e} />{/each}
									</Select.Content>
								</Select.Root>
							</label>
						</div>
						{#if fellBack && tpl}
							<Alert.Root variant="warning" class="flex items-center gap-2.5">
								<Route />
								<div class="flex-1">
									<Alert.Title>폴백 2순위로 시작해요</Alert.Title>
									<Alert.Description>템플릿 기본 {accountOf(tpl.runtime).plan}가 주간 {accountOf(tpl.runtime).week}% 남음 → {accountOf(runtime).plan} ({runtimeName(runtime)})</Alert.Description>
								</div>
							</Alert.Root>
						{/if}
						<div class="tag-row">
							<span class="font-semibold text-muted-foreground">폴백</span>
							{#each chain as a, i (a.runtime)}
								<span class="pill-soft gap-1.25">
									<span class="font-mono text-2xs text-muted-foreground">{i + 1}</span>
									{a.plan}
									<span class={cn('font-mono text-2xs', low(a.week) ? 'text-status-blocked' : 'text-muted-foreground')}>{a.week}%</span>
								</span>
								<ChevronRight class="size-3 text-subtle-foreground" />
							{/each}
							<span class="pill-soft gap-1.25">
								<span class="font-mono text-2xs text-muted-foreground">3</span>
								OmniRoute
								<span class="font-mono text-2xs text-muted-foreground">${Math.round(gateway.used)}/${gateway.limit}</span>
							</span>
						</div>
					</section>
					{#if tpl}
						<section class="flex flex-col gap-2.5">
							{@render heading('스킬 · 도구', '템플릿에서 복사 · 켜고 끄기는 추가 후 멤버 상세 › Skills · Tools & MCP에서 해요')}
							<div class="soft-box flex-col gap-2.5 p-3.5">
								<div class="flex text-xs">
									<span class="flex-1 font-medium">템플릿에서 복사 · {tpl.config.skills.length + tpl.config.mcp.length}개 활성</span>
									<span class="font-mono text-caption text-muted-foreground">+{((tpl.config.skills.length + tpl.config.mcp.length) * 0.24).toFixed(1)}K tok / Run</span>
								</div>
								<div class="flex flex-wrap gap-1.5">
									{#each tpl.config.skills as sk (sk)}<Pill class="bg-card"><Sparkles />{sk}</Pill>{/each}
									{#each tpl.config.mcp as mc (mc)}<Pill class="bg-card"><Plug />{mc}</Pill>{/each}
								</div>
							</div>
							<div class="info-line-wrap">
								<span class="flex items-center gap-1.25"><ShieldCheck class="size-3" />Trust {tpl.config.trust}</span>
								<span class="flex items-center gap-1.25"><FolderGit2 class="size-3" />{scopeText(tpl.config)}</span>
								<span class="flex items-center gap-1.25"><GitPullRequest class="size-3" />PR 승인 필요</span>
							</div>
						</section>
					{/if}
					<section class="flex flex-col gap-2.5">
						{@render heading('첫 작업', '추가 직후 무엇을 할지')}
						<ChoiceCards.Root aria-label="첫 작업" class="grid-cols-3" bind:value={() => first, (v) => (first = v as typeof first)}>
							{#each [{ v: 'orch', icon: Sparkles, t: 'Orch에게 맡기기', d: `팀 진행 정책(${modeLabel(policy)})에 따라 대기열에서 배정` }, { v: 'task', icon: ListChecks, t: '지금 태스크 지정', d: firstLabel.task }, { v: 'wait', icon: Pause, t: '대기', d: '추가만 하고 배정하지 않음' }] as const as o (o.v)}
								{@const on = first === o.v}
								<ChoiceCards.Item value={o.v} class="rounded-md">
									<span class="label-xs-strong">
										<o.icon class={cn('size-3.5', on ? 'text-primary' : 'text-muted-foreground')} />
										<span class="flex-1">{o.t}</span>
										{#if on}<CircleCheck class="size-3.5 text-primary" />{:else}<Circle class="size-3.5 text-subtle-foreground" />{/if}
									</span>
									<span class="truncate text-caption text-muted-foreground">{o.d}</span>
								</ChoiceCards.Item>
							{/each}
						</ChoiceCards.Root>
					</section>
				{/if}
			</div>

			<!-- 미리보기 -->
			<aside class="aside-col w-80 gap-3.5" aria-label="미리보기">
				{#if step === 0}
					{#if tpl}
						{@const soul = tpl.files.find((f) => f.name === 'SOUL.md')}
						<div class="card-col gap-2.5 p-4">
							<div class="flex flex-col gap-0.5">
								<span class="text-sm font-semibold">{tpl.name} v{tpl.version}</span>
								<span class="text-xs text-muted-foreground">{tpl.desc}</span>
							</div>
							<div>
								<KeyValueRow label="복사되는 것" value={`Instructions · 파일 ${tpl.files.length}`} />
								<KeyValueRow label="스킬" value={`${tpl.config.skills.length} · MCP ${tpl.config.mcp.length}`} />
								<KeyValueRow label="기본 런타임" value={`${runtimeName(tpl.runtime)} · ${tpl.model}`} />
								<KeyValueRow label="권한" value={`Trust ${tpl.config.trust} · ${scopeText(tpl.config)}`} />
								<KeyValueRow label="최근 성과" value={tpl.success ? `멤버 ${tpl.members}명 · Run 성공률 ${tpl.success}%` : '기록 없음'} />
							</div>
							{#if soul}<pre class="pre-preview">{soul.body}</pre>{/if}
						</div>
					{:else}
						<div class="empty-note text-muted-foreground">빈 캐릭터는 AGENT.md 한 파일로 시작해요. 런타임은 다음 단계에서 정해요.</div>
					{/if}
				{:else}
					<div class="card-col gap-2.5 p-4">
						<span class="text-caption font-semibold text-muted-foreground">미리보기</span>
						<div class="flex items-center gap-3 pb-1">
							<RoleAvatar {role} icon={glyphs[role][glyph]} size="lg" />
							<span class="flex flex-col gap-0.5">
								<span class="text-sm font-semibold">{name.trim() || '이름 없음'}</span>
								<span class="text-caption text-muted-foreground">{tpl?.name ?? 'Agent'} · {team.name}</span>
							</span>
						</div>
						<div>
							<KeyValueRow label="원본" value={tpl ? `${tpl.name} v${tpl.version}` : '빈 캐릭터'} />
							<KeyValueRow label="템플릿 대비" value={diff} />
							<KeyValueRow label="런타임" value={tpl && runtime === tpl.runtime && model === tpl.model ? '템플릿과 같음' : `${runtimeName(runtime)} · ${accountOf(runtime).plan.split(' · ')[1]}`} />
							<KeyValueRow label="컨텍스트" value={`약 ${(tokAll / 1000).toFixed(1)}K tok / Run`} />
							<KeyValueRow label="첫 작업" value={first === 'task' ? `#${firstTask.num} 지정` : firstLabel[first]} />
						</div>
					</div>
					<div class="tip-box">
						<Info class="mt-0.5 size-3.5 shrink-0 text-primary" />
						<span>‘팀에 추가’를 누르면 팀 멤버 목록에 들어가요. 어떤 작업을 맡을지는 Orch가 팀 진행 정책에 따라 배정해요.</span>
					</div>
				{/if}
			</aside>
		</Sheet.Body>

		<Sheet.Footer
			note={step === 0 ? '다음 단계에서 이름 · 성격 · 런타임을 다듬어요' : step === 1 ? '이름 · 성격을 정해요 · 이후 변경은 이 멤버에게만' : '추가 후 멤버 상세에서 계속 편집할 수 있어요'}
			noteIcon={step === 0 ? Info : Copy}
		>
			{#if step === 0}
				<Button variant="ghost" size="sm" onclick={() => (adding = false)}>취소</Button>
				<Button size="sm" onclick={toCharacter}>다음 · 캐릭터<ArrowRight /></Button>
			{:else}
				<Button variant="ghost" size="sm" onclick={() => step--}><ArrowLeft />이전</Button>
				{#if step === 1}
					<Button size="sm" disabled={!name.trim()} onclick={() => (step = 2)}>다음 · 런타임 · 도구<ArrowRight /></Button>
				{:else}
					<Button size="sm" disabled={!name.trim()} onclick={add}><UserPlus />팀에 추가</Button>
				{/if}
			{/if}
		</Sheet.Footer>
	</Sheet.Content>
</Sheet.Root>

<!-- .pen BarChart — 요일별 막대. b가 있으면 위에 실패색으로 쌓는다 -->
{#snippet chart(cols: [number, number?][], legend: [string, string?], total: string)}
	{@const max = Math.max(1, ...cols.map(([a, b]) => a + (b ?? 0)))}
	<div class="flex flex-col gap-2.5">
		<div class="meta-line gap-3.5">
			<span class="flex items-center gap-1.25"><span class="size-2 rounded-xs bg-primary"></span>{legend[0]}</span>
			{#if legend[1]}<span class="flex items-center gap-1.25"><span class="size-2 rounded-xs bg-status-blocked"></span>{legend[1]}</span>{/if}
			<span class="flex-1"></span>
			<span>{total}</span>
		</div>
		<div class="flex h-25 gap-2 border-b" role="img" aria-label="{legend[0]} {total}">
			{#each cols as [a, b], i (i)}
				<div class="bar-col">
					{#if b}<span class="w-4 rounded-t-sm bg-status-blocked" style="height: {(b / max) * 88}px"></span>{/if}
					{#if a}<span class={cn('w-4 bg-primary', !b && 'rounded-t-sm')} style="height: {(a / max) * 88}px"></span>{/if}
				</div>
			{/each}
		</div>
		<div class="bar-axis">
			{#each days as d (d)}<span class="flex-1">{d}</span>{/each}
		</div>
	</div>
{/snippet}

{#snippet mini(label: string, value: string, sub: string, tone?: string)}
	<div class="stat-card">
		<span class="text-xs text-muted-foreground">{label}</span>
		<span class="font-mono text-xl font-semibold">{value}</span>
		<span class={cn('text-caption text-subtle-foreground', tone)}>{sub}</span>
	</div>
{/snippet}

<!-- .pen TaskListRow -->
{#snippet taskRow(num: number, title: string, note: string | undefined, issue: string, meta: string, priority: string, order?: number, state: 'todo' | 'wait' | 'done' = 'todo', noteTone?: string)}
	<div class="queue-row">
		{#if order}<span class="queue-num">{order}</span>{/if}
		{#if state === 'done'}<CircleCheck class="size-3.5 text-status-done" />{:else if state === 'wait'}<Circle class="size-3.5 text-status-todo" />{:else}<CircleDashed class="size-3.5 text-status-todo" />{/if}
		<span class="font-mono text-xs text-muted-foreground">#{num}</span>
		<span class="row-text">
			<span class="truncate text-body font-medium">{title}</span>
			{#if note}<span class={cn('truncate text-caption', noteTone ?? 'text-muted-foreground')}>{note}</span>{/if}
		</span>
		<span class="font-mono text-caption text-subtle-foreground">{issue}</span>
		<span class="num-cell w-28 whitespace-nowrap">{meta}</span>
		<span class="w-5 text-caption font-semibold text-muted-foreground">{priority}</span>
	</div>
{/snippet}

<Sheet.Root bind:open={() => viewing !== undefined, (v) => !v && (viewing = undefined)}>
	<Sheet.Content side="right" size="xl">
		{#if viewed}
			{@const m = viewed}
			{@const st = states[m.status]}
			{@const isPaused = paused.includes(m.sn)}
			<Sheet.Header>
				{#snippet lead()}<RoleAvatar role={m.role} icon={glyphOf(m)} size="lg" />{/snippet}
					<div class="flex items-center gap-2.5">
						<Sheet.Title class="text-xl font-bold">{m.name}</Sheet.Title>
						{#if isPaused}
							<Pill class="rounded-sm bg-warning-soft px-2 text-warning"><Pause />Paused</Pill>
						{:else}
							<Pill class={cn('rounded-sm px-2', m.status === 'running' ? 'bg-primary-soft text-status-in-progress' : m.status === 'waiting' ? 'bg-warning-soft text-status-waiting' : '')}>
								<st.icon />{st.label}{#if m.status === 'running' && det?.now}{` · #${det.now.task}`}{/if}
							</Pill>
						{/if}
					</div>
					<div class="meta-xs gap-2">
						<span>{m.title} · {team.name}</span>
						{#if origin}
							<span class="text-subtle-foreground">·</span>
							<span class="chip-box gap-1.25 py-0.5">
								<LayoutTemplate class="size-3" />원본 {origin.name} v{origin.version}{#if det?.diff.length}{` · 변경 ${det.diff.length}`}{/if}
							</span>
						{/if}
					</div>
				{#snippet actions()}
					<Button variant="outline" size="sm" onclick={() => togglePause(m.sn)}>
						{#if isPaused}<Play />Resume{:else}<Pause />Pause{/if}
					</Button>
				{/snippet}
			</Sheet.Header>

			<Sheet.Body padded={false} class="flex-row">
				<nav aria-label="멤버 메뉴" class="side-nav w-52.5 px-2.5 py-3.5">
					{#each nav as g, gi (g.group)}
						<span class={cn('list-label px-2 pt-0.5 pb-1.5', gi > 0 && 'mt-1.5 border-t pt-3.5')}>{g.group}</span>
						{#each g.items as it (it.v)}
							<button
								type="button"
								aria-current={tab === it.v ? 'page' : undefined}
								onclick={() => (tab = it.v)}
								class={cn(
									'side-nav-item h-8.5 text-muted-foreground',
									tab === it.v && 'bg-accent font-semibold text-foreground'
								)}
							>
								<it.icon class="size-3.75" />
								<span class="flex-1 text-left">{it.label}</span>
								{#if 'changed' in it && it.changed}<span class="size-1.5 rounded-full bg-primary" aria-label="템플릿과 다름"></span>{/if}
							</button>
						{/each}
					{/each}
					<span class="subtle-meta gap-1.5 px-2 pt-3.5"><span class="size-1.5 rounded-full bg-primary"></span>템플릿과 다른 항목</span>
				</nav>

				<div class="member-main">
					{#if !det && ['overview', 'tasks', 'runs', 'activity', 'usage'].includes(tab)}
						<Empty.Root class="bg-card">
							<Empty.Header>
								<Empty.Media variant="icon"><Activity /></Empty.Media>
								<Empty.Title>아직 기록이 없어요</Empty.Title>
								<Empty.Description>{m.name}의 태스크 · Run · 활동이 쌓이면 여기에 보여요. 지금: {m.work}</Empty.Description>
							</Empty.Header>
						</Empty.Root>
					{:else if det && tab === 'overview'}
						{@const ok = det.runs.filter((r) => r.status === 'done').length}
						{@const ended = det.runs.filter((r) => r.status === 'done' || r.status === 'failed').length}
						{@const acc = accounts.find((a) => a.runtime === m.runtime)!}
						<div class="flex gap-3">
							{@render mini('이번 주 완료', String(det.week.done), det.week.doneDelta, 'text-status-done')}
							{@render mini('Run 성공률', `${Math.round((ok / Math.max(1, ended)) * 100)}%`, `${ok}/${ended} · 진행 ${det.runs.filter((r) => r.status === 'running').length}`)}
							{@render mini('평균 Cycle', det.week.cycle, `팀 평균 ${team.stats.cycle}`)}
							{@render mini('이번 주 토큰', `${det.week.tokens}K`, `${acc.name} 계정 주간의 62%`)}
						</div>
						<Card.Root size="sm">
							<Card.Header>
								<Card.Title>이번 주 토큰 사용</Card.Title>
								<Card.Action><Button variant="link" size="xs" onclick={() => (tab = 'runs')}>Runs 보기</Button></Card.Action>
							</Card.Header>
							<Card.Content>{@render chart(det.week.days.map((d) => [d]), ['토큰 (K)'], `이번 주 · ${det.week.tokens}K · 하루 평균 ${Math.round(det.week.tokens / 7)}K`)}</Card.Content>
						</Card.Root>
						{#if det.now && now}
							<Card.Root size="sm">
								<Card.Header><Card.Title>지금 하는 일</Card.Title></Card.Header>
								<Card.Content class="gap-2.5">
									<div class="flex items-center gap-2 text-body">
										<SquareCheck class="size-3.5 text-node-task" />
										<span class="flex-1 font-medium">#{now.num} · {now.title}</span>
										<span class="font-mono text-xs font-semibold">{det.now.pct}%</span>
									</div>
									<Progress value={det.now.pct} class="h-1.5" aria-label="#{now.num} 진행" />
									<div class="meta-wrap">
										<span class="flex items-center gap-1.25"><LoaderCircle class="size-3" />Run #{det.now.run} · {det.now.elapsed}</span>
										<span class="flex items-center gap-1.25"><GitBranch class="size-3" />{det.now.branch} · {det.now.commits} commits</span>
										<span class="flex items-center gap-1.25"><Timer class="size-3" />ETA {det.now.eta}</span>
									</div>
								</Card.Content>
							</Card.Root>
						{/if}
						<Card.Root size="sm">
							<Card.Header><Card.Title>주의 필요 · {det.attention.length}</Card.Title></Card.Header>
							<Card.Content class="gap-0">
								{#each det.attention as at, i (i)}
									{@const Icon = at.kind === 'quota' ? TriangleAlert : at.kind === 'wait' ? Link2 : Undo2}
									<div class={cn('flex items-center gap-3 py-2.5', i > 0 && 'border-t')}>
										<Icon class={cn('size-4 shrink-0', at.kind === 'quota' ? 'text-destructive' : at.kind === 'wait' ? 'text-status-waiting' : 'text-status-review')} />
										<span class="row-text">
											<span class="text-body font-medium">{at.title}</span>
											<span class="text-xs text-muted-foreground">{at.sub}</span>
										</span>
									</div>
								{/each}
							</Card.Content>
						</Card.Root>
						<Card.Root size="sm">
							<Card.Header>
								<Card.Title>다음 실행 대기열</Card.Title>
								<Card.Action><Button variant="link" size="xs" onclick={() => (tab = 'tasks')}>Tasks 탭</Button></Card.Action>
							</Card.Header>
							<Card.Content class="gap-0">
								{#each det.queue as q, i (q.num)}
									{@render taskRow(q.num, q.title, q.note, q.issue, q.est, q.priority, i + 1, q.blocked ? 'wait' : 'todo', q.blocked ? 'text-status-waiting' : undefined)}
								{/each}
							</Card.Content>
						</Card.Root>
						<Card.Root size="sm">
							<Card.Header><Card.Title>템플릿 대비 변경 · {det.diff.length}</Card.Title></Card.Header>
							<Card.Content class="gap-2">
								{#each det.diff as d (d.file)}
									<div class="flex gap-2.5 text-xs">
										{#if d.file === 'SOUL.md'}<BookOpen class="mt-0.5 size-3.5 text-muted-foreground" />{:else}<FileText class="mt-0.5 size-3.5 text-muted-foreground" />{/if}
										<span class="flex flex-col gap-0.5"><span class="font-mono font-medium">{d.file}</span><span class="text-muted-foreground">{d.note}</span></span>
									</div>
								{/each}
								<span class="meta-line gap-1.5 border-t pt-2"><GitCompare class="size-3" />템플릿이 새 버전으로 바뀌면 여기서 골라서 가져올 수 있어요 (자동 반영 없음)</span>
							</Card.Content>
						</Card.Root>
					{:else if det && tab === 'tasks'}
						{@const active = (det.now ? 1 : 0) + det.queue.length}
						<div class="flex items-center gap-2">
							{#each [['all', '전체', active + det.done.length], ['active', '활성', active], ['done', '완료', det.done.length]] as const as [v, l, n] (v)}
								<Toggle variant="chip" count={n} pressed={taskFilter === v} onPressedChange={() => (taskFilter = v)}>{l}</Toggle>
							{/each}
							<span class="flex-1"></span>
							<InputGroup.Root class="h-8 w-56 bg-card">
								<InputGroup.Addon><Search /></InputGroup.Addon>
								<InputGroup.Input bind:value={taskQuery} placeholder="태스크 검색" aria-label="태스크 검색" />
							</InputGroup.Root>
						</div>
						{#if taskFilter !== 'done'}
							{#if det.now && now && hit(now.title)}
								<Card.Root size="sm">
									<Card.Header><Card.Title>진행 중 · 1</Card.Title></Card.Header>
									<Card.Content class="gap-0">
										<div class="queue-row">
											<CircleDot class="size-3.5 text-status-in-progress" />
											<span class="font-mono text-xs text-muted-foreground">#{now.num}</span>
											<span class="row-text">
												<span class="text-body font-medium">{now.title}</span>
												<span class="text-caption text-status-in-progress">Run #{det.now.run} · {det.now.pct}%</span>
											</span>
											<span class="font-mono text-caption text-subtle-foreground">{issuePath(now.issue)}</span>
											<span class="num-cell w-28 whitespace-nowrap">{det.now.eta} 남음</span>
											<span class="w-5 text-caption font-semibold text-muted-foreground">{now.priority}</span>
										</div>
									</Card.Content>
								</Card.Root>
							{/if}
							<Card.Root size="sm">
								<Card.Header>
									<Card.Title>실행 대기열 · {det.queue.length}</Card.Title>
									<Card.Description>위에서부터 순서대로 실행돼요.</Card.Description>
								</Card.Header>
								<Card.Content class="gap-0">
									{#each det.queue.filter((q) => hit(q.title)) as q, i (q.num)}
										{@render taskRow(q.num, q.title, q.note, q.issue, q.est, q.priority, i + 1, q.blocked ? 'wait' : 'todo', q.blocked ? 'text-status-waiting' : undefined)}
									{:else}
										<p class="border-t py-3 text-xs text-muted-foreground">대기 중인 태스크가 없어요.</p>
									{/each}
								</Card.Content>
							</Card.Root>
							<Card.Root size="sm">
								<Card.Header><Card.Title>막힘 · {blocked.length}</Card.Title></Card.Header>
								<Card.Content class="gap-0">
									{#each blocked as b (b.num)}
										{@render taskRow(b.num, b.title, '막힘', issuePath(b.issue), b.updated, b.priority, undefined, 'wait', 'text-status-blocked')}
									{:else}
										<p class="text-xs text-muted-foreground">막힌 태스크가 없어요.</p>
									{/each}
								</Card.Content>
							</Card.Root>
						{/if}
						{#if taskFilter !== 'active'}
							<Card.Root size="sm">
								<Card.Header><Card.Title>최근 완료 · 7일</Card.Title></Card.Header>
								<Card.Content class="gap-0">
									{#each det.done.filter((d) => hit(d.title)) as d (d.num)}
										{@render taskRow(d.num, d.title, undefined, d.tokens, d.when, '', undefined, 'done')}
									{:else}
										<p class="border-t py-3 text-xs text-muted-foreground">검색 결과가 없어요.</p>
									{/each}
								</Card.Content>
							</Card.Root>
						{/if}
						<div class="grid grid-cols-2 gap-3.5">
							<Card.Root size="sm">
								<Card.Header><Card.Title>대기열 규칙</Card.Title></Card.Header>
								<Card.Content class="gap-0">
									{#each [[ArrowDownWideNarrow, '정렬 기준', det.rules.sort], [Play, '자동 시작', det.rules.autostart ? '켜짐' : '꺼짐'], [Shuffle, 'Orch 재배치', det.rules.rebalance ? '허용' : '안 함'], [Layers, '동시 실행', det.rules.runs]] as const as [Icon, l, v] (l)}
										<div class="value-line">
											<Icon class="size-3.25 text-muted-foreground" />
											<span class="flex-1 text-muted-foreground">{l}</span>
											<span class="font-medium">{v}</span>
										</div>
									{/each}
								</Card.Content>
							</Card.Root>
							<Card.Root size="sm">
								<Card.Header><Card.Title>배정 출처 · 30일</Card.Title></Card.Header>
								<Card.Content class="gap-2.5">
									{#each [['Orch 자동 배정', det.source.orch], ['사용자 수동', det.source.manual]] as const as [l, n] (l)}
										<div class="flex flex-col gap-1">
											<span class="flex text-xs"><span class="flex-1 text-muted-foreground">{l}</span><span class="font-mono font-medium">{n}</span></span>
											<Progress value={(n / (det.source.orch + det.source.manual)) * 100} class="h-1.5 bg-muted" aria-label={l} />
										</div>
									{/each}
								</Card.Content>
							</Card.Root>
						</div>
					{:else if det && tab === 'runs'}
						{@const ok = det.runs.filter((r) => r.status === 'done').length}
						{@const fail = det.runs.filter((r) => r.status === 'failed').length}
						{@const total = det.runs.reduce((s, r) => s + r.tokens, 0)}
						{@const sel = det.failure?.run === runSel ? det.failure : undefined}
						<div class="flex gap-3">
							{@render mini('Runs · 30일', String(det.runs.length), `진행 ${det.runs.filter((r) => r.status === 'running').length}`)}
							{@render mini('성공', String(ok), `${Math.round((ok / Math.max(1, ok + fail)) * 100)}% · 완료 ${ok + fail} 중`)}
							{@render mini('실패', String(fail), det.runs.find((r) => r.status === 'failed')?.result ?? '—', fail ? 'text-status-blocked' : undefined)}
							{@render mini('토큰', `${Math.round(total)}K`, `Run당 ${Math.round(total / Math.max(1, det.runs.length))}K`)}
						</div>
						<Card.Root size="sm">
							<Card.Header><Card.Title>최근 7일 Run</Card.Title></Card.Header>
							<Card.Content>{@render chart(det.runDays, ['성공', '실패'], `7일 · ${det.runs.length} runs · 성공 ${ok} · 실패 ${fail}`)}</Card.Content>
						</Card.Root>
						<div class="overflow-hidden rounded-lg border bg-card">
							<Table.Root class="table-fixed">
								<Table.Header>
									<Table.Row class="hover:bg-transparent">
										<Table.Head class="w-10 pl-3.5"><span class="sr-only">상태</span></Table.Head>
										<Table.Head class="w-15">Run</Table.Head>
										<Table.Head>Task</Table.Head>
										<Table.Head class="w-24">시작</Table.Head>
										<Table.Head class="w-16">소요</Table.Head>
										<Table.Head class="w-16">토큰</Table.Head>
										<Table.Head class="w-40 pr-3.5">결과</Table.Head>
									</Table.Row>
								</Table.Header>
								<Table.Body>
									{#each det.runs as r (r.num)}
										{@const ri = runIcon[r.status]}
										<Table.Row
											class={cn('h-11 text-xs', det.failure?.run === r.num && 'cursor-pointer', runSel === r.num && 'bg-primary-soft hover:bg-primary-soft')}
											onclick={() => det.failure?.run === r.num && (runSel = runSel === r.num ? undefined : r.num)}
										>
											<Table.Cell class="pl-3.5"><ri.icon class={cn('size-3.5', ri.text)} aria-label={r.status} /></Table.Cell>
											<Table.Cell class="font-mono font-semibold text-muted-foreground">#{r.num}</Table.Cell>
											<Table.Cell class="truncate">{r.task}</Table.Cell>
											<Table.Cell class="font-mono text-muted-foreground">{r.start}</Table.Cell>
											<Table.Cell class="font-mono text-muted-foreground">{r.dur}</Table.Cell>
											<Table.Cell class="font-mono text-muted-foreground">{r.tokens.toFixed(1)}K</Table.Cell>
											<Table.Cell class={cn('truncate pr-3.5', r.status === 'failed' ? 'text-status-blocked' : 'text-muted-foreground')}>{r.result}</Table.Cell>
										</Table.Row>
									{/each}
								</Table.Body>
							</Table.Root>
						</div>
						{#if sel}
							<section class="flex flex-col gap-3.5" aria-label="Run #{sel.run} 상세">
								<div class="flex items-center gap-2">
									<CircleX class="size-4 text-status-blocked" />
									<h3 class="text-sm font-semibold">Run #{sel.run} · Failed</h3>
									<span class="text-xs text-muted-foreground">{sel.meta}</span>
								</div>
								<div class="grid grid-cols-2 gap-3.5">
									<Card.Root size="sm">
										<Card.Header><Card.Title>실행 단계</Card.Title></Card.Header>
										<Card.Content class="gap-1.5">
											{#each sel.steps as step (step.name)}
												<div class="flex items-center gap-2 text-xs">
													{#if step.state === 'done'}<CircleCheck class="size-3.5 text-status-done" />{:else if step.state === 'failed'}<CircleX class="size-3.5 text-status-blocked" />{:else}<CircleDashed class="size-3.5 text-subtle-foreground" />{/if}
													<span class={cn('flex-1', step.state === 'failed' && 'font-medium text-status-blocked', step.state === 'skipped' && 'text-muted-foreground')}>{step.name}</span>
													<span class="font-mono text-caption text-muted-foreground">{step.time}</span>
												</div>
											{/each}
										</Card.Content>
									</Card.Root>
									<Card.Root size="sm">
										<Card.Header><Card.Title>실패 원인</Card.Title></Card.Header>
										<Card.Content class="gap-2">
											<pre class="pre-error">{sel.errors.join('\n')}</pre>
											<span class="meta-xs gap-1.5"><Redo2 class="size-3.5 text-primary" />{sel.retry}</span>
										</Card.Content>
									</Card.Root>
								</div>
								<Card.Root size="sm">
									<Card.Header><Card.Title>변경 파일 · {sel.files.length}</Card.Title></Card.Header>
									<Card.Content class="gap-1.5">
										{#each sel.files as f (f.path)}
											<div class="run-meta">
												<span class={cn('w-3 font-semibold', f.kind === 'A' ? 'text-status-done' : 'text-status-waiting')}>{f.kind}</span>
												<span class="flex-1 truncate">{f.path}</span>
												<span class="text-muted-foreground">{f.diff}</span>
											</div>
										{/each}
									</Card.Content>
								</Card.Root>
							</section>
						{/if}
					{:else if det && tab === 'activity'}
						<div class="flex flex-wrap items-center gap-2">
							<Toggle variant="chip" pressed={actFilter === 'all'} onPressedChange={() => (actFilter = 'all')}>전체</Toggle>
							{#each actKinds as { k, n } (k)}
								<Toggle variant="chip" count={n} pressed={actFilter === k} onPressedChange={() => (actFilter = k)}>{k}</Toggle>
							{/each}
						</div>
						<div class="card-col gap-2 px-4 py-3">
							{#each det.activity as d (d.day)}
								{@const items = d.items.filter((a) => actFilter === 'all' || actKind(a) === actFilter)}
								{#if items.length}
									<span class="list-label pt-1">{d.day}</span>
									{#each items as a, i (i)}
										{#if a.type === 'decision'}
											<div class="outline-row">
												<BookMarked class={cn('size-3.5', a.orch ? 'text-status-review' : 'text-primary')} />
												<span class="font-semibold">{a.who}</span>
												<span class="soft-tag font-medium">{a.kind}</span>
												<span class="flex-1">{a.text}</span>
												<span class="font-mono text-caption text-subtle-foreground">{a.time}</span>
											</div>
										{:else}
											{@const KindIcon = kindIcon[a.kind]}
											<div class="row-divided gap-2.5 py-2">
												<RoleAvatar role={a.role} />
												<div class="col-fill gap-0.75">
													<div class="flex items-center gap-1.5 text-xs">
														<span class="font-semibold">{a.who}</span>
														<Badge variant="secondary" class="gap-1"><KindIcon class="size-3" />{a.kind}</Badge>
														<span class="flex-1 truncate">{a.title}</span>
														<span class="font-mono text-caption text-subtle-foreground">{a.time}</span>
													</div>
													{#if a.note}<span class="text-xs text-muted-foreground">{a.note}</span>{/if}
												</div>
											</div>
										{/if}
									{/each}
								{/if}
							{/each}
						</div>
						<Card.Root size="sm">
							<Card.Header><Card.Title>협업 관계 · 7일</Card.Title></Card.Header>
							<Card.Content class="gap-2.5">
								{#each det.relations as r (r.name)}
									<div class="flex items-center gap-2.5">
										<RoleAvatar role={r.role} />
										<span class="flex w-40 flex-col gap-px">
											<span class="text-xs font-medium">{r.name} · {r.label}</span>
											<span class="text-caption text-muted-foreground">{r.note}</span>
										</span>
										<Progress value={r.value} class="h-1.5 bg-muted" aria-label="{r.name} 협업 빈도" />
									</div>
								{/each}
							</Card.Content>
						</Card.Root>
					{:else if det && tab === 'usage'}
						{@const acc = accounts.find((a) => a.runtime === m.runtime)!}
						{@const top = det.runs.reduce((a, b) => (b.tokens > a.tokens ? b : a))}
						<div class="flex items-center gap-3">
							<h3 class="text-xl font-bold">사용량</h3>
							<span class="text-xs text-muted-foreground">{m.name} · 토큰 · 한도 · 비용 · 이번 주</span>
						</div>
						<div class="flex gap-3">
							{@render mini('이번 주 토큰', `${det.week.tokens}K`, `${acc.name} 주간의 62%`)}
							{@render mini('오늘', k(m.tokens), `팀 오늘 ${k(tokens)}의 ${Math.round((m.tokens / tokens) * 100)}%`)}
							{@render mini('Run당 평균', `${Math.round(det.runs.reduce((s, r) => s + r.tokens, 0) / det.runs.length)}K`, `${det.runs.length} Run · 최대 ${top.tokens.toFixed(1)}K (#${top.num})`)}
							{@render mini('비용 환산', '$0', '구독 내 사용 · 폴백 API $0')}
						</div>
						<Card.Root size="sm">
							<Card.Header><Card.Title>이번 주 토큰 사용</Card.Title></Card.Header>
							<Card.Content>{@render chart(det.week.days.map((d) => [d]), ['토큰 (K)'], `이번 주 · ${det.week.tokens}K · 하루 평균 ${Math.round(det.week.tokens / 7)}K`)}</Card.Content>
						</Card.Root>
						<div class="grid grid-cols-2 gap-3.5">
							{#each [acc, ...accounts.filter((a) => a !== acc)] as a (a.runtime)}
								{@const warn = low(a.week)}
								<div class={cn('fallback-card bg-card', warn && 'border-destructive bg-destructive-soft')}>
									<div class="flex items-center gap-2">
										<RuntimeLogo runtime={a.runtime} class="size-5 ring-0" />
										<span class="flex flex-1 flex-col"><span class="text-body font-semibold">{a.name}</span><span class="font-mono text-caption text-muted-foreground">{a.login}</span></span>
										{#if a.runtime === m.runtime}<Pill class="text-2xs">{m.name}</Pill>{/if}
									</div>
									{#each [{ label: '5H', pct: a.h5, reset: a.h5Reset }, { label: '주간', pct: a.week, reset: a.weekReset }] as q (q.label)}
										<div class="flex flex-col gap-1">
											<span class="flex gap-1.5 text-caption">
												<span class="flex-1 text-muted-foreground">{q.label}</span>
												<span class={cn('font-semibold', low(q.pct) && 'text-destructive')}><span class="font-mono">{q.pct}%</span> 남음</span>
												<span class="text-subtle-foreground">· {q.reset} 리셋</span>
											</span>
											<Progress value={q.pct} class="h-1.25 bg-muted" indicator={low(q.pct) ? 'bg-destructive' : 'bg-success'} aria-label="{a.name} {q.label} 잔량" />
										</div>
									{/each}
								</div>
							{/each}
						</div>
						<Card.Root size="sm">
							<Card.Header>
								<Card.Title>Run별 토큰</Card.Title>
								<Card.Description>이번 주 {det.runs.length} Run · 합계 {det.runs.reduce((s, r) => s + r.tokens, 0).toFixed(1)}K</Card.Description>
							</Card.Header>
							<Card.Content class="gap-0">
								{#each [...det.runs].sort((a, b) => b.tokens - a.tokens) as r (r.num)}
									<KeyValueRow label={`#${r.num} · ${r.task} · ${r.start}${r.status === 'running' ? ' (진행 중)' : r.status === 'failed' ? ' (실패)' : r.status === 'cancelled' ? ' (취소)' : ''}`} value={`${r.tokens.toFixed(1)}K`} />
								{/each}
							</Card.Content>
						</Card.Root>
					{:else if viewed?.config && tab === 'harness'}
						<HarnessPanel
							config={viewed.config}
							base={origin?.config}
							runtime={m.runtime}
							model={m.model.split(' · ')[1] ?? m.model}
							baseRt={origin ? { runtime: origin.runtime, model: origin.model } : undefined}
							onchange={(r, md) => ((m.runtime = r), (m.model = `${runtimeName(r)} · ${md}`))}
						/>
					{:else if viewed?.config && tab === 'skills'}
						<SkillsPanel config={viewed.config} base={origin?.config} runtime={m.runtime} target={m.name} />
					{:else if viewed?.config && tab === 'tools'}
						<ToolsPanel config={viewed.config} base={origin?.config} teamName={team.name} />
					{:else if viewed?.config && tab === 'perm'}
						<PermPanel config={viewed.config} base={origin?.config} />
					{:else if tab === 'instructions'}
						<MdEditor
							files={memberFiles}
							bind:active={memberFile}
							bind:mode={memberMode}
							base={origin ? liveMap(origin.files) : undefined}
							baseLabel={origin ? `템플릿 v${origin.version}` : '템플릿'}
							currentLabel="{m.name} · 현재"
							class="h-150"
						>
							{#snippet status()}→ AGENTS.md / CLAUDE.md로 변환{/snippet}
						</MdEditor>
						<div class="picked-card">
							<span class="flex-1 text-xs text-muted-foreground">
								{#if proposed}<span class="text-primary">{proposed}을 템플릿 초안에 제안했어요 · </span>{/if}
								{memberChanged.length ? `템플릿과 다른 파일 ${memberChanged.length}개 · 이 멤버에게만 적용 중` : '템플릿과 같아요'}
							</span>
							<Button variant="ghost" size="sm" disabled={!memberChanged.some((f) => f.name === memberFiles[memberFile]?.name)} onclick={revertFile}><Undo2 />템플릿으로 되돌리기</Button>
							<Button size="sm" disabled={!memberChanged.some((f) => f.name === memberFiles[memberFile]?.name)} onclick={propose}><Upload />템플릿에 제안</Button>
						</div>
					{:else}
						<Empty.Root class="bg-card">
							<Empty.Header>
								<Empty.Media variant="icon"><SlidersHorizontal /></Empty.Media>
								<Empty.Title>{nav.flatMap((g) => g.items).find((i) => i.v === tab)?.label}</Empty.Title>
								<Empty.Description>템플릿 상세(#65)와 같은 설정 화면이라 함께 만들어요. (#63 T-3b)</Empty.Description>
							</Empty.Header>
						</Empty.Root>
					{/if}
				</div>
			</Sheet.Body>
		{/if}
	</Sheet.Content>
</Sheet.Root>

<!-- Orch 진행 정책 편집 (.pen Team Settings · Orch 진행) -->
<Dialog.Root bind:open={() => draft !== undefined, (v) => !v && (draft = undefined)}>
	<Dialog.Content size="full" tall>
		{#if draft}
			{@const d = draft}
			<Dialog.Header crumb={[team.name, '팀 설정']} closable={false}>
				<Dialog.Title>Orch 진행 정책</Dialog.Title>
				<Dialog.Description>다음 작업 배정 · 분배 · 사용자 판단이 필요한 순간을 Orch가 어떻게 처리할지 정해요.</Dialog.Description>
				{#snippet actions()}
					<Button variant="ghost" size="sm" onclick={() => (draft = undefined)}>취소</Button>
					<Button size="sm" disabled={!!spawnError || !!childError} onclick={savePolicy}>저장</Button>
				{/snippet}
			</Dialog.Header>
			<Dialog.Body class="gap-7">
				<section class="flex flex-col gap-3">
					{@render heading('1 · 진행 모드', '태스크가 끝났을 때 Orch가 다음 행동을 제안하고 실행하는 방식')}
					<div role="radiogroup" aria-label="진행 모드" class="grid grid-cols-3 gap-3">
						{#each [{ v: 'manual', icon: Hand, t: 'Manual', d: '매번 사용자가 확인해야 진행' }, { v: 'timer', icon: Timer, t: 'Auto · 타이머', d: '제안 후 카운트다운 · 개입 없으면 진행' }, { v: 'full', icon: Zap, t: 'Full auto', d: '대기 없이 바로 진행 (가드는 항상 적용)' }] as const as o (o.v)}
							{@const on = d.mode === o.v}
							<div class={cn('box-col gap-2 rounded-md p-3.5', on && 'option-on')}>
								<button type="button" role="radio" aria-checked={on} onclick={() => (d.mode = o.v)} class="option-link">
									<span class="row-title-strong">
										<o.icon class={cn('size-4', on ? 'text-primary' : 'text-muted-foreground')} />
										<span class="flex-1">{o.t}</span>
										{#if on}<CircleCheck class="size-4 text-primary" />{:else}<Circle class="size-4 text-subtle-foreground" />{/if}
									</span>
									<span class="text-xs text-muted-foreground">{o.d}</span>
								</button>
								{#if o.v === 'timer'}
									<div class="tag-row-xs">
										<span class="mr-1 text-muted-foreground">대기</span>
										{#each [3, 5, 10, 30] as sec (sec)}
											{@const sel = on && !customTimer && d.timer === sec}
											<button
												type="button"
												aria-pressed={sel}
												onclick={() => ((d.mode = 'timer'), (d.timer = sec), (customTimer = false))}
												class={cn('toggle-tag', sel ? 'border-primary bg-primary text-on-solid' : 'bg-card')}
											>{sec}초</button>
										{/each}
										{#if customTimer}
											<Input type="number" min={1} max={600} bind:value={d.timer} aria-label="대기 초" class="h-6.5 w-16 px-2 text-xs" />
										{:else}
											<button type="button" onclick={() => ((d.mode = 'timer'), (customTimer = true))} class="toggle-tag-on">직접</button>
										{/if}
									</div>
								{/if}
							</div>
						{/each}
					</div>
				</section>

				<section class="flex flex-col gap-3">
					{@render heading('2 · 작업 레벨별 처리', '결정의 무게에 따라 처리 방식을 나눠요. 레벨이 높을수록 사람의 판단이 필요해요.')}
					<div class="overflow-hidden rounded-md border">
						<div class="list-table-head gap-4 px-4 py-2">
							<span class="w-57.5">레벨</span><span class="flex-1">예시</span><span class="w-75">처리</span><span class="w-42.5">응답 없으면</span>
						</div>
						{#each d.levels as l, i (i)}
							<div class="list-table-row gap-4 py-3">
								<span class="flex w-57.5 items-center gap-2.5">
									<span class={cn('mono-tag py-px text-on-solid', levelTone[i].bg)}>L{i}</span>
									<span class="text-body font-semibold">{l.name}</span>
								</span>
								<span class="flex-1 text-xs text-muted-foreground">{l.example}</span>
								<Segmented class="w-75" options={actions} bind:value={() => l.action, (v) => (l.action = v as LevelAction)} disabled={l.locked} aria-label="L{i} 처리" />
								<span class="w-42.5 text-xs">
									{#if l.locked}
										<span class="text-subtle-foreground">변경 불가</span>
									{:else if l.action === 'timer'}
										<span>{d.mode === 'timer' ? `${d.timer}초 후 진행` : '모드 타이머 사용'}</span>
									{:else if l.action === 'wait'}
										<Select.Root type="single" bind:value={l.timeout}>
											<Select.Trigger size="sm" class="h-7 w-full">{l.timeout ? `${l.timeout} 후 Orch가 판단` : '계속 대기'}</Select.Trigger>
											<Select.Content>
												{#each waitOptions as o (o)}<Select.Item value={o} label={o ? `${o} 후 Orch가 판단` : '계속 대기'} />{/each}
											</Select.Content>
										</Select.Root>
									{:else}
										<span class="text-subtle-foreground">—</span>
									{/if}
								</span>
							</div>
						{/each}
					</div>
					<p class="info-line">
						<Info class="size-3.5 shrink-0" />“대기”는 사용자 응답을 기다리되, 정해진 시간이 지나면 Orch가 근거를 남기고 판단해요. 그 판단은 Activity에 ‘사용자 대신 결정’으로 표시돼요.
					</p>
				</section>

				<section class="flex flex-col gap-3">
					{@render heading('3 · 루프 가드', '모드와 상관없이 조건에 걸리면 자동 진행을 멈추고 알려요. 사용자가 자리를 비워도 무한 반복되지 않게 해요.')}
					<div class="grid grid-cols-3 gap-2.5">
						{#each d.guards as g (g.key)}
							{@const Icon = guardIcon[g.key]}
							<label class="card-row gap-2.5 p-3">
								<Icon class="size-4 shrink-0 text-muted-foreground" />
								<span class="row-text">
									<span class="text-body font-medium">{g.name}</span>
									<span class="truncate text-caption text-muted-foreground">{g.value}</span>
								</span>
								<Switch bind:checked={g.on} aria-label={g.name} />
							</label>
						{/each}
					</div>
				</section>

				<section class="flex flex-col gap-3">
					{@render heading('4 · 하위 작업', '리드 에이전트가 태스크 안에서 작업을 나눠 돌리는 방식이에요. 태스크마다 바꿀 수 있고, 비워 두면 이 기본값을 써요 (Task 상세 › 하위 작업 방식).')}
					<div class="grid grid-cols-3 gap-2.5">
						<div class="box-col gap-2.5 rounded-md p-3.5">
							<span class="text-caption font-semibold text-muted-foreground">기본 방식</span>
							<Segmented
								aria-label="기본 방식"
								class={cn(spawnError && 'ring-1 ring-destructive')}
								options={spawnModes.map((m) => ({ value: m.value, label: m.value, icon: m.icon, disabled: !d.spawn.allow.includes(m.value), hint: '허용 방식에서 꺼져 있어요' }))}
								bind:value={() => d.spawn.mode, (v) => (d.spawn.mode = v as SpawnMode)}
							/>
							{#if spawnError}
								<span class="error-note"><CircleAlert class="size-3 shrink-0" />{spawnError}</span>
							{:else}
								<span class="text-caption text-muted-foreground">runner · 별도 Run으로 돌려요. 모델 등급(S/M/L)은 작업 종류로 정해지고, 토큰은 따로 집계돼요.</span>
							{/if}
						</div>
						<div class="box-col gap-2 rounded-md p-3.5">
							<span class="text-caption font-semibold text-muted-foreground">허용 방식</span>
							{#each spawnDesc as [m, desc] (m)}
								<label class="flex items-center gap-2 text-xs"><Checkbox checked={d.spawn.allow.includes(m)} onCheckedChange={(v) => toggleAllow(d, m, !!v)} aria-label="{m} 허용" />{m} · {desc}</label>
							{/each}
						</div>
						<div class="box-col gap-2.5 rounded-md p-3.5">
							<span class="text-caption font-semibold text-muted-foreground">리드당 동시 하위 작업</span>
							<InputGroup.Root class={cn('w-28', childError && 'border-destructive')}>
								<InputGroup.Addon><Layers /></InputGroup.Addon>
								<InputGroup.Input type="number" min="1" bind:value={d.spawn.maxChild} aria-label="리드당 동시 하위 작업" class="font-mono" />
							</InputGroup.Root>
							{#if childError}
								<span class="error-note"><CircleAlert class="size-3 shrink-0" />{childError}</span>
							{/if}
							<span class="text-caption text-muted-foreground">최소 1 · paths가 겹치는 하위 작업은 규칙 엔진이 순서대로 돌려요 (queued · waits).</span>
						</div>
					</div>
					{#if d.spawn.allow.includes('fork')}
						<Alert.Root variant="warning">
							<GitFork />
							<Alert.Title>fork는 부모 컨텍스트를 그대로 상속해요</Alert.Title>
							<Alert.Description>켜면 부모 컨텍스트를 그대로 복사해 비용이 커요.</Alert.Description>
						</Alert.Root>
					{/if}
				</section>
			</Dialog.Body>
		{/if}
	</Dialog.Content>
</Dialog.Root>
