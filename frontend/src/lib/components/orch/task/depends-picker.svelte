<script lang="ts">
	/// 의존 선택 창 (.pen C · Depends Picker): depends on / blocks 전환 · 태스크 검색 · 순환 의존 경고.
	/// 고르면 그 관계를 넣거나 뺀다(토글). 트리거는 호출부가 그린다.
	import type { Snippet } from 'svelte';
	import * as Combobox from '$lib/components/ui/combobox';
	import { Segmented } from '$lib/components/ui/segmented';
	import { statuses } from '$lib/status';
	import type { Task, TaskDetail } from '$lib/mock';
	
	type Dep = TaskDetail['deps'][number];

	let {
		value = $bindable(),
		current,
		tasks,
		depsOf,
		trigger
	}: {
		/** 이 태스크의 의존 관계. */
		value: Dep[];
		/** 이 태스크 번호. 새 태스크면 undefined (순환 검사 없음). */
		current?: number;
		tasks: Task[];
		/** 다른 태스크의 의존 관계 (순환 검사). */
		depsOf: (num: number) => Dep[];
		trigger: Snippet<[Record<string, unknown>]>;
	} = $props();

	let open = $state(false);
	let kind = $state<Dep['kind']>('depends');

	const has = (num: number) => value.some((d) => d.kind === kind && d.num === num);
	/// 순환: X에 의존하려는데 X가 이미 이 태스크에 의존(또는 이 태스크가 X를 막음)하는 경우, blocks는 그 반대.
	function cycle(num: number) {
		if (current === undefined) return false;
		const other = depsOf(num);
		return kind === 'depends'
			? other.some((d) => d.kind === 'depends' && d.num === current) || value.some((d) => d.kind === 'blocks' && d.num === num)
			: other.some((d) => d.kind === 'blocks' && d.num === current) || value.some((d) => d.kind === 'depends' && d.num === num);
	}
	function toggle(num: number) {
		value = has(num) ? value.filter((d) => !(d.kind === kind && d.num === num)) : [...value, { kind, num }];
	}
</script>

<Combobox.Root bind:open>
	<Combobox.Trigger>
		{#snippet child({ props })}{@render trigger(props)}{/snippet}
	</Combobox.Trigger>
	<Combobox.Content class="w-75 p-1">
		{#snippet header()}
			<Segmented
				class="w-full"
				aria-label="관계"
				options={[
					{ value: 'depends', label: 'depends on' },
					{ value: 'blocks', label: 'blocks' }
				]}
				bind:value={() => kind, (v) => (kind = v as Dep['kind'])}
			/>
		{/snippet}
		<Combobox.Search placeholder="#번호 · 제목 검색" />
		<Combobox.List>
			<Combobox.Empty>찾는 태스크가 없어요</Combobox.Empty>
			{#each tasks.filter((t) => t.num !== current) as t (t.num)}
				{@const m = statuses[t.status]}
				{@const loop = cycle(t.num)}
				<!-- 여러 개를 고를 수 있어 고른 뒤에도 열어 둔다 -->
				<Combobox.Item value="#{t.num} {t.title}" selected={has(t.num)} closeOnSelect={false} disabled={loop && !has(t.num)} onSelect={() => toggle(t.num)} class="gap-2 py-1.5">
					<m.icon class={['size-3.5', m.text]} />
					<span class="flex min-w-0 flex-1 flex-col gap-0.5">
						<span class="flex min-w-0 items-center gap-1.5"><span class="font-medium">#{t.num}</span><span class="truncate text-xs text-muted-foreground">{t.title}</span></span>
						{#if loop}<span class="text-xs text-status-blocked">⚠ 순환 의존 — #{t.num}이 이미 #{current}{kind === 'depends' ? '에 의존' : '을 막음'}</span>{/if}
					</span>
				</Combobox.Item>
			{/each}
		</Combobox.List>
	</Combobox.Content>
</Combobox.Root>
