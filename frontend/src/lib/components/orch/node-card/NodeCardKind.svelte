<script lang="ts">
	/// 노드 종류 타일 — 색은 class(bg-*)로.
	import type { HTMLAttributes } from "svelte/elements";
	import { cn, type WithElementRef } from "$lib/utils.js";

	let { ref = $bindable(null), class: className, children, ...restProps }: WithElementRef<HTMLAttributes<HTMLElement>> = $props();
</script>

<span bind:this={ref} data-slot="node-card-kind" class={cn("flex size-6 items-center justify-center rounded-sm text-on-solid [&>svg]:size-3.5", className)} {...restProps}>
	{@render children?.()}
</span>
