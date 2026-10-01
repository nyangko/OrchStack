<script lang="ts">
	/// 선택 창 트리거 — 호출부가 그린 요소(child)에 열기 props를 펼친다.
	import type { Snippet } from "svelte";
	import { PopoverTrigger } from "$lib/components/ui/popover";

	let { child }: { child: Snippet<[{ props: Record<string, unknown> }]> } = $props();
</script>

<PopoverTrigger {child} />
