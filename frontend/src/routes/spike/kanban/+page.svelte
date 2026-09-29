<script lang="ts">
	import { DragDropProvider } from "@dnd-kit-svelte/svelte";
	import { move } from "@dnd-kit/helpers";
	import CircleDot from "@lucide/svelte/icons/circle-dot";
	import Coins from "@lucide/svelte/icons/coins";
	import MessageSquare from "@lucide/svelte/icons/message-square";
	import SquareCheck from "@lucide/svelte/icons/square-check";
	import Timer from "@lucide/svelte/icons/timer";
	import * as Avatar from "$lib/components/ui/avatar/index.js";
	import { Pill } from "$lib/components/ui/pill/index.js";
	import { Progress } from "$lib/components/ui/progress/index.js";
	import KanbanCard from "$lib/components/orch/kanban/kanban-card.svelte";
	import KanbanColumn from "$lib/components/orch/kanban/kanban-column.svelte";

	const columns = [
		{ id: "todo", label: "Todo", dot: "text-status-todo" },
		{ id: "progress", label: "In Progress", dot: "text-status-in-progress" },
		{ id: "review", label: "Review", dot: "text-status-review" },
		{ id: "done", label: "Done", dot: "text-status-done" }
	];
	const tasks = Object.fromEntries(
		[
			[129, "Login UI 구현"],
			[130, "Signup 폼"],
			[131, "세션 API 연동"],
			[132, "Token 갱신"]
		].map(([n, title]) => [`t${n}`, { id: `t${n}`, title: `#${n} · ${title}` }])
	);

	let items = $state<Record<string, string[]>>({
		todo: ["t129", "t130"],
		progress: ["t131", "t132"],
		review: [],
		done: []
	});
</script>

<DragDropProvider
	onDragOver={(event) => (items = move(items, event))}
	onDragEnd={(event) => {
		if (!event.canceled) items = move(items, event);
	}}
>
	<div class="bg-canvas flex min-h-screen items-start gap-3 overflow-x-auto p-6">
		{#each columns as col (col.id)}
			<KanbanColumn id={col.id}>
				<div class="flex h-10 items-center gap-2 pr-1 pl-2">
					<CircleDot class="size-3.5 {col.dot}" />
					<span class="text-sm font-semibold">{col.label}</span>
					<Pill class="font-mono font-semibold">{items[col.id].length}</Pill>
				</div>
				<div class="flex min-h-24 flex-col gap-2">
					{#each items[col.id] as id, i (id)}
						<KanbanCard {id} index={i} group={col.id}>
							<div class="flex items-center gap-2 border-b px-4 py-3">
								<Avatar.Root size="sm">
									<Avatar.Fallback class="bg-role-frontend text-on-solid text-xs font-semibold">진</Avatar.Fallback>
								</Avatar.Root>
								<span class="text-sm font-semibold">진</span>
								<span class="text-muted-foreground text-xs">Frontend</span>
							</div>
							<div class="flex flex-col gap-2.5 px-4 py-3.5">
								<div class="flex items-center gap-2 font-semibold">
									<SquareCheck class="text-node-task size-3.5 shrink-0" />
									{tasks[id].title}
								</div>
								<div class="text-muted-foreground flex items-center gap-2 text-xs">
									<CircleDot class="text-node-issue size-3.5 shrink-0" />
									Issue #51 · Authentication Flow 개선
								</div>
								<div class="flex items-center gap-2 pt-1.5">
									<Progress value={62} class="h-1.5" />
									<span class="text-muted-foreground shrink-0 font-mono text-xs font-semibold whitespace-nowrap">3/5 · 62%</span>
								</div>
							</div>
							<div class="text-muted-foreground flex items-center gap-3.5 border-t px-4 py-2.5 font-mono text-xs">
								<span class="inline-flex items-center gap-1"><Coins class="size-3" />41.2K</span>
								<span class="inline-flex items-center gap-1"><MessageSquare class="size-3" />8</span>
								<span class="ml-auto inline-flex items-center gap-1"><Timer class="size-3" />Run 15m</span>
							</div>
						</KanbanCard>
					{/each}
				</div>
			</KanbanColumn>
		{/each}
	</div>
</DragDropProvider>
