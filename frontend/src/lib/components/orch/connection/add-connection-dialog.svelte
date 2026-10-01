<script lang="ts" module>
	import type { Component } from 'svelte';
	import Sparkles from '@lucide/svelte/icons/sparkles';
	import Bot from '@lucide/svelte/icons/bot';
	import MousePointer2 from '@lucide/svelte/icons/mouse-pointer-2';
	import Moon from '@lucide/svelte/icons/moon';
	import Route from '@lucide/svelte/icons/route';
	import GitFork from '@lucide/svelte/icons/git-fork';
	import Triangle from '@lucide/svelte/icons/triangle';
	import Cpu from '@lucide/svelte/icons/cpu';
	import MonitorCog from '@lucide/svelte/icons/monitor-cog';
	import Plug from '@lucide/svelte/icons/plug';
	import Zap from '@lucide/svelte/icons/zap';
	import Cloud from '@lucide/svelte/icons/cloud';
	import type { Runtime } from '$lib/components/ui/runtime-logo';
	import type { Provider } from '$lib/mock';

	/// 제공자 표시 — 실행기 로고가 있으면 로고, 없으면 아이콘.
	export const providerMark: Record<string, Runtime | Component> = {
		anthropic: 'claude', chatgpt: 'codex', gemini: Sparkles, copilot: Bot, cursor: MousePointer2, kimi: Moon, zai: Zap, kiro: Cloud,
		'openai-api': 'codex', 'anthropic-api': 'claude', 'ai-studio': Sparkles,
		omniroute: Route, openrouter: GitFork, vercel: Triangle, ollama: Cpu, lmstudio: MonitorCog, compat: Plug,
	};

	/** 추가된 연결 — 부르는 쪽 목록에 한 줄로 보여줄 값. */
	export type AddedConnection = { provider: Provider; title: string; note: string };
</script>

