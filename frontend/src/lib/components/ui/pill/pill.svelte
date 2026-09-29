<script lang="ts">
	import { cn, type WithElementRef } from "$lib/utils.js";
	import type { HTMLAttributes } from "svelte/elements";

	let {
		ref = $bindable(null),
		class: className,
		dot,
		children,
		...restProps
	}: WithElementRef<HTMLAttributes<HTMLSpanElement>> & {
		/** Tailwind bg class for a leading dot, e.g. "bg-status-done". */
		dot?: string;
	} = $props();
</script>

<span
	bind:this={ref}
	data-slot="pill"
	class={cn(
		"bg-secondary text-muted-foreground [&_svg:not([class*='size-'])]:size-3 inline-flex w-fit items-center gap-[5px] rounded-full px-2 py-0.5 text-[11px] font-medium whitespace-nowrap",
		className
	)}
	{...restProps}
>
	{#if dot}<span class={cn("size-1.5 shrink-0 rounded-full", dot)}></span>{/if}
	{@render children?.()}
</span>
