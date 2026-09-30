<script lang="ts">
	import { cn, type WithElementRef } from "$lib/utils.js";
	import type { HTMLAttributes } from "svelte/elements";

	let {
		ref = $bindable(null),
		class: className,
		children,
		...restProps
	}: WithElementRef<HTMLAttributes<HTMLElement>> = $props();
</script>

<kbd
	bind:this={ref}
	data-slot="kbd"
	class={cn(
		"bg-muted text-muted-foreground in-data-[slot=tooltip-content]:bg-primary-foreground/20 in-data-[slot=tooltip-content]:text-primary-foreground w-fit gap-1 rounded-xs px-1.5 py-px font-mono text-caption font-medium [&_svg:not([class*='size-'])]:size-3 pointer-events-none inline-flex items-center justify-center select-none",
		className
	)}
	{...restProps}
>
	{@render children?.()}
</kbd>
