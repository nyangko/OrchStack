<script lang="ts">
	/// 선택 창 항목. selected면 체크 표시, 고르면 onSelect 뒤 창을 닫는다(closeOnSelect=false면 열어 둠 · 여러 개 고를 때).
	import type { ComponentProps } from "svelte";
	import { CommandItem } from "$lib/components/ui/command";
	import { getCombobox } from "./context.js";

	let {
		selected = false,
		closeOnSelect = true,
		onSelect,
		children,
		...restProps
	}: ComponentProps<typeof CommandItem> & { selected?: boolean; closeOnSelect?: boolean } = $props();

	const close = getCombobox();
</script>

<CommandItem
	data-slot="combobox-item"
	data-checked={selected}
	onSelect={() => {
		onSelect?.();
		if (closeOnSelect) close();
	}}
	{...restProps}
>
	{@render children?.()}
</CommandItem>
