<script lang="ts">
	/// 다이얼로그 머리 (.pen Dialog/Header): 아이콘(icon · lead) · 제목/설명(children) · 오른쪽 actions · 닫기.
	/// crumb는 제목 위 작은 경로(예: 팀 이름 › 팀 설정), sub는 머리 아래 한 줄(단계 표시 · 검색 · 필터).
	import type { Component, Snippet } from "svelte";
	import type { HTMLAttributes } from "svelte/elements";
	import { Dialog as DialogPrimitive } from "bits-ui";
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
		/** 제목 왼쪽 아이콘. */
		icon?: Component;
		/** 아이콘 대신 그릴 것 (아바타 · 글자 타일). */
		lead?: Snippet;
		/** 제목 위 경로. */
		crumb?: string[];
		/** 오른쪽 버튼 (취소 · 저장 · 링크). */
		actions?: Snippet;
		/** 머리 아래 한 줄. */
		sub?: Snippet;
		/** 닫기 버튼 표시. actions가 닫기를 대신하면 끈다. */
		closable?: boolean;
	} = $props();
</script>

<div bind:this={ref} data-slot="dialog-header" class={cn("flex shrink-0 flex-col border-b", className)} {...restProps}>
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
			<DialogPrimitive.Close data-slot="dialog-close">
				{#snippet child({ props })}
					<Button variant="ghost" size="icon-sm" aria-label="닫기" {...props}><XIcon /></Button>
				{/snippet}
			</DialogPrimitive.Close>
		{/if}
	</div>
	{#if sub}
		<div class="px-6 pb-4">{@render sub()}</div>
	{/if}
</div>
