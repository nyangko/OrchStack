<script lang="ts">
	/// 칸반 보드 (.pen Kanban Board). 열 → 항목 id 목록(value)을 들고, 끌어 놓기 · 순서 이동을 맡는다.
	/// 놓으면 value가 바뀐 뒤 onDragEnd(value)를 부른다 — 항목 상태 반영은 호출부가 한다.
	import type { HTMLAttributes } from "svelte/elements";
	import { DragDropProvider } from "@dnd-kit-svelte/svelte";
	import { move } from "@dnd-kit/helpers";
	import { cn, type WithElementRef } from "$lib/utils.js";
	import { setBoard, type KanbanValue } from "./context.js";

	let {
		ref = $bindable(null),
		value = $bindable(),
		onDragEnd,
		class: className,
		children,
		...restProps
	}: WithElementRef<HTMLAttributes<HTMLDivElement>> & {
		value: KanbanValue;
		onDragEnd?: (value: KanbanValue) => void;
	} = $props();

	setBoard(() => value);
</script>

<DragDropProvider
	onDragOver={(e) => (value = move(value, e))}
	onDragEnd={(e) => {
		if (e.canceled) return;
		value = move(value, e);
		onDragEnd?.(value);
	}}
>
	<div bind:this={ref} data-slot="kanban" class={cn("flex h-full items-start gap-3 p-4", className)} {...restProps}>
		{@render children?.()}
	</div>
</DragDropProvider>
