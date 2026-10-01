<script lang="ts">
	/// 담당 선택 창 (.pen C · Assignee Picker): 검색 · 추천(역할 일치) · 다른 에이전트 · Unassigned · Orch에게 배정 맡기기.
	/// 에이전트마다 부하 막대와 지금 상태(작업 중 · 대기 · 여유)를 보여 준다. 트리거는 호출부가 그린다.
	import type { Snippet } from 'svelte';
	import UserRoundX from '@lucide/svelte/icons/user-round-x';
	import Sparkles from '@lucide/svelte/icons/sparkles';
	import * as Combobox from '$lib/components/ui/combobox';
	import { RoleAvatar } from '$lib/components/ui/role-avatar';
	import { Progress } from '$lib/components/ui/progress';
	import { roles } from '$lib/roles';
	import type { Agent, Task } from '$lib/mock';

	let {
		value = $bindable(),
		agents,
		tasks,
		recommend = [],
		onorch,
		trigger
	}: {
		/** 담당 에이전트 sn. undefined = Unassigned. */
		value?: number;
		agents: Agent[];
		/** 부하 · 상태 계산용 (프로젝트 태스크). */
		tasks: Task[];
		/** 추천 · 역할 일치로 위에 둘 에이전트 sn. */
		recommend?: number[];
		/** Orch에게 배정 맡기기. 없으면 항목을 숨긴다. */
		onorch?: () => void;
		trigger: Snippet<[Record<string, unknown>]>;
	} = $props();

	let open = $state(false);

	const OPEN = ['todo', 'in_progress', 'waiting', 'blocked', 'review'];
	/// 부하(열린 태스크 수 / 4)와 상태 한 줄.
	function load(sn: number) {
		const mine = tasks.filter((t) => t.agent === sn && OPEN.includes(t.status));
		const working = mine.find((t) => t.status === 'in_progress');
		const waiting = mine.filter((t) => t.status === 'waiting').length;
		const note = working ? `작업 중 · #${working.num}` : waiting ? `대기 태스크 ${waiting}` : '여유';
		const tone = working ? 'text-status-in-progress' : waiting ? 'text-status-waiting' : 'text-status-done';
		return { pct: Math.min(100, (mine.length / 4) * 100), note, tone };
	}
	const groups = $derived([
		{ heading: '추천 · 역할 일치', list: agents.filter((a) => recommend.includes(a.sn)) },
		{ heading: recommend.length ? '다른 에이전트' : '에이전트', list: agents.filter((a) => !recommend.includes(a.sn)) }
	].filter((g) => g.list.length));

</script>

<Combobox.Root bind:open>
	<Combobox.Trigger>
		{#snippet child({ props })}{@render trigger(props)}{/snippet}
	</Combobox.Trigger>
	<Combobox.Content class="w-70">
		<Combobox.Search placeholder="에이전트 검색…" />
		<Combobox.List>
			<Combobox.Empty>찾는 에이전트가 없어요</Combobox.Empty>
			{#each groups as g (g.heading)}
				<Combobox.Group heading={g.heading}>
					{#each g.list as a (a.sn)}
						{@const l = load(a.sn)}
						<Combobox.Item value="{a.name} {roles[a.role].label}" selected={value === a.sn} onSelect={() => (value = a.sn)} class="gap-2 py-1.5">
							<RoleAvatar role={a.role} size="sm" />
							<span class="flex min-w-0 flex-1 flex-col gap-0.5">
								<span class="flex items-center gap-1.5"><span class="font-medium">{a.name}</span><span class="text-xs text-muted-foreground">{roles[a.role].label}</span></span>
								<span class="flex items-center gap-1.5 text-xs">
									<Progress value={l.pct} class="h-1 w-10" aria-label="{a.name} 부하" />
									<span class={l.tone}>{l.note}</span>
								</span>
							</span>
						</Combobox.Item>
					{/each}
				</Combobox.Group>
			{/each}
			<Combobox.Group>
				<Combobox.Item value="Unassigned 미배정" selected={value === undefined} onSelect={() => (value = undefined)} class="gap-2">
					<span class="flex items-center justify-center size-5 rounded-xs bg-muted"><UserRoundX class="size-3 text-muted-foreground" /></span>
					<span class="font-medium">Unassigned</span>
				</Combobox.Item>
			</Combobox.Group>
			{#if onorch}
				<Combobox.Separator />
				<Combobox.Group>
					<Combobox.Item value="Orch에게 배정 맡기기" onSelect={() => onorch()} class="gap-2">
						<span class="flex size-5 items-center justify-center"><Sparkles class="size-3.5 text-primary" /></span>
						<span class="flex flex-col gap-0.5"><span class="font-medium">Orch에게 배정 맡기기</span><span class="text-xs text-muted-foreground">역할 · 부하 · 컨텍스트 기준</span></span>
					</Combobox.Item>
				</Combobox.Group>
			{/if}
		</Combobox.List>
	</Combobox.Content>
</Combobox.Root>
