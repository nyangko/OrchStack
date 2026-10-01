<script lang="ts">
	/// 노드 카드 바닥 줄: 왼쪽 상태 · 오른쪽 보조 값.
	import type { HTMLAttributes } from "svelte/elements";
	import { cn, type WithElementRef } from "$lib/utils.js";

	let { ref = $bindable(null), class: className, children, ...restProps }: WithElementRef<HTMLAttributes<HTMLElement>> = $props();
</script>

<div bind:this={ref} data-slot="node-card-footer" class={cn("flex items-center justify-between gap-2", className)} {...restProps}>
	{@render children?.()}
</div>
