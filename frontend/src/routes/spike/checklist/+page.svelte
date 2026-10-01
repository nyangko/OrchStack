<script lang="ts">
	/// ui/checklist 스파이크 — 태스크와 무관한 데이터 · 읽기 전용 목록.
	import { Checklist, type ChecklistItem } from "$lib/components/orch/checklist/index.js";

	let groceries = $state<ChecklistItem[]>([{ text: "우유", done: true }, { text: "달걀", done: false }]);
	const steps: ChecklistItem[] = [{ text: "설치", done: true }, { text: "로그인", done: false }];
</script>

<main class="flex max-w-md flex-col gap-6 p-8">
	<Checklist label="장보기" placeholder="살 것 추가" bind:items={groceries} />
	<Checklist label="준비 단계" items={steps} sortable={false} editable={false} addable={false} />
	<p data-testid="out">{groceries.map((g) => `${g.done ? '✓' : '·'}${g.text}`).join(' ')}</p>
</main>
