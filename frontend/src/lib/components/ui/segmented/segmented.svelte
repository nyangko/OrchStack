<script lang="ts" module>
	import type { Component } from "svelte";

	export type SegmentedOption = { value: string; label: string; icon?: Component };
</script>

<script lang="ts">
	import { cn } from "$lib/utils.js";

	let {
		options,
		value = $bindable(),
		class: className,
	}: { options: SegmentedOption[]; value?: string; class?: string } = $props();
</script>

<div
	role="radiogroup"
	data-slot="segmented"
	class={cn("bg-muted flex w-full gap-0.5 rounded-sm p-[3px]", className)}
>
	{#each options as opt (opt.value)}
		{@const active = value === opt.value}
		<button
			type="button"
			role="radio"
			aria-checked={active}
			onclick={() => (value = opt.value)}
			class={cn(
				"flex flex-1 items-center justify-center gap-1 rounded-xs px-2.5 py-[5px] text-xs transition-colors outline-none focus-visible:ring-3 focus-visible:ring-ring/50",
				active
					? "bg-card text-foreground font-semibold shadow-xs"
					: "text-muted-foreground hover:text-foreground"
			)}
		>
			{#if opt.icon}<opt.icon class="size-[11px]" />{/if}
			{opt.label}
		</button>
	{/each}
</div>
