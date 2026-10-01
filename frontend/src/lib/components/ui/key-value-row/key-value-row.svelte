<script lang="ts">
	/// 키 · 값 한 줄 (.pen KeyValueRow, 30px): 왼쪽 회색 라벨 · 오른쪽 값. 위 테두리. 값을 꾸밀 땐 children.
	import type { Snippet } from "svelte";
	import type { HTMLAttributes } from "svelte/elements";
	import { cn, type WithElementRef } from "$lib/utils.js";

	let {
		ref = $bindable(null),
		class: className,
		label,
		value,
		children,
		...restProps
	}: WithElementRef<HTMLAttributes<HTMLDivElement>> & { label: string; value?: string; children?: Snippet } = $props();
</script>

<div bind:this={ref} data-slot="key-value-row" class={cn("flex h-7.5 items-center border-t text-xs", className)} {...restProps}>
	<span class="flex-1 text-muted-foreground">{label}</span>
	{#if children}{@render children()}{:else}<span class="font-medium">{value}</span>{/if}
</div>
