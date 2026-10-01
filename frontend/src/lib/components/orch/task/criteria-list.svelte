<script lang="ts">
	/// 완료 조건 목록 (.pen TaskEditor · Acceptance criteria / B · Task Detail 인라인).
	/// 체크 · 끌어서 순서 바꾸기 · hover에 편집 · 삭제 · 마지막 줄에서 추가(Enter). 끝난 조건은 흐리게.
	import GripVertical from '@lucide/svelte/icons/grip-vertical';
	import Pencil from '@lucide/svelte/icons/pencil';
	import Trash2 from '@lucide/svelte/icons/trash-2';
	import Plus from '@lucide/svelte/icons/plus';
	import { Checkbox } from '$lib/components/ui/checkbox';
	import { Button } from '$lib/components/ui/button';
	import * as InputGroup from '$lib/components/ui/input-group';
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
			class={cn('criterion-row group', dragFrom === i && 'opacity-50')}
			draggable={editing !== i}
			ondragstart={() => (dragFrom = i)}
			ondragover={(e) => e.preventDefault()}
			ondrop={() => drop(i)}
		>
			<GripVertical class="size-3.25 shrink-0 cursor-grab text-transparent group-hover:text-subtle-foreground" aria-hidden="true" />
			<Checkbox bind:checked={c.done} aria-label={c.text} />
			{#if editing === i}
				<InputGroup.Root class="h-7 flex-1">
					<!-- svelte-ignore a11y_autofocus -->
					<InputGroup.Input
						autofocus
						bind:value={c.text}
						aria-label="조건 고치기"
						onblur={() => (editing = undefined)}
						onkeydown={(e) => (e.key === 'Enter' || e.key === 'Escape') && !e.isComposing && (e.preventDefault(), (editing = undefined))}
					/>
				</InputGroup.Root>
			{:else}
				<span class={cn('min-w-0 flex-1 text-body', c.done && 'text-muted-foreground')}>{c.text}</span>
				<span class="hover-actions">
					<Button variant="ghost" size="icon-xs" aria-label="조건 고치기" onclick={() => (editing = i)}><Pencil /></Button>
					<Button variant="ghost" size="icon-xs" aria-label="조건 지우기" onclick={() => (items = items.filter((_, k) => k !== i))}><Trash2 /></Button>
				</span>
			{/if}
		</li>
	{/each}
	<li class="pl-5">
		<InputGroup.Root class="h-8">
			<InputGroup.Addon><Plus /></InputGroup.Addon>
			<InputGroup.Input bind:value={draft} onkeydown={add} {placeholder} aria-label="완료 조건 추가" />
		</InputGroup.Root>
	</li>
</ul>
