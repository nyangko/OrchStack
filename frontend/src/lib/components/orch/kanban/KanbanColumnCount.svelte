<script lang="ts">
	/// 칸반 열 항목 수 — 보드 value에서 읽는다 (넘기지 않아도 된다).
	import { Badge } from "$lib/components/ui/badge/index.js";
	import { cn } from "$lib/utils.js";
	import { getBoard, getColumn } from "./context.js";

	let { class: className }: { class?: string } = $props();
	const board = getBoard();
	const column = getColumn();
</script>

<Badge data-slot="kanban-column-count" variant="secondary" class={cn("rounded-full px-1.5 font-mono", className)}>{board()[column()]?.length ?? 0}</Badge>
