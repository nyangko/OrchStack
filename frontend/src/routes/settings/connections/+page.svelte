<script lang="ts">
	/// Settings › 모델 연결 — 연결 목록 · 폴백 체인 · 이번 달 비용 (.pen Settings · 모델 연결 · 오류 · 한도 소진 상태).
	/// 연결 추가 · 로그인 · 키 갱신은 인트로와 같은 연결 추가 다이얼로그.
	/// 서버 모드(A-4 #95): 목록 · 한도는 GET /connections · /quotas, 추가는 POST /connections. 폴백 체인 · 비용은 서버 계약이 달라(#60) 화면 상태.
	import { onDestroy, onMount, type Component } from 'svelte';
	import Activity from '@lucide/svelte/icons/activity';
	import Plus from '@lucide/svelte/icons/plus';
	import Settings2 from '@lucide/svelte/icons/settings-2';
	import LogIn from '@lucide/svelte/icons/log-in';
	import RefreshCw from '@lucide/svelte/icons/refresh-cw';
	import CircleCheck from '@lucide/svelte/icons/circle-check';
	import CirclePlus from '@lucide/svelte/icons/circle-plus';
	import TriangleAlert from '@lucide/svelte/icons/triangle-alert';
	import OctagonX from '@lucide/svelte/icons/octagon-x';
	import KeyRound from '@lucide/svelte/icons/key-round';
	import LoaderCircle from '@lucide/svelte/icons/loader-circle';
	import GripVertical from '@lucide/svelte/icons/grip-vertical';
	import ArrowDown from '@lucide/svelte/icons/arrow-down';
	import BadgeCheck from '@lucide/svelte/icons/badge-check';
	import Route from '@lucide/svelte/icons/route';
	import HardDrive from '@lucide/svelte/icons/hard-drive';
	import Layers from '@lucide/svelte/icons/layers';
	import Info from '@lucide/svelte/icons/info';
	import Cpu from '@lucide/svelte/icons/cpu';
	import { DropdownMenu, DropdownMenuTrigger, DropdownMenuContent, DropdownMenuRadioGroup, DropdownMenuRadioItem } from '$lib/components/ui/dropdown-menu';
	import { tierTone } from '$lib/components/orch/diagram/sub-run-node.svelte';
	import { Card, CardHeader, CardTitle, CardAction, CardContent } from '$lib/components/ui/card';
	import { Alert, AlertTitle, AlertDescription } from '$lib/components/ui/alert';
	import { Button } from '$lib/components/ui/button';
	import { PageHeader } from '$lib/components/orch/page-header';
	import { Toggle } from '$lib/components/ui/toggle';
	import { Pill } from '$lib/components/orch/pill';
	import { Progress } from '$lib/components/ui/progress';
	import { RuntimeLogo, type Runtime } from '$lib/components/orch/runtime-logo';
	import { AddConnectionDialog, providerMark, type AddedConnection } from '$lib/components/orch/connection';
	import { connections, fallbackChains, monthCost, type Connection, type ProviderKind, type Quota, type SubRunTier } from '$lib/mock';
	import { Empty, EmptyHeader, EmptyMedia, EmptyTitle, EmptyDescription, EmptyContent } from '$lib/components/ui/empty';
	import { api } from '$lib/api/client';
	import { useMock } from '$lib/api/env';
	import type { components } from '$lib/api/schema';
	import { langCode } from '$lib/lang';
	
	type Filter = '전체' | '구독 · 플랜' | 'API 키' | '게이트웨이' | '로컬';
	/// 한도 문제 알림 — 점검 결과로 생긴다.
	type Notice = { key: string; title: string; desc: string };

	let list = $state(useMock ? structuredClone(connections) : []);
	let loadState = $state<'loading' | 'ready' | 'error'>(useMock ? 'ready' : 'loading');
	let chains = $state(structuredClone(fallbackChains));
	let chainOf = $state<Runtime>('claude');
	let filter = $state<Filter>('전체');
	let notices = $state<Notice[]>([]);
	let checking = $state(false);
	let addOpen = $state(false);
	let addKey = $state<string>();
	/// 서버 모드에서 키 갱신 · 로그인으로 연 기존 연결 — 있으면 새로 만들지 않고 고친다.
	let addSn = $state<number>();
	let dragFrom = -1;
	let timer: ReturnType<typeof setTimeout> | undefined;
	onDestroy(() => clearTimeout(timer));

	const filters: Filter[] = ['전체', '구독 · 플랜', 'API 키', '게이트웨이', '로컬'];
	const inFilter = (c: Connection, f: Filter) => f === '전체' || (f === '구독 · 플랜' ? c.kind === '구독' || c.kind === '플랜' : c.kind === f);
	const isOn = (c: Connection) => c.state !== 'login' && c.state !== 'open';
	const groups = $derived([
		{ title: '연결됨', items: list.filter((c) => isOn(c) && inFilter(c, filter)) },
		{ title: '로그인 필요 · 연결 가능', items: list.filter((c) => !isOn(c) && inFilter(c, filter)) }
	]);

	const kindPill: Record<ProviderKind, string> = {
		구독: 'bg-primary-soft text-primary',
		플랜: 'bg-primary-soft text-primary',
		'API 키': 'bg-review-soft text-status-review',
		게이트웨이: 'bg-info-soft text-info',
		로컬: ''
	};
	const stateMeta: Record<Connection['state'], { icon: Component; tone: string }> = {
		ok: { icon: CircleCheck, tone: 'text-status-done' },
		low: { icon: TriangleAlert, tone: 'text-status-blocked' },
		exhausted: { icon: OctagonX, tone: 'text-status-blocked' },
		expired: { icon: KeyRound, tone: 'text-status-blocked' },
		login: { icon: LogIn, tone: 'text-info' },
		open: { icon: CirclePlus, tone: 'text-muted-foreground' }
	};
	const costIcon = [BadgeCheck, KeyRound, Route];
	const total = monthCost.reduce((n, c) => n + c.used, 0);

	// ---- 서버 모드: 서버 행 → 화면 줄 (Connection 모양)
	type ApiConnection = components['schemas']['Connection'];
	type ApiQuota = components['schemas']['Quota'];
	const kindCode: Record<ProviderKind, string> = { 구독: 'subscription', 플랜: 'plan', 'API 키': 'api_key', 게이트웨이: 'gateway', 로컬: 'local' };
	const kindOf = (code: string) => (Object.keys(kindCode) as ProviderKind[]).find((k) => kindCode[k] === code) ?? '로컬';
	const periodLabel: Record<string, string> = { minute: '분', '5h': '5H', day: '일', week: '주간', month: '월' };
	const statusNote: Record<string, string> = { connected: '연결됨', checking: '확인 중', login_required: '로그인 필요', expired: '키 · 토큰 만료', error: '연결 오류', available: '연결 가능' };
	const stateOf: Record<string, Connection['state']> = { connected: 'ok', checking: 'ok', login_required: 'login', expired: 'expired', error: 'exhausted', available: 'open' };

	/// 한도 1건 → 칩. 달러는 사용액/한도, 나머지는 남은 비율(모르면 칩 없음).
	function quotaOf(q: ApiQuota): Quota | undefined {
		const label = periodLabel[q.period] ?? q.period;
		if (q.unit === 'usd' && q.limit_value != null) return { label, used: q.used_value, limit: q.limit_value };
		const pct = q.remain_percent ?? (q.unit === 'percent' ? 100 - q.used_value : q.limit_value ? Math.round(100 - (q.used_value / q.limit_value) * 100) : undefined);
		return pct === undefined ? undefined : { label, pct: Math.max(0, pct) };
	}

	function view(c: ApiConnection, qs: ApiQuota[]): Connection {
		const quotas = qs.map(quotaOf).filter((q): q is Quota => !!q);
		const low = c.status === 'connected' && quotas.some((q) => q.pct !== undefined && q.pct < 20);
		return { sn: c.sn, key: c.provider_code, name: c.provider_name, kind: kindOf(c.kind), state: low ? 'low' : (stateOf[c.status] ?? 'open'), note: c.status_message ?? statusNote[c.status] ?? c.status, quotas };
	}

	/// 연결 목록 + 연결별 한도를 읽는다. 한도를 못 읽은 연결은 칩 없이.
	async function load() {
		loadState = 'loading';
		const { data } = await api.GET('/connections').catch(() => ({ data: undefined }));
		if (!data) return void (loadState = 'error');
		const qs = await Promise.all(data.map((c) => api.GET('/connections/{sn}/quotas', { params: { path: { sn: c.sn } } }).then((r) => r.data ?? []).catch(() => [])));
		list = data.map((c, i) => view(c, qs[i]));
		loadState = 'ready';
	}
	onMount(() => {
		if (!useMock) void load();
	});

	/// 전체 연결 점검 — 한도를 새로 읽고 키를 시험한다. 서버 모드는 다시 읽기만(점검 · 한도 갱신은 실행기 #13).
	/// 목데이터: Codex 주간 소진 · Anthropic API 키 만료가 발견된다.
	async function checkAll() {
		if (!useMock) {
			checking = true;
			await load();
			checking = false;
			return;
		}
		checking = true;
		timer = setTimeout(() => {
			const codex = list.find((c) => c.key === 'chatgpt');
			if (codex) {
				codex.state = 'exhausted';
				codex.note = '주간 한도 소진 · 폴백 중';
				codex.quotas = codex.quotas.map((q) => (q.label === '주간' ? { ...q, pct: 0 } : q));
			}
			const api = list.find((c) => c.key === 'anthropic-api');
			if (api) {
				api.state = 'expired';
				api.note = '키 만료 · 401 · 폴백에서 제외';
			}
			notices = [
				{ key: 'chatgpt', title: 'Codex · ChatGPT Pro 주간 한도 소진', desc: '진 · 유나의 Run이 폴백 2순위 Anthropic · Max로 넘어갔어요 · 화 09:00 리셋' },
				{ key: 'anthropic-api', title: 'Anthropic API · prod 키 만료 (401)', desc: '폴백 체인에서 자동 제외됨 · 방금 · 키를 갱신하면 다시 포함돼요' }
			];
			checking = false;
		}, 1200);
	}

	function openAdd(key?: string, sn?: number) {
		addKey = key;
		addSn = sn;
		addOpen = true;
	}

	/// 서버 모드 추가 — 키 원문은 보내지 않는다(키체인 저장은 실행기 Task) · 끝 4자리만. 실패하면 false(창 유지).
	async function addOnServer(a: AddedConnection): Promise<boolean> {
		const { data: runtimes } = a.provider.cli ? await api.GET('/runtimes').catch(() => ({ data: undefined })) : { data: undefined };
		const body = {
			provider_name: a.title, name: a.name, plan_name: a.provider.plan,
			runtime_sn: runtimes?.find((r) => r.name === a.provider.cli)?.sn, login_method: a.method === 'device' ? 'device_code' : a.method,
			base_url: a.baseUrl || undefined, key_hint: a.keyHint || undefined, monthly_budget_usd_micro: a.budget ? a.budget * 1_000_000 : undefined,
			scope: a.scope, report_language: langCode[a.lang] ?? a.lang, commit_language: langCode[a.commitLang] ?? a.commitLang
		};
		const res = await (addSn
			? api.PATCH('/connections/{sn}', { params: { path: { sn: addSn } }, body })
			: api.POST('/connections', { body: { ...body, kind: kindCode[a.provider.kind], provider_code: a.provider.key } })
		).catch(() => undefined);
		if (!res || res.error) return false;
		await load();
		return true;
	}

	/// 다이얼로그에서 연결을 마치면 목록에 반영 — 있던 연결은 정상으로, 없으면 새 줄.
	function onAdded(a: AddedConnection) {
		if (!useMock) return addOnServer(a);
		const c = list.find((x) => x.key === a.provider.key);
		if (c) {
			c.state = 'ok';
			c.note = a.note;
		} else {
			list.push({ key: a.provider.key, name: a.title, kind: a.provider.kind, state: 'ok', note: a.note, quotas: [] });
		}
		notices = notices.filter((n) => n.key !== a.provider.key);
	}

	/// 폴백 순서 바꾸기 (끌기 · 방향키 공용).
	function move(from: number, to: number) {
		const steps = chains[chainOf];
		if (to < 0 || to >= steps.length || from === to) return;
		const [s] = steps.splice(from, 1);
		steps.splice(to, 0, s);
	}
