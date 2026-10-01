<script lang="ts" module>
	import type { TaskStatus } from "$lib/status";
	import type { Role } from "$lib/roles";
	import type { Runtime } from "$lib/components/orch/runtime-logo";

	import type { MenuEntry } from "$lib/components/ui/dropdown-menu/index.js";
	export type { MenuEntry };

	/// Diagram 노드 데이터 (.pen DiagramNode). 담당 · 진행 · 경고 줄은 있을 때만 그린다.
	export type DiagramNodeData = {
		kind: "project" | "issue" | "orch" | "task" | "agent";
		/** 머리 줄 대문자 표기 (예: "TASK #129"). */
		ref: string;
		title: string;
		who?: { role: Role; name: string; runtime?: Runtime; model?: string };
		progress?: { value: number; text: string };
		alert?: { text: string; tone: "warning" | "destructive" };
		/** 태스크는 상태 배지, 그 외는 글자 배지. */
		status?: TaskStatus;
		badge?: string;
		meta?: string;
		/** 리드 토큰 (K) — 자기 사용량 + 하위 runner 합계 (.pen TokenMeter lead). */
		tokens?: { self: number; runner: number };
		/** 머리 줄 … 메뉴 (.pen ContextMenu / Task). menuLabel은 메뉴 머리 (예: "Task #130 · QA"). */
		menu?: MenuEntry[];
		menuLabel?: string;
	};
</script>

<script lang="ts">
	/// @xyflow/svelte 노드 어댑터 — 라이브러리가 nodeTypes로 컴포넌트를 요구한다. 연결 손잡이 + ui/node-card 조립만.
	import { Handle, Position, type NodeProps, type Node } from "@xyflow/svelte";
	import FolderKanban from "@lucide/svelte/icons/folder-kanban";
	import CircleDot from "@lucide/svelte/icons/circle-dot";
	import Sparkles from "@lucide/svelte/icons/sparkles";
	import SquareCheck from "@lucide/svelte/icons/square-check";
	import Bot from "@lucide/svelte/icons/bot";
	import MessageCircle from "@lucide/svelte/icons/message-circle";
	import TriangleAlert from "@lucide/svelte/icons/triangle-alert";
	import Ellipsis from "@lucide/svelte/icons/ellipsis";
	import { DropdownMenu, DropdownMenuTrigger, DropdownMenuContent, DropdownMenuLabel, DropdownMenuSeparator, DropdownMenuEntries } from "$lib/components/ui/dropdown-menu";
	import { NodeCard, NodeCardHeader, NodeCardKind, NodeCardRef, NodeCardTitle, NodeCardFooter } from "$lib/components/orch/node-card";
	import { Button } from "$lib/components/ui/button";
	import { Badge } from "$lib/components/ui/badge";
	import { Progress } from "$lib/components/ui/progress";
	import { StatusBadge } from "$lib/components/orch/status-badge";
	import { RoleAvatar } from "$lib/components/orch/role-avatar";
	import { RuntimeLogo } from "$lib/components/orch/runtime-logo";
	import TokenMeter from "./token-meter.svelte";
	
	let { data, selected }: NodeProps<Node<DiagramNodeData>> = $props();

	// 종류별 아이콘 · 색 (.pen KindIcon)
	const kinds = {
		project: { icon: FolderKanban, bg: "bg-foreground" },
		issue: { icon: CircleDot, bg: "bg-node-issue" },
		orch: { icon: Sparkles, bg: "bg-primary" },
		task: { icon: SquareCheck, bg: "bg-node-task" },
		agent: { icon: Bot, bg: "bg-node-agent" },
	};
	const kind = $derived(kinds[data.kind]);
</script>

<!-- 연결선은 좌우(열 사이)와 위아래(같은 열) 양쪽을 쓴다. 손잡이는 보이지 않게 둔다. -->
<Handle type="target" position={Position.Left} id="l" class="opacity-0" />
<Handle type="source" position={Position.Right} id="r" class="opacity-0" />
<Handle type="target" position={Position.Top} id="t" class="opacity-0" />
<Handle type="source" position={Position.Bottom} id="b" class="opacity-0" />

<NodeCard {selected}>
	<NodeCardHeader>
		<NodeCardKind class={kind.bg}><kind.icon /></NodeCardKind>
		<NodeCardRef>{data.ref}</NodeCardRef>
		{#if data.menu?.length}
			<DropdownMenu>
				<DropdownMenuTrigger>
					{#snippet child({ props })}
						<!-- nodrag: 메뉴 버튼을 눌러도 노드가 끌리지 않게 (xyflow) -->
						<Button {...props} variant="ghost" size="icon-xs" class="nodrag -my-1" aria-label="{data.ref} 메뉴"><Ellipsis /></Button>
					{/snippet}
				</DropdownMenuTrigger>
				<DropdownMenuContent align="start" class="w-56">
					{#if data.menuLabel}<DropdownMenuLabel class="truncate">{data.menuLabel}</DropdownMenuLabel><DropdownMenuSeparator />{/if}
					<DropdownMenuEntries entries={data.menu} />
				</DropdownMenuContent>
			</DropdownMenu>
		{/if}
	</NodeCardHeader>
	<NodeCardTitle>{data.title}</NodeCardTitle>
	{#if data.tokens}<TokenMeter self={data.tokens.self} runner={data.tokens.runner} />{/if}
	{#if data.who}
		<div class="flex items-center gap-1.5 text-xs">
			<RoleAvatar role={data.who.role} size="sm" />
			<span class="font-medium">{data.who.name}</span>
			{#if data.who.runtime}<RuntimeLogo runtime={data.who.runtime} class="size-3 ring-0" />{/if}
			{#if data.who.model}<span class="truncate font-mono text-muted-foreground">{data.who.model}</span>{/if}
		</div>
	{/if}
	{#if data.progress}
		<div class="flex items-center gap-2">
			<Progress value={data.progress.value} class="h-1" aria-label="진행" />
			<span class="mono-meta shrink-0">{data.progress.text}</span>
		</div>
	{/if}
	{#if data.alert}
		<div class={["label-xs rounded-sm px-2 py-1", data.alert.tone === "warning" ? "bg-warning-soft text-warning" : "bg-destructive-soft text-destructive"]}>
			{#if data.alert.tone === "warning"}<MessageCircle class="size-3" />{:else}<TriangleAlert class="size-3" />{/if}
			{data.alert.text}
		</div>
	{/if}
	<NodeCardFooter>
		{#if data.status}<StatusBadge status={data.status} />{:else if data.badge}<Badge variant="secondary">{data.badge}</Badge>{:else}<span></span>{/if}
		{#if data.meta}<span class="mono-meta truncate">{data.meta}</span>{/if}
	</NodeCardFooter>
</NodeCard>
