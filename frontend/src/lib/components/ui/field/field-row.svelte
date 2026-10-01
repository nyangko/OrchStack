<script lang="ts">
	/// 폼 한 줄 (.pen FormRow): 라벨 · 힌트 · 컨트롤(children) · 오류. 아래 테두리.
	/// 기본은 div — Select를 <label>로 감싸면 트리거 클릭이 두 번 전달돼 목록이 다시 열린다. Input 한 개만 감쌀 땐 as="label".
	import type { Snippet } from "svelte";
	import type { HTMLAttributes } from "svelte/elements";
	import { cn, type WithElementRef } from "$lib/utils.js";

	let {
		ref = $bindable(null),
		class: className,
		as = "div",
		label,
		hint,
		error,
		children,
		...restProps
	}: WithElementRef<HTMLAttributes<HTMLElement>> & {
		as?: "div" | "label";
		label: string;
		hint?: string;
		/** 있으면 컨트롤 아래 빨간 글씨. */
		error?: string;
		children?: Snippet;
	} = $props();
</script>

<svelte:element this={as} bind:this={ref} data-slot="field-row" class={cn("flex flex-col gap-2.5 border-b py-4", className)} {...restProps}>
	<span class="flex items-baseline gap-2">
		<span class="text-xs font-semibold">{label}</span>
		{#if hint}<span class="text-caption text-muted-foreground">{hint}</span>{/if}
	</span>
	{@render children?.()}
	{#if error}<span class="text-caption text-destructive">{error}</span>{/if}
</svelte:element>
