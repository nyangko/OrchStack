<script lang="ts">
	/// 인스펙터 섹션 (.pen Inspector/SectionHeader + 내용): 대문자 작은 제목 · ValueRow들. 아래 테두리, 마지막은 없음.
	import type { HTMLAttributes } from "svelte/elements";
	import { cn, type WithElementRef } from "$lib/utils.js";

	let { ref = $bindable(null), class: className, title, children, ...restProps }: WithElementRef<HTMLAttributes<HTMLElement>> & { title: string } = $props();
</script>

<section bind:this={ref} data-slot="inspector-section" class={cn("flex flex-col gap-0.5 border-b px-4 py-3 text-xs last:border-b-0", className)} {...restProps}>
	<h3 class="pb-1 text-2xs font-semibold tracking-wider text-muted-foreground">{title}</h3>
	{@render children?.()}
</section>
