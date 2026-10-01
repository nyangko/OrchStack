<script lang="ts">
	/// 페이지 머리 (.pen SettingsPageHeader): 제목 · 설명 · 상태 문구(status) · 오른쪽 버튼(children).
	/// status는 "저장됨" 같은 짧은 확인 문구 — 비어 있으면 자리만 지킨다(role=status, 버튼이 밀리지 않게).
	import type { Component, Snippet } from "svelte";
	import type { HTMLAttributes } from "svelte/elements";
	import CircleCheck from "@lucide/svelte/icons/circle-check";
	import { cn, type WithElementRef } from "$lib/utils.js";

	let {
		ref = $bindable(null),
		class: className,
		title,
		desc,
		status,
		statusIcon = CircleCheck,
		children,
		...restProps
	}: WithElementRef<HTMLAttributes<HTMLElement>> & {
		title: string;
		desc?: string;
		/** 짧은 확인 문구. undefined면 자리만 유지, false면 자리도 없음. */
		status?: string | false;
		statusIcon?: Component;
		children?: Snippet;
	} = $props();
</script>

<header bind:this={ref} data-slot="page-header" class={cn("flex items-end gap-3", className)} {...restProps}>
	<div class="flex flex-1 flex-col gap-1">
		<h1 class="text-2xl font-semibold">{title}</h1>
		{#if desc}<p class="text-xs text-muted-foreground">{desc}</p>{/if}
	</div>
	{#if status !== false}
		{@const StatusIcon = statusIcon}
		<span class="flex items-center gap-1 text-xs font-medium text-status-done" role="status">
			{#if status}<StatusIcon class="size-3.25" />{status}{/if}
		</span>
	{/if}
	{@render children?.()}
</header>
