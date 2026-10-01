<script lang="ts">
	/** **Orch 카드 머리** — 아이콘 · 제목 · 오른쪽 꼬리(수준 배지 · 태그는 children). */
	import type { Component, Snippet } from 'svelte';

	let {
		/** 앞 아이콘 */
		icon: Icon,
		/** 아이콘 색 (예: text-status-done) */
		tone,
		/** 제목 한 줄 */
		title,
		children
	}: { icon: Component; tone: string; title: string; children?: Snippet } = $props();
</script>

<header class="flex items-center gap-2">
	<Icon class={['size-4 shrink-0', tone]} />
	<h3 class="min-w-0 flex-1 truncate text-sm font-semibold">{title}</h3>
	{@render children?.()}
</header>
