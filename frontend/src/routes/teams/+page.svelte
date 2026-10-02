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
	import GitCompare from '@lucide/svelte/icons/git-compare';
	import ArrowDownWideNarrow from '@lucide/svelte/icons/arrow-down-wide-narrow';
	import Shuffle from '@lucide/svelte/icons/shuffle';
	import MessageSquarePlus from '@lucide/svelte/icons/message-square-plus';
	import MessageCircle from '@lucide/svelte/icons/message-circle';
	import ArrowRightLeft from '@lucide/svelte/icons/arrow-right-left';
	import { Empty, EmptyHeader, EmptyMedia, EmptyTitle, EmptyDescription } from '$lib/components/ui/empty';
	import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogBody, DialogFooter } from '$lib/components/ui/dialog';
	import { FieldRow } from '$lib/components/ui/field';
	import Archive from '@lucide/svelte/icons/archive';
	import Trash2 from '@lucide/svelte/icons/trash-2';
	import Users from '@lucide/svelte/icons/users';
	import { toast } from 'svelte-sonner';
	import { api, failureOf } from '$lib/api/client';
	import { goto } from '$app/navigation';
	import { MdEditor, estimateTokens, type MdFile } from '$lib/components/orch/md-editor';
	import Upload from '@lucide/svelte/icons/upload';
	import { Segmented } from '$lib/components/orch/segmented';
	import { Checkbox } from '$lib/components/ui/checkbox';
	import CircleAlert from '@lucide/svelte/icons/circle-alert';
	import GitFork from '@lucide/svelte/icons/git-fork';
	import { Switch } from '$lib/components/ui/switch';
	import Hand from '@lucide/svelte/icons/hand';
	import Zap from '@lucide/svelte/icons/zap';
	import Repeat from '@lucide/svelte/icons/repeat';
	import ListPlus from '@lucide/svelte/icons/list-plus';
	import Moon from '@lucide/svelte/icons/moon';
	import { untrack, type Component } from 'svelte';
	import { DecisionRecord } from '$lib/components/orch/decision-record';
	import { Sheet, SheetContent, SheetHeader, SheetTitle, SheetBody, SheetFooter } from '$lib/components/ui/sheet';
	import { Select, SelectTrigger, SelectContent, SelectItem } from '$lib/components/ui/select';
	import { Steps } from '$lib/components/orch/steps';
	import { Input } from '$lib/components/ui/input';
	import { Badge } from '$lib/components/ui/badge';
	import type { Role } from '$lib/roles';
	import type { Runtime } from '$lib/components/orch/runtime-logo';
	import { AvatarGroup } from '$lib/components/ui/avatar';
	import { Card, CardHeader, CardTitle, CardContent, CardAction, CardDescription } from '$lib/components/ui/card';
	import { Table, TableHeader, TableRow, TableHead, TableBody, TableCell } from '$lib/components/ui/table';
	import { Alert, AlertTitle, AlertDescription } from '$lib/components/ui/alert';
	import { InputGroup, InputGroupAddon, InputGroupInput } from '$lib/components/ui/input-group';
	import { Button } from '$lib/components/ui/button';
	import { Toggle } from '$lib/components/ui/toggle';
	import { Pill } from '$lib/components/orch/pill';
	import { Progress } from '$lib/components/ui/progress';
	import { RoleAvatar } from '$lib/components/orch/role-avatar';
	import { RuntimeLogo } from '$lib/components/orch/runtime-logo';
	import { accounts, meters, teamPolicy, recommend, tasks, issues, memberDetails, type TeamMember, type OrchPolicy, type LevelAction, type SpawnMode } from '$lib/mock';
	import { store, defaultTeam, glyphs, glyphOf, runtimeName, accountOf, low, scopeText, liveMap, k, putDraft, addMember, saveTeamSpawn } from '$lib/teams.svelte';
	import { useMock } from '$lib/api/env';
	import { LimitRow } from '$lib/components/orch/limit-row';
	import { KeyValueRow } from '$lib/components/orch/key-value-row';
	import SkillsPanel from '$lib/components/orch/agent/skills-panel.svelte';
	import ToolsPanel from '$lib/components/orch/agent/tools-panel.svelte';
	import PermPanel from '$lib/components/orch/agent/perm-panel.svelte';
	import HarnessPanel from '$lib/components/orch/agent/harness-panel.svelte';
		import { ChoiceCards, ChoiceCard } from '$lib/components/orch/choice-cards';

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
	// 팀 정책 (.pen 팀 설정) — 목데이터는 화면 상태로 고친다. repo는 범위 · 권한으로 나눠 둔다.
	const repoPerms = { read: '읽기만', branch: '브랜치', push: 'push' } as const;
	type RepoPerm = keyof typeof repoPerms;
	let tp = $state({ ...teamPolicy, reviewOn: true, repoScope: teamPolicy.repo.split(' · ')[0], repoPerm: (teamPolicy.repo.split(' · ')[1] ?? 'branch') as RepoPerm });
	const hot = $derived(hottest.context >= tp.contextWarn);
	/// 팀 설정 (.pen Teams · 팀 설정) — 헤더 '팀 편집' · 팀 정책 카드 '편집'이 같은 창을 연다.
	let teamEdit = $state<{ name: string; desc: string; tokenBudget: number; contextWarn: number; maxRuns: number; reviewOn: boolean; review: string; repoScope: string; repoPerm: RepoPerm }>();
	let teamSaving = $state(false);
	let confirmDelete = $state(false);
	function editTeam() {
		confirmDelete = false;
		teamEdit = { name: team.name, desc: team.desc, tokenBudget: tp.tokenBudget, contextWarn: tp.contextWarn, maxRuns: tp.maxRuns, reviewOn: tp.reviewOn, review: tp.review, repoScope: tp.repoScope, repoPerm: tp.repoPerm };
	}
	/// 저장 — 서버 모드는 PATCH /teams/{sn} (이름 · 한도 · 리뷰 · 저장소). 실패하면 창을 그대로 둔다.
	async function saveTeam() {
		const e = teamEdit;
		if (!e || !e.name.trim()) return;
		if (!useMock) {
			teamSaving = true;
			const res = await api
				.PATCH('/teams/{sn}', { params: { path: { sn: team.sn } }, body: { name: e.name.trim(), daily_token_budget: e.tokenBudget * 1000, context_warn_percent: e.contextWarn, max_concurrent_run: e.maxRuns, is_review_required: e.reviewOn ? 1 : 0, review_stage: e.review === 'PR 병합 전' ? 'before_merge' : 'before_done', repo_scope: e.repoScope, repo_permission: e.repoPerm } })
				.catch(() => undefined);
			teamSaving = false;
			if (!res || res.error) return;
		}
		team.name = e.name.trim();
		team.desc = e.desc;
		Object.assign(tp, { tokenBudget: e.tokenBudget, contextWarn: e.contextWarn, maxRuns: e.maxRuns, reviewOn: e.reviewOn, review: e.review, repoScope: e.repoScope, repoPerm: e.repoPerm });
		teamEdit = undefined;
		toast.success(`${team.name} 설정을 저장했어요`);
	}
	/// 보관(목데이터: 목록에서 뺌) · 삭제(한 번 더 눌러 확정). 서버 삭제가 막히면(Run 기록 · 409) 문구로 알린다.
	async function removeTeam(kind: 'archive' | 'delete') {
		if (kind === 'delete' && !confirmDelete) return void (confirmDelete = true);
		const name = team.name;
		if (!useMock && kind === 'delete') {
			const res = await api.DELETE('/teams/{sn}', { params: { path: { sn: team.sn } } }).catch(() => undefined);
			if (!res) return;
			if (res.error) return void toast.warning(failureOf(res.error, res.response).message);
		}
		store.crew = store.crew.filter((t) => t.sn !== team.sn);
		teamEdit = undefined;
		toast.success(kind === 'archive' ? `${name} 팀을 보관했어요` : `${name} 팀을 삭제했어요`);
		goto('/teams', { replaceState: true });
	}
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
	// 서버 모드 저장 — 하위 작업 정책만 서버(tbl_team)에, Orch 진행 레벨 · 가드는 아직 화면 상태(#60). 서버 검증 문구는 그 칸 아래에.
	let policySaving = $state(false);
	let spawnServerError = $state('');
	async function savePolicy() {
		if (!draft) return;
		if (!useMock) {
			policySaving = true;
			spawnServerError = (await saveTeamSpawn(team.sn, draft.spawn)) ?? '';
			policySaving = false;
			if (spawnServerError) return;
		}
		store.policies[team.sn] = draft;
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
	// 다른 화면에서 여는 주소 — 쓰고 나면 주소에서 뗀다.
	// ?add=템플릿 → 멤버 추가 시트 (템플릿 화면 "팀에 추가" · 값이 비면 추천 템플릿, Workbench Menu / Add)
	// ?member=멤버 → 멤버 상세 (Workbench 에이전트 메뉴 "멤버 상세 열기")
	$effect(() => {
		const add = page.url.searchParams.get('add');
		const member = Number(page.url.searchParams.get('member'));
		if (add === null && !member) return;
		untrack(() => {
			if (add !== null) startAdd(Number(add) || undefined);
			else openMember(member);
		});
		const url = new URL(page.url);
		url.searchParams.delete('add');
		url.searchParams.delete('member');
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
	/// 팀에 추가. 서버 모드는 POST 후 그 팀 멤버를 다시 읽는다 — 런타임 · 모델 · 지침 파일은 프로필 API(#47 · #45) 뒤에 저장.
	let addSaving = $state(false);
	async function add() {
		if (!useMock) {
			addSaving = true;
			// 템플릿 목록은 아직 화면 데이터라 template_sn은 보내지 않는다 — 서버 템플릿(지침 파일 · 리비전 #45) 연결 뒤에
			const sn = await addMember(team.sn, { name: name.trim(), role_name: tpl?.name ?? 'Agent', first_task_mode: first });
			addSaving = false;
			if (sn === undefined) return;
			adding = false;
			openMember(sn);
			return;
		}
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

{#snippet kpi(label: string, value: string, sub: string, delta?: string, trend?: number[])}
	<div class="flex flex-1 flex-col gap-1 border-r px-4.5 py-3.5 last:border-r-0">
		<span class="text-xs text-muted-foreground">{label}</span>
		<span class="flex items-end gap-2">
			<span class="font-mono text-xl font-semibold whitespace-nowrap">{value}</span>
			{#if delta}<span class="pb-0.5 font-mono text-xs font-medium text-status-done">{delta}</span>{/if}
			<span class="flex-1"></span>
			{#if trend}
				<span class="flex h-6 w-18 items-end gap-1" aria-hidden="true">
					{#each trend as h, i (i)}
						<span class={['flex-1 rounded-t-xs bg-primary', i < trend.length - 1 && 'opacity-35']} style="height: {h}px"></span>
					{/each}
				</span>
			{/if}
		</span>
		<span class="text-caption text-subtle-foreground">{sub}</span>
	</div>
{/snippet}

<!-- 역할 아바타 + 오른쪽 아래 실행기 로고 -->
{#snippet who(m: TeamMember, small = false)}
	<RoleAvatar role={m.role} icon={glyphOf(m)} size={small ? 'sm' : 'default'} class="overflow-visible">
		<RuntimeLogo runtime={m.runtime} class={['absolute -right-1 -bottom-1 ring-[1.5px]', small ? 'size-2.5' : 'size-3']} />
	</RoleAvatar>
{/snippet}

<!-- Team Detail -->
<main class="flex min-w-0 flex-1 flex-col overflow-y-auto">
		<header class="flex flex-col gap-4 border-b px-8 pt-6 pb-5">
			<nav aria-label="Breadcrumb" class="meta-xs gap-1.5">
				<span>Teams</span>
				<ChevronRight class="size-3" />
				<span class="font-medium text-foreground">{team.name}</span>
			</nav>
			<div class="flex items-center gap-4">
				<div class="flex flex-1 flex-col gap-1.5">
					<div class="flex items-center gap-2.5">
						<h1 class="text-2xl font-bold">{team.name}</h1>
						{#if team.project}<Pill dot="bg-success" class="rounded-sm px-2 py-1 text-xs text-foreground">{team.project}</Pill>{/if}
					</div>
					<p class="max-w-160 text-body text-muted-foreground">{team.desc}</p>
				</div>
				<Button variant="outline" size="sm" class="text-body" onclick={editTeam}><Pencil class="size-3.5" />팀 편집</Button>
				<Button size="sm" class="text-body" onclick={() => startAdd()}><UserPlus class="size-3.5" />멤버 추가</Button>
			</div>
			<div class="card flex rounded-lg">
				{@render kpi('멤버', String(team.members.length), `${running} running · ${count('waiting')} waiting · ${count('idle')} idle`)}
				{@render kpi('열린 태스크', String(team.stats.open), team.stats.openNote)}
				{@render kpi('이번 주 완료', String(team.stats.done), team.stats.doneNote, team.stats.doneDelta, team.stats.doneTrend)}
				{@render kpi('오늘 토큰', k(tokens), `예산 ${tp.tokenBudget}K · ${Math.round((tokens / tp.tokenBudget) * 100)}%`, undefined, team.stats.tokenTrend)}
				{@render kpi('평균 사이클', team.stats.cycle, 'Task 생성 → Done', team.stats.cycleDelta, team.stats.cycleTrend)}
			</div>
		</header>

		<!-- 3xl(1760px) 미만에서는 사이드 카드를 멤버 아래로 내린다 (멤버 표 최소 폭 확보). -->
		<div class="flex flex-col gap-6 px-8 py-6 3xl:flex-row">
			<section class="flex min-w-0 flex-1 flex-col gap-3" aria-label="멤버">
				<div class="flex items-center gap-2">
					<h2 class="text-sm font-semibold">멤버</h2>
					<span class="flex-1 text-body text-muted-foreground">{team.members.length}</span>
					<Toggle variant="chip" count={team.members.length} pressed={filter === 'all'} onPressedChange={() => (filter = 'all')}>All</Toggle>
					{#each Object.entries(states) as [s, v] (s)}
						<Toggle variant="chip" count={count(s as State)} pressed={filter === s} onPressedChange={() => (filter = s as State)}>{v.label}</Toggle>
					{/each}
				</div>

				{#each alerts as { a, hit } (a.runtime)}
					<Alert variant="destructive" class="flex items-center gap-3">
						<TriangleAlert />
						<div class="flex min-w-0 flex-1 flex-col gap-0.5">
							<AlertTitle>{a.name} 계정 주간 잔량 {a.week}% — {a.weekReset} 리셋까지 {a.left}</AlertTitle>
							<AlertDescription>{hit.map((m) => m.name).join(' · ')} 영향. {a.forecast}. 정책: 잔량 {teamPolicy.quotaWarn}% 미만 → 확인 요청</AlertDescription>
						</div>
						<Button variant="outline" size="sm" class="h-7.5 text-xs"><Filter class="size-3.5" />P0–P1만 실행</Button>
						<Button variant="outline" size="sm" class="h-7.5 text-xs"><Route class="size-3.5" />폴백 적용 · Anthropic Max</Button>
					</Alert>
				{/each}

				<div class="card overflow-hidden rounded-lg">
					<Table class="table-fixed">
						<TableHeader class="bg-muted">
							<TableRow class="hover:bg-muted">
								<TableHead class="h-11 w-48 pl-4">멤버</TableHead>
								<TableHead class="h-11 w-28">상태</TableHead>
								<TableHead class="h-11">현재 작업</TableHead>
								<TableHead class="h-11 w-56">Runtime</TableHead>
								<TableHead class="h-11 w-28">컨텍스트</TableHead>
								<TableHead class="h-11 w-32 leading-tight">주간 잔량<br /><span class="text-2xs font-normal text-subtle-foreground">계정 공유</span></TableHead>
								<TableHead class="h-11 w-22 pr-4 text-right">오늘 토큰</TableHead>
							</TableRow>
						</TableHeader>
						<TableBody>
							{#each members as m (m.sn)}
								{@const st = states[m.status]}
								{@const week = weekOf(m)}
								{@const full = m.context >= tp.contextWarn}
								<TableRow class="h-15 cursor-pointer" onclick={() => openMember(m.sn)}>
									<TableCell class="pl-4">
										<span class="flex items-center gap-2.5">
											{@render who(m)}
											<span class="flex min-w-0 flex-col gap-px">
												<button type="button" class="text-left text-body font-semibold outline-none hover:underline focus-visible:underline" onclick={(e) => { e.stopPropagation(); openMember(m.sn); }}>{m.name}</button>
												<span class="truncate text-caption text-muted-foreground">{m.title}</span>
											</span>
										</span>
									</TableCell>
									<TableCell>
										<span class={['label-xs', st.text]}><st.icon class="size-3.5" />{st.label}</span>
									</TableCell>
									<TableCell>
										<span class={['flex items-center gap-1.5 text-xs', m.status === 'idle' && 'text-muted-foreground']}>
											{#if m.next}<CircleDashed class="size-3.5 shrink-0 text-node-task" />{:else}<SquareCheck class="size-3.5 shrink-0 text-node-task" />{/if}
											<span class="truncate">{m.work}</span>
										</span>
									</TableCell>
									<TableCell>
										<span class="code-tag font-medium">
											<Terminal class="size-2.5" />{m.model}
										</span>
									</TableCell>
									<TableCell>
										<span class="flex items-center gap-2">
											<Progress value={m.context} class="h-1.5 bg-muted" indicator={full ? 'bg-status-blocked' : undefined} aria-label="{m.name} 컨텍스트" />
											<span class={['font-mono text-caption', full ? 'font-semibold text-status-blocked' : 'text-muted-foreground']}>{m.context}%</span>
										</span>
									</TableCell>
									<TableCell>
										<span class="flex items-center gap-1.5 text-caption">
											<span class="font-medium text-muted-foreground">주</span>
											<Progress value={week} class="h-1 w-16 bg-muted" indicator={low(week) ? 'bg-destructive' : 'bg-success'} aria-label="{m.name} 주간 잔량" />
											<span class={['font-mono', low(week) ? 'font-semibold text-destructive' : 'font-medium']}>{week}%</span>
										</span>
									</TableCell>
									<TableCell class="pr-4 text-right font-mono text-xs">{k(m.tokens)}</TableCell>
								</TableRow>
							{:else}
								<TableRow>
									<TableCell colspan={7} class="h-15 text-center text-xs text-muted-foreground">해당 상태의 멤버가 없어요.</TableCell>
								</TableRow>
							{/each}
						</TableBody>
					</Table>
				</div>

				<Card size="sm">
					<CardHeader><CardTitle>작업량</CardTitle></CardHeader>
					<CardContent>
						<div class="flex items-center gap-3">
							{#each Object.values(loads) as l (l.label)}
								<span class="meta-line gap-1.5"><span class={['size-2 rounded-xs', l.bg]}></span>{l.label}</span>
							{/each}
						</div>
						{#each team.members as m (m.sn)}
							<div class="flex items-center gap-3">
								<span class="flex w-24 items-center gap-2">
									{@render who(m, true)}
									<span class="text-xs font-medium">{m.name}</span>
								</span>
								<span class="flex h-3.5 w-60 gap-0.5 rounded-xs bg-muted" aria-hidden="true">
									{#each m.load as s, i (i)}<span class={['w-9.5 rounded-xs', loads[s].bg]}></span>{/each}
								</span>
								<span class="w-14 text-xs whitespace-nowrap text-muted-foreground tabular-nums">작업 {m.load.length}개</span>
								<span
									class={[
										'flex-1 truncate text-xs font-medium',
										m.load.includes('blocked') ? 'text-status-blocked' : m.status === 'waiting' ? 'text-status-waiting' : 'text-muted-foreground'
									]}>{m.loadNote}</span
								>
							</div>
						{/each}
					</CardContent>
				</Card>
			</section>

			<aside class="grid shrink-0 grid-cols-1 items-start gap-5 lg:grid-cols-3 3xl:flex 3xl:w-100 3xl:flex-col 3xl:items-stretch">
				<Card size="sm">
					<CardHeader>
						<CardTitle>모델 연결 사용량</CardTitle>
						<CardAction><Button variant="link" size="xs" href="/settings">모델 연결</Button></CardAction>
					</CardHeader>
					<CardContent>
						{#each accounts as a (a.runtime)}
							{@const users = team.members.filter((m) => m.runtime === a.runtime)}
							{@const warn = low(a.week)}
							<div class={['flex flex-col border gap-2.5 rounded-md p-3', warn && 'border-destructive bg-destructive-soft']}>
								<div class="flex items-center gap-2">
									<RuntimeLogo runtime={a.runtime} class="size-5 ring-0" />
									<span class="flex min-w-0 flex-1 flex-col">
										<span class="text-body font-semibold">{a.name}</span>
										<span class="font-mono text-caption text-muted-foreground">{a.login}</span>
									</span>
									<AvatarGroup class={warn ? '*:data-[slot=avatar]:ring-destructive-soft' : undefined}>
										{#each users as m (m.sn)}<RoleAvatar role={m.role} icon={glyphOf(m)} size="sm" />{/each}
									</AvatarGroup>
								</div>
								{#each [{ label: '5H', pct: a.h5, reset: a.h5Reset }, { label: '주간', pct: a.week, reset: a.weekReset }] as q (q.label)}
									<div class="flex flex-col gap-1">
										<span class="flex gap-1.5 text-caption">
											<span class="flex-1 text-muted-foreground">{q.label}</span>
											<span class={['font-semibold', low(q.pct) && 'text-destructive']}><span class="font-mono">{q.pct}%</span> 남음</span>
											<span class="text-subtle-foreground">· {q.reset} 리셋</span>
										</span>
										<Progress value={q.pct} class="h-1.5 bg-muted" indicator={low(q.pct) ? 'bg-destructive' : 'bg-success'} aria-label="{a.name} {q.label} 잔량" />
									</div>
								{/each}
							</div>
						{/each}
						<div class="flex flex-col gap-0.5 border-t pt-1">
							{#each meters as mt (mt.name)}
								<LimitRow icon={mt.kind === 'key' ? KeyRound : Route} label={mt.name} used="${mt.used}" max="/ ${mt.limit}" value={(mt.used / mt.limit) * 100} note={mt.note} />
							{/each}
						</div>
					</CardContent>
				</Card>

				<Card size="sm">
					<CardHeader>
						<CardTitle>Orch 진행</CardTitle>
						<CardAction><Button variant="link" size="xs" onclick={editPolicy}>정책 편집</Button></CardAction>
					</CardHeader>
					<CardContent>
						{@const ModeIcon = policy.mode === 'manual' ? Hand : policy.mode === 'full' ? Zap : Timer}
						<div class="flex items-center gap-2 rounded-sm bg-primary-soft px-2.5 py-2">
							<ModeIcon class="size-3.5 text-primary" />
							<span class="flex-1 text-xs font-semibold">{modeLabel(policy)}</span>
							{#if policy.guards.find((g) => g.key === 'streak')?.on}<span class="text-caption text-muted-foreground">연속 {policy.streak}/{streakLimit(policy)}</span>{/if}
						</div>
						<div class="flex flex-col">
							{#each policy.levels as l, i (i)}
								<div class="flex h-7 items-center gap-2.5 text-xs">
									<span class={['mono-tag py-px text-on-solid', levelTone[i].bg]}>L{i}</span>
									<span class="flex-1">{l.name}</span>
									<span class={['font-medium', levelTone[i].text]}>{actionLabel(policy, l.action, l.timeout)}</span>
								</div>
							{/each}
						</div>
						<div class="meta-xs gap-1.5 border-t pt-2">
							<ShieldCheck class="size-3.5 shrink-0 text-status-done" />루프 가드 {policy.guards.filter((g) => g.on).length}개 켜짐 · 마지막 정지: {policy.lastStop}
						</div>
						<div class="meta-xs gap-1.5">
							<Layers class="size-3.5 shrink-0" />하위 작업 · 기본 {policy.spawn.mode} · 허용 {policy.spawn.allow.join(' · ')} · 리드당 {policy.spawn.maxChild}
						</div>
					</CardContent>
				</Card>

				<Card size="sm">
					<CardHeader>
						<CardTitle>팀 정책</CardTitle>
						<CardAction><Button variant="link" size="xs" onclick={editTeam}>편집</Button></CardAction>
					</CardHeader>
					<CardContent class="gap-1">
						<div class="value-line">
							<Terminal class="size-3.5 text-muted-foreground" />
							<span class="flex-1 text-muted-foreground">기본 Runtime</span>
							<span class="font-medium">{tp.runtime}</span>
						</div>
						<LimitRow icon={Coins} label="오늘 토큰 예산" used={k(tokens)} max="/ {tp.tokenBudget}K" value={(tokens / tp.tokenBudget) * 100} />
						<LimitRow
							icon={Layers}
							label="Context 경고 · {tp.contextWarn}%"
							used="{hottest.context}%"
							max="/ 100%"
							value={hottest.context}
							note={hot ? `${hottest.name} ${hottest.context}% 초과 · 나머지 ${cool}명 50% 미만` : undefined}
							warn={hot}
							mark={tp.contextWarn}
						/>
						<LimitRow
							icon={Play}
							label="동시 실행"
							used={String(running)}
							max="/ {tp.maxRuns} Run"
							value={(running / tp.maxRuns) * 100}
							note={running < tp.maxRuns ? `${tp.maxRuns - running}개 여유` : '가득 참'}
						/>
						<div class="value-line">
							<ShieldCheck class="size-3.5 text-muted-foreground" />
							<span class="flex-1 text-muted-foreground">Review 필수</span>
							{#if tp.reviewOn}<Pill class="bg-review-soft text-status-review"><GitMerge />{tp.review}</Pill>{:else}<span class="text-muted-foreground">끔</span>{/if}
						</div>
						<div class="value-line">
							<GitBranch class="size-3.5 text-muted-foreground" />
							<span class="flex-1 text-muted-foreground">Repo 권한</span>
							<span class="font-mono font-medium">{tp.repoScope} · {tp.repoPerm}</span>
						</div>
					</CardContent>
				</Card>
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

<Sheet bind:open={adding}>
	<SheetContent side="right" size="xl">
		<SheetHeader>
			<SheetTitle>{team.name}에 멤버 추가</SheetTitle>
			{#snippet sub()}<Steps steps={['템플릿 선택', '캐릭터', '런타임 · 도구']} current={step} />{/snippet}
		</SheetHeader>

		<SheetBody class="flex-row gap-6">
			<div class="flex min-w-0 flex-1 flex-col gap-4.5">
				{#if step === 0}
					<InputGroup class="h-9">
						<InputGroupAddon><Search /></InputGroupAddon>
						<InputGroupInput bind:value={tq} placeholder="템플릿 검색 · 역할, 스킬" aria-label="템플릿 검색" />
					</InputGroup>
					{@const rec = store.templates.find((t) => t.sn === recommend.template)!}
					<button
						type="button"
						onclick={() => (pick = rec.sn)}
						class="flex items-center gap-2 rounded-sm bg-primary-soft px-3 py-2.5 text-left text-xs outline-none focus-visible:ring-3 focus-visible:ring-ring/50"
					>
						<Sparkles class="size-3.5 shrink-0 text-primary" />
						<span><span class="font-semibold">Orch 추천</span> · {recommend.reason} → {rec.name}</span>
					</button>
					<ChoiceCards aria-label="템플릿" class="grid-cols-2" bind:value={() => pick, (v) => (pick = v as typeof pick)}>
						{#each tplShown as t (t.sn)}
							{@const on = pick === t.sn}
							<ChoiceCard value={t.sn} layout="row" ondblclick={toCharacter} class="gap-2.5 bg-card px-2 py-2">
								<RoleAvatar role={t.role} />
								<span class="flex min-w-0 flex-1 flex-col gap-1">
									<span class="text-xs font-semibold">{t.name}</span>
									<span class="text-caption text-muted-foreground">v{t.version} · {t.focus}</span>
									<span class="subtle-meta gap-1.5">
										<RuntimeLogo runtime={t.runtime} class="size-3 ring-0" />
										{runtimeName(t.runtime)} · {t.model} · {t.members ? `멤버 ${t.members}` : '미사용'}
									</span>
								</span>
								{#if on}<CircleCheck class="size-4 shrink-0 text-primary" />{/if}
							</ChoiceCard>
						{:else}
							<p class="col-span-2 py-6 text-center text-xs text-muted-foreground">검색 결과가 없어요.</p>
						{/each}
					</ChoiceCards>
					<!-- 같은 선택(pick)의 한 칸 — 배치는 그대로 두려고 contents -->
					<ChoiceCards aria-label="빈 캐릭터" class="contents" bind:value={() => pick, (v) => (pick = v as typeof pick)}>
					<ChoiceCard value={null} layout="row" class="gap-2.5 p-3.5">
						<span class="flex size-7 items-center justify-center rounded-md border border-dashed text-muted-foreground"><Plus class="size-3.5" /></span>
						<span class="flex flex-1 flex-col gap-0.5">
							<span class="text-xs font-semibold">빈 캐릭터로 시작</span>
							<span class="text-caption text-muted-foreground">템플릿 없이 Instructions를 직접 작성</span>
						</span>
						{#if pick === null}<CircleCheck class="size-4 text-primary" />{/if}
					</ChoiceCard>
					</ChoiceCards>
				{:else}
					<!-- 고른 템플릿 (2 · 3단계 공통) -->
					<div class="flex rounded-md bg-muted items-center gap-3 p-3">
						<RoleAvatar {role} />
						<span class="flex min-w-0 flex-1 flex-col gap-0.5">
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
										class={['rounded-md outline-none focus-visible:ring-3 focus-visible:ring-ring/50', glyph === i ? 'ring-2 ring-primary ring-offset-2 ring-offset-background' : 'opacity-50 hover:opacity-100']}
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
								<Select type="single" bind:value={runtime} onValueChange={(v) => (model = models[v as Runtime][0])}>
									<SelectTrigger class="w-full">
										<span class="flex items-center gap-2"><RuntimeLogo runtime={runtime} class="size-4 ring-0" />{runtimeName(runtime)}</span>
									</SelectTrigger>
									<SelectContent>
										{#each ['claude', 'codex'] as const as r (r)}<SelectItem value={r} label={runtimeName(r)} />{/each}
									</SelectContent>
								</Select>
							</label>
							<div class="flex flex-col gap-1.5">
								<span class="text-caption font-semibold text-muted-foreground">연결</span>
								<!-- 연결은 실행기에 딸려 정해진다 (연결 여러 개는 Settings · #31) -->
								<div class="field-box h-9">
									<RuntimeLogo runtime={runtime} class="size-4 ring-0" />
									{accountOf(runtime).plan}
									<Pill class="text-2xs">구독</Pill>
									<span class={['ml-auto text-caption font-normal', low(accountOf(runtime).week) ? 'text-destructive' : 'text-muted-foreground']}>주간 {accountOf(runtime).week}% 남음</span>
								</div>
							</div>
							<label class="flex flex-col gap-1.5">
								<span class="text-caption font-semibold text-muted-foreground">모델</span>
								<Select type="single" bind:value={model}>
									<SelectTrigger class="w-full"><span class="flex items-center gap-2"><Cpu class="size-3.5 text-muted-foreground" />{model}</span></SelectTrigger>
									<SelectContent>
										{#each models[runtime] as m (m)}<SelectItem value={m} label={m} />{/each}
									</SelectContent>
								</Select>
							</label>
							<label class="flex flex-col gap-1.5">
								<span class="text-caption font-semibold text-muted-foreground">Effort</span>
								<Select type="single" bind:value={effort}>
									<SelectTrigger class="w-full"><span class="flex items-center gap-2"><Gauge class="size-3.5 text-muted-foreground" />{effort}</span></SelectTrigger>
									<SelectContent>
										{#each ['Auto', 'Low', 'Medium', 'High'] as e (e)}<SelectItem value={e} label={e} />{/each}
									</SelectContent>
								</Select>
							</label>
						</div>
						{#if fellBack && tpl}
							<Alert variant="warning" class="flex items-center gap-2.5">
								<Route />
								<div class="flex-1">
									<AlertTitle>폴백 2순위로 시작해요</AlertTitle>
									<AlertDescription>템플릿 기본 {accountOf(tpl.runtime).plan}가 주간 {accountOf(tpl.runtime).week}% 남음 → {accountOf(runtime).plan} ({runtimeName(runtime)})</AlertDescription>
								</div>
							</Alert>
						{/if}
						<div class="flex flex-wrap items-center gap-1.5 text-caption">
							<span class="font-semibold text-muted-foreground">폴백</span>
							{#each chain as a, i (a.runtime)}
								<span class="pill-soft gap-1.5">
									<span class="font-mono text-2xs text-muted-foreground">{i + 1}</span>
									{a.plan}
									<span class={['font-mono text-2xs', low(a.week) ? 'text-status-blocked' : 'text-muted-foreground']}>{a.week}%</span>
								</span>
								<ChevronRight class="size-3 text-subtle-foreground" />
							{/each}
							<span class="pill-soft gap-1.5">
								<span class="font-mono text-2xs text-muted-foreground">3</span>
								OmniRoute
								<span class="font-mono text-2xs text-muted-foreground">${Math.round(gateway.used)}/${gateway.limit}</span>
							</span>
						</div>
					</section>
					{#if tpl}
						<section class="flex flex-col gap-2.5">
							{@render heading('스킬 · 도구', '템플릿에서 복사 · 켜고 끄기는 추가 후 멤버 상세 › Skills · Tools & MCP에서 해요')}
							<div class="flex rounded-md bg-muted flex-col gap-2.5 p-3.5">
								<div class="flex text-xs">
									<span class="flex-1 font-medium">템플릿에서 복사 · {tpl.config.skills.length + tpl.config.mcp.length}개 활성</span>
									<span class="font-mono text-caption text-muted-foreground">+{((tpl.config.skills.length + tpl.config.mcp.length) * 0.24).toFixed(1)}K tok / Run</span>
								</div>
								<div class="flex flex-wrap gap-1.5">
									{#each tpl.config.skills as sk (sk)}<Pill class="bg-card"><Sparkles />{sk}</Pill>{/each}
									{#each tpl.config.mcp as mc (mc)}<Pill class="bg-card"><Plug />{mc}</Pill>{/each}
								</div>
							</div>
							<div class="flex flex-wrap items-center gap-4 rounded-sm bg-muted px-3 py-2.5 text-xs text-muted-foreground">
								<span class="flex items-center gap-1.5"><ShieldCheck class="size-3" />Trust {tpl.config.trust}</span>
								<span class="flex items-center gap-1.5"><FolderGit2 class="size-3" />{scopeText(tpl.config)}</span>
								<span class="flex items-center gap-1.5"><GitPullRequest class="size-3" />PR 승인 필요</span>
							</div>
						</section>
					{/if}
					<section class="flex flex-col gap-2.5">
						{@render heading('첫 작업', '추가 직후 무엇을 할지')}
						<ChoiceCards aria-label="첫 작업" class="grid-cols-3" bind:value={() => first, (v) => (first = v as typeof first)}>
							{#each [{ v: 'orch', icon: Sparkles, t: 'Orch에게 맡기기', d: `팀 진행 정책(${modeLabel(policy)})에 따라 대기열에서 배정` }, { v: 'task', icon: ListChecks, t: '지금 태스크 지정', d: firstLabel.task }, { v: 'wait', icon: Pause, t: '대기', d: '추가만 하고 배정하지 않음' }] as const as o (o.v)}
								{@const on = first === o.v}
								<ChoiceCard value={o.v} class="rounded-md">
									<span class="label-xs-strong">
										<o.icon class={['size-3.5', on ? 'text-primary' : 'text-muted-foreground']} />
										<span class="flex-1">{o.t}</span>
										{#if on}<CircleCheck class="size-3.5 text-primary" />{:else}<Circle class="size-3.5 text-subtle-foreground" />{/if}
									</span>
									<span class="truncate text-caption text-muted-foreground">{o.d}</span>
								</ChoiceCard>
							{/each}
						</ChoiceCards>
					</section>
				{/if}
			</div>

			<!-- 미리보기 -->
			<aside class="flex shrink-0 flex-col w-80 gap-3.5" aria-label="미리보기">
				{#if step === 0}
					{#if tpl}
						{@const soul = tpl.files.find((f) => f.name === 'SOUL.md')}
						<div class="card rounded-lg flex flex-col gap-2.5 p-4">
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
							{#if soul}<pre class="rounded-sm bg-muted p-2.5 font-sans text-caption leading-relaxed whitespace-pre-wrap text-muted-foreground">{soul.body}</pre>{/if}
						</div>
					{:else}
						<div class="empty-note text-muted-foreground">빈 캐릭터는 AGENT.md 한 파일로 시작해요. 런타임은 다음 단계에서 정해요.</div>
					{/if}
				{:else}
					<div class="card rounded-lg flex flex-col gap-2.5 p-4">
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
					<div class="flex gap-2 rounded-md bg-primary-soft p-3 text-xs">
						<Info class="mt-0.5 size-3.5 shrink-0 text-primary" />
						<span>‘팀에 추가’를 누르면 팀 멤버 목록에 들어가요. 어떤 작업을 맡을지는 Orch가 팀 진행 정책에 따라 배정해요.</span>
					</div>
				{/if}
			</aside>
		</SheetBody>

		<SheetFooter
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
					<Button size="sm" disabled={!name.trim() || addSaving} onclick={add}>{#if addSaving}<LoaderCircle class="animate-spin" />{:else}<UserPlus />{/if}팀에 추가</Button>
				{/if}
			{/if}
		</SheetFooter>
	</SheetContent>
</Sheet>

<!-- .pen BarChart — 요일별 막대. b가 있으면 위에 실패색으로 쌓는다 -->
{#snippet chart(cols: [number, number?][], legend: [string, string?], total: string)}
	{@const max = Math.max(1, ...cols.map(([a, b]) => a + (b ?? 0)))}
	<div class="flex flex-col gap-2.5">
		<div class="meta-line gap-3.5">
			<span class="flex items-center gap-1.5"><span class="size-2 rounded-xs bg-primary"></span>{legend[0]}</span>
			{#if legend[1]}<span class="flex items-center gap-1.5"><span class="size-2 rounded-xs bg-status-blocked"></span>{legend[1]}</span>{/if}
			<span class="flex-1"></span>
			<span>{total}</span>
		</div>
		<div class="flex h-25 gap-2 border-b" role="img" aria-label="{legend[0]} {total}">
			{#each cols as [a, b], i (i)}
				<div class="flex flex-1 flex-col items-center justify-end gap-0.5">
					{#if b}<span class="w-4 rounded-t-sm bg-status-blocked" style="height: {(b / max) * 88}px"></span>{/if}
					{#if a}<span class={['w-4 bg-primary', !b && 'rounded-t-sm']} style="height: {(a / max) * 88}px"></span>{/if}
				</div>
			{/each}
		</div>
		<div class="flex gap-2 text-center text-caption text-muted-foreground">
			{#each days as d (d)}<span class="flex-1">{d}</span>{/each}
		</div>
	</div>
{/snippet}

{#snippet mini(label: string, value: string, sub: string, tone?: string)}
	<div class="stat-card">
		<span class="text-xs text-muted-foreground">{label}</span>
		<span class="font-mono text-xl font-semibold">{value}</span>
		<span class={['text-caption', tone ?? 'text-subtle-foreground']}>{sub}</span>
	</div>
{/snippet}

<!-- .pen TaskListRow -->
{#snippet taskRow(num: number, title: string, note: string | undefined, issue: string, meta: string, priority: string, order?: number, state: 'todo' | 'wait' | 'done' = 'todo', noteTone?: string)}
	<div class="queue-row">
		{#if order}<span class="flex size-5 items-center justify-center rounded-full bg-muted font-mono text-caption font-semibold text-muted-foreground">{order}</span>{/if}
		{#if state === 'done'}<CircleCheck class="size-3.5 text-status-done" />{:else if state === 'wait'}<Circle class="size-3.5 text-status-todo" />{:else}<CircleDashed class="size-3.5 text-status-todo" />{/if}
		<span class="font-mono text-xs text-muted-foreground">#{num}</span>
		<span class="flex min-w-0 flex-1 flex-col gap-0.5">
			<span class="truncate text-body font-medium">{title}</span>
			{#if note}<span class={['truncate text-caption', noteTone ?? 'text-muted-foreground']}>{note}</span>{/if}
		</span>
		<span class="font-mono text-caption text-subtle-foreground">{issue}</span>
		<span class="num-cell w-28 whitespace-nowrap">{meta}</span>
		<span class="w-5 text-caption font-semibold text-muted-foreground">{priority}</span>
	</div>
{/snippet}

<Sheet bind:open={() => viewing !== undefined, (v) => !v && (viewing = undefined)}>
	<SheetContent side="right" size="xl">
		{#if viewed}
			{@const m = viewed}
			{@const st = states[m.status]}
			{@const isPaused = paused.includes(m.sn)}
			<SheetHeader>
				{#snippet lead()}<RoleAvatar role={m.role} icon={glyphOf(m)} size="lg" />{/snippet}
					<div class="flex items-center gap-2.5">
						<SheetTitle class="text-xl font-bold">{m.name}</SheetTitle>
						{#if isPaused}
							<Pill class="rounded-sm bg-warning-soft px-2 text-warning"><Pause />Paused</Pill>
						{:else}
							<Pill class={['rounded-sm px-2', m.status === 'running' ? 'bg-primary-soft text-status-in-progress' : m.status === 'waiting' ? 'bg-warning-soft text-status-waiting' : '']}>
								<st.icon />{st.label}{#if m.status === 'running' && det?.now}{` · #${det.now.task}`}{/if}
							</Pill>
						{/if}
					</div>
					<div class="meta-xs gap-2">
						<span>{m.title} · {team.name}</span>
						{#if origin}
							<span class="text-subtle-foreground">·</span>
							<span class="chip-box gap-1.5 py-0.5">
								<LayoutTemplate class="size-3" />원본 {origin.name} v{origin.version}{#if det?.diff.length}{` · 변경 ${det.diff.length}`}{/if}
							</span>
						{/if}
					</div>
				{#snippet actions()}
					<Button variant="outline" size="sm" onclick={() => togglePause(m.sn)}>
						{#if isPaused}<Play />Resume{:else}<Pause />Pause{/if}
					</Button>
				{/snippet}
			</SheetHeader>

			<SheetBody padded={false} class="flex-row">
				<nav aria-label="멤버 메뉴" class="side-nav w-53 px-2.5 py-3.5">
					{#each nav as g, gi (g.group)}
						<span class={['list-label px-2 pb-1.5', gi > 0 ? 'mt-1.5 border-t pt-3.5' : 'pt-0.5']}>{g.group}</span>
						{#each g.items as it (it.v)}
							<button
								type="button"
								aria-current={tab === it.v ? 'page' : undefined}
								onclick={() => (tab = it.v)}
								class={['side-nav-item h-8.5', tab === it.v ? 'bg-accent font-semibold text-foreground' : 'text-muted-foreground']}
							>
								<it.icon class="size-4" />
								<span class="flex-1 text-left">{it.label}</span>
								{#if 'changed' in it && it.changed}<span class="size-1.5 rounded-full bg-primary" aria-label="템플릿과 다름"></span>{/if}
							</button>
						{/each}
					{/each}
					<span class="subtle-meta gap-1.5 px-2 pt-3.5"><span class="size-1.5 rounded-full bg-primary"></span>템플릿과 다른 항목</span>
				</nav>

				<div class="flex min-w-0 flex-1 flex-col gap-4 overflow-y-auto bg-muted px-6 py-5 *:shrink-0">
					{#if !det && ['overview', 'tasks', 'runs', 'activity', 'usage'].includes(tab)}
						<Empty class="bg-card">
							<EmptyHeader>
								<EmptyMedia variant="icon"><Activity /></EmptyMedia>
								<EmptyTitle>아직 기록이 없어요</EmptyTitle>
								<EmptyDescription>{m.name}의 태스크 · Run · 활동이 쌓이면 여기에 보여요. 지금: {m.work}</EmptyDescription>
							</EmptyHeader>
						</Empty>
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
						<Card size="sm">
							<CardHeader>
								<CardTitle>이번 주 토큰 사용</CardTitle>
								<CardAction><Button variant="link" size="xs" onclick={() => (tab = 'runs')}>Runs 보기</Button></CardAction>
							</CardHeader>
							<CardContent>{@render chart(det.week.days.map((d) => [d]), ['토큰 (K)'], `이번 주 · ${det.week.tokens}K · 하루 평균 ${Math.round(det.week.tokens / 7)}K`)}</CardContent>
						</Card>
						{#if det.now && now}
							<Card size="sm">
								<CardHeader><CardTitle>지금 하는 일</CardTitle></CardHeader>
								<CardContent class="gap-2.5">
									<div class="flex items-center gap-2 text-body">
										<SquareCheck class="size-3.5 text-node-task" />
										<span class="flex-1 font-medium">#{now.num} · {now.title}</span>
										<span class="font-mono text-xs font-semibold">{det.now.pct}%</span>
									</div>
									<Progress value={det.now.pct} class="h-1.5" aria-label="#{now.num} 진행" />
									<div class="flex flex-wrap gap-4 text-xs text-muted-foreground">
										<span class="flex items-center gap-1.5"><LoaderCircle class="size-3" />Run #{det.now.run} · {det.now.elapsed}</span>
										<span class="flex items-center gap-1.5"><GitBranch class="size-3" />{det.now.branch} · {det.now.commits} commits</span>
										<span class="flex items-center gap-1.5"><Timer class="size-3" />ETA {det.now.eta}</span>
									</div>
								</CardContent>
							</Card>
						{/if}
						<Card size="sm">
							<CardHeader><CardTitle>주의 필요 · {det.attention.length}</CardTitle></CardHeader>
							<CardContent class="gap-0">
								{#each det.attention as at, i (i)}
									{@const Icon = at.kind === 'quota' ? TriangleAlert : at.kind === 'wait' ? Link2 : Undo2}
									<div class={['flex items-center gap-3 py-2.5', i > 0 && 'border-t']}>
										<Icon class={['size-4 shrink-0', at.kind === 'quota' ? 'text-destructive' : at.kind === 'wait' ? 'text-status-waiting' : 'text-status-review']} />
										<span class="flex min-w-0 flex-1 flex-col gap-0.5">
											<span class="text-body font-medium">{at.title}</span>
											<span class="text-xs text-muted-foreground">{at.sub}</span>
										</span>
									</div>
								{/each}
							</CardContent>
						</Card>
						<Card size="sm">
							<CardHeader>
								<CardTitle>다음 실행 대기열</CardTitle>
								<CardAction><Button variant="link" size="xs" onclick={() => (tab = 'tasks')}>Tasks 탭</Button></CardAction>
							</CardHeader>
							<CardContent class="gap-0">
								{#each det.queue as q, i (q.num)}
									{@render taskRow(q.num, q.title, q.note, q.issue, q.est, q.priority, i + 1, q.blocked ? 'wait' : 'todo', q.blocked ? 'text-status-waiting' : undefined)}
								{/each}
							</CardContent>
						</Card>
						<Card size="sm">
							<CardHeader><CardTitle>템플릿 대비 변경 · {det.diff.length}</CardTitle></CardHeader>
							<CardContent class="gap-2">
								{#each det.diff as d (d.file)}
									<div class="flex gap-2.5 text-xs">
										{#if d.file === 'SOUL.md'}<BookOpen class="mt-0.5 size-3.5 text-muted-foreground" />{:else}<FileText class="mt-0.5 size-3.5 text-muted-foreground" />{/if}
										<span class="flex flex-col gap-0.5"><span class="font-mono font-medium">{d.file}</span><span class="text-muted-foreground">{d.note}</span></span>
									</div>
								{/each}
								<span class="meta-line gap-1.5 border-t pt-2"><GitCompare class="size-3" />템플릿이 새 버전으로 바뀌면 여기서 골라서 가져올 수 있어요 (자동 반영 없음)</span>
							</CardContent>
						</Card>
					{:else if det && tab === 'tasks'}
						{@const active = (det.now ? 1 : 0) + det.queue.length}
						<div class="flex items-center gap-2">
							{#each [['all', '전체', active + det.done.length], ['active', '활성', active], ['done', '완료', det.done.length]] as const as [v, l, n] (v)}
								<Toggle variant="chip" count={n} pressed={taskFilter === v} onPressedChange={() => (taskFilter = v)}>{l}</Toggle>
							{/each}
							<span class="flex-1"></span>
							<InputGroup class="h-8 w-56 bg-card">
								<InputGroupAddon><Search /></InputGroupAddon>
								<InputGroupInput bind:value={taskQuery} placeholder="태스크 검색" aria-label="태스크 검색" />
							</InputGroup>
						</div>
						{#if taskFilter !== 'done'}
							{#if det.now && now && hit(now.title)}
								<Card size="sm">
									<CardHeader><CardTitle>진행 중 · 1</CardTitle></CardHeader>
									<CardContent class="gap-0">
										<div class="queue-row">
											<CircleDot class="size-3.5 text-status-in-progress" />
											<span class="font-mono text-xs text-muted-foreground">#{now.num}</span>
											<span class="flex min-w-0 flex-1 flex-col gap-0.5">
												<span class="text-body font-medium">{now.title}</span>
												<span class="text-caption text-status-in-progress">Run #{det.now.run} · {det.now.pct}%</span>
											</span>
											<span class="font-mono text-caption text-subtle-foreground">{issuePath(now.issue)}</span>
											<span class="num-cell w-28 whitespace-nowrap">{det.now.eta} 남음</span>
											<span class="w-5 text-caption font-semibold text-muted-foreground">{now.priority}</span>
										</div>
									</CardContent>
								</Card>
							{/if}
							<Card size="sm">
								<CardHeader>
									<CardTitle>실행 대기열 · {det.queue.length}</CardTitle>
									<CardDescription>위에서부터 순서대로 실행돼요.</CardDescription>
								</CardHeader>
								<CardContent class="gap-0">
									{#each det.queue.filter((q) => hit(q.title)) as q, i (q.num)}
										{@render taskRow(q.num, q.title, q.note, q.issue, q.est, q.priority, i + 1, q.blocked ? 'wait' : 'todo', q.blocked ? 'text-status-waiting' : undefined)}
									{:else}
										<p class="border-t py-3 text-xs text-muted-foreground">대기 중인 태스크가 없어요.</p>
									{/each}
								</CardContent>
							</Card>
							<Card size="sm">
								<CardHeader><CardTitle>막힘 · {blocked.length}</CardTitle></CardHeader>
								<CardContent class="gap-0">
									{#each blocked as b (b.num)}
										{@render taskRow(b.num, b.title, '막힘', issuePath(b.issue), b.updated, b.priority, undefined, 'wait', 'text-status-blocked')}
									{:else}
										<p class="text-xs text-muted-foreground">막힌 태스크가 없어요.</p>
									{/each}
								</CardContent>
							</Card>
						{/if}
						{#if taskFilter !== 'active'}
							<Card size="sm">
								<CardHeader><CardTitle>최근 완료 · 7일</CardTitle></CardHeader>
								<CardContent class="gap-0">
									{#each det.done.filter((d) => hit(d.title)) as d (d.num)}
										{@render taskRow(d.num, d.title, undefined, d.tokens, d.when, '', undefined, 'done')}
									{:else}
										<p class="border-t py-3 text-xs text-muted-foreground">검색 결과가 없어요.</p>
									{/each}
								</CardContent>
							</Card>
						{/if}
						<div class="grid grid-cols-2 gap-3.5">
							<Card size="sm">
								<CardHeader><CardTitle>대기열 규칙</CardTitle></CardHeader>
								<CardContent class="gap-0">
									{#each [[ArrowDownWideNarrow, '정렬 기준', det.rules.sort], [Play, '자동 시작', det.rules.autostart ? '켜짐' : '꺼짐'], [Shuffle, 'Orch 재배치', det.rules.rebalance ? '허용' : '안 함'], [Layers, '동시 실행', det.rules.runs]] as const as [Icon, l, v] (l)}
										<div class="value-line">
											<Icon class="size-3.5 text-muted-foreground" />
											<span class="flex-1 text-muted-foreground">{l}</span>
											<span class="font-medium">{v}</span>
										</div>
									{/each}
								</CardContent>
							</Card>
							<Card size="sm">
								<CardHeader><CardTitle>배정 출처 · 30일</CardTitle></CardHeader>
								<CardContent class="gap-2.5">
									{#each [['Orch 자동 배정', det.source.orch], ['사용자 수동', det.source.manual]] as const as [l, n] (l)}
										<div class="flex flex-col gap-1">
											<span class="flex text-xs"><span class="flex-1 text-muted-foreground">{l}</span><span class="font-mono font-medium">{n}</span></span>
											<Progress value={(n / (det.source.orch + det.source.manual)) * 100} class="h-1.5 bg-muted" aria-label={l} />
										</div>
									{/each}
								</CardContent>
							</Card>
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
						<Card size="sm">
							<CardHeader><CardTitle>최근 7일 Run</CardTitle></CardHeader>
							<CardContent>{@render chart(det.runDays, ['성공', '실패'], `7일 · ${det.runs.length} runs · 성공 ${ok} · 실패 ${fail}`)}</CardContent>
						</Card>
						<div class="card overflow-hidden rounded-lg">
							<Table class="table-fixed">
								<TableHeader>
									<TableRow class="hover:bg-transparent">
										<TableHead class="w-10 pl-3.5"><span class="sr-only">상태</span></TableHead>
										<TableHead class="w-15">Run</TableHead>
										<TableHead>Task</TableHead>
										<TableHead class="w-24">시작</TableHead>
										<TableHead class="w-16">소요</TableHead>
										<TableHead class="w-16">토큰</TableHead>
										<TableHead class="w-40 pr-3.5">결과</TableHead>
									</TableRow>
								</TableHeader>
								<TableBody>
									{#each det.runs as r (r.num)}
										{@const ri = runIcon[r.status]}
										<TableRow
											class={['h-11 text-xs', det.failure?.run === r.num && 'cursor-pointer', runSel === r.num && 'bg-primary-soft hover:bg-primary-soft']}
											onclick={() => det.failure?.run === r.num && (runSel = runSel === r.num ? undefined : r.num)}
										>
											<TableCell class="pl-3.5"><ri.icon class={['size-3.5', ri.text]} aria-label={r.status} /></TableCell>
											<TableCell class="font-mono font-semibold text-muted-foreground">#{r.num}</TableCell>
											<TableCell class="truncate">{r.task}</TableCell>
											<TableCell class="font-mono text-muted-foreground">{r.start}</TableCell>
											<TableCell class="font-mono text-muted-foreground">{r.dur}</TableCell>
											<TableCell class="font-mono text-muted-foreground">{r.tokens.toFixed(1)}K</TableCell>
											<TableCell class={['truncate pr-3.5', r.status === 'failed' ? 'text-status-blocked' : 'text-muted-foreground']}>{r.result}</TableCell>
										</TableRow>
									{/each}
								</TableBody>
							</Table>
						</div>
						{#if sel}
							<section class="flex flex-col gap-3.5" aria-label="Run #{sel.run} 상세">
								<div class="flex items-center gap-2">
									<CircleX class="size-4 text-status-blocked" />
									<h3 class="text-sm font-semibold">Run #{sel.run} · Failed</h3>
									<span class="text-xs text-muted-foreground">{sel.meta}</span>
								</div>
								<div class="grid grid-cols-2 gap-3.5">
									<Card size="sm">
										<CardHeader><CardTitle>실행 단계</CardTitle></CardHeader>
										<CardContent class="gap-1.5">
											{#each sel.steps as step (step.name)}
												<div class="flex items-center gap-2 text-xs">
													{#if step.state === 'done'}<CircleCheck class="size-3.5 text-status-done" />{:else if step.state === 'failed'}<CircleX class="size-3.5 text-status-blocked" />{:else}<CircleDashed class="size-3.5 text-subtle-foreground" />{/if}
													<span class={['flex-1', step.state === 'failed' && 'font-medium text-status-blocked', step.state === 'skipped' && 'text-muted-foreground']}>{step.name}</span>
													<span class="font-mono text-caption text-muted-foreground">{step.time}</span>
												</div>
											{/each}
										</CardContent>
									</Card>
									<Card size="sm">
										<CardHeader><CardTitle>실패 원인</CardTitle></CardHeader>
										<CardContent class="gap-2">
											<pre class="rounded-sm bg-destructive-soft p-2.5 font-mono text-caption leading-relaxed whitespace-pre-wrap text-status-blocked">{sel.errors.join('\n')}</pre>
											<span class="meta-xs gap-1.5"><Redo2 class="size-3.5 text-primary" />{sel.retry}</span>
										</CardContent>
									</Card>
								</div>
								<Card size="sm">
									<CardHeader><CardTitle>변경 파일 · {sel.files.length}</CardTitle></CardHeader>
									<CardContent class="gap-1.5">
										{#each sel.files as f (f.path)}
											<div class="flex items-center gap-2.5 font-mono text-xs">
												<span class={['w-3 font-semibold', f.kind === 'A' ? 'text-status-done' : 'text-status-waiting']}>{f.kind}</span>
												<span class="flex-1 truncate">{f.path}</span>
												<span class="text-muted-foreground">{f.diff}</span>
											</div>
										{/each}
									</CardContent>
								</Card>
							</section>
						{/if}
					{:else if det && tab === 'activity'}
						<div class="flex flex-wrap items-center gap-2">
							<Toggle variant="chip" pressed={actFilter === 'all'} onPressedChange={() => (actFilter = 'all')}>전체</Toggle>
							{#each actKinds as { k, n } (k)}
								<Toggle variant="chip" count={n} pressed={actFilter === k} onPressedChange={() => (actFilter = k)}>{k}</Toggle>
							{/each}
						</div>
						<div class="card rounded-lg flex flex-col gap-2 px-4 py-3">
							{#each det.activity as d (d.day)}
								{@const items = d.items.filter((a) => actFilter === 'all' || actKind(a) === actFilter)}
								{#if items.length}
									<span class="list-label pt-1">{d.day}</span>
									{#each items as a, i (i)}
										{#if a.type === 'decision'}
											<!-- 결정 기록 (.pen Decision Record) — 한 줄, 누르면 카드. 재검토 · 답변 전문은 프로젝트 판단 패널로 -->
											{@const r = a.record}
											<DecisionRecord record={r} onreview={() => goto(`/p/${store.projects.find((p) => p.name === team.project)?.sn ?? 1}?decide=${r.task}`)} />
										{:else}
											{@const KindIcon = kindIcon[a.kind]}
											<div class="row-divided gap-2.5 py-2">
												<RoleAvatar role={a.role} />
												<div class="flex min-w-0 flex-1 flex-col gap-1">
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
						<Card size="sm">
							<CardHeader><CardTitle>협업 관계 · 7일</CardTitle></CardHeader>
							<CardContent class="gap-2.5">
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
							</CardContent>
						</Card>
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
						<Card size="sm">
							<CardHeader><CardTitle>이번 주 토큰 사용</CardTitle></CardHeader>
							<CardContent>{@render chart(det.week.days.map((d) => [d]), ['토큰 (K)'], `이번 주 · ${det.week.tokens}K · 하루 평균 ${Math.round(det.week.tokens / 7)}K`)}</CardContent>
						</Card>
						<div class="grid grid-cols-2 gap-3.5">
							{#each [acc, ...accounts.filter((a) => a !== acc)] as a (a.runtime)}
								{@const warn = low(a.week)}
								<div class={['card flex flex-col gap-2.5 p-3 rounded-md', warn && 'border-destructive bg-destructive-soft']}>
									<div class="flex items-center gap-2">
										<RuntimeLogo runtime={a.runtime} class="size-5 ring-0" />
										<span class="flex flex-1 flex-col"><span class="text-body font-semibold">{a.name}</span><span class="font-mono text-caption text-muted-foreground">{a.login}</span></span>
										{#if a.runtime === m.runtime}<Pill class="text-2xs">{m.name}</Pill>{/if}
									</div>
									{#each [{ label: '5H', pct: a.h5, reset: a.h5Reset }, { label: '주간', pct: a.week, reset: a.weekReset }] as q (q.label)}
										<div class="flex flex-col gap-1">
											<span class="flex gap-1.5 text-caption">
												<span class="flex-1 text-muted-foreground">{q.label}</span>
												<span class={['font-semibold', low(q.pct) && 'text-destructive']}><span class="font-mono">{q.pct}%</span> 남음</span>
												<span class="text-subtle-foreground">· {q.reset} 리셋</span>
											</span>
											<Progress value={q.pct} class="h-1.5 bg-muted" indicator={low(q.pct) ? 'bg-destructive' : 'bg-success'} aria-label="{a.name} {q.label} 잔량" />
										</div>
									{/each}
								</div>
							{/each}
						</div>
						<Card size="sm">
							<CardHeader>
								<CardTitle>Run별 토큰</CardTitle>
								<CardDescription>이번 주 {det.runs.length} Run · 합계 {det.runs.reduce((s, r) => s + r.tokens, 0).toFixed(1)}K</CardDescription>
							</CardHeader>
							<CardContent class="gap-0">
								{#each [...det.runs].sort((a, b) => b.tokens - a.tokens) as r (r.num)}
									<KeyValueRow label={`#${r.num} · ${r.task} · ${r.start}${r.status === 'running' ? ' (진행 중)' : r.status === 'failed' ? ' (실패)' : r.status === 'cancelled' ? ' (취소)' : ''}`} value={`${r.tokens.toFixed(1)}K`} />
								{/each}
							</CardContent>
						</Card>
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
						<div class="card flex items-center gap-2 px-3.5 py-2.5 rounded-md">
							<span class="flex-1 text-xs text-muted-foreground">
								{#if proposed}<span class="text-primary">{proposed}을 템플릿 초안에 제안했어요 · </span>{/if}
								{memberChanged.length ? `템플릿과 다른 파일 ${memberChanged.length}개 · 이 멤버에게만 적용 중` : '템플릿과 같아요'}
							</span>
							<Button variant="ghost" size="sm" disabled={!memberChanged.some((f) => f.name === memberFiles[memberFile]?.name)} onclick={revertFile}><Undo2 />템플릿으로 되돌리기</Button>
							<Button size="sm" disabled={!memberChanged.some((f) => f.name === memberFiles[memberFile]?.name)} onclick={propose}><Upload />템플릿에 제안</Button>
						</div>
					{:else}
						<Empty class="bg-card">
							<EmptyHeader>
								<EmptyMedia variant="icon"><SlidersHorizontal /></EmptyMedia>
								<EmptyTitle>{nav.flatMap((g) => g.items).find((i) => i.v === tab)?.label}</EmptyTitle>
								<EmptyDescription>템플릿 상세(#65)와 같은 설정 화면이라 함께 만들어요. (#63 T-3b)</EmptyDescription>
							</EmptyHeader>
						</Empty>
					{/if}
				</div>
			</SheetBody>
		{/if}
	</SheetContent>
</Sheet>

<!-- 팀 설정 (.pen Teams · 팀 설정) -->
<Dialog bind:open={() => teamEdit !== undefined, (v) => !v && (teamEdit = undefined)}>
	<DialogContent size="md" tall>
		{#if teamEdit}
			{@const e = teamEdit}
			<DialogHeader icon={Users}>
				<DialogTitle>팀 설정 · {team.name}</DialogTitle>
				<DialogDescription>이름 · 실행 한도 · 리뷰 · 저장소. Orch 진행 방식은 ‘정책 편집’에서 정해요.</DialogDescription>
			</DialogHeader>
			<DialogBody class="gap-6">
				<section class="flex flex-col">
					{@render heading('1 · 기본 정보', '')}
					<FieldRow label="이름" as="label" error={e.name.trim() ? undefined : '이름을 입력하세요'}><Input bind:value={e.name} /></FieldRow>
					<FieldRow label="설명" hint="팀 목록 · 헤더에 보여요" as="label"><Input bind:value={e.desc} /></FieldRow>
					<FieldRow label="연결된 프로젝트" hint="프로젝트 화면에서 바꿔요">
						{#if team.project}<Pill class="w-fit">{team.project}</Pill>{:else}<span class="text-xs text-muted-foreground">없음</span>{/if}
					</FieldRow>
				</section>
				<section class="flex flex-col">
					{@render heading('2 · 실행 한도', '넘으면 Orch가 다음 태스크를 기다리게 해요')}
					<FieldRow label="오늘 토큰 예산" hint="팀 전체 · 0시에 초기화 · K 단위" as="label"><Input type="number" min="1" bind:value={e.tokenBudget} /></FieldRow>
					<FieldRow label="Context 경고" hint="넘으면 요약 · 새 세션 제안">
						<Segmented aria-label="Context 경고" options={[70, 80, 90].map((v) => ({ value: String(v), label: `${v}%` }))} bind:value={() => String(e.contextWarn), (v) => (e.contextWarn = Number(v))} />
					</FieldRow>
					<FieldRow label="동시 실행" hint="팀 전체 Run 수">
						<Segmented aria-label="동시 실행" options={[1, 2, 3, 4].map((v) => ({ value: String(v), label: `${v} Run` }))} bind:value={() => String(e.maxRuns), (v) => (e.maxRuns = Number(v))} />
					</FieldRow>
				</section>
				<section class="flex flex-col">
					{@render heading('3 · 리뷰 · 저장소', '에이전트가 만든 변경을 누가 · 언제 확인하고, 어디까지 쓸 수 있는지')}
					<FieldRow label="Review 필수" hint="켜면 리뷰어 통과 전 병합 · 완료 안 됨">
						<span class="flex items-center gap-3">
							<Switch bind:checked={e.reviewOn} aria-label="Review 필수" />
							{#if e.reviewOn}<Segmented aria-label="리뷰 단계" options={['PR 병합 전', '완료 처리 전'].map((v) => ({ value: v, label: v }))} bind:value={() => e.review, (v) => (e.review = v ?? e.review)} />{/if}
						</span>
					</FieldRow>
					<FieldRow label="Repo 범위" hint="이 팀이 손댈 수 있는 저장소" as="label"><Input class="font-mono" bind:value={e.repoScope} /></FieldRow>
					<FieldRow label="Repo 권한" hint="read · 브랜치 만들기 · push까지">
						<Segmented aria-label="Repo 권한" options={(Object.keys(repoPerms) as RepoPerm[]).map((v) => ({ value: v, label: repoPerms[v] }))} bind:value={() => e.repoPerm, (v) => (e.repoPerm = (v ?? e.repoPerm) as RepoPerm)} />
					</FieldRow>
				</section>
				<section class="flex flex-col gap-3">
					{@render heading('4 · 위험 구역', '')}
					<div class="flex flex-col rounded-md border border-destructive">
						<div class="flex items-center gap-3 border-b px-3.5 py-3">
							<span class="flex flex-1 flex-col gap-0.5"><span class="text-xs font-semibold">팀 보관</span><span class="text-caption text-muted-foreground">멤버와 기록은 남고, 새 배정 · 실행이 멈춰요.{useMock ? '' : ' 서버 보관 API 전에는 쓸 수 없어요.'}</span></span>
							<Button variant="outline" size="sm" disabled={!useMock} onclick={() => removeTeam('archive')}><Archive />보관</Button>
						</div>
						<div class="flex items-center gap-3 px-3.5 py-3">
							<span class="flex flex-1 flex-col gap-0.5"><span class="text-xs font-semibold">팀 삭제</span><span class="text-caption text-muted-foreground">멤버 프로필도 지워져요. Run 기록이 있는 멤버가 있으면 삭제 대신 보관만 돼요.</span></span>
							<Button variant="destructive" size="sm" onclick={() => removeTeam('delete')}><Trash2 />{confirmDelete ? '한 번 더 누르면 삭제' : '팀 삭제'}</Button>
						</div>
					</div>
				</section>
			</DialogBody>
			<DialogFooter note="저장하면 다음 Run부터 적용돼요 · 진행 중 Run은 그대로">
				<Button variant="ghost" size="sm" onclick={() => (teamEdit = undefined)}>취소</Button>
				<Button size="sm" disabled={!e.name.trim() || teamSaving} onclick={saveTeam}>{#if teamSaving}<LoaderCircle class="animate-spin" />{/if}저장</Button>
			</DialogFooter>
		{/if}
	</DialogContent>
</Dialog>

<!-- Orch 진행 정책 편집 (.pen Team Settings · Orch 진행) -->
<Dialog bind:open={() => draft !== undefined, (v) => !v && (draft = undefined)}>
	<DialogContent size="full" tall>
		{#if draft}
			{@const d = draft}
			<DialogHeader crumb={[team.name, '팀 설정']} closable={false}>
				<DialogTitle>Orch 진행 정책</DialogTitle>
				<DialogDescription>다음 작업 배정 · 분배 · 사용자 판단이 필요한 순간을 Orch가 어떻게 처리할지 정해요.</DialogDescription>
				{#snippet actions()}
					<Button variant="ghost" size="sm" onclick={() => (draft = undefined)}>취소</Button>
					<Button size="sm" disabled={!!spawnError || !!childError || policySaving} onclick={savePolicy}>{#if policySaving}<LoaderCircle class="animate-spin" />{/if}저장</Button>
				{/snippet}
			</DialogHeader>
			<DialogBody class="gap-7">
				<section class="flex flex-col gap-3">
					{@render heading('1 · 진행 모드', '태스크가 끝났을 때 Orch가 다음 행동을 제안하고 실행하는 방식')}
					<div role="radiogroup" aria-label="진행 모드" class="grid grid-cols-3 gap-3">
						{#each [{ v: 'manual', icon: Hand, t: 'Manual', d: '매번 사용자가 확인해야 진행' }, { v: 'timer', icon: Timer, t: 'Auto · 타이머', d: '제안 후 카운트다운 · 개입 없으면 진행' }, { v: 'full', icon: Zap, t: 'Full auto', d: '대기 없이 바로 진행 (가드는 항상 적용)' }] as const as o (o.v)}
							{@const on = d.mode === o.v}
							<div class={['flex flex-col border gap-2 rounded-md p-3.5', on && 'option-on']}>
								<button type="button" role="radio" aria-checked={on} onclick={() => (d.mode = o.v)} class="flex flex-col gap-2 text-left outline-none focus-visible:underline">
									<span class="row-title-strong">
										<o.icon class={['size-4', on ? 'text-primary' : 'text-muted-foreground']} />
										<span class="flex-1">{o.t}</span>
										{#if on}<CircleCheck class="size-4 text-primary" />{:else}<Circle class="size-4 text-subtle-foreground" />{/if}
									</span>
									<span class="text-xs text-muted-foreground">{o.d}</span>
								</button>
								{#if o.v === 'timer'}
									<div class="flex flex-wrap items-center gap-1 pt-1 text-xs">
										<span class="mr-1 text-muted-foreground">대기</span>
										{#each [3, 5, 10, 30] as sec (sec)}
											{@const sel = on && !customTimer && d.timer === sec}
											<button
												type="button"
												aria-pressed={sel}
												onclick={() => ((d.mode = 'timer'), (d.timer = sec), (customTimer = false))}
												class={['rounded-xs border px-2 py-1 font-medium outline-none focus-visible:ring-3 focus-visible:ring-ring/50', sel ? 'border-primary bg-primary text-on-solid' : 'bg-card']}
											>{sec}초</button>
										{/each}
										{#if customTimer}
											<Input type="number" min={1} max={600} bind:value={d.timer} aria-label="대기 초" class="h-6.5 w-16 px-2 text-xs" />
										{:else}
											<button type="button" onclick={() => ((d.mode = 'timer'), (customTimer = true))} class="rounded-xs border bg-card px-2 py-1 font-medium outline-none focus-visible:ring-3 focus-visible:ring-ring/50">직접</button>
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
						<div class="flex bg-muted text-caption font-medium text-muted-foreground gap-4 px-4 py-2">
							<span class="w-58">레벨</span><span class="flex-1">예시</span><span class="w-75">처리</span><span class="w-42">응답 없으면</span>
						</div>
						{#each d.levels as l, i (i)}
							<div class="flex items-center border-t px-4 gap-4 py-3">
								<span class="flex w-58 items-center gap-2.5">
									<span class={['mono-tag py-px text-on-solid', levelTone[i].bg]}>L{i}</span>
									<span class="text-body font-semibold">{l.name}</span>
								</span>
								<span class="flex-1 text-xs text-muted-foreground">{l.example}</span>
								<Segmented class="w-75" options={actions} bind:value={() => l.action, (v) => (l.action = v as LevelAction)} disabled={l.locked} aria-label="L{i} 처리" />
								<span class="w-42 text-xs">
									{#if l.locked}
										<span class="text-subtle-foreground">변경 불가</span>
									{:else if l.action === 'timer'}
										<span>{d.mode === 'timer' ? `${d.timer}초 후 진행` : '모드 타이머 사용'}</span>
									{:else if l.action === 'wait'}
										<Select type="single" bind:value={l.timeout}>
											<SelectTrigger size="sm" class="h-7 w-full">{l.timeout ? `${l.timeout} 후 Orch가 판단` : '계속 대기'}</SelectTrigger>
											<SelectContent>
												{#each waitOptions as o (o)}<SelectItem value={o} label={o ? `${o} 후 Orch가 판단` : '계속 대기'} />{/each}
											</SelectContent>
										</Select>
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
							<label class="flex items-center rounded-md border gap-2.5 p-3">
								<Icon class="size-4 shrink-0 text-muted-foreground" />
								<span class="flex min-w-0 flex-1 flex-col gap-0.5">
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
						<div class="flex flex-col border gap-2.5 rounded-md p-3.5">
							<span class="text-caption font-semibold text-muted-foreground">기본 방식</span>
							<Segmented
								aria-label="기본 방식"
								class={spawnError ? 'ring-1 ring-destructive' : undefined}
								options={spawnModes.map((m) => ({ value: m.value, label: m.value, icon: m.icon, disabled: !d.spawn.allow.includes(m.value), hint: '허용 방식에서 꺼져 있어요' }))}
								bind:value={() => d.spawn.mode, (v) => (d.spawn.mode = v as SpawnMode)}
							/>
							{#if spawnError || spawnServerError}
								<span class="error-note"><CircleAlert class="size-3 shrink-0" />{spawnError || spawnServerError}</span>
							{:else}
								<span class="text-caption text-muted-foreground">runner · 별도 Run으로 돌려요. 모델 등급(S/M/L)은 작업 종류로 정해지고, 토큰은 따로 집계돼요.</span>
							{/if}
						</div>
						<div class="flex flex-col border gap-2 rounded-md p-3.5">
							<span class="text-caption font-semibold text-muted-foreground">허용 방식</span>
							{#each spawnDesc as [m, desc] (m)}
								<label class="flex items-center gap-2 text-xs"><Checkbox checked={d.spawn.allow.includes(m)} onCheckedChange={(v) => toggleAllow(d, m, !!v)} aria-label="{m} 허용" />{m} · {desc}</label>
							{/each}
						</div>
						<div class="flex flex-col border gap-2.5 rounded-md p-3.5">
							<span class="text-caption font-semibold text-muted-foreground">리드당 동시 하위 작업</span>
							<InputGroup class={['w-28', childError && 'border-destructive']}>
								<InputGroupAddon><Layers /></InputGroupAddon>
								<InputGroupInput type="number" min="1" bind:value={d.spawn.maxChild} aria-label="리드당 동시 하위 작업" class="font-mono" />
							</InputGroup>
							{#if childError}
								<span class="error-note"><CircleAlert class="size-3 shrink-0" />{childError}</span>
							{/if}
							<span class="text-caption text-muted-foreground">최소 1 · paths가 겹치는 하위 작업은 규칙 엔진이 순서대로 돌려요 (queued · waits).</span>
						</div>
					</div>
					{#if d.spawn.allow.includes('fork')}
						<Alert variant="warning">
							<GitFork />
							<AlertTitle>fork는 부모 컨텍스트를 그대로 상속해요</AlertTitle>
							<AlertDescription>켜면 부모 컨텍스트를 그대로 복사해 비용이 커요.</AlertDescription>
						</Alert>
					{/if}
				</section>
			</DialogBody>
		{/if}
	</DialogContent>
</Dialog>
