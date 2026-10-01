<script lang="ts">
	/// 선택 카드 묶음 (.pen OptionCard/on · off). type="single"(기본) = 하나 고르기(라디오), "multiple" = 여러 개(체크 · value는 배열).
	/// 배치(grid · 열 수 · 간격)는 class로.
	import type { HTMLAttributes } from "svelte/elements";
	import { cn, type WithElementRef } from "$lib/utils.js";
	import { setChoice, type ChoiceValue } from "./context.js";

	let {
		ref = $bindable(null),
		type = "single",
		value = $bindable(),
		onValueChange,
		disabled = false,
		class: className,
		children,
		...restProps
	}: WithElementRef<HTMLAttributes<HTMLDivElement>> & {
		type?: "single" | "multiple";
		value: ChoiceValue | ChoiceValue[];
		onValueChange?: (value: ChoiceValue | ChoiceValue[]) => void;
		disabled?: boolean;
	} = $props();

	const list = () => (Array.isArray(value) ? value : []);
	setChoice({
		multiple: () => type === "multiple",
		has: (v) => (type === "multiple" ? list().includes(v) : value === v),
		toggle: (v) => {
			value = type === "multiple" ? (list().includes(v) ? list().filter((x) => x !== v) : [...list(), v]) : v;
			onValueChange?.(value);
		},
		disabled: () => disabled
	});
</script>

<div bind:this={ref} role={type === "multiple" ? "group" : "radiogroup"} data-slot="choice-cards" class={cn("grid gap-2.5", className)} {...restProps}>
	{@render children?.()}
</div>
