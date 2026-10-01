<script lang="ts">
	/// 칸반 카드 드래그 래퍼. 카드 자체가 버튼이라 클릭(선택)과 드래그를 같은 요소가 받는다.
	import { useSortable } from "@dnd-kit-svelte/svelte/sortable";
	import { KeyboardSensor, PointerSensor } from "@dnd-kit/dom";
	import type { Snippet } from "svelte";
	import type { HTMLButtonAttributes } from "svelte/elements";
	import { cn } from "$lib/utils.js";

	/** `group` is the id of the column the card sits in. */
	let {
		id,
		index,
		group,
		class: className,
		children,
		...restProps
	}: Omit<HTMLButtonAttributes, "id"> & { id: string | number; index: number; group: string; children: Snippet } = $props();

	const { ref, isDragSource } = useSortable({
		id: () => id,
		index: () => index,
		group: () => group,
		type: "item",
		accept: "item",
		data: () => ({ group }),
		// 기본값은 closest('button')이면 드래그를 막아 카드(버튼) 안 어디를 눌러도 막힌다.
		// 카드 안의 실제 입력 요소에서만 막는다. 클릭과 드래그 구분은 기본 제약(200ms 또는 5px)을 그대로 쓴다.
		sensors: [
			PointerSensor.configure({
				preventActivation: (e) => e.target instanceof Element && e.target.closest("input, select, textarea, a[href]") !== null
			}),
			KeyboardSensor
		]
	});
</script>

<button
	{@attach ref}
	type="button"
	class={cn(
		"kanban-card",
		isDragSource.current && "opacity-40",
		className
	)}
	{...restProps}
>
	{@render children()}
</button>
