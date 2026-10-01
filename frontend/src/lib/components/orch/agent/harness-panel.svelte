<script lang="ts">
	/// 에이전트 실행 방식 (.pen Harness 기본값 · 모델 선택). 실행기 · 모델은 템플릿/멤버 값을 받고 바꾸면 onchange로 돌려준다.
	/// 나머지(Effort · 폴백 · 한도)는 config.harness를 바로 고친다.
	import CircleCheck from '@lucide/svelte/icons/circle-check';
	import Cpu from '@lucide/svelte/icons/cpu';
	import Gauge from '@lucide/svelte/icons/gauge';
	import Route from '@lucide/svelte/icons/route';
	import Terminal from '@lucide/svelte/icons/terminal';
	import ChevronDown from '@lucide/svelte/icons/chevron-down';
	import ChevronUp from '@lucide/svelte/icons/chevron-up';
	import ChevronsUpDown from '@lucide/svelte/icons/chevrons-up-down';
	import Coins from '@lucide/svelte/icons/coins';
	import BadgeCheck from '@lucide/svelte/icons/badge-check';
	import Wallet from '@lucide/svelte/icons/wallet';
	import PlugZap from '@lucide/svelte/icons/plug-zap';
	import Check from '@lucide/svelte/icons/check';
	import Timer from '@lucide/svelte/icons/timer';
	import Repeat from '@lucide/svelte/icons/repeat';
	import History from '@lucide/svelte/icons/history';
	import Play from '@lucide/svelte/icons/play';
	import Search from '@lucide/svelte/icons/search';
	import Layers from '@lucide/svelte/icons/layers';
	import CircleDot from '@lucide/svelte/icons/circle-dot';
	import Circle from '@lucide/svelte/icons/circle';
	import ThumbsUp from '@lucide/svelte/icons/thumbs-up';
	import * as Card from '$lib/components/ui/card';
	import * as Dialog from '$lib/components/ui/dialog';
	import * as Select from '$lib/components/ui/select';
	import * as InputGroup from '$lib/components/ui/input-group';
	import { Button } from '$lib/components/ui/button';
	import { Toggle } from '$lib/components/ui/toggle';
	import { Pill } from '$lib/components/ui/pill';
	import { Switch } from '$lib/components/ui/switch';
	import { Input } from '$lib/components/ui/input';
	import { Progress } from '$lib/components/ui/progress';
	import { LimitRow } from '$lib/components/ui/limit-row';
	import { RuntimeLogo, type Runtime } from '$lib/components/ui/runtime-logo';
	import { fallbackSteps, meters, modelCatalog, type AgentConfig, type ModelInfo } from '$lib/mock';
	import { accountOf, low, runtimeName } from '$lib/teams.svelte';
	import { cn } from '$lib/utils';

	let {
		config: c,
		base,
		runtime,
		model,
		baseRt,
		onchange,
	}: {
		config: AgentConfig;
		base?: AgentConfig;
		runtime: Runtime;
		model: string;
		/** 원본 템플릿의 실행기 · 모델 (멤버 화면에서 차이 표시). */
		baseRt?: { runtime: Runtime; model: string };
		onchange: (runtime: Runtime, model: string) => void;
	} = $props();

	const gateway = meters.find((m) => m.kind === 'route')!;
	let picker = $state(false);
	let pickQuery = $state('');
	let pickFilter = $state<'추천' | 'Tested' | 'NEW' | 'cheap' | 'long' | 'all'>('all');
	let pickProvider = $state('all');
	let showCmd = $state(false);
	const pickModels = $derived(
		modelCatalog
			.filter((p) => pickProvider === 'all' || p.key === pickProvider)
			.map((p) => {
				let list = p.models.filter((m) => `${m.name} ${m.vendor}`.toLowerCase().includes(pickQuery.trim().toLowerCase()));
				if (pickFilter === '추천' || pickFilter === 'Tested' || pickFilter === 'NEW') list = list.filter((m) => m.tags.includes(pickFilter as ModelInfo['tags'][number]));
				if (pickFilter === 'cheap') list = [...list].sort((a, b) => a.cost - b.cost);
				if (pickFilter === 'long') list = list.filter((m) => m.ctxK >= 400);
				return { ...p, models: list };
			})
			.filter((p) => p.models.length)
	);
	/// Run 한 번이 구독 주간 창에서 쓰는 비율 (%). 실측 전 어림값.
	const perRun = (r: Runtime) => (r === 'codex' ? 1.2 : 0.8);
	/// 실행 명령 미리보기 — 권한 · 한도 설정에서 만든다.
	const command = $derived(
		runtime === 'codex'
			? `codex exec --model ${model} --sandbox ${c.trust >= 3 ? 'workspace-write' : 'read-only'} --ask-for-approval ${c.trust >= 4 ? 'never' : 'on-request'} -c reasoning_effort=${c.harness.effort.toLowerCase()}${c.harness.resume ? ' --resume' : ''}`
			: `claude -p --model ${model} --permission-mode ${c.trust >= 3 ? 'acceptEdits' : 'plan'} --max-turns ${c.harness.maxTurns}${c.harness.resume ? ' --continue' : ''} --allowedTools "Read,Edit${c.trust >= 2 ? ',Bash(pnpm *)' : ''}"`
	);
	const h = $derived(c.harness);
	const acc = $derived(accountOf(runtime));
	const changed = $derived(!!base && (JSON.stringify(h) !== JSON.stringify(base.harness) || runtime !== baseRt?.runtime || model !== baseRt?.model));
	function moveStep(list: string[], i: number, d: -1 | 1) {
		const j = i + d;
		if (j < 0 || j >= list.length) return;
		[list[i], list[j]] = [list[j], list[i]];
	}
