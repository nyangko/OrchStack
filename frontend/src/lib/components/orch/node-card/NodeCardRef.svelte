<script lang="ts">
	/// 대문자 참조 표기 (예: TASK #129).
	import type { HTMLAttributes } from "svelte/elements";
	import { cn, type WithElementRef } from "$lib/utils.js";

	let { ref = $bindable(null), class: className, children, ...restProps }: WithElementRef<HTMLAttributes<HTMLElement>> = $props();
</script>

<span bind:this={ref} data-slot="node-card-ref" class={cn("flex-1 truncate font-mono text-xs font-semibold tracking-wide text-muted-foreground", className)} {...restProps}>
	{@render children?.()}
</span>
