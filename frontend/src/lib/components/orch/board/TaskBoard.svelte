<script lang="ts">
	/**
	 * **태스크 칸반 보드** (.pen #26 Workbench / Kanban Board). 상태별 열에 태스크 카드를 두고, 다른 열에 놓으면 상태를 바꾼다.
	 * 카드 우클릭 = 태스크 메뉴(Diagram 노드와 같은 목록), 카드 위에 머물면 미리보기(.pen KanbanCard/HoverPreview).
	 * 열(`TaskBoardLane`) · 카드(`TaskCard`)가 따로 파일인 것은 dnd-kit 훅(useDroppable · useSortable)이 컴포넌트마다 하나라서다.
	 */
	import Plus from '@lucide/svelte/icons/plus';
	import Ellipsis from '@lucide/svelte/icons/ellipsis';
	import SquarePen from '@lucide/svelte/icons/square-pen';
	import FilterIcon from '@lucide/svelte/icons/filter';
	import { DragDropProvider } from '@dnd-kit-svelte/svelte';
	import { move } from '@dnd-kit/helpers';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import { Progress } from '$lib/components/ui/progress';
	import { StatusBadge } from '$lib/components/ui/status-badge';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
	import * as HoverCard from '$lib/components/ui/hover-card';
	import type { MenuEntry } from '$lib/components/ui/dropdown-menu';
	import { statuses, statusOrder, type TaskStatus } from '$lib/status';
	import { roles } from '$lib/roles';
	import type { Agent, Issue, Task, TaskDetail } from '$lib/mock';
	import TaskBoardLane from './TaskBoardLane.svelte';
	import TaskCard from './TaskCard.svelte';

	let {
		/** 보드에 놓을 태스크 */
		tasks,
		/** 카드 담당 · 미리보기 이름 — 없는 sn은 「Unassigned」 */
		agents,
		/** 카드의 이슈 제목 */
		issues,
		/** 미리보기의 단계 · 의존 · 활동 · 컨텍스트 — 없는 태스크는 「—」 */
		details,
		/** 강조할 태스크(상세에 열린 것) — 없으면 강조 없음 */
		selected,
		/** 카드 우클릭 메뉴 */
		menu,
		/** 카드를 눌렀을 때 */
		onopen,
		/** 다른 열에 놓았을 때 — 상태가 바뀐 태스크만 부른다 */
		onmove,
		/** 열 머리 + (QuickAdd) */
		onadd,
		/** 열 메뉴 「새 태스크」 (편집기) */
		onnew,
		/** 열 메뉴 「Quick Panel에서 이 상태만」 */
		onfilter
	}: {
		tasks: Task[];
		agents: Agent[];
		issues: Issue[];
		details: Record<number, TaskDetail>;
		selected?: number;
		menu: (num: number) => MenuEntry[];
		onopen: (num: number) => void;
		onmove: (num: number, status: TaskStatus) => void;
		onadd: (status: TaskStatus) => void;
		onnew: (status: TaskStatus) => void;
		onfilter: (status: TaskStatus) => void;
	} = $props();

	const task = (num: number) => tasks.find((t) => t.num === num)!;
	const agentOf = (sn?: number) => agents.find((a) => a.sn === sn);
	// Failed 열은 해당 태스크가 있을 때만.
	const lanes = $derived(statusOrder.filter((s) => s !== 'failed' || tasks.some((t) => t.status === s)));

	// 끄는 동안의 열 배치. 태스크가 바뀌면 다시 맞추고, 놓을 때 태스크 상태에 반영한다.
	let board = $state<Record<string, number[]>>({});
	$effect.pre(() => {
		board = Object.fromEntries(statusOrder.map((s) => [s, tasks.filter((t) => t.status === s).map((t) => t.num)]));
	});
	function drop() {
		for (const [s, nums] of Object.entries(board)) for (const n of nums) if (task(n).status !== s) onmove(n, s as TaskStatus);
	}

	// 미리보기 — 커서 +18px, 350ms 머문 뒤. 끌 때 · 메뉴 열 때는 숨긴다.
	let hover = $state<{ num: number; x: number; y: number }>();
	let hoverTimer: ReturnType<typeof setTimeout> | undefined;
	function hoverAt(num: number, e: PointerEvent) {
		if (e.buttons) return void (hover = undefined);
		const at = { num, x: e.clientX, y: e.clientY };
		if (hover?.num === num) return void (hover = at);
		clearTimeout(hoverTimer);
		hoverTimer = setTimeout(() => (hover = at), 350);
	}
	function hoverOff() {
		clearTimeout(hoverTimer);
		hover = undefined;
	}
	$effect(() => () => clearTimeout(hoverTimer));
</script>

<DragDropProvider
	onDragOver={(e) => (board = move(board, e))}
	onDragEnd={(e) => {
		if (e.canceled) return;
		board = move(board, e);
		drop();
	}}
