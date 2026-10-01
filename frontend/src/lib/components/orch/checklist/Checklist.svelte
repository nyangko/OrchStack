<script lang="ts" module>
	/** 체크리스트 한 항목. */
	export type ChecklistItem = { text: string; done: boolean };
</script>

<script lang="ts">
	/// 체크리스트 (.pen CriterionRow · TaskEditor Acceptance criteria). 항목 { text, done } 목록.
	/// 체크 · 끌어서 순서 바꾸기 · hover에 편집 · 삭제 · 마지막 줄에서 추가(Enter). 끝난 항목은 흐리게.
	/// sortable · editable · addable로 기능을 끌 수 있다 (읽기 전용 목록 = 셋 다 false).
	import type { HTMLAttributes } from "svelte/elements";
	import GripVertical from "@lucide/svelte/icons/grip-vertical";
	import Pencil from "@lucide/svelte/icons/pencil";
	import Trash2 from "@lucide/svelte/icons/trash-2";
	import Plus from "@lucide/svelte/icons/plus";
	import { Checkbox } from "$lib/components/ui/checkbox/index.js";
	import { Button } from "$lib/components/ui/button/index.js";
	import { InputGroup, InputGroupInput, InputGroupAddon } from "$lib/components/ui/input-group";
	import { cn, type WithElementRef } from "$lib/utils.js";

	let {
		ref = $bindable(null),
		items = $bindable(),
		placeholder = "항목 추가",
		label = "체크리스트",
		sortable = true,
		editable = true,
		addable = true,
		class: className,
		...restProps
	}: WithElementRef<Omit<HTMLAttributes<HTMLUListElement>, "children">> & {
		items: ChecklistItem[];
		placeholder?: string;
		/** 목록 · 입력칸 접근성 이름. */
		label?: string;
		sortable?: boolean;
		editable?: boolean;
		addable?: boolean;
	} = $props();

	let draft = $state("");
	let editing = $state<number>();
	let dragFrom = $state<number>();

	function add(e: KeyboardEvent) {
		if (e.key !== "Enter" || e.isComposing || !draft.trim()) return;
		e.preventDefault();
		items = [...items, { text: draft.trim(), done: false }];
		draft = "";
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

<ul bind:this={ref} data-slot="checklist" aria-label={label} class={cn("flex flex-col gap-0.5", className)} {...restProps}>
	{#each items as c, i (i)}
		<li
			data-slot="checklist-item"
			data-done={c.done || undefined}
			data-dragging={dragFrom === i || undefined}
			class="group flex h-8 items-center gap-2 rounded-sm px-1.5 hover:bg-muted/60 data-dragging:opacity-50"
			draggable={sortable && editing !== i}
			ondragstart={() => (dragFrom = i)}
			ondragover={(e) => sortable && e.preventDefault()}
			ondrop={() => drop(i)}
		>
			{#if sortable}
				<GripVertical class="size-3.25 shrink-0 cursor-grab text-transparent group-hover:text-subtle-foreground" aria-hidden="true" />
			{/if}
			<Checkbox bind:checked={c.done} aria-label={c.text} />
			{#if editing === i}
				<InputGroup class="h-7 flex-1">
					<!-- svelte-ignore a11y_autofocus -->
					<InputGroupInput
						autofocus
						bind:value={c.text}
						aria-label="{label} 고치기"
						onblur={() => (editing = undefined)}
						onkeydown={(e) => (e.key === "Enter" || e.key === "Escape") && !e.isComposing && (e.preventDefault(), (editing = undefined))}
					/>
				</InputGroup>
			{:else}
				<span class="min-w-0 flex-1 text-body group-data-done:text-muted-foreground">{c.text}</span>
				{#if editable}
					<span class="flex items-center opacity-0 group-focus-within:opacity-100 group-hover:opacity-100">
						<Button variant="ghost" size="icon-xs" aria-label="{c.text} 고치기" onclick={() => (editing = i)}><Pencil /></Button>
						<Button variant="ghost" size="icon-xs" aria-label="{c.text} 지우기" onclick={() => (items = items.filter((_, k) => k !== i))}><Trash2 /></Button>
					</span>
				{/if}
			{/if}
		</li>
	{/each}
	{#if addable}
		<li data-slot="checklist-add" class="pl-5">
			<InputGroup class="h-8">
				<InputGroupAddon><Plus /></InputGroupAddon>
				<InputGroupInput bind:value={draft} onkeydown={add} {placeholder} aria-label="{label} 추가" />
			</InputGroup>
		</li>
	{/if}
</ul>