<script lang="ts">
	/// 연결 추가 (.pen Settings · 연결 추가 1 · 2 · 2' · 3) — 제공자 선택 → 인증 → 사용 범위 · 확인.
	/// 인트로 1단계와 설정 › 모델 연결에서 같이 쓴다. 로그인 · 연결 테스트는 목데이터 (서버 연결 #59).
	import { untrack } from 'svelte';
	import PlugZap from '@lucide/svelte/icons/plug-zap';
	import Search from '@lucide/svelte/icons/search';
	import Check from '@lucide/svelte/icons/check';
	import Circle from '@lucide/svelte/icons/circle';
	import CircleDot from '@lucide/svelte/icons/circle-dot';
	import CircleCheck from '@lucide/svelte/icons/circle-check';
	import CircleMinus from '@lucide/svelte/icons/circle-minus';
	import CirclePlus from '@lucide/svelte/icons/circle-plus';
	import ScanSearch from '@lucide/svelte/icons/scan-search';
	import LogIn from '@lucide/svelte/icons/log-in';
	import LoaderCircle from '@lucide/svelte/icons/loader-circle';
	import ChevronsUpDown from '@lucide/svelte/icons/chevrons-up-down';
	import ArrowLeft from '@lucide/svelte/icons/arrow-left';
	import ArrowRight from '@lucide/svelte/icons/arrow-right';
	import Info from '@lucide/svelte/icons/info';
	import RefreshCw from '@lucide/svelte/icons/refresh-cw';
	import Terminal from '@lucide/svelte/icons/terminal';
	import Globe from '@lucide/svelte/icons/globe';
	import Smartphone from '@lucide/svelte/icons/smartphone';
	import Tag from '@lucide/svelte/icons/tag';
	import Link from '@lucide/svelte/icons/link';
	import KeyRound from '@lucide/svelte/icons/key-round';
	import Wallet from '@lucide/svelte/icons/wallet';
	import Activity from '@lucide/svelte/icons/activity';
	import Users from '@lucide/svelte/icons/users';
	import User from '@lucide/svelte/icons/user';
	import Languages from '@lucide/svelte/icons/languages';
	import GitCommitHorizontal from '@lucide/svelte/icons/git-commit-horizontal';
	import Bell from '@lucide/svelte/icons/bell';
	import * as Dialog from '$lib/components/ui/dialog';
	import * as Select from '$lib/components/ui/select';
	import * as InputGroup from '$lib/components/ui/input-group';
	import { Button } from '$lib/components/ui/button';
	import { Steps } from '$lib/components/ui/steps';
	import { Pill } from '$lib/components/ui/pill';
	import { Progress } from '$lib/components/ui/progress';
	import { Segmented } from '$lib/components/ui/segmented';
	import { RuntimeLogo } from '$lib/components/ui/runtime-logo';
	import * as Field from '$lib/components/ui/field';
	import { providers, type ProviderKind } from '$lib/mock';
	import { store } from '$lib/teams.svelte';
	import { cn } from '$lib/utils';
	import * as ChoiceCards from '$lib/components/ui/choice-cards';

	let {
		open = $bindable(false),
		provider,
		onadd,
	}: {
		open?: boolean;
		/** 열 때 고른 제공자 key — 있으면 인증 단계부터 시작한다. */
		provider?: string;
		onadd?: (c: AddedConnection) => void;
	} = $props();

	const kinds: ProviderKind[] = ['구독', '플랜', 'API 키', '게이트웨이', '로컬'];
	const kindMeta: Record<ProviderKind, { desc: string; pill: string }> = {
		구독: { desc: 'CLI 계정으로 로그인 · 5시간 / 주간 한도로 표시', pill: 'bg-primary-soft text-primary' },
		플랜: { desc: '코딩 도구 요금제 · 월 사용량 한도', pill: 'bg-primary-soft text-primary' },
		'API 키': { desc: '사용한 만큼 과금 · 월 예산으로 표시', pill: 'bg-review-soft text-status-review' },
		게이트웨이: { desc: 'OpenAI 호환 엔드포인트 · 여러 모델 라우팅 · 비용 + 폴백 횟수', pill: 'bg-primary-soft text-primary' },
		로컬: { desc: '내 컴퓨터에서 실행 · 한도 없음, 속도로 표시', pill: '' },
	};
	const stateMeta: Record<Provider['state'], { icon: Component; tone: string }> = {
		ok: { icon: CircleCheck, tone: 'text-status-done' },
		detected: { icon: ScanSearch, tone: 'text-primary' },
		login: { icon: LogIn, tone: 'text-status-waiting' },
		open: { icon: CirclePlus, tone: 'text-muted-foreground' },
	};
	/// 구독 로그인은 그 CLI 안에서만 쓴다 — 다른 실행기가 못 쓰는 이유.
	const subOnly: Record<string, string> = {
		'Claude Code': 'Claude 구독 전용', 'Codex CLI': 'ChatGPT 구독 전용', 'Gemini CLI': 'Google 구독 전용',
		'Cursor Agent': 'Cursor 플랜 전용', 'Kiro CLI': 'Kiro 플랜 전용', OpenCode: '구독 토큰 공유 불가',
	};
	const loginCmd: Record<string, string> = {
		'Claude Code': 'claude /login', 'Codex CLI': 'codex login', 'Gemini CLI': 'gemini auth login',
		'Cursor Agent': 'cursor-agent login', 'Kiro CLI': 'kiro-cli login', OpenCode: 'opencode auth login',
	};
	const fallbacks = [
		{ value: 'claude', label: 'Claude Code 체인 · 4순위', meta: 'OmniRoute 다음' },
		{ value: 'codex', label: 'Codex CLI 체인 · 3순위', meta: 'Anthropic API 다음' },
		{ value: 'none', label: '폴백에 넣지 않음', meta: '직접 고른 멤버만 사용' },
	];
	const langs = [
		{ value: '한국어', meta: '워크스페이스 기본 따름' },
		{ value: 'English', meta: '' },
		{ value: '日本語', meta: '' },
	];
	const commitLangs = [
		{ value: 'English', meta: '저장소 규칙 따름' },
		{ value: '한국어', meta: '' },
	];
	const channels = [
		{ value: '앱 + Telegram 알림방', meta: '설정 › 알림에서 관리' },
		{ value: '앱만', meta: '' },
	];
	const teamNames = $derived(store.crew.filter((t) => !t.orch).map((t) => t.name).join(' · '));

	let step = $state(0);
	let q = $state('');
	let filter = $state<ProviderKind | '전체'>('전체');
	let sel = $state<Provider>();
	// 2 인증
	let name = $state('');
	let baseUrl = $state('');
	let apiKey = $state('');
	let budget = $state('50');
	let method = $state('browser');
	let login = $state<'idle' | 'pending' | 'ok'>('idle');
	let test = $state<'idle' | 'pending' | 'ok'>('idle');
	// 3 사용 범위 · 확인
	let scope = $state('team');
	let fallback = $state('claude');
	let lang = $state('한국어');
	let commitLang = $state('English');
	let channel = $state('앱 + Telegram 알림방');
	let notify = $state<{ name: string; desc: string; on: boolean }[]>([]);

	/// 제공자를 고르면 인증 입력을 그 제공자 기본값으로 채운다.
	function pick(p: Provider) {
		sel = p;
		name = p.kind === '로컬' ? p.key : `${p.key}-team`;
		baseUrl = p.baseUrl ?? '';
		apiKey = '';
		login = 'idle';
		test = 'idle';
	}
	// 열 때마다 처음부터 (provider가 있으면 인증 단계부터).
	$effect.pre(() => {
		if (!open) return;
		untrack(() => {
			const p = providers.find((x) => x.key === provider);
			q = '';
			filter = '전체';
			step = p ? 1 : 0;
			sel = undefined;
			if (p) pick(p);
			scope = 'team';
			fallback = 'claude';
			method = 'browser';
		});
	});

	const isSub = $derived(sel?.kind === '구독' || sel?.kind === '플랜');
	const needsKey = $derived(sel?.kind === 'API 키' || sel?.kind === '게이트웨이');
	const needsUrl = $derived(sel?.kind === '게이트웨이' || sel?.kind === '로컬');
	const vendor = $derived(sel?.name.split(' · ')[0] ?? '');
	const title = $derived(isSub ? (sel?.plan?.startsWith(vendor) ? sel.plan : `${vendor} · ${sel?.plan}`) : name.trim() || (sel?.name ?? ''));
	const authed = $derived(isSub ? login === 'ok' : test === 'ok' && (!needsKey || !!apiKey.trim()) && (!needsUrl || !!baseUrl.trim()));
	const hits = $derived(providers.filter((p) => (filter === '전체' || p.kind === filter) && `${p.name} ${p.kind}`.toLowerCase().includes(q.trim().toLowerCase())));
	const compat = $derived<[string, string, boolean][]>(
		!sel
			? []
			: isSub
				? [[sel.cli!, '로그인한 CLI', true], ...(['Claude Code', 'Codex CLI', 'OpenCode', 'Gemini CLI'] as const).filter((c) => c !== sel!.cli).slice(0, 3).map((c): [string, string, boolean] => [c, subOnly[c], false])]
				: [
						['Claude Code', 'ANTHROPIC_BASE_URL로 연결', true],
						['Codex CLI', 'model_provider 추가', true],
						['OpenCode', 'provider 설정에 추가', true],
						['Gemini CLI', 'OpenAI 호환 미지원', false],
					]
	);

	/// 인증 → 사용 범위로 넘어갈 때 제공자 종류에 맞는 알림 기본값을 만든다.
	function toScope() {
		notify = isSub
			? [
					{ name: '한도 80% 사용', desc: '5시간 · 주간 한도 중 먼저 닿는 쪽', on: true },
					{ name: '한도 소진 → 폴백으로 전환', desc: '리셋까지 다음 순위 연결을 써요', on: true },
					{ name: '로그인 만료', desc: `${sel?.cli} 토큰이 만료되면 알림`, on: true },
					{ name: '폴백으로 사용될 때마다', desc: '잦으면 소음이 될 수 있어요', on: false },
				]
			: needsKey
				? [
						{ name: '예산 80% 도달', desc: `월 $${Number(budget) * 0.8} 사용 시 경고`, on: true },
						{ name: '예산 초과 → 폴백에서 제외', desc: `$${budget} 도달 시 자동 제외하고 알림`, on: true },
						{ name: '연결 오류 · 키 만료', desc: '테스트 실패 3회 또는 401 응답', on: true },
						{ name: '폴백으로 사용될 때마다', desc: '잦으면 소음이 될 수 있어요', on: false },
					]
				: [
						{ name: '연결 오류', desc: '로컬 서버가 응답하지 않으면 알림', on: true },
						{ name: '폴백으로 사용될 때마다', desc: '잦으면 소음이 될 수 있어요', on: false },
					];
		step = 2;
	}

	/// 로그인 · 연결 테스트 (목데이터: 잠시 후 성공).
	function run(set: (v: 'pending' | 'ok') => void) {
		set('pending');
		setTimeout(() => set('ok'), 1200);
	}

	function add() {
		if (!sel) return;
		onadd?.({
			provider: sel,
			title,
			note: isSub ? `연결됨 · ${sel.cli}` : sel.kind === '로컬' ? `연결됨 · 모델 ${sel.models}` : `연결됨 · ${sel.models} 모델 · 월 $${budget}`,
		});
		open = false;
	}

	const confirmRows = $derived<[string, string][]>(
		!sel
			? []
			: [
					...(isSub ? [['실행기', `${sel.cli} · ${sel.plan}`] as [string, string]] : []),
					...(needsUrl ? [['엔드포인트', baseUrl.replace(/^https?:\/\//, '')] as [string, string]] : []),
					...(needsKey ? [['API 키', `키체인 · •••${apiKey.slice(-4)}`], ['월 예산', `$${budget} · 80% 경고`]] as [string, string][] : []),
					['사용 범위', scope === 'workspace' ? '워크스페이스 전체' : scope === 'team' ? teamNames : '나만'],
					['폴백', fallbacks.find((f) => f.value === fallback)!.label.replace(' · ', ' ')],
					['언어', `보고 ${lang} · 커밋 ${commitLang}`],
					['알림', `${notify.filter((n) => n.on).length}개 켜짐 · ${channel}`],
				]
	);
</script>

{#snippet field(label: string, hint: string)}
	<span class="flex items-baseline gap-2 text-xs"><span class="font-semibold">{label}</span><span class="text-muted-foreground">{hint}</span></span>
{/snippet}

{#snippet optLabel(Icon: Component, value: string, opts: { value: string; meta: string }[])}
	<span class="flex items-center gap-2"><Icon class="size-4 text-muted-foreground" /><span class="font-medium">{value}</span><span class="text-muted-foreground">{opts.find((o) => o.value === value)?.meta}</span></span>
{/snippet}

{#snippet logoOf(key: string, size: 'sm' | 'lg')}
	{@const M = providerMark[key]}
	{#if typeof M === 'string'}<RuntimeLogo runtime={M} class={cn('ring-0', size === 'lg' ? 'size-5' : 'size-4')} />{:else}<M class={size === 'lg' ? 'size-5' : 'size-4'} />{/if}
{/snippet}

<!-- 추가되면 이렇게 보여요 — 인증 · 확인 단계 오른쪽 -->
{#snippet preview()}
	{#if sel}
		<span class="list-label px-0 pt-0">추가되면 이렇게 보여요</span>
		<div class="flex items-center gap-3 rounded-lg border bg-card px-4 py-3.5">
			<span class="icon-tile">{@render logoOf(sel.key, 'lg')}</span>
			<span class="flex min-w-0 flex-1 flex-col gap-1">
				<span class="row-title-strong">{isSub ? sel.name : title}<Pill class={kindMeta[sel.kind].pill}>{sel.kind}</Pill></span>
				{#if authed}
					<span class="saved-note"><CircleCheck class="size-3" />{isSub ? `${sel.plan} · ${sel.cli}` : `${name} · ${sel.models} 모델`}</span>
				{:else}
					<span class="text-caption text-muted-foreground">{isSub ? '로그인 전' : '연결 테스트 전'}</span>
				{/if}
			</span>
			<span class="menu-line rounded-sm border text-caption">
				{#if isSub}일<Progress value={authed ? 100 : 0} class="h-1 w-10 bg-muted" indicator="bg-success" aria-label="일 한도" /><span class="font-mono text-status-done">{authed ? '100%' : '—'}</span>
				{:else if needsKey}월<Progress value={0} class="h-1 w-10 bg-muted" aria-label="월 예산" /><span class="font-mono text-primary">$0/${budget}</span>
				{:else}한도 없음{/if}
			</span>
		</div>
	{/if}
{/snippet}

<Dialog.Root bind:open={() => open, (v) => (open = v)}>
	<Dialog.Content size="xl" tall>
		<Dialog.Header icon={PlugZap}>
			<Dialog.Title>연결 추가</Dialog.Title>
			<Dialog.Description>에이전트가 모델을 부를 경로 · 워크스페이스 전체에서 사용</Dialog.Description>
			{#snippet sub()}<Steps steps={['제공자 선택', '인증', '사용 범위 · 확인']} current={step} />{/snippet}
		</Dialog.Header>

		<Dialog.Body class="gap-4">
			{#if step === 0}
				<div class="flex items-center gap-3">
					<InputGroup.Root class="h-9 flex-1">
						<InputGroup.Addon><Search /></InputGroup.Addon>
						<InputGroup.Input bind:value={q} placeholder="제공자 검색 · OpenAI, Gemini, DeepSeek, OpenAI 호환…" aria-label="제공자 검색" />
					</InputGroup.Root>
					<div class="flex items-center gap-1" role="radiogroup" aria-label="제공자 종류">
						{#each ['전체', ...kinds] as const as k (k)}
							<button
								type="button"
								role="radio"
								aria-checked={filter === k}
								onclick={() => (filter = k)}
								class={cn('flex items-center gap-1 rounded-full px-2.5 py-1 text-caption font-medium outline-none focus-visible:ring-3 focus-visible:ring-ring/50', filter === k ? 'bg-foreground text-background' : 'bg-muted text-muted-foreground hover:text-foreground')}
							>
								{k}<span class="opacity-70">{k === '전체' ? providers.length : providers.filter((p) => p.kind === k).length}</span>
							</button>
						{/each}
					</div>
				</div>
				{#each kinds as k (k)}
					{@const list = hits.filter((p) => p.kind === k)}
					{#if list.length}
						<section class="flex flex-col gap-2">
							<span class="flex items-baseline gap-2 text-xs"><span class="font-semibold">{k}</span><span class="text-muted-foreground">{kindMeta[k].desc}</span></span>
							<ChoiceCards.Root aria-label={k} class="grid-cols-3" value={sel?.key ?? null} onValueChange={(v) => pick(list.find((x) => x.key === v)!)}>
								{#each list as p (p.key)}
									{@const on = sel?.key === p.key}
									{@const s = stateMeta[p.state]}
									<ChoiceCards.Item value={p.key} ondblclick={() => (pick(p), (step = 1))} class="gap-2 rounded-lg px-3.5 py-3 hover:bg-muted/50">
										<span class="row-title-strong">
											{@render logoOf(p.key, 'sm')}<span class="flex-1 truncate">{p.name}</span>
											{#if on}<CircleDot class="size-4 text-primary" />{:else}<Circle class="size-4 text-subtle-foreground" />{/if}
										</span>
										<span class="flex items-center gap-2 text-caption">
											<Pill class={kindMeta[p.kind].pill}>{p.kind}</Pill>
											<span class={cn('flex min-w-0 items-center gap-1', s.tone)}><s.icon class="size-3 shrink-0" /><span class="truncate">{p.note}</span></span>
										</span>
									</ChoiceCards.Item>
								{/each}
							</ChoiceCards.Root>
						</section>
					{/if}
				{/each}
				{#if !hits.length}
					<p class="py-6 text-center text-xs text-muted-foreground">‘{q}’에 맞는 제공자가 없어요. 직접 입력하려면 로컬 › OpenAI 호환을 고르세요.</p>
				{/if}
			{:else if sel}
				<div class="grid grid-cols-2 gap-8">
					<div class="flex flex-col gap-4">
						{#if step === 1}
						<div class="flex flex-col gap-2">
							{@render field('제공자', isSub ? '구독 로그인' : sel.kind === 'API 키' ? '사용한 만큼 과금' : sel.kind === '게이트웨이' ? 'OpenAI 호환 게이트웨이' : '내 컴퓨터에서 실행')}
							<button type="button" onclick={() => (step = 0)} class="flex h-10 items-center gap-2 rounded-md border bg-card px-3 text-left text-body outline-none hover:bg-muted/50 focus-visible:ring-3 focus-visible:ring-ring/50">
								{@render logoOf(sel.key, 'sm')}<span class="font-medium">{sel.name}</span><Pill class={kindMeta[sel.kind].pill}>{sel.kind}</Pill>
								<span class="text-xs text-muted-foreground">변경</span><ChevronsUpDown class="ml-auto size-4 text-muted-foreground" />
							</button>
						</div>
						{/if}

						{#if step === 1 && isSub}
							<div class="form-block">
								{@render field('실행기', '구독 로그인은 이 CLI 안에서만 쓸 수 있어요')}
								<span class="flex h-10 items-center gap-2 rounded-md border bg-muted/50 px-3 text-body"><Terminal class="size-4 text-muted-foreground" /><span class="font-medium">{sel.cli}</span><span class="text-xs text-muted-foreground">설치됨</span></span>
							</div>
							<div class="form-block">
								{@render field('로그인 방식', '브라우저가 없으면 기기 코드로 로그인')}
								<Segmented aria-label="로그인 방식" disabled={login !== 'idle'} options={[{ value: 'browser', label: '브라우저 로그인', icon: Globe }, { value: 'device', label: '기기 코드', icon: Smartphone }, { value: 'terminal', label: '터미널에서 직접', icon: Terminal }]} bind:value={method} />
							</div>
							<div class="form-block">
								{@render field('로그인 상태', login === 'ok' ? `브라우저에서 ${vendor} 계정으로 로그인했어요` : `${sel.cli}가 로그인 토큰을 보관해요`)}
								{#if login === 'ok'}
									<div class="strip strip-success py-2.5">
										<CircleCheck class="size-4 text-status-done" /><span class="font-semibold">로그인됨</span><span class="text-muted-foreground">{sel.plan} 플랜 감지 · 토큰은 {sel.cli}가 보관</span>
									</div>
								{:else if login === 'pending'}
									<div class="strip border py-2.5 text-muted-foreground">
										<LoaderCircle class="size-4 animate-spin text-primary" />
										{#if method === 'device'}<span>코드 <span class="font-mono font-semibold text-foreground">WDJB-MJHT</span> 를 다른 기기에서 입력하는 중…</span>
										{:else if method === 'terminal'}<span>터미널에서 로그인 확인 중…</span>
										{:else}<span>열린 브라우저에서 로그인을 기다리는 중…</span>{/if}
									</div>
								{:else}
									<div class="flex items-center gap-3">
										{#if method === 'terminal'}<code class="flex-1 rounded-md bg-code-bg px-3 py-2 font-mono text-xs text-code-fg">{loginCmd[sel.cli!]}</code>{/if}
										<Button variant="outline" size="sm" onclick={() => run((v) => (login = v))}>
											{#if method === 'terminal'}<RefreshCw />로그인 확인{:else if method === 'device'}<Smartphone />기기 코드 받기{:else}<LogIn />브라우저에서 로그인{/if}
										</Button>
									</div>
								{/if}
							</div>
							{#if login === 'ok'}
								<div class="form-block">
									{@render field('감지된 한도', '구독은 금액 대신 남은 비율로 보여요')}
									<span class="meta-line gap-2">
										{#each ['5H', '주간'] as l (l)}
											<span class="chip-box gap-1.5 py-1">{l}<Progress value={100} class="h-1 w-10 bg-muted" indicator="bg-success" aria-label="{l} 잔량" /><span class="font-mono text-status-done">100%</span></span>
										{/each}
										리셋 시각은 첫 사용부터 계산돼요
									</span>
								</div>
							{/if}
						{:else if step === 1}
							<label class="form-block">
								{@render field('이름', '멤버 · 템플릿에서 이 이름으로 보여요')}
								<InputGroup.Root class="h-10"><InputGroup.Addon><Tag /></InputGroup.Addon><InputGroup.Input bind:value={name} placeholder="연결 이름" /></InputGroup.Root>
							</label>
							{#if needsUrl}
								<label class="form-block">
									{@render field('Base URL', 'OpenAI 호환 엔드포인트')}
									<InputGroup.Root class="h-10"><InputGroup.Addon><Link /></InputGroup.Addon><InputGroup.Input bind:value={baseUrl} oninput={() => (test = 'idle')} placeholder="http://localhost:8000/v1" class="font-mono" /></InputGroup.Root>
								</label>
							{/if}
							{#if needsKey}
								<label class="form-block">
									{@render field('API 키', '키체인에 저장 · 에이전트 프롬프트에 원문이 들어가지 않아요')}
									<InputGroup.Root class="h-10"><InputGroup.Addon><KeyRound /></InputGroup.Addon><InputGroup.Input type="password" bind:value={apiKey} oninput={() => (test = 'idle')} placeholder="키 붙여넣기" class="font-mono" /></InputGroup.Root>
								</label>
								<div class="form-block">
									{@render field('월 예산', '넘으면 이 연결을 폴백에서 건너뛰어요')}
									<Select.Root type="single" bind:value={budget}>
										<Select.Trigger class="h-10 w-full" aria-label="월 예산"><span class="flex items-center gap-2"><Wallet class="size-4 text-muted-foreground" />${budget} · 80%에서 경고</span></Select.Trigger>
										<Select.Content>{#each ['20', '50', '100', '200'] as b (b)}<Select.Item value={b} label="${b}">${b}</Select.Item>{/each}</Select.Content>
									</Select.Root>
								</div>
							{/if}
							<div class="flex flex-col gap-2.5 border-t pt-4">
								<div class="flex items-center gap-3">
									<Button variant="outline" size="sm" disabled={test === 'pending' || (needsKey && !apiKey.trim()) || (needsUrl && !baseUrl.trim())} onclick={() => run((v) => (test = v))}>
										{#if test === 'pending'}<LoaderCircle class="animate-spin" />{:else}<Activity />{/if}연결 테스트
									</Button>
									<span class="text-xs text-muted-foreground">{needsKey && !apiKey.trim() ? 'API 키를 넣으면 테스트할 수 있어요' : '모델 목록을 불러오고 짧은 요청 1회를 보내요'}</span>
								</div>
								{#if test === 'ok'}
									<div class="strip strip-success py-2.5">
										<CircleCheck class="size-4 text-status-done" /><span class="font-semibold">연결 성공</span>
										<span class="text-muted-foreground">{sel.kind === '로컬' ? `모델 ${sel.models}개 · 응답 42ms · 로컬이라 비용 없음` : `모델 ${sel.models}개 · 응답 310ms · 가격표 불러옴 (Run당 비용 추정에 사용)`}</span>
									</div>
								{/if}
							</div>
						{:else}
							<!-- 3 사용 범위 · 언어 · 알림 -->
							<div class="flex flex-col gap-2">
								{@render field('허용 대상', scope === 'team' ? `${teamNames} 멤버만 모델 선택 · 폴백에서 볼 수 있어요` : scope === 'workspace' ? '모든 팀 멤버가 모델 선택 · 폴백에서 볼 수 있어요' : '내가 시작한 Run에서만 써요')}
								<Segmented aria-label="허용 대상" options={[{ value: 'workspace', label: '워크스페이스', icon: Globe }, { value: 'team', label: '선택한 팀', icon: Users }, { value: 'me', label: '나만', icon: User }]} bind:value={scope} />
							</div>
							<div class="form-block">
								{@render field('폴백 체인', '연결이 준비되면 이 위치에 추가돼요')}
								<Select.Root type="single" bind:value={fallback}>
									<Select.Trigger class="h-10 w-full" aria-label="폴백 체인"><span class="flex items-center gap-2"><Route class="size-4 text-muted-foreground" /><span class="font-medium">{fallbacks.find((f) => f.value === fallback)?.label}</span><span class="text-muted-foreground">{fallbacks.find((f) => f.value === fallback)?.meta}</span></span></Select.Trigger>
									<Select.Content>{#each fallbacks as f (f.value)}<Select.Item value={f.value} label={f.label}>{f.label} <span class="text-muted-foreground">{f.meta}</span></Select.Item>{/each}</Select.Content>
								</Select.Root>
							</div>
							<div class="form-block">
								{@render field('응답 · 보고 언어', '완료 보고 · 요약 · 질문')}
								<Select.Root type="single" bind:value={lang}>
									<Select.Trigger class="h-10 w-full" aria-label="응답 · 보고 언어">{@render optLabel(Languages, lang, langs)}</Select.Trigger>
									<Select.Content>{#each langs as o (o.value)}<Select.Item value={o.value} label={o.value}>{o.value} <span class="text-muted-foreground">{o.meta}</span></Select.Item>{/each}</Select.Content>
								</Select.Root>
							</div>
							<div class="form-block">
								{@render field('커밋 · PR 언어', '코드 주석 · 커밋 메시지 · PR 본문')}
								<Select.Root type="single" bind:value={commitLang}>
									<Select.Trigger class="h-10 w-full" aria-label="커밋 · PR 언어">{@render optLabel(GitCommitHorizontal, commitLang, commitLangs)}</Select.Trigger>
									<Select.Content>{#each commitLangs as o (o.value)}<Select.Item value={o.value} label={o.value}>{o.value} <span class="text-muted-foreground">{o.meta}</span></Select.Item>{/each}</Select.Content>
								</Select.Root>
							</div>
							<div class="flex flex-col border-t pt-4">
								{@render field('알림', '한도 · 오류를 어디로 알릴지')}
								{#each notify as n (n.name)}
									<Field.SwitchRow label={n.name} hint={n.desc} bind:checked={n.on} />
								{/each}
							</div>
							<div class="flex flex-col gap-2">
								{@render field('알림 채널', '설정 › 알림의 채널을 사용해요')}
								<Select.Root type="single" bind:value={channel}>
									<Select.Trigger class="h-10 w-full" aria-label="알림 채널"><span class="flex items-center gap-2"><Bell class="size-4 text-muted-foreground" /><span class="font-medium">{channel}</span><span class="text-muted-foreground">{channels.find((c) => c.value === channel)?.meta}</span></span></Select.Trigger>
									<Select.Content>{#each channels as c (c.value)}<Select.Item value={c.value} label={c.value}>{c.value}</Select.Item>{/each}</Select.Content>
								</Select.Root>
							</div>
						{/if}
					</div>

					<aside class="flex flex-col gap-2">
						{@render preview()}
						{#if step === 1}
							<span class="list-label px-0 pt-4">이 연결을 쓸 수 있는 실행기</span>
							{#each compat as [cli, why, ok] (cli)}
								<span class={cn('flex items-center gap-2 text-body', !ok && 'text-muted-foreground')}>
									{#if ok}<CircleCheck class="size-4 text-status-done" />{:else}<CircleMinus class="size-4" />{/if}
									<span class={cn('flex-1', ok && 'font-medium')}>{cli}</span><span class="text-caption text-muted-foreground">{why}</span>
								</span>
							{/each}
						{:else}
							<span class="list-label px-0 pt-4">확인</span>
							{#each confirmRows as [k, v] (k)}
								<span class="flex items-center justify-between gap-4 border-t py-2 text-body"><span class="text-muted-foreground">{k}</span><span class="text-right">{v}</span></span>
							{/each}
							<span class="strip strip-muted mt-2 py-2.5">
								<RefreshCw class="size-3.5 text-primary" /><span class="font-semibold">추가 후</span>
								<span class="text-muted-foreground">{isSub ? `남은 한도를 5분마다 읽어 멤버 카드에 표시` : `모델 ${sel.models}개를 매일 동기화 · 가격 변경 시 알림`}</span>
							</span>
						{/if}
					</aside>
				</div>
			{/if}
		</Dialog.Body>

		<Dialog.Footer note={step === 0 ? '이미 연결된 제공자도 계정 · 키를 하나 더 추가할 수 있어요 (예: 개인 / 팀 계정)' : step === 1 ? (isSub ? '구독 토큰은 실행기(CLI)가 보관해요 · OrchStack은 한도만 읽어요' : '키는 이 기기 키체인에만 저장돼요 · 팀원에게는 연결 이름만 공유') : '언어 · 알림 기본값은 설정 › 일반 · 알림에서 바꿀 수 있어요'}>
			{#if step === 0}
				<Button variant="ghost" size="sm" onclick={() => (open = false)}>취소</Button>
				<Button size="sm" disabled={!sel} onclick={() => (step = 1)}><ArrowRight />다음 · 인증</Button>
			{:else if step === 1}
				<Button variant="ghost" size="sm" onclick={() => (step = 0)}><ArrowLeft />이전</Button>
				<Button size="sm" disabled={!authed} onclick={toScope}><ArrowRight />다음 · 사용 범위</Button>
			{:else}
				<Button variant="ghost" size="sm" onclick={() => (step = 1)}><ArrowLeft />이전</Button>
				<Button size="sm" onclick={add}><Check />연결 추가</Button>
			{/if}
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
