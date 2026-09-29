<script lang="ts">
	import { useSortable } from "@dnd-kit-svelte/svelte/sortable";
	import type { Snippet } from "svelte";

	/** `group` is the id of the column the card sits in. */
	let { id, index, group, children }: { id: string; index: number; group: string; children: Snippet } = $props();

	const { ref, isDragSource } = useSortable({
		id: () => id,
		index: () => index,
		group: () => group,
		type: "item",
		accept: "item",
		data: () => ({ group })
	});
</script>

<div
	{@attach ref}
	class="bg-card text-card-foreground border-border w-full rounded-lg border text-sm shadow-xs {isDragSource.current ? 'opacity-40' : ''}"
>
	{@render children()}
</div>
