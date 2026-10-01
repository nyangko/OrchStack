<script lang="ts">
	/// 칸반 열 이름.
	import type { HTMLAttributes } from "svelte/elements";
	import { cn, type WithElementRef } from "$lib/utils.js";

	let { ref = $bindable(null), class: className, children, ...restProps }: WithElementRef<HTMLAttributes<HTMLHeadingElement>> = $props();
</script>

<h3 bind:this={ref} data-slot="kanban-column-title" class={cn("text-body font-semibold", className)} {...restProps}>
	{@render children?.()}
</h3>
