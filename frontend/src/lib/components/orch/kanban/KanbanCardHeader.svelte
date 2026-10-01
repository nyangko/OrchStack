<script lang="ts">
	/// 칸반 카드 머리 (아래 테두리).
	import type { HTMLAttributes } from "svelte/elements";
	import { cn, type WithElementRef } from "$lib/utils.js";

	let { ref = $bindable(null), class: className, children, ...restProps }: WithElementRef<HTMLAttributes<HTMLSpanElement>> = $props();
</script>

<!-- 카드가 button이라 안쪽은 span(block 표시) -->
<span bind:this={ref} data-slot="kanban-card-header" class={cn("flex items-center gap-2 border-b px-4 py-3", className)} {...restProps}>
	{@render children?.()}
</span>
