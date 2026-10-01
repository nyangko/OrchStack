<script lang="ts">
	/// 다이얼로그 본문. tall 다이얼로그에서는 남은 높이를 채우고 스크롤한다. padded=false면 여백 없이(목록 · 2단 구성).
	import type { HTMLAttributes } from "svelte/elements";
	import { cn, type WithElementRef } from "$lib/utils.js";

	let {
		ref = $bindable(null),
		class: className,
		padded = true,
		children,
		...restProps
	}: WithElementRef<HTMLAttributes<HTMLDivElement>> & { padded?: boolean } = $props();
</script>

<div bind:this={ref} data-slot="dialog-body" class={cn("flex min-h-0 flex-1 flex-col overflow-y-auto", padded && "px-6 py-5", className)} {...restProps}>
	{@render children?.()}
</div>
