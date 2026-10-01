<script lang="ts">
	/// 첫 실행 (.pen ⓪ 인트로) — 스플래시 → 1 에이전트 연결 → 2 프로젝트 → 3 기본 팀 → Workbench. 끝나면 다시 보이지 않는다.
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import symbol from '$lib/assets/logo/symbol.png';
	import logo from '$lib/assets/logo/logo.png';
	import Sparkles from '@lucide/svelte/icons/sparkles';
	import Code from '@lucide/svelte/icons/code';
	import MousePointer2 from '@lucide/svelte/icons/mouse-pointer-2';
	import Cloud from '@lucide/svelte/icons/cloud';
	import CircleCheck from '@lucide/svelte/icons/circle-check';
	import CircleArrowUp from '@lucide/svelte/icons/circle-arrow-up';
	import LogIn from '@lucide/svelte/icons/log-in';
	import CircleDashed from '@lucide/svelte/icons/circle-dashed';
	import LoaderCircle from '@lucide/svelte/icons/loader-circle';
	import Download from '@lucide/svelte/icons/download';
	import Plus from '@lucide/svelte/icons/plus';
	import RefreshCw from '@lucide/svelte/icons/refresh-cw';
	import ArrowRight from '@lucide/svelte/icons/arrow-right';
	import ArrowLeft from '@lucide/svelte/icons/arrow-left';
	import Info from '@lucide/svelte/icons/info';
	import Folder from '@lucide/svelte/icons/folder';
	import GitBranch from '@lucide/svelte/icons/git-branch';
	import GitFork from '@lucide/svelte/icons/git-fork';
	import Users from '@lucide/svelte/icons/users';
	import Circle from '@lucide/svelte/icons/circle';
	import Play from '@lucide/svelte/icons/play';
	import Hand from '@lucide/svelte/icons/hand';
	import Timer from '@lucide/svelte/icons/timer';
	import Zap from '@lucide/svelte/icons/zap';
	import type { Component } from 'svelte';
	import * as Select from '$lib/components/ui/select';
	import * as InputGroup from '$lib/components/ui/input-group';
	import { Button } from '$lib/components/ui/button';
	import { Steps } from '$lib/components/ui/steps';
	import { Switch } from '$lib/components/ui/switch';
	import { Pill } from '$lib/components/ui/pill';
	import { Progress } from '$lib/components/ui/progress';
	import { Segmented } from '$lib/components/ui/segmented';
	import { RoleAvatar } from '$lib/components/ui/role-avatar';
	import { RuntimeLogo, type Runtime } from '$lib/components/ui/runtime-logo';
	import { accounts, templates, type OrchPolicy } from '$lib/mock';
	import { store, defaultTeam, runtimeName, low } from '$lib/teams.svelte';
	import { SETUP_KEY } from '$lib/setup';
	import { AddConnectionDialog, providerMark, type AddedConnection } from '$lib/components/orch/connection';
		import * as ChoiceCards from '$lib/components/ui/choice-cards';

	// -1 = 스플래시, 0 · 1 · 2 = 단계.
	let step = $state(-1);

	// 스플래시 — 이 기기의 실행기(CLI)를 하나씩 찾는다 (목데이터: 0.25초 간격).
	type Cli = { key: string; name: string; logo?: Runtime; icon?: Component; version: string; state: 'ok' | 'update' | 'updating' | 'gateway' | 'login' | 'missing' | 'installing'; note: string };
	let clis = $state<Cli[]>([
		{ key: 'codex', name: 'Codex CLI', logo: 'codex', version: 'v0.41', state: 'ok', note: '로그인됨 · ChatGPT Pro · 멤버 2' },
		{ key: 'claude', name: 'Claude Code', logo: 'claude', version: 'v2.3', state: 'update', note: 'v2.4 업데이트 가능 · Anthropic Max · 멤버 3' },
		{ key: 'gemini', name: 'Gemini CLI', icon: Sparkles, version: 'v0.9', state: 'ok', note: '로그인됨 · Google AI Pro · 멤버 0' },
		{ key: 'opencode', name: 'OpenCode', icon: Code, version: 'v0.14', state: 'gateway', note: '게이트웨이 · API 키 연결만 사용' },
		{ key: 'cursor', name: 'Cursor Agent', icon: MousePointer2, version: 'v1.2', state: 'login', note: '설치됨 · Cursor 로그인 필요' },
		{ key: 'kiro', name: 'Kiro CLI', icon: Cloud, version: '—', state: 'missing', note: '설치 안 됨 · Amazon Q / Kiro 플랜용' },
	]);
	let found = $state(0);
	onMount(() => {
		const t = setInterval(() => {
			found++;
			if (found >= clis.length) {
				clearInterval(t);
				setTimeout(() => ((step = 0), checkAll()), 400);
			}
		}, 250);
		return () => clearInterval(t);
	});
	const detected = $derived(clis.filter((c) => c.state !== 'missing' && c.state !== 'installing').length);

	// 1 에이전트 연결 — 구독 연결을 확인한다. Anthropic은 Claude Code 업데이트 후에 확인할 수 있다.
	type Conn = { runtime: Runtime; plan: string; h5: number; week: number; state: 'checking' | 'waiting' | 'ok' };
	let conns = $state<Conn[]>(accounts.map((a) => ({ runtime: a.runtime, plan: a.plan, h5: a.h5, week: a.week, state: 'waiting' })));
	const connOk = $derived(conns.filter((c) => c.state === 'ok').length);
	/// 연결 확인 (목데이터: 1.2초 후 성공). Claude Code가 업데이트 전이면 대기로 둔다.
	function check(c: Conn) {
		if (c.runtime === 'claude' && clis.find((x) => x.key === 'claude')!.state !== 'ok') {
			c.state = 'waiting';
			return;
		}
		c.state = 'checking';
		setTimeout(() => (c.state = 'ok'), 1200);
	}
	function checkAll() {
		for (const c of conns) check(c);
	}
	// 연결 추가 다이얼로그 — 실행기 로그인과 '다른 연결 추가'에서 연다.
	let addOpen = $state(false);
	let addProvider = $state<string>();
	let added = $state<AddedConnection[]>([]);
	/// 실행기 로그인은 해당 제공자의 인증 단계로 연다 (구독 · 플랜 토큰은 CLI가 보관).
	const cliProvider: Record<string, string> = { cursor: 'cursor', gemini: 'gemini', claude: 'anthropic', codex: 'chatgpt' };
	function openAdd(provider?: string) {
		addProvider = provider;
		addOpen = true;
	}
	/// 연결이 추가되면 목록에 넣고, 그 로그인을 보관하는 실행기를 로그인됨으로 바꾼다.
	function onAdded(c: AddedConnection) {
		added.push(c);
		const cli = clis.find((x) => x.name === c.provider.cli);
		if (cli && cli.state === 'login') {
			cli.state = 'ok';
			cli.note = `로그인됨 · ${c.provider.plan} · 멤버 0`;
		}
	}
	const connOkAll = $derived(connOk + added.length);

	/// 실행기 동작 (목데이터): 업데이트 · 설치 후 상태를 바꾼다. 로그인은 연결 추가 다이얼로그로.
	function act(c: Cli) {
		if (c.state === 'update') {
			c.state = 'updating';
			setTimeout(() => {
				c.state = 'ok';
				c.version = 'v2.4';
				c.note = '로그인됨 · Anthropic Max · 멤버 3';
				const conn = conns.find((x) => x.runtime === 'claude');
				if (conn) check(conn);
			}, 1200);
		} else if (c.state === 'login' && cliProvider[c.key]) {
			openAdd(cliProvider[c.key]);
		} else if (c.state === 'login') {
			c.state = 'ok';
			c.note = '로그인됨 · Cursor Pro · 멤버 0';
		} else if (c.state === 'missing') {
			c.state = 'installing';
			setTimeout(() => {
				c.state = 'login';
				c.version = 'v0.3';
				c.note = '설치됨 · Kiro 로그인 필요';
			}, 1500);
		}
	}
	const cliMeta: Record<Cli['state'], { icon: Component; tone: string; action?: { icon: Component; label: string } }> = {
		ok: { icon: CircleCheck, tone: 'text-status-done' },
		update: { icon: CircleArrowUp, tone: 'text-status-waiting', action: { icon: Download, label: '업데이트' } },
		updating: { icon: LoaderCircle, tone: 'text-status-in-progress' },
		gateway: { icon: CircleCheck, tone: 'text-status-done' },
		login: { icon: LogIn, tone: 'text-status-waiting', action: { icon: LogIn, label: '로그인' } },
		missing: { icon: CircleDashed, tone: 'text-muted-foreground', action: { icon: Plus, label: '설치' } },
		installing: { icon: LoaderCircle, tone: 'text-status-in-progress' },
	};

	// 2 프로젝트
	let projectName = $state('OrchStack');
	const repos = [
		{ value: 'orchstack/app', meta: 'main · 비공개 · 열린 이슈 12개', issues: 12 },
		{ value: 'orchstack/web', meta: 'main · 공개 · 열린 이슈 4개', issues: 4 },
		{ value: 'orchstack/infra', meta: 'main · 비공개 · 열린 이슈 0개', issues: 0 },
	];
	let repo = $state(repos[0].value);
	const repoInfo = $derived(repos.find((r) => r.value === repo)!);
	const branches = [
		{ value: 'main', meta: '보호됨 · PR 필요' },
		{ value: 'develop', meta: '보호 안 됨' },
	];
	let branch = $state('main');
	let importIssues = $state(true);
	let firstPlan = $state(true);

	// 3 기본 팀 — 추천 구성. 체크 해제하면 빼고 시작한다.
	const recommended = defaultTeam();
	let teamName = $state(recommended.name);
	let picked = $state<number[]>(recommended.members.map((m) => m.sn));
	let extra = $state(false);
	/// 멤버 줄의 요금제 표기 — 'Anthropic · Max' → 'Anthropic Max', 'Codex · ChatGPT Pro' → 'ChatGPT Pro' (.pen 인트로 3).
	const planLabel = (p?: string) => p?.replace('Codex · ', '').replace(' · ', ' ');
	const security = templates.find((t) => t.name === 'Security Reviewer')!;
	let mode = $state<OrchPolicy['mode']>('timer');

	/// 시작 — 고른 구성을 팀에 반영하고 첫 실행을 끝낸다 (서버 저장은 #45 · #47).
	function finish() {
		const team = defaultTeam();
		team.name = teamName.trim() || team.name;
		team.members = team.members.filter((m) => picked.includes(m.sn));
		if (extra) {
			team.members.push({
				sn: Math.max(...store.crew.flatMap((t) => t.members.map((m) => m.sn))) + 1,
				name: '하준', role: security.role, title: security.name, runtime: security.runtime,
				model: `${runtimeName(security.runtime)} · ${security.model}`, status: 'idle', work: 'Orch 배정 대기', next: true,
				context: 0, tokens: 0, load: [], loadNote: '여유',
			});
		}
		store.policies[team.sn].mode = mode;
		localStorage.setItem(SETUP_KEY, '1');
		goto('/p/1', { replaceState: true });
	}

	const stepInfo = $derived([
		{ title: '에이전트 연결', desc: '에이전트는 이 기기의 실행기(CLI)와 모델 연결로 일해요. 연결이 1개 이상 확인되면 다음으로 넘어갈 수 있어요.' },
		{ title: '첫 프로젝트', desc: '에이전트가 일할 저장소를 연결해요. 이슈를 가져오면 Orch가 첫 계획을 세울 수 있어요.' },
		{ title: '기본 팀', desc: `${projectName || '프로젝트'}에 맞춘 추천 구성이에요. 확인된 연결을 역할에 맞게 배정했어요. 그대로 시작하거나 바꿔도 돼요 — 나중에 Teams에서 언제든 수정할 수 있어요.` },
	]);
