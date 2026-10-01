<script lang="ts">
	/// 검색 선택 창 (.pen C · Assignee · Depends Picker) — 누르면 뜨는 검색 목록. Popover + Command 조립을 한 번에.
	/// Root · Trigger(child) · Content(header 슬롯) · Search · List · Empty · Group · Item(selected) · Separator.
	import type { Snippet } from "svelte";
	import * as Popover from "$lib/components/ui/popover/index.js";
	import { setCombobox } from "./context.js";

	let { open = $bindable(false), children }: { open?: boolean; children?: Snippet } = $props();

	setCombobox(() => (open = false));
</script>

<Popover.Root bind:open>
	{@render children?.()}
</Popover.Root>