>
	<div class="task-board">
		{#each lanes as s (s)}
			{@const meta = statuses[s]}
			<TaskBoardLane status={s}>
				<div class="task-board-lane-head">
					<meta.icon class={meta.text} />
					<h3 class="text-body font-semibold">{meta.label}</h3>
					<Badge variant="secondary" class="rounded-full px-1.5 font-mono">{board[s]?.length ?? 0}</Badge>
					<div class="ml-auto flex items-center gap-2">
						<Button variant="ghost" size="icon-xs" aria-label="{meta.label}에 태스크 추가" onclick={() => onadd(s)}><Plus /></Button>
						<!-- 열 메뉴 — .pen에 항목이 없어 있는 동작만 (#60) -->
						<DropdownMenu.Root>
							<DropdownMenu.Trigger>{#snippet child({ props })}<Button {...props} variant="ghost" size="icon-xs" aria-label="{meta.label} 열 메뉴"><Ellipsis /></Button>{/snippet}</DropdownMenu.Trigger>
							<DropdownMenu.Content align="end" class="w-48">
								<DropdownMenu.Item onSelect={() => onnew(s)}><SquarePen />{meta.label}로 새 태스크</DropdownMenu.Item>
								<DropdownMenu.Item onSelect={() => onfilter(s)}><FilterIcon />Quick Panel에서 이 상태만</DropdownMenu.Item>
							</DropdownMenu.Content>
						</DropdownMenu.Root>
					</div>
				</div>
				<div class="flex min-h-24 flex-col gap-2">
					{#each board[s] ?? [] as num, index (num)}
						{@const t = task(num)}
						<TaskCard
							task={t}
							lane={s}
							{index}
							agent={agentOf(t.agent)}
							issue={issues.find((i) => i.num === t.issue)?.title}
							selected={selected === num}
							menu={menu(num)}
							onopen={() => (hoverOff(), onopen(num))}
							onhover={(e) => hoverAt(num, e)}
							onhoveroff={hoverOff}
						/>
					{:else}
						<p class="empty-note text-center text-subtle-foreground">비어 있어요</p>
					{/each}
				</div>
			</TaskBoardLane>
		{/each}
	</div>
</DragDropProvider>

{#if hover}
	{@const t = task(hover.num)}
	{@const d = details[hover.num]}
	{@const next = d?.criteria.find((c) => !c.done)}
	{@const blocks = (d?.deps ?? []).map((x) => (x.kind === 'blocks' ? tasks.find((b) => b.num === x.num) : undefined)).filter((b) => b !== undefined)}
	{@const last = d?.activity.at(-1)}
	{@const ctx = d?.context}
	{@const at = { x: hover.x, y: hover.y }}
	<!-- .pen KanbanCard/HoverPreview — 커서 위치에 띄운다 (+18px, 화면 끝에선 floating-ui가 뒤집음) -->
	<HoverCard.Root open onOpenChange={(o) => !o && hoverOff()}>
		<HoverCard.Content
			customAnchor={{ getBoundingClientRect: () => new DOMRect(at.x + 18, at.y, 0, 0) }}
			side="bottom"
			align="start"
			sideOffset={18}
			class="pointer-events-none w-75 gap-0 overflow-hidden p-0"
		>
			<div class="hover-preview-head">
				<span class="pt-0.5 text-xs font-semibold text-muted-foreground">#{t.num}</span>
				<span class="min-w-0 flex-1 text-body font-semibold">{t.title}</span>
				<StatusBadge status={t.status} />
			</div>
			<dl class="hover-preview-body">
				{#each [['현재 단계', t.steps[1] ? `${t.steps[0]}/${t.steps[1]}${next ? ` · ${next.text}` : d?.criteria.length ? ' · 모두 완료' : ''}` : '—'], ['최근 활동', last ? `${last.type.toLowerCase()} ${last.text}` : '—'], ['막고 있는 Task', blocks.length ? blocks.map((b) => `#${b.num} ${b.title} · ${agentOf(b.agent)?.name ?? '미배정'}`).join(', ') : '—'], ['완료 시 전달', blocks[0] ? `${agentOf(blocks[0].agent)?.name ?? '미배정'} · ${roles[agentOf(blocks[0].agent)?.role ?? 'agent'].label} (REQUEST_VERIFICATION)` : '—'], ['ETA', d?.eta || '—']] as [k, v] (k)}
					<div class="flex gap-2"><dt class="shrink-0 text-muted-foreground">{k}</dt><dd class="hover-preview-value">{v}</dd></div>
				{/each}
				{#if ctx}
					<div class="flex flex-col gap-1.5 pt-1.5">
						<div class="flex text-xs font-medium"><span class="flex-1 text-muted-foreground">Context</span><span class={ctx[0] / ctx[1] > 0.9 ? 'text-warning' : ''}>{ctx[0]}K / {ctx[1]}K</span></div>
						<Progress value={(ctx[0] / ctx[1]) * 100} class="h-2" aria-label="컨텍스트" />
					</div>
					{#if ctx[0] / ctx[1] > 0.9}<p class="font-medium text-warning">⚠ Context {Math.round((ctx[0] / ctx[1]) * 100)}% — 요약 또는 새 Session 권장</p>{/if}
				{/if}
			</dl>
			<p class="hover-preview-foot">클릭 → 상세 보기 · 우클릭 → 메뉴</p>
		</HoverCard.Content>
	</HoverCard.Root>
{/if}
