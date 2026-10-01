<script lang="ts">
	/// 사용량 · 한도 한 줄 (.pen Inspector/LimitRow). 아이콘 · 이름 · 사용/한도 · 막대 · 메모.
	/// warn이면 막힘색, mark는 경고 기준선 위치(%).
	import type { Component } from "svelte";
	import { Progress } from "$lib/components/ui/progress/index.js";
	import { cn } from "$lib/utils.js";

	let {
		icon: Icon,
		label,
		used,
		max,
		value,
		note,
		warn = false,
		mark,
		class: className,
	}: {
		icon: Component;
		label: string;
		used: string;
		max: string;
		/** 막대 채움 % (0–100). */
		value: number;
		note?: string;
		warn?: boolean;
		mark?: number;
		class?: string;
	} = $props();
</script>

<div data-slot="limit-row" class={cn("flex flex-col gap-1.5 py-1.5", className)}>
	<span class="flex items-center gap-2 text-xs">
		<Icon class="size-3.25 text-muted-foreground" />
		<span class="flex-1 text-muted-foreground">{label}</span>
		<span class={cn("font-mono font-medium", warn && "text-status-blocked")}>{used}</span>
		<span class="font-mono text-subtle-foreground">{max}</span>
	</span>
	<span class="relative">
		<Progress {value} class="h-1.5 bg-muted" indicator={cn("rounded-full", warn && "bg-status-blocked")} aria-label={label} />
		{#if mark}<span class="absolute top-0 h-full w-0.5 bg-card" style="left: {mark}%"></span>{/if}
	</span>
	{#if note}<span class={cn("text-caption", warn ? "text-status-blocked" : "text-muted-foreground")}>{note}</span>{/if}
</div>
