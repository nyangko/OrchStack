<script lang="ts" module>
	import type { Component } from "svelte";
	import LoaderCircle from "@lucide/svelte/icons/loader-circle";
	import CircleCheck from "@lucide/svelte/icons/circle-check";
	import OctagonX from "@lucide/svelte/icons/octagon-x";
	import CircleDashed from "@lucide/svelte/icons/circle-dashed";
	import Cpu from "@lucide/svelte/icons/cpu";
	import GitBranch from "@lucide/svelte/icons/git-branch";
	import GitFork from "@lucide/svelte/icons/git-fork";
	import type { SubRun, SubRunTier } from "$lib/mock";

	/// 하위 작업 상태 · 모드 · 등급 표시 (.pen SubRunNode · TierChip). 노드와 Workbench의 하위 작업 카드가 같이 쓴다.
	export const subRunStatus: Record<SubRun["status"], { label: string; icon: Component; tone: string }> = {
		running: { label: "Running", icon: LoaderCircle, tone: "bg-primary-soft text-status-in-progress" },
		done: { label: "Done", icon: CircleCheck, tone: "bg-success-soft text-status-done" },
		failed: { label: "Failed", icon: OctagonX, tone: "bg-destructive-soft text-status-blocked" },
		queued: { label: "Queued", icon: CircleDashed, tone: "bg-muted text-status-backlog" },
	};
	export const subRunMode: Record<SubRun["mode"], { icon: Component; tile: string }> = {
		runner: { icon: Cpu, tile: "bg-node-agent" },
		sub: { icon: GitBranch, tile: "bg-muted-foreground" },
		fork: { icon: GitFork, tile: "bg-muted-foreground" },
	};
	export const tierTone: Record<SubRunTier, string> = { S: "bg-success-soft text-status-done", M: "bg-primary-soft text-primary", L: "bg-review-soft text-status-review" };
</script>

<script lang="ts">
	/// @xyflow/svelte 하위 작업 노드 (.pen SubRunNode) — runner는 실선 · 등급 칩 · 자기 토큰, sub · fork는 점선 · 리드 합계에 포함.
	import { Handle, Position, type NodeProps, type Node } from "@xyflow/svelte";
	import FolderTree from "@lucide/svelte/icons/folder-tree";
	import ListChecks from "@lucide/svelte/icons/list-checks";
	import TokenMeter from "./token-meter.svelte";
	import * as NodeCard from "$lib/components/ui/node-card";
	import { cn } from "$lib/utils";

	let { data, selected }: NodeProps<Node<SubRun>> = $props();

	const st = $derived(subRunStatus[data.status]);
	const md = $derived(subRunMode[data.mode]);
	const asked = $derived(data.paths.filter((p) => p.from === "ask").length);
	const outside = $derived(data.paths.filter((p) => p.from === "bad").length);
	const done = $derived(data.ac.filter((a) => a.ok).length);
</script>

<Handle type="target" position={Position.Left} id="l" class="opacity-0" />
<Handle type="source" position={Position.Top} id="t" class="opacity-0" />
<Handle type="target" position={Position.Bottom} id="b" class="opacity-0" />

<NodeCard.Root {selected} dashed={data.mode !== "runner"} class="h-41 px-3.5 py-3">
	<NodeCard.Header>
		<NodeCard.Kind class={md.tile}><md.icon /></NodeCard.Kind>
		<NodeCard.Ref>{data.mode.toUpperCase()} · {data.id}</NodeCard.Ref>
		{#if data.tier}<span class={cn("tier-chip", tierTone[data.tier])}><Cpu class="size-2.5" />{data.tier}</span>{/if}
	</NodeCard.Header>
	<!-- 글자 크기를 바꾸면 cn이 leading도 지워서 같이 넘긴다 -->
	<NodeCard.Title class="text-body leading-tight">{data.goal}</NodeCard.Title>
	<div class="meta-line gap-3">
		<span class="flex items-center gap-1"><FolderTree class="size-3" />paths {data.paths.length - outside}</span>
		{#if asked}<span class="font-medium text-primary">+{asked} via @ASK</span>{/if}
		{#if outside}<span class="font-medium text-destructive">paths 밖 {outside}</span>{/if}
		<span class="flex items-center gap-1"><ListChecks class="size-3" />ac {done}/{data.ac.length}</span>
	</div>
	<span class="flex-1"></span>
	<NodeCard.Footer>
		<span class={cn("chip-round gap-1 text-caption font-medium", st.tone)}>
			<st.icon class={cn("size-3", data.status === "running" && "animate-spin motion-reduce:animate-none")} />{st.label}
		</span>
		<span class="flex items-center gap-1.5">
			{#if data.mode === "runner"}
				<!-- 이 노드는 지금 Run의 사용량만 (재시도 전 Run은 리드 합계 · Run 목록에서) -->
				{#if data.runs.length}<TokenMeter self={data.runs.at(-1)!.tokens} />{/if}
			{:else}
				<TokenMeter included />
			{/if}
			<span class={cn("text-caption text-muted-foreground", data.status !== "queued" && "font-mono")}>{data.status === "queued" ? "대기" : `${data.minutes}m`}</span>
		</span>
	</NodeCard.Footer>
</NodeCard.Root>
