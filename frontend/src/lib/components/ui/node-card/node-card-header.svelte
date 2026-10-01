<script lang="ts">
	/// 노드 카드 머리 줄: 종류 타일 · 참조 · 동작.
	import type { HTMLAttributes } from "svelte/elements";
	import { cn, type WithElementRef } from "$lib/utils.js";

	let { ref = $bindable(null), class: className, children, ...restProps }: WithElementRef<HTMLAttributes<HTMLElement>> = $props();
</script>

<div bind:this={ref} data-slot="node-card-header" class={cn("flex items-center gap-2", className)} {...restProps}>
	{@render children?.()}
</div>
