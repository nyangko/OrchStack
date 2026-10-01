<script lang="ts">
	/// 노드 카드 (.pen DiagramNode · SubRunNode) — 캔버스 위 카드. selected면 파란 테두리, dashed면 점선(가벼운 노드).
	/// Header(Kind · Ref · 동작) · Title · 내용 · Footer로 조립한다. 연결 손잡이 등 캔버스 일은 호출부가.
	import type { HTMLAttributes } from "svelte/elements";
	import { cn, type WithElementRef } from "$lib/utils.js";

	let {
		ref = $bindable(null),
		selected = false,
		dashed = false,
		class: className,
		children,
		...restProps
	}: WithElementRef<HTMLAttributes<HTMLDivElement>> & { selected?: boolean; dashed?: boolean } = $props();
</script>

<div
	bind:this={ref}
	data-slot="node-card"
	data-selected={selected || undefined}
	class={cn("flex w-55 flex-col gap-2 rounded-lg border bg-card p-3 text-left shadow-sm", dashed && "border-dashed border-input", selected && "ring-2 ring-primary", className)}
	{...restProps}
>
	{@render children?.()}
</div>
