<script lang="ts">
	/// 노드 제목.
	import type { HTMLAttributes } from "svelte/elements";
	import { cn, type WithElementRef } from "$lib/utils.js";

	let { ref = $bindable(null), class: className, children, ...restProps }: WithElementRef<HTMLAttributes<HTMLElement>> = $props();
</script>

<p bind:this={ref} data-slot="node-card-title" class={cn("text-sm leading-tight font-semibold", className)} {...restProps}>
	{@render children?.()}
</p>
