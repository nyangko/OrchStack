<script lang="ts">
	/// 인스펙터 카드 (.pen Inspector): 화면 안에 뜨는 비모달 상세 카드. Header · Body를 세로로 쌓는다.
	/// floating이면 부모 오른쪽 위에 340px로 떠 있고, 아니면 부모 높이를 채운다.
	import type { Snippet } from "svelte";
	import type { HTMLAttributes } from "svelte/elements";
	import { cn, type WithElementRef } from "$lib/utils.js";

	let {
		ref = $bindable(null),
		class: className,
		label,
		floating = false,
		children,
		...restProps
	}: WithElementRef<HTMLAttributes<HTMLDivElement>> & { label: string; floating?: boolean; children?: Snippet } = $props();
</script>

<div
	bind:this={ref}
	role="dialog"
	aria-modal="false"
	aria-label={label}
	data-slot="inspector"
	class={cn("flex flex-col overflow-hidden rounded-xl border bg-card shadow-lg", floating ? "absolute top-4 right-4 bottom-4 z-10 w-85" : "h-full", className)}
	{...restProps}
>
	{@render children?.()}
</div>
