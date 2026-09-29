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
	class="flex w-75 shrink-0 flex-col gap-2 rounded-xl p-1.5 transition-colors {isDropTarget.current ? 'bg-primary-soft' : ''}"
>
	{@render children()}
</section>
