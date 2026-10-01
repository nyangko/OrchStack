<script lang="ts">
	/// 선택 창 몸체 — 떠 있는 상자 + 검색 목록. header는 목록 위(전환 탭 등), 너비 · 여백은 class로.
	/// data-slot은 Popover 것(popover-content)을 그대로 둔다 — 다른 스타일이 그 이름에 걸려 있다.
	import type { Snippet } from "svelte";
	import { PopoverContent } from "$lib/components/ui/popover";
	import { Command } from "$lib/components/ui/command";
	import { cn } from "$lib/utils.js";

	let {
		class: className,
		align = "start",
		header,
		children
	}: { class?: string; align?: "start" | "center" | "end"; header?: Snippet; children?: Snippet } = $props();
</script>

<PopoverContent {align} class={cn("w-72 gap-0 p-0", className)}>
	{@render header?.()}
	<Command>
		{@render children?.()}
	</Command>
</PopoverContent>
