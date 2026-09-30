<script lang="ts">
	/// 에이전트 권한 (.pen 권한 기본값). Trust 레벨 · 파일 범위 glob · 승인 규칙. used가 있으면(템플릿) 쓰는 멤버별 차이도 보인다.
	import CircleCheck from '@lucide/svelte/icons/circle-check';
	import X from '@lucide/svelte/icons/x';
	import * as Card from '$lib/components/ui/card';
	import { Input } from '$lib/components/ui/input';
	import { Pill } from '$lib/components/ui/pill';
	import { RoleAvatar } from '$lib/components/ui/role-avatar';
	import TrustPicker from './trust-picker.svelte';
	import ApprovalTable from './approval-table.svelte';
	import type { AgentConfig, TeamMember } from '$lib/mock';
	import { cfgDiff, glyphOf } from '$lib/teams.svelte';

	let {
		config: c,
		base,
		used,
	}: {
		config: AgentConfig;
		base?: AgentConfig;
		/** 이 템플릿을 쓰는 멤버 (템플릿 화면에서만). */
		used?: { m: TeamMember; team: string }[];
	} = $props();

	let includeDraft = $state('');
	let excludeDraft = $state('');
</script>

{#snippet sameAs(changed: boolean, base?: AgentConfig)}
	{#if base}
		<Pill class={changed ? 'bg-primary-soft text-primary' : ''}>{changed ? '템플릿과 다름' : '템플릿과 동일'}</Pill>
	{/if}
{/snippet}

{#snippet globs(list: string[], draft: string, set: (v: string) => void, label: string)}
	<div class="flex flex-col gap-1.5">
		<span class="text-caption font-semibold text-muted-foreground">{label}</span>
		<div class="flex flex-wrap items-center gap-1.5">
			{#each list as g, i (g)}
				<span class="flex items-center gap-1 rounded-sm border bg-muted px-2 py-0.75 font-mono text-xs">
					{g}
					<button type="button" aria-label="{g} 삭제" onclick={() => list.splice(i, 1)} class="text-subtle-foreground hover:text-foreground"><X class="size-3" /></button>
				</span>
			{/each}
			<form class="flex items-center gap-1" onsubmit={(e) => (e.preventDefault(), draft.trim() && !list.includes(draft.trim()) && list.push(draft.trim()), set(''))}>
				<Input value={draft} oninput={(e) => set(e.currentTarget.value)} placeholder="패턴 추가 · Enter" aria-label="{label} 패턴 추가" class="h-7 w-44 font-mono text-xs placeholder:font-sans" />
			</form>
		</div>
	</div>
{/snippet}

<div class="flex items-center gap-2.5">
	<h2 class="text-lg font-semibold">{base ? '권한' : '권한 기본값'}</h2>
	{@render sameAs(cfgDiff(c, base) > 0, base)}
	<span class="flex-1"></span>
	<span class="flex items-center gap-1 text-caption text-status-done"><CircleCheck class="size-3" />저장됨</span>
</div>
<Card.Root size="sm">
	<Card.Header>
		<Card.Title>Trust 레벨</Card.Title>
		<Card.Description>에이전트가 사람 확인 없이 할 수 있는 범위</Card.Description>
	</Card.Header>
	<Card.Content>
		<TrustPicker bind:value={c.trust} base={base?.trust} />
	</Card.Content>
</Card.Root>
<Card.Root size="sm">
	<Card.Header>
		<Card.Title>파일 범위</Card.Title>
		<Card.Description>glob 패턴 · 제외가 포함보다 우선</Card.Description>
	</Card.Header>
	<Card.Content class="gap-3.5">
		{@render globs(c.include, includeDraft, (v) => (includeDraft = v), '포함')}
		{@render globs(c.exclude, excludeDraft, (v) => (excludeDraft = v), '제외')}
	</Card.Content>
</Card.Root>
<Card.Root size="sm">
	<Card.Header><Card.Title>승인 규칙</Card.Title></Card.Header>
	<Card.Content class="gap-0">
		<ApprovalTable approvals={c.approvals} base={base?.approvals} />
	</Card.Content>
</Card.Root>
{#if used}
	<Card.Root size="sm">
		<Card.Header><Card.Title>이 템플릿을 쓰는 멤버 · {used.length}</Card.Title></Card.Header>
		<Card.Content class="gap-0">
			{#each used as { m, team: tn } (m.sn)}
				{@const n = cfgDiff(m.config, c)}
				<div class="flex items-center gap-2.5 border-t py-2 first:border-t-0">
					<RoleAvatar role={m.role} icon={glyphOf(m)} size="sm" />
					<span class="flex flex-1 flex-col gap-px text-xs"><span class="font-medium">{m.name} · {tn}</span><span class="text-muted-foreground">{!m.config || n === 0 ? '권한 템플릿과 동일' : `권한 설정 ${n}곳 다름`}</span></span>
				</div>
			{:else}
				<p class="text-xs text-muted-foreground">없어요.</p>
			{/each}
			<span class="border-t pt-2 text-caption text-subtle-foreground">기본값을 바꿔도 이미 만든 멤버에게는 반영되지 않아요.</span>
		</Card.Content>
	</Card.Root>
{/if}