</script>

{#snippet sameAs(changed: boolean, base?: AgentConfig)}
	{#if base}
		<Pill class={changed ? 'bg-primary-soft text-primary' : ''}>{changed ? '템플릿과 다름' : '템플릿과 동일'}</Pill>
	{/if}
{/snippet}

<div class="flex items-center gap-2.5">
	<h2 class="text-lg font-semibold">{base ? 'Harness' : 'Harness 기본값'}</h2>
	{@render sameAs(changed, base)}
	<Pill>다음 Run부터 적용</Pill>
	<span class="flex-1"></span>
	<span class="saved-note"><CircleCheck class="size-3" />저장됨</span>
</div>
<div class="flex items-start gap-4">
	<div class="col-fill gap-4">
		<Card.Root size="sm">
			<Card.Header><Card.Title>실행</Card.Title></Card.Header>
			<Card.Content class="gap-3">
				<div class="grid grid-cols-2 gap-2.5">
					<div class="flex flex-col gap-1.5">
						<span class="text-caption font-semibold text-muted-foreground">실행기</span>
						<Select.Root type="single" value={runtime} onValueChange={(v) => onchange(v as Runtime, modelCatalog.find((p) => p.runtime === v)!.models[0].name)}>
							<Select.Trigger class="w-full"><span class="flex items-center gap-2"><RuntimeLogo runtime={runtime} class="size-4 ring-0" />{runtimeName(runtime)}<span class="font-normal text-muted-foreground">로그인됨</span></span></Select.Trigger>
							<Select.Content>{#each ['codex', 'claude'] as const as r (r)}<Select.Item value={r} label={runtimeName(r)} />{/each}</Select.Content>
						</Select.Root>
					</div>
					<div class="flex flex-col gap-1.5">
						<span class="text-caption font-semibold text-muted-foreground">연결</span>
						<div class="field-box h-9">
							<RuntimeLogo runtime={runtime} class="size-4 ring-0" /><span class="truncate">{acc.plan}</span><Pill class="text-2xs">구독</Pill>
							<span class={cn('ml-auto text-caption font-normal whitespace-nowrap', low(acc.week) ? 'text-destructive' : 'text-muted-foreground')}>주간 {acc.week}%</span>
						</div>
					</div>
					<div class="flex flex-col gap-1.5">
						<span class="text-caption font-semibold text-muted-foreground">모델</span>
						<button type="button" onclick={() => ((picker = true), (pickQuery = ''), (pickFilter = 'all'), (pickProvider = 'all'))} class="field-box pressable h-9 text-left">
							<Cpu class="size-3.5 text-muted-foreground" />{model}
							<span class="flex-1 truncate font-normal text-muted-foreground">{modelCatalog.flatMap((p) => p.models).find((m) => m.name === model)?.ctx ?? ''}</span>
							<ChevronsUpDown class="size-3.5 text-muted-foreground" />
						</button>
					</div>
					<div class="flex flex-col gap-1.5">
						<span class="text-caption font-semibold text-muted-foreground">Effort</span>
						<Select.Root type="single" bind:value={h.effort}>
							<Select.Trigger class="w-full"><span class="flex items-center gap-2"><Gauge class="size-3.5 text-muted-foreground" />{h.effort}</span></Select.Trigger>
							<Select.Content>{#each ['Auto', 'Low', 'Medium', 'High'] as e (e)}<Select.Item value={e} label={e} />{/each}</Select.Content>
						</Select.Root>
					</div>
				</div>
				<span class="text-caption text-muted-foreground">한도가 소진된 연결의 모델은 폴백으로만 쓰여요</span>
			</Card.Content>
		</Card.Root>
		<Card.Root size="sm">
			<Card.Header>
				<Card.Title>폴백</Card.Title>
				<Card.Description>첫 연결이 막히면 순서대로 시도</Card.Description>
			</Card.Header>
			<Card.Content class="gap-0">
				{#each h.fallback as key, i (key)}
					{@const st = fallbackSteps[key]}
					<div class="list-row">
						<span class="step-num">{i + 1}</span>
						{#if key === 'sub'}<RuntimeLogo runtime={runtime} class="size-4 ring-0" />{:else if key === 'omni'}<Route class="size-4 text-muted-foreground" />{:else}<Cpu class="size-4 text-muted-foreground" />{/if}
						<span class="row-text">
							<span class="row-title">{key === 'sub' ? acc.plan : 'name' in st ? st.name : ''}<Pill class="text-2xs">{st.kind}</Pill></span>
							<span class="text-xs text-muted-foreground">{st.desc}</span>
						</span>
						<span class="font-mono text-caption text-muted-foreground">{st.cost}</span>
						<span class="flex">
							<Button variant="ghost" size="icon-xs" aria-label="위로" disabled={i === 0} onclick={() => moveStep(h.fallback, i, -1)}><ChevronUp /></Button>
							<Button variant="ghost" size="icon-xs" aria-label="아래로" disabled={i === h.fallback.length - 1} onclick={() => moveStep(h.fallback, i, 1)}><ChevronDown /></Button>
						</span>
					</div>
				{/each}
			</Card.Content>
		</Card.Root>
		<div class="box-col rounded-md bg-card">
			<button type="button" aria-expanded={showCmd} onclick={() => (showCmd = !showCmd)} class="harness-preview-toggle">
				<Terminal class="size-3.5 text-muted-foreground" /><span class="flex-1">실행 명령 미리보기</span><ChevronDown class={cn('size-4 text-muted-foreground transition-transform', showCmd && 'rotate-180')} />
			</button>
			{#if showCmd}<pre class="harness-preview-code">{command}</pre>{/if}
		</div>
	</div>
	<aside class="aside-col w-72 gap-4">
		<Card.Root size="sm">
			<Card.Header><Card.Title>이 조합으로 실행하면</Card.Title></Card.Header>
			<Card.Content class="gap-1">
				<div class="value-line"><Coins class="size-3.25 text-muted-foreground" /><span class="flex-1 text-muted-foreground">Run당 평균</span><span class="font-mono font-medium">35K tok</span></div>
				<LimitRow icon={BadgeCheck} label="구독 주간 창 사용" used="{perRun(runtime)}%" max="/ Run" value={perRun(runtime) * 10} note="주간 잔량 {acc.week}% → 약 {Math.floor(acc.week / perRun(runtime))} Run 가능" warn={low(acc.week)} />
				<div class="value-line"><Wallet class="size-3.25 text-muted-foreground" /><span class="flex-1 text-muted-foreground">폴백 시 비용</span><span class="font-medium">{h.fallback[0] === 'sub' ? 'OmniRoute · Run당 ~$0.4' : '첫 단계부터 과금'}</span></div>
				<div class="value-line"><PlugZap class="size-3.25 text-muted-foreground" /><span class="flex-1 text-muted-foreground">호환성</span><Pill class="bg-success-soft text-status-done"><Check />모두 호환</Pill></div>
			</Card.Content>
		</Card.Root>
		<Card.Root size="sm">
			<Card.Header><Card.Title>실행 한도</Card.Title></Card.Header>
			<Card.Content class="gap-1.5 text-xs">
				<div class="flex items-center gap-2">
					<Timer class="size-3.25 text-muted-foreground" /><span class="flex-1 text-muted-foreground">Run 최대 시간</span>
					<Select.Root type="single" bind:value={h.maxTime}>
						<Select.Trigger size="sm" class="h-7 w-24">{h.maxTime}</Select.Trigger>
						<Select.Content>{#each ['30m', '1h', '2h', '4h'] as t (t)}<Select.Item value={t} label={t} />{/each}</Select.Content>
					</Select.Root>
				</div>
				<label class="flex items-center gap-2">
					<Repeat class="size-3.25 text-muted-foreground" /><span class="flex-1 text-muted-foreground">최대 turn</span>
					<Input type="number" min={10} max={500} bind:value={h.maxTurns} class="h-7 w-24 text-xs" />
				</label>
				<label class="flex h-7 items-center gap-2">
					<History class="size-3.25 text-muted-foreground" /><span class="flex-1 text-muted-foreground">세션 resume · 태스크 단위</span><Switch bind:checked={h.resume} />
				</label>
				<label class="flex h-7 items-center gap-2">
					<Play class="size-3.25 text-muted-foreground" /><span class="flex-1 text-muted-foreground">한도 리셋 후 자동 재개</span><Switch bind:checked={h.autoResume} />
				</label>
			</Card.Content>
		</Card.Root>
	</aside>
</div>

<!-- 모델 선택 (.pen Model Picker Dialog) — 연결별 모델 · 한도 잔량 -->
<Dialog.Root bind:open={picker}>
	<Dialog.Content size="xl" tall>
		{#if picker}
			<Dialog.Header>
				<Dialog.Title>모델 선택</Dialog.Title>
				<Dialog.Description>연결별 모델 목록 · 한도 잔량을 보고 고르세요 · 소진된 제공자는 폴백으로만 쓰여요</Dialog.Description>
				{#snippet sub()}
			<div class="flex flex-wrap items-center gap-2">
				<InputGroup.Root class="h-8 w-60">
					<InputGroup.Addon><Search /></InputGroup.Addon>
					<InputGroup.Input bind:value={pickQuery} placeholder="모델 검색…" aria-label="모델 검색" />
				</InputGroup.Root>
				{#each [['all', '전체'], ['추천', '추천'], ['Tested', 'Tested'], ['NEW', 'NEW'], ['cheap', '저렴한 순'], ['long', '긴 컨텍스트']] as const as [v, l] (v)}
					<Toggle variant="chip" pressed={pickFilter === v} onPressedChange={() => (pickFilter = v)}>{l}</Toggle>
				{/each}
				<span class="flex-1"></span>
				<span class="text-caption text-muted-foreground">{pickModels.reduce((n, p) => n + p.models.length, 0)}개 · 가격 in/out per 1M</span>
			</div>
				{/snippet}
			</Dialog.Header>
			<Dialog.Body padded={false} class="flex-row">
				<nav aria-label="제공자" class="model-provider-nav">
					<span class="list-label px-2 pt-1 pb-1.5">제공자 · 연결됨 {modelCatalog.length}</span>
					<button type="button" aria-pressed={pickProvider === 'all'} onclick={() => (pickProvider = 'all')} class={cn('model-provider-item', pickProvider === 'all' && 'bg-accent font-medium')}><Layers class="size-3.5" /><span class="flex-1">전체</span></button>
					{#each modelCatalog as p (p.key)}
						{@const a = p.kind === '구독' ? accountOf(p.runtime) : undefined}
						<button type="button" aria-pressed={pickProvider === p.key} onclick={() => (pickProvider = p.key)} class={cn('model-provider-item', pickProvider === p.key && 'bg-accent font-medium')}>
							{#if a}<RuntimeLogo runtime={p.runtime} class="size-3.5 ring-0" />{:else}<Route class="size-3.5" />{/if}
							<span class="flex-1 truncate">{p.label}</span>
							<span class={cn('font-mono text-caption', a && low(a.week) ? 'text-destructive' : 'text-muted-foreground')}>{a ? `${a.week}%` : `$${Math.round(gateway.used)}`}</span>
						</button>
					{/each}
				</nav>
				<div class="page-scroll gap-4 px-5 py-4">
					{#each pickModels as p (p.key)}
						{@const a = p.kind === '구독' ? accountOf(p.runtime) : undefined}
						<section class="flex flex-col gap-1">
							<div class="model-group-head">
								<span class="font-semibold">{p.label}</span><Pill class="text-2xs">{p.kind}</Pill>
								<span class="flex-1"></span>
								{#if a}
									{#each [['5H', a.h5], ['주간', a.week]] as const as [l, v] (l)}
										<span class="meta-line gap-1.5">{l}<Progress value={v} class="h-1 w-12 bg-muted" indicator={low(v) ? 'bg-destructive' : 'bg-success'} aria-label="{p.label} {l}" /><span class={cn('font-mono', low(v) && 'text-destructive')}>{v}%</span></span>
									{/each}
								{:else}
									<span class="font-mono text-caption text-muted-foreground">월 ${gateway.used}/${gateway.limit}</span>
								{/if}
							</div>
							{#each p.models as md (md.name)}
								{@const on = model === md.name && runtime === p.runtime}
								<button
									type="button"
									aria-pressed={on}
									onclick={() => (onchange(p.runtime, md.name), (picker = false))}
									class={cn('card-press gap-3 px-3 py-2.5', on && 'option-on')}
								>
									{#if on}<CircleDot class="size-4 text-primary" />{:else}<Circle class="size-4 text-subtle-foreground" />{/if}
									<span class="row-text">
										<span class="row-title">
											{md.name}
											{#each md.tags as tg (tg)}<Pill class={cn('text-2xs', tg === 'NEW' ? 'bg-primary-soft text-primary' : tg === '추천' ? 'bg-success-soft text-status-done' : '')}>{#if tg === '추천'}<ThumbsUp />{:else if tg === 'Tested'}<CircleCheck />{/if}{tg}</Pill>{/each}
										</span>
										<span class="text-xs text-muted-foreground">{md.vendor}</span>
									</span>
									<span class="num-cell w-24">{md.price}</span>
									<span class="w-12 text-right font-mono text-caption">{md.ctx}</span>
									<span class="w-12 text-right text-caption text-muted-foreground">● {md.speed}</span>
								</button>
							{/each}
						</section>
					{:else}
						<p class="py-10 text-center text-xs text-muted-foreground">조건에 맞는 모델이 없어요.</p>
					{/each}
				</div>
			</Dialog.Body>
		{/if}
	</Dialog.Content>
</Dialog.Root>
