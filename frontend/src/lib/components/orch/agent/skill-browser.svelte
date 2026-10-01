<script lang="ts">
	/// skills.sh 탐색 (.pen 스킬 추가 · skills.sh 탐색 · 스킬 상세). 에이전트 Skills 탭과 Skills & MCP 페이지가 같이 쓴다.
	/// added로 이미 추가된 스킬을 표시하고, 추가는 onadd로 돌려준다 (어디에 추가할지는 쓰는 쪽이 정한다).
	import Search from '@lucide/svelte/icons/search';
	import Plus from '@lucide/svelte/icons/plus';
	import Download from '@lucide/svelte/icons/download';
	import ShieldCheck from '@lucide/svelte/icons/shield-check';
	import Layers from '@lucide/svelte/icons/layers';
	import Check from '@lucide/svelte/icons/check';
	import TriangleAlert from '@lucide/svelte/icons/triangle-alert';
	import ExternalLink from '@lucide/svelte/icons/external-link';
	import Hash from '@lucide/svelte/icons/hash';
	import Scale from '@lucide/svelte/icons/scale';
	import Users from '@lucide/svelte/icons/users';
	import Zap from '@lucide/svelte/icons/zap';
	import GitCommitHorizontal from '@lucide/svelte/icons/git-commit-horizontal';
	import * as Dialog from '$lib/components/ui/dialog';
	import * as Tabs from '$lib/components/ui/tabs';
	import * as InputGroup from '$lib/components/ui/input-group';
	import { Button } from '$lib/components/ui/button';
	import { Toggle } from '$lib/components/ui/toggle';
	import { Pill } from '$lib/components/ui/pill';
	import { MdEditor } from '$lib/components/ui/md-editor';
	import { RuntimeLogo } from '$lib/components/ui/runtime-logo';
	import { skillsSh, type SkillHit } from '$lib/mock';
	import { store } from '$lib/teams.svelte';

	let {
		added,
		onadd,
		target,
	}: {
		added: (name: string) => boolean;
		onadd: (h: SkillHit) => void;
		/** 추가 대상 이름 (예: "진", "워크스페이스"). */
		target: string;
	} = $props();

	let shQuery = $state('');
	let shFilter = $state<'all' | SkillHit['tags'][number]>('all');
	let skillView = $state<SkillHit>();
	const shHits = $derived(
		skillsSh.filter((h) => (shFilter === 'all' || h.tags.includes(shFilter)) && `${h.name} ${h.desc} ${h.repo}`.toLowerCase().includes(shQuery.trim().toLowerCase()))
	);
</script>

