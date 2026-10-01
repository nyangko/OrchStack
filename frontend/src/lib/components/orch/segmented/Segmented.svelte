<script lang="ts" module>
	import type { Component } from "svelte";

	/** disabled 옵션은 고를 수 없고, hint가 있으면 마우스를 올렸을 때 툴팁으로 이유를 보여준다. */
	export type SegmentedOption = { value: string; label: string; icon?: Component; disabled?: boolean; hint?: string };
</script>

<script lang="ts">
	import { cn } from "$lib/utils.js";
	import * as Tooltip from "$lib/components/ui/tooltip";

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
		{@const off = disabled || !!opt.disabled}
		{#if opt.disabled && opt.hint}
			<!-- disabled 속성을 주면 마우스 이벤트가 안 와서 툴팁이 못 뜬다 → aria-disabled로만 막는다 -->
			<Tooltip.Provider delayDuration={150}>
				<Tooltip.Root>
					<Tooltip.Trigger>
						{#snippet child({ props })}{@render seg(opt, active, off, props)}{/snippet}
					</Tooltip.Trigger>
					<Tooltip.Content>{opt.hint}</Tooltip.Content>
				</Tooltip.Root>
			</Tooltip.Provider>
		{:else}
			{@render seg(opt, active, off, {})}
		{/if}
	{/each}
</div>

{#snippet seg(opt: SegmentedOption, active: boolean, off: boolean, props: Record<string, unknown>)}
	<button
		{...props}
		type="button"
		role="radio"
		aria-checked={active}
		aria-disabled={off || undefined}
		disabled={off && !opt.hint}
		onclick={() => {
			if (!off) value = opt.value;
		}}
		class={cn(
			"flex flex-1 items-center justify-center gap-1 rounded-xs px-2.5 py-[5px] text-xs whitespace-nowrap transition-colors outline-none focus-visible:ring-3 focus-visible:ring-ring/50 disabled:cursor-not-allowed disabled:opacity-60 aria-disabled:cursor-not-allowed aria-disabled:opacity-40",
			active ? "bg-card text-foreground font-semibold shadow-xs" : "text-muted-foreground hover:text-foreground"
		)}
	>
		{#if opt.icon}<opt.icon class="size-[11px]" />{/if}
		{opt.label}
	</button>
{/snippet}
