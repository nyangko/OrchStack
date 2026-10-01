<script lang="ts">
	/// ui/kanban 스파이크 — 태스크와 무관한 데이터로도 조립되는지 확인.
	import CircleDot from "@lucide/svelte/icons/circle-dot";
	import Coins from "@lucide/svelte/icons/coins";
	import SquareCheck from "@lucide/svelte/icons/square-check";
	import * as Kanban from "$lib/components/ui/kanban/index.js";

	const columns = [
		{ id: "todo", label: "Todo", tone: "text-status-todo" },
		{ id: "progress", label: "In Progress", tone: "text-status-in-progress" },
		{ id: "review", label: "Review", tone: "text-status-review" },
		{ id: "done", label: "Done", tone: "text-status-done" }
	];
	const titles: Record<string, string> = { t129: "#129 · Login UI 구현", t130: "#130 · Signup 폼", t131: "#131 · 세션 API 연동", t132: "#132 · Token 갱신" };

	let items = $state<Kanban.KanbanValue>({ todo: ["t129", "t130"], progress: ["t131", "t132"], review: [], done: [] });
	let last = $state("");
</script>

<Kanban.Root bind:value={items} onDragEnd={(v) => (last = JSON.stringify(v))} class="min-h-screen overflow-x-auto bg-muted p-6">
	{#each columns as col (col.id)}
		<Kanban.Column value={col.id}>
			<Kanban.ColumnHeader>
				<CircleDot class={col.tone} />
				<Kanban.ColumnTitle>{col.label}</Kanban.ColumnTitle>
				<Kanban.ColumnCount />
			</Kanban.ColumnHeader>
			<Kanban.ColumnContent>
				{#each items[col.id] as id (id)}
					<Kanban.Item value={id}>
						<Kanban.ItemContent><span class="title-sm gap-2"><SquareCheck class="size-3.5 text-node-task" />{titles[id]}</span></Kanban.ItemContent>
						<Kanban.ItemFooter><Coins class="size-3" />41.2K</Kanban.ItemFooter>
					</Kanban.Item>
				{/each}
				{#snippet empty()}<p class="empty-note text-center text-subtle-foreground">비어 있어요</p>{/snippet}
			</Kanban.ColumnContent>
		</Kanban.Column>
	{/each}
</Kanban.Root>
<p class="sr-only" data-testid="last">{last}</p>
