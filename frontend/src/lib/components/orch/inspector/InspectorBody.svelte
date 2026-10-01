<script lang="ts">
	/// 인스펙터 본문: 남은 높이를 채우고 세로 스크롤. 2단 구성이면 class="flex-row".
	import type { HTMLAttributes } from "svelte/elements";
	import { cn, type WithElementRef } from "$lib/utils.js";

	let { ref = $bindable(null), class: className, children, ...restProps }: WithElementRef<HTMLAttributes<HTMLDivElement>> = $props();
</script>

<div bind:this={ref} data-slot="inspector-body" class={cn("flex min-h-0 flex-1 flex-col overflow-y-auto", className)} {...restProps}>
	{@render children?.()}
</div>
