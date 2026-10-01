<script lang="ts">
	/// 칸반 카드 (.pen KanbanCard) — 끌 수 있는 카드 버튼. 열 · 순서는 보드 value에서 찾는다.
	/// 메뉴 트리거 등에서 받은 props는 그대로 펼친다(...restProps) — data-slot은 펼친 뒤에 둬서 덮이지 않게. 카드 안은 KanbanCardHeader · KanbanCardContent · KanbanCardFooter로 조립.
	import type { HTMLButtonAttributes } from "svelte/elements";
	import { useSortable } from "@dnd-kit-svelte/svelte/sortable";
	import { KeyboardSensor, PointerSensor } from "@dnd-kit/dom";
	import { cn } from "$lib/utils.js";
	import { getBoard, getColumn, type KanbanId } from "./context.js";

	let { value, class: className, children, ...restProps }: Omit<HTMLButtonAttributes, "value"> & { value: KanbanId } = $props();

	const board = getBoard();
	const column = getColumn();
	const index = $derived(board()[column()]?.indexOf(value) ?? 0);

	const { ref, isDragSource } = useSortable({
		id: () => value,
		index: () => index,
		group: () => column(),
		type: "item",
		accept: "item",
		data: () => ({ group: column() }),
		// 기본값은 closest('button')이면 끌기를 막아 카드(버튼) 안 어디를 눌러도 막힌다 → 실제 입력 요소에서만 막는다.
		// 클릭과 끌기 구분은 기본 제약(200ms 또는 5px).
		sensors: [
			PointerSensor.configure({ preventActivation: (e) => e.target instanceof Element && e.target.closest("input, select, textarea, a[href]") !== null }),
			KeyboardSensor
		]
	});
</script>

<button
	{@attach ref}
	type="button"
	data-dragging={isDragSource.current || undefined}
	class={cn(
		"w-full rounded-lg border border-border bg-card text-left text-sm text-card-foreground shadow-xs outline-none focus-visible:ring-3 focus-visible:ring-ring/50 data-dragging:opacity-40 aria-pressed:ring-2 aria-pressed:ring-primary",
		className
	)}
	{...restProps}
	data-slot="kanban-card"
>
	{@render children?.()}
</button>
