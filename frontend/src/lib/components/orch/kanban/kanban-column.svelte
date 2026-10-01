<script lang="ts">
	import { useDroppable } from "@dnd-kit-svelte/svelte";
	import { CollisionPriority } from "@dnd-kit/abstract";
	import type { Snippet } from "svelte";

	let { id, children }: { id: string; children: Snippet } = $props();

	const { ref, isDropTarget } = useDroppable({
		id: () => id,
		type: "column",
		accept: "item",
		collisionPriority: CollisionPriority.Low
	});
</script>

<section
	{@attach ref}
	class="kanban-column {isDropTarget.current ? 'bg-primary-soft' : ''}"
>
	{@render children()}
</section>