</script>

<svelte:head><title>모델 연결 · Settings · OrchStack</title></svelte:head>

<!-- 제공자 로고 · 아이콘 -->
{#snippet mark(key: string, size: string)}
	{@const M = providerMark[key]}
	{#if typeof M === 'string'}<RuntimeLogo runtime={M} class={[size, 'ring-0']} />{:else if M}<M class={size} />{/if}
{/snippet}

<main class="page-main">
	<PageHeader title="모델 연결" desc="에이전트가 모델을 부르는 경로예요. 실행기(CLI)마다 구독 로그인 · API 키 · 게이트웨이 · 로컬 모델을 연결할 수 있어요." status={false}>
		<Button variant="outline" disabled={checking} onclick={checkAll}>
			{#if checking}<LoaderCircle class="animate-spin" />점검 중…{:else}<Activity />전체 연결 점검{/if}
		</Button>
		<Button onclick={() => openAdd()}><Plus />연결 추가</Button>
	</PageHeader>

	{#if notices.length}
		<div class="flex gap-2" role="status">
			{#each notices as n (n.key)}
				{@const expired = list.find((c) => c.key === n.key)?.state === 'expired'}
				<Alert variant={expired ? 'destructive' : 'warning'}>
					{#if expired}<KeyRound />{:else}<TriangleAlert />{/if}
					<AlertTitle>{n.title}</AlertTitle>
					<AlertDescription>{n.desc}</AlertDescription>
				</Alert>
			{/each}
		</div>
	{/if}

	<div class="flex gap-1.5" role="group" aria-label="종류">
		{#each filters as f (f)}
			<Toggle variant="chip" count={list.filter((c) => inFilter(c, f)).length} bind:pressed={() => filter === f, (v) => { if (v) filter = f; }}>{f}</Toggle>
		{/each}
	</div>

	{#if loadState !== 'ready'}
		<!-- 서버 모드 불러오기 -->
		<Empty class="card py-16 rounded-lg">
			<EmptyHeader>
				{#if loadState === 'error'}
					<EmptyTitle>연결을 불러오지 못했어요</EmptyTitle>
					<EmptyDescription>서버 연결을 확인하고 다시 시도하세요.</EmptyDescription>
				{:else}
					<EmptyMedia variant="icon"><LoaderCircle class="animate-spin" /></EmptyMedia>
					<EmptyTitle>연결을 불러오는 중…</EmptyTitle>
				{/if}
			</EmptyHeader>
			{#if loadState === 'error'}<EmptyContent><Button variant="outline" onclick={load}>다시 시도</Button></EmptyContent>{/if}
		</Empty>
	{/if}

	{#each groups as g (g.title)}
		{#if g.items.length}
			<section class="flex flex-col gap-2.5">
				<h2 class="label-xs-strong text-muted-foreground">{g.title}<span class="font-mono text-subtle-foreground">{g.items.length}</span></h2>
				<div class="grid grid-cols-2 gap-2.5">
					{#each g.items as c (c.sn ?? c.key)}
						{@const st = stateMeta[c.state]}
						<div class={['flex items-center rounded-md border gap-3 px-4 py-3.5', isOn(c) ? 'bg-card' : 'bg-muted']}>
							<span class="icon-tile size-9.5">{@render mark(c.key, 'size-5')}</span>
							<span class="flex min-w-0 flex-1 flex-col gap-1.25">
								<span class="label-xs-strong">{c.name}<Pill class={kindPill[c.kind]}>{c.kind}</Pill></span>
								<span class={['meta-truncate gap-1.25', st.tone]}><st.icon class="size-3 shrink-0" />{c.note}</span>
							</span>
							{#each c.quotas as q (q.label)}
								{@const money = q.limit !== undefined}
								{@const v = money ? ((q.used ?? 0) / q.limit!) * 100 : (q.pct ?? 0)}
								{@const tone = money ? 'text-primary' : v < 20 ? 'text-status-blocked' : 'text-status-done'}
								<span class="flex shrink-0 items-center gap-1.5 rounded-sm border bg-card px-2 py-0.75">
									<span class="text-2xs font-semibold text-muted-foreground">{q.label}</span>
									<Progress value={v} class="h-1 w-8 bg-muted" indicator={money ? 'bg-primary' : v < 20 ? 'bg-status-blocked' : 'bg-status-done'} aria-label="{c.name} {q.label}" />
									<span class={['font-mono text-caption font-semibold', tone]}>{money ? `$${q.used}/$${q.limit}` : `${q.pct}%`}</span>
								</span>
							{/each}
							{#if c.state === 'expired'}
								<Button variant="ghost" size="sm" onclick={() => openAdd(c.key, c.sn)}><RefreshCw />키 갱신</Button>
							{:else if c.state === 'login'}
								<Button variant="ghost" size="sm" onclick={() => openAdd(c.key, c.sn)}><LogIn />로그인</Button>
							{:else if c.state === 'open'}
								<Button variant="ghost" size="sm" onclick={() => openAdd(c.key, c.sn)}><Plus />{c.action ?? '연결'}</Button>
							{:else}
								<!-- 연결 관리 화면은 .pen에 아직 없음 -->
								<Button variant="ghost" size="sm"><Settings2 />관리</Button>
							{/if}
						</div>
					{/each}
				</div>
			</section>
		{/if}
	{/each}

	<div class="flex items-start gap-5">
		<Card size="sm" class="min-w-0 flex-1">
			<CardHeader>
				<CardTitle>폴백 체인 · {chainOf === 'claude' ? 'Claude Code' : 'Codex CLI'}</CardTitle>
				<CardAction>
					<Button variant="link" size="xs" onclick={() => (chainOf = chainOf === 'claude' ? 'codex' : 'claude')}>
						{chainOf === 'claude' ? 'Codex CLI' : 'Claude Code'} 체인 보기
					</Button>
				</CardAction>
			</CardHeader>
			<CardContent class="flex flex-col gap-1.5">
				<!-- 등급은 리드가 아니라 규칙 엔진이 정한다 (#67) — 이 목록은 등급별로 어느 연결 · 모델을 쓸지만 정한다 -->
				<div class="flex items-center gap-2 rounded-md bg-muted px-2.5 py-2 text-xs font-medium"><Info class="size-3.5 shrink-0 text-muted-foreground" />리드는 모델을 고르지 않습니다. 작업 종류로 등급이 정해지고, 이 목록에서 위부터 시도합니다.</div>
				<p class="py-1 text-xs text-muted-foreground">{chainOf === 'claude' ? 'Claude Code' : 'Codex CLI'} 실행기 기준 · 행 순서 = 시도 순서 · 끌어서 바꿔요. 같은 연결을 등급만 다르게 여러 번 넣을 수 있어요.</p>
				<ol class="flex flex-col gap-1.5">
					{#each chains[chainOf] as s, i (s.name)}
						{@const off = list.find((c) => c.key === s.key)?.state === 'expired'}
						{#if i}<li class="flex pl-8 text-subtle-foreground" aria-hidden="true"><ArrowDown class="size-3.5" /></li>{/if}
						<li
							draggable="true"
							ondragstart={() => (dragFrom = i)}
							ondragover={(e) => e.preventDefault()}
							ondrop={() => move(dragFrom, i)}
							class={['card flex items-center gap-2.5 px-3 py-2.5 rounded-md', off && 'opacity-60']}
						>
							<button
								type="button"
								class="focus-ring cursor-grab rounded-sm text-subtle-foreground"
								aria-label="{s.name} 순서 · 위아래 방향키로 이동"
								onkeydown={(e) => {
									if (e.key === 'ArrowUp' || e.key === 'ArrowDown') {
										e.preventDefault();
										move(i, e.key === 'ArrowUp' ? i - 1 : i + 1);
									}
								}}
							>
								<GripVertical class="size-3.25" />
							</button>
							<span class="flex size-5 shrink-0 items-center justify-center rounded-full bg-foreground text-caption font-semibold text-background">{i + 1}</span>
							<span class="flex items-center justify-center size-6 shrink-0">{@render mark(s.key, 'size-5')}</span>
							<span class="flex min-w-0 flex-1 flex-col gap-0.5">
								<span class="label-xs">{s.name}<Pill class={kindPill[s.kind]}>{s.kind}</Pill></span>
								<span class="truncate text-caption text-muted-foreground">{off ? '키 만료 · 폴백에서 제외됨' : s.cond}</span>
							</span>
							<DropdownMenu>
								<DropdownMenuTrigger>
									{#snippet child({ props })}
										<button {...props} type="button" aria-label="{s.name} 등급" class={['flex shrink-0 items-center gap-1 rounded-xs px-1.5 py-px text-2xs font-bold outline-none focus-visible:ring-3 focus-visible:ring-ring/50', s.tier ? `font-mono ${tierTone[s.tier]}` : 'bg-muted text-muted-foreground']}>
											{#if s.tier}<Cpu class="size-2.5" />{s.tier}{:else}<Layers class="size-2.5" />모든 등급{/if}
										</button>
									{/snippet}
								</DropdownMenuTrigger>
								<DropdownMenuContent align="end" class="w-36">
									<DropdownMenuRadioGroup value={s.tier ?? 'all'} onValueChange={(v) => (s.tier = v === 'all' ? undefined : (v as SubRunTier))}>
										{#each [['all', '모든 등급'], ['S', 'S · 소형'], ['M', 'M · 중형'], ['L', 'L · 대형']] as [v, l] (v)}<DropdownMenuRadioItem value={v}>{l}</DropdownMenuRadioItem>{/each}
									</DropdownMenuRadioGroup>
								</DropdownMenuContent>
							</DropdownMenu>
							<span class="w-14 text-right text-caption text-subtle-foreground">{s.cost}</span>
						</li>
					{/each}
				</ol>
			</CardContent>
		</Card>

		<Card size="sm" class="w-105 shrink-0">
			<CardHeader>
				<CardTitle>이번 달 비용</CardTitle>
				<!-- 비용 상세 화면은 .pen에 아직 없음 -->
				<CardAction><span class="text-xs font-medium text-primary">상세</span></CardAction>
			</CardHeader>
			<CardContent class="flex flex-col gap-1.5">
				{#each monthCost as c, i (c.label)}
					{@const Icon = costIcon[i]}
					<div class="flex flex-col gap-1.5 py-1.5">
						<span class="flex items-center gap-2 text-xs">
							<Icon class="size-3.25 text-muted-foreground" />
							<span class="flex-1 text-muted-foreground">{c.label}</span>
							<span class="font-mono font-medium">${c.used}</span>
							<span class="text-subtle-foreground"><span class="font-mono">/ {c.limit ? `$${c.limit}` : ''}</span>{c.limit ? '' : '월'}</span>
						</span>
						<Progress value={c.limit ? (c.used / c.limit) * 100 : 100} class="h-1.5 bg-muted" indicator={c.limit ? 'bg-primary' : 'bg-muted-foreground'} aria-label="{c.label} 사용액" />
						{#if c.note}<span class="text-caption text-muted-foreground">{c.note}</span>{/if}
					</div>
				{/each}
				<span class="value-line">
					<HardDrive class="size-3.25 text-muted-foreground" />
					<span class="flex-1 text-muted-foreground">로컬</span>
					<span class="font-medium"><span class="font-mono">$0</span> · 비용 없음</span>
				</span>
				<span class="flex items-center border-t pt-2 text-xs">
					<span class="flex-1 text-muted-foreground">이번 달 합계</span>
					<span class="font-mono text-sm font-semibold">${total.toFixed(1)}</span>
				</span>
			</CardContent>
		</Card>
	</div>
</main>

<AddConnectionDialog bind:open={addOpen} provider={addKey} onadd={onAdded} />
