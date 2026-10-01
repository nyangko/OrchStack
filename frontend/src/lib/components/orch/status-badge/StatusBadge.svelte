<script lang="ts">
	/// Task 상태 표시 (.pen StatusBadge). shadcn Badge + $lib/status 매핑.
	import { Badge } from "$lib/components/ui/badge/index.js";
	import { statuses, type TaskStatus } from "$lib/status.js";
	import { cn } from "$lib/utils.js";

	let { status, class: className }: { status: TaskStatus; class?: string } = $props();

	const meta = $derived(statuses[status]);
</script>

<Badge class={cn("rounded-full", meta.soft, meta.text, className)}>
	<meta.icon />
	{meta.label}
</Badge>
