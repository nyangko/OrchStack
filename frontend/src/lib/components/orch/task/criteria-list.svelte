<script lang="ts">
	/// 완료 조건 목록 (.pen TaskEditor · Acceptance criteria / B · Task Detail 인라인).
	/// 체크 · 끌어서 순서 바꾸기 · hover에 편집 · 삭제 · 마지막 줄에서 추가(Enter). 끝난 조건은 흐리게.
	import GripVertical from '@lucide/svelte/icons/grip-vertical';
	import Square from '@lucide/svelte/icons/square';
	import SquareCheck from '@lucide/svelte/icons/square-check';
	import Pencil from '@lucide/svelte/icons/pencil';
	import Trash2 from '@lucide/svelte/icons/trash-2';
	import Plus from '@lucide/svelte/icons/plus';
	import { cn } from '$lib/utils';

	type Criterion = { text: string; done: boolean };
	let { items = $bindable(), placeholder = '조건 추가' }: { items: Criterion[]; placeholder?: string } = $props();

	let draft = $state('');
	let editing = $state<number>();
	let dragFrom = $state<number>();

	function add(e: KeyboardEvent) {
		if (e.key !== 'Enter' || e.isComposing || !draft.trim()) return;
		e.preventDefault();
		items = [...items, { text: draft.trim(), done: false }];
		draft = '';
	}
	function drop(to: number) {
		if (dragFrom === undefined || dragFrom === to) return;
		const next = [...items];
		const [moved] = next.splice(dragFrom, 1);
		next.splice(to, 0, moved);
		items = next;
		dragFrom = undefined;
	}
</script>

<ul class="flex flex-col gap-0.5" aria-label="완료 조건">
	{#each items as c, i (i)}
		<li
			class={cn('group flex items-center gap-2 rounded-sm px-1.5 py-1.25 hover:bg-muted/60', dragFrom === i && 'opacity-50')}
			draggable={editing !== i}
			ondragstart={() => (dragFrom = i)}
			ondragover={(e) => e.preventDefault()}
			ondrop={() => drop(i)}
		>
			<GripVertical class="size-3.25 shrink-0 cursor-grab text-transparent group-hover:text-subtle-foreground" aria-hidden="true" />
			<button type="button" role="checkbox" aria-checked={c.done} aria-label={c.text} onclick={() => (c.done = !c.done)} class="shrink-0 outline-none focus-visible:ring-3 focus-visible:ring-ring/50">
				{#if c.done}<SquareCheck class="size-3.75 text-status-done" />{:else}<Square class="size-3.75 text-muted-foreground" />{/if}
			</button>
			{#if editing === i}
				<!-- svelte-ignore a11y_autofocus -->
				<input
					autofocus
					bind:value={c.text}
					aria-label="조건 고치기"
					onblur={() => (editing = undefined)}
					onkeydown={(e) => (e.key === 'Enter' || e.key === 'Escape') && !e.isComposing && (e.preventDefault(), (editing = undefined))}
					class="min-w-0 flex-1 bg-transparent text-body outline-none"
				/>
			{:else}
				<span class={cn('min-w-0 flex-1 text-body', c.done && 'text-muted-foreground')}>{c.text}</span>
				<span class="flex items-center gap-1.5 opacity-0 group-focus-within:opacity-100 group-hover:opacity-100">
					<button type="button" aria-label="조건 고치기" onclick={() => (editing = i)} class="text-muted-foreground hover:text-foreground"><Pencil class="size-3.25" /></button>
					<button type="button" aria-label="조건 지우기" onclick={() => (items = items.filter((_, k) => k !== i))} class="text-muted-foreground hover:text-destructive"><Trash2 class="size-3.25" /></button>
				</span>
			{/if}
		</li>
	{/each}
	<li class="flex items-center gap-2 rounded-sm px-1.5 py-1 focus-within:ring-1 focus-within:ring-ring">
		<span class="size-3.25 shrink-0"></span>
		<Plus class="size-3.75 shrink-0 text-muted-foreground" />
		<input bind:value={draft} onkeydown={add} {placeholder} aria-label="완료 조건 추가" class="min-w-0 flex-1 bg-transparent text-body outline-none placeholder:text-muted-foreground" />
	</li>
</ul>
