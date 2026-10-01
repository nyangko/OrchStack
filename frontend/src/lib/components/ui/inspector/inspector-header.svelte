<script lang="ts">
	/// 인스펙터 머리: 첫 줄(children: 타일/아바타 · 제목 · 버튼) + 닫기, 그 아래 sub(상태 칩 · 액션 줄 · 경고).
	import type { Snippet } from "svelte";
	import type { HTMLAttributes } from "svelte/elements";
	import XIcon from "@lucide/svelte/icons/x";
	import { Button } from "$lib/components/ui/button/index.js";
	import { cn, type WithElementRef } from "$lib/utils.js";

	let {
		ref = $bindable(null),
		class: className,
		onclose,
		closeLabel = "닫기",
		sub,
		children,
		...restProps
	}: WithElementRef<HTMLAttributes<HTMLElement>> & { onclose: () => void; closeLabel?: string; sub?: Snippet; children?: Snippet } = $props();
</script>

<header bind:this={ref} data-slot="inspector-header" class={cn("flex shrink-0 flex-col gap-3 border-b p-4", className)} {...restProps}>
	<div class="flex items-start gap-3">
		{@render children?.()}
		<Button variant="ghost" size="icon-sm" aria-label={closeLabel} onclick={onclose}><XIcon /></Button>
	</div>
	{@render sub?.()}
</header>
