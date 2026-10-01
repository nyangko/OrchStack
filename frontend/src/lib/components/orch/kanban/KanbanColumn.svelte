<script lang="ts">
	/// 칸반 열 (.pen KanbanColumn) — 항목을 놓을 수 있는 곳. 끌어 오는 중이면 data-drop-target.
	import type { HTMLAttributes } from "svelte/elements";
	import { useDroppable } from "@dnd-kit-svelte/svelte";
	import { CollisionPriority } from "@dnd-kit/abstract";
	import { cn } from "$lib/utils.js";
	import { setColumn } from "./context.js";

	let { value, class: className, children, ...restProps }: HTMLAttributes<HTMLElement> & { value: string } = $props();

	setColumn(() => value);
	const { ref, isDropTarget } = useDroppable({ id: () => value, type: "column", accept: "item", collisionPriority: CollisionPriority.Low });
</script>

<section
	{@attach ref}
	data-slot="kanban-column"
	data-drop-target={isDropTarget.current || undefined}
	class={cn("flex w-85 shrink-0 flex-col gap-2 rounded-xl p-1.5 transition-colors data-drop-target:bg-primary-soft", className)}
	{...restProps}
>
	{@render children?.()}
</section>