</script>

<svelte:head><title>시작하기 · OrchStack</title></svelte:head>

<!-- 폼 한 줄 (.pen FormRow) — 이름 · 설명 · 입력 -->
{#snippet field(label: string, hint: string)}
	<span class="flex items-baseline gap-2 text-xs"><span class="font-semibold">{label}</span><span class="text-muted-foreground">{hint}</span></span>
{/snippet}

<main class="flex h-full flex-col items-center overflow-y-auto bg-muted px-6 py-10">
	{#if step < 0}
		<!-- 스플래시 (.pen 인트로 0 · LogoLoader) — 회전 링은 로딩 표시만, 키프레임 연출은 애니메이션 작업 때 -->
		<div class="m-auto flex flex-col items-center gap-4" role="status" aria-live="polite">
			<span class="flex items-center justify-center relative size-30">
				<svg viewBox="0 0 120 120" class="absolute inset-0 size-full" aria-hidden="true">
					<circle cx="60" cy="60" r="56" fill="none" stroke="var(--border)" stroke-width="4" />
					<circle cx="60" cy="60" r="56" fill="none" stroke="var(--primary)" stroke-width="4" stroke-linecap="round" stroke-dasharray="88 264" class="origin-center animate-spin motion-reduce:animate-none" />
				</svg>
				<img src={symbol} alt="" class="size-14" />
			</span>
			<!-- 심볼이 로고의 O와 같아 여기선 글자만 둔다 -->
			<span class="text-2xl font-semibold">OrchStack</span>
			<span class="text-xs text-muted-foreground">이 기기에서 실행기(CLI)를 찾는 중… {found} / {clis.length}</span>
			<Progress value={(found / clis.length) * 100} class="h-1 w-60" aria-label="실행기 찾기" />
			<span class="font-mono text-caption text-subtle-foreground">v0.1 Alpha</span>
		</div>
	{:else}
		<img src={logo} alt="OrchStack" class="mb-6 h-7 w-auto" />
		<div class="card flex w-full max-w-230 flex-col overflow-hidden shadow-sm">
			<header class="flex flex-col gap-3 border-b px-8 pt-7 pb-6">
				<Steps steps={['에이전트 연결', '프로젝트', '기본 팀']} current={step} />
				<h1 class="mt-2 text-2xl font-bold">{stepInfo[step].title}</h1>
				<p class="text-body text-muted-foreground">{stepInfo[step].desc}</p>
			</header>

			<div class="flex flex-col gap-4 px-8 py-6">
				{#if step === 0}
					<span class="text-xs font-semibold text-muted-foreground">실행기 · 이 기기에서 {detected} / {clis.length} 감지</span>
					<div class="grid grid-cols-2 gap-2.5">
						{#each clis as c (c.key)}
							{@const m = cliMeta[c.state]}
							<div class="option-card rounded-lg">
								<span class="icon-tile">
									{#if c.logo}<RuntimeLogo runtime={c.logo} class="size-5 ring-0" />{:else if c.icon}<c.icon class="size-4.5" />{/if}
								</span>
								<span class="flex min-w-0 flex-1 flex-col gap-1">
									<span class="row-title-strong">{c.name}<Pill class="font-mono text-2xs">{c.version}</Pill></span>
									<span class={['meta-truncate gap-1', m.tone]}>
										<m.icon class={['size-3 shrink-0', (c.state === 'updating' || c.state === 'installing') && 'animate-spin']} />
										{c.state === 'updating' ? '업데이트 중…' : c.state === 'installing' ? '설치 중…' : c.note}
									</span>
								</span>
								{#if m.action}
									<Button variant="ghost" size="sm" onclick={() => act(c)}><m.action.icon />{m.action.label}</Button>
								{/if}
							</div>
						{/each}
					</div>
					<span class="mt-2 text-xs font-semibold text-muted-foreground">모델 연결 · 실행기가 쓸 계정</span>
					{#each conns as c (c.runtime)}
						<div class="option-card rounded-lg">
							<span class="icon-tile"><RuntimeLogo runtime={c.runtime} class="size-5 ring-0" /></span>
							<span class="flex min-w-0 flex-1 flex-col gap-1">
								<span class="row-title-strong">{c.plan}<Pill class="bg-primary-soft text-primary">구독</Pill></span>
								<span class={['flex items-center gap-1 text-caption', c.state === 'ok' ? 'text-status-done' : 'text-muted-foreground']}>
									{#if c.state === 'ok'}<CircleCheck class="size-3" />연결됨 · {runtimeName(c.runtime)} {clis.find((x) => x.key === c.runtime)?.version}
									{:else if c.state === 'checking'}<LoaderCircle class="size-3 animate-spin" />연결 확인 중… {runtimeName(c.runtime)} 로그인 토큰 확인
									{:else}<Timer class="size-3" />대기 중 · {runtimeName(c.runtime)} {clis.find((x) => x.key === c.runtime)?.state === 'ok' ? '로그인 확인 필요' : '업데이트 후 확인'}{/if}
								</span>
							</span>
							{#if c.state === 'ok'}
								{#each [['5H', c.h5], ['주간', c.week]] as const as [l, v] (l)}
									<span class="menu-line rounded-sm border text-caption">
										{l}<Progress value={v} class="h-1 w-10 bg-muted" indicator={low(v) ? 'bg-destructive' : 'bg-success'} aria-label="{c.plan} {l} 잔량" />
										<span class={['font-mono', low(v) ? 'text-destructive' : 'text-status-done']}>{v}%</span>
									</span>
								{/each}
							{:else}
								<Button variant="ghost" size="sm" disabled={c.state === 'checking'} onclick={() => check(c)}><RefreshCw />다시 확인</Button>
							{/if}
						</div>
					{/each}
					{#each added as a, i (i)}
						{@const M = providerMark[a.provider.key]}
						<div class="option-card rounded-lg">
							<span class="icon-tile">{#if typeof M === 'string'}<RuntimeLogo runtime={M} class="size-5 ring-0" />{:else}<M class="size-4.5" />{/if}</span>
							<span class="flex min-w-0 flex-1 flex-col gap-1">
								<span class="row-title-strong">{a.title}<Pill class={a.provider.kind === 'API 키' ? 'bg-review-soft text-status-review' : a.provider.kind === '로컬' ? '' : 'bg-primary-soft text-primary'}>{a.provider.kind}</Pill></span>
								<span class="saved-note"><CircleCheck class="size-3" />{a.note}</span>
							</span>
						</div>
					{/each}
					<button type="button" onclick={() => openAdd()} class="flex items-center gap-2 rounded-lg px-3.5 py-2 text-body font-medium outline-none hover:bg-muted focus-visible:ring-3 focus-visible:ring-ring/50">
						<Plus class="size-4" />다른 연결 추가 <span class="font-normal text-muted-foreground">(API 키 · 게이트웨이 · 로컬)</span>
					</button>
					{#if connOkAll}
						<div class="strip strip-success py-2.5">
							<CircleCheck class="size-4 text-status-done" />
							<span class="font-semibold">{connOkAll}개 연결 확인됨</span>
							<span class="text-muted-foreground">다음 단계의 기본 팀 멤버에게 역할에 맞게 자동 배정돼요</span>
						</div>
					{/if}
				{:else if step === 1}
					<label class="flex flex-col gap-2">
						{@render field('이름', '프로젝트 탭 · 브레드크럼에 보여요')}
						<InputGroup.Root class="h-10">
							<InputGroup.Addon><Folder /></InputGroup.Addon>
							<InputGroup.Input bind:value={projectName} placeholder="프로젝트 이름" />
						</InputGroup.Root>
					</label>
					<div class="form-block">
						{@render field('저장소', 'GitHub App이 설치된 저장소만 보여요')}
						<Select.Root type="single" bind:value={repo}>
							<Select.Trigger class="h-10 w-full" aria-label="저장소">
								<span class="flex items-center gap-2"><GitFork class="size-4 text-muted-foreground" /><span class="font-semibold">{repo}</span><span class="font-normal text-muted-foreground">{repoInfo.meta}</span></span>
							</Select.Trigger>
							<Select.Content>{#each repos as r (r.value)}<Select.Item value={r.value} label={r.value}>{r.value} <span class="text-muted-foreground">{r.meta}</span></Select.Item>{/each}</Select.Content>
						</Select.Root>
					</div>
					<div class="form-block">
						{@render field('기본 브랜치', '에이전트는 여기서 feat/<이슈>-<slug> 브랜치를 만들어요')}
						<Select.Root type="single" bind:value={branch}>
							<Select.Trigger class="h-10 w-full" aria-label="기본 브랜치">
								<span class="flex items-center gap-2"><GitBranch class="size-4 text-muted-foreground" /><span class="font-semibold">{branch}</span><span class="font-normal text-muted-foreground">{branches.find((b) => b.value === branch)?.meta}</span></span>
							</Select.Trigger>
							<Select.Content>{#each branches as b (b.value)}<Select.Item value={b.value} label={b.value}>{b.value} <span class="text-muted-foreground">{b.meta}</span></Select.Item>{/each}</Select.Content>
						</Select.Root>
					</div>
					<label class="setup-toggle">
						<span class="flex flex-1 flex-col gap-0.5">
							<span class="text-body font-semibold">GitHub 이슈 가져오기</span>
							<span class="text-xs text-muted-foreground">열린 이슈 {repoInfo.issues}개 → Backlog · 라벨 bug · feature만</span>
						</span>
						<Switch bind:checked={importIssues} />
					</label>
					<label class="setup-toggle">
						<span class="flex flex-1 flex-col gap-0.5">
							<span class="text-body font-semibold">Orch가 첫 계획 세우기</span>
							<span class="text-xs text-muted-foreground">가져온 이슈를 태스크로 나누고 배정안을 제안해요 (승인 후 실행)</span>
						</span>
						<Switch bind:checked={firstPlan} disabled={!importIssues} />
					</label>
				{:else}
					<label class="flex flex-col gap-2">
						{@render field('팀 이름', `프로젝트 ${projectName || ''}을 맡아요`)}
						<InputGroup.Root class="h-10">
							<InputGroup.Addon><Users /></InputGroup.Addon>
							<InputGroup.Input bind:value={teamName} placeholder="팀 이름" />
						</InputGroup.Root>
					</label>
					<div class="strip bg-primary-soft py-2.5">
						<Sparkles class="size-3.5 text-primary" />
						<span class="font-semibold">Orch · PM</span>
						<span class="text-muted-foreground">팀을 이끌고 태스크를 배정해요 · Claude Code · claude-opus-5.5 · Anthropic Max</span>
					</div>
					<span class="text-xs font-semibold text-muted-foreground">멤버 {picked.length + (extra ? 1 : 0)} · 체크 해제하면 빼고 시작해요</span>
					<!-- 여러 명 고르기 — 추천 멤버 + 추가한 역할(extra) -->
					<ChoiceCards.Root
						type="multiple"
						aria-label="기본 팀 멤버"
						class="grid-cols-2"
						bind:value={() => [...picked, ...(extra ? ['extra'] : [])], (v) => ((picked = (v as (number | string)[]).filter((x): x is number => typeof x === 'number')), (extra = (v as unknown[]).includes('extra')))}
					>
						{#each recommended.members as m (m.sn)}
							{@const on = picked.includes(m.sn)}
							<ChoiceCards.Item value={m.sn} layout="row" tone="dim" class="items-start rounded-lg px-3.5 py-3">
								<RoleAvatar role={m.role} />
								<span class="flex min-w-0 flex-1 flex-col gap-0.5">
									<span class="text-body font-semibold">{m.name} · {m.title}</span>
									<span class="text-caption text-muted-foreground">템플릿 {m.title}</span>
									<span class="meta-line gap-1"><RuntimeLogo runtime={m.runtime} class="size-3 ring-0" />{m.model} · {planLabel(conns.find((c) => c.runtime === m.runtime)?.plan)}</span>
								</span>
								{#if on}<CircleCheck class="size-4 shrink-0 text-primary" />{:else}<Circle class="size-4 shrink-0 text-subtle-foreground" />{/if}
							</ChoiceCards.Item>
						{/each}
						{#if extra}
							<ChoiceCards.Item value="extra" layout="row" tone="dim" class="items-start rounded-lg px-3.5 py-3">
								<RoleAvatar role={security.role} />
								<span class="flex min-w-0 flex-1 flex-col gap-0.5">
									<span class="text-body font-semibold">하준 · {security.name}</span>
									<span class="text-caption text-muted-foreground">템플릿 {security.name} v{security.version}</span>
									<span class="meta-line gap-1"><RuntimeLogo runtime={security.runtime} class="size-3 ring-0" />{runtimeName(security.runtime)} · {security.model}</span>
								</span>
								<CircleCheck class="size-4 shrink-0 text-primary" />
							</ChoiceCards.Item>
						{:else}
							<button type="button" onclick={() => (extra = true)} class="flex items-center justify-center gap-2 rounded-lg border border-dashed bg-muted/50 px-3.5 py-3 text-xs text-muted-foreground outline-none hover:bg-muted focus-visible:ring-3 focus-visible:ring-ring/50">
								<Plus class="size-3.5" />역할 추가 (Security Reviewer 등)
							</button>
						{/if}
					</ChoiceCards.Root>
					<div class="form-block">
						{@render field('Orch 진행 방식', mode === 'timer' ? 'Auto = 판단이 필요 없는 일은 5초 타이머 후 자동 진행' : mode === 'manual' ? 'Manual = 매번 확인 후 진행' : 'Full auto = 대기 없이 진행 (루프 가드는 항상 적용)')}
						<Segmented
							aria-label="Orch 진행 방식"
							options={[{ value: 'manual', label: 'Manual', icon: Hand }, { value: 'timer', label: 'Auto · 5초 (추천)', icon: Timer }, { value: 'full', label: 'Full auto', icon: Zap }]}
							bind:value={() => mode, (v) => (mode = v as OrchPolicy['mode'])}
						/>
					</div>
				{/if}
			</div>

			<footer class="flex items-center gap-2.5 border-t bg-muted/50 px-8 py-4">
				{#if step === 0}
					<span class="meta-xs flex-1 gap-1.5">
						{#if connOkAll}<CircleCheck class="size-3.5 text-status-done" />준비 완료 · 실행기 {detected} · 연결 {connOkAll}{:else}<LoaderCircle class="size-3.5 animate-spin" />연결 확인 중 · 1개 이상 확인되면 다음 버튼이 켜져요{/if}
					</span>
					<Button variant="ghost" size="sm" onclick={() => (step = 1)}>나중에 설정</Button>
					<Button size="sm" disabled={!connOkAll} onclick={() => (step = 1)}><ArrowRight />다음 · 프로젝트</Button>
				{:else if step === 1}
					<span class="meta-xs flex-1 gap-1.5"><Info class="size-3.5" />프로젝트는 나중에 상단 탭 + 에서 더 추가할 수 있어요</span>
					<Button variant="ghost" size="sm" onclick={() => (step = 0)}><ArrowLeft />이전</Button>
					<Button size="sm" disabled={!projectName.trim()} onclick={() => (step = 2)}><ArrowRight />다음 · 기본 팀</Button>
				{:else}
					<span class="meta-xs flex-1 gap-1.5">
						<Info class="size-3.5" />{importIssues && firstPlan ? `시작하면 Orch가 가져온 이슈 ${repoInfo.issues}개로 첫 계획을 제안해요` : '시작하면 빈 Workbench에서 Orch에게 목표를 알려주세요'}
					</span>
					<Button variant="ghost" size="sm" onclick={() => (step = 1)}><ArrowLeft />이전</Button>
					<Button size="sm" disabled={!picked.length && !extra} onclick={finish}><Play />그대로 시작 → Workbench</Button>
				{/if}
			</footer>
		</div>
	{/if}
</main>

<AddConnectionDialog bind:open={addOpen} provider={addProvider} onadd={onAdded} />
