<script lang="ts">
	/// 우선순위 선택 창 (.pen C · Priority Menu): P0 긴급 ~ P3 낮음 · 열려 있을 때 숫자 키 0~3. 트리거는 호출부가 그린다.
	import type { Snippet } from 'svelte';
	import Check from '@lucide/svelte/icons/check';
	import { DropdownMenu, DropdownMenuTrigger, DropdownMenuContent, DropdownMenuItem, DropdownMenuShortcut } from '$lib/components/ui/dropdown-menu';
	import { priorities, priorityOrder, type Priority } from '$lib/priority';
	
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

<DropdownMenu bind:open>
	<DropdownMenuTrigger>
		{#snippet child({ props })}{@render trigger(props)}{/snippet}
	</DropdownMenuTrigger>
	<DropdownMenuContent align="start" class="w-50" {onkeydown}>
		{#each priorityOrder as p (p)}
			{@const m = priorities[p]}
			<DropdownMenuItem onSelect={() => (value = p)} class="gap-2">
				<m.icon class={['size-3.5', m.text]} />
				<span class="font-medium">{p}</span><span class="flex-1 text-xs text-muted-foreground">{m.label}</span>
				{#if value === p}<Check class="size-3.5" />{:else}<DropdownMenuShortcut>{p.slice(1)}</DropdownMenuShortcut>{/if}
			</DropdownMenuItem>
		{/each}
	</DropdownMenuContent>
</DropdownMenu>
