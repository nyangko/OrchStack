<script lang="ts">
	/// 칸반 열의 항목 목록. 열이 비면 empty를 그린다.
	import type { Snippet } from "svelte";
	import type { HTMLAttributes } from "svelte/elements";
	import { cn, type WithElementRef } from "$lib/utils.js";
	import { getBoard, getColumn } from "./context.js";

	let {
		ref = $bindable(null),
		class: className,
		empty,
		children,
		...restProps
	}: WithElementRef<HTMLAttributes<HTMLDivElement>> & { empty?: Snippet } = $props();

	const board = getBoard();
	const column = getColumn();
	const count = $derived(board()[column()]?.length ?? 0);
</script>

<div bind:this={ref} data-slot="kanban-column-content" class={cn("flex min-h-24 flex-col gap-2", className)} {...restProps}>
	{#if count === 0 && empty}
		{@render empty()}
	{:else}
		{@render children?.()}
	{/if}
</div>
