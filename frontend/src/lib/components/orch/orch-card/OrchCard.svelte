<script lang="ts">
	/**
	 * **Orch 진행 카드 틀** (.pen OrchCard · PM Dock Cards A–H). 테두리 색이 카드의 뜻을 말한다.
	 * 머리는 `OrchCardHeader`, 본문 · 버튼 줄은 children으로 그 자리에서 짠다.
	 */
	import type { Snippet } from 'svelte';
	import type { HTMLAttributes } from 'svelte/elements';

	let {
		/** 테두리 · 바탕 — default 회색 · primary 진행 · waiting 대기/경고 · review 승인 · done 완료 · danger 정지/차단(붉은 바탕) */
		tone = 'default',
		children,
		...rest
	}: HTMLAttributes<HTMLElement> & {
		tone?: 'default' | 'primary' | 'waiting' | 'review' | 'done' | 'danger';
		children: Snippet;
	} = $props();

	const tones = {
		default: '',
		primary: 'border-primary',
		waiting: 'border-status-waiting',
		review: 'border-status-review',
		done: 'border-status-done',
		danger: 'border-destructive bg-destructive-soft'
	};
</script>

<article {...rest} class={['card flex flex-col gap-3 rounded-lg p-3.5', tones[tone]]}>
	{@render children()}
</article>
