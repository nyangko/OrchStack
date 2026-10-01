<script lang="ts">
	/// 시트 머리 — 다이얼로그 머리(.pen Dialog/Header)와 같은 구조: 아이콘(icon · lead) · 제목/설명(children) · actions · 닫기 · sub.
	import type { Component, Snippet } from "svelte";
	import type { HTMLAttributes } from "svelte/elements";
	import { Dialog as SheetPrimitive } from "bits-ui";
	import XIcon from "@lucide/svelte/icons/x";
	import ChevronRight from "@lucide/svelte/icons/chevron-right";
	import { Button } from "$lib/components/ui/button/index.js";
	import { cn, type WithElementRef } from "$lib/utils.js";

	let {
		ref = $bindable(null),
		class: className,
		icon,
		lead,
		crumb,
		actions,
		sub,
		closable = true,
		children,
		...restProps
	}: WithElementRef<HTMLAttributes<HTMLDivElement>> & {
		icon?: Component;
		lead?: Snippet;
		crumb?: string[];
		actions?: Snippet;
		sub?: Snippet;
		closable?: boolean;
	} = $props();
</script>

<div bind:this={ref} data-slot="sheet-header" class={cn("flex shrink-0 flex-col border-b", className)} {...restProps}>
	<div class="flex items-center gap-3 px-6 py-5">
		{#if lead}
			{@render lead()}
		{:else if icon}
			{@const Icon = icon}
			<Icon class="size-4.5 shrink-0" />
		{/if}
		<div class="flex min-w-0 flex-1 flex-col gap-0.5">
			{#if crumb?.length}
				<span class="flex items-center gap-1 text-caption text-muted-foreground">
					{#each crumb as c, i (i)}{#if i}<ChevronRight class="size-3" />{/if}{c}{/each}
				</span>
			{/if}
			{@render children?.()}
		</div>
		{@render actions?.()}
		{#if closable}
			<SheetPrimitive.Close data-slot="sheet-close">
				{#snippet child({ props })}
					<Button variant="ghost" size="icon-sm" aria-label="닫기" {...props}><XIcon /></Button>
				{/snippet}
			</SheetPrimitive.Close>
		{/if}
	</div>
	{#if sub}
		<div class="px-6 pb-4">{@render sub()}</div>
	{/if}
</div>
