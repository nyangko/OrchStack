<script lang="ts" module>
	import type { Component } from "svelte";

	export type SegmentedOption = { value: string; label: string; icon?: Component };
</script>

<script lang="ts">
	import { cn } from "$lib/utils.js";

	let {
		options,
		value = $bindable(),
		disabled = false,
		class: className,
		...rest
	}: {
		options: SegmentedOption[];
		value?: string;
		/** 값을 보여주기만 하고 바꿀 수 없게 한다 (예: 고정 정책). */
		disabled?: boolean;
		class?: string;
		"aria-label"?: string;
	} = $props();
</script>

<div
	role="radiogroup"
	aria-disabled={disabled || undefined}
	{...rest}
	data-slot="segmented"
	class={cn("bg-muted flex w-full gap-0.5 rounded-sm p-[3px]", className)}
>
	{#each options as opt (opt.value)}
		{@const active = value === opt.value}
		<button
			type="button"
			role="radio"
			aria-checked={active}
			{disabled}
			onclick={() => (value = opt.value)}
			class={cn(
				"flex flex-1 items-center justify-center gap-1 rounded-xs px-2.5 py-[5px] text-xs transition-colors outline-none focus-visible:ring-3 focus-visible:ring-ring/50 disabled:cursor-not-allowed disabled:opacity-60",
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
