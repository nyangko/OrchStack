<script lang="ts" module>
	import { tv, type VariantProps } from "tailwind-variants";

	/// 카드 모양: column = 세로(제목 · 설명), row = 가로(아이콘 · 글 · 끝). 고르면 .pen OptionCard/on (option-on).
	export const choiceCardVariants = tv({
		base: "rounded-md border text-left outline-none hover:bg-muted focus-visible:ring-3 focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50 data-[state=checked]:option-on",
		variants: {
			layout: {
				column: "flex flex-col gap-1 p-3",
				row: "flex items-center gap-3 px-3 py-2.5",
			},
			tone: {
				default: "",
				// 안 고른 카드를 흐리게 (.pen 첫 실행 · 기본 팀 — 체크 해제하면 빼고 시작)
				dim: "data-[state=unchecked]:bg-muted/50 data-[state=unchecked]:opacity-60 data-[state=unchecked]:hover:opacity-100",
			},
		},
		defaultVariants: { layout: "column", tone: "default" },
	});
	export type ChoiceCardLayout = VariantProps<typeof choiceCardVariants>["layout"];
	export type ChoiceCardTone = VariantProps<typeof choiceCardVariants>["tone"];
</script>

<script lang="ts">
	/// 선택 카드 하나 — 누르면 고른다(single) 또는 넣고 뺀다(multiple). 안 내용은 자유(제목 · 설명 · 아이콘).
	import type { HTMLButtonAttributes } from "svelte/elements";
	import { cn, type WithElementRef } from "$lib/utils.js";
	import { getChoice, type ChoiceValue } from "./context.js";

	let {
		ref = $bindable(null),
		value,
		layout = "column",
		tone = "default",
		disabled = false,
		onclick,
		class: className,
		children,
		...restProps
	}: WithElementRef<Omit<HTMLButtonAttributes, "value">> & { value: ChoiceValue; layout?: ChoiceCardLayout; tone?: ChoiceCardTone } = $props();

	const choice = getChoice();
	const checked = $derived(choice.has(value));
</script>

<button
	bind:this={ref}
	type="button"
	role={choice.multiple() ? "checkbox" : "radio"}
	aria-checked={checked}
	data-slot="choice-card"
	data-state={checked ? "checked" : "unchecked"}
	disabled={disabled || choice.disabled()}
	class={cn(choiceCardVariants({ layout, tone }), className)}
	onclick={(e) => {
		choice.toggle(value);
		onclick?.(e);
	}}
	{...restProps}
>
	{@render children?.()}
</button>
