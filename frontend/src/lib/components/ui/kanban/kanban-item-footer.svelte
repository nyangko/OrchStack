<script lang="ts">
	/// 칸반 항목 카드 바닥 (위 테두리 · 고정폭 보조 값).
	import type { HTMLAttributes } from "svelte/elements";
	import { cn, type WithElementRef } from "$lib/utils.js";

	let { ref = $bindable(null), class: className, children, ...restProps }: WithElementRef<HTMLAttributes<HTMLSpanElement>> = $props();
</script>

<!-- 카드가 button이라 안쪽은 span(block 표시) -->
<span bind:this={ref} data-slot="kanban-item-footer" class={cn("flex items-center gap-3.5 border-t px-4 py-2.5 font-mono text-xs text-muted-foreground", className)} {...restProps}>
	{@render children?.()}
</span>
