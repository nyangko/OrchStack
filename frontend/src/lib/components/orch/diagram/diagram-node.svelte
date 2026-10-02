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
		/** 에이전트 대기열 (.pen AgentQueueTray) — 있으면 카드 아래 받침에 다음 1개 + 남은 수. 빈 배열이면 '대기열 비어 있음'. */
		queue?: { num: number; title: string; stuck?: boolean }[];
	};

	/// 왼쪽 목록에서 끌어 온 태스크 (.pen 드래그로 배정). 캔버스를 그리는 쪽이 setContext로 넘긴다 — 에이전트 노드만 받는다.
	export type DiagramDrag = { task?: { num: number; title: string }; assign: (nodeId: string) => void };
	export const DIAGRAM_DRAG = Symbol("diagram-drag");
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
	import ListStart from "@lucide/svelte/icons/list-start";
	import OctagonAlert from "@lucide/svelte/icons/octagon-alert";
	import Inbox from "@lucide/svelte/icons/inbox";
	import UserPlus from "@lucide/svelte/icons/user-plus";
	import { getContext } from "svelte";
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
	
	let { id, data, selected }: NodeProps<Node<DiagramNodeData>> = $props();
	const drag = getContext<DiagramDrag | undefined>(DIAGRAM_DRAG);
	// 끄는 동안 에이전트 노드만 받고, 나머지는 흐리게
	const target = $derived(!!drag?.task && data.kind === "agent");
	let over = $state(false);

	// 종류별 아이콘 · 색 (.pen KindIcon)
	const kinds = {
		project: { icon: FolderKanban, bg: "bg-foreground" },
		issue: { icon: CircleDot, bg: "bg-node-issue" },
		orch: { icon: Sparkles, bg: "bg-primary" },
		task: { icon: SquareCheck, bg: "bg-node-task" },
		agent: { icon: Bot, bg: "bg-node-agent" },
	};
	const kind = $derived(kinds[data.kind]);
	// 받침이 있으면 위아래 연결선은 노드 오른쪽 끝으로 — 받침 글자를 가리지 않게 (.pen Diagram)
	const side = $derived(data.queue ? "left: 92%" : undefined);
	// 놓기 전 미리보기: 끄는 태스크가 대기열 맨 뒤에 붙은 모양
	const queue = $derived<DiagramNodeData["queue"]>(over && drag?.task ? [...(data.queue ?? []), drag.task] : data.queue);
	const next = $derived(queue?.[0]);
	const rest = $derived(Math.max(0, (queue?.length ?? 0) - 1));
</script>

<!-- 연결선은 좌우(열 사이)와 위아래(같은 열) 양쪽을 쓴다. 손잡이는 보이지 않게 둔다. -->
<Handle type="target" position={Position.Left} id="l" class="opacity-0" />
<Handle type="source" position={Position.Right} id="r" class="opacity-0" />
<Handle type="target" position={Position.Top} id="t" class="opacity-0" style={side} />
<Handle type="source" position={Position.Bottom} id="b" class="opacity-0" style={side} />

<NodeCard
	{selected}
	class={["relative z-1 transition-opacity", drag?.task && !target && "opacity-40", over && "ring-2 ring-primary"]}
	ondragover={(e) => target && (e.preventDefault(), (over = true))}
	ondragleave={(e) => !e.currentTarget.contains(e.relatedTarget as globalThis.Node | null) && (over = false)}
	ondrop={(e) => {
		if (!target) return;
		e.preventDefault();
		over = false;
		drag!.assign(id);
	}}
>
	{#if over && drag?.task}
		<span class="absolute -top-8 left-0 flex items-center gap-1.5 rounded-full bg-primary px-2.5 py-1 text-caption font-semibold whitespace-nowrap text-primary-foreground">
			<UserPlus class="size-3" />#{drag.task.num} → {data.title}에게 배정 · 대기열 {(data.queue?.length ?? 0) + 1}번째
		</span>
	{/if}
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
			<Progress value={data.progress.value} class="h-1" aria-label="진행" tip={`${data.ref} 진행\n${data.progress.text} · ${Math.round(data.progress.value)}%`} />
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
{#if data.queue}
	<!-- 대기열 받침: 카드 뒤로 6px 겹쳐 붙고, 뒤 얇은 줄 수 = 남은 개수(1 · 2줄까지) -->
	<div class="-mt-1.5 ml-2 flex w-47 flex-col items-center">
		<div class="flex w-full items-center gap-1.5 rounded-b-lg border bg-card px-2.5 pt-3 pb-1.5 text-caption">
			{#if next}
				{#if next.stuck}<OctagonAlert class="size-3 shrink-0 text-status-blocked" />{:else}<ListStart class="size-3 shrink-0 text-muted-foreground" />{/if}
				<span class="shrink-0 font-semibold text-muted-foreground">다음</span>
				<span class={["flex-1 truncate font-medium", next.stuck && "text-status-blocked"]}>#{next.num} {next.title}</span>
				{#if rest}<span class="shrink-0 rounded-xs bg-muted px-1 font-mono text-2xs font-semibold text-muted-foreground">+{rest}</span>{/if}
			{:else}
				<Inbox class="size-3 shrink-0 text-muted-foreground" /><span class="font-semibold text-muted-foreground">대기열 비어 있음</span>
			{/if}
		</div>
		{#if rest >= 1}<div class="h-1 w-44 rounded-b-sm border border-t-0 bg-card"></div>{/if}
		{#if rest >= 2}<div class="h-1 w-41 rounded-b-sm border border-t-0 bg-card"></div>{/if}
	</div>
{/if}
