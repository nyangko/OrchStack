<script lang="ts">
	/// Settings › 모델 연결 — 연결 목록 · 폴백 체인 · 이번 달 비용 (.pen Settings · 모델 연결 · 오류 · 한도 소진 상태).
	/// 연결 추가 · 로그인 · 키 갱신은 인트로와 같은 연결 추가 다이얼로그. 점검 · 한도는 목데이터 (서버 연결 #47).
	import { onDestroy, type Component } from 'svelte';
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
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
	import { tierTone } from '$lib/components/orch/diagram/sub-run-node.svelte';
	import * as Card from '$lib/components/ui/card';
	import * as Alert from '$lib/components/ui/alert';
	import { Button } from '$lib/components/ui/button';
	import { PageHeader } from '$lib/components/ui/page-header';
	import { Toggle } from '$lib/components/ui/toggle';
	import { Pill } from '$lib/components/ui/pill';
	import { Progress } from '$lib/components/ui/progress';
	import { RuntimeLogo, type Runtime } from '$lib/components/ui/runtime-logo';
	import { AddConnectionDialog, providerMark, type AddedConnection } from '$lib/components/orch/connection';
	import { connections, fallbackChains, monthCost, type Connection, type ProviderKind, type SubRunTier } from '$lib/mock';
	import { cn } from '$lib/utils';

	type Filter = '전체' | '구독 · 플랜' | 'API 키' | '게이트웨이' | '로컬';
	/// 한도 문제 알림 — 점검 결과로 생긴다.
	type Notice = { key: string; title: string; desc: string };

	let list = $state(structuredClone(connections));
	let chains = $state(structuredClone(fallbackChains));
	let chainOf = $state<Runtime>('claude');
	let filter = $state<Filter>('전체');
	let notices = $state<Notice[]>([]);
	let checking = $state(false);
	let addOpen = $state(false);
	let addKey = $state<string>();
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

	/// 전체 연결 점검 — 한도를 새로 읽고 키를 시험한다. 목데이터: Codex 주간 소진 · Anthropic API 키 만료가 발견된다.
	function checkAll() {
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

	function openAdd(key?: string) {
		addKey = key;
		addOpen = true;
	}

	/// 다이얼로그에서 연결을 마치면 목록에 반영 — 있던 연결은 정상으로, 없으면 새 줄.
	function onAdded(a: AddedConnection) {
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
	{#if typeof M === 'string'}<RuntimeLogo runtime={M} class={cn(size, 'ring-0')} />{:else if M}<M class={size} />{/if}
{/snippet}

<main class="flex flex-col gap-5 px-8 py-7">
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
				<Alert.Root variant={expired ? 'destructive' : 'warning'}>
					{#if expired}<KeyRound />{:else}<TriangleAlert />{/if}
					<Alert.Title>{n.title}</Alert.Title>
					<Alert.Description>{n.desc}</Alert.Description>
				</Alert.Root>
			{/each}
		</div>
	{/if}

	<div class="flex gap-1.5" role="group" aria-label="종류">
		{#each filters as f (f)}
			<Toggle variant="chip" count={list.filter((c) => inFilter(c, f)).length} bind:pressed={() => filter === f, (v) => { if (v) filter = f; }}>{f}</Toggle>
		{/each}
	</div>

	{#each groups as g (g.title)}
		{#if g.items.length}
			<section class="flex flex-col gap-2.5">
				<h2 class="flex items-center gap-1.5 text-xs font-semibold text-muted-foreground">{g.title}<span class="font-mono text-subtle-foreground">{g.items.length}</span></h2>
				<div class="grid grid-cols-2 gap-2.5">
					{#each g.items as c (c.key)}
						{@const st = stateMeta[c.state]}
						<div class={cn('flex items-center gap-3 rounded-md border px-4 py-3.5', isOn(c) ? 'bg-card' : 'bg-muted')}>
							<span class="flex size-9.5 shrink-0 items-center justify-center rounded-md bg-muted">{@render mark(c.key, 'size-5')}</span>
							<span class="flex min-w-0 flex-1 flex-col gap-1.25">
								<span class="flex items-center gap-1.5 text-xs font-semibold">{c.name}<Pill class={kindPill[c.kind]}>{c.kind}</Pill></span>
								<span class={cn('flex items-center gap-1.25 truncate text-caption', st.tone)}><st.icon class="size-3 shrink-0" />{c.note}</span>
							</span>
							{#each c.quotas as q (q.label)}
								{@const money = q.limit !== undefined}
								{@const v = money ? ((q.used ?? 0) / q.limit!) * 100 : (q.pct ?? 0)}
								{@const tone = money ? 'text-primary' : v < 20 ? 'text-status-blocked' : 'text-status-done'}
								<span class="flex shrink-0 items-center gap-1.5 rounded-sm border bg-card px-2 py-0.75">
									<span class="text-2xs font-semibold text-muted-foreground">{q.label}</span>
									<Progress value={v} class="h-1 w-8 bg-muted" indicator={money ? 'bg-primary' : v < 20 ? 'bg-status-blocked' : 'bg-status-done'} aria-label="{c.name} {q.label}" />
									<span class={cn('font-mono text-caption font-semibold', tone)}>{money ? `$${q.used}/$${q.limit}` : `${q.pct}%`}</span>
								</span>
							{/each}
							{#if c.state === 'expired'}
								<Button variant="ghost" size="sm" onclick={() => openAdd(c.key)}><RefreshCw />키 갱신</Button>
							{:else if c.state === 'login'}
								<Button variant="ghost" size="sm" onclick={() => openAdd(c.key)}><LogIn />로그인</Button>
							{:else if c.state === 'open'}
								<Button variant="ghost" size="sm" onclick={() => openAdd(c.key)}><Plus />{c.action ?? '연결'}</Button>
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
		<Card.Root size="sm" class="min-w-0 flex-1">
			<Card.Header>
				<Card.Title>폴백 체인 · {chainOf === 'claude' ? 'Claude Code' : 'Codex CLI'}</Card.Title>
				<Card.Action>
					<Button variant="link" size="xs" onclick={() => (chainOf = chainOf === 'claude' ? 'codex' : 'claude')}>
						{chainOf === 'claude' ? 'Codex CLI' : 'Claude Code'} 체인 보기
					</Button>
				</Card.Action>
			</Card.Header>
			<Card.Content class="flex flex-col gap-1.5">
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
							class={cn('flex items-center gap-2.5 rounded-md border bg-card px-3 py-2.5', off && 'opacity-60')}
						>
							<button
								type="button"
								class="cursor-grab rounded-sm text-subtle-foreground outline-none focus-visible:ring-3 focus-visible:ring-ring/50"
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
							<span class="flex size-6 shrink-0 items-center justify-center">{@render mark(s.key, 'size-5')}</span>
							<span class="flex min-w-0 flex-1 flex-col gap-0.5">
								<span class="flex items-center gap-1.5 text-xs font-medium">{s.name}<Pill class={kindPill[s.kind]}>{s.kind}</Pill></span>
								<span class="truncate text-caption text-muted-foreground">{off ? '키 만료 · 폴백에서 제외됨' : s.cond}</span>
							</span>
							<DropdownMenu.Root>
								<DropdownMenu.Trigger>
									{#snippet child({ props })}
										<button {...props} type="button" aria-label="{s.name} 등급" class={cn('flex shrink-0 items-center gap-1 rounded-xs px-1.5 py-px text-2xs font-bold outline-none focus-visible:ring-3 focus-visible:ring-ring/50', s.tier ? `font-mono ${tierTone[s.tier]}` : 'bg-muted text-muted-foreground')}>
											{#if s.tier}<Cpu class="size-2.5" />{s.tier}{:else}<Layers class="size-2.5" />모든 등급{/if}
										</button>
									{/snippet}
								</DropdownMenu.Trigger>
								<DropdownMenu.Content align="end" class="w-36">
									<DropdownMenu.RadioGroup value={s.tier ?? 'all'} onValueChange={(v) => (s.tier = v === 'all' ? undefined : (v as SubRunTier))}>
										{#each [['all', '모든 등급'], ['S', 'S · 소형'], ['M', 'M · 중형'], ['L', 'L · 대형']] as [v, l] (v)}<DropdownMenu.RadioItem value={v}>{l}</DropdownMenu.RadioItem>{/each}
									</DropdownMenu.RadioGroup>
								</DropdownMenu.Content>
							</DropdownMenu.Root>
							<span class="w-14 text-right text-caption text-subtle-foreground">{s.cost}</span>
						</li>
					{/each}
				</ol>
			</Card.Content>
		</Card.Root>

		<Card.Root size="sm" class="w-105 shrink-0">
			<Card.Header>
				<Card.Title>이번 달 비용</Card.Title>
				<!-- 비용 상세 화면은 .pen에 아직 없음 -->
				<Card.Action><span class="text-xs font-medium text-primary">상세</span></Card.Action>
			</Card.Header>
			<Card.Content class="flex flex-col gap-1.5">
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
				<span class="flex h-7 items-center gap-2 text-xs">
					<HardDrive class="size-3.25 text-muted-foreground" />
					<span class="flex-1 text-muted-foreground">로컬</span>
					<span class="font-medium"><span class="font-mono">$0</span> · 비용 없음</span>
				</span>
				<span class="flex items-center border-t pt-2 text-xs">
					<span class="flex-1 text-muted-foreground">이번 달 합계</span>
					<span class="font-mono text-sm font-semibold">${total.toFixed(1)}</span>
				</span>
			</Card.Content>
		</Card.Root>
	</div>
</main>

<AddConnectionDialog bind:open={addOpen} provider={addKey} onadd={onAdded} />
