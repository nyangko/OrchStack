<script lang="ts">
	/// Task 상태 선택 (.pen StatusSelect + StatusMenu). 트리거는 현재 상태 색, 메뉴는 전체 상태 목록.
	import * as Select from "$lib/components/ui/select/index.js";
	import ChevronDown from "@lucide/svelte/icons/chevron-down";
	import Check from "@lucide/svelte/icons/check";
	import { statuses, statusOrder, type TaskStatus } from "$lib/status.js";
	import { cn } from "$lib/utils.js";

	let {
		value = $bindable(),
		disabled = false,
		class: className,
		onValueChange,
	}: {
		value: TaskStatus;
		disabled?: boolean;
		class?: string;
		onValueChange?: (value: TaskStatus) => void;
	} = $props();

	const current = $derived(statuses[value]);
</script>

<Select.Root
	type="single"
	bind:value={() => value, (v) => (value = v as TaskStatus)}
	onValueChange={(v) => onValueChange?.(v as TaskStatus)}
	{disabled}
>
	<Select.Trigger
		aria-label="Status"
		class={cn(
			"data-[size=default]:h-auto gap-1 rounded-md border-0 py-0.5 pr-1.5 pl-2 font-medium shadow-none [&_svg:last-child]:hidden",
			current.soft,
			current.text,
			className
		)}
	>
		<current.icon class="size-3" />
		{current.label}
		<ChevronDown class="size-3" />
	</Select.Trigger>
	<Select.Content class="w-47 p-1">
		{#each statusOrder as s (s)}
			{@const m = statuses[s]}
			<Select.Item value={s} label={m.label} class="h-7 gap-2 rounded-xs px-2 text-body [&>span:first-child]:hidden">
				<m.icon class={cn("size-3.5", m.text)} />
				<span class="flex-1">{m.label}</span>
				{#if s === value}<Check class="size-3.5" />{/if}
			</Select.Item>
		{/each}
	</Select.Content>
</Select.Root>
