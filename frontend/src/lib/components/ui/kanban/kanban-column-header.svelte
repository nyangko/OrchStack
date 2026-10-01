<script lang="ts">
	/// 칸반 열 머리 (.pen KanbanColumnHeader): 아이콘 · 제목 · 개수 · 동작.
	import type { HTMLAttributes } from "svelte/elements";
	import { cn, type WithElementRef } from "$lib/utils.js";

	let { ref = $bindable(null), class: className, children, ...restProps }: WithElementRef<HTMLAttributes<HTMLDivElement>> = $props();
</script>

<div bind:this={ref} data-slot="kanban-column-header" class={cn("flex h-10 items-center gap-2 pr-1 pl-2 [&>svg]:size-3.5", className)} {...restProps}>
	{@render children?.()}
</div>
