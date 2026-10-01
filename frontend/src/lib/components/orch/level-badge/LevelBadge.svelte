<script lang="ts">
	/**
	 * **판단 수준 배지 L0–L4** (.pen LevelBadge). 색은 수준의 뜻에 고정한다:
	 * L0 자동 · L1 알림 후 자동 · L2 사용자 판단 · L3 승인 · L4 차단.
	 */
	let {
		/** 0–4 */
		level
	}: { level: 0 | 1 | 2 | 3 | 4 } = $props();

	const tone = ['bg-status-done', 'bg-primary', 'bg-status-waiting', 'bg-status-review', 'bg-destructive'];
</script>

<span class={['inline-flex shrink-0 items-center rounded-xs px-1.5 py-px font-mono text-2xs font-bold text-on-solid', tone[level]]}>L{level}</span>
