<script lang="ts">
	/// 우선순위 선택 창 (.pen C · Priority Menu): P0 긴급 ~ P3 낮음 · 열려 있을 때 숫자 키 0~3. 트리거는 호출부가 그린다.
	import type { Snippet } from 'svelte';
	import Check from '@lucide/svelte/icons/check';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
	import { priorities, priorityOrder, type Priority } from '$lib/priority';
	import { cn } from '$lib/utils';

	let { value = $bindable(), trigger }: { value: Priority; trigger: Snippet<[Record<string, unknown>]> } = $props();

	let open = $state(false);
	function onkeydown(e: KeyboardEvent) {
		const p = `P${e.key}` as Priority;
		if (priorityOrder.includes(p)) {
			value = p;
			open = false;
		}
	}
</script>

<DropdownMenu.Root bind:open>
	<DropdownMenu.Trigger>
		{#snippet child({ props })}{@render trigger(props)}{/snippet}
	</DropdownMenu.Trigger>
	<DropdownMenu.Content align="start" class="w-50" {onkeydown}>
		{#each priorityOrder as p (p)}
			{@const m = priorities[p]}
			<DropdownMenu.Item onSelect={() => (value = p)} class="gap-2">
				<m.icon class={cn('size-3.5', m.text)} />
				<span class="font-medium">{p}</span><span class="flex-1 text-xs text-muted-foreground">{m.label}</span>
				{#if value === p}<Check class="size-3.5" />{:else}<DropdownMenu.Shortcut>{p.slice(1)}</DropdownMenu.Shortcut>{/if}
			</DropdownMenu.Item>
		{/each}
	</DropdownMenu.Content>
</DropdownMenu.Root>
