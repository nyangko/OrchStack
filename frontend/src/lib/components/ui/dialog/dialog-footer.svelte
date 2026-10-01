<script lang="ts">
	/// 다이얼로그 바닥 (.pen Dialog/Footer): 왼쪽 안내문(note) · 오른쪽 버튼(children).
	import type { Component } from "svelte";
	import type { HTMLAttributes } from "svelte/elements";
	import Info from "@lucide/svelte/icons/info";
	import { cn, type WithElementRef } from "$lib/utils.js";

	let {
		ref = $bindable(null),
		class: className,
		note,
		noteIcon = Info,
		children,
		...restProps
	}: WithElementRef<HTMLAttributes<HTMLDivElement>> & {
		/** 왼쪽 안내문. */
		note?: string;
		/** 안내문 아이콘 (기본 info). */
		noteIcon?: Component;
	} = $props();
</script>

<div bind:this={ref} data-slot="dialog-footer" class={cn("flex shrink-0 items-center gap-2.5 border-t px-6 py-3.5", className)} {...restProps}>
	{#if note}
		{@const NoteIcon = noteIcon}
		<span class="flex min-w-0 flex-1 items-center gap-1.5 text-caption text-muted-foreground"><NoteIcon class="size-3.5 shrink-0" />{note}</span>
	{:else}
		<span class="flex-1"></span>
	{/if}
	{@render children?.()}
</div>
