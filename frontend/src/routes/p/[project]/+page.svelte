<script lang="ts">
	/// Workbench — 좌측 Quick Panel · 가운데 뷰(Diagram / Kanban / Issues) + Ops · 우측 PM Dock.
	import Circle from '@lucide/svelte/icons/circle';
	import CircleDashed from '@lucide/svelte/icons/circle-dashed';
	import OctagonAlert from '@lucide/svelte/icons/octagon-alert';
	import ChevronsDown from '@lucide/svelte/icons/chevrons-down';
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import Workflow from '@lucide/svelte/icons/workflow';
	import KanbanSquare from '@lucide/svelte/icons/kanban-square';
	import ListTree from '@lucide/svelte/icons/list-tree';
	import Search from '@lucide/svelte/icons/search';
	import Plus from '@lucide/svelte/icons/plus';
	import PanelLeftClose from '@lucide/svelte/icons/panel-left-close';
	import PanelLeftOpen from '@lucide/svelte/icons/panel-left-open';
	import PanelRightClose from '@lucide/svelte/icons/panel-right-close';
	import PanelRightOpen from '@lucide/svelte/icons/panel-right-open';
	import ListPlus from '@lucide/svelte/icons/list-plus';
	import RotateCw from '@lucide/svelte/icons/rotate-cw';
	import MessageSquareShare from '@lucide/svelte/icons/message-square-share';
	import LayoutTemplate from '@lucide/svelte/icons/layout-template';
	import Clock3 from '@lucide/svelte/icons/clock-3';
	import CircleCheck from '@lucide/svelte/icons/circle-check';
	import Sparkles from '@lucide/svelte/icons/sparkles';
	import Eye from '@lucide/svelte/icons/eye';
	import Inbox from '@lucide/svelte/icons/inbox';
	import Archive from '@lucide/svelte/icons/archive';
	import CircleCheckBig from '@lucide/svelte/icons/circle-check-big';
	import CirclePause from '@lucide/svelte/icons/circle-pause';
	import ShieldAlert from '@lucide/svelte/icons/shield-alert';
	import Undo2 from '@lucide/svelte/icons/undo-2';
	import Repeat from '@lucide/svelte/icons/repeat';
	import GitPullRequest from '@lucide/svelte/icons/git-pull-request';
	import FileDiff from '@lucide/svelte/icons/file-diff';
	import ShieldCheck from '@lucide/svelte/icons/shield-check';
	import FlaskConical from '@lucide/svelte/icons/flask-conical';
	import X from '@lucide/svelte/icons/x';
	import Ban from '@lucide/svelte/icons/ban';
	import ShieldX from '@lucide/svelte/icons/shield-x';
	import Lightbulb from '@lucide/svelte/icons/lightbulb';
	import Terminal from '@lucide/svelte/icons/terminal';
	import TriangleAlert from '@lucide/svelte/icons/triangle-alert';
	import Funnel from '@lucide/svelte/icons/funnel';
	import CircleX from '@lucide/svelte/icons/circle-x';
	import User from '@lucide/svelte/icons/user';
	import ListChecks from '@lucide/svelte/icons/list-checks';
	import FileText from '@lucide/svelte/icons/file-text';
	import Pause from '@lucide/svelte/icons/pause';
	import Play from '@lucide/svelte/icons/play';
	import Download from '@lucide/svelte/icons/download';
	import ChevronDown from '@lucide/svelte/icons/chevron-down';
	import ChevronUp from '@lucide/svelte/icons/chevron-up';
	import Paperclip from '@lucide/svelte/icons/paperclip';
	import ArrowUp from '@lucide/svelte/icons/arrow-up';
	import Ellipsis from '@lucide/svelte/icons/ellipsis';
	import SquareCheck from '@lucide/svelte/icons/square-check';
	import CircleDot from '@lucide/svelte/icons/circle-dot';
	import Gauge from '@lucide/svelte/icons/gauge';
	import SignalHigh from '@lucide/svelte/icons/signal-high';
	import Coins from '@lucide/svelte/icons/coins';
	import MessageSquare from '@lucide/svelte/icons/message-square';
	import Cpu from '@lucide/svelte/icons/cpu';
	import Timer from '@lucide/svelte/icons/timer';
	import UserRound from '@lucide/svelte/icons/user-round';
	import ChevronRight from '@lucide/svelte/icons/chevron-right';
	import GitBranch from '@lucide/svelte/icons/git-branch';
	import CornerDownRight from '@lucide/svelte/icons/corner-down-right';
	import { TableRow, TableCell, Table, TableHeader, TableHead, TableBody } from '$lib/components/ui/table';
	import { StatusSelect } from '$lib/components/orch/status-select';
	import { Checkbox } from '$lib/components/ui/checkbox';
	import { Alert, AlertTitle, AlertDescription } from '$lib/components/ui/alert';
	import Link2 from '@lucide/svelte/icons/link-2';
	import OctagonX from '@lucide/svelte/icons/octagon-x';
	import Square from '@lucide/svelte/icons/square';
	import MessageCircleQuestion from '@lucide/svelte/icons/message-circle-question';
	import Check from '@lucide/svelte/icons/check';
	import Hourglass from '@lucide/svelte/icons/hourglass';
	import { Message, MessageContent, MessageFooter, MessageHeader } from '$lib/components/orch/message';
	import { Bubble, BubbleContent } from '$lib/components/orch/bubble';
	import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogBody } from '$lib/components/ui/dialog';
	import { Textarea } from '$lib/components/ui/textarea';
	import { untrack, onDestroy, tick, setContext, flushSync, type Component } from 'svelte';
	import GripVertical from '@lucide/svelte/icons/grip-vertical';
	import MousePointer2 from '@lucide/svelte/icons/mouse-pointer-2';
	import LoaderCircle from '@lucide/svelte/icons/loader-circle';
	import { toast } from 'svelte-sonner';
	import { useMock } from '$lib/api/env';
	import type { ApiRun } from '$lib/api/types';
	import { ago } from '$lib/time';
	import { project as wb, openProject, closeProject, viewTasks, viewIssues, viewAgents, moveTask, runsOf, stopRun } from '$lib/project.svelte';
	import { SvelteFlow, SvelteFlowProvider, Background, BackgroundVariant, Panel, MarkerType, type Node, type Edge } from '@xyflow/svelte';
	import DiagramToolbar from '$lib/components/orch/diagram/diagram-toolbar.svelte';
	import Spline from '@lucide/svelte/icons/spline';
	import Puzzle from '@lucide/svelte/icons/puzzle';
	import Plug from '@lucide/svelte/icons/plug';
	import LayoutGrid from '@lucide/svelte/icons/layout-grid';
	import Maximize from '@lucide/svelte/icons/maximize';
	import '@xyflow/svelte/dist/style.css';
	import DiagramNode, { DIAGRAM_DRAG, type DiagramDrag, type DiagramNodeData } from '$lib/components/orch/diagram/diagram-node.svelte';
	import type { MenuEntry } from '$lib/components/ui/dropdown-menu';
	import { ContextMenu, ContextMenuTrigger, ContextMenuContent, ContextMenuLabel, ContextMenuSeparator, ContextMenuEntries } from '$lib/components/ui/context-menu';
	import UserPlus from '@lucide/svelte/icons/user-plus';
	import StepForward from '@lucide/svelte/icons/step-forward';
	import PanelRight from '@lucide/svelte/icons/panel-right';
	import FilterIcon from '@lucide/svelte/icons/filter';
	import SquarePen from '@lucide/svelte/icons/square-pen';
	import SubRunNode, { subRunStatus, subRunMode, tierTone } from '$lib/components/orch/diagram/sub-run-node.svelte';
	import TokenMeter from '$lib/components/orch/diagram/token-meter.svelte';
	import FileCode from '@lucide/svelte/icons/file-code';
	import FilePlus from '@lucide/svelte/icons/file-plus';
	import FileX from '@lucide/svelte/icons/file-x';
	import Split from '@lucide/svelte/icons/split';
	import RotateCcw from '@lucide/svelte/icons/rotate-ccw';
	import ArrowRight from '@lucide/svelte/icons/arrow-right';
	import FolderGit2 from '@lucide/svelte/icons/folder-git-2';
	import Trash2 from '@lucide/svelte/icons/trash-2';
	import { Badge } from '$lib/components/ui/badge';
	import { KanbanBoard, KanbanColumn, KanbanColumnHeader, KanbanColumnTitle, KanbanColumnCount, KanbanColumnActions, KanbanColumnContent, KanbanCard, KanbanCardHeader, KanbanCardContent, KanbanCardFooter, type KanbanValue } from '$lib/components/orch/kanban';
	import { Progress } from '$lib/components/ui/progress';
	import { Tabs, TabsList, TabsTrigger, TabsContent } from '$lib/components/ui/tabs';
	import { Empty, EmptyHeader, EmptyMedia, EmptyTitle, EmptyDescription } from '$lib/components/ui/empty';
	import { InputGroup, InputGroupAddon, InputGroupInput, InputGroupButton } from '$lib/components/ui/input-group';
	import { Item, ItemContent, ItemDescription, ItemActions, ItemTitle } from '$lib/components/ui/item';
	import { AvatarGroup, AvatarBadge } from '$lib/components/ui/avatar';
	import { Button } from '$lib/components/ui/button';
	import { Toggle } from '$lib/components/ui/toggle';
	import { OrchCard, OrchCardHeader } from '$lib/components/orch/orch-card';
	import { LevelBadge } from '$lib/components/orch/level-badge';
	import { DecisionRecord } from '$lib/components/orch/decision-record';
	import { StatusBadge } from '$lib/components/orch/status-badge';
	import { RoleAvatar } from '$lib/components/orch/role-avatar';
	import { RuntimeLogo } from '$lib/components/orch/runtime-logo';
	import { statuses, statusOrder, type TaskStatus } from '$lib/status';
	import { roles } from '$lib/roles';
	import { tasks, agents, logs, issues, taskDetails, agentActivity, decisions, thread, subRuns, leadRuns, agentQueues, agentUsage, type Issue, type Chat, type DockCard, type SubRun, type SpawnMode } from '$lib/mock';
	import { store, defaultTeam } from '$lib/teams.svelte';
	import { Segmented } from '$lib/components/orch/segmented';
	import { Pill } from '$lib/components/orch/pill';
	import { Inspector, InspectorHeader, InspectorBody, InspectorSection, InspectorValueRow } from '$lib/components/orch/inspector';
	import Bot from '@lucide/svelte/icons/bot';
	import Users from '@lucide/svelte/icons/users';
	import GitFork from '@lucide/svelte/icons/git-fork';
		import Pencil from '@lucide/svelte/icons/pencil';
	import ChevronsUpDown from '@lucide/svelte/icons/chevrons-up-down';
	import Maximize2 from '@lucide/svelte/icons/maximize-2';
	import CircleDotIcon from '@lucide/svelte/icons/circle-dot';
	import List from '@lucide/svelte/icons/list';
	import Code from '@lucide/svelte/icons/code';
	import Link2Icon from '@lucide/svelte/icons/link-2';
	import TimerIcon from '@lucide/svelte/icons/timer';
	import Tag from '@lucide/svelte/icons/tag';
	import Flag from '@lucide/svelte/icons/flag';
	import UserRoundX from '@lucide/svelte/icons/user-round-x';
	import { DropdownMenu, DropdownMenuTrigger, DropdownMenuContent, DropdownMenuItem, DropdownMenuLabel, DropdownMenuSeparator, DropdownMenuEntries } from '$lib/components/ui/dropdown-menu';
	import { Kbd } from '$lib/components/ui/kbd';
	import { ChoiceCards, ChoiceCard } from '$lib/components/orch/choice-cards';
	import { mergeProps } from 'bits-ui';
	import { Input } from '$lib/components/ui/input';
	import { HoverCard, HoverCardContent } from '$lib/components/ui/hover-card';
	import AssigneePicker from '$lib/components/orch/task/assignee-picker.svelte';
	import TaskEditorDialog, { blankDraft, type TaskDraft } from '$lib/components/orch/task/task-editor-dialog.svelte';
	import { recommendFor, orchPick } from '$lib/assign';
	import DependsPicker from '$lib/components/orch/task/depends-picker.svelte';
	import PriorityPicker from '$lib/components/orch/task/priority-picker.svelte';
	import { Checklist } from '$lib/components/orch/checklist';
	import { priorities, priorityOrder, type Priority } from '$lib/priority';
	import type { Role } from '$lib/roles';
	import type { TaskDetail } from '$lib/mock';

	const project = $derived(store.projects.find((p) => p.sn === Number(page.params.project)));

	// 태스크 · 이슈 · 에이전트 — 목데이터 모드(VITE_MOCK=1)면 화면 안에서만 바꾸고, 아니면 프로젝트 스토어(snapshot + SSE)를 본다.
	let local = $state(tasks.map((t) => ({ ...t })));
	// 일시정지한 태스크 (태스크 메뉴 Pause · Resume). Diagram 노드가 스크립트 초기화 중 메뉴를 만들어 위에 둔다.
	let paused = $state<number[]>([]);
	// 일시정지 · 중지한 에이전트 (에이전트 메뉴 · 카드). Diagram 에이전트 노드도 초기화 중 메뉴를 만들어 위에 둔다.
	let agentHold = $state<Record<number, 'paused' | 'stopped'>>({});
	const list = $derived(useMock ? local : viewTasks());
	const issueList = $derived(useMock ? issues : viewIssues());
	const agentList = $derived(useMock ? agents : viewAgents());
	// 프로젝트가 바뀌면 그 프로젝트의 snapshot · 스트림으로 갈아탄다
	$effect(() => {
		const sn = Number(page.params.project);
		if (!useMock && sn) void openProject(sn);
	});
	onDestroy(closeProject);
	/// 상태 변경 = 서버 command(POST /tasks/{sn}/move). 막힌 전이(409)는 문구로 알리고 화면은 서버 값으로 돌아간다.
	async function setStatus(num: number, status: TaskStatus) {
		if (useMock) {
			local.find((t) => t.num === num)!.status = status;
			return;
		}
		const err = await moveTask(num, status);
		if (err) toast.warning(err);
	}

	// 뷰는 URL(?view=)에 둔다 — 새로고침 · 링크 공유 시 유지. 기본은 Kanban (#18).
	const views = [
		{ value: 'diagram', label: 'Diagram', icon: Workflow, issue: '#56' },
		{ value: 'kanban', label: 'Kanban', icon: KanbanSquare, issue: '#54' },
		{ value: 'issues', label: 'Issues', icon: ListTree, issue: '#55' }
	];
	const view = $derived(views.find((v) => v.value === page.url.searchParams.get('view')) ?? views[1]);

	/// 뷰 전환. 기록을 쌓지 않고 현재 URL만 바꾼다.
	function setView(v: string) {
		const url = new URL(page.url);
		url.searchParams.set('view', v);
		goto(url, { replaceState: true, keepFocus: true, noScroll: true });
	}

	// 영역 접기
	let leftOpen = $state(true);
	let dockOpen = $state(true);
	let opsOpen = $state(true);

	// Quick Panel — 선택된 태스크는 뷰 · Inspector(#57)와 공유한다.
	let panelTab = $state('tasks');
	let query = $state('');
	let filter = $state<TaskStatus | 'all'>('all');
	let selected = $state<number>();
	// 상세(Task Detail View)는 선택과 함께 열리고, 닫아도 선택은 남는다.
	let detail = $state(false);
	/// 태스크 선택 + 상세 열기 (Quick Panel · Kanban · Issue Board · Diagram 공통).
	function open(num: number) {
		selected = num;
		detail = true;
		inspect = undefined;
		subSel = undefined;
		issueSel = undefined;
	}
	const count = (s: TaskStatus) => list.filter((t) => t.status === s).length;
	const agentName = (sn?: number) => agentList.find((a) => a.sn === sn)?.name;
	const shown = $derived(
		list.filter(
			(t) =>
				(filter === 'all' || t.status === filter) &&
				(query === '' || `#${t.num} ${t.title}`.toLowerCase().includes(query.toLowerCase()))
		)
	);

	// Kanban (.pen #26 Workbench / Kanban Board) — 상태별 열. Failed 열은 해당 태스크가 있을 때만.
	const agentOf = (sn?: number) => agentList.find((a) => a.sn === sn);
	/// 에이전트 대기열 (목데이터 · 화면 상태) — 끌어서 배정하면 늘어난다. Diagram 받침 · Agent 카드 NEXT가 본다.
	let queues = $state(structuredClone(agentQueues));
	const task = (num: number) => list.find((t) => t.num === num)!;

	// ── 드래그로 배정 · 상태 변경 (.pen 드래그로 배정) — 왼쪽 Tasks의 Backlog · Todo 줄만 끈다 ─────────
	const movable = (t: { status: TaskStatus }) => t.status === 'backlog' || t.status === 'todo';
	const drag: DiagramDrag = $state({ task: undefined, assign: (nodeId: string) => dropOnAgent(Number(nodeId.replace('agent-', ''))) });
	setContext(DIAGRAM_DRAG, drag);
	/// Agents 탭 줄 · Kanban 열 중 지금 커서가 올라간 곳.
	let dropAgent = $state<number>();
	let dropLane = $state<TaskStatus>();
	/// 커서를 따라오는 카드 (.pen Drag Ghost) — 화면 밖에 그려 두고 끌기 이미지로 쓴다.
	let ghost = $state<HTMLElement>();
	function startDrag(e: DragEvent, t: { num: number; title: string }) {
		// 끌기 이미지는 이 순간의 모양을 찍으므로 카드 내용을 먼저 그린다
		flushSync(() => (drag.task = { num: t.num, title: t.title }));
		e.dataTransfer?.setData('text/plain', `#${t.num} ${t.title}`);
		if (e.dataTransfer) {
			e.dataTransfer.effectAllowed = 'move';
			if (ghost) e.dataTransfer.setDragImage(ghost, 20, 24);
		}
	}
	function endDrag() {
		drag.task = undefined;
		dropAgent = undefined;
		dropLane = undefined;
	}
	/// 에이전트에 놓기 = Assign Agent와 같은 동작 + 대기열 맨 뒤. 되돌리기는 알림에서.
	function dropOnAgent(sn: number) {
		const num = drag.task?.num;
		endDrag();
		if (num === undefined) return;
		const t = task(num);
		const prev = t.agent;
		if (prev === sn) return;
		t.agent = sn;
		if (useMock) {
			if (prev !== undefined) queues[prev] = (queues[prev] ?? []).filter((x) => x.num !== num);
			const q = (queues[sn] ??= []);
			if (!q.some((x) => x.num === num && x.title === t.title)) q.push({ num, title: t.title, note: '바로 시작 가능', issue: `#${t.issue}`, est: '', priority: t.priority });
			refreshQueue(sn, prev);
		}
		const at = (queues[sn] ?? []).findIndex((x) => x.num === num && x.title === t.title) + 1;
		toast.success(`#${num} → ${agentName(sn)} 배정${at ? ` · 대기 ${at}번째` : ''}`, {
			action: {
				label: '되돌리기',
				onClick: () => {
					t.agent = prev;
					if (useMock) {
						queues[sn] = queues[sn].filter((x) => !(x.num === num && x.title === t.title));
						refreshQueue(sn, prev);
					}
				}
			}
		});
	}
	/// Kanban 열에 놓기 = 상태 변경. Backlog ↔ Todo만 받는다 — 실행 상태는 Orch · 담당자가 연다.
	function dropOnLane(status: TaskStatus) {
		const num = drag.task?.num;
		endDrag();
		if (num !== undefined && movable({ status })) void setStatus(num, status);
	}
	/// 대기열이 바뀐 에이전트 노드의 받침을 다시 그린다 (노드 위치는 그대로).
	function refreshQueue(...sns: (number | undefined)[]) {
		nodes = nodes.map((n) => (sns.some((sn) => sn !== undefined && n.id === `agent-${sn}`) ? { ...n, data: { ...n.data, queue: [...(queues[Number(n.id.slice(6))] ?? [])] } } : n)) as typeof nodes;
	}
	const lanes = $derived(statusOrder.filter((s) => s !== 'failed' || count(s) > 0));
	// 드래그 중 열 배치는 따로 두고, 놓을 때 태스크 상태에 반영한다.
	let board = $state<KanbanValue>({});
	$effect.pre(() => {
		board = Object.fromEntries(statusOrder.map((s) => [s, list.filter((t) => t.status === s).map((t) => t.num)]));
	});
	/// 놓은 열을 태스크 상태로 반영한다 — 바뀐 태스크만 command.
	function drop() {
		for (const [s, nums] of Object.entries(board)) for (const n of nums as number[]) if (task(n).status !== s) void setStatus(n, s as TaskStatus);
	}

	// Issue Board (.pen #26 Workbench / Issue Board) — 이슈 → 하위 이슈 → 태스크 트리.
	const issueOf = (num: number) => issueList.find((i) => i.num === num);
	const subIssues = (num: number) => issueList.filter((i) => i.parent === num);
	const tasksOf = (num: number) => list.filter((t) => t.issue === num);
	/// 하위 이슈까지 포함한 태스크.
	const allTasks = (num: number): typeof list => [...tasksOf(num), ...subIssues(num).flatMap((i) => allTasks(i.num))];
	// 이슈 상태(open · in_progress · done · closed)를 태스크 상태 아이콘으로 표시한다.
	const issueIcon = { open: 'todo', in_progress: 'in_progress', done: 'done', closed: 'cancelled' } as const;
	const issueLabel = { open: 'Open', in_progress: 'In Progress', done: 'Done', closed: 'Closed' } as const;
	// Issue 상세 — .pen에 별도 화면이 없어 Task Detail 틀을 그대로 쓴다. 태스크 상세 · 에이전트 카드와 동시에 열지 않는다.
	let issueSel = $state<number>();
	const curIssue = $derived(issueSel === undefined ? undefined : issueOf(issueSel));
	/// 이슈 상세 열기 (Issue Board · Diagram 공통).
	function openIssue(num: number) {
		issueSel = num;
		detail = false;
		inspect = undefined;
	}
	// 공유 링크(?task= · ?issue= · ?agent=)로 오면 상세를 연다 (Copy link · Tasks 화면).
	$effect(() => {
		const t = Number(page.url.searchParams.get('task'));
		const i = Number(page.url.searchParams.get('issue'));
		// 알림(상단 벨)의 결정 답하기 → 그 태스크의 결정 패널
		const d = Number(page.url.searchParams.get('decide'));
		// 멤버 상세 … "Workbench에서 보기" → 그 에이전트 카드
		const ag = Number(page.url.searchParams.get('agent'));
		untrack(() => {
			if (d) openDecisions(d);
			else if (ag && agentOf(ag)) openAgent(ag);
			else if (t && list.some((x) => x.num === t)) open(t);
			else if (i && issueOf(i)) openIssue(i);
		});
	});
	let expanded = $state(new Set([51, 52, 53]));
	/// 이슈 펼침/접기.
	function toggle(num: number) {
		const next = new Set(expanded);
		if (!next.delete(num)) next.add(num);
		expanded = next;
	}

	// Diagram (.pen #26 Workbench / Diagram) — 3열: Project → Orch → Issue / Task / Agent. 사용자 배치 저장은 #59.
	// 1열은 관계 순서대로 둔다 (프로젝트의 PM이 이슈를 위임). .pen은 Project · Issue · Orch 순이라 흐름이 거꾸로 읽혔다.
	// 하위 작업 방식 (.pen Task 상세 · 하위 작업 방식) — 태스크 값이 없으면 팀 기본값. 팀이 허용하지 않은 방식은 고를 수 없다 (#67).
	const team = $derived(store.crew.find((t) => t.project === project?.name) ?? defaultTeam());
	const spawnPolicy = $derived(store.policies[team.sn].spawn);
	const spawnModes: { value: SpawnMode; icon: typeof Cpu }[] = [{ value: 'sub', icon: GitBranch }, { value: 'fork', icon: GitFork }, { value: 'runner', icon: Cpu }];
	const spawnOptions = $derived([
		{ value: 'team', label: `팀 기본: ${spawnPolicy.mode}`, icon: Users },
		...spawnModes.map((m) => ({ ...m, label: m.value, disabled: !spawnPolicy.allow.includes(m.value), hint: `${team.name}에서 허용하지 않은 방식이에요 · Teams › Orch 진행 정책` }))
	]);
	// 4 · 5열은 리드의 하위 작업 (#67)
	const col = [0, 260, 520, 780, 1020];
	const nodeTypes = { diagram: DiagramNode, subrun: SubRunNode };
	// 하위 작업 (.pen Diagram · 하위 작업) — 리드 토큰은 자기 사용량 + runner Run 합계 (재시도 전 Run 포함).
	const runnerTotal = (task: number) => subRuns.filter((s) => s.task === task && s.mode === 'runner').reduce((n, s) => n + s.runs.reduce((m, r) => m + r.tokens, 0), 0);
	let subSel = $state<string>();
	const subCur = $derived(subRuns.find((s) => s.id === subSel));
	/// 하위 작업 카드 열기 — 다른 카드 · 상세는 닫는다.
	function openSub(id: string) {
		detail = false;
		issueSel = undefined;
		inspect = undefined;
		subSel = id;
	}
	/// 태스크 노드 데이터. 상태가 바뀌면 다시 계산한다.
	function taskData(num: number): DiagramNodeData {
		const t = task(num);
		const a = agentOf(t.agent);
		const alerts: Record<number, DiagramNodeData['alert']> = {
			128: { text: '컨텍스트 92% · 경고', tone: 'destructive' },
			129: { text: '판단 대기 L2 · 9:42', tone: 'warning' }
		};
		return {
			kind: 'task',
			ref: `TASK #${t.num}`,
			title: t.title,
			who: a && { role: a.role, name: a.name, runtime: a.runtime, model: t.model },
			progress: t.steps[1] ? { value: (t.steps[0] / t.steps[1]) * 100, text: `${t.steps[0]}/${t.steps[1]}` } : undefined,
			alert: alerts[t.num],
			status: t.status,
			meta: t.run ? `Run ${t.run}` : t.priority,
			menuLabel: `Task #${t.num} · ${t.title}`,
			menu: taskMenu(t.num)
		};
	}
	/// 에이전트 노드 데이터.
	function agentData(sn: number, ctx: string): DiagramNodeData {
		const a = agentOf(sn)!;
		const lead = subRuns.filter((s) => s.lead === sn);
		const leadTask = lead[0]?.task;
		return {
			...(leadTask && { tokens: { self: leadRuns[leadTask].self, runner: runnerTotal(leadTask) } }),
			kind: 'agent',
			ref: roles[a.role].label.toUpperCase(),
			title: a.name,
			who: { role: a.role, name: a.runtime === 'claude' ? 'Claude Code' : 'Codex CLI', runtime: a.runtime },
			badge: agentState(a),
			menuLabel: `${a.name} · ${roles[a.role].label}`,
			menu: agentMenu(sn),
			meta: lead.length ? `sub-run ${lead.length} · ctx ${ctx}` : `ctx ${ctx}`,
			// 대기열 받침 — 서버 큐 API 전에는 목데이터에서만 (#88)
			...(useMock && { queue: [...(queues[sn] ?? [])] })
		};
	}
	const diagramTasks = $derived(useMock ? [128, 129, 130, 131] : list.map((t) => t.num));
	// #131은 소라보다 조금 위 — 완료 보고 선(소라 아래 → Orch)의 라벨이 카드에 가리지 않게
	const taskY = [0, 228, 457, 630];
	/// 목데이터 배치 (.pen Diagram 그대로).
	function mockNodes(): (Node<DiagramNodeData> | Node<SubRun>)[] {
		return [
		{ id: 'project', type: 'diagram', position: { x: col[0], y: 0 }, data: { kind: 'project', ref: 'PROJECT', title: 'OrchStack', badge: 'Active', meta: 'Core Team' } },
		{ id: 'issue-51', type: 'diagram', position: { x: col[0], y: 360 }, data: { kind: 'issue', ref: 'ISSUE #51', title: 'Authentication Flow 개선', progress: { value: 29, text: '86K / 300K tok' }, badge: 'In Progress' } },
		{ id: 'orch', type: 'diagram', position: { x: col[0], y: 140 }, data: { kind: 'orch', ref: 'ORCH · PM', title: 'Orch', who: { role: 'orch', name: 'Project Manager', runtime: 'claude', model: 'claude-opus-5.5' }, alert: { text: '판단 대기 3 · 제안 4초 후 진행', tone: 'warning' }, badge: 'Auto · 5초', meta: '자동 3/10' } },
		...diagramTasks.map((n, i) => ({ id: `task-${n}`, type: 'diagram', position: { x: col[1], y: taskY[i] }, data: taskData(n) })),
		...[[2, '92%', 0], [1, '32%', 228], [3, '12%', 457], [4, '6%', 660], [5, '20%', 862]].map(([sn, ctx, y]) => ({
			id: `agent-${sn}`, type: 'diagram', position: { x: col[2], y: y as number }, data: agentData(sn as number, ctx as string)
		})),
		// 진(#129)의 하위 작업 — 2열 · 행마다 리드 옆부터
		...subRuns.map((s, i) => ({ id: `sub-${s.id}`, type: 'subrun', position: { x: col[3 + (i % 2)], y: 228 + Math.floor(i / 2) * 190 }, data: s }))
		];
	}
	/// 서버 데이터 배치: 1열 프로젝트 · Orch · 이슈, 2열 태스크, 3열 멤버. 사용자 배치 저장은 #89.
	function liveNodes(): Node<DiagramNodeData>[] {
		const top = issueList.filter((i) => !i.parent);
		return [
			{ id: 'project', type: 'diagram', position: { x: col[0], y: 0 }, data: { kind: 'project', ref: 'PROJECT', title: project?.name ?? '', badge: project?.status ?? '', meta: team.name } },
			{ id: 'orch', type: 'diagram', position: { x: col[0], y: 140 }, data: { kind: 'orch', ref: 'ORCH · PM', title: 'Orch', who: { role: 'orch', name: 'Project Manager', runtime: 'claude' }, badge: 'Auto', meta: '' } },
			...top.map((i, k) => ({ id: `issue-${i.num}`, type: 'diagram', position: { x: col[0], y: 360 + k * 170 }, data: { kind: 'issue', ref: `ISSUE #${i.num}`, title: i.title, badge: issueLabel[i.status], meta: `tasks ${allTasks(i.num).length}` } as DiagramNodeData })),
			...diagramTasks.map((n, k) => ({ id: `task-${n}`, type: 'diagram', position: { x: col[1], y: k * 200 }, data: taskData(n) })),
			...agentList.map((a, k) => ({ id: `agent-${a.sn}`, type: 'diagram', position: { x: col[2], y: k * 200 }, data: agentData(a.sn, '—') }))
		];
	}
	let nodes = $state.raw<(Node<DiagramNodeData> | Node<SubRun>)[]>(useMock ? mockNodes() : []);
	// 연결선 종류별 색 (.pen Workbench/EdgeLegend). 라벨은 .pen EdgeLabel 칩 모양.
	const stroke = { contains: 'var(--input)', delegate: 'var(--primary)', assigned: 'var(--node-agent)', idle: 'var(--status-review)', live: 'var(--primary)', spawn: 'var(--node-agent)', waits: 'var(--subtle-foreground)' };
	const chip = (live = false) =>
		`font: 600 10px var(--font-mono); padding: 2px 6px; border-radius: 4px; border: 1px solid ${live ? 'var(--primary)' : 'var(--border)'}; background: ${live ? 'var(--primary)' : 'var(--card)'}; color: ${live ? 'var(--primary-foreground)' : 'var(--status-review)'};`;
	/// 연결선 하나.
	function link(id: string, source: string, target: string, kind: keyof typeof stroke, label?: string, handles = ['r', 'l'], dashed = false): Edge {
		return {
			id, source, target, sourceHandle: handles[0], targetHandle: handles[1], type: 'smoothstep', label,
			animated: kind === 'live',
			// 방향: 누가 누구에게 (위임 · 배정 · 요청 · 보고)
			markerEnd: { type: MarkerType.ArrowClosed, color: stroke[kind], width: 16, height: 16 },
			// spawn(sub · fork) · waits는 점선
			style: `stroke: ${stroke[kind]}; stroke-width: ${kind === 'live' ? 3 : 1.5};${dashed ? ' stroke-dasharray: 5 4;' : ''}`,
			labelStyle: label ? chip(kind === 'live') : undefined
		};
	}
	/// 목데이터 연결선.
	function mockEdges(): Edge[] {
		return [
		link('e-p-o', 'project', 'orch', 'contains', 'PM', ['b', 't']),
		link('e-o-i', 'orch', 'issue-51', 'delegate', '위임', ['b', 't']),
		...diagramTasks.map((n) => link(`e-i-${n}`, 'issue-51', `task-${n}`, 'contains')),
		link('e-128', 'task-128', 'agent-2', 'assigned'),
		link('e-129', 'task-129', 'agent-1', 'assigned', '배정'),
		link('e-130', 'task-130', 'agent-3', 'assigned'),
		link('e-131', 'task-131', 'agent-4', 'assigned'),
		link('e-verify', 'agent-1', 'agent-3', 'live', '검증 요청 · 진행 중', ['b', 't']),
		link('e-review', 'agent-3', 'agent-4', 'idle', '리뷰 요청', ['b', 't']),
		link('e-report', 'agent-4', 'orch', 'idle', '완료 보고', ['b', 'l']),
		...subRuns.map((s, i) => link(`e-spawn-${s.id}`, `agent-${s.lead}`, `sub-${s.id}`, 'spawn', i ? undefined : 'spawn', ['r', 'l'], s.mode !== 'runner')),
		...subRuns.filter((s) => s.waits).map((s) => link(`e-waits-${s.id}`, `sub-${s.id}`, `sub-${s.waits!.id}`, 'waits', `queued · waits ${s.waits!.id} · ${s.waits!.glob}`, ['t', 'b'], true))
		];
	}
	/// 서버 데이터 연결선: 프로젝트→Orch, Orch→이슈(위임), 이슈→태스크(포함), 태스크→담당(배정).
	function liveEdges(): Edge[] {
		return [
			link('e-p-o', 'project', 'orch', 'contains', 'PM', ['b', 't']),
			...issueList.filter((i) => !i.parent).map((i) => link(`e-o-${i.num}`, 'orch', `issue-${i.num}`, 'delegate', undefined, ['b', 't'])),
			...list.filter((t) => t.issue).map((t) => link(`e-i-${t.num}`, `issue-${topIssue(t.issue)}`, `task-${t.num}`, 'contains')),
			...list.filter((t) => t.agent !== undefined && agentOf(t.agent)).map((t) => link(`e-${t.num}`, `task-${t.num}`, `agent-${t.agent}`, 'assigned'))
		];
	}
	/// 하위 이슈의 태스크는 최상위 이슈 노드에 단다 (이슈 노드는 최상위만 그린다).
	function topIssue(num: number): number {
		const i = issueOf(num);
		return i?.parent ? topIssue(i.parent) : num;
	}
	let edges = $state.raw<Edge[]>(useMock ? mockEdges() : []);
	// 서버 모드: 태스크 · 이슈 · 멤버 구성이 바뀌면 다시 배치한다 (같은 id는 자리를 유지).
	$effect(() => {
		if (useMock) return;
		const key = [list.map((t) => `${t.num}:${t.agent}:${t.issue}`).join(), issueList.map((i) => `${i.num}:${i.parent}`).join(), agentList.map((a) => a.sn).join()].join('|');
		untrack(() => {
			void key;
			const pos = new Map(nodes.map((n) => [n.id, n.position]));
			nodes = liveNodes().map((n) => ({ ...n, position: pos.get(n.id) ?? n.position }));
			edges = liveEdges();
		});
	});
	/// 에이전트가 쓰는 스킬 · MCP · 도구 묶음 (.pen Node/Skill · MCP · Tools). 목데이터에만 있다.
	function capsOf(sn: number): DiagramNodeData['caps'] {
		const u = useMock ? agentUsage[sn] : undefined;
		if (!u) return undefined;
		const calls = u.skills.reduce((n, x) => n + x.calls, 0);
		return [
			...(u.skills.length ? [{ kind: 'skill' as const, title: `${u.skills[0].name} Skills ×${u.skills.length}`, sub: `${u.skills[1]?.name ?? u.skills[0].name} · ${calls} calls` }] : []),
			...(u.mcp.length ? [{ kind: 'mcp' as const, title: `MCP ×${u.mcp.length}`, sub: u.mcp.map((m) => m.name.replace(' MCP', '')).join(' · ') }] : []),
			...(u.tools.length ? [{ kind: 'tools' as const, title: `Tools ×${u.tools.length}`, sub: `${u.tools.slice(0, 4).join(', ')}${u.tools.length > 4 ? '…' : ''}` }] : [])
		];
	}
	// 에이전트 카드를 열면 그 노드 옆에 쓰는 스킬 · MCP · 도구를 붙이고, 다른 노드 위로 올린다.
	$effect(() => {
		const sn = inspect;
		untrack(() => {
			nodes = nodes.map((n) => {
				if (!n.id.startsWith('agent-')) return n;
				const on = n.id === `agent-${sn}`;
				const d = n.data as DiagramNodeData;
				if (!on && !d.caps) return n;
				return { ...n, zIndex: on ? 10 : undefined, data: { ...d, caps: on ? capsOf(sn!) : undefined } };
			}) as typeof nodes;
		});
	});
	// 태스크 상태 · 선택이 바뀌면 노드에 반영한다 (위치는 유지).
	$effect(() => {
		const sel = selected;
		const data = Object.fromEntries(diagramTasks.map((n) => [`task-${n}`, taskData(n)]));
		untrack(() => {
			nodes = nodes.map((n) => ({ ...n, data: data[n.id] ?? n.data, selected: n.id === `task-${sel}` }));
		});
	});

	// ── 캔버스 도구 (.pen ViewHeader · Canvas Tools · ContextMenu / Canvas) ─────────
	let toolbar = $state<ReturnType<typeof DiagramToolbar>>();
	let grid = $state(false);
	/// Auto Layout — 기본 배치로 되돌린다 (노드 · 연결선은 그대로).
	function autoLayout() {
		const pos = new Map((useMock ? mockNodes() : liveNodes()).map((n) => [n.id, n.position]));
		nodes = nodes.map((n) => ({ ...n, position: pos.get(n.id) ?? n.position }));
	}
	/// Grid Layout 켜기 · 끄기 — 켜면 노드를 20px 격자(점 배경 간격)에 붙이고, 옮길 때도 격자에 붙인다.
	function gridLayout() {
		grid = !grid;
		if (grid) nodes = nodes.map((n) => ({ ...n, position: { x: Math.round(n.position.x / 20) * 20, y: Math.round(n.position.y / 20) * 20 } }));
	}
	/// 빈 캔버스 우클릭 메뉴 위치 (화면 좌표).
	let paneMenu = $state<{ x: number; y: number }>();
	const canvasMenu: MenuEntry[] = [
		{ label: 'New Task', icon: SquarePen, shortcut: 'N', onSelect: () => newTask() },
		{ label: 'New Issue', icon: CircleDot, onSelect: () => askPm('새 이슈') },
		'sep',
		{ label: 'Auto Layout', icon: Sparkles, onSelect: () => toolbar?.auto() },
		{ label: 'Grid Layout', icon: LayoutGrid, onSelect: gridLayout },
		{ label: 'Fit View', icon: Maximize, onSelect: () => toolbar?.fit() }
	];

	// ── 연결 모드 (.pen ContextMenu / 연결하기 · Hint · 연결 모드) — 태스크 메뉴에서 종류를 고르고 대상 노드를 누른다 ─────────
	// 목데이터: 의존 · 배정은 태스크 값과 연결선을, 검증 · 리뷰 요청은 연결선만 바꾼다. 서버 저장은 API 단계(#92).
	type LinkKind = 'dep' | 'assign' | 'verify' | 'review';
	const linkKinds: Record<LinkKind, string> = { dep: '의존 관계', assign: '멤버에게 배정', verify: '검증 요청', review: '리뷰 요청' };
	let linking = $state<{ from: number; kind: LinkKind }>();
	function startLink(from: number, kind: LinkKind) {
		if (view.value !== 'diagram') setView('diagram');
		detail = false;
		// 메뉴 항목을 누른 클릭이 아래 노드까지 내려가 바로 연결되지 않게, 클릭이 끝난 뒤 켠다
		setTimeout(() => (linking = { from, kind }), 50);
	}
	/// 대상 노드를 눌렀다. 맞지 않는 노드면 안내만 하고 연결 모드를 유지한다.
	function finishLink(nodeId: string) {
		const l = linking!;
		if (l.kind === 'dep') {
			if (!nodeId.startsWith('task-')) return void toast('먼저 끝나야 할 태스크 노드를 눌러 주세요');
			const n = Number(nodeId.slice(5));
			if (n === l.from) return void toast('같은 태스크끼리는 연결할 수 없어요');
			const d = detailOf(l.from);
			if (!d.deps.some((x) => x.kind === 'depends' && x.num === n)) d.deps.push({ kind: 'depends', num: n });
			const o = detailOf(n);
			if (!o.deps.some((x) => x.kind === 'blocks' && x.num === l.from)) o.deps.push({ kind: 'blocks', num: l.from });
			const id = `e-dep-${n}-${l.from}`;
			edges = [...edges.filter((e) => e.id !== id), link(id, nodeId, `task-${l.from}`, 'waits', '먼저', ['b', 't'], true)];
			toast(`#${l.from}은 #${n}이 끝난 뒤 시작해요`);
		} else {
			if (!nodeId.startsWith('agent-')) return void toast('에이전트 노드를 눌러 주세요');
			const sn = Number(nodeId.slice(6));
			if (l.kind === 'assign') {
				task(l.from).agent = sn;
				edges = [...edges.filter((e) => !(e.source === `task-${l.from}` && e.target.startsWith('agent-'))), link(`e-${l.from}`, `task-${l.from}`, nodeId, 'assigned')];
				toast(`#${l.from}을 ${agentName(sn)}에게 맡겼어요`);
			} else {
				const owner = task(l.from).agent;
				const src = owner !== undefined ? `agent-${owner}` : `task-${l.from}`;
				if (src === nodeId) return void toast('담당자 자신에게는 요청할 수 없어요');
				const id = `e-${l.kind}-${l.from}-${sn}`;
				edges = [...edges.filter((e) => e.id !== id), link(id, src, nodeId, 'idle', linkKinds[l.kind], ['b', 't'])];
				toast(`#${l.from} ${linkKinds[l.kind]} → ${agentName(sn)}`);
			}
		}
		linking = undefined;
	}

	// Ops — Activity 외 탭은 실행기 연동(#13) 후 채운다. 라벨은 i18n 도입 시 교체.
	const opsTabs = [
		['activity', 'Activity'],
		['logs', 'Logs'],
		['tools', 'Tool Calls'],
		['events', 'Events'],
		['tests', 'Tests'],
		['git', 'Git']
	];
	let opsTab = $state('activity');
	let live = $state(true);

	// Task Detail View (.pen TaskDetailView) — 가운데 뷰 위에 겹쳐 연다. 상세 데이터가 없는 태스크는 빈 상태로.
	let details = $state(structuredClone(taskDetails));

	// ── 태스크 메뉴 (.pen ContextMenu / Task) — Kanban 카드 우클릭 · Diagram 노드 … 가 같은 목록 ─────────
	// Run 제어(Start · Pause · Resume · Stop · Requeue)는 목데이터: 상태 · 일시정지 표시만 바꾼다. 실제 실행은 실행기(#13) · API 단계.
	function taskMenu(num: number): MenuEntry[] {
		const t = task(num);
		const running = t.status === 'in_progress';
		const isPaused = paused.includes(num);
		return [
			{
				label: 'Assign Agent', icon: UserPlus,
				sub: [...agentList.map((a) => ({ label: `${a.name} · ${roles[a.role].label}`, checked: t.agent === a.sn, onSelect: () => (task(num).agent = a.sn) })), { label: 'Unassigned', checked: t.agent === undefined, onSelect: () => (task(num).agent = undefined) }]
			},
			{ label: 'Start', icon: Play, disabled: running || ['done', 'review'].includes(t.status), onSelect: () => void setStatus(num, 'in_progress') },
			{ label: 'Pause', icon: Pause, disabled: !running || isPaused, onSelect: () => ((paused = [...paused, num]), toast(`#${num} 일시정지 · 현재 단계가 끝나면 멈춰요`)) },
			{ label: 'Resume', icon: StepForward, disabled: !isPaused, onSelect: () => ((paused = paused.filter((x) => x !== num)), toast(`#${num} 다시 진행`)) },
			{ label: 'Stop', icon: Square, disabled: !running, onSelect: () => ((paused = paused.filter((x) => x !== num)), void setStatus(num, 'todo'), toast(`#${num} Run을 멈추고 Todo로 돌렸어요`)) },
			{ label: 'Requeue', icon: RotateCcw, disabled: !['failed', 'cancelled', 'blocked'].includes(t.status), onSelect: () => void setStatus(num, 'todo') },
			'sep',
			{ label: 'Change Priority', icon: Flag, sub: priorityOrder.map((p) => ({ label: `${p} ${priorities[p].label}`, icon: priorities[p].icon, tone: priorities[p].text, checked: t.priority === p, onSelect: () => (task(num).priority = p) })) },
			{ label: 'Add Dependency', icon: Link2Icon, onSelect: () => editTask(num) },
			{
				label: '연결하기…', icon: Spline, subLabel: `연결 종류 · #${num} ${t.title}`,
				sub: [
					{ label: '의존 관계 · 먼저 끝나야 함', icon: Link2Icon, onSelect: () => startLink(num, 'dep') },
					{ label: '멤버에게 배정', icon: UserPlus, onSelect: () => startLink(num, 'assign') },
					{ label: '검증 요청', icon: CircleCheck, onSelect: () => startLink(num, 'verify') },
					{ label: '리뷰 요청', icon: Eye, onSelect: () => startLink(num, 'review') }
				]
			},
			'sep',
			{ label: 'Open Details', icon: PanelRight, shortcut: '↵', onSelect: () => open(num) }
		];
	}

	// ── 에이전트 메뉴 (.pen Menu / Agent) — Quick Panel Agents 행 우클릭 · ⋯ · Diagram 에이전트 노드가 같은 목록 ─────────
	// Pause · Resume · Restart · Stop은 대화를 거치지 않는 즉시 운영 제어. 목데이터: 화면 상태만 바꾼다(실행기 #13 · API 단계).
	/// 지금 상태 한 낱말 (카드 · 노드 배지). 일시정지 · 중지가 우선.
	function agentState(a: { sn: number; activity: string }) {
		return agentHold[a.sn] === 'paused' ? 'Paused' : agentHold[a.sn] === 'stopped' ? 'Stopped' : a.activity.split(' · ')[0];
	}
	function pauseAgent(sn: number) {
		agentHold[sn] = 'paused';
		toast(`${agentName(sn)} 일시정지 · 현재 단계가 끝나면 멈춰요`);
	}
	function stopAgent(sn: number) {
		agentHold[sn] = 'stopped';
		toast(`${agentName(sn)} 실행을 멈췄어요`);
	}
	/// 실행 중 지시 입력으로 (.pen: Agent Card 하단 Runtime instruction 입력에 포커스).
	async function instructAgent(sn: number) {
		openAgent(sn);
		await tick();
		document.getElementById('runtime-instruction')?.focus();
	}
	function agentMenu(sn: number): MenuEntry[] {
		const a = agentOf(sn)!;
		const hold = agentHold[sn];
		const running = !hold && a.activity.startsWith('Running');
		const free = list.filter((t) => t.agent === undefined && !['done', 'cancelled'].includes(t.status));
		const mine = list.find((t) => t.agent === sn && t.status === 'in_progress');
		return [
			{ label: 'Assign Task', icon: ListPlus, disabled: !free.length, sub: free.map((t) => ({ label: `#${t.num} ${t.title}`, onSelect: () => ((task(t.num).agent = sn), toast(`#${t.num}을 ${a.name}에게 맡겼어요`)) })) },
			{ label: 'Send Runtime Instruction', icon: CornerDownRight, onSelect: () => void instructAgent(sn) },
			'sep',
			{ label: 'Pause', icon: Pause, disabled: !running, onSelect: () => pauseAgent(sn) },
			{ label: 'Resume', icon: Play, disabled: hold !== 'paused', onSelect: () => (delete agentHold[sn], toast(`${a.name} 다시 진행`)) },
			{ label: 'Restart', icon: RotateCw, onSelect: () => (delete agentHold[sn], toast(`${a.name} 세션을 다시 시작했어요`)) },
			{ label: 'Stop', icon: Square, tone: 'text-destructive', disabled: hold === 'stopped', onSelect: () => stopAgent(sn) },
			'sep',
			{ label: 'Ask PM about this', icon: MessageSquareShare, tone: 'text-primary', onSelect: () => {
					const run = mine && liveRunOf(mine.num);
					askPm(`Agent ${a.name}${mine ? ` @Task #${mine.num}` : ''}${run ? ` @Run #${run.num}` : ''}`);
				} },
			{ label: '멤버 상세 열기', icon: PanelRightOpen, onSelect: () => goto(`/teams?member=${sn}`) }
		];
	}
	// 추가 메뉴 (.pen Menu / Add) — Agents 탭 머리 +. 템플릿 · 팀 만들기 흐름은 아직 디자인에 없어 막아 둔다(#60).
	const addMenu: MenuEntry[] = [
		{ label: '멤버 추가 · 템플릿에서', icon: UserPlus, onSelect: () => goto('/teams?add=') },
		{ label: '새 에이전트 템플릿', icon: LayoutTemplate, disabled: true },
		'sep',
		{ label: '새 팀', icon: Users, disabled: true }
	];

	// Kanban 카드 hover 미리보기 (.pen KanbanCard/HoverPreview) — 커서 +18px, 화면 끝에선 반대쪽. 끌 때 · 메뉴 열 때는 숨긴다.
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

	// ── Task Editor (orch/task/task-editor-dialog) · QuickAdd (.pen biSss) ─────────────────────
	// 목데이터 단계: 만들고 고친 값은 페이지 목록(local) · 상세(details)에 바로 넣는다. 서버 저장은 API 단계(#92).
	let editor = $state<ReturnType<typeof TaskEditorDialog>>();
	let editorOpen = $state(false);
	/// 새 태스크 기본 이슈: 열린 상세의 이슈 → 첫 진행 중 이슈 → 첫 이슈.
	const defaultIssue = () => (cur ? cur.issue : (issueList.find((i) => i.status === 'in_progress') ?? issueList[0])?.num ?? 0);

	/// 새 태스크 편집기 (QuickAdd ⤢ · N).
	function newTask(over: Partial<TaskDraft> = {}) {
		quick = undefined;
		editor?.create({ issue: defaultIssue(), ...over });
	}
	/// 편집 모드 (Task 상세 ✎ · E).
	const editTask = (num: number) => editor?.edit(num);
	/// 상세 기록이 없는 태스크는 빈 상세를 만들어 둔다 (인라인 편집 · 의존 추가).
	function detailOf(num: number): TaskDetail {
		return (details[num] ??= { description: task(num).description ?? '', criteria: [], deps: [], runs: [], activity: [], eta: '', labels: [] });
	}
	// 완료 조건을 바꾸면 카드 · 목록의 진행(steps)도 맞춘다 (.pen B · 인라인 체크).
	$effect(() => {
		const n = cur?.num;
		const c = n === undefined ? undefined : details[n]?.criteria;
		if (n === undefined || !c) return;
		const steps: [number, number] = [c.filter((x) => x.done).length, c.length];
		untrack(() => {
			const t = list.find((x) => x.num === n);
			if (t && (t.steps[0] !== steps[0] || t.steps[1] !== steps[1])) t.steps = steps;
		});
	});
	/// 진행 중 Run (편집 경고 줄 · Runtime Instruction 전달).
	const liveRunOf = (num?: number) => (num === undefined ? undefined : details[num]?.runs.find((r) => r.live));

	/// 태스크 1개를 목록 · 상세에 넣는다. 번호는 프로젝트 안에서 다음 번호.
	function createTask(d: TaskDraft): number {
		const num = Math.max(0, ...list.map((t) => t.num)) + 1;
		local.push({ num, project: project?.sn, title: d.title.trim(), status: d.status, priority: d.priority, agent: d.agent, issue: d.issue, steps: [d.criteria.filter((c) => c.done).length, d.criteria.length], messages: 0, updated: 'just now' });
		details[num] = { description: d.body, criteria: d.criteria, deps: d.deps, runs: [], activity: [{ type: 'CREATE', who: '나', time: now(), text: `Task #${num} 생성` }], eta: d.eta, labels: d.labels };
		return num;
	}
	/// 편집기 저장 — 새 태스크면 만든 번호, 편집이면 그 번호. 서버 모드는 아직 저장하지 않는다(undefined).
	function saveDraft(d: TaskDraft): number | undefined {
		if (!useMock) {
			toast.info('서버 저장은 API 단계에서 붙여요');
			return;
		}
		if (d.num === undefined) {
			const num = createTask(d);
			toast.success(`Task #${num}을 만들었어요`);
			return num;
		}
		const t = task(d.num);
		Object.assign(t, { title: d.title.trim(), status: d.status, priority: d.priority, agent: d.agent, issue: d.issue, steps: [d.criteria.filter((c) => c.done).length, d.criteria.length], updated: 'just now' });
		const prev = details[d.num];
		details[d.num] = { ...(prev ?? { runs: [], activity: [] }), description: d.body, criteria: d.criteria, deps: d.deps, eta: d.eta, labels: d.labels };
		const run = liveRunOf(d.num);
		if (run) {
			details[d.num].activity.push({ type: 'RUNTIME_INSTRUCTION', who: `나 → ${agentName(t.agent) ?? '담당'}`, time: now(), text: `Task #${d.num} 변경 사항 전달 · Run #${run.num}` });
			toast.success('저장했어요', { description: `Run #${run.num}에 Runtime Instruction으로 전달했어요` });
		} else toast.success('저장했어요');
		return d.num;
	}

	// QuickAdd — Quick Panel 헤더 +. Enter로 바로 만들고, ⤢로 편집기에 이어서.
	let quick = $state<{ title: string; status: TaskStatus; agent?: number; priority: Priority }>();
	function openQuick(status: TaskStatus = 'todo') {
		leftOpen = true;
		panelTab = 'tasks';
		quick = { title: '', status, priority: 'P2' };
	}
	function quickSubmit(e: SubmitEvent) {
		e.preventDefault();
		if (!quick?.title.trim()) return;
		if (!useMock) return toast.info('서버 저장은 API 단계에서 붙여요');
		const num = createTask(blankDraft(defaultIssue(), { ...quick }));
		toast.success(`Task #${num}을 만들었어요`);
		quick = { ...quick, title: '' };
	}
	const cur = $derived(detail && selected !== undefined ? list.find((t) => t.num === selected) : undefined);
	const info = $derived(cur ? details[cur.num] : undefined);
	// Run 목록 — 목데이터는 상세에 있고, 서버는 상세를 열 때 GET /tasks/{sn}/runs. 토큰은 #88 뒤에.
	let apiRuns = $state<ApiRun[]>([]);
	$effect(() => {
		const n = cur?.num;
		apiRuns = [];
		if (!useMock && n !== undefined) runsOf(n).then((r) => (apiRuns = r));
	});
	const activeRun = ['queued', 'starting', 'running', 'waiting', 'review'];
	const liveRun = $derived(apiRuns.find((r) => activeRun.includes(r.status)));
	const runs = $derived(
		info?.runs ?? apiRuns.map((r) => ({ num: r.num, note: r.result_summary ? `${r.status} · ${r.result_summary}` : r.status, time: ago(r.start_at ?? r.create_at), tokens: '—', live: activeRun.includes(r.status) }))
	);
	/// 이 태스크 · 이슈를 PM Dock 대화로 넘긴다 (.pen "Ask PM about this"). ref 예: "Task #129".
	function askPm(ref: string) {
		dockOpen = true;
		draft = `@${ref} `;
	}

	// Agent Inspector (.pen Agent Inspector Card) — 에이전트를 누르면 뷰 오른쪽에 카드로 연다. 태스크 상세와 동시에 열지 않는다.
	let inspect = $state<number>();
	let inspectTab = $state('overview');
	let activity = $state(structuredClone(agentActivity));
	let instruction = $state('');
	const agentSel = $derived(agentOf(inspect));
	/// 에이전트 카드 열기.
	function openAgent(sn: number) {
		detail = false;
		issueSel = undefined;
		subSel = undefined;
		inspect = sn;
	}
	/// 실행 중 지시 보내기 (.pen RuntimeInstructionComposer). 서버 전송은 #59, 지금은 활동 기록에만 남긴다.
	function instruct(e: SubmitEvent) {
		e.preventDefault();
		const text = instruction.trim();
		if (!text || inspect === undefined) return;
		(activity[inspect] ??= []).push({ type: 'RUNTIME_INSTRUCTION', who: `나 → ${agentSel?.name}`, time: new Date().toTimeString().slice(0, 5), text });
		instruction = '';
	}

	// PM Dock — Orch 대화 (.pen ProjectPMChatDock). 서버 대화 API(#59) 전까지 화면 안에서만 유지한다.
	let draft = $state('');
	// 검색(⌘K) 명령에서 ?ask=로 오면 PM Dock 입력창에 채워 둔다 (보내기는 사용자가).
	$effect(() => {
		const ask = page.url.searchParams.get('ask');
		if (ask) untrack(() => (draft = ask));
	});
	let chat = $state<Chat[]>(structuredClone(thread));
	const now = () => new Date().toTimeString().slice(0, 5);
	let threadEl = $state<HTMLElement>();
	// 새 메시지가 오면 맨 아래로
	$effect(() => {
		chat.length;
		threadEl?.scrollTo({ top: threadEl.scrollHeight });
	});

	/// 입력을 보내고 입력창을 비운다.
	function send(e: SubmitEvent) {
		e.preventDefault();
		const text = draft.trim();
		if (!text) return;
		chat.push({ kind: 'user', text, time: now() });
		draft = '';
	}
	/// 작업 제안 진행 → 명령 결과 카드 (.pen WorkProposalCard → CommandResultCard). 실제 생성은 서버 command(#59).
	function proceed(c: Extract<Chat, { kind: 'proposal' }>) {
		c.status = 'proceeded';
		chat.push({
			kind: 'result',
			time: now(),
			lines: [`Issue "${c.issue}" created`, ...c.tasks.map((t, i) => `Task ${i + 1}. ${t.title} → ${agentOf(t.agent)?.name} · ${roles[agentOf(t.agent)!.role].label}`)]
		});
	}

	// ── PM Dock 진행 카드 (.pen Kk8hB A · C–H) — 목데이터: 버튼을 누르면 결과 한 줄 + 알림. 실제 명령은 Orch API(#87).
	// A 타이머 · G 재시도는 초를 세다 0이면 스스로 진행한다. 카드를 보고 있으면(포인터가 위에) 멈추고, '멈춤'을 누르면 계속 멈춘다.
	let cardHold = $state<Record<number, 'view' | 'stop'>>({});
	let altOpen = $state<number>();
	function settle(card: DockCard, result: string) {
		card.done = result;
		toast(result);
	}
	function assignNext(card: Extract<DockCard, { type: 'timer' }>, agent = card.proposal.agent, label?: string) {
		task(card.proposal.task).agent = agent;
		settle(card, label ?? `#${card.proposal.task}을 ${agentName(agent)}에게 배정했어요`);
	}
	$effect(() => {
		const id = setInterval(() => {
			chat.forEach((c, i) => {
				if (c.kind !== 'card' || c.card.done || cardHold[i]) return;
				const k = c.card;
				if (k.type !== 'timer' && k.type !== 'failed') return;
				if (k.seconds > 0) k.seconds -= 1;
				if (k.seconds > 0) return;
				if (k.type === 'timer') assignNext(k);
				else settle(k, `Run 재시도 · #${k.task} (${k.tries[0]} / ${k.tries[1]}회)`);
			});
		}, 1000);
		return () => clearInterval(id);
	});

	// DecisionPanel (.pen DecisionPanel) — 판단 대기는 하나의 다이얼로그. 범위: 전체 / 이 태스크만. 질문 위 · 답변 입력 아래.
	let queue = $state(structuredClone(decisions));
	let panel = $state(false);
	let scope = $state<number>();
	let active = $state<number>();
	let pick = $state<string>();
	let answer = $state('');
	const pending = $derived(queue.filter((d) => d.left));
	const shownQueue = $derived(scope === undefined ? queue : queue.filter((d) => d.task === scope));
	const dec = $derived(queue.find((d) => d.id === active));
	const qi = $derived(dec ? dec.questions.findIndex((q) => !q.answer) : -1);
	/// 판단 패널 열기. task를 주면 그 태스크 것만.
	function openDecisions(task?: number) {
		scope = task;
		active = (task === undefined ? pending[0] : queue.find((d) => d.task === task && d.left))?.id ?? shownQueue[0]?.id;
		pick = undefined;
		answer = '';
		panel = true;
	}
	/// 현재 질문에 답하고 다음 질문으로. 모두 답하면 결정됨 — 에이전트에게는 활동 기록(Runtime Instruction)으로 전달.
	function reply(e: SubmitEvent) {
		e.preventDefault();
		submitAnswer();
	}
	/// PM Dock 카드 B에서 선택지를 바로 눌러 답한다.
	function quickAnswer(id: number, key: string) {
		active = id;
		pick = key;
		answer = '';
		submitAnswer();
	}
	function submitAnswer() {
		if (!dec || qi < 0) return;
		const q = dec.questions[qi];
		const opt = q.options?.find((o) => o.key === pick);
		const text = [opt && `${opt.key} · ${opt.title}`, answer.trim()].filter(Boolean).join(' — ');
		if (!text) return;
		q.answer = text;
		(activity[dec.agent] ??= []).push({ type: 'DECISION', who: `나 → ${agentOf(dec.agent)?.name}`, time: now(), text: `Q ${q.q} → ${text}` });
		// 결정 기록 (.pen Decision Record) — Task 상세 Activity에 한 줄, 누르면 카드
		detailOf(dec.task).activity.push({ type: 'DECISION', who: '나', time: now(), text: `Q ${q.q} → ${text}`, record: { by: 'me', task: dec.task, time: now(), q: q.q, a: text, summary: `#${dec.task} ${q.q} → ${opt?.title ?? text}`, sent: `${agentOf(dec.agent)?.name ?? '담당'}에게 전달됨` } });
		if (dec.questions.every((x) => x.answer)) {
			dec.left = undefined;
			dec.decided = '방금';
			if (details[dec.task]) details[dec.task].decision = undefined;
		}
		pick = undefined;
		answer = '';
	}
</script>

<svelte:head><title>{project?.name ?? '프로젝트'} · OrchStack</title></svelte:head>
<!-- onmousemove: 끄던 줄이 탭 전환으로 사라지면 dragend가 오지 않는다 — 드래그가 끝난 뒤 첫 마우스 이동에서 정리 -->
<svelte:window
	onmousemove={() => drag.task && endDrag()}
	onkeydown={(e) => {
		// 편집기 · 다이얼로그가 열려 있으면 그쪽이 키를 처리한다
		if (editorOpen || panel) return;
		const typing = e.target instanceof HTMLElement && (e.target.closest('input, textarea, [contenteditable]') !== null);
		// 선택 창 · 메뉴 안에서 누른 Esc는 그 창만 닫는다
		const layer = e.target instanceof HTMLElement && e.target.closest('[data-slot$="-content"], [role="menu"], [role="listbox"], [cmdk-root]') !== null;
		if (e.key === 'Escape') {
			if (layer || e.defaultPrevented) return;
			if (linking) return void (linking = undefined);
			if (quick) return void (quick = undefined);
			detail = false;
			inspect = undefined;
			issueSel = undefined;
			return;
		}
		if (typing || e.altKey) return;
		// N · ⌘N 새 태스크 (브라우저가 ⌘N을 가로채면 N만), E 편집 (.pen A · A')
		if (e.key.toLowerCase() === 'n' && !e.shiftKey) {
			e.preventDefault();
			newTask();
		} else if (e.key.toLowerCase() === 'e' && !e.metaKey && !e.ctrlKey && cur) {
			e.preventDefault();
			editTask(cur.num);
		}
	}}
/>

<!-- .pen Drag Ghost — 화면 밖에 두고 끌기 이미지로만 쓴다 (기울기 -2°, primary 테두리) -->
<div class="pointer-events-none fixed -top-50 -left-100 p-4" aria-hidden="true">
	<div bind:this={ghost} class="flex w-63 -rotate-2 flex-col gap-1.5 rounded-md border border-primary bg-card px-3 py-2.5 shadow-lg">
		<span class="flex items-center gap-1.5">
			<GripVertical class="size-3.5 shrink-0 text-muted-foreground" />
			<span class="font-mono text-caption font-semibold text-muted-foreground">#{drag.task?.num}</span>
			<span class="truncate text-body font-semibold">{drag.task?.title}</span>
		</span>
		<span class="flex items-center gap-1 text-caption font-medium text-primary"><MousePointer2 class="size-3" />에이전트 · 열 위에 놓으세요 · Esc 취소</span>
	</div>
</div>

<!-- 속성 칩 (.pen TaskEditor · Properties · QuickAdd · Options) — 선택 창 트리거 -->
{#snippet propChip(props: Record<string, unknown>, Icon: Component, label: string, tone = 'text-muted-foreground')}
	<Button {...props} variant="outline" size="xs"><Icon class={tone} />{label}</Button>
{/snippet}
<!-- Task 상세 속성 한 줄 (.pen B3 · Properties). props가 있으면 선택 창 트리거, 없으면 표시만 -->
{#snippet propRow(props: Record<string, unknown> | undefined, label: string, Icon: Component, value: string, tone = 'text-muted-foreground')}
	<Item variant="row" size="xs">
		{#snippet child({ props: row })}
			<svelte:element this={props ? 'button' : 'div'} {...mergeProps(row, props ?? {})}>
				<ItemContent><ItemDescription>{label}</ItemDescription></ItemContent>
				<ItemActions class="min-w-0 text-xs font-medium"><Icon class={['size-3.5 shrink-0', tone]} /><span class="truncate">{value}</span>{#if props}<ChevronsUpDown class="size-3 text-muted-foreground" />{/if}</ItemActions>
			</svelte:element>
		{/snippet}
	</Item>
{/snippet}

<!-- Issue Board 행: 이슈와 하위 이슈가 같은 모양이라 재귀로 그린다 -->
{#snippet issueRow(i: Issue, depth: number)}
	{@const all = allTasks(i.num)}
	{@const done = all.filter((t) => t.status === 'done').length}
	{@const who = [...new Set(all.map((t) => t.agent).filter((a) => a !== undefined))].map((sn) => agentOf(sn)!)}
	{@const isOpen = expanded.has(i.num)}
	{@const icon = statuses[issueIcon[i.status]]}
	<TableRow class={depth === 0 ? 'h-13 bg-card' : 'h-11'}>
		<TableCell class={depth === 0 ? 'pl-4' : 'pl-10'}>
			<span class="flex items-center gap-3">
				<button type="button" aria-expanded={isOpen} aria-label="#{i.num} {isOpen ? '접기' : '펼치기'}" onclick={() => toggle(i.num)} class="rounded-xs text-muted-foreground hover:text-foreground">
					<ChevronRight class={['size-3.5 transition-transform', isOpen && 'rotate-90']} />
				</button>
				{#if depth === 0}
					<span class="kind-mark size-5 bg-node-issue"><CircleDot class="size-3" /></span>
				{:else}
					<span class="flex size-4.5 items-center justify-center rounded-xs border border-node-issue text-node-issue"><GitBranch class="size-3" /></span>
				{/if}
				<icon.icon class={['size-3.5', icon.text]} aria-label={icon.label} />
				<span class="font-mono text-xs font-medium text-muted-foreground">#{i.num}</span>
			</span>
		</TableCell>
		<TableCell class={depth === 0 ? 'font-semibold' : 'font-medium'}>
			<button type="button" aria-pressed={issueSel === i.num} onclick={() => openIssue(i.num)} class="text-left outline-none hover:underline focus-visible:underline">{i.title}</button>
		</TableCell>
		<TableCell>
			<span class="flex items-center gap-2">
				<Progress value={all.length ? (done / all.length) * 100 : 0} class="h-1.5" aria-label="#{i.num} 진행" tip={`#${i.num} 진행\n태스크 ${done} / ${all.length} 완료`} />
				<span class="font-mono text-xs text-muted-foreground">{done}/{all.length}</span>
			</span>
		</TableCell>
		<TableCell>
			<span class="flex items-center gap-2">
				<AvatarGroup>
					{#each who as a (a.sn)}<RoleAvatar role={a.role} size="sm" />{/each}
				</AvatarGroup>
				<span class="truncate text-xs text-muted-foreground">{who.map((a) => a.name).join(' · ') || 'Unassigned'}</span>
			</span>
		</TableCell>
		<TableCell class="pr-4 text-right font-mono text-xs text-subtle-foreground">{i.updated}</TableCell>
	</TableRow>
	{#if isOpen}
		{#each subIssues(i.num) as sub (sub.num)}
			{@render issueRow(sub, depth + 1)}
		{/each}
		{#each tasksOf(i.num) as t (t.num)}
			{@const a = agentOf(t.agent)}
			<TableRow
				onclick={() => open(t.num)}
				class={['h-10 cursor-pointer', selected === t.num && 'bg-primary-soft hover:bg-primary-soft']}
			>
				<TableCell class={depth === 0 ? 'pl-14.5' : 'pl-21'}>
					<span class="flex items-center gap-3">
						<CornerDownRight class="size-3 text-subtle-foreground" />
						<StatusSelect bind:value={t.status} compact />
						<span class="font-mono text-xs text-muted-foreground">#{t.num}</span>
					</span>
				</TableCell>
				<TableCell>
					<button type="button" aria-pressed={selected === t.num} onclick={() => open(t.num)} class="text-left outline-none hover:underline focus-visible:underline">{t.title}</button>
				</TableCell>
				<TableCell>
					<span class="flex items-center gap-2">
						{#if t.steps[1]}
							<Progress value={(t.steps[0] / t.steps[1]) * 100} class="h-1.5" aria-label="#{t.num} 진행" tip={`#${t.num} 완료 조건\n${t.steps[0]} / ${t.steps[1]} 완료`} />
							<span class="font-mono text-xs text-muted-foreground">{t.steps[0]}/{t.steps[1]}</span>
						{:else}
							<span class="font-mono text-xs text-subtle-foreground">no steps</span>
						{/if}
					</span>
				</TableCell>
				<TableCell>
					{#if a}
						<span class="flex items-center gap-1.5 text-xs">
							<RoleAvatar role={a.role} size="sm" />
							<span class="font-medium">{a.name}</span>
							<span class="text-muted-foreground">{roles[a.role].label}</span>
							{#if t.model}<Badge variant="mono" class="text-2xs">{t.model}</Badge>{/if}
						</span>
					{:else}
						<span class="text-xs text-muted-foreground">Unassigned</span>
					{/if}
				</TableCell>
				<TableCell class="pr-4 text-right font-mono text-xs text-subtle-foreground">{t.updated}</TableCell>
			</TableRow>
		{/each}
	{/if}
{/snippet}

{#if project}
	<div class="flex h-full min-h-0">
		<!-- 좌측 Quick Panel (.pen Workbench/LeftQuickPanel) -->
		{#if leftOpen}
			<aside class="flex shrink-0 flex-col w-66 border-r bg-card" aria-label="Quick panel">
				<Tabs bind:value={panelTab} class="flex min-h-0 flex-1 flex-col gap-0">
					<div class="panel-section gap-2.5">
						<div class="flex items-center gap-0.5">
							<TabsList>
								<TabsTrigger value="tasks">Tasks</TabsTrigger>
								<!-- 끄는 중 Agents 탭 위에 오면 탭을 연다 -->
								<TabsTrigger value="agents" ondragenter={() => drag.task && (panelTab = 'agents')}>Agents</TabsTrigger>
							</TabsList>
							<span class="flex-1"></span>
							{#if panelTab === 'agents'}
								<!-- Menu / Add (.pen yFujN) -->
								<DropdownMenu>
									<DropdownMenuTrigger>{#snippet child({ props })}<Button {...props} variant="ghost" size="icon-sm" aria-label="추가"><Plus /></Button>{/snippet}</DropdownMenuTrigger>
									<DropdownMenuContent align="end" class="w-56">
										<DropdownMenuLabel>추가</DropdownMenuLabel>
										<DropdownMenuEntries entries={addMenu} />
									</DropdownMenuContent>
								</DropdownMenu>
							{:else}
								<Button variant="ghost" size="icon-sm" aria-label="새 태스크" aria-expanded={quick !== undefined} onclick={() => (quick ? (quick = undefined) : openQuick())}><Plus /></Button>
							{/if}
							<Button variant="ghost" size="icon-sm" aria-label="Quick panel 접기" onclick={() => (leftOpen = false)}><PanelLeftClose /></Button>
						</div>
						{#if panelTab === 'tasks'}
							<InputGroup class="h-8">
								<InputGroupAddon><Search /></InputGroupAddon>
								<InputGroupInput bind:value={query} placeholder="Filter tasks" aria-label="태스크 검색" />
							</InputGroup>
							<div class="flex flex-wrap gap-1" role="group" aria-label="상태 필터">
								<Toggle variant="chip" count={list.length} pressed={filter === 'all'} onPressedChange={() => (filter = 'all')}>All</Toggle>
								{#each statusOrder as s (s)}
									<Toggle variant="chip" count={count(s)} pressed={filter === s} onPressedChange={() => (filter = s)}>{statuses[s].label}</Toggle>
								{/each}
							</div>
						{/if}
					</div>

					{#if quick && panelTab === 'tasks'}
						{@const q = quick}
						{@const qa = agentOf(q.agent)}
						<!-- QuickAdd (.pen QuickAdd · open) — Enter 만들기 · Esc 닫기 · ⤢ 편집기 -->
						<form onsubmit={quickSubmit} class="panel-section gap-2" aria-label="빠른 태스크 추가">
							<InputGroup class="h-8 border-ring">
								<InputGroupAddon><Plus /></InputGroupAddon>
								<!-- svelte-ignore a11y_autofocus -->
								<InputGroupInput autofocus bind:value={q.title} placeholder="태스크 제목" aria-label="태스크 제목" onkeydown={(e) => e.key === 'Escape' && (quick = undefined)} />
								<InputGroupAddon align="inline-end">
									<InputGroupButton size="icon-xs" aria-label="편집기로 열기" title="편집기로 열기" onclick={() => newTask({ title: q.title, status: q.status, agent: q.agent, priority: q.priority })}><Maximize2 /></InputGroupButton>
								</InputGroupAddon>
							</InputGroup>
							<div class="flex items-center gap-1">
								<StatusSelect bind:value={q.status}>
									{#snippet trigger(props)}{@const m = statuses[q.status]}{@render propChip(props, m.icon, m.label)}{/snippet}
								</StatusSelect>
								<AssigneePicker bind:value={q.agent} agents={agentList} tasks={list} recommend={recommendFor(agentList, q.title, [])}>
									{#snippet trigger(props)}{@render propChip(props, Bot, qa?.name ?? 'Assignee')}{/snippet}
								</AssigneePicker>
								<PriorityPicker bind:value={q.priority}>
									{#snippet trigger(props)}{@render propChip(props, Flag, q.priority)}{/snippet}
								</PriorityPicker>
								<span class="flex-1"></span>
								<Kbd>↵</Kbd>
							</div>
						</form>
					{/if}

					<TabsContent value="tasks" class="min-h-0 flex-1 overflow-y-auto">
						{#each shown as t (t.num)}
							<Item
								variant="row"
								size="sm"
								aria-pressed={selected === t.num}
								onclick={() => open(t.num)}
								class={['w-full px-3 text-left', selected === t.num ? 'bg-primary-soft hover:bg-primary-soft' : 'hover:bg-muted', drag.task?.num === t.num && 'bg-primary-soft opacity-40', drag.task && !movable(t) && 'opacity-50']}
							>
								{#snippet child({ props })}
									<!-- Backlog · Todo만 끌 수 있다 (.pen 드래그로 배정 ①) -->
									<button type="button" {...props} draggable={movable(t)} ondragstart={(e) => startDrag(e, t)} ondragend={endDrag}>
										<ItemContent class="gap-1.5">
											<span class="flex items-center justify-between">
												<span class="font-mono text-xs font-medium text-muted-foreground">#{t.num}</span>
												<StatusBadge status={t.status} />
											</span>
											<ItemTitle class="text-sm">{t.title}</ItemTitle>
											<ItemDescription class="flex items-center gap-1.5">
												{agentName(t.agent) ?? '미배정'}
												<span class={['font-mono font-semibold', t.priority === 'P0' ? 'text-destructive' : 'text-subtle-foreground']}>{t.priority}</span>
											</ItemDescription>
										</ItemContent>
									</button>
								{/snippet}
							</Item>
						{:else}
							<p class="p-6 text-center text-xs text-muted-foreground">조건에 맞는 태스크가 없어요.</p>
						{/each}
					</TabsContent>

					<TabsContent value="agents" class="min-h-0 flex-1 overflow-y-auto">
						{#each agentList as a (a.sn)}
							<!-- 우클릭 · ⋯ → Menu / Agent (.pen znwIq, Diagram 에이전트 노드와 같은 목록) -->
							<ContextMenu>
								<ContextMenuTrigger>
									{#snippet child({ props })}
										<div {...props} class="group relative">
											<button
												type="button"
												aria-pressed={inspect === a.sn}
												onclick={() => openAgent(a.sn)}
												ondragover={(e) => drag.task && (e.preventDefault(), (dropAgent = a.sn))}
												ondragleave={(e) => !e.currentTarget.contains(e.relatedTarget as globalThis.Node | null) && dropAgent === a.sn && (dropAgent = undefined)}
												ondrop={(e) => (e.preventDefault(), dropOnAgent(a.sn))}
												class={['flex w-full items-center gap-2.5 border-b px-3 py-2.5 text-left outline-none focus-visible:bg-muted', inspect === a.sn || dropAgent === a.sn ? 'bg-primary-soft hover:bg-primary-soft' : 'hover:bg-muted', dropAgent === a.sn && 'ring-2 ring-primary ring-inset']}
											>
												<RoleAvatar role={a.role}>
													<AvatarBadge class={a.online ? 'bg-success' : 'bg-subtle-foreground'} aria-label={a.online ? '온라인' : '오프라인'} />
												</RoleAvatar>
												<span class="flex min-w-0 flex-1 flex-col gap-0.5">
													<span class="flex items-center gap-1.5">
														<span class="text-sm font-medium">{a.name}</span>
														<span class="font-mono text-xs text-muted-foreground">{a.runtime === 'claude' ? 'Claude Code' : 'Codex CLI'}</span>
													</span>
													<span class={['truncate text-xs', dropAgent === a.sn ? 'font-medium text-primary' : 'text-muted-foreground']}>{dropAgent === a.sn ? `놓으면 ${a.name}에게 배정` : a.activity}</span>
												</span>
												<span class="font-mono text-xs font-medium text-muted-foreground">{a.tokens}</span>
											</button>
											<DropdownMenu>
												<DropdownMenuTrigger>
													{#snippet child({ props })}<Button {...props} variant="ghost" size="icon-xs" class="absolute top-1/2 right-2 -translate-y-1/2 bg-card opacity-0 group-hover:opacity-100 focus-visible:opacity-100 aria-expanded:opacity-100" aria-label="{a.name} 메뉴"><Ellipsis /></Button>{/snippet}
												</DropdownMenuTrigger>
												<DropdownMenuContent align="end" class="w-56">
													<DropdownMenuLabel class="truncate">{a.name} · {roles[a.role].label}</DropdownMenuLabel>
													<DropdownMenuSeparator />
													<DropdownMenuEntries entries={agentMenu(a.sn)} />
												</DropdownMenuContent>
											</DropdownMenu>
										</div>
									{/snippet}
								</ContextMenuTrigger>
								<ContextMenuContent class="w-56">
									<ContextMenuLabel class="truncate">{a.name} · {roles[a.role].label}</ContextMenuLabel>
									<ContextMenuSeparator />
									<ContextMenuEntries entries={agentMenu(a.sn)} />
								</ContextMenuContent>
							</ContextMenu>
						{/each}
					</TabsContent>
				</Tabs>
			</aside>
		{/if}

		<main class="flex min-w-0 flex-1 flex-col">
		<SvelteFlowProvider>
			<!-- 뷰 머리글 (.pen Workbench/ViewHeader) -->
			<header class="flex h-12 shrink-0 items-center gap-3 border-b bg-background px-4">
				{#if !leftOpen}
					<Button variant="ghost" size="icon-sm" aria-label="Quick panel 펼치기" onclick={() => (leftOpen = true)}><PanelLeftOpen /></Button>
				{/if}
				<Tabs value={view.value} onValueChange={setView}>
					<TabsList aria-label="보기">
						{#each views as v (v.value)}
							<TabsTrigger value={v.value}><v.icon />{v.label}</TabsTrigger>
						{/each}
					</TabsList>
				</Tabs>
				<p class="min-w-0 flex-1 truncate text-body text-muted-foreground">
					{project.name} · {list.length} tasks · {agentList.length} agents
				</p>
				{#if !useMock && wb.state !== 'live'}
					<Badge variant={wb.state === 'error' ? 'destructive' : 'secondary'} class="gap-1" role="status" data-testid="stream-state">
						{#if wb.state === 'error'}연결 실패{:else}<LoaderCircle class="size-3 animate-spin" />{wb.state === 'reconnecting' ? '재연결 중' : '불러오는 중'}{/if}
					</Badge>
					{#if wb.state === 'error'}
						<Button variant="outline" size="xs" onclick={() => (closeProject(), void openProject(Number(page.params.project)))}>다시 시도</Button>
					{/if}
				{/if}
				{#if view.value === 'diagram'}
					<DiagramToolbar bind:this={toolbar} {grid} onauto={autoLayout} ongrid={gridLayout} />
				{/if}
				{#if !dockOpen}
					<Button variant="ghost" size="icon-sm" aria-label="PM Dock 펼치기" onclick={() => (dockOpen = true)}><PanelRightOpen /></Button>
				{/if}
			</header>

			<!-- 뷰 (#54 Kanban · #55 Issues · #56 Diagram) -->
			<div class="relative min-h-0 flex-1">
			<div class="h-full overflow-auto bg-canvas">
				{#if view.value === 'kanban'}
					<!-- orch/kanban 조립: 열 = 상태, 항목 = 태스크 번호. 놓으면 drop()이 상태를 command로 반영 -->
					<KanbanBoard bind:value={board} onDragEnd={drop}>
						{#each lanes as s (s)}
							{@const meta = statuses[s]}
							<!-- 왼쪽 목록에서 끌어 오면 Backlog · Todo 열만 받는다 (.pen 드래그로 배정 ④) -->
							<KanbanColumn
								value={s}
								class={[drag.task && !movable({ status: s }) && 'cursor-not-allowed opacity-50', dropLane === s && 'bg-primary-soft ring-2 ring-primary']}
								ondragover={(e) => drag.task && movable({ status: s }) && (e.preventDefault(), (dropLane = s))}
								ondragleave={(e) => !e.currentTarget.contains(e.relatedTarget as globalThis.Node | null) && dropLane === s && (dropLane = undefined)}
								ondrop={(e) => (e.preventDefault(), dropOnLane(s))}
							>
								<KanbanColumnHeader>
									<meta.icon class={meta.text} />
									<KanbanColumnTitle>{meta.label}</KanbanColumnTitle>
									<KanbanColumnCount />
									<KanbanColumnActions>
										<Button variant="ghost" size="icon-xs" aria-label="{meta.label}에 태스크 추가" onclick={() => openQuick(s)}><Plus /></Button>
																<!-- 열 메뉴 — .pen에 항목이 없어 있는 동작만 (#60) -->
																<DropdownMenu>
																	<DropdownMenuTrigger>{#snippet child({ props })}<Button {...props} variant="ghost" size="icon-xs" aria-label="{meta.label} 열 메뉴"><Ellipsis /></Button>{/snippet}</DropdownMenuTrigger>
																	<DropdownMenuContent align="end" class="w-48">
																		<DropdownMenuItem onSelect={() => newTask({ status: s })}><SquarePen />{meta.label}로 새 태스크</DropdownMenuItem>
																		<DropdownMenuItem onSelect={() => ((leftOpen = true), (panelTab = 'tasks'), (filter = s))}><FilterIcon />Quick Panel에서 이 상태만</DropdownMenuItem>
																	</DropdownMenuContent>
																</DropdownMenu>
									</KanbanColumnActions>
								</KanbanColumnHeader>
								<KanbanColumnContent>
									{#each board[s] ?? [] as num (num)}
										{@const t = task(num as number)}
										{@const a = agentOf(t.agent)}
										<!-- 우클릭: ContextMenu / Task (Diagram과 같은 목록) -->
										<ContextMenu onOpenChange={(o) => o && hoverOff()}>
											<ContextMenuTrigger>
												{#snippet child({ props })}
													<KanbanCard
														{...props}
														value={num}
														aria-pressed={selected === t.num}
														onclick={() => (hoverOff(), open(t.num))}
														onpointermove={(e) => hoverAt(t.num, e)}
														onpointerleave={hoverOff}
														onpointerdown={hoverOff}
													>
														<KanbanCardHeader>
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
															<Badge class={['rounded-xs', ['P0', 'P1'].includes(t.priority) ? 'bg-destructive-soft text-destructive' : 'bg-muted text-muted-foreground']}><SignalHigh />{t.priority}</Badge>

														</KanbanCardHeader>
														<KanbanCardContent>
															<span class="title-sm gap-2">
																<SquareCheck class="size-3.5 shrink-0 text-node-task" />#{t.num} · {t.title}
															</span>
															<span class="meta-xs gap-2">
																<CircleDot class="size-3.5 shrink-0 text-node-issue" />Issue #{t.issue} · {issueOf(t.issue)?.title}
															</span>
															<span class="flex items-center gap-2 pt-1.5">
																{#if t.steps[1]}
																	<Progress value={(t.steps[0] / t.steps[1]) * 100} class="h-1.5" aria-label="완료 조건 진행" tip={`완료 조건\n${t.steps[0]} / ${t.steps[1]} 완료`} />
																	<span class="shrink-0 font-mono text-xs font-semibold whitespace-nowrap text-muted-foreground">
																		{t.steps[0]}/{t.steps[1]} · {Math.round((t.steps[0] / t.steps[1]) * 100)}%
																	</span>
																{:else}
																	<span class="font-mono text-xs text-subtle-foreground">no steps</span>
																{/if}
															</span>

														</KanbanCardContent>
														<KanbanCardFooter>
															<span class={['inline-flex items-center gap-1', t.over && 'text-warning']}><Coins class="size-3" />{t.tokens ?? '—'}{t.over ? ' ⚠' : ''}</span>
															<span class="inline-flex items-center gap-1"><MessageSquare class="size-3" />{t.messages}</span>
															{#if t.model}<Badge variant="mono" class="text-2xs"><Cpu />{t.model}</Badge>{/if}
															<span class="ml-auto inline-flex items-center gap-1"><Timer class="size-3" />{t.run ? `Run ${t.run}` : '—'}</span>

														</KanbanCardFooter>
													</KanbanCard>
												{/snippet}
											</ContextMenuTrigger>
											<ContextMenuContent class="w-56">
												<ContextMenuLabel class="truncate">Task #{t.num} · {t.title}</ContextMenuLabel>
												<ContextMenuSeparator />
												<ContextMenuEntries entries={taskMenu(t.num)} />
											</ContextMenuContent>
										</ContextMenu>
									{/each}
									{#snippet empty()}<p class="empty-note text-center text-subtle-foreground">비어 있어요</p>{/snippet}
								</KanbanColumnContent>
							</KanbanColumn>
						{/each}
					</KanbanBoard>
					{#if hover}
						{@const t = task(hover.num)}
						{@const d = details[hover.num]}
						{@const next = d?.criteria.find((c) => !c.done)}
						{@const blocks = (d?.deps ?? []).filter((x) => x.kind === 'blocks').map((x) => task(x.num)).filter(Boolean)}
						{@const last = d?.activity.at(-1)}
						{@const ctx = d?.context}
						{@const at = { x: hover.x, y: hover.y }}
						<!-- .pen KanbanCard/HoverPreview — ui/hover-card를 커서 위치에 띄운다 (+18px, 화면 끝에선 floating-ui가 뒤집음) -->
						<HoverCard open onOpenChange={(o) => !o && hoverOff()}>
							<HoverCardContent
								customAnchor={{ getBoundingClientRect: () => new DOMRect(at.x + 18, at.y, 0, 0) }}
								side="bottom"
								align="start"
								sideOffset={18}
								class="pointer-events-none w-75 gap-0 overflow-hidden p-0"
							>
								<div class="flex items-start gap-2 border-b px-3 py-2.5">
									<span class="pt-0.5 text-xs font-semibold text-muted-foreground">#{t.num}</span>
									<span class="min-w-0 flex-1 text-body font-semibold">{t.title}</span>
									<StatusBadge status={t.status} />
								</div>
								<dl class="flex flex-col gap-2 px-3 py-2.5 text-caption">
									{#each [['현재 단계', t.steps[1] ? `${t.steps[0]}/${t.steps[1]}${next ? ` · ${next.text}` : d?.criteria.length ? ' · 모두 완료' : ''}` : '—'], ['최근 활동', last ? `${last.type.toLowerCase()} ${last.text}` : '—'], ['막고 있는 Task', blocks.length ? blocks.map((b) => `#${b.num} ${b.title} · ${agentName(b.agent) ?? '미배정'}`).join(', ') : '—'], ['완료 시 전달', blocks[0] ? `${agentName(blocks[0].agent) ?? '미배정'} · ${roles[agentOf(blocks[0].agent)?.role ?? 'agent'].label} (REQUEST_VERIFICATION)` : '—'], ['ETA', d?.eta || '—']] as [k, v] (k)}
										<div class="flex gap-2"><dt class="shrink-0 text-muted-foreground">{k}</dt><dd class="min-w-0 flex-1 truncate text-right font-medium">{v}</dd></div>
									{/each}
									{#if ctx}
										<div class="flex flex-col gap-1.5 pt-1.5">
											<div class="flex text-xs font-medium"><span class="flex-1 text-muted-foreground">Context</span><span class={ctx[0] / ctx[1] > 0.9 ? 'text-warning' : ''}>{ctx[0]}K / {ctx[1]}K</span></div>
											<Progress value={(ctx[0] / ctx[1]) * 100} class="h-2" aria-label="컨텍스트" tip={`Run 컨텍스트\n${ctx[0]} / ${ctx[1]} · ${Math.round((ctx[0] / ctx[1]) * 100)}% · 90% 넘으면 경고`} />
										</div>
										{#if ctx[0] / ctx[1] > 0.9}<p class="font-medium text-warning">⚠ Context {Math.round((ctx[0] / ctx[1]) * 100)}% — 요약 또는 새 Session 권장</p>{/if}
									{/if}
								</dl>
								<p class="bg-muted px-3 py-2 text-caption text-muted-foreground">클릭 → 상세 보기 · 우클릭 → 메뉴</p>
							</HoverCardContent>
						</HoverCard>
					{/if}
				{:else if view.value === 'diagram'}
					<SvelteFlow
						bind:nodes
						bind:edges
						{nodeTypes}
						initialViewport={{ x: 24, y: 24, zoom: 0.85 }}
						minZoom={0.3}
						nodesConnectable={false}
						snapGrid={grid ? [20, 20] : undefined}
						onpaneclick={() => (linking = undefined)}
						onpanecontextmenu={({ event }) => {
							event.preventDefault();
							linking = undefined;
							paneMenu = { x: event.clientX, y: event.clientY };
						}}
						onnodeclick={({ node }) => {
							if (linking) return finishLink(node.id);
							if (node.id.startsWith('task-')) open(Number(node.id.slice(5)));
							else if (node.id.startsWith('issue-')) openIssue(Number(node.id.slice(6)));
							else if (node.id.startsWith('agent-')) openAgent(Number(node.id.slice(6)));
							else if (node.id.startsWith('sub-')) openSub(node.id.slice(4));
						}}
						class="bg-canvas"
					>
						<!-- canvas-grid 토큰은 canvas 배경과 거의 같아 점이 안 보인다 → 한 단계 진한 input 색 -->
						<Background variant={BackgroundVariant.Dots} gap={20} size={1.5} patternColor="var(--input)" />
						{#if linking}
							<Panel position="top-center">
								<p class="flex items-center gap-2 rounded-full bg-foreground px-3 py-1.5 text-xs font-medium text-background shadow-md" role="status">
									<Spline class="size-3.5" />연결 모드 · #{linking.from}에서 {linkKinds[linking.kind]} → 대상 노드 클릭 · Esc 취소
								</p>
							</Panel>
						{/if}
						<Panel position="bottom-left">
							<div class="card flex items-center gap-3 px-2.5 py-1.5 text-xs text-muted-foreground rounded-md" aria-label="연결선 범례">
								{#each [['contains', 'bg-input', 'h-0.5'], ['delegate', 'bg-primary', 'h-0.5'], ['assigned', 'bg-node-agent', 'h-0.5'], ['interaction (idle)', 'bg-status-review', 'h-0.5'], ['live event', 'bg-primary', 'h-1'], ['spawn', 'bg-node-agent', 'h-0.5'], ['queued · waits', 'bg-subtle-foreground', 'h-0.5']] as [l, bg, h] (l)}
									<span class="flex items-center gap-1.5"><span class={['w-3.5 rounded-full', bg, h]}></span>{l}</span>
								{/each}
							</div>
						</Panel>
					</SvelteFlow>
					{#if paneMenu}
						{@const at = paneMenu}
						<!-- .pen ContextMenu / Canvas — 빈 캔버스 우클릭 자리에 연다 -->
						<DropdownMenu open onOpenChange={(o) => !o && (paneMenu = undefined)}>
							<DropdownMenuContent customAnchor={{ getBoundingClientRect: () => new DOMRect(at.x, at.y, 0, 0) }} side="bottom" align="start" sideOffset={2} class="w-52">
								<DropdownMenuLabel>Canvas</DropdownMenuLabel>
								<DropdownMenuSeparator />
								<DropdownMenuEntries entries={canvasMenu} />
							</DropdownMenuContent>
						</DropdownMenu>
					{/if}
				{:else if view.value === 'issues'}
					<Table class="bg-background">
						<TableHeader class="bg-muted">
							<TableRow>
								<TableHead class="w-40 pl-4">Issue</TableHead>
								<TableHead>Title</TableHead>
								<TableHead class="w-44">Progress</TableHead>
								<TableHead class="w-75">Agents</TableHead>
								<TableHead class="w-20 pr-4 text-right">Updated</TableHead>
							</TableRow>
						</TableHeader>
						<TableBody>
							{#each issueList.filter((i) => !i.parent) as i (i.num)}
								{@render issueRow(i, 0)}
							{/each}
						</TableBody>
					</Table>
				{:else}
				<Empty class="h-full">
					<EmptyHeader>
						<EmptyMedia variant="icon"><view.icon /></EmptyMedia>
						<EmptyTitle>{view.label}</EmptyTitle>
						<EmptyDescription>{view.issue}에서 구현해요.</EmptyDescription>
					</EmptyHeader>
				</Empty>
				{/if}
			</div>
			{#if cur}
				{@const a = agentOf(cur.agent)}
				{@const pm = priorities[cur.priority]}
				{@const deps = info?.deps ?? []}
				{@const iss = issueOf(cur.issue)}
				<!-- 태스크 상세: 뷰 위 scrim + 패널. Esc · 닫기 버튼으로 닫는다 -->
				<div class="scrim">
					<Inspector label="Task #{cur.num} 상세">
						<InspectorHeader onclose={() => (detail = false)} closeLabel="상세 닫기">
								<span class="kind-tile size-8 bg-node-task"><SquareCheck class="size-4" /></span>
								<div class="flex min-w-0 flex-1 flex-col gap-0.5">
									<p class="truncate text-xs text-muted-foreground">{project.name} / Issue #{cur.issue} {iss?.title} / <span class="font-mono">TASK #{cur.num}</span></p>
									<h2 class="text-lg font-semibold">{cur.title}</h2>
								</div>
								<Button variant="ghost" size="icon-sm" aria-label="편집 (E)" title="편집 (E)" onclick={() => editTask(cur.num)}><Pencil /></Button>
								<Button variant="ghost" size="sm" onclick={() => navigator.clipboard?.writeText(`${page.url.origin}${page.url.pathname}?task=${cur.num}`)}><Link2 />Copy link</Button>
							{#snippet sub()}
							<div class="flex flex-wrap items-center gap-2 pl-11">
								<StatusSelect bind:value={() => cur.status, (v) => void setStatus(cur.num, v)} />
								<Badge variant="outline">{cur.priority}</Badge>
								{#if a}
									<span class="flex items-center gap-1.5 text-xs">
										<RoleAvatar role={a.role} size="sm" />
										<span class="font-medium">{a.name}</span>
										<span class="text-muted-foreground">{roles[a.role].label}</span>
										{#if cur.model}<Badge variant="mono" class="text-2xs">{a.runtime === 'claude' ? 'Claude Code' : 'Codex CLI'} · {cur.model}</Badge>{/if}
									</span>
								{/if}
								{#if cur.run}<span class="text-xs text-muted-foreground">Run {cur.run}</span>{:else if liveRun}<span class="text-xs text-muted-foreground">Run #{liveRun.num}</span>{/if}
								<span class="flex-1"></span>
								<Button variant="ghost" size="icon-sm" class="bg-warning-soft text-warning" aria-label="일시정지"><Pause /></Button>
								<!-- 중지 = POST /runs/{sn}/stop · 진행 중 Run이 있을 때만 -->
								<Button variant="ghost" size="icon-sm" class="bg-destructive-soft text-destructive" aria-label="중지" disabled={!useMock && !liveRun} onclick={() => liveRun && stopRun(liveRun.sn).then((e) => e && toast.warning(e))}><Square /></Button>
								<Button variant="ghost" size="sm" onclick={() => askPm(`Task #${cur.num}`)}><MessageCircleQuestion />Ask PM about this</Button>
							</div>
							{#if info?.decision}
								<Alert variant="warning" class="flex items-center gap-2.5">
									<MessageCircleQuestion />
									<div class="flex-1">
										<AlertTitle>{info.decision.question}</AlertTitle>
										<AlertDescription>{info.decision.note}</AlertDescription>
									</div>
									<Badge class="rounded-xs bg-status-waiting font-mono text-2xs text-on-solid">{info.decision.level}</Badge>
									<Button size="sm" onclick={() => openDecisions(cur.num)}>답변하기</Button>
								</Alert>
							{/if}
							{/snippet}
						</InspectorHeader>
						<InspectorBody class="flex-row">
							<div class="detail-main">
								<section class="flex flex-col gap-2">
									<h3 class="text-body font-semibold">Description</h3>
									<p class="text-body leading-relaxed">{info?.description ?? cur.description ?? '설명이 없어요.'}</p>
								</section>
								<section class="flex flex-col gap-1">
									<h3 class="text-body font-semibold">Acceptance criteria</h3>
									<!-- .pen B · 인라인: 체크 · 끌기 · hover 편집/삭제 · 조건 추가 -->
									<Checklist label="완료 조건" placeholder="조건 추가" bind:items={() => info?.criteria ?? [], (v) => (detailOf(cur.num).criteria = v)} />
								</section>
								{#if info?.deps.length}
									<section class="flex flex-col gap-2">
										<h3 class="text-body font-semibold">Dependencies</h3>
										{#each info.deps as d (d.num)}
											{@const dt = task(d.num)}
											<div class="flex items-center gap-2 text-xs">
												<span class="w-18 text-muted-foreground">{d.kind === 'depends' ? 'depends on' : 'blocks'}</span>
												<button type="button" class="rounded-sm bg-muted px-2 py-0.5 font-medium hover:bg-muted-strong" onclick={() => open(d.num)}>#{d.num} {dt.title} · {agentOf(dt.agent)?.name ?? '미배정'}</button>
												<StatusBadge status={dt.status} />
											</div>
										{/each}
									</section>
								{/if}
								{#if runs.length}
									<section class="flex flex-col">
										<h3 class="mb-1 text-body font-semibold">Runs</h3>
										{#each runs as r (r.num)}
											<Item variant="row" size="xs">
												<ItemContent>
													<ItemTitle>Run #{r.num} {#if r.live}<span class="size-1.5 rounded-full bg-success" aria-label="진행 중"></span>{/if}</ItemTitle>
													<ItemDescription>{r.note}</ItemDescription>
												</ItemContent>
												<ItemActions class="font-mono text-xs text-muted-foreground">{r.time} · {r.tokens}</ItemActions>
											</Item>
										{/each}
									</section>
								{/if}
								{#if leadRuns[cur.num]}
									{@const subs = subRuns.filter((s) => s.task === cur.num)}
									<!-- Run 토큰 (.pen Run 상세 · 토큰) — Run마다 따로 쌓고 여기서만 합친다. runner는 리드와 따로, sub · fork는 리드 사용량에 포함 -->
									<section class="flex flex-col">
										<h3 class="mb-1 text-body font-semibold">Run 토큰</h3>
										<Item variant="row" size="xs">
											<ItemContent><ItemTitle><Bot class="size-3.5 text-muted-foreground" />리드 Run #{leadRuns[cur.num].run} · {agentName(a?.sn)}</ItemTitle></ItemContent>
											<ItemActions><TokenMeter self={leadRuns[cur.num].self} runner={runnerTotal(cur.num)} /></ItemActions>
										</Item>
										{#each subs as s (s.id)}
											{#if s.mode === 'runner'}
												{#each s.runs as r, i (r.num)}
													<Item variant="row" size="xs">
														<ItemContent><ItemTitle><Cpu class="size-3.5 text-muted-foreground" />Run #{r.num} · {s.id} runner{i < s.runs.length - 1 ? ' · 실패(재시도 전)' : ''}</ItemTitle></ItemContent>
														<ItemActions><TokenMeter self={r.tokens} /></ItemActions>
													</Item>
												{:else}
													<Item variant="row" size="xs">
														<ItemContent><ItemTitle><Cpu class="size-3.5 text-muted-foreground" />{s.id} runner · 대기</ItemTitle></ItemContent>
														<ItemActions><TokenMeter self={0} /></ItemActions>
													</Item>
												{/each}
											{:else}
												{@const M = subRunMode[s.mode].icon}
												<Item variant="row" size="xs">
													<ItemContent><ItemTitle><M class="size-3.5 text-muted-foreground" />{s.id} {s.mode}</ItemTitle></ItemContent>
													<ItemActions><TokenMeter included /></ItemActions>
												</Item>
											{/if}
										{/each}
										<p class="pt-1.5 text-caption text-muted-foreground">runner는 리드와 따로 보이고 합계에만 더해요. sub · fork는 리드 Run 안에서 돌아 리드 사용량에 이미 들어 있어요.</p>
									</section>
								{/if}
								{#if info?.activity.length}
									<section class="flex flex-col">
										<h3 class="mb-1 text-body font-semibold">Activity</h3>
										{#each info.activity as ev, i (i)}
											{#if ev.record}
												<!-- 결정 기록 (.pen Decision Record) — 한 줄, 누르면 카드 -->
												{@const r = ev.record}
												<div class="py-1"><DecisionRecord record={r} onreview={() => openDecisions(r.task)} /></div>
											{:else}
												<Item variant="row" size="xs">
													<ItemContent>
														<ItemTitle><Badge variant="mono" class="text-2xs">{ev.type}</Badge><span class="text-xs font-normal text-muted-foreground">{ev.who}</span></ItemTitle>
														<ItemDescription>{ev.text}</ItemDescription>
													</ItemContent>
													<ItemActions class="font-mono text-xs text-subtle-foreground">{ev.time}</ItemActions>
												</Item>
											{/if}
										{/each}
									</section>
								{/if}
							</div>
							<aside class="detail-aside" aria-label="속성">
								<section class="flex flex-col">
									<h3 class="mb-1 text-body font-semibold">Properties</h3>
									<!-- .pen B · Properties hover / click: 담당 · 우선순위 · 의존은 선택 창(C), 나머지는 표시 -->
									{@render propRow(undefined, 'Issue', CircleDotIcon, `#${cur.issue} ${iss?.title ?? ''}`, 'text-node-issue')}
									<AssigneePicker bind:value={() => cur.agent, (v) => (task(cur.num).agent = v)} agents={agentList} tasks={list} recommend={recommendFor(agentList, cur.title, info?.labels ?? [])} onorch={() => (task(cur.num).agent = orchPick(agentList, list, cur.title, info?.labels ?? []))}>
										{#snippet trigger(props)}{@render propRow(props, 'Assignee', a ? roles[a.role].icon : UserRoundX, a ? `${a.name} · ${roles[a.role].label}` : 'Unassigned', a ? roles[a.role].text : undefined)}{/snippet}
									</AssigneePicker>
									<PriorityPicker bind:value={() => cur.priority, (v) => (task(cur.num).priority = v)}>
										{#snippet trigger(props)}{@render propRow(props, 'Priority', pm.icon, cur.priority, pm.text)}{/snippet}
									</PriorityPicker>
									<DependsPicker bind:value={() => deps, (v) => (detailOf(cur.num).deps = v)} current={cur.num} tasks={list} depsOf={(n) => details[n]?.deps ?? []}>
										{#snippet trigger(props)}
											{@const on = deps.filter((x) => x.kind === 'depends')}
											{@render propRow(props, 'Depends on', Link2Icon, on.length ? on.map((x) => `#${x.num} ${task(x.num)?.title ?? ''}`).join(', ') : '—')}
										{/snippet}
									</DependsPicker>
									{@render propRow(undefined, 'ETA', TimerIcon, info?.eta || '—')}
									{@render propRow(undefined, 'Labels', Tag, info?.labels.join(' · ') || '—')}
								</section>
								<section class="flex flex-col gap-2">
									<div class="flex items-center gap-2">
										<h3 class="text-body font-semibold">하위 작업 방식</h3>
										<Pill class="ml-auto">{cur.spawnMode ? '이 태스크만' : '팀 기본값 사용'}</Pill>
									</div>
									<Segmented aria-label="하위 작업 방식" options={spawnOptions} bind:value={() => cur.spawnMode ?? 'team', (v) => (task(cur.num).spawnMode = v === 'team' ? undefined : (v as SpawnMode))} />
									{#if (cur.spawnMode ?? spawnPolicy.mode) === 'fork'}
										<Alert variant="warning">
											<GitFork />
											<AlertTitle>fork는 부모 컨텍스트를 상속해요</AlertTitle>
											<AlertDescription>부모 컨텍스트를 그대로 복사해 비용이 커요.</AlertDescription>
										</Alert>
									{/if}
									<p class="text-caption text-muted-foreground">바꾸면 이 태스크에만 저장돼요. 비워 두면 팀 기본값을 써요 (Teams › Orch 진행 정책).</p>
								</section>
								{#if info?.context}
									<section class="flex flex-col gap-2">
										<h3 class="text-body font-semibold">Tokens · this task</h3>
										<div class="flex items-center justify-between text-xs"><span class="text-muted-foreground">Run context</span><span class="font-mono">{info.context[0]}K / {info.context[1]}K</span></div>
										<Progress value={(info.context[0] / info.context[1]) * 100} class="h-1.5" aria-label="컨텍스트 사용" tip={`Run 컨텍스트\n${info.context[0]}K / ${info.context[1]}K · ${Math.round((info.context[0] / info.context[1]) * 100)}%`} />
									</section>
								{/if}
								{#if info?.git}
									<section class="flex flex-col">
										<h3 class="mb-1 text-body font-semibold">Git</h3>
										{#each [['Branch', info.git.branch], ['Commits', info.git.commits], ['PR', info.git.pr ?? '—']] as [k, v] (k)}
											<Item variant="row" size="xs">
												<ItemContent><ItemDescription>{k}</ItemDescription></ItemContent>
												<ItemActions class="font-mono text-xs">{v}</ItemActions>
											</Item>
										{/each}
									</section>
								{/if}
							</aside>
						</InspectorBody>
					</Inspector>
				</div>
			{/if}
			{#if curIssue}
				{@const iss = curIssue}
				{@const parent = iss.parent === undefined ? undefined : issueOf(iss.parent)}
				{@const all = allTasks(iss.num)}
				{@const done = all.filter((t) => t.status === 'done').length}
				{@const who = [...new Set(all.map((t) => t.agent).filter((a) => a !== undefined))].map((sn) => agentOf(sn)!)}
				{@const icon = statuses[issueIcon[iss.status]]}
				<!-- 이슈 상세: Task Detail과 같은 틀 (scrim + 패널). 하위 이슈 · 태스크를 눌러 이어서 연다 -->
				<div class="scrim">
					<Inspector label="Issue #{iss.num} 상세">
						<InspectorHeader onclose={() => (issueSel = undefined)} closeLabel="상세 닫기">
								<span class="kind-tile size-8 bg-node-issue"><CircleDot class="size-4" /></span>
								<div class="flex min-w-0 flex-1 flex-col gap-0.5">
									<p class="truncate text-xs text-muted-foreground">
										{project?.name}{' / '}{#if parent}<button type="button" class="hover:underline" onclick={() => openIssue(parent.num)}>Issue #{parent.num} {parent.title}</button>{' / '}{/if}<span class="font-mono">ISSUE #{iss.num}</span>
									</p>
									<h2 class="text-lg font-semibold">{iss.title}</h2>
								</div>
								<Button variant="ghost" size="sm" onclick={() => navigator.clipboard?.writeText(`${page.url.origin}${page.url.pathname}?issue=${iss.num}`)}><Link2 />Copy link</Button>
							{#snippet sub()}
							<div class="flex flex-wrap items-center gap-3 pl-11">
								<span class={['label-xs', icon.text]}><icon.icon class="size-3.5" />{issueLabel[iss.status]}</span>
								<span class="flex w-40 items-center gap-2">
									<Progress value={all.length ? (done / all.length) * 100 : 0} class="h-1.5" aria-label="#{iss.num} 진행" tip={`#${iss.num} 진행\n태스크 ${done} / ${all.length} 완료`} />
									<span class="font-mono text-xs text-muted-foreground">{done}/{all.length}</span>
								</span>
								{#if who.length}
									<span class="flex items-center gap-1.5 text-xs">
										<AvatarGroup>
											{#each who as a (a.sn)}<RoleAvatar role={a.role} size="sm" />{/each}
										</AvatarGroup>
										<span class="text-muted-foreground">{who.map((a) => a.name).join(' · ')}</span>
									</span>
								{/if}
								<span class="flex-1"></span>
								<Button variant="ghost" size="sm" onclick={() => askPm(`Issue #${iss.num}`)}><MessageCircleQuestion />Ask PM about this</Button>
							</div>
							{/snippet}
						</InspectorHeader>
						<InspectorBody class="flex-row">
							<div class="detail-main">
								<section class="flex flex-col gap-2">
									<h3 class="text-body font-semibold">Description</h3>
									<p class="text-body leading-relaxed">{iss.description}</p>
								</section>
								{#if subIssues(iss.num).length}
									<section class="flex flex-col">
										<h3 class="mb-1 text-body font-semibold">Sub-issues</h3>
										{#each subIssues(iss.num) as sub (sub.num)}
											{@const st = allTasks(sub.num)}
											{@const si = statuses[issueIcon[sub.status]]}
											<Item variant="row" size="xs" onclick={() => openIssue(sub.num)}>
												<ItemContent>
													<ItemTitle><si.icon class={['size-3.5', si.text]} aria-label={issueLabel[sub.status]} /><span class="font-mono text-xs text-muted-foreground">#{sub.num}</span>{sub.title}</ItemTitle>
												</ItemContent>
												<ItemActions class="font-mono text-xs text-muted-foreground">{st.filter((t) => t.status === 'done').length}/{st.length}</ItemActions>
											</Item>
										{/each}
									</section>
								{/if}
								<section class="flex flex-col">
									<h3 class="mb-1 text-body font-semibold">Tasks</h3>
									{#each tasksOf(iss.num) as t (t.num)}
										{@const a = agentOf(t.agent)}
										<Item variant="row" size="xs" onclick={() => open(t.num)}>
											<ItemContent>
												<ItemTitle><span class="font-mono text-xs text-muted-foreground">#{t.num}</span>{t.title}</ItemTitle>
											</ItemContent>
											<ItemActions class="gap-2 text-xs text-muted-foreground">
												{#if a}<RoleAvatar role={a.role} size="sm" />{a.name}{:else}Unassigned{/if}
												<StatusBadge status={t.status} />
											</ItemActions>
										</Item>
									{:else}
										<p class="text-xs text-muted-foreground">이 이슈에 직접 속한 태스크가 없어요.</p>
									{/each}
								</section>
							</div>
							<aside class="detail-aside" aria-label="속성">
								<section class="flex flex-col">
									<h3 class="mb-1 text-body font-semibold">Properties</h3>
									{#each [['Parent', parent ? `#${parent.num} ${parent.title}` : '—'], ['Status', issueLabel[iss.status]], ['Tasks', `${done} / ${all.length} done`], ['Assignees', who.map((a) => a.name).join(' · ') || 'Unassigned'], ['Updated', iss.updated], ['Labels', iss.labels.join(' · ') || '—']] as [k, v] (k)}
										<Item variant="row" size="xs">
											<ItemContent><ItemDescription>{k}</ItemDescription></ItemContent>
											<ItemActions class="min-w-0 truncate text-xs font-medium">{v}</ItemActions>
										</Item>
									{/each}
								</section>
							</aside>
						</InspectorBody>
					</Inspector>
				</div>
			{/if}
			{#if subCur}
				{@const s = subCur}
				{@const st = subRunStatus[s.status]}
				{@const md = subRunMode[s.mode]}
				{@const lt = list.find((t) => t.num === s.task)}
				{@const bad = s.paths.filter((p) => p.from === 'bad')}
				{@const cur = s.runs.at(-1)}
				{@const pathsTitle = `PATHS · ${s.paths.length - bad.length}` + (bad.length ? ' · 범위 위반' : s.paths.some((p) => p.from === 'ask') ? ` · 처음 ${s.paths.filter((p) => p.from === 'orig').length} + @ASK ${s.paths.filter((p) => p.from === 'ask').length}` : '')}
				<!-- 하위 작업 카드: 뷰 오른쪽 (.pen SubRun Inspector Card, 340px) -->
				<Inspector label="하위 작업 {s.id}" floating>
					<InspectorHeader onclose={() => (subSel = undefined)} closeLabel="하위 작업 카드 닫기">
							<span class={['flex size-7 shrink-0 items-center justify-center rounded-sm text-on-solid', md.tile]}><md.icon class="size-4" /></span>
							<div class="flex min-w-0 flex-1 flex-col gap-0.5">
								<span class="meta-line gap-1.5 font-mono font-semibold">
									{s.mode.toUpperCase()} · {s.id}
									{#if s.tier}<span class={['mono-tag flex items-center gap-0.5', tierTone[s.tier]]}><Cpu class="size-2.5" />{s.tier}</span>{/if}
								</span>
								<span class="text-sm font-semibold">{s.goal}</span>
							</div>
						{#snippet sub()}
						<div class="meta-line gap-2">
							<span class={['chip-round gap-1 font-medium', st.tone]}><st.icon class={['size-3', s.status === 'running' && 'animate-spin motion-reduce:animate-none']} />{st.label}</span>
							<span class="flex-1 truncate">{agentName(s.lead)}이 spawn · #{s.task} {lt?.title}</span>
							<Button variant="ghost" size="icon-sm" class="bg-destructive-soft text-destructive" aria-label="하위 작업 중지" disabled={s.status !== 'running'}><Square /></Button>
						</div>
						{/snippet}
					</InspectorHeader>
					<InspectorBody>
						<InspectorSection title="SUB-RUN">
							<InspectorValueRow icon={Split} label="mode" value={{ runner: 'runner · 별도 Run', sub: 'sub · 리드 Run 안', fork: 'fork · 부모 컨텍스트 상속' }[s.mode]} />
							<InspectorValueRow icon={Play} label="시작" value={`리드 Run #${s.leadRun}`} />
							{#if s.retry}
								<InspectorValueRow icon={RotateCcw} label="재시도">
									<span class="flex items-center gap-1 font-medium">Run #{s.runs[0].num}<ArrowRight class="size-3" /><span class="text-muted-foreground">등급</span>
										<span class={['mono-tag', tierTone[s.retry.from]]}>{s.retry.from}</span><ArrowRight class="size-3" /><span class={['mono-tag', tierTone[s.retry.to]]}>{s.retry.to}</span>
									</span>
								</InspectorValueRow>
							{/if}
							{#if s.tier}<InspectorValueRow icon={Gauge} label="tier" value={`${s.tier} · ${s.retry ? `재시도로 ${s.retry.from}에서 올림` : `kind ${s.kind} → 규칙 엔진`}`} />{/if}
							{#if s.model}<InspectorValueRow icon={Cpu} label="model" value={s.model} />{/if}
							<InspectorValueRow icon={Timer} label="소요" value={s.status === 'queued' ? '대기 중' : `${s.minutes}m`} />
						</InspectorSection>
						<InspectorSection title={pathsTitle}>
							{#if bad.length}
								<Alert variant="destructive" class="mb-1">
									<OctagonX />
									<AlertTitle>paths 밖 변경 · {bad.map((p) => p.path).join(', ')}</AlertTitle>
									<AlertDescription>허용된 paths 밖 파일을 고쳐 실패로 처리됐어요. 변경은 적용되지 않았어요.</AlertDescription>
								</Alert>
							{/if}
							{#each s.paths as p (p.path)}
								{@const Icon = p.from === 'ask' ? FilePlus : p.from === 'bad' ? FileX : FileCode}
								<div class="flex h-7 items-center gap-2">
									<span class={['kind-mark size-5 shrink-0', p.from === 'ask' ? 'bg-primary' : p.from === 'bad' ? 'bg-destructive' : 'bg-muted-foreground']}><Icon class="size-3" /></span>
									<span class={['flex-1 truncate font-mono', p.from === 'bad' && 'text-destructive']}>{p.path}</span>
									<span class={['text-caption', p.from === 'ask' ? 'text-primary' : p.from === 'bad' ? 'text-destructive' : 'text-muted-foreground']}>{p.from === 'ask' ? `@ASK · ${p.at}` : p.from === 'bad' ? 'paths 밖' : '처음'}</span>
								</div>
							{:else}
								<p class="text-muted-foreground">파일을 고치지 않는 작업이에요 (조사 · 탐색).</p>
							{/each}
							<p class="pt-1 text-caption text-muted-foreground">
								{#if s.waits}paths가 겹쳐 {s.waits.id}가 끝날 때까지 대기해요 ({s.waits.glob}).
								{:else if bad.length}재시도하면 등급이 한 단계 올라가요. 경로가 더 필요하면 @ASK로 요청해요.
								{:else}paths 밖 파일을 고치면 실패로 처리돼요. 겹치는 경로의 하위 작업은 순서대로 실행돼요.{/if}
							</p>
						</InspectorSection>
						<InspectorSection title={`@REPORT · ac ${s.ac.filter((a) => a.ok).length}/${s.ac.length}`}>
							{#each s.ac as a (a.text)}
								<span class="flex items-center gap-2 py-0.5"><Checkbox checked={a.ok} disabled aria-label={a.text} />{a.text}</span>
							{/each}
							{#if s.report}<p class="mt-1 rounded-md bg-muted px-2.5 py-2 text-caption text-muted-foreground">{s.report}</p>{/if}
						</InspectorSection>
						<InspectorSection title="TOKENS">
							<InspectorValueRow icon={Coins} label="이 Run">
								{#if s.mode !== 'runner'}<TokenMeter included />{:else if cur}<TokenMeter self={cur.tokens} class="text-foreground" />{:else}<span class="text-muted-foreground">시작 전</span>{/if}
							</InspectorValueRow>
							{#if s.runs.length > 1}
								<InspectorValueRow icon={RotateCcw} label="재시도 전 Run #{s.runs[0].num}"><TokenMeter self={s.runs[0].tokens} /></InspectorValueRow>
							{/if}
							<p class="pt-1 text-caption text-muted-foreground">{s.mode === 'runner' ? '리드와 따로 쌓이고 리드 합계에만 더해져요.' : '리드 Run 안에서 돌아 리드 사용량에 이미 들어 있어요.'}</p>
						</InspectorSection>
						<InspectorSection title="WORKTREE · {s.workdir.mode === 'repo' ? 'repo 모드' : 'worktree'}">
							<InspectorValueRow icon={GitBranch} label="모드" value={s.workdir.mode === 'repo' ? 'repo 모드 · worktree 없음' : 'worktree'} />
							<InspectorValueRow icon={FolderGit2} label="경로" value={s.workdir.path ?? '저장소 그대로 · orchstack/app'} />
							<InspectorValueRow icon={GitBranch} label="브랜치" value={s.workdir.branch} />
							{#if s.workdir.mode === 'worktree'}<InspectorValueRow icon={Trash2} label="정리" value={s.status === 'failed' ? '실패 · 24시간 보관 후 삭제' : '완료 후 병합 · 삭제'} />{/if}
						</InspectorSection>
					</InspectorBody>
				</Inspector>
			{/if}
			{#if agentSel}
				{@const a = agentSel}
				{@const mine = list.filter((t) => t.agent === a.sn)}
				<!-- 에이전트 카드: 뷰 오른쪽에 뜬다 (.pen Agent Inspector Card, 340px) -->
				<Inspector label="{a.name} 에이전트" floating>
					<InspectorHeader onclose={() => (inspect = undefined)} closeLabel="에이전트 카드 닫기">
							<RoleAvatar role={a.role} />
							<div class="flex min-w-0 flex-1 flex-col gap-0.5">
								<span class="title-sm gap-1.5">{a.name}<RuntimeLogo runtime={a.runtime} class="size-3.5 ring-0" /></span>
								<span class="text-xs text-muted-foreground">{roles[a.role].label}</span>
								<Badge variant="mono" class="w-fit text-2xs">{a.runtime === 'claude' ? 'Claude Code' : 'Codex CLI'}</Badge>
							</div>
						{#snippet sub()}
						<div class="flex items-center gap-2">
							<Badge variant="secondary">{agentState(a)}</Badge>
							<span class="flex-1"></span>
							<Button variant="ghost" size="icon-sm" class="bg-warning-soft text-warning" aria-label="일시정지 · 현재 단계 끝나면 멈춤" title="일시정지 · 현재 단계 끝나면 멈춤" disabled={agentHold[a.sn] !== undefined} onclick={() => pauseAgent(a.sn)}><Pause /></Button>
							<Button variant="ghost" size="icon-sm" class="bg-destructive-soft text-destructive" aria-label="중지" disabled={agentHold[a.sn] === 'stopped'} onclick={() => stopAgent(a.sn)}><Square /></Button>
						</div>
						{/snippet}
					</InspectorHeader>
					<Tabs bind:value={inspectTab} class="flex min-h-0 flex-1 flex-col gap-0">
						<TabsList variant="line" class="w-full justify-start px-4">
							{#each [['overview', 'Overview'], ['activity', 'Activity'], ['runs', 'Runs'], ['config', 'Config']] as [v, l] (v)}<TabsTrigger value={v}>{l}</TabsTrigger>{/each}
						</TabsList>
						<TabsContent value="overview" class="flex min-h-0 flex-1 flex-col overflow-y-auto px-4 py-3">
							{@const q = queues[a.sn] ?? []}
							{@const now = mine.find((t) => t.status === 'in_progress')}
							<!-- NOW (.pen InspectorSection · Now) — 지금 실행 중인 태스크 -->
							<section class="-mx-4 mb-3 flex flex-col gap-2 border-b px-4 pb-3" aria-label="지금 하는 작업">
								<h3 class="font-mono text-2xs font-semibold tracking-wider text-muted-foreground">NOW</h3>
								{#if now}
									<button type="button" class="flex flex-col gap-2.5 rounded-md border-l-2 border-status-in-progress bg-primary-soft p-3 text-left" onclick={() => open(now.num)}>
										<span class="flex items-center gap-1.5">
											<SquareCheck class="size-3.5 shrink-0 text-node-task" />
											<span class="flex-1 truncate text-body font-semibold">#{now.num} · {now.title}</span>
											<span class="font-mono text-xs font-semibold text-status-in-progress">{Math.round((now.steps[0] / Math.max(1, now.steps[1])) * 100)}%</span>
										</span>
										<Progress value={(now.steps[0] / Math.max(1, now.steps[1])) * 100} class="h-1.5" indicator="bg-status-in-progress" aria-label="#{now.num} 진행" tip={`#${now.num} 완료 조건\n${now.steps[0]} / ${now.steps[1]} 완료${now.run ? ` · Run ${now.run}` : ""}`} />
										<span class="flex items-center gap-1.5 font-mono text-caption text-muted-foreground"><LoaderCircle class="size-3 text-status-in-progress" />완료 조건 {now.steps[0]}/{now.steps[1]}{now.run ? ` · Run ${now.run}` : ''}</span>
									</button>
								{:else}
									<p class="text-xs text-muted-foreground">지금 실행 중인 작업이 없어요.</p>
								{/if}
							</section>
							{#if useMock}
								<!-- NEXT (.pen Agent Inspector · InspectorSection · Queue) — 3줄까지, 넘으면 +N개 더. 서버 큐 API 전에는 목데이터에서만 (#88) -->
								<section class="-mx-4 mb-3 flex flex-col border-b px-4 pb-2.5" aria-label="다음 작업 대기열">
									<div class="flex items-center gap-1.5 pb-1">
										<h3 class="font-mono text-2xs font-semibold tracking-wider text-muted-foreground">NEXT</h3>
										<span class="rounded-xs bg-muted px-1.5 text-2xs font-semibold text-muted-foreground">대기 {q.length}</span>
										<span class="flex-1"></span>
										<a href="/teams?member={a.sn}" class="text-caption font-medium text-primary hover:underline">Tasks 탭 →</a>
									</div>
									{#each q.slice(0, 3) as item, i (`${item.num}·${item.title}`)}
										<div class="flex items-center gap-2 border-b py-2 last:border-b-0">
											<span class="flex size-4.5 shrink-0 items-center justify-center rounded-full bg-muted font-mono text-2xs font-semibold text-muted-foreground">{i + 1}</span>
											{#if item.stuck}<OctagonAlert class="size-3.5 shrink-0 text-status-blocked" />{:else if item.blocked}<CircleDashed class="size-3.5 shrink-0 text-status-waiting" />{:else}<Circle class="size-3.5 shrink-0 text-status-todo" />{/if}
											<span class="flex min-w-0 flex-1 flex-col gap-0.5">
												<span class="truncate text-body font-medium">#{item.num} · {item.title}</span>
												<span class={['truncate text-caption', item.stuck ? 'text-status-blocked' : item.blocked ? 'text-status-waiting' : 'text-status-done']}>{item.note}</span>
											</span>
											<span class="shrink-0 font-mono text-caption font-semibold text-muted-foreground">{item.priority}</span>
										</div>
									{:else}
										<p class="py-2 text-xs text-muted-foreground">대기열이 비어 있어요.</p>
									{/each}
									{#if q.length > 3}
										<div class="flex items-center gap-1.5 py-2 text-caption font-medium text-muted-foreground">
											<ChevronsDown class="size-3" />+{q.length - 3}개 더{#if q.slice(3).some((x) => x.stuck)} · 막힘 {q.slice(3).filter((x) => x.stuck).length}{/if}
											<span class="flex-1"></span><a href="/teams?member={a.sn}" class="text-primary hover:underline">전체 대기열</a>
										</div>
									{/if}
								</section>
							{/if}
							{#if useMock && agentUsage[a.sn]}
								{@const u = agentUsage[a.sn]}
								{@const used = u.tokens.input + u.tokens.cached + u.tokens.output}
								{@const k = (n: number) => `${Math.round(n * 10) / 10}K`}
								<!-- .pen InspectorSection · Skills & MCP — 이 Run에서 쓰는 스킬(호출 수) · MCP(연결 상태). 편집은 멤버 상세 -->
								<section class="-mx-4 mb-3 flex flex-col border-b px-4 pb-2.5" aria-label="쓰는 스킬 · MCP">
									<div class="flex items-center justify-between pb-1">
										<h3 class="text-body font-semibold">Active Skills & MCPs ({u.skills.length + u.mcp.length})</h3>
										<Button variant="outline" size="xs" href="/teams?member={a.sn}"><PanelRightOpen />편집</Button>
									</div>
									{#each u.skills as x (x.name)}
										<div class="flex h-7 items-center gap-2">
											<span class="kind-mark size-5 bg-node-skill"><Puzzle class="size-3" /></span>
											<span class="flex-1 truncate text-body">{x.name}</span>
											<span class="font-mono text-xs text-muted-foreground">{x.calls} calls</span>
										</div>
									{/each}
									{#each u.mcp as x (x.name)}
										<div class="flex h-7 items-center gap-2">
											<span class="kind-mark size-5 bg-node-mcp"><Plug class="size-3" /></span>
											<span class="flex-1 truncate text-body">{x.name}</span>
											<span class={['text-xs font-medium', x.state === 'Connected' ? 'text-success' : 'text-warning']}>{x.state}</span>
										</div>
									{/each}
								</section>
								<!-- .pen InspectorSection · Token & Context — 지금 컨텍스트 = 새 입력 + 캐시 + 출력, 컨텍스트 창 대비 -->
								<section class="-mx-4 mb-3 flex flex-col border-b px-4 pb-3" aria-label="토큰 · 컨텍스트">
									<div class="flex items-center justify-between pb-1">
										<h3 class="text-body font-semibold">Token & Context</h3>
										<Button variant="outline" size="xs" onclick={() => (now ? open(now.num) : (inspectTab = 'runs'))}>View Details</Button>
									</div>
									{#each [['Input (new)', 'bg-token-input', u.tokens.input], ['Cached input', 'bg-token-cached', u.tokens.cached], ['Output', 'bg-token-output', u.tokens.output]] as [l, bg, v] (l)}
										<div class="flex h-6 items-center gap-2 text-body">
											<span class={['size-2 shrink-0 rounded-xs', bg]}></span>
											<span class="flex-1 text-muted-foreground">{l}</span>
											<span class="font-mono">{k(v as number)}</span>
										</div>
									{/each}
									<div class="flex h-6 items-center gap-2 text-body font-medium">
										<span class="size-2 shrink-0"></span>
										<span class="flex-1">Total</span>
										<span class="font-mono">{k(used)} / {u.tokens.limit}K</span>
									</div>
									<div class="mt-1.5 flex h-1.5 overflow-hidden rounded-full bg-muted" role="img" aria-label="컨텍스트 {Math.round((used / u.tokens.limit) * 100)}% 사용 · 입력 {k(u.tokens.input)} · 캐시 {k(u.tokens.cached)} · 출력 {k(u.tokens.output)}" title="컨텍스트 {k(used)} / {u.tokens.limit}K · {Math.round((used / u.tokens.limit) * 100)}%">
										<span class="bg-token-input" style="width: {(u.tokens.input / u.tokens.limit) * 100}%"></span>
										<span class="bg-token-cached" style="width: {(u.tokens.cached / u.tokens.limit) * 100}%"></span>
										<span class="bg-token-output" style="width: {(u.tokens.output / u.tokens.limit) * 100}%"></span>
									</div>
									{#if u.repeat}
										<div class="mt-3 flex gap-2.5 rounded-md border border-warning/40 bg-warning-soft p-3" role="status">
											<TriangleAlert class="size-4 shrink-0 text-warning" />
											<p class="flex flex-col gap-0.5"><span class="text-body font-semibold text-warning">Repeated context detected</span><span class="text-xs">{u.repeat}</span></p>
										</div>
									{/if}
								</section>
							{/if}
							<h3 class="mb-1 text-body font-semibold">맡은 태스크</h3>
							{#each mine as t (t.num)}
								<Item variant="row" size="xs" onclick={() => open(t.num)}>
									{#snippet child({ props })}
										<button type="button" {...props}>
											<ItemContent><ItemTitle>#{t.num} {t.title}</ItemTitle></ItemContent>
											<ItemActions><StatusBadge status={t.status} /></ItemActions>
										</button>
									{/snippet}
								</Item>
							{:else}
								<p class="text-xs text-muted-foreground">맡은 태스크가 없어요.</p>
							{/each}
							<div class="mt-4 flex items-center justify-between text-xs"><span class="text-muted-foreground">토큰 (오늘)</span><span class="font-mono">{a.tokens}</span></div>
						</TabsContent>
						<TabsContent value="activity" class="min-h-0 flex-1 overflow-y-auto px-4 py-2">
							{#each activity[a.sn] ?? [] as ev, i (i)}
								<Item variant="row" size="xs">
									<ItemContent>
										<ItemTitle><Badge variant="mono" class="text-2xs">{ev.type}</Badge><span class="text-xs font-normal text-muted-foreground">{ev.who}</span></ItemTitle>
										<ItemDescription>{ev.text}</ItemDescription>
									</ItemContent>
									<ItemActions class="font-mono text-xs text-subtle-foreground">{ev.time}</ItemActions>
								</Item>
							{:else}
								<p class="py-4 text-center text-xs text-muted-foreground">활동 기록이 없어요.</p>
							{/each}
						</TabsContent>
						<TabsContent value="runs" class="min-h-0 flex-1 overflow-y-auto px-4 py-2">
							{#each mine.flatMap((t) => (details[t.num]?.runs ?? []).map((r) => ({ ...r, task: t.num }))) as r (r.num)}
								<Item variant="row" size="xs">
									<ItemContent>
										<ItemTitle>Run #{r.num} · #{r.task}</ItemTitle>
										<ItemDescription>{r.note}</ItemDescription>
									</ItemContent>
									<ItemActions class="font-mono text-xs text-muted-foreground">{r.time} · {r.tokens}</ItemActions>
								</Item>
							{:else}
								<p class="py-4 text-center text-xs text-muted-foreground">Run 기록이 없어요.</p>
							{/each}
						</TabsContent>
						<TabsContent value="config" class="min-h-0 flex-1 overflow-y-auto px-4 py-2">
							{#each [['Role', roles[a.role].label], ['Runtime', a.runtime === 'claude' ? 'Claude Code' : 'Codex CLI'], ['Status', a.online ? 'Online' : 'Offline']] as [k, v] (k)}
								<Item variant="row" size="xs">
									<ItemContent><ItemDescription>{k}</ItemDescription></ItemContent>
									<ItemActions class="text-xs font-medium">{v}</ItemActions>
								</Item>
							{/each}
							<p class="mt-3 text-xs text-muted-foreground">스킬 · 도구 · 권한 편집은 Teams › 멤버 상세(#27)에서.</p>
						</TabsContent>
					</Tabs>
					<form class="border-t p-3" onsubmit={instruct}>
						<div class="flex items-center gap-1.5 rounded-md border border-input bg-background py-1.5 pr-1.5 pl-3 focus-within:ring-3 focus-within:ring-ring/50">
							<input id="runtime-instruction" bind:value={instruction} placeholder="{a.name}에게 실행 중 지시…" aria-label="{a.name}에게 실행 중 지시" class="bare-input" />
							<Button type="submit" size="icon-sm" aria-label="지시 보내기" disabled={!instruction.trim()}><ArrowUp /></Button>
						</div>
					</form>
				</Inspector>
			{/if}
			</div>

			<!-- 하단 Ops (.pen Workbench/BottomOpsPanel) -->
			<section class={['flex shrink-0 flex-col border-t bg-card', opsOpen ? 'h-49' : 'h-10']} aria-label="Ops">
				<Tabs bind:value={opsTab} class="flex min-h-0 flex-1 flex-col gap-0">
					<div class="flex h-10 shrink-0 items-center gap-4 border-b pr-3 pl-4">
						<TabsList variant="line" class="h-full flex-1 justify-start border-b-0">
							{#each opsTabs as [v, l] (v)}<TabsTrigger value={v} class="h-full">{l}</TabsTrigger>{/each}
						</TabsList>
						<span class={['label-xs', live ? 'text-success' : 'text-muted-foreground']}>
							<span class={['size-1.5 rounded-full', live ? 'bg-success' : 'bg-subtle-foreground']}></span>
							{live ? 'Live' : 'Paused'}
						</span>
						<Button variant="ghost" size="xs" onclick={() => (live = !live)}>
							{#if live}<Pause />Pause stream{:else}<Play />Resume{/if}
						</Button>
						<Button variant="ghost" size="xs"><Download />Export</Button>
						<Button variant="ghost" size="icon-sm" aria-label={opsOpen ? 'Ops 패널 접기' : 'Ops 패널 펼치기'} aria-expanded={opsOpen} onclick={() => (opsOpen = !opsOpen)}>
							{#if opsOpen}<ChevronDown />{:else}<ChevronUp />{/if}
						</Button>
					</div>
					{#if opsOpen}
						<TabsContent value="activity" class="min-h-0 flex-1 overflow-y-auto px-2 py-1.5 font-mono text-xs">
							{#each logs as l, i (i)}
								<div class="flex items-center gap-3 px-2 py-1">
									<span class="text-muted-foreground">{l.time}</span>
									<span class={['font-semibold', roles[l.role].text]}>{l.source}</span>
									<span class="min-w-0 flex-1 truncate">{l.message}</span>
								</div>
							{/each}
						</TabsContent>
						{#each opsTabs.slice(1) as [v] (v)}
							<TabsContent value={v} class="flex flex-1 items-center justify-center text-xs text-muted-foreground">실행기 연동 후 표시돼요.</TabsContent>
						{/each}
					{/if}
				</Tabs>
			</section>
		</SvelteFlowProvider>
		</main>

		<!-- 우측 PM Dock (.pen ProjectPMChatDock) — 메시지 종류 · 결정 패널은 #58 -->
		{#if dockOpen}
			<aside class="flex shrink-0 flex-col w-100 border-l bg-background" aria-label="PM Dock">
				<header class="flex items-center gap-2.5 border-b bg-card px-4 py-2.5">
					<RoleAvatar role="orch" />
					<span class="flex min-w-0 flex-1 flex-col gap-px">
						<span class="text-body font-semibold">Orch · Project Manager</span>
						<span class="truncate text-2xs text-muted-foreground">Project PM · {project.name} · 새 업무 생성/분해/할당</span>
					</span>
					<Button variant="ghost" size="icon-sm" aria-label="PM Dock 접기" onclick={() => (dockOpen = false)}><PanelRightClose /></Button>
				</header>
				<!-- 짧으면 아래에 붙고 길면 스크롤 (justify-end는 넘친 위쪽을 스크롤 밖으로 밀어낸다) -->
				<div bind:this={threadEl} class="flex min-h-0 flex-1 flex-col overflow-y-auto px-4 py-3" aria-live="polite">
					<div class="mt-auto flex flex-col gap-3">
					{#if pending.length}
						{@const d = pending[0]}
						{@const qn = d.questions.findIndex((x) => !x.answer)}
						{@const q = d.questions[qn]}
						{@const rec = q?.options?.find((o) => o.rec)}
						<!-- B · 사용자 판단 대기 요약 (.pen Kk8hB B) — 가장 급한 판단. 선택지를 누르면 바로 답, 긴 답은 판단 패널 -->
						<OrchCard tone="waiting">
							<OrchCardHeader icon={MessageCircleQuestion} tone="text-status-waiting" title="{agentName(d.agent)}이 묻고 있어요 · #{d.task}"><LevelBadge level={2} /></OrchCardHeader>
							<div class="flex flex-col gap-2.5">
								<p class="line-clamp-2 text-body">{q?.q}{#if q?.context} — {q.context}{/if}</p>
								<div class="meta-xs gap-3">
									<span class="flex items-center gap-1"><List class="size-3" />질문 {qn + 1} / {d.questions.length}</span>
									{#if d.files}<span class="flex items-center gap-1"><Paperclip class="size-3" />파일 {d.files}</span>{/if}
									{#if d.code}<span class="flex items-center gap-1"><Code class="size-3" />코드 {d.code}</span>{/if}
								</div>
								{#if q?.options}
									<div class="flex flex-wrap items-center gap-1.5">
										{#each q.options as o (o.key)}
											<Toggle variant="chip" pressed={o.rec} onPressedChange={() => quickAnswer(d.id, o.key)} aria-label="{o.title}로 답하기">{o.title}{o.rec ? ' · 추천' : ''}</Toggle>
										{/each}
									</div>
								{/if}
								<div class="flex items-center gap-2 border-t pt-2">
									<Clock3 class="size-3.5 shrink-0 text-status-waiting" />
									<span class="flex-1 text-xs font-medium text-status-waiting">{d.left} 후 Orch가 {rec ? `‘${rec.title}’으로` : '추천안으로'} 결정</span>
									<Button size="sm" onclick={() => openDecisions()}><Maximize2 />답변하기</Button>
								</div>
								<p class="text-caption text-subtle-foreground">선택지를 바로 누르면 즉시 답변 · 긴 답은 ‘답변하기’로 결정 패널에서</p>
							</div>
						</OrchCard>
					{/if}
					{#each chat as c, i (i)}
						{#if c.kind === 'user'}
							<Message align="end">
								<MessageContent>
									<Bubble align="end"><BubbleContent class="text-body whitespace-pre-wrap">{c.text}</BubbleContent></Bubble>
									<MessageFooter>You · {c.time}</MessageFooter>
								</MessageContent>
							</Message>
						{:else if c.kind === 'orch'}
							<Message>
								<RoleAvatar role="orch" size="sm" />
								<MessageContent>
									<MessageHeader>Orch · {c.time}</MessageHeader>
									<Bubble variant="muted"><BubbleContent class="text-body">{c.text}</BubbleContent></Bubble>
								</MessageContent>
							</Message>
						{:else if c.kind === 'proposal'}
							<!-- 작업 제안 (.pen WorkProposalCard) -->
							<div class="card rounded-lg flex flex-col gap-3 p-3 shadow-xs">
								<div class="flex items-center gap-2">
									<span class="card-kicker">WORK PROPOSAL</span>
									<span class="flex-1"></span>
									<Badge variant="outline">{c.status === 'draft' ? 'Draft' : c.status === 'proceeded' ? 'Proceeded' : 'Cancelled'}</Badge>
								</div>
								<p class="text-sm font-semibold">{c.title}</p>
								<div class="flex flex-col gap-1 text-xs">
									<span class="font-mono text-2xs text-muted-foreground">ISSUE</span>
									<span>{c.issue}</span>
									<span class="mt-1 font-mono text-2xs text-muted-foreground">TASKS</span>
									{#each c.tasks as t, n (n)}
										{@const who = agentOf(t.agent)}
										<span class="flex items-center gap-1.5">
											<span class="w-4 text-muted-foreground">{n + 1}.</span>{t.title}
											<span class="flex-1"></span>
											{#if who}<RoleAvatar role={who.role} size="sm" />{who.name}{/if}
										</span>
									{/each}
									<span class="mt-1 font-mono text-2xs text-muted-foreground">DEPENDENCIES</span>
									<span class="text-muted-foreground">{c.deps}</span>
								</div>
								{#if c.status === 'draft'}
									<div class="flex justify-end gap-2">
										<Button size="sm" variant="ghost" onclick={() => (draft = `${c.title} 제안 수정: `)}>Edit</Button>
										<Button size="sm" variant="outline" onclick={() => (c.status = 'cancelled')}>Cancel</Button>
										<Button size="sm" onclick={() => proceed(c)}>Proceed</Button>
									</div>
								{/if}
							</div>
						{:else if c.kind === 'card'}
							{@const k = c.card}
							{@const i = chat.indexOf(c)}
							<!-- Orch 진행 카드 (.pen Kk8hB) -->
							{#if k.type === 'timer'}
								<OrchCard tone="primary" onpointerenter={() => cardHold[i] !== 'stop' && (cardHold[i] = 'view')} onpointerleave={() => cardHold[i] === 'view' && delete cardHold[i]}>
									<OrchCardHeader icon={CircleCheck} tone="text-status-done" title={k.title}><LevelBadge level={1} /></OrchCardHeader>
									<div class="flex flex-col gap-2.5">
										<div class="flex items-center gap-2.5 rounded-md bg-primary-soft p-2.5">
											<Sparkles class="size-4 shrink-0 text-primary" />
											<div class="flex min-w-0 flex-1 flex-col gap-0.5">
												<span class="text-xs font-semibold">{k.proposal.title}</span>
												<span class="text-caption text-muted-foreground">{k.proposal.desc}</span>
											</div>
											{#if agentOf(k.proposal.agent)}<RoleAvatar role={agentOf(k.proposal.agent)!.role} size="sm" />{/if}
										</div>
										{#if k.done}
											<p class="saved-note"><Check class="size-3" />{k.done}</p>
										{:else}
											<div class="flex flex-col gap-1.5">
												<div class="flex items-center gap-2">
													{#if cardHold[i]}
														<Pill class="bg-primary-soft px-2.5 py-1 text-xs font-semibold text-primary"><Eye />{cardHold[i] === 'view' ? '보는 중 · 타이머 멈춤' : '타이머 멈춤'}</Pill>
													{:else}
														<Pill class="bg-primary-soft px-2.5 py-1 text-xs font-semibold text-primary"><Timer />{k.seconds}초 후 자동 진행 · 보면 멈춤</Pill>
													{/if}
													<span class="flex-1"></span>
													<span class="font-mono text-caption text-subtle-foreground">연속 자동 {k.streak[0]} / {k.streak[1]}</span>
												</div>
												<Progress value={(k.seconds / 4) * 100} class="h-1 bg-primary-soft" aria-label="자동 진행까지" tip={`자동 진행까지\n${k.seconds}초 · 개입하면 멈춰요`} />
											</div>
											<div class="flex flex-wrap items-center gap-1.5">
												<Button size="sm" onclick={() => assignNext(k)}><Play />지금 진행</Button>
												<Button size="sm" variant="outline" onclick={() => (cardHold[i] = 'stop')}><Pause />멈춤</Button>
												<Button size="sm" variant="ghost" aria-expanded={altOpen === i} onclick={() => ((cardHold[i] = 'stop'), (altOpen = altOpen === i ? undefined : i))}><ChevronDown />다른 선택</Button>
											</div>
											{#if altOpen === i}
												<div class="flex flex-col gap-1 border-t pt-2">
													<span class="text-caption font-semibold text-subtle-foreground">다른 선택</span>
													{#each k.alternatives as alt, n (alt)}
														{@const AltIcon = [UserRound, Inbox, Archive, CircleCheckBig][n] ?? Check}
														<button type="button" class="flex items-center gap-2 py-1 text-left text-xs outline-none hover:underline focus-visible:underline" onclick={() => settle(k, alt)}><AltIcon class="size-3.5 text-muted-foreground" />{alt}</button>
													{/each}
												</div>
											{/if}
										{/if}
									</div>
								</OrchCard>
							{:else if k.type === 'guard'}
								<OrchCard tone="danger">
									<OrchCardHeader icon={CirclePause} tone="text-destructive" title={k.title}><Pill class="bg-card text-status-blocked"><ShieldAlert />{k.tag}</Pill></OrchCardHeader>
									<div class="flex flex-col gap-2.5 text-xs">
										<p>{k.desc}</p>
										<div class="flex flex-col gap-1 rounded-sm bg-card p-2.5">
											{#each k.rounds as r (r)}<span class="flex items-center gap-1.5"><Undo2 class="size-3.5 text-destructive" />{r}</span>{/each}
										</div>
										<div class="meta-xs gap-3 text-caption">
											{#each k.cost as v, n (v)}{@const CostIcon = [Repeat, Coins, Timer][n]}<span class="flex items-center gap-1"><CostIcon class="size-3" />{v}</span>{/each}
										</div>
										<p class="text-muted-foreground">{k.judgment}</p>
										{#if k.done}
											<p class="saved-note"><Check class="size-3" />{k.done}</p>
										{:else}
											<div class="flex flex-wrap items-center gap-1.5">
												<Button size="sm" onclick={() => (editTask(k.task), settle(k, `#${k.task} 완료 조건에 기준 추가 후 재개`))}><ListPlus />기준 추가 후 재개</Button>
												<Button size="sm" variant="outline" onclick={() => settle(k, `#${k.task} 반려 1회 더 허용`)}><RotateCcw />1회 더 허용</Button>
												<Button size="sm" variant="ghost" onclick={() => settle(k, `#${k.task} Manual로 전환`)}>Manual로 전환</Button>
											</div>
										{/if}
									</div>
								</OrchCard>
							{:else if k.type === 'approval'}
								<OrchCard tone="review">
									<OrchCardHeader icon={GitPullRequest} tone="text-status-review" title={k.title}><LevelBadge level={3} /></OrchCardHeader>
									<div class="flex flex-col gap-2.5 text-xs">
										<p>{k.desc}</p>
										<div class="flex flex-col gap-1.5 rounded-md bg-muted p-2.5 text-caption">
											<div class="flex flex-wrap gap-3 text-muted-foreground"><span class="flex items-center gap-1"><GitBranch class="size-3" />{k.branch}</span><span class="flex items-center gap-1"><FileDiff class="size-3" />{k.files}</span></div>
											<div class="flex flex-wrap gap-3 text-status-done"><span class="flex items-center gap-1"><ShieldCheck class="size-3" />{k.review}</span><span class="flex items-center gap-1"><FlaskConical class="size-3" />{k.qa}</span></div>
										</div>
										{#if k.done}
											<p class="saved-note"><Check class="size-3" />{k.done}</p>
										{:else}
											<div class="flex flex-wrap items-center gap-1.5">
												<Button size="sm" onclick={() => settle(k, '승인 · 병합했어요 · 스테이징 배포 시작')}><Check />승인 · 병합</Button>
												<Button size="sm" variant="outline" onclick={() => toast('변경 보기는 Git 연결(API 단계)에서 열려요')}><FileDiff />변경 보기</Button>
												<span class="flex-1"></span>
												<Button size="sm" variant="ghost" onclick={() => settle(k, '병합을 거절했어요')}><X />거절</Button>
											</div>
										{/if}
									</div>
								</OrchCard>
							{:else if k.type === 'blocked'}
								<OrchCard tone="danger">
									<OrchCardHeader icon={Ban} tone="text-status-blocked" title={k.title}><LevelBadge level={4} /></OrchCardHeader>
									<div class="flex flex-col gap-2.5 text-xs">
										<p>{k.desc}</p>
										<div class="flex flex-col gap-1.5 rounded-md bg-card p-2.5 text-caption">
											<span class="flex items-center gap-1 text-status-blocked"><ShieldX class="size-3" />{k.policy}</span>
											<span class="flex items-center gap-1 text-muted-foreground"><Lightbulb class="size-3" />{k.alternative}</span>
										</div>
										{#if k.done}
											<p class="saved-note"><Check class="size-3" />{k.done}</p>
										{:else}
											<div class="flex flex-wrap items-center gap-1.5">
												<Button size="sm" onclick={() => settle(k, '대안으로 진행 · 새 브랜치로 push 후 PR')}><GitBranch />대안으로 진행</Button>
												<Button size="sm" variant="outline" onclick={() => ((opsOpen = true), (opsTab = 'logs'))}><Terminal />로그 보기</Button>
												<span class="flex-1"></span>
												<Button size="sm" variant="ghost" onclick={() => (pauseAgent(k.agent), settle(k, `${agentName(k.agent)} 멈춤`))}><Pause />{agentName(k.agent)} 멈춤</Button>
											</div>
										{/if}
									</div>
								</OrchCard>
							{:else if k.type === 'limit'}
								<OrchCard tone="waiting">
									<OrchCardHeader icon={Gauge} tone="text-status-waiting" title={k.title}><Pill class="bg-warning-soft text-status-waiting"><TriangleAlert />한도</Pill></OrchCardHeader>
									<div class="flex flex-col gap-2.5 text-xs">
										<p>{k.desc}</p>
										<div class="flex flex-col gap-1.5 py-1.5">
											<div class="flex items-center gap-2"><Terminal class="size-3.5 text-muted-foreground" /><span class="flex-1 text-muted-foreground">{k.label}</span><span class="font-mono font-medium text-status-blocked">{k.used}%</span><span class="font-mono text-subtle-foreground">· {k.reset}</span></div>
											<Progress value={k.used} class="h-1.5 bg-muted" indicator="bg-status-blocked" aria-label={k.label} tip={`${k.label}\n${k.used}% 사용 · ${k.reset}`} />
										</div>
										{#if k.done}
											<p class="saved-note"><Check class="size-3" />{k.done}</p>
										{:else}
											<div class="flex flex-wrap items-center gap-1.5">
												<Button size="sm" onclick={() => settle(k, 'Claude로 전환했어요 · 다음 Run부터')}><Repeat />Claude로 전환</Button>
												<Button size="sm" variant="outline" onclick={() => settle(k, '한도 동안 P0–P1만 실행')}><Funnel />P0–P1만 실행</Button>
												<span class="flex-1"></span>
												<Button size="sm" variant="ghost" onclick={() => settle(k, '나중에 다시 알려 드릴게요')}><Clock3 />나중에</Button>
											</div>
										{/if}
									</div>
								</OrchCard>
							{:else if k.type === 'failed'}
								<OrchCard onpointerenter={() => cardHold[i] !== 'stop' && (cardHold[i] = 'view')} onpointerleave={() => cardHold[i] === 'view' && delete cardHold[i]}>
									<OrchCardHeader icon={CircleX} tone="text-status-blocked" title={k.title}><LevelBadge level={1} /></OrchCardHeader>
									<div class="flex flex-col gap-2.5 text-xs">
										<p>{k.desc}</p>
										<div class="flex flex-col gap-1.5 rounded-md bg-code-bg p-2.5 font-mono"><span class="text-code-fg">{k.log[0]}</span><span class="text-code-muted">{k.log[1]}</span></div>
										{#if k.done}
											<p class="saved-note"><Check class="size-3" />{k.done}</p>
										{:else}
											<Pill class="bg-primary-soft px-2.5 py-1 text-xs font-semibold text-primary">{#if cardHold[i]}<Eye />보는 중 · 타이머 멈춤{:else}<Timer />{k.seconds}초 후 재시도 · {k.tries[0]} / {k.tries[1]}회{/if}</Pill>
											<div class="flex flex-wrap items-center gap-1.5">
												<Button size="sm" onclick={() => settle(k, `Run 재시도 · #${k.task}`)}><RotateCcw />지금 재시도</Button>
												<Button size="sm" variant="outline" onclick={() => ((opsOpen = true), (opsTab = 'logs'))}><Terminal />로그 보기</Button>
												<span class="flex-1"></span>
												<Button size="sm" variant="ghost" onclick={() => ((cardHold[i] = 'stop'), open(k.task))}><User />직접 볼게요</Button>
											</div>
										{/if}
									</div>
								</OrchCard>
							{:else}
								<OrchCard tone="done">
									<OrchCardHeader icon={Flag} tone="text-status-done" title={k.title}><Pill class="bg-success-soft text-status-done"><Check />완료</Pill></OrchCardHeader>
									<div class="flex flex-col gap-2.5 text-xs">
										<p>{k.desc}</p>
										<div class="meta-xs flex-wrap gap-3 text-caption">
											<span class="flex items-center gap-1 text-status-done"><ListChecks class="size-3" />{k.stats.tasks}</span>
											<span class="flex items-center gap-1"><Coins class="size-3" />{k.stats.tokens}</span>
											<span class="flex items-center gap-1"><Timer class="size-3" />{k.stats.time}</span>
											<span class="flex items-center gap-1"><Repeat class="size-3" />{k.stats.rework}</span>
										</div>
										{#if k.done}
											<p class="saved-note"><Check class="size-3" />{k.done}</p>
										{:else}
											<div class="flex flex-wrap items-center gap-1.5">
												<Button size="sm" onclick={() => toast('보고서 양식은 Settings › 보고서 양식에서 정해요', { description: 'API 단계에서 실제 보고서가 열려요' })}><FileText />보고서 보기</Button>
												<Button size="sm" variant="outline" onclick={() => settle(k, '이슈를 닫았어요')}><Archive />이슈 닫기</Button>
												<span class="flex-1"></span>
												<Button size="sm" variant="ghost" onclick={() => (draft = '다음 이슈 제안해줘 ')}><Sparkles />다음 이슈 제안</Button>
											</div>
										{/if}
									</div>
								</OrchCard>
							{/if}
						{:else}
							<!-- 명령 결과 (.pen CommandResultCard) -->
							<div class="card rounded-lg flex flex-col gap-2 p-3 shadow-xs">
								<div class="flex items-center gap-2">
									<span class="card-kicker">COMMAND RESULT</span>
									<span class="text-xs text-muted-foreground">Orch · {c.time}</span>
									<span class="flex-1"></span>
									<Badge variant="outline">from Proposal</Badge>
								</div>
								{#each c.lines as l, n (n)}<p class="flex items-center gap-1.5 text-xs"><Check class="size-3 text-status-done" />{l}</p>{/each}
							</div>
						{/if}
					{/each}
					</div>
				</div>
				<form class="px-4 pt-2 pb-3.5" onsubmit={send}>
					<div class="flex items-center gap-1.5 rounded-md border border-input bg-card py-2 pr-2.5 pl-2 focus-within:ring-3 focus-within:ring-ring/50">
						<Button variant="ghost" size="icon-sm" aria-label="파일 첨부"><Paperclip /></Button>
						<input
							bind:value={draft}
							placeholder="Orch에게 기능 논의 · Issue/Task 요청 · 할당 · 시작…"
							aria-label="Orch에게 메시지"
							class="bare-input"
						/>
						<Button type="submit" size="icon-sm" aria-label="보내기" disabled={!draft.trim()}><ArrowUp /></Button>
					</div>
				</form>
			</aside>
		{/if}
	</div>
	<!-- 판단 대기 패널 (.pen DecisionPanel) — 전체 / 이 태스크만 -->
	<Dialog bind:open={panel}>
		<DialogContent size="xl" tall>
			<DialogHeader>
				<DialogTitle class="flex items-center gap-2">판단 대기 <Badge variant="secondary">{pending.length}</Badge></DialogTitle>
				<DialogDescription>{scope === undefined ? `${project.name} · L2 모호한 판단` : `Task #${scope}만`}</DialogDescription>
			</DialogHeader>
			<DialogBody padded={false} class="flex-row">
				<nav class="flex w-72 shrink-0 flex-col overflow-y-auto border-r bg-background p-2" aria-label="판단 대기 목록">
					{#each shownQueue as d (d.id)}
						{@const who = agentOf(d.agent)}
						<button
							type="button"
							aria-pressed={active === d.id}
							onclick={() => ((active = d.id), (pick = undefined), (answer = ''))}
							class={['flex flex-col gap-1 rounded-md px-3 py-2.5 text-left outline-none hover:bg-muted focus-visible:ring-3 focus-visible:ring-ring/50', active === d.id && 'bg-accent']}
						>
							<span class="flex items-center gap-1.5 text-sm font-medium">{#if who}<RoleAvatar role={who.role} size="sm" />{who.name}{/if} · #{d.task}</span>
							<span class="truncate text-xs text-muted-foreground">{d.topic}</span>
							<span class={['font-mono text-2xs', d.left ? 'text-status-waiting' : 'text-subtle-foreground']}>{d.left ? `${d.left} 남음` : `결정됨 · ${d.decided}`}</span>
						</button>
					{:else}
						<p class="p-4 text-center text-xs text-muted-foreground">판단 대기가 없어요.</p>
					{/each}
				</nav>
				{#if dec}
					{@const who = agentOf(dec.agent)}
					<section class="flex min-w-0 flex-1 flex-col">
						<div class="flex items-start gap-3 border-b px-6 py-4">
							{#if who}<RoleAvatar role={who.role} />{/if}
							<div class="flex min-w-0 flex-1 flex-col gap-0.5">
								<span class="text-sm font-semibold">{who?.name} · #{dec.task} {dec.title}</span>
								<span class="text-xs text-muted-foreground">{dec.sub}</span>
							</div>
							<Badge variant="secondary" class="font-mono">{dec.left ? `${dec.left} 남음` : `결정됨 · ${dec.decided}`}</Badge>
						</div>
						<ol class="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto px-6 py-4">
							{#each dec.questions as q, n (n)}
								<li class={['flex flex-col gap-2', n !== qi && 'opacity-80']}>
									<p class="title-sm gap-2">
										<span class="font-mono text-xs text-muted-foreground">Q{n + 1}</span>{q.q}
										{#if q.answer}<span class="text-xs font-normal text-status-done">→ {q.answer}</span>{:else if n !== qi}<span class="text-xs font-normal text-muted-foreground">대기</span>{/if}
									</p>
									{#if n === qi}
										{#if q.context}<p class="text-body leading-relaxed text-muted-foreground">{q.context}</p>{/if}
										{#if q.options}
											<!-- .pen OptionCard on/off -->
											<ChoiceCards aria-label="Q{n + 1} 선택지" class="grid-cols-3 gap-2" bind:value={() => pick ?? null, (v) => (pick = (v ?? undefined) as typeof pick)}>
												{#each q.options as o (o.key)}
													<ChoiceCard value={o.key} class="rounded-lg">
														<span class="title-sm gap-1.5"><span class="font-mono text-xs text-muted-foreground">{o.key}</span>{o.title}{#if o.rec}<Badge variant="secondary" class="text-2xs">추천</Badge>{/if}</span>
														{#if o.desc}<span class="text-xs text-muted-foreground">{o.desc}</span>{/if}
													</ChoiceCard>
												{/each}
											</ChoiceCards>
										{/if}
									{/if}
								</li>
							{/each}
						</ol>
						{#if qi >= 0}
							<form class="flex flex-col gap-2 border-t px-6 py-3" onsubmit={reply}>
								<Textarea bind:value={answer} rows={2} placeholder="선택지에 덧붙일 말이나 직접 답을 적어주세요" aria-label="답변" onkeydown={(e) => (e.metaKey || e.ctrlKey) && e.key === 'Enter' && e.currentTarget.form?.requestSubmit()} />
								<div class="flex items-center gap-2">
									<p class="flex-1 text-xs text-muted-foreground">답변은 #{dec.task} 결정 기록으로 남고 {who?.name}에게 전달돼요 · ⌘↵ 보내기</p>
									<Button type="submit" size="sm" disabled={!pick && !answer.trim()}>보내기</Button>
								</div>
							</form>
						{/if}
					</section>
				{/if}
			</DialogBody>
		</DialogContent>
	</Dialog>

	<!-- Task Editor (.pen XBNVi A · 새 태스크 / A' · 편집) -->
	<TaskEditorDialog bind:this={editor} bind:open={editorOpen} project={project.name} tasks={list} issues={issueList} agents={agentList} {details} onsave={saveDraft} oncreated={open} />
{:else}
	<Empty class="h-full">
		<EmptyHeader>
			<EmptyTitle>프로젝트를 찾을 수 없어요</EmptyTitle>
			<EmptyDescription><a href="/p">All Projects</a>에서 다시 선택하세요.</EmptyDescription>
		</EmptyHeader>
	</Empty>
{/if}
