<script lang="ts">
	/**
	 * **보드의 상태 열 하나** — 카드를 놓을 수 있는 곳. 끌어 오는 중이면 `data-drop-target`으로 바탕을 칠한다.
	 * `TaskBoard` 안에서만 쓴다. 열마다 useDroppable 하나가 필요해 따로 둔 파일이고, 머리 · 카드 목록은 TaskBoard가 children으로 그린다.
	 */
	import type { Snippet } from 'svelte';
	import { useDroppable } from '@dnd-kit-svelte/svelte';
	import { CollisionPriority } from '@dnd-kit/abstract';
	import type { TaskStatus } from '$lib/status';

	let {
		/** 이 열의 상태 — 놓으면 태스크가 이 상태가 된다 */
		status,
		children
	}: { status: TaskStatus; children: Snippet } = $props();

	const { ref, isDropTarget } = useDroppable({ id: () => status, type: 'column', accept: 'item', collisionPriority: CollisionPriority.Low });
</script>

<section {@attach ref} data-drop-target={isDropTarget.current || undefined} class="task-board-lane data-drop-target:bg-primary-soft">
	{@render children()}
</section>
