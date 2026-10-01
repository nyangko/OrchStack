<script lang="ts">
	/**
	 * **보드의 태스크 카드** (.pen KanbanCard) — 담당 · 우선순위 / 제목 · 이슈 · 진행 / 토큰 · 메시지 · 모델 · Run.
	 * 끌어서 열을 옮기고, 누르면 상세, 우클릭하면 태스크 메뉴. `TaskBoard` 안에서만 쓴다 — 카드마다 useSortable 하나가 필요해 따로 둔 파일.
	 */
	import UserRound from '@lucide/svelte/icons/user-round';
	import Gauge from '@lucide/svelte/icons/gauge';
	import SignalHigh from '@lucide/svelte/icons/signal-high';
	import SquareCheck from '@lucide/svelte/icons/square-check';
	import CircleDot from '@lucide/svelte/icons/circle-dot';
	import Coins from '@lucide/svelte/icons/coins';
	import MessageSquare from '@lucide/svelte/icons/message-square';
	import Cpu from '@lucide/svelte/icons/cpu';
	import Timer from '@lucide/svelte/icons/timer';
	import { useSortable } from '@dnd-kit-svelte/svelte/sortable';
	import { KeyboardSensor, PointerSensor } from '@dnd-kit/dom';
	import { Badge } from '$lib/components/ui/badge';
	import { Progress } from '$lib/components/ui/progress';
	import { RoleAvatar } from '$lib/components/ui/role-avatar';
	import { ContextMenu, ContextMenuTrigger, ContextMenuContent, ContextMenuLabel, ContextMenuSeparator, ContextMenuEntries } from '$lib/components/ui/context-menu';
	import type { MenuEntry } from '$lib/components/ui/dropdown-menu';
	import type { TaskStatus } from '$lib/status';
	import { roles } from '$lib/roles';
	import type { Agent, Task } from '$lib/mock';
	import { cn } from '$lib/utils';

	let {
		task: t,
		/** 카드가 놓인 열 */
		lane,
		/** 열 안의 순서 */
		index,
		/** 담당 — 없으면 「Unassigned」 */
		agent: a,
		/** 이슈 제목 — 없으면 번호만 */
		issue,
		/** 상세에 열린 카드 — 테두리 강조 */
		selected,
		/** 우클릭 메뉴 */
		menu,
		onopen,
		/** 카드 위에서 포인터가 움직일 때 (미리보기) */
		onhover,
		/** 카드를 벗어나거나 누를 때 · 메뉴를 열 때 (미리보기 닫기) */
		onhoveroff
	}: {
		task: Task;
		lane: TaskStatus;
		index: number;
		agent?: Agent;
		issue?: string;
		selected: boolean;
		menu: MenuEntry[];
		onopen: () => void;
		onhover: (e: PointerEvent) => void;
		onhoveroff: () => void;
	} = $props();

	const { ref, isDragSource } = useSortable({
		id: () => t.num,
		index: () => index,
		group: () => lane,
		type: 'item',
		accept: 'item',
		data: () => ({ group: lane }),
		// 기본값은 closest('button')이면 끌기를 막아 카드(버튼) 안 어디를 눌러도 막힌다 → 실제 입력 요소에서만 막는다.
		// 클릭과 끌기 구분은 기본 제약(200ms 또는 5px).
		sensors: [
			PointerSensor.configure({ preventActivation: (e) => e.target instanceof Element && e.target.closest('input, select, textarea, a[href]') !== null }),
			KeyboardSensor
		]
	});
</script>

<!-- 우클릭: ContextMenu / Task (Diagram과 같은 목록) -->
<ContextMenu onOpenChange={(o) => o && onhoveroff()}>
	<ContextMenuTrigger>
		{#snippet child({ props })}
			<button
				{...props}
				{@attach ref}
				type="button"
				data-dragging={isDragSource.current || undefined}
				aria-pressed={selected}
				class="task-card"
				onclick={onopen}
				onpointermove={onhover}
				onpointerleave={onhoveroff}
				onpointerdown={onhoveroff}
			>
				<!-- 카드가 button이라 안쪽은 span(block 표시) -->
				<span class="task-card-head">
					{#if a}
						<RoleAvatar role={a.role} size="sm" />
						<span class="text-sm font-semibold whitespace-nowrap">{a.name}</span>
						<span class="text-xs whitespace-nowrap text-muted-foreground">{roles[a.role].label}</span>
					{:else}
						<UserRound class="size-4 text-subtle-foreground" />
						<span class="text-xs text-muted-foreground">Unassigned</span>
					{/if}
					<span class="flex-1"></span>
					{#if t.effort}
						<Badge class="rounded-xs bg-review-soft text-node-skill"><Gauge />Effort {t.effort}</Badge>
					{/if}
					<Badge class={cn('rounded-xs', ['P0', 'P1'].includes(t.priority) ? 'bg-destructive-soft text-destructive' : 'bg-muted text-muted-foreground')}><SignalHigh />{t.priority}</Badge>
				</span>
				<span class="task-card-body">
					<span class="title-sm gap-2">
						<SquareCheck class="size-3.5 shrink-0 text-node-task" />#{t.num} · {t.title}
					</span>
					<span class="meta-xs gap-2">
						<CircleDot class="size-3.5 shrink-0 text-node-issue" />Issue #{t.issue} · {issue}
					</span>
					<span class="flex items-center gap-2 pt-1.5">
						{#if t.steps[1]}
							<Progress value={(t.steps[0] / t.steps[1]) * 100} class="h-1.5" aria-label="완료 조건 진행" />
							<span class="task-card-steps">
								{t.steps[0]}/{t.steps[1]} · {Math.round((t.steps[0] / t.steps[1]) * 100)}%
							</span>
						{:else}
							<span class="font-mono text-xs text-subtle-foreground">no steps</span>
						{/if}
					</span>
				</span>
				<span class="task-card-foot">
					<span class={cn('inline-flex items-center gap-1', t.over && 'text-warning')}><Coins class="size-3" />{t.tokens ?? '—'}{t.over ? ' ⚠' : ''}</span>
					<span class="inline-flex items-center gap-1"><MessageSquare class="size-3" />{t.messages}</span>
					{#if t.model}<Badge variant="mono" class="text-2xs"><Cpu />{t.model}</Badge>{/if}
					<span class="ml-auto inline-flex items-center gap-1"><Timer class="size-3" />{t.run ? `Run ${t.run}` : '—'}</span>
				</span>
			</button>
		{/snippet}
	</ContextMenuTrigger>
	<ContextMenuContent class="w-56">
		<ContextMenuLabel class="truncate">Task #{t.num} · {t.title}</ContextMenuLabel>
		<ContextMenuSeparator />
		<ContextMenuEntries entries={menu} />
	</ContextMenuContent>
</ContextMenu>
