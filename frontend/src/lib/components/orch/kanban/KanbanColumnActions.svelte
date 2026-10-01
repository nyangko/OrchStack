<script lang="ts">
	/// 칸반 열 머리 오른쪽 동작 (추가 · 메뉴).
	import type { HTMLAttributes } from "svelte/elements";
	import { cn, type WithElementRef } from "$lib/utils.js";

	let { ref = $bindable(null), class: className, children, ...restProps }: WithElementRef<HTMLAttributes<HTMLDivElement>> = $props();
</script>

<div bind:this={ref} data-slot="kanban-column-actions" class={cn("ml-auto flex items-center gap-2", className)} {...restProps}>
	{@render children?.()}
</div>