<div class="flex flex-col gap-3">
	<div class="flex flex-wrap items-center gap-2">
		<InputGroup.Root class="h-8 w-64 bg-card">
			<InputGroup.Addon><Search /></InputGroup.Addon>
			<InputGroup.Input bind:value={shQuery} placeholder="skills.sh 검색" aria-label="skills.sh 검색" />
		</InputGroup.Root>
		{#each [['all', '전체'], ['trending', 'Trending'], ['official', '공식'], ['test', '테스트'], ['a11y', '접근성']] as const as [v, l] (v)}
			<Toggle variant="chip" pressed={shFilter === v} onPressedChange={() => (shFilter = v)}>{l}</Toggle>
		{/each}
		<span class="flex-1"></span>
		<span class="text-caption text-muted-foreground">결과 {shHits.length} · 설치 많은 순</span>
	</div>
	<div class="rounded-lg border bg-card px-4 py-1">
		{#each shHits as h (h.name)}
			{@const isAdded = added(h.name)}
			{@const warn = h.audit[0] < h.audit[1]}
			<div class="flex items-center gap-3 border-t py-3 first:border-t-0">
				<span class="flex size-9 shrink-0 items-center justify-center rounded-md bg-muted font-mono text-sm font-semibold text-muted-foreground uppercase">{h.name[0]}</span>
				<button type="button" class="flex min-w-0 flex-1 flex-col gap-1 text-left outline-none focus-visible:underline" onclick={() => (skillView = h)}>
					<span class="flex items-baseline gap-2"><span class="text-body font-semibold">{h.name}</span><span class="font-mono text-caption text-muted-foreground">{h.repo}</span></span>
					<span class="text-xs text-muted-foreground">{h.desc}</span>
					<span class="flex flex-wrap items-center gap-3 text-caption text-muted-foreground">
						<span class="flex items-center gap-1"><Download class="size-3" />{h.installs}</span>
						<span class="flex items-center gap-1"><RuntimeLogo runtime="claude" class="size-3.5 ring-0" /><RuntimeLogo runtime="codex" class="size-3.5 ring-0" />+{h.agents - 2}</span>
						<Pill class={warn ? 'bg-warning-soft text-status-waiting' : 'bg-success-soft text-status-done'}><ShieldCheck />보안 검사 {h.audit[0]}/{h.audit[1]}</Pill>
						<span class="flex items-center gap-1 font-mono"><Layers class="size-3" />+{(h.tok / 1000).toFixed(1)}K tok</span>
					</span>
				</button>
				{#if isAdded}
					<Pill class="bg-success-soft text-status-done"><Check />추가됨</Pill>
				{:else}
					<Button variant="outline" size="sm" onclick={() => (warn && store.sources.policy[0].on ? (skillView = h) : onadd(h))}><Plus />{warn && store.sources.policy[0].on ? '검토 후 추가' : '추가'}</Button>
				{/if}
			</div>
		{:else}
			<p class="py-6 text-center text-xs text-muted-foreground">검색 결과가 없어요.</p>
		{/each}
	</div>
</div>

<!-- 스킬 상세 (.pen Skill Detail Dialog) -->
<Dialog.Root bind:open={() => skillView !== undefined, (v) => !v && (skillView = undefined)}>
	<Dialog.Content size="xl" tall>
		{#if skillView}
			{@const h = skillView}
			{@const isAdded = added(h.name)}
			<Dialog.Header>
				{#snippet lead()}<span class="flex size-11 shrink-0 items-center justify-center rounded-md bg-muted font-mono text-lg font-semibold text-muted-foreground uppercase">{h.name[0]}</span>{/snippet}
					<Dialog.Title class="flex items-baseline gap-2">{h.name}<span class="font-mono text-xs font-normal text-muted-foreground">{h.repo}</span></Dialog.Title>
					<div class="flex flex-wrap items-center gap-3 text-caption text-muted-foreground">
						<span class="flex items-center gap-1"><Download class="size-3" />{h.installs} installs</span>
						<span class="flex items-center gap-1"><GitCommitHorizontal class="size-3" />{h.version}</span>
						<span class="flex items-center gap-1"><Scale class="size-3" />{h.license}</span>
						<Pill class={h.audit[0] < h.audit[1] ? 'bg-warning-soft text-status-waiting' : 'bg-success-soft text-status-done'}><ShieldCheck />보안 검사 {h.audit[0]}/{h.audit[1]}</Pill>
					</div>
				{#snippet actions()}<Button variant="ghost" size="icon-sm" href="https://skills.sh" target="_blank" rel="noreferrer" aria-label="skills.sh에서 보기"><ExternalLink /></Button>{/snippet}
			</Dialog.Header>
			<Dialog.Body padded={false} class="flex-row">
				<Tabs.Root value="overview" class="min-w-0 flex-1 gap-0 overflow-y-auto px-6 py-4">
					<Tabs.List variant="line" class="mb-4">
						<Tabs.Trigger value="overview">개요</Tabs.Trigger>
						<Tabs.Trigger value="md">SKILL.md</Tabs.Trigger>
						<Tabs.Trigger value="audit">보안 검사</Tabs.Trigger>
					</Tabs.List>
					<Tabs.Content value="overview" class="flex flex-col gap-2 text-body">
						<p>{h.desc}</p>
						{#each h.skillMd.filter((l) => l.startsWith('- ')) as l (l)}<p class="text-xs text-muted-foreground">• {l.slice(2)}</p>{/each}
					</Tabs.Content>
					<Tabs.Content value="md">
						<MdEditor files={[{ name: 'SKILL.md', body: h.skillMd.join('\n') }]} readonly tabs={false} class="h-100" />
					</Tabs.Content>
					<Tabs.Content value="audit" class="flex flex-col gap-2">
						{#each ['Socket', 'Snyk', 'skills.sh 검토'] as a, i (a)}
							{@const ok = i < h.audit[0]}
							<div class="flex items-center gap-2 text-xs">
								<ShieldCheck class="size-3.5 text-muted-foreground" /><span class="flex-1">{a}</span>
								<Pill class={ok ? 'bg-success-soft text-status-done' : 'bg-warning-soft text-status-waiting'}>{#if ok}<Check />통과{:else}<TriangleAlert />경고 · 외부 네트워크 호출{/if}</Pill>
							</div>
						{/each}
					</Tabs.Content>
				</Tabs.Root>
				<aside class="flex w-64 shrink-0 flex-col gap-4 border-l bg-background px-5 py-4 text-xs">
					<span class="font-semibold">이 스킬은</span>
					{#each [[Zap, '트리거', h.skillMd.find((l) => l.startsWith('description:'))?.slice(13) ?? '관련 작업 시'], [Layers, '컨텍스트', `+${(h.tok / 1000).toFixed(1)}K tok`], [Hash, 'sha256', '3f9a…c21e'], [Users, '이 팀 사용', store.library.some((k) => k.name === h.name) ? '설치됨' : '처음 추가']] as const as [Icon, k, v] (k)}
						<div class="flex items-start gap-2"><Icon class="mt-0.5 size-3.25 text-muted-foreground" /><span class="w-16 text-muted-foreground">{k}</span><span class="flex-1 font-medium">{v}</span></div>
					{/each}
					<span class="border-t pt-3 font-semibold">지원 에이전트</span>
					<div class="flex items-center gap-2"><RuntimeLogo runtime="claude" class="size-4 ring-0" />Claude Code</div>
					<div class="flex items-center gap-2"><RuntimeLogo runtime="codex" class="size-4 ring-0" />Codex CLI<span class="text-muted-foreground">+{h.agents - 2}</span></div>
				</aside>
			</Dialog.Body>
			<Dialog.Footer note="{target}에 추가 · 다음 Run부터 적용">
				<Button variant="ghost" size="sm" onclick={() => (skillView = undefined)}>닫기</Button>
				<Button size="sm" disabled={isAdded} onclick={() => (onadd(h), (skillView = undefined))}>{#if isAdded}<Check />추가됨{:else}<Plus />{h.audit[0] < h.audit[1] ? '검토했어요 · 추가' : '추가'}{/if}</Button>
			</Dialog.Footer>
		{/if}
	</Dialog.Content>
</Dialog.Root>
