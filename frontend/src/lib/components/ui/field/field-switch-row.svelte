<script lang="ts">
	/// 설정 스위치 한 줄 (.pen SettingSwitchRow): 이름 · 설명 · 오른쪽 Switch. 아래 테두리.
	import type { HTMLLabelAttributes } from "svelte/elements";
	import { Switch } from "$lib/components/ui/switch/index.js";
	import { cn, type WithElementRef } from "$lib/utils.js";

	let {
		ref = $bindable(null),
		class: className,
		label,
		hint,
		checked = $bindable(false),
		disabled = false,
		...restProps
	}: WithElementRef<HTMLLabelAttributes> & {
		label: string;
		hint?: string;
		checked?: boolean;
		disabled?: boolean;
	} = $props();
</script>

<label bind:this={ref} data-slot="field-switch-row" class={cn("flex items-center gap-3 border-b py-2.5", className)} {...restProps}>
	<span class="flex flex-1 flex-col gap-0.5">
		<span class="text-xs font-semibold">{label}</span>
		{#if hint}<span class="text-caption text-muted-foreground">{hint}</span>{/if}
	</span>
	<Switch bind:checked {disabled} aria-label={label} />
</label>
