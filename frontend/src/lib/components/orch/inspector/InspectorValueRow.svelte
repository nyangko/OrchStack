<script lang="ts">
	/// 인스펙터 값 한 줄 (.pen Inspector/ValueRow, 28px): 아이콘 · 회색 라벨 · 값. 값을 꾸밀 땐 children.
	import type { Component, Snippet } from "svelte";
	import type { HTMLAttributes } from "svelte/elements";
	import { cn, type WithElementRef } from "$lib/utils.js";

	let {
		ref = $bindable(null),
		class: className,
		icon: Icon,
		label,
		value,
		children,
		...restProps
	}: WithElementRef<HTMLAttributes<HTMLDivElement>> & { icon: Component; label: string; value?: string; children?: Snippet } = $props();
</script>

<div bind:this={ref} data-slot="inspector-value-row" class={cn("flex h-7 items-center gap-2", className)} {...restProps}>
	<Icon class="size-3.5 shrink-0 text-muted-foreground" />
	<span class="flex-1 text-muted-foreground">{label}</span>
	{#if children}{@render children()}{:else}<span class="font-medium">{value}</span>{/if}
</div>
