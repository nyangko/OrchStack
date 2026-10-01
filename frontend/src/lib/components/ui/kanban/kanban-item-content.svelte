<script lang="ts">
	/// 칸반 항목 카드 본문.
	import type { HTMLAttributes } from "svelte/elements";
	import { cn, type WithElementRef } from "$lib/utils.js";

	let { ref = $bindable(null), class: className, children, ...restProps }: WithElementRef<HTMLAttributes<HTMLSpanElement>> = $props();
</script>

<!-- 카드가 button이라 안쪽은 span(block 표시) -->
<span bind:this={ref} data-slot="kanban-item-content" class={cn("flex flex-col gap-2.5 px-4 py-3.5", className)} {...restProps}>
	{@render children?.()}
</span>
