<script lang="ts">
	/// Workbench — 좌측 Quick Panel · 가운데 뷰(Diagram / Kanban / Issues) + Ops · 우측 PM Dock.
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
	import * as Table from '$lib/components/ui/table';
	import { StatusSelect } from '$lib/components/ui/status-select';
	import { Checkbox } from '$lib/components/ui/checkbox';
	import * as Alert from '$lib/components/ui/alert';
	import Link2 from '@lucide/svelte/icons/link-2';
	import OctagonX from '@lucide/svelte/icons/octagon-x';
	import Square from '@lucide/svelte/icons/square';
	import MessageCircleQuestion from '@lucide/svelte/icons/message-circle-question';
	import Check from '@lucide/svelte/icons/check';
	import Hourglass from '@lucide/svelte/icons/hourglass';
	import * as Message from '$lib/components/ui/message';
	import * as Bubble from '$lib/components/ui/bubble';
	import * as Dialog from '$lib/components/ui/dialog';
	import { Textarea } from '$lib/components/ui/textarea';
	import { untrack, onDestroy, type Component } from 'svelte';
	import LoaderCircle from '@lucide/svelte/icons/loader-circle';
	import { toast } from 'svelte-sonner';
	import { useMock } from '$lib/api/env';
	import type { ApiRun } from '$lib/api/types';
	import { ago } from '$lib/time';
	import { project as wb, openProject, closeProject, viewTasks, viewIssues, viewAgents, moveTask, runsOf, stopRun } from '$lib/project.svelte';
	import { SvelteFlow, Background, BackgroundVariant, Controls, Panel, MarkerType, type Node, type Edge } from '@xyflow/svelte';
	import '@xyflow/svelte/dist/style.css';
	import DiagramNode, { type DiagramNodeData, type MenuEntry } from '$lib/components/orch/diagram/diagram-node.svelte';
	import * as ContextMenu from '$lib/components/ui/context-menu';
	import UserPlus from '@lucide/svelte/icons/user-plus';
	import StepForward from '@lucide/svelte/icons/step-forward';
	import PanelRight from '@lucide/svelte/icons/panel-right';
	import FilterIcon from '@lucide/svelte/icons/filter';
	import SquarePen from '@lucide/svelte/icons/square-pen';
	import SubRunNode, { subRunStatus, subRunMode, tierTone } from '$lib/components/orch/diagram/sub-run-node.svelte';
	import TokenMeter from '$lib/components/orch/diagram/token-meter.svelte';
	import FolderTree from '@lucide/svelte/icons/folder-tree';
	import FileCode from '@lucide/svelte/icons/file-code';
	import FilePlus from '@lucide/svelte/icons/file-plus';
	import FileX from '@lucide/svelte/icons/file-x';
	import Split from '@lucide/svelte/icons/split';
	import RotateCcw from '@lucide/svelte/icons/rotate-ccw';
	import ArrowRight from '@lucide/svelte/icons/arrow-right';
	import FolderGit2 from '@lucide/svelte/icons/folder-git-2';
	import Trash2 from '@lucide/svelte/icons/trash-2';
	import { Badge } from '$lib/components/ui/badge';
	import * as Kanban from '$lib/components/ui/kanban';
	import { Progress } from '$lib/components/ui/progress';
	import * as Tabs from '$lib/components/ui/tabs';
	import * as Empty from '$lib/components/ui/empty';
	import * as InputGroup from '$lib/components/ui/input-group';
	import * as Item from '$lib/components/ui/item';
	import * as Avatar from '$lib/components/ui/avatar';
	import { Button } from '$lib/components/ui/button';
	import { Toggle } from '$lib/components/ui/toggle';
	import { StatusBadge } from '$lib/components/ui/status-badge';
	import { RoleAvatar } from '$lib/components/ui/role-avatar';
	import { RuntimeLogo } from '$lib/components/ui/runtime-logo';
	import { statuses, statusOrder, type TaskStatus } from '$lib/status';
	import { roles } from '$lib/roles';
	import { tasks, agents, logs, issues, taskDetails, agentActivity, decisions, thread, subRuns, leadRuns, type Issue, type Chat, type SubRun, type SpawnMode } from '$lib/mock';
	import { store, defaultTeam } from '$lib/teams.svelte';
	import { Segmented } from '$lib/components/ui/segmented';
	import { Pill } from '$lib/components/ui/pill';
	import * as Inspector from '$lib/components/ui/inspector';
	import Bot from '@lucide/svelte/icons/bot';
	import Users from '@lucide/svelte/icons/users';
	import GitFork from '@lucide/svelte/icons/git-fork';
	import { cn } from '$lib/utils';
	import Pencil from '@lucide/svelte/icons/pencil';
	import ChevronsUpDown from '@lucide/svelte/icons/chevrons-up-down';
	import Maximize2 from '@lucide/svelte/icons/maximize-2';
	import Minimize2 from '@lucide/svelte/icons/minimize-2';
	import CircleDotIcon from '@lucide/svelte/icons/circle-dot';
	import GitBranchIcon from '@lucide/svelte/icons/git-branch';
	import Bold from '@lucide/svelte/icons/bold';
	import Italic from '@lucide/svelte/icons/italic';
	import List from '@lucide/svelte/icons/list';
	import ListChecks from '@lucide/svelte/icons/list-checks';
	import Code from '@lucide/svelte/icons/code';
	import LinkIcon from '@lucide/svelte/icons/link';
	import AtSign from '@lucide/svelte/icons/at-sign';
	import FileText from '@lucide/svelte/icons/file-text';
	import Link2Icon from '@lucide/svelte/icons/link-2';
	import TimerIcon from '@lucide/svelte/icons/timer';
	import Tag from '@lucide/svelte/icons/tag';
	import Flag from '@lucide/svelte/icons/flag';
	import WandSparkles from '@lucide/svelte/icons/wand-sparkles';
	import Sparkles from '@lucide/svelte/icons/sparkles';
	import X from '@lucide/svelte/icons/x';
	import Radio from '@lucide/svelte/icons/radio';
	import UserRoundX from '@lucide/svelte/icons/user-round-x';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
	import { Kbd } from '$lib/components/ui/kbd';
	import { mergeProps } from 'bits-ui';
	import { Input } from '$lib/components/ui/input';
	import * as Attachment from '$lib/components/ui/attachment';
	import * as HoverCard from '$lib/components/ui/hover-card';
	import { Switch } from '$lib/components/ui/switch';
	import { applyMd, type MdFormat } from '$lib/components/ui/md-editor';
	import AssigneePicker from '$lib/components/orch/task/assignee-picker.svelte';
	import DependsPicker from '$lib/components/orch/task/depends-picker.svelte';
	import PriorityPicker from '$lib/components/orch/task/priority-picker.svelte';
	import { Checklist } from '$lib/components/ui/checklist';
	import { priorities, priorityOrder, type Priority } from '$lib/priority';
	import type { Role } from '$lib/roles';
	import type { TaskDetail } from '$lib/mock';

	const project = $derived(store.projects.find((p) => p.sn === Number(page.params.project)));

	// 태스크 · 이슈 · 에이전트 — 목데이터 모드(VITE_MOCK=1)면 화면 안에서만 바꾸고, 아니면 프로젝트 스토어(snapshot + SSE)를 본다.
	let local = $state(tasks.map((t) => ({ ...t })));
	// 일시정지한 태스크 (태스크 메뉴 Pause · Resume). Diagram 노드가 스크립트 초기화 중 메뉴를 만들어 위에 둔다.
	let paused = $state<number[]>([]);
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
	const task = (num: number) => list.find((t) => t.num === num)!;
	const lanes = $derived(statusOrder.filter((s) => s !== 'failed' || count(s) > 0));
	// 드래그 중 열 배치는 따로 두고, 놓을 때 태스크 상태에 반영한다.
	let board = $state<Kanban.KanbanValue>({});
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
	// 공유 링크(?task= · ?issue=)로 오면 상세를 연다 (Copy link · Tasks 화면).
	$effect(() => {
		const t = Number(page.url.searchParams.get('task'));
		const i = Number(page.url.searchParams.get('issue'));
		// 알림(상단 벨)의 결정 답하기 → 그 태스크의 결정 패널
		const d = Number(page.url.searchParams.get('decide'));
		untrack(() => {
			if (d) openDecisions(d);
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
			badge: a.activity.split(' · ')[0],
			meta: lead.length ? `sub-run ${lead.length} · ctx ${ctx}` : `ctx ${ctx}`
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
	// 태스크 상태 · 선택이 바뀌면 노드에 반영한다 (위치는 유지).
	$effect(() => {
		const sel = selected;
		const data = Object.fromEntries(diagramTasks.map((n) => [`task-${n}`, taskData(n)]));
		untrack(() => {
			nodes = nodes.map((n) => ({ ...n, data: data[n.id] ?? n.data, selected: n.id === `task-${sel}` }));
		});
	});

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
			'sep',
			{ label: 'Open Details', icon: PanelRight, shortcut: '↵', onSelect: () => open(num) }
		];
	}

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

	// ── Task Editor (.pen XBNVi A · 새 태스크 / A' · 편집) · QuickAdd (.pen biSss) ─────────────────────
	// 목데이터 단계: 만들고 고친 값은 페이지 목록(local) · 상세(details)에 바로 넣는다. 서버 저장은 API 단계(#92).
	type Draft = {
		num?: number;
		title: string;
		body: string;
		issue: number;
		status: TaskStatus;
		agent?: number;
		priority: Priority;
		criteria: TaskDetail['criteria'];
		deps: TaskDetail['deps'];
		eta: string;
		labels: string[];
		files: { name: string; size: number; url?: string }[];
	};
	let editor = $state<Draft>();
	let editorWide = $state(false);
	let more = $state(false);
	let bodyArea = $state<HTMLTextAreaElement | null>(null);
	let fileInput = $state<HTMLInputElement | null>(null);
	/// 새 태스크 기본 이슈: 열린 상세의 이슈 → 첫 진행 중 이슈 → 첫 이슈.
	const defaultIssue = () => (cur ? cur.issue : (issueList.find((i) => i.status === 'in_progress') ?? issueList[0])?.num ?? 0);
	const blank = (over: Partial<Draft> = {}): Draft => ({ title: '', body: '', issue: defaultIssue(), status: 'backlog', priority: 'P2', criteria: [], deps: [], eta: '', labels: [], files: [], ...over });

	/// 새 태스크 편집기 (QuickAdd ⤢ · N).
	function newTask(over: Partial<Draft> = {}) {
		quick = undefined;
		editorWide = false;
		editor = blank(over);
	}
	/// 편집 모드 (Task 상세 ✎ · E).
	function editTask(num: number) {
		const t = task(num);
		const d = details[num];
		editorWide = false;
		editor = {
			num, title: t.title, body: d?.description ?? t.description ?? '', issue: t.issue, status: t.status, agent: t.agent, priority: t.priority,
			criteria: structuredClone($state.snapshot(d?.criteria ?? [])), deps: [...(d?.deps ?? [])], eta: d?.eta ?? '', labels: [...(d?.labels ?? [])], files: []
		};
	}
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
	function createTask(d: Draft): number {
		const num = Math.max(0, ...list.map((t) => t.num)) + 1;
		local.push({ num, project: project?.sn, title: d.title.trim(), status: d.status, priority: d.priority, agent: d.agent, issue: d.issue, steps: [d.criteria.filter((c) => c.done).length, d.criteria.length], messages: 0, updated: 'just now' });
		details[num] = { description: d.body, criteria: d.criteria, deps: d.deps, runs: [], activity: [{ type: 'CREATE', who: '나', time: now(), text: `Task #${num} 생성` }], eta: d.eta, labels: d.labels };
		return num;
	}
	/// 편집기 제출 (⌘↵). 새 태스크면 만들고(계속 만들기면 비운 편집기 유지), 편집이면 덮어쓴다.
	function saveTask(e?: Event) {
		e?.preventDefault();
		const d = editor;
		if (!d || !d.title.trim()) return;
		if (!useMock) {
			toast.info('서버 저장은 API 단계에서 붙여요');
			return;
		}
		if (d.num === undefined) {
			const num = createTask(d);
			toast.success(`Task #${num}을 만들었어요`);
			if (more) {
				editor = blank({ issue: d.issue, status: d.status, agent: d.agent, priority: d.priority });
				return;
			}
			editor = undefined;
			open(num);
			return;
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
		editor = undefined;
	}

	/// 설명 툴바 서식 (MdEditor와 같은 규칙). @는 언급.
	async function format(kind: MdFormat) {
		if (!bodyArea || !editor) return;
		const { next, cursor } = applyMd(bodyArea.value, bodyArea.selectionStart, bodyArea.selectionEnd, kind);
		editor.body = next;
		await Promise.resolve();
		bodyArea.focus();
		bodyArea.setSelectionRange(cursor, cursor);
	}
	const bodyTools: { k: MdFormat; icon: typeof Bold; label: string }[] = [
		{ k: 'b', icon: Bold, label: '굵게' },
		{ k: 'i', icon: Italic, label: '기울임' },
		{ k: 'ul', icon: List, label: '목록' },
		{ k: 'task', icon: ListChecks, label: '체크리스트' },
		{ k: 'code', icon: Code, label: '코드' },
		{ k: 'link', icon: LinkIcon, label: '링크' },
		{ k: 'mention', icon: AtSign, label: '언급' }
	];
	/// 첨부 — 이미지는 썸네일, 그 밖은 파일 상자. 목데이터라 브라우저 안에서만 보여 준다.
	function attach(files: FileList | null) {
		if (!editor || !files) return;
		for (const f of files) editor.files.push({ name: f.name, size: f.size, url: f.type.startsWith('image/') ? URL.createObjectURL(f) : undefined });
	}
	const fileMeta = (f: { name: string; size: number }) => `${f.name.split('.').pop()?.toUpperCase() ?? 'FILE'} · ${f.size < 1024 ? `${f.size}B` : `${Math.round(f.size / 1024)}KB`}`;

	/// 추천 담당 — 제목 · 라벨에 역할 낱말이 있으면 그 역할 (.pen 추천 · 역할 일치).
	const roleWords: Partial<Record<Role, string[]>> = {
		frontend: ['ui', 'front', '화면', 'login', '폼'],
		backend: ['api', 'db', 'server', 'token', 'backend', '스키마'],
		qa: ['qa', 'test', '검증', 'e2e'],
		designer: ['design', '디자인', '시안', 'ui'],
		reviewer: ['review', '리뷰']
	};
	function recommendFor(title: string, labels: string[]) {
		const words = `${title} ${labels.join(' ')}`.toLowerCase();
		return agentList.filter((a) => roleWords[a.role]?.some((w) => words.includes(w))).map((a) => a.sn);
	}
	const openCount = (sn: number) => list.filter((t) => t.agent === sn && !['done', 'cancelled'].includes(t.status)).length;
	/// Orch에게 배정 맡기기 — 추천 중(없으면 전체) 열린 태스크가 가장 적은 에이전트. 실제 판단은 서버 Orch(#87).
	function orchPick(title: string, labels: string[]) {
		const pool = recommendFor(title, labels);
		const best = (pool.length ? agentList.filter((a) => pool.includes(a.sn)) : agentList).toSorted((a, b) => openCount(a.sn) - openCount(b.sn))[0];
		if (best) toast(`Orch가 ${best.name}에게 배정했어요`, { description: '역할 · 부하 기준' });
		return best?.sn;
	}
	/// 제목만 적으면 Orch가 설명 · 완료 조건 · 담당 · 의존을 제안 (.pen Orch Draft). 목데이터: 정해진 틀로 채운다.
	function orchDraft() {
		const d = editor;
		if (!d?.title.trim()) return toast.warning('제목을 먼저 적어 주세요');
		if (!d.body.trim()) d.body = `${d.title.trim()} — 범위 · 완료 기준을 정리했어요. 필요하면 고쳐 주세요.`;
		if (!d.criteria.length) d.criteria = [{ text: '동작 구현', done: false }, { text: '오류 · 빈 상태 처리', done: false }, { text: '테스트 통과', done: false }];
		d.agent ??= orchPick(d.title, d.labels);
		toast.success('Orch 초안을 채웠어요');
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
		const num = createTask(blank({ ...quick }));
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
	let inspectTab = $state('activity');
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
		if (!dec || qi < 0) return;
		const q = dec.questions[qi];
		const opt = q.options?.find((o) => o.key === pick);
		const text = [opt && `${opt.key} · ${opt.title}`, answer.trim()].filter(Boolean).join(' — ');
		if (!text) return;
		q.answer = text;
		(activity[dec.agent] ??= []).push({ type: 'DECISION', who: `나 → ${agentOf(dec.agent)?.name}`, time: now(), text: `Q ${q.q} → ${text}` });
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
<svelte:window
	onkeydown={(e) => {
		// 편집기 · 다이얼로그가 열려 있으면 그쪽이 키를 처리한다
		if (editor || panel) return;
		const typing = e.target instanceof HTMLElement && (e.target.closest('input, textarea, [contenteditable]') !== null);
		// 선택 창 · 메뉴 안에서 누른 Esc는 그 창만 닫는다
		const layer = e.target instanceof HTMLElement && e.target.closest('[data-slot$="-content"], [role="menu"], [role="listbox"], [cmdk-root]') !== null;
		if (e.key === 'Escape') {
			if (layer || e.defaultPrevented) return;
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

<!-- 속성 칩 (.pen TaskEditor · Properties · QuickAdd · Options) — 선택 창 트리거 -->
{#snippet propChip(props: Record<string, unknown>, Icon: Component, label: string, tone = 'text-muted-foreground')}
	<Button {...props} variant="outline" size="xs"><Icon class={tone} />{label}</Button>
{/snippet}
<!-- Task 상세 속성 한 줄 (.pen B3 · Properties). props가 있으면 선택 창 트리거, 없으면 표시만 -->
{#snippet propRow(props: Record<string, unknown> | undefined, label: string, Icon: Component, value: string, tone = 'text-muted-foreground')}
	<Item.Root variant="row" size="xs">
		{#snippet child({ props: row })}
			<svelte:element this={props ? 'button' : 'div'} {...mergeProps(row, props ?? {})}>
				<Item.Content><Item.Description>{label}</Item.Description></Item.Content>
				<Item.Actions class="min-w-0 text-xs font-medium"><Icon class={cn('size-3.25 shrink-0', tone)} /><span class="truncate">{value}</span>{#if props}<ChevronsUpDown class="size-3 text-muted-foreground" />{/if}</Item.Actions>
			</svelte:element>
		{/snippet}
	</Item.Root>
{/snippet}

<!-- Issue Board 행: 이슈와 하위 이슈가 같은 모양이라 재귀로 그린다 -->
{#snippet issueRow(i: Issue, depth: number)}
	{@const all = allTasks(i.num)}
	{@const done = all.filter((t) => t.status === 'done').length}
	{@const who = [...new Set(all.map((t) => t.agent).filter((a) => a !== undefined))].map((sn) => agentOf(sn)!)}
	{@const isOpen = expanded.has(i.num)}
	{@const icon = statuses[issueIcon[i.status]]}
	<Table.Row class={cn('h-11', depth === 0 && 'h-13 bg-card')}>
		<Table.Cell class={depth === 0 ? 'pl-4' : 'pl-10'}>
			<span class="flex items-center gap-3">
				<button type="button" aria-expanded={isOpen} aria-label="#{i.num} {isOpen ? '접기' : '펼치기'}" onclick={() => toggle(i.num)} class="rounded-xs text-muted-foreground hover:text-foreground">
					<ChevronRight class={cn('size-3.5 transition-transform', isOpen && 'rotate-90')} />
				</button>
				{#if depth === 0}
					<span class="kind-mark size-5 bg-node-issue"><CircleDot class="size-3" /></span>
				{:else}
					<span class="sub-issue-mark"><GitBranch class="size-2.75" /></span>
				{/if}
				<icon.icon class={cn('size-3.5', icon.text)} aria-label={icon.label} />
				<span class="font-mono text-xs font-medium text-muted-foreground">#{i.num}</span>
			</span>
		</Table.Cell>
		<Table.Cell class={depth === 0 ? 'font-semibold' : 'font-medium'}>
			<button type="button" aria-pressed={issueSel === i.num} onclick={() => openIssue(i.num)} class="text-left outline-none hover:underline focus-visible:underline">{i.title}</button>
		</Table.Cell>
		<Table.Cell>
			<span class="flex items-center gap-2">
				<Progress value={all.length ? (done / all.length) * 100 : 0} class="h-1.5" aria-label="#{i.num} 진행" />
				<span class="font-mono text-xs text-muted-foreground">{done}/{all.length}</span>
			</span>
		</Table.Cell>
		<Table.Cell>
			<span class="flex items-center gap-2">
				<Avatar.Group>
					{#each who as a (a.sn)}<RoleAvatar role={a.role} size="sm" />{/each}
				</Avatar.Group>
				<span class="truncate text-xs text-muted-foreground">{who.map((a) => a.name).join(' · ') || 'Unassigned'}</span>
			</span>
		</Table.Cell>
		<Table.Cell class="pr-4 text-right font-mono text-xs text-subtle-foreground">{i.updated}</Table.Cell>
	</Table.Row>
	{#if isOpen}
		{#each subIssues(i.num) as sub (sub.num)}
			{@render issueRow(sub, depth + 1)}
		{/each}
		{#each tasksOf(i.num) as t (t.num)}
			{@const a = agentOf(t.agent)}
			<Table.Row
				onclick={() => open(t.num)}
				class={cn('h-10 cursor-pointer', selected === t.num && 'bg-primary-soft hover:bg-primary-soft')}
			>
				<Table.Cell class={depth === 0 ? 'pl-14.5' : 'pl-21'}>
					<span class="flex items-center gap-3">
						<CornerDownRight class="size-3 text-subtle-foreground" />
						<StatusSelect bind:value={t.status} compact />
						<span class="font-mono text-xs text-muted-foreground">#{t.num}</span>
					</span>
				</Table.Cell>
				<Table.Cell>
					<button type="button" aria-pressed={selected === t.num} onclick={() => open(t.num)} class="text-left outline-none hover:underline focus-visible:underline">{t.title}</button>
				</Table.Cell>
				<Table.Cell>
					<span class="flex items-center gap-2">
						{#if t.steps[1]}
							<Progress value={(t.steps[0] / t.steps[1]) * 100} class="h-1.5" aria-label="#{t.num} 진행" />
							<span class="font-mono text-xs text-muted-foreground">{t.steps[0]}/{t.steps[1]}</span>
						{:else}
							<span class="font-mono text-xs text-subtle-foreground">no steps</span>
						{/if}
					</span>
				</Table.Cell>
				<Table.Cell>
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
				</Table.Cell>
				<Table.Cell class="pr-4 text-right font-mono text-xs text-subtle-foreground">{t.updated}</Table.Cell>
			</Table.Row>
		{/each}
	{/if}
{/snippet}

{#if project}
	<div class="flex h-full min-h-0">
		<!-- 좌측 Quick Panel (.pen Workbench/LeftQuickPanel) -->
		{#if leftOpen}
			<aside class="aside-col w-66 border-r bg-card" aria-label="Quick panel">
				<Tabs.Root bind:value={panelTab} class="flex min-h-0 flex-1 flex-col gap-0">
					<div class="panel-section gap-2.5">
						<div class="flex items-center gap-0.5">
							<Tabs.List>
								<Tabs.Trigger value="tasks">Tasks</Tabs.Trigger>
								<Tabs.Trigger value="agents">Agents</Tabs.Trigger>
							</Tabs.List>
							<span class="flex-1"></span>
							<Button variant="ghost" size="icon-sm" aria-label="새 태스크" aria-expanded={quick !== undefined} onclick={() => (quick ? (quick = undefined) : openQuick())}><Plus /></Button>
							<Button variant="ghost" size="icon-sm" aria-label="Quick panel 접기" onclick={() => (leftOpen = false)}><PanelLeftClose /></Button>
						</div>
						{#if panelTab === 'tasks'}
							<InputGroup.Root class="h-8">
								<InputGroup.Addon><Search /></InputGroup.Addon>
								<InputGroup.Input bind:value={query} placeholder="Filter tasks" aria-label="태스크 검색" />
							</InputGroup.Root>
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
							<InputGroup.Root class="h-8 border-ring">
								<InputGroup.Addon><Plus /></InputGroup.Addon>
								<!-- svelte-ignore a11y_autofocus -->
								<InputGroup.Input autofocus bind:value={q.title} placeholder="태스크 제목" aria-label="태스크 제목" onkeydown={(e) => e.key === 'Escape' && (quick = undefined)} />
								<InputGroup.Addon align="inline-end">
									<InputGroup.Button size="icon-xs" aria-label="편집기로 열기" title="편집기로 열기" onclick={() => newTask({ title: q.title, status: q.status, agent: q.agent, priority: q.priority })}><Maximize2 /></InputGroup.Button>
								</InputGroup.Addon>
							</InputGroup.Root>
							<div class="flex items-center gap-1">
								<StatusSelect bind:value={q.status}>
									{#snippet trigger(props)}{@const m = statuses[q.status]}{@render propChip(props, m.icon, m.label)}{/snippet}
								</StatusSelect>
								<AssigneePicker bind:value={q.agent} agents={agentList} tasks={list} recommend={recommendFor(q.title, [])}>
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

					<Tabs.Content value="tasks" class="min-h-0 flex-1 overflow-y-auto">
						{#each shown as t (t.num)}
							<Item.Root
								variant="row"
								size="sm"
								aria-pressed={selected === t.num}
								onclick={() => open(t.num)}
								class={cn('w-full px-3 text-left hover:bg-muted', selected === t.num && 'bg-primary-soft hover:bg-primary-soft')}
							>
								{#snippet child({ props })}
									<button type="button" {...props}>
										<Item.Content class="gap-1.5">
											<span class="flex items-center justify-between">
												<span class="font-mono text-xs font-medium text-muted-foreground">#{t.num}</span>
												<StatusBadge status={t.status} />
											</span>
											<Item.Title class="text-sm">{t.title}</Item.Title>
											<Item.Description class="flex items-center gap-1.5">
												{agentName(t.agent) ?? '미배정'}
												<span class={cn('font-mono font-semibold', t.priority === 'P0' ? 'text-destructive' : 'text-subtle-foreground')}>{t.priority}</span>
											</Item.Description>
										</Item.Content>
									</button>
								{/snippet}
							</Item.Root>
						{:else}
							<p class="p-6 text-center text-xs text-muted-foreground">조건에 맞는 태스크가 없어요.</p>
						{/each}
					</Tabs.Content>

					<Tabs.Content value="agents" class="min-h-0 flex-1 overflow-y-auto">
						{#each agentList as a (a.sn)}
							<button type="button" aria-pressed={inspect === a.sn} onclick={() => openAgent(a.sn)} class={cn('agent-row hover:bg-muted', inspect === a.sn && 'bg-primary-soft hover:bg-primary-soft')}>
								<RoleAvatar role={a.role}>
									<Avatar.Badge class={a.online ? 'bg-success' : 'bg-subtle-foreground'} aria-label={a.online ? '온라인' : '오프라인'} />
								</RoleAvatar>
								<span class="row-text">
									<span class="flex items-center gap-1.5">
										<span class="text-sm font-medium">{a.name}</span>
										<span class="font-mono text-xs text-muted-foreground">{a.runtime === 'claude' ? 'Claude Code' : 'Codex CLI'}</span>
									</span>
									<span class="truncate text-xs text-muted-foreground">{a.activity}</span>
								</span>
								<span class="font-mono text-xs font-medium text-muted-foreground">{a.tokens}</span>
							</button>
						{/each}
					</Tabs.Content>
				</Tabs.Root>
			</aside>
		{/if}

		<main class="flex min-w-0 flex-1 flex-col">
			<!-- 뷰 머리글 (.pen Workbench/ViewHeader) -->
			<header class="view-header">
				{#if !leftOpen}
					<Button variant="ghost" size="icon-sm" aria-label="Quick panel 펼치기" onclick={() => (leftOpen = true)}><PanelLeftOpen /></Button>
				{/if}
				<Tabs.Root value={view.value} onValueChange={setView}>
					<Tabs.List aria-label="보기">
						{#each views as v (v.value)}
							<Tabs.Trigger value={v.value}><v.icon />{v.label}</Tabs.Trigger>
						{/each}
					</Tabs.List>
				</Tabs.Root>
				<p class="view-summary">
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
				{#if !dockOpen}
					<Button variant="ghost" size="icon-sm" aria-label="PM Dock 펼치기" onclick={() => (dockOpen = true)}><PanelRightOpen /></Button>
				{/if}
			</header>

			<!-- 뷰 (#54 Kanban · #55 Issues · #56 Diagram) -->
			<div class="relative min-h-0 flex-1">
			<div class="h-full overflow-auto bg-canvas">
				{#if view.value === 'kanban'}
					<!-- ui/kanban 조립: 열 = 상태, 항목 = 태스크 번호. 놓으면 drop()이 상태를 command로 반영 -->
					<Kanban.Root bind:value={board} onDragEnd={drop}>
						{#each lanes as s (s)}
							{@const meta = statuses[s]}
							<Kanban.Column value={s}>
								<Kanban.ColumnHeader>
									<meta.icon class={meta.text} />
									<Kanban.ColumnTitle>{meta.label}</Kanban.ColumnTitle>
									<Kanban.ColumnCount />
									<Kanban.ColumnActions>
										<Button variant="ghost" size="icon-xs" aria-label="{meta.label}에 태스크 추가" onclick={() => openQuick(s)}><Plus /></Button>
																<!-- 열 메뉴 — .pen에 항목이 없어 있는 동작만 (#60) -->
																<DropdownMenu.Root>
																	<DropdownMenu.Trigger>{#snippet child({ props })}<Button {...props} variant="ghost" size="icon-xs" aria-label="{meta.label} 열 메뉴"><Ellipsis /></Button>{/snippet}</DropdownMenu.Trigger>
																	<DropdownMenu.Content align="end" class="w-48">
																		<DropdownMenu.Item onSelect={() => newTask({ status: s })}><SquarePen />{meta.label}로 새 태스크</DropdownMenu.Item>
																		<DropdownMenu.Item onSelect={() => ((leftOpen = true), (panelTab = 'tasks'), (filter = s))}><FilterIcon />Quick Panel에서 이 상태만</DropdownMenu.Item>
																	</DropdownMenu.Content>
																</DropdownMenu.Root>
									</Kanban.ColumnActions>
								</Kanban.ColumnHeader>
								<Kanban.ColumnContent>
									{#each board[s] ?? [] as num (num)}
										{@const t = task(num as number)}
										{@const a = agentOf(t.agent)}
										<!-- 우클릭: ContextMenu / Task (Diagram과 같은 목록) -->
										<ContextMenu.Root onOpenChange={(o) => o && hoverOff()}>
											<ContextMenu.Trigger>
												{#snippet child({ props })}
													<Kanban.Item
														{...props}
														value={num}
														aria-pressed={selected === t.num}
														onclick={() => (hoverOff(), open(t.num))}
														onpointermove={(e) => hoverAt(t.num, e)}
														onpointerleave={hoverOff}
														onpointerdown={hoverOff}
													>
														<Kanban.ItemHeader>
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

														</Kanban.ItemHeader>
														<Kanban.ItemContent>
															<span class="title-sm gap-2">
																<SquareCheck class="size-3.5 shrink-0 text-node-task" />#{t.num} · {t.title}
															</span>
															<span class="meta-xs gap-2">
																<CircleDot class="size-3.5 shrink-0 text-node-issue" />Issue #{t.issue} · {issueOf(t.issue)?.title}
															</span>
															<span class="flex items-center gap-2 pt-1.5">
																{#if t.steps[1]}
																	<Progress value={(t.steps[0] / t.steps[1]) * 100} class="h-1.5" aria-label="완료 조건 진행" />
																	<span class="kanban-card-steps">
																		{t.steps[0]}/{t.steps[1]} · {Math.round((t.steps[0] / t.steps[1]) * 100)}%
																	</span>
																{:else}
																	<span class="font-mono text-xs text-subtle-foreground">no steps</span>
																{/if}
															</span>

														</Kanban.ItemContent>
														<Kanban.ItemFooter>
															<span class={cn('inline-flex items-center gap-1', t.over && 'text-warning')}><Coins class="size-3" />{t.tokens ?? '—'}{t.over ? ' ⚠' : ''}</span>
															<span class="inline-flex items-center gap-1"><MessageSquare class="size-3" />{t.messages}</span>
															{#if t.model}<Badge variant="mono" class="text-2xs"><Cpu />{t.model}</Badge>{/if}
															<span class="ml-auto inline-flex items-center gap-1"><Timer class="size-3" />{t.run ? `Run ${t.run}` : '—'}</span>

														</Kanban.ItemFooter>
													</Kanban.Item>
												{/snippet}
											</ContextMenu.Trigger>
											<ContextMenu.Content class="w-56">
												<ContextMenu.Label class="truncate">Task #{t.num} · {t.title}</ContextMenu.Label>
												<ContextMenu.Separator />
												{#each taskMenu(t.num) as m, k (k)}
													{#if m === 'sep'}
														<ContextMenu.Separator />
													{:else if m.sub}
														<ContextMenu.Sub>
															<ContextMenu.SubTrigger disabled={m.disabled}>{#if m.icon}<m.icon class="text-muted-foreground" />{/if}{m.label}</ContextMenu.SubTrigger>
															<ContextMenu.SubContent class="w-48">
																{#each m.sub as x (x.label)}
																	<ContextMenu.Item onSelect={x.onSelect}>{#if x.icon}<x.icon class={x.tone} />{/if}<span class="flex-1">{x.label}</span>{#if x.checked}<Check />{/if}</ContextMenu.Item>
																{/each}
															</ContextMenu.SubContent>
														</ContextMenu.Sub>
													{:else}
														<ContextMenu.Item disabled={m.disabled} onSelect={m.onSelect}>
															{#if m.icon}<m.icon class="text-muted-foreground" />{/if}{m.label}
															{#if m.shortcut}<ContextMenu.Shortcut>{m.shortcut}</ContextMenu.Shortcut>{/if}
														</ContextMenu.Item>
													{/if}
												{/each}
											</ContextMenu.Content>
										</ContextMenu.Root>
									{/each}
									{#snippet empty()}<p class="empty-note text-center text-subtle-foreground">비어 있어요</p>{/snippet}
								</Kanban.ColumnContent>
							</Kanban.Column>
						{/each}
					</Kanban.Root>
					{#if hover}
						{@const t = task(hover.num)}
						{@const d = details[hover.num]}
						{@const next = d?.criteria.find((c) => !c.done)}
						{@const blocks = (d?.deps ?? []).filter((x) => x.kind === 'blocks').map((x) => task(x.num)).filter(Boolean)}
						{@const last = d?.activity.at(-1)}
						{@const ctx = d?.context}
						{@const at = { x: hover.x, y: hover.y }}
						<!-- .pen KanbanCard/HoverPreview — ui/hover-card를 커서 위치에 띄운다 (+18px, 화면 끝에선 floating-ui가 뒤집음) -->
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
									{#each [['현재 단계', t.steps[1] ? `${t.steps[0]}/${t.steps[1]}${next ? ` · ${next.text}` : d?.criteria.length ? ' · 모두 완료' : ''}` : '—'], ['최근 활동', last ? `${last.type.toLowerCase()} ${last.text}` : '—'], ['막고 있는 Task', blocks.length ? blocks.map((b) => `#${b.num} ${b.title} · ${agentName(b.agent) ?? '미배정'}`).join(', ') : '—'], ['완료 시 전달', blocks[0] ? `${agentName(blocks[0].agent) ?? '미배정'} · ${roles[agentOf(blocks[0].agent)?.role ?? 'agent'].label} (REQUEST_VERIFICATION)` : '—'], ['ETA', d?.eta || '—']] as [k, v] (k)}
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
				{:else if view.value === 'diagram'}
					<SvelteFlow
						bind:nodes
						bind:edges
						{nodeTypes}
						initialViewport={{ x: 24, y: 24, zoom: 0.85 }}
						minZoom={0.3}
						nodesConnectable={false}
						onnodeclick={({ node }) => {
							if (node.id.startsWith('task-')) open(Number(node.id.slice(5)));
							else if (node.id.startsWith('issue-')) openIssue(Number(node.id.slice(6)));
							else if (node.id.startsWith('agent-')) openAgent(Number(node.id.slice(6)));
							else if (node.id.startsWith('sub-')) openSub(node.id.slice(4));
						}}
						class="bg-canvas"
					>
						<!-- canvas-grid 토큰은 canvas 배경과 거의 같아 점이 안 보인다 → 한 단계 진한 input 색 -->
						<Background variant={BackgroundVariant.Dots} gap={20} size={1.5} patternColor="var(--input)" />
						<Controls position="bottom-right" showLock={false} />
						<Panel position="bottom-left">
							<div class="diagram-legend" aria-label="연결선 범례">
								{#each [['contains', 'bg-input', 'h-0.5'], ['delegate', 'bg-primary', 'h-0.5'], ['assigned', 'bg-node-agent', 'h-0.5'], ['interaction (idle)', 'bg-status-review', 'h-0.5'], ['live event', 'bg-primary', 'h-0.75'], ['spawn', 'bg-node-agent', 'h-0.5'], ['queued · waits', 'bg-subtle-foreground', 'h-0.5']] as [l, bg, h] (l)}
									<span class="flex items-center gap-1.5"><span class={cn('w-3.5 rounded-full', bg, h)}></span>{l}</span>
								{/each}
							</div>
						</Panel>
					</SvelteFlow>
				{:else if view.value === 'issues'}
					<Table.Root class="bg-background">
						<Table.Header class="bg-muted">
							<Table.Row>
								<Table.Head class="w-40 pl-4">Issue</Table.Head>
								<Table.Head>Title</Table.Head>
								<Table.Head class="w-44">Progress</Table.Head>
								<Table.Head class="w-75">Agents</Table.Head>
								<Table.Head class="w-20 pr-4 text-right">Updated</Table.Head>
							</Table.Row>
						</Table.Header>
						<Table.Body>
							{#each issueList.filter((i) => !i.parent) as i (i.num)}
								{@render issueRow(i, 0)}
							{/each}
						</Table.Body>
					</Table.Root>
				{:else}
				<Empty.Root class="h-full">
					<Empty.Header>
						<Empty.Media variant="icon"><view.icon /></Empty.Media>
						<Empty.Title>{view.label}</Empty.Title>
						<Empty.Description>{view.issue}에서 구현해요.</Empty.Description>
					</Empty.Header>
				</Empty.Root>
				{/if}
			</div>
			{#if cur}
				{@const a = agentOf(cur.agent)}
				{@const pm = priorities[cur.priority]}
				{@const deps = info?.deps ?? []}
				{@const iss = issueOf(cur.issue)}
				<!-- 태스크 상세: 뷰 위 scrim + 패널. Esc · 닫기 버튼으로 닫는다 -->
				<div class="scrim">
					<Inspector.Root label="Task #{cur.num} 상세">
						<Inspector.Header onclose={() => (detail = false)} closeLabel="상세 닫기">
								<span class="kind-tile size-8 bg-node-task"><SquareCheck class="size-4" /></span>
								<div class="row-text">
									<p class="truncate text-xs text-muted-foreground">{project.name} / Issue #{cur.issue} {iss?.title} / <span class="font-mono">TASK #{cur.num}</span></p>
									<h2 class="text-lg font-semibold">{cur.title}</h2>
								</div>
								<Button variant="ghost" size="icon-sm" aria-label="편집 (E)" title="편집 (E)" onclick={() => editTask(cur.num)}><Pencil /></Button>
								<Button variant="ghost" size="sm" onclick={() => navigator.clipboard?.writeText(`${page.url.origin}${page.url.pathname}?task=${cur.num}`)}><Link2 />Copy link</Button>
							{#snippet sub()}
							<div class="detail-meta">
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
								<Alert.Root variant="warning" class="flex items-center gap-2.5">
									<MessageCircleQuestion />
									<div class="flex-1">
										<Alert.Title>{info.decision.question}</Alert.Title>
										<Alert.Description>{info.decision.note}</Alert.Description>
									</div>
									<Badge class="rounded-xs bg-status-waiting font-mono text-2xs text-on-solid">{info.decision.level}</Badge>
									<Button size="sm" onclick={() => openDecisions(cur.num)}>답변하기</Button>
								</Alert.Root>
							{/if}
							{/snippet}
						</Inspector.Header>
						<Inspector.Body class="flex-row">
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
												<button type="button" class="dep-link" onclick={() => open(d.num)}>#{d.num} {dt.title} · {agentOf(dt.agent)?.name ?? '미배정'}</button>
												<StatusBadge status={dt.status} />
											</div>
										{/each}
									</section>
								{/if}
								{#if runs.length}
									<section class="flex flex-col">
										<h3 class="mb-1 text-body font-semibold">Runs</h3>
										{#each runs as r (r.num)}
											<Item.Root variant="row" size="xs">
												<Item.Content>
													<Item.Title>Run #{r.num} {#if r.live}<span class="size-1.5 rounded-full bg-success" aria-label="진행 중"></span>{/if}</Item.Title>
													<Item.Description>{r.note}</Item.Description>
												</Item.Content>
												<Item.Actions class="font-mono text-xs text-muted-foreground">{r.time} · {r.tokens}</Item.Actions>
											</Item.Root>
										{/each}
									</section>
								{/if}
								{#if leadRuns[cur.num]}
									{@const subs = subRuns.filter((s) => s.task === cur.num)}
									<!-- Run 토큰 (.pen Run 상세 · 토큰) — Run마다 따로 쌓고 여기서만 합친다. runner는 리드와 따로, sub · fork는 리드 사용량에 포함 -->
									<section class="flex flex-col">
										<h3 class="mb-1 text-body font-semibold">Run 토큰</h3>
										<Item.Root variant="row" size="xs">
											<Item.Content><Item.Title><Bot class="size-3.5 text-muted-foreground" />리드 Run #{leadRuns[cur.num].run} · {agentName(a?.sn)}</Item.Title></Item.Content>
											<Item.Actions><TokenMeter self={leadRuns[cur.num].self} runner={runnerTotal(cur.num)} /></Item.Actions>
										</Item.Root>
										{#each subs as s (s.id)}
											{#if s.mode === 'runner'}
												{#each s.runs as r, i (r.num)}
													<Item.Root variant="row" size="xs">
														<Item.Content><Item.Title><Cpu class="size-3.5 text-muted-foreground" />Run #{r.num} · {s.id} runner{i < s.runs.length - 1 ? ' · 실패(재시도 전)' : ''}</Item.Title></Item.Content>
														<Item.Actions><TokenMeter self={r.tokens} /></Item.Actions>
													</Item.Root>
												{:else}
													<Item.Root variant="row" size="xs">
														<Item.Content><Item.Title><Cpu class="size-3.5 text-muted-foreground" />{s.id} runner · 대기</Item.Title></Item.Content>
														<Item.Actions><TokenMeter self={0} /></Item.Actions>
													</Item.Root>
												{/each}
											{:else}
												{@const M = subRunMode[s.mode].icon}
												<Item.Root variant="row" size="xs">
													<Item.Content><Item.Title><M class="size-3.5 text-muted-foreground" />{s.id} {s.mode}</Item.Title></Item.Content>
													<Item.Actions><TokenMeter included /></Item.Actions>
												</Item.Root>
											{/if}
										{/each}
										<p class="pt-1.5 text-caption text-muted-foreground">runner는 리드와 따로 보이고 합계에만 더해요. sub · fork는 리드 Run 안에서 돌아 리드 사용량에 이미 들어 있어요.</p>
									</section>
								{/if}
								{#if info?.activity.length}
									<section class="flex flex-col">
										<h3 class="mb-1 text-body font-semibold">Activity</h3>
										{#each info.activity as ev, i (i)}
											<Item.Root variant="row" size="xs">
												<Item.Content>
													<Item.Title><Badge variant="mono" class="text-2xs">{ev.type}</Badge><span class="text-xs font-normal text-muted-foreground">{ev.who}</span></Item.Title>
													<Item.Description>{ev.text}</Item.Description>
												</Item.Content>
												<Item.Actions class="font-mono text-xs text-subtle-foreground">{ev.time}</Item.Actions>
											</Item.Root>
										{/each}
									</section>
								{/if}
							</div>
							<aside class="detail-aside" aria-label="속성">
								<section class="flex flex-col">
									<h3 class="mb-1 text-body font-semibold">Properties</h3>
									<!-- .pen B · Properties hover / click: 담당 · 우선순위 · 의존은 선택 창(C), 나머지는 표시 -->
									{@render propRow(undefined, 'Issue', CircleDotIcon, `#${cur.issue} ${iss?.title ?? ''}`, 'text-node-issue')}
									<AssigneePicker bind:value={() => cur.agent, (v) => (task(cur.num).agent = v)} agents={agentList} tasks={list} recommend={recommendFor(cur.title, info?.labels ?? [])} onorch={() => (task(cur.num).agent = orchPick(cur.title, info?.labels ?? []))}>
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
										<Alert.Root variant="warning">
											<GitFork />
											<Alert.Title>fork는 부모 컨텍스트를 상속해요</Alert.Title>
											<Alert.Description>부모 컨텍스트를 그대로 복사해 비용이 커요.</Alert.Description>
										</Alert.Root>
									{/if}
									<p class="text-caption text-muted-foreground">바꾸면 이 태스크에만 저장돼요. 비워 두면 팀 기본값을 써요 (Teams › Orch 진행 정책).</p>
								</section>
								{#if info?.context}
									<section class="flex flex-col gap-2">
										<h3 class="text-body font-semibold">Tokens · this task</h3>
										<div class="flex items-center justify-between text-xs"><span class="text-muted-foreground">Run context</span><span class="font-mono">{info.context[0]}K / {info.context[1]}K</span></div>
										<Progress value={(info.context[0] / info.context[1]) * 100} class="h-1.5" aria-label="컨텍스트 사용" />
									</section>
								{/if}
								{#if info?.git}
									<section class="flex flex-col">
										<h3 class="mb-1 text-body font-semibold">Git</h3>
										{#each [['Branch', info.git.branch], ['Commits', info.git.commits], ['PR', info.git.pr ?? '—']] as [k, v] (k)}
											<Item.Root variant="row" size="xs">
												<Item.Content><Item.Description>{k}</Item.Description></Item.Content>
												<Item.Actions class="font-mono text-xs">{v}</Item.Actions>
											</Item.Root>
										{/each}
									</section>
								{/if}
							</aside>
						</Inspector.Body>
					</Inspector.Root>
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
					<Inspector.Root label="Issue #{iss.num} 상세">
						<Inspector.Header onclose={() => (issueSel = undefined)} closeLabel="상세 닫기">
								<span class="kind-tile size-8 bg-node-issue"><CircleDot class="size-4" /></span>
								<div class="row-text">
									<p class="truncate text-xs text-muted-foreground">
										{project?.name}{' / '}{#if parent}<button type="button" class="hover:underline" onclick={() => openIssue(parent.num)}>Issue #{parent.num} {parent.title}</button>{' / '}{/if}<span class="font-mono">ISSUE #{iss.num}</span>
									</p>
									<h2 class="text-lg font-semibold">{iss.title}</h2>
								</div>
								<Button variant="ghost" size="sm" onclick={() => navigator.clipboard?.writeText(`${page.url.origin}${page.url.pathname}?issue=${iss.num}`)}><Link2 />Copy link</Button>
							{#snippet sub()}
							<div class="detail-meta-wide">
								<span class={cn('label-xs', icon.text)}><icon.icon class="size-3.5" />{issueLabel[iss.status]}</span>
								<span class="flex w-40 items-center gap-2">
									<Progress value={all.length ? (done / all.length) * 100 : 0} class="h-1.5" aria-label="#{iss.num} 진행" />
									<span class="font-mono text-xs text-muted-foreground">{done}/{all.length}</span>
								</span>
								{#if who.length}
									<span class="flex items-center gap-1.5 text-xs">
										<Avatar.Group>
											{#each who as a (a.sn)}<RoleAvatar role={a.role} size="sm" />{/each}
										</Avatar.Group>
										<span class="text-muted-foreground">{who.map((a) => a.name).join(' · ')}</span>
									</span>
								{/if}
								<span class="flex-1"></span>
								<Button variant="ghost" size="sm" onclick={() => askPm(`Issue #${iss.num}`)}><MessageCircleQuestion />Ask PM about this</Button>
							</div>
							{/snippet}
						</Inspector.Header>
						<Inspector.Body class="flex-row">
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
											<Item.Root variant="row" size="xs" onclick={() => openIssue(sub.num)}>
												<Item.Content>
													<Item.Title><si.icon class={cn('size-3.5', si.text)} aria-label={issueLabel[sub.status]} /><span class="font-mono text-xs text-muted-foreground">#{sub.num}</span>{sub.title}</Item.Title>
												</Item.Content>
												<Item.Actions class="font-mono text-xs text-muted-foreground">{st.filter((t) => t.status === 'done').length}/{st.length}</Item.Actions>
											</Item.Root>
										{/each}
									</section>
								{/if}
								<section class="flex flex-col">
									<h3 class="mb-1 text-body font-semibold">Tasks</h3>
									{#each tasksOf(iss.num) as t (t.num)}
										{@const a = agentOf(t.agent)}
										<Item.Root variant="row" size="xs" onclick={() => open(t.num)}>
											<Item.Content>
												<Item.Title><span class="font-mono text-xs text-muted-foreground">#{t.num}</span>{t.title}</Item.Title>
											</Item.Content>
											<Item.Actions class="gap-2 text-xs text-muted-foreground">
												{#if a}<RoleAvatar role={a.role} size="sm" />{a.name}{:else}Unassigned{/if}
												<StatusBadge status={t.status} />
											</Item.Actions>
										</Item.Root>
									{:else}
										<p class="text-xs text-muted-foreground">이 이슈에 직접 속한 태스크가 없어요.</p>
									{/each}
								</section>
							</div>
							<aside class="detail-aside" aria-label="속성">
								<section class="flex flex-col">
									<h3 class="mb-1 text-body font-semibold">Properties</h3>
									{#each [['Parent', parent ? `#${parent.num} ${parent.title}` : '—'], ['Status', issueLabel[iss.status]], ['Tasks', `${done} / ${all.length} done`], ['Assignees', who.map((a) => a.name).join(' · ') || 'Unassigned'], ['Updated', iss.updated], ['Labels', iss.labels.join(' · ') || '—']] as [k, v] (k)}
										<Item.Root variant="row" size="xs">
											<Item.Content><Item.Description>{k}</Item.Description></Item.Content>
											<Item.Actions class="min-w-0 truncate text-xs font-medium">{v}</Item.Actions>
										</Item.Root>
									{/each}
								</section>
							</aside>
						</Inspector.Body>
					</Inspector.Root>
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
				<Inspector.Root label="하위 작업 {s.id}" floating>
					<Inspector.Header onclose={() => (subSel = undefined)} closeLabel="하위 작업 카드 닫기">
							<span class={cn('subrun-mark', md.tile)}><md.icon class="size-4" /></span>
							<div class="row-text">
								<span class="meta-line gap-1.5 font-mono font-semibold">
									{s.mode.toUpperCase()} · {s.id}
									{#if s.tier}<span class={cn('mono-tag flex items-center gap-0.5', tierTone[s.tier])}><Cpu class="size-2.5" />{s.tier}</span>{/if}
								</span>
								<span class="text-sm font-semibold">{s.goal}</span>
							</div>
						{#snippet sub()}
						<div class="meta-line gap-2">
							<span class={cn('chip-round gap-1 font-medium', st.tone)}><st.icon class={cn('size-3', s.status === 'running' && 'animate-spin motion-reduce:animate-none')} />{st.label}</span>
							<span class="flex-1 truncate">{agentName(s.lead)}이 spawn · #{s.task} {lt?.title}</span>
							<Button variant="ghost" size="icon-sm" class="bg-destructive-soft text-destructive" aria-label="하위 작업 중지" disabled={s.status !== 'running'}><Square /></Button>
						</div>
						{/snippet}
					</Inspector.Header>
					<Inspector.Body>
						<Inspector.Section title="SUB-RUN">
							<Inspector.ValueRow icon={Split} label="mode" value={{ runner: 'runner · 별도 Run', sub: 'sub · 리드 Run 안', fork: 'fork · 부모 컨텍스트 상속' }[s.mode]} />
							<Inspector.ValueRow icon={Play} label="시작" value={`리드 Run #${s.leadRun}`} />
							{#if s.retry}
								<Inspector.ValueRow icon={RotateCcw} label="재시도">
									<span class="flex items-center gap-1 font-medium">Run #{s.runs[0].num}<ArrowRight class="size-3" /><span class="text-muted-foreground">등급</span>
										<span class={cn('mono-tag', tierTone[s.retry.from])}>{s.retry.from}</span><ArrowRight class="size-3" /><span class={cn('mono-tag', tierTone[s.retry.to])}>{s.retry.to}</span>
									</span>
								</Inspector.ValueRow>
							{/if}
							{#if s.tier}<Inspector.ValueRow icon={Gauge} label="tier" value={`${s.tier} · ${s.retry ? `재시도로 ${s.retry.from}에서 올림` : `kind ${s.kind} → 규칙 엔진`}`} />{/if}
							{#if s.model}<Inspector.ValueRow icon={Cpu} label="model" value={s.model} />{/if}
							<Inspector.ValueRow icon={Timer} label="소요" value={s.status === 'queued' ? '대기 중' : `${s.minutes}m`} />
						</Inspector.Section>
						<Inspector.Section title={pathsTitle}>
							{#if bad.length}
								<Alert.Root variant="destructive" class="mb-1">
									<OctagonX />
									<Alert.Title>paths 밖 변경 · {bad.map((p) => p.path).join(', ')}</Alert.Title>
									<Alert.Description>허용된 paths 밖 파일을 고쳐 실패로 처리됐어요. 변경은 적용되지 않았어요.</Alert.Description>
								</Alert.Root>
							{/if}
							{#each s.paths as p (p.path)}
								{@const Icon = p.from === 'ask' ? FilePlus : p.from === 'bad' ? FileX : FileCode}
								<div class="flex h-7 items-center gap-2">
									<span class={cn('kind-mark size-5 shrink-0', p.from === 'ask' ? 'bg-primary' : p.from === 'bad' ? 'bg-destructive' : 'bg-muted-foreground')}><Icon class="size-3" /></span>
									<span class={cn('flex-1 truncate font-mono', p.from === 'bad' && 'text-destructive')}>{p.path}</span>
									<span class={cn('text-caption', p.from === 'ask' ? 'text-primary' : p.from === 'bad' ? 'text-destructive' : 'text-muted-foreground')}>{p.from === 'ask' ? `@ASK · ${p.at}` : p.from === 'bad' ? 'paths 밖' : '처음'}</span>
								</div>
							{:else}
								<p class="text-muted-foreground">파일을 고치지 않는 작업이에요 (조사 · 탐색).</p>
							{/each}
							<p class="pt-1 text-caption text-muted-foreground">
								{#if s.waits}paths가 겹쳐 {s.waits.id}가 끝날 때까지 대기해요 ({s.waits.glob}).
								{:else if bad.length}재시도하면 등급이 한 단계 올라가요. 경로가 더 필요하면 @ASK로 요청해요.
								{:else}paths 밖 파일을 고치면 실패로 처리돼요. 겹치는 경로의 하위 작업은 순서대로 실행돼요.{/if}
							</p>
						</Inspector.Section>
						<Inspector.Section title={`@REPORT · ac ${s.ac.filter((a) => a.ok).length}/${s.ac.length}`}>
							{#each s.ac as a (a.text)}
								<span class="flex items-center gap-2 py-0.5"><Checkbox checked={a.ok} disabled aria-label={a.text} />{a.text}</span>
							{/each}
							{#if s.report}<p class="report-note">{s.report}</p>{/if}
						</Inspector.Section>
						<Inspector.Section title="TOKENS">
							<Inspector.ValueRow icon={Coins} label="이 Run">
								{#if s.mode !== 'runner'}<TokenMeter included />{:else if cur}<TokenMeter self={cur.tokens} class="text-foreground" />{:else}<span class="text-muted-foreground">시작 전</span>{/if}
							</Inspector.ValueRow>
							{#if s.runs.length > 1}
								<Inspector.ValueRow icon={RotateCcw} label="재시도 전 Run #{s.runs[0].num}"><TokenMeter self={s.runs[0].tokens} /></Inspector.ValueRow>
							{/if}
							<p class="pt-1 text-caption text-muted-foreground">{s.mode === 'runner' ? '리드와 따로 쌓이고 리드 합계에만 더해져요.' : '리드 Run 안에서 돌아 리드 사용량에 이미 들어 있어요.'}</p>
						</Inspector.Section>
						<Inspector.Section title="WORKTREE · {s.workdir.mode === 'repo' ? 'repo 모드' : 'worktree'}">
							<Inspector.ValueRow icon={GitBranch} label="모드" value={s.workdir.mode === 'repo' ? 'repo 모드 · worktree 없음' : 'worktree'} />
							<Inspector.ValueRow icon={FolderGit2} label="경로" value={s.workdir.path ?? '저장소 그대로 · orchstack/app'} />
							<Inspector.ValueRow icon={GitBranch} label="브랜치" value={s.workdir.branch} />
							{#if s.workdir.mode === 'worktree'}<Inspector.ValueRow icon={Trash2} label="정리" value={s.status === 'failed' ? '실패 · 24시간 보관 후 삭제' : '완료 후 병합 · 삭제'} />{/if}
						</Inspector.Section>
					</Inspector.Body>
				</Inspector.Root>
			{/if}
			{#if agentSel}
				{@const a = agentSel}
				{@const mine = list.filter((t) => t.agent === a.sn)}
				<!-- 에이전트 카드: 뷰 오른쪽에 뜬다 (.pen Agent Inspector Card, 340px) -->
				<Inspector.Root label="{a.name} 에이전트" floating>
					<Inspector.Header onclose={() => (inspect = undefined)} closeLabel="에이전트 카드 닫기">
							<RoleAvatar role={a.role} />
							<div class="row-text">
								<span class="title-sm gap-1.5">{a.name}<RuntimeLogo runtime={a.runtime} class="size-3.5 ring-0" /></span>
								<span class="text-xs text-muted-foreground">{roles[a.role].label}</span>
								<Badge variant="mono" class="w-fit text-2xs">{a.runtime === 'claude' ? 'Claude Code' : 'Codex CLI'}</Badge>
							</div>
						{#snippet sub()}
						<div class="flex items-center gap-2">
							<Badge variant="secondary">{a.activity.split(' · ')[0]}</Badge>
							<span class="flex-1"></span>
							<Button variant="ghost" size="icon-sm" class="bg-warning-soft text-warning" aria-label="일시정지 · 현재 단계 끝나면 멈춤" title="일시정지 · 현재 단계 끝나면 멈춤"><Pause /></Button>
							<Button variant="ghost" size="icon-sm" class="bg-destructive-soft text-destructive" aria-label="중지"><Square /></Button>
						</div>
						{/snippet}
					</Inspector.Header>
					<Tabs.Root bind:value={inspectTab} class="flex min-h-0 flex-1 flex-col gap-0">
						<Tabs.List variant="line" class="w-full justify-start px-4">
							{#each [['overview', 'Overview'], ['activity', 'Activity'], ['runs', 'Runs'], ['config', 'Config']] as [v, l] (v)}<Tabs.Trigger value={v}>{l}</Tabs.Trigger>{/each}
						</Tabs.List>
						<Tabs.Content value="overview" class="flex min-h-0 flex-1 flex-col overflow-y-auto px-4 py-3">
							<h3 class="mb-1 text-body font-semibold">맡은 태스크</h3>
							{#each mine as t (t.num)}
								<Item.Root variant="row" size="xs" onclick={() => open(t.num)}>
									{#snippet child({ props })}
										<button type="button" {...props}>
											<Item.Content><Item.Title>#{t.num} {t.title}</Item.Title></Item.Content>
											<Item.Actions><StatusBadge status={t.status} /></Item.Actions>
										</button>
									{/snippet}
								</Item.Root>
							{:else}
								<p class="text-xs text-muted-foreground">맡은 태스크가 없어요.</p>
							{/each}
							<div class="inspector-foot"><span class="text-muted-foreground">토큰 (오늘)</span><span class="font-mono">{a.tokens}</span></div>
						</Tabs.Content>
						<Tabs.Content value="activity" class="min-h-0 flex-1 overflow-y-auto px-4 py-2">
							{#each activity[a.sn] ?? [] as ev, i (i)}
								<Item.Root variant="row" size="xs">
									<Item.Content>
										<Item.Title><Badge variant="mono" class="text-2xs">{ev.type}</Badge><span class="text-xs font-normal text-muted-foreground">{ev.who}</span></Item.Title>
										<Item.Description>{ev.text}</Item.Description>
									</Item.Content>
									<Item.Actions class="font-mono text-xs text-subtle-foreground">{ev.time}</Item.Actions>
								</Item.Root>
							{:else}
								<p class="py-4 text-center text-xs text-muted-foreground">활동 기록이 없어요.</p>
							{/each}
						</Tabs.Content>
						<Tabs.Content value="runs" class="min-h-0 flex-1 overflow-y-auto px-4 py-2">
							{#each mine.flatMap((t) => (details[t.num]?.runs ?? []).map((r) => ({ ...r, task: t.num }))) as r (r.num)}
								<Item.Root variant="row" size="xs">
									<Item.Content>
										<Item.Title>Run #{r.num} · #{r.task}</Item.Title>
										<Item.Description>{r.note}</Item.Description>
									</Item.Content>
									<Item.Actions class="font-mono text-xs text-muted-foreground">{r.time} · {r.tokens}</Item.Actions>
								</Item.Root>
							{:else}
								<p class="py-4 text-center text-xs text-muted-foreground">Run 기록이 없어요.</p>
							{/each}
						</Tabs.Content>
						<Tabs.Content value="config" class="min-h-0 flex-1 overflow-y-auto px-4 py-2">
							{#each [['Role', roles[a.role].label], ['Runtime', a.runtime === 'claude' ? 'Claude Code' : 'Codex CLI'], ['Status', a.online ? 'Online' : 'Offline']] as [k, v] (k)}
								<Item.Root variant="row" size="xs">
									<Item.Content><Item.Description>{k}</Item.Description></Item.Content>
									<Item.Actions class="text-xs font-medium">{v}</Item.Actions>
								</Item.Root>
							{/each}
							<p class="mt-3 text-xs text-muted-foreground">스킬 · 도구 · 권한 편집은 Teams › 멤버 상세(#27)에서.</p>
						</Tabs.Content>
					</Tabs.Root>
					<form class="border-t p-3" onsubmit={instruct}>
						<div class="inspector-composer">
							<input bind:value={instruction} placeholder="{a.name}에게 실행 중 지시…" aria-label="{a.name}에게 실행 중 지시" class="bare-input" />
							<Button type="submit" size="icon-sm" aria-label="지시 보내기" disabled={!instruction.trim()}><ArrowUp /></Button>
						</div>
					</form>
				</Inspector.Root>
			{/if}
			</div>

			<!-- 하단 Ops (.pen Workbench/BottomOpsPanel) -->
			<section class={cn('aside-col border-t bg-card', opsOpen ? 'h-49' : 'h-10')} aria-label="Ops">
				<Tabs.Root bind:value={opsTab} class="flex min-h-0 flex-1 flex-col gap-0">
					<div class="ops-tabbar">
						<Tabs.List variant="line" class="h-full flex-1 justify-start border-b-0">
							{#each opsTabs as [v, l] (v)}<Tabs.Trigger value={v} class="h-full">{l}</Tabs.Trigger>{/each}
						</Tabs.List>
						<span class={cn('label-xs', live ? 'text-success' : 'text-muted-foreground')}>
							<span class={cn('size-1.5 rounded-full', live ? 'bg-success' : 'bg-subtle-foreground')}></span>
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
						<Tabs.Content value="activity" class="min-h-0 flex-1 overflow-y-auto px-2 py-1.5 font-mono text-xs">
							{#each logs as l, i (i)}
								<div class="log-line">
									<span class="text-muted-foreground">{l.time}</span>
									<span class={cn('font-semibold', roles[l.role].text)}>{l.source}</span>
									<span class="min-w-0 flex-1 truncate">{l.message}</span>
								</div>
							{/each}
						</Tabs.Content>
						{#each opsTabs.slice(1) as [v] (v)}
							<Tabs.Content value={v} class="flex flex-1 items-center justify-center text-xs text-muted-foreground">실행기 연동 후 표시돼요.</Tabs.Content>
						{/each}
					{/if}
				</Tabs.Root>
			</section>
		</main>

		<!-- 우측 PM Dock (.pen ProjectPMChatDock) — 메시지 종류 · 결정 패널은 #58 -->
		{#if dockOpen}
			<aside class="aside-col w-100 border-l bg-background" aria-label="PM Dock">
				<header class="dock-head">
					<RoleAvatar role="orch" />
					<span class="col-fill gap-px">
						<span class="text-body font-semibold">Orch · Project Manager</span>
						<span class="truncate text-2xs text-muted-foreground">Project PM · {project.name} · 새 업무 생성/분해/할당</span>
					</span>
					<Button variant="ghost" size="icon-sm" aria-label="PM Dock 접기" onclick={() => (dockOpen = false)}><PanelRightClose /></Button>
				</header>
				<!-- 짧으면 아래에 붙고 길면 스크롤 (justify-end는 넘친 위쪽을 스크롤 밖으로 밀어낸다) -->
				<div bind:this={threadEl} class="dock-body" aria-live="polite">
					<div class="mt-auto flex flex-col gap-3">
					{#if pending.length}
						<!-- Orch 카드: 판단 대기 요약 (.pen OrchCard) -->
						<div class="card-col gap-2 p-3 shadow-xs">
							<div class="flex items-center gap-2">
								<Hourglass class="size-4 text-status-waiting" />
								<span class="flex-1 text-sm font-semibold">판단 대기 {pending.length}</span>
								<Badge class="rounded-xs bg-primary font-mono text-2xs">L2</Badge>
							</div>
							<p class="text-xs text-muted-foreground">가장 급한 것 {pending[0].left} 남음 · 시간이 지나면 Orch가 추천안으로 결정해요.</p>
							<Button size="sm" variant="outline" class="self-start" onclick={() => openDecisions()}>판단 대기 열기</Button>
						</div>
					{/if}
					{#each chat as c, i (i)}
						{#if c.kind === 'user'}
							<Message.Root align="end">
								<Message.Content>
									<Bubble.Root align="end"><Bubble.Content class="text-body whitespace-pre-wrap">{c.text}</Bubble.Content></Bubble.Root>
									<Message.Footer>You · {c.time}</Message.Footer>
								</Message.Content>
							</Message.Root>
						{:else if c.kind === 'orch'}
							<Message.Root>
								<RoleAvatar role="orch" size="sm" />
								<Message.Content>
									<Message.Header>Orch · {c.time}</Message.Header>
									<Bubble.Root variant="muted"><Bubble.Content class="text-body">{c.text}</Bubble.Content></Bubble.Root>
								</Message.Content>
							</Message.Root>
						{:else if c.kind === 'proposal'}
							<!-- 작업 제안 (.pen WorkProposalCard) -->
							<div class="card-col gap-3 p-3 shadow-xs">
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
						{:else}
							<!-- 명령 결과 (.pen CommandResultCard) -->
							<div class="card-col gap-2 p-3 shadow-xs">
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
					<div class="dock-composer">
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
	<Dialog.Root bind:open={panel}>
		<Dialog.Content size="xl" tall>
			<Dialog.Header>
				<Dialog.Title class="flex items-center gap-2">판단 대기 <Badge variant="secondary">{pending.length}</Badge></Dialog.Title>
				<Dialog.Description>{scope === undefined ? `${project.name} · L2 모호한 판단` : `Task #${scope}만`}</Dialog.Description>
			</Dialog.Header>
			<Dialog.Body padded={false} class="flex-row">
				<nav class="decision-queue" aria-label="판단 대기 목록">
					{#each shownQueue as d (d.id)}
						{@const who = agentOf(d.agent)}
						<button
							type="button"
							aria-pressed={active === d.id}
							onclick={() => ((active = d.id), (pick = undefined), (answer = ''))}
							class={cn('decision-item', active === d.id && 'bg-accent')}
						>
							<span class="label-sm">{#if who}<RoleAvatar role={who.role} size="sm" />{who.name}{/if} · #{d.task}</span>
							<span class="truncate text-xs text-muted-foreground">{d.topic}</span>
							<span class={cn('font-mono text-2xs', d.left ? 'text-status-waiting' : 'text-subtle-foreground')}>{d.left ? `${d.left} 남음` : `결정됨 · ${d.decided}`}</span>
						</button>
					{:else}
						<p class="p-4 text-center text-xs text-muted-foreground">판단 대기가 없어요.</p>
					{/each}
				</nav>
				{#if dec}
					{@const who = agentOf(dec.agent)}
					<section class="flex min-w-0 flex-1 flex-col">
						<div class="decision-head">
							{#if who}<RoleAvatar role={who.role} />{/if}
							<div class="row-text">
								<span class="text-sm font-semibold">{who?.name} · #{dec.task} {dec.title}</span>
								<span class="text-xs text-muted-foreground">{dec.sub}</span>
							</div>
							<Badge variant="secondary" class="font-mono">{dec.left ? `${dec.left} 남음` : `결정됨 · ${dec.decided}`}</Badge>
						</div>
						<ol class="decision-body">
							{#each dec.questions as q, n (n)}
								<li class={cn('flex flex-col gap-2', n !== qi && 'opacity-80')}>
									<p class="title-sm gap-2">
										<span class="font-mono text-xs text-muted-foreground">Q{n + 1}</span>{q.q}
										{#if q.answer}<span class="text-xs font-normal text-status-done">→ {q.answer}</span>{:else if n !== qi}<span class="text-xs font-normal text-muted-foreground">대기</span>{/if}
									</p>
									{#if n === qi}
										{#if q.context}<p class="text-body leading-relaxed text-muted-foreground">{q.context}</p>{/if}
										{#if q.options}
											<div class="grid grid-cols-3 gap-2" role="radiogroup" aria-label="Q{n + 1} 선택지">
												{#each q.options as o (o.key)}
													<!-- .pen OptionCard on/off -->
													<button
														type="button"
														role="radio"
														aria-checked={pick === o.key}
														onclick={() => (pick = o.key)}
														class={cn('option-press rounded-lg', pick === o.key && 'option-on')}
													>
														<span class="title-sm gap-1.5"><span class="font-mono text-xs text-muted-foreground">{o.key}</span>{o.title}{#if o.rec}<Badge variant="secondary" class="text-2xs">추천</Badge>{/if}</span>
														{#if o.desc}<span class="text-xs text-muted-foreground">{o.desc}</span>{/if}
													</button>
												{/each}
											</div>
										{/if}
									{/if}
								</li>
							{/each}
						</ol>
						{#if qi >= 0}
							<form class="decision-foot" onsubmit={reply}>
								<Textarea bind:value={answer} rows={2} placeholder="선택지에 덧붙일 말이나 직접 답을 적어주세요" aria-label="답변" onkeydown={(e) => (e.metaKey || e.ctrlKey) && e.key === 'Enter' && e.currentTarget.form?.requestSubmit()} />
								<div class="flex items-center gap-2">
									<p class="flex-1 text-xs text-muted-foreground">답변은 #{dec.task} 결정 기록으로 남고 {who?.name}에게 전달돼요 · ⌘↵ 보내기</p>
									<Button type="submit" size="sm" disabled={!pick && !answer.trim()}>보내기</Button>
								</div>
							</form>
						{/if}
					</section>
				{/if}
			</Dialog.Body>
		</Dialog.Content>
	</Dialog.Root>

	<!-- Task Editor (.pen XBNVi A · 새 태스크 / A' · 편집) -->
	<Dialog.Root bind:open={() => editor !== undefined, (v) => !v && (editor = undefined)}>
		<!-- ⌘↵ 제출 -->
		<Dialog.Content size={editorWide ? 'xl' : 'md'} onkeydown={(e) => (e.metaKey || e.ctrlKey) && e.key === 'Enter' && saveTask(e)}>
			{#if editor}
				{@const d = editor}
				{@const top = issueOf(topIssue(d.issue))}
				{@const subs = top ? subIssues(top.num) : []}
				{@const leaf = issueOf(d.issue)}
				{@const who = agentOf(d.agent)}
				{@const st = statuses[d.status]}
				{@const pr = priorities[d.priority]}
				{@const run = liveRunOf(d.num)}
				<form onsubmit={saveTask} class="contents">
					<Dialog.Header>
						{#snippet lead()}<span class="task-mark"><SquareCheck class="size-3" /></span>{/snippet}
						<Dialog.Title class="sr-only">{d.num === undefined ? '새 태스크' : `Task #${d.num} 편집`}</Dialog.Title>
						<!-- 경로: 프로젝트 / 이슈 / 하위 이슈 -->
						<div class="editor-path">
							<span class="text-muted-foreground">{project.name}</span>
							<span class="text-subtle-foreground">/</span>
							<DropdownMenu.Root>
								<DropdownMenu.Trigger>
									{#snippet child({ props })}
										<Button {...props} variant="outline" size="xs"><CircleDotIcon class="text-node-issue" />{top ? `#${top.num} ${top.title}` : '이슈 선택'}<ChevronDown class="text-muted-foreground" /></Button>
									{/snippet}
								</DropdownMenu.Trigger>
								<DropdownMenu.Content align="start" class="w-64">
									{#each issueList.filter((i) => !i.parent) as i (i.num)}
										<DropdownMenu.Item onSelect={() => (d.issue = i.num)}><CircleDotIcon class="text-node-issue" />#{i.num} {i.title}{#if top?.num === i.num}<Check class="ml-auto" />{/if}</DropdownMenu.Item>
									{/each}
								</DropdownMenu.Content>
							</DropdownMenu.Root>
							{#if subs.length}
								<span class="text-subtle-foreground">/</span>
								<DropdownMenu.Root>
									<DropdownMenu.Trigger>
										{#snippet child({ props })}
											<Button {...props} variant="outline" size="xs"><GitBranchIcon class="text-node-issue" />{leaf?.parent ? `#${leaf.num} ${leaf.title}` : '하위 이슈'}<ChevronDown class="text-muted-foreground" /></Button>
										{/snippet}
									</DropdownMenu.Trigger>
									<DropdownMenu.Content align="start" class="w-60">
										<DropdownMenu.Item onSelect={() => top && (d.issue = top.num)}>하위 이슈 없음{#if !leaf?.parent}<Check class="ml-auto" />{/if}</DropdownMenu.Item>
										{#each subs as i (i.num)}
											<DropdownMenu.Item onSelect={() => (d.issue = i.num)}><GitBranchIcon class="text-node-issue" />#{i.num} {i.title}{#if d.issue === i.num}<Check class="ml-auto" />{/if}</DropdownMenu.Item>
										{/each}
									</DropdownMenu.Content>
								</DropdownMenu.Root>
							{/if}
						</div>
						{#snippet actions()}
							<Button variant="ghost" size="icon-sm" aria-label={editorWide ? '작게' : '크게'} onclick={() => (editorWide = !editorWide)}>{#if editorWide}<Minimize2 />{:else}<Maximize2 />{/if}</Button>
						{/snippet}
					</Dialog.Header>

					<Dialog.Body class="gap-3.5">
						<!-- svelte-ignore a11y_autofocus -->
						<Input variant="title" autofocus bind:value={d.title} placeholder="태스크 제목" aria-label="제목" />

						<!-- 설명 (.pen Description Editor) -->
						<InputGroup.Root>
							<InputGroup.Addon align="block-start" class="border-b bg-muted">
								{#each bodyTools as t (t.k)}
									<InputGroup.Button size="icon-xs" aria-label={t.label} title={t.label} onclick={() => format(t.k)}><t.icon /></InputGroup.Button>
								{/each}
								<InputGroup.Button size="icon-xs" aria-label="첨부" title="첨부" onclick={() => fileInput?.click()}><Paperclip /></InputGroup.Button>
								<Input bind:ref={fileInput} type="file" multiple class="hidden" onchange={(e) => attach(e.currentTarget.files)} />
								<InputGroup.Text class="ml-auto text-caption">Markdown · @ 언급</InputGroup.Text>
							</InputGroup.Addon>
							<InputGroup.Textarea bind:ref={bodyArea} bind:value={d.body} placeholder="무엇을 · 왜 · 참고할 것" aria-label="설명" class="field-sizing-content min-h-16" />
							{#if d.files.length}
								<InputGroup.Addon align="block-end">
									<Attachment.Group>
										{#each d.files as f, i (i)}
											<Attachment.Root size="sm">
												{#if f.url}
													<Attachment.Media variant="image"><img src={f.url} alt={f.name} /></Attachment.Media>
												{:else}
													<Attachment.Media><FileText /></Attachment.Media>
													<Attachment.Content><Attachment.Title>{f.name}</Attachment.Title><Attachment.Description>{fileMeta(f)}</Attachment.Description></Attachment.Content>
												{/if}
												<Attachment.Actions><Attachment.Action aria-label="{f.name} 빼기" onclick={() => d.files.splice(i, 1)}><X /></Attachment.Action></Attachment.Actions>
											</Attachment.Root>
										{/each}
									</Attachment.Group>
								</InputGroup.Addon>
							{/if}
						</InputGroup.Root>

						<!-- 완료 조건 -->
						<section class="box-col gap-0.5 rounded-md p-3" aria-label="완료 조건">
							<h3 class="pb-1 text-xs font-semibold text-muted-foreground">완료 조건</h3>
							<Checklist label="완료 조건" placeholder="조건 추가" bind:items={d.criteria} />
						</section>

						<!-- 속성 칩 (.pen Properties) — 선택 창은 C -->
						<div class="flex flex-wrap items-center gap-1.5">
							<StatusSelect bind:value={d.status}>
								{#snippet trigger(props)}{@render propChip(props, st.icon, st.label, st.text)}{/snippet}
							</StatusSelect>
							<AssigneePicker bind:value={d.agent} agents={agentList} tasks={list} recommend={recommendFor(d.title, d.labels)} onorch={() => (d.agent = orchPick(d.title, d.labels))}>
								{#snippet trigger(props)}{@render propChip(props, who ? roles[who.role].icon : UserRoundX, who ? `${who.name} · ${roles[who.role].label}` : 'Unassigned', who ? roles[who.role].text : undefined)}{/snippet}
							</AssigneePicker>
							<PriorityPicker bind:value={d.priority}>
								{#snippet trigger(props)}{@render propChip(props, pr.icon, d.priority, pr.text)}{/snippet}
							</PriorityPicker>
							<DependsPicker bind:value={d.deps} current={d.num} tasks={list} depsOf={(n) => details[n]?.deps ?? []}>
								{#snippet trigger(props)}
									{@const dep = d.deps.filter((x) => x.kind === 'depends')}
									{@render propChip(props, Link2Icon, dep.length ? `depends on ${dep.map((x) => `#${x.num}`).join(', ')}` : d.deps.length ? `blocks ${d.deps.map((x) => `#${x.num}`).join(', ')}` : 'Depends')}
								{/snippet}
							</DependsPicker>
							<!-- ETA · 라벨 선택 창은 .pen에 없어 표시만 (#60) -->
							<Badge variant="outline"><TimerIcon />{d.eta || 'ETA'}</Badge>
							<Badge variant="outline"><Tag />{d.labels.length ? d.labels.join(' · ') : 'Labels'}</Badge>
						</div>

						{#if d.num === undefined}
							<Alert.Root variant="primary">
								<Sparkles />
								<Alert.Description>제목만 적으면 Orch가 설명 · 완료 조건 · 담당자 · 의존 관계를 제안해요</Alert.Description>
								<Alert.Action><Button variant="outline" size="sm" onclick={orchDraft}><WandSparkles />초안 요청</Button></Alert.Action>
							</Alert.Root>
						{:else if run}
							<Alert.Root variant="warning">
								<Radio />
								<Alert.Description>{who?.name ?? '담당'}이 Run #{run.num} 실행 중 — 저장하면 변경 사항이 Runtime Instruction으로 전달돼요</Alert.Description>
							</Alert.Root>
						{/if}
					</Dialog.Body>

					<Dialog.Footer>
						{#snippet lead()}
							{#if d.num === undefined}
								<label class="meta-xs gap-1.5"><Switch size="sm" bind:checked={more} aria-label="계속 만들기" />계속 만들기</label>
							{/if}
						{/snippet}
						<Dialog.Close>{#snippet child({ props })}<Button type="button" variant="ghost" {...props}>취소</Button>{/snippet}</Dialog.Close>
						<Button type="submit" disabled={!d.title.trim()}>{d.num === undefined ? '태스크 생성' : '저장'}<Kbd class="bg-primary-foreground/15 text-primary-foreground">⌘↵</Kbd></Button>
					</Dialog.Footer>
				</form>
			{/if}
		</Dialog.Content>
	</Dialog.Root>
{:else}
	<Empty.Root class="h-full">
		<Empty.Header>
			<Empty.Title>프로젝트를 찾을 수 없어요</Empty.Title>
			<Empty.Description><a href="/p">All Projects</a>에서 다시 선택하세요.</Empty.Description>
		</Empty.Header>
	</Empty.Root>
{/if}
