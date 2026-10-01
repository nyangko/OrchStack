<script lang="ts" module>
	import type { TaskStatus } from "$lib/status";
	import type { Role } from "$lib/roles";
	import type { Runtime } from "$lib/components/ui/runtime-logo";
	import type { Component } from "svelte";

	/// 노드 · 카드 메뉴 한 줄 (.pen ContextMenu / Task). "sep"는 구분선, sub가 있으면 하위 메뉴.
	/// Diagram 노드 … 메뉴와 Kanban 카드 우클릭 메뉴가 같은 목록을 그린다.
	export type MenuEntry =
		| "sep"
		| {
				label: string;
				icon?: Component;
				shortcut?: string;
				disabled?: boolean;
				onSelect?: () => void;
				sub?: { label: string; icon?: Component; tone?: string; checked?: boolean; onSelect: () => void }[];
		  };

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
	/// @xyflow/svelte 노드 컴포넌트 — 라이브러리가 nodeTypes로 컴포넌트를 요구해서 분리한다.
	import { Handle, Position, type NodeProps, type Node } from "@xyflow/svelte";
	import FolderKanban from "@lucide/svelte/icons/folder-kanban";
	import CircleDot from "@lucide/svelte/icons/circle-dot";
	import Sparkles from "@lucide/svelte/icons/sparkles";
	import SquareCheck from "@lucide/svelte/icons/square-check";
	import Bot from "@lucide/svelte/icons/bot";
	import MessageCircle from "@lucide/svelte/icons/message-circle";
	import TriangleAlert from "@lucide/svelte/icons/triangle-alert";
	import Ellipsis from "@lucide/svelte/icons/ellipsis";
	import Check from "@lucide/svelte/icons/check";
	import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
	import { Button } from "$lib/components/ui/button";
	import { Badge } from "$lib/components/ui/badge";
	import { Progress } from "$lib/components/ui/progress";
	import { StatusBadge } from "$lib/components/ui/status-badge";
	import { RoleAvatar } from "$lib/components/ui/role-avatar";
	import { RuntimeLogo } from "$lib/components/ui/runtime-logo";
	import TokenMeter from "./token-meter.svelte";
	import { cn } from "$lib/utils";

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

<div
	class={cn(
		"diagram-node",
		selected && "ring-2 ring-primary"
	)}
>
	<div class="flex items-center gap-2">
		<span class={cn("center-box size-6 rounded-sm text-on-solid", kind.bg)}>
			<kind.icon class="size-3.5" />
		</span>
		<span class="mono-ref flex-1 truncate">{data.ref}</span>
		{#if data.menu?.length}
			<DropdownMenu.Root>
				<DropdownMenu.Trigger>
					{#snippet child({ props })}
						<!-- nodrag: 메뉴 버튼을 눌러도 노드가 끌리지 않게 (xyflow) -->
						<Button {...props} variant="ghost" size="icon-xs" class="nodrag -my-1" aria-label="{data.ref} 메뉴"><Ellipsis /></Button>
					{/snippet}
				</DropdownMenu.Trigger>
				<DropdownMenu.Content align="start" class="w-56">
					{#if data.menuLabel}<DropdownMenu.Label class="truncate">{data.menuLabel}</DropdownMenu.Label><DropdownMenu.Separator />{/if}
					{#each data.menu as m, i (i)}
						{#if m === "sep"}
							<DropdownMenu.Separator />
						{:else if m.sub}
							<DropdownMenu.Sub>
								<DropdownMenu.SubTrigger disabled={m.disabled}>{#if m.icon}<m.icon class="text-muted-foreground" />{/if}{m.label}</DropdownMenu.SubTrigger>
								<DropdownMenu.SubContent class="w-48">
									{#each m.sub as x (x.label)}
										<DropdownMenu.Item onSelect={x.onSelect}>{#if x.icon}<x.icon class={x.tone} />{/if}<span class="flex-1">{x.label}</span>{#if x.checked}<Check />{/if}</DropdownMenu.Item>
									{/each}
								</DropdownMenu.SubContent>
							</DropdownMenu.Sub>
						{:else}
							<DropdownMenu.Item disabled={m.disabled} onSelect={m.onSelect}>
								{#if m.icon}<m.icon class="text-muted-foreground" />{/if}{m.label}
								{#if m.shortcut}<DropdownMenu.Shortcut>{m.shortcut}</DropdownMenu.Shortcut>{/if}
							</DropdownMenu.Item>
						{/if}
					{/each}
				</DropdownMenu.Content>
			</DropdownMenu.Root>
		{/if}
	</div>
	<p class="text-sm leading-tight font-semibold">{data.title}</p>
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
		<div class={cn("label-xs rounded-sm px-2 py-1", data.alert.tone === "warning" ? "bg-warning-soft text-warning" : "bg-destructive-soft text-destructive")}>
			{#if data.alert.tone === "warning"}<MessageCircle class="size-3" />{:else}<TriangleAlert class="size-3" />{/if}
			{data.alert.text}
		</div>
	{/if}
	<div class="flex items-center justify-between gap-2">
		{#if data.status}<StatusBadge status={data.status} />{:else if data.badge}<Badge variant="secondary">{data.badge}</Badge>{:else}<span></span>{/if}
		{#if data.meta}<span class="mono-meta truncate">{data.meta}</span>{/if}
	</div>
</div>
