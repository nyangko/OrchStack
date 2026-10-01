<script lang="ts">
	/// 이력 한 줄 (.pen HistoryRow): 둥근 아이콘 타일 · 종류(kind) · 누가 · 시각 · 내용. 위 테두리.
	/// tone은 타일 색 (예: "bg-primary-soft text-primary").
	import type { Component } from "svelte";
	import type { HTMLAttributes } from "svelte/elements";
	import { cn, type WithElementRef } from "$lib/utils.js";

	let {
		ref = $bindable(null),
		class: className,
		icon: Icon,
		tone = "bg-primary-soft text-primary",
		kind,
		who,
		when,
		text,
		...restProps
	}: WithElementRef<HTMLAttributes<HTMLDivElement>> & {
		icon: Component;
		tone?: string;
		kind: string;
		who?: string;
		when?: string;
		text: string;
	} = $props();
</script>

<div bind:this={ref} data-slot="history-row" class={cn("flex gap-2.5 border-t py-2.5", className)} {...restProps}>
	<span class={cn("flex size-6 shrink-0 items-center justify-center rounded-full", tone)}><Icon class="size-3" /></span>
	<span class="flex min-w-0 flex-1 flex-col gap-0.5 text-xs">
		<span class="flex items-center gap-1.5">
			<span class="font-mono text-caption font-semibold text-muted-foreground">{kind}</span>
			{#if who}<span class="text-muted-foreground">{who}</span>{/if}
			{#if when}<span class="ml-auto text-caption text-subtle-foreground">{when}</span>{/if}
		</span>
		<span>{text}</span>
	</span>
</div>
