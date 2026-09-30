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
	import X from '@lucide/svelte/icons/x';
	import Square from '@lucide/svelte/icons/square';
	import MessageCircleQuestion from '@lucide/svelte/icons/message-circle-question';
	import Check from '@lucide/svelte/icons/check';
	import Hourglass from '@lucide/svelte/icons/hourglass';
	import * as Message from '$lib/components/ui/message';
	import * as Bubble from '$lib/components/ui/bubble';
	import * as Dialog from '$lib/components/ui/dialog';
	import { Textarea } from '$lib/components/ui/textarea';
	import { untrack } from 'svelte';
	import { SvelteFlow, Background, BackgroundVariant, Controls, Panel, MarkerType, type Node, type Edge } from '@xyflow/svelte';
	import '@xyflow/svelte/dist/style.css';
	import DiagramNode, { type DiagramNodeData } from '$lib/components/orch/diagram/diagram-node.svelte';
	import { DragDropProvider } from '@dnd-kit-svelte/svelte';
	import { move } from '@dnd-kit/helpers';
	import KanbanCard from '$lib/components/orch/kanban/kanban-card.svelte';
	import KanbanColumn from '$lib/components/orch/kanban/kanban-column.svelte';
	import { Badge } from '$lib/components/ui/badge';
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
	import { tasks, agents, logs, issues, taskDetails, agentActivity, decisions, thread, type Issue, type Chat } from '$lib/mock';
	import { store } from '$lib/teams.svelte';
	import { cn } from '$lib/utils';

	const project = $derived(store.projects.find((p) => p.sn === Number(page.params.project)));

	// 태스크 목록은 페이지가 소유한다 (Kanban 이동 · Quick Panel이 같은 목록을 본다). 서버 연결은 #59.
	let list = $state(tasks.map((t) => ({ ...t })));

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
		issueSel = undefined;
	}
	const count = (s: TaskStatus) => list.filter((t) => t.status === s).length;
	const agentName = (sn?: number) => agents.find((a) => a.sn === sn)?.name;
	const shown = $derived(
		list.filter(
			(t) =>
				(filter === 'all' || t.status === filter) &&
				(query === '' || `#${t.num} ${t.title}`.toLowerCase().includes(query.toLowerCase()))
		)
	);

	// Kanban (.pen #26 Workbench / Kanban Board) — 상태별 열. Failed 열은 해당 태스크가 있을 때만.
	const agentOf = (sn?: number) => agents.find((a) => a.sn === sn);
	const task = (num: number) => list.find((t) => t.num === num)!;
	const lanes = $derived(statusOrder.filter((s) => s !== 'failed' || count(s) > 0));
	// 드래그 중 열 배치는 따로 두고, 놓을 때 태스크 상태에 반영한다.
	let board = $state<Record<string, number[]>>({});
	$effect.pre(() => {
		board = Object.fromEntries(statusOrder.map((s) => [s, list.filter((t) => t.status === s).map((t) => t.num)]));
	});
	/// 놓은 열을 태스크 상태로 반영한다. (서버 command 호출은 #59)
	function drop() {
		for (const [s, nums] of Object.entries(board)) for (const n of nums) task(n).status = s as TaskStatus;
	}

	// Issue Board (.pen #26 Workbench / Issue Board) — 이슈 → 하위 이슈 → 태스크 트리.
	const issueOf = (num: number) => issues.find((i) => i.num === num);
	const subIssues = (num: number) => issues.filter((i) => i.parent === num);
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
	const col = [0, 260, 520];
	const nodeTypes = { diagram: DiagramNode };
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
			menu: [
				{ label: '상세 보기', onSelect: () => open(t.num) },
				...statusOrder.filter((st) => st !== t.status && st !== 'waiting').map((st) => ({ label: `→ ${statuses[st].label}`, onSelect: () => (task(num).status = st) }))
			]
		};
	}
	/// 에이전트 노드 데이터.
	function agentData(sn: number, ctx: string): DiagramNodeData {
		const a = agentOf(sn)!;
		return {
			kind: 'agent',
			ref: roles[a.role].label.toUpperCase(),
			title: a.name,
			who: { role: a.role, name: a.runtime === 'claude' ? 'Claude Code' : 'Codex CLI', runtime: a.runtime },
			badge: a.activity.split(' · ')[0],
			meta: `ctx ${ctx}`
		};
	}
	const diagramTasks = [128, 129, 130, 131];
	// #131은 소라보다 조금 위 — 완료 보고 선(소라 아래 → Orch)의 라벨이 카드에 가리지 않게
	const taskY = [0, 228, 457, 630];
	let nodes = $state.raw<Node<DiagramNodeData>[]>([
		{ id: 'project', type: 'diagram', position: { x: col[0], y: 0 }, data: { kind: 'project', ref: 'PROJECT', title: 'OrchStack', badge: 'Active', meta: 'Core Team' } },
		{ id: 'issue-51', type: 'diagram', position: { x: col[0], y: 360 }, data: { kind: 'issue', ref: 'ISSUE #51', title: 'Authentication Flow 개선', progress: { value: 29, text: '86K / 300K tok' }, badge: 'In Progress' } },
		{ id: 'orch', type: 'diagram', position: { x: col[0], y: 140 }, data: { kind: 'orch', ref: 'ORCH · PM', title: 'Orch', who: { role: 'orch', name: 'Project Manager', runtime: 'claude', model: 'claude-opus-5.5' }, alert: { text: '판단 대기 3 · 제안 4초 후 진행', tone: 'warning' }, badge: 'Auto · 5초', meta: '자동 3/10' } },
		...diagramTasks.map((n, i) => ({ id: `task-${n}`, type: 'diagram', position: { x: col[1], y: taskY[i] }, data: taskData(n) })),
		...[[2, '92%', 0], [1, '32%', 228], [3, '12%', 457], [4, '6%', 660], [5, '20%', 862]].map(([sn, ctx, y]) => ({
			id: `agent-${sn}`, type: 'diagram', position: { x: col[2], y: y as number }, data: agentData(sn as number, ctx as string)
		}))
	]);
	// 연결선 종류별 색 (.pen Workbench/EdgeLegend). 라벨은 .pen EdgeLabel 칩 모양.
	const stroke = { contains: 'var(--input)', delegate: 'var(--primary)', assigned: 'var(--node-agent)', idle: 'var(--status-review)', live: 'var(--primary)' };
	const chip = (live = false) =>
		`font: 600 10px var(--font-mono); padding: 2px 6px; border-radius: 4px; border: 1px solid ${live ? 'var(--primary)' : 'var(--border)'}; background: ${live ? 'var(--primary)' : 'var(--card)'}; color: ${live ? 'var(--primary-foreground)' : 'var(--status-review)'};`;
	/// 연결선 하나.
	function link(id: string, source: string, target: string, kind: keyof typeof stroke, label?: string, handles = ['r', 'l']): Edge {
		return {
			id, source, target, sourceHandle: handles[0], targetHandle: handles[1], type: 'smoothstep', label,
			animated: kind === 'live',
			// 방향: 누가 누구에게 (위임 · 배정 · 요청 · 보고)
			markerEnd: { type: MarkerType.ArrowClosed, color: stroke[kind], width: 16, height: 16 },
			style: `stroke: ${stroke[kind]}; stroke-width: ${kind === 'live' ? 3 : 1.5};`,
			labelStyle: label ? chip(kind === 'live') : undefined
		};
	}
	let edges = $state.raw<Edge[]>([
		link('e-p-o', 'project', 'orch', 'contains', 'PM', ['b', 't']),
		link('e-o-i', 'orch', 'issue-51', 'delegate', '위임', ['b', 't']),
		...diagramTasks.map((n) => link(`e-i-${n}`, 'issue-51', `task-${n}`, 'contains')),
		link('e-128', 'task-128', 'agent-2', 'assigned'),
		link('e-129', 'task-129', 'agent-1', 'assigned', '배정'),
		link('e-130', 'task-130', 'agent-3', 'assigned'),
		link('e-131', 'task-131', 'agent-4', 'assigned'),
		link('e-verify', 'agent-1', 'agent-3', 'live', '검증 요청 · 진행 중', ['b', 't']),
		link('e-review', 'agent-3', 'agent-4', 'idle', '리뷰 요청', ['b', 't']),
		link('e-report', 'agent-4', 'orch', 'idle', '완료 보고', ['b', 'l'])
	]);
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
	const cur = $derived(detail && selected !== undefined ? list.find((t) => t.num === selected) : undefined);
	const info = $derived(cur ? details[cur.num] : undefined);
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
		if (e.key !== 'Escape') return;
		detail = false;
		inspect = undefined;
		issueSel = undefined;
	}}
/>

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
					<span class="flex size-5 items-center justify-center rounded-xs bg-node-issue text-on-solid"><CircleDot class="size-3" /></span>
				{:else}
					<span class="flex size-4.5 items-center justify-center rounded-xs border border-node-issue text-node-issue"><GitBranch class="size-2.75" /></span>
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
			<aside class="flex w-66 shrink-0 flex-col border-r bg-card" aria-label="Quick panel">
				<Tabs.Root bind:value={panelTab} class="flex min-h-0 flex-1 flex-col gap-0">
					<div class="flex flex-col gap-2.5 border-b p-3">
						<div class="flex items-center gap-0.5">
							<Tabs.List>
								<Tabs.Trigger value="tasks">Tasks</Tabs.Trigger>
								<Tabs.Trigger value="agents">Agents</Tabs.Trigger>
							</Tabs.List>
							<span class="flex-1"></span>
							<Button variant="ghost" size="icon-sm" aria-label="새 태스크"><Plus /></Button>
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
						{#each agents as a (a.sn)}
							<button type="button" aria-pressed={inspect === a.sn} onclick={() => openAgent(a.sn)} class={cn('flex w-full items-center gap-2.5 border-b px-3 py-2.5 text-left outline-none hover:bg-muted focus-visible:bg-muted', inspect === a.sn && 'bg-primary-soft hover:bg-primary-soft')}>
								<RoleAvatar role={a.role}>
									<Avatar.Badge class={a.online ? 'bg-success' : 'bg-subtle-foreground'} aria-label={a.online ? '온라인' : '오프라인'} />
								</RoleAvatar>
								<span class="flex min-w-0 flex-1 flex-col gap-0.5">
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
			<header class="flex h-12 shrink-0 items-center gap-3 border-b bg-background px-4">
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
				<p class="min-w-0 flex-1 truncate text-body text-muted-foreground">
					{project.name} · {list.length} tasks · {agents.length} agents
				</p>
				{#if !dockOpen}
					<Button variant="ghost" size="icon-sm" aria-label="PM Dock 펼치기" onclick={() => (dockOpen = true)}><PanelRightOpen /></Button>
				{/if}
			</header>

			<!-- 뷰 (#54 Kanban · #55 Issues · #56 Diagram) -->
			<div class="relative min-h-0 flex-1">
			<div class="h-full overflow-auto bg-canvas">
				{#if view.value === 'kanban'}
					<DragDropProvider
						onDragOver={(e) => (board = move(board, e))}
						onDragEnd={(e) => {
							if (!e.canceled) {
								board = move(board, e);
								drop();
							}
						}}
					>
						<div class="flex h-full items-start gap-3 p-4">
							{#each lanes as s (s)}
								{@const meta = statuses[s]}
								<KanbanColumn id={s}>
									<div class="flex h-10 items-center gap-2 pr-1 pl-2">
										<meta.icon class={cn('size-3.5', meta.text)} />
										<h3 class="text-body font-semibold">{meta.label}</h3>
										<Badge variant="secondary" class="rounded-full px-1.5 font-mono">{board[s]?.length ?? 0}</Badge>
										<span class="flex-1"></span>
										<Button variant="ghost" size="icon-xs" aria-label="{meta.label}에 태스크 추가"><Plus /></Button>
										<Button variant="ghost" size="icon-xs" aria-label="{meta.label} 열 메뉴"><Ellipsis /></Button>
									</div>
									<div class="flex min-h-24 flex-col gap-2">
										{#each board[s] ?? [] as num, i (num)}
											{@const t = task(num)}
											{@const a = agentOf(t.agent)}
											<KanbanCard
												id={num}
												index={i}
												group={s}
												aria-pressed={selected === t.num}
												onclick={() => open(t.num)}
												class={cn(selected === t.num && 'ring-2 ring-primary')}
											>
													<span class="flex items-center gap-2 border-b px-4 py-3">
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
													<span class="flex flex-col gap-2.5 px-4 py-3.5">
														<span class="flex items-center gap-2 text-sm font-semibold">
															<SquareCheck class="size-3.5 shrink-0 text-node-task" />#{t.num} · {t.title}
														</span>
														<span class="flex items-center gap-2 text-xs text-muted-foreground">
															<CircleDot class="size-3.5 shrink-0 text-node-issue" />Issue #{t.issue} · {issueOf(t.issue)?.title}
														</span>
														<span class="flex items-center gap-2 pt-1.5">
															{#if t.steps[1]}
																<Progress value={(t.steps[0] / t.steps[1]) * 100} class="h-1.5" aria-label="완료 조건 진행" />
																<span class="shrink-0 font-mono text-xs font-semibold whitespace-nowrap text-muted-foreground">
																	{t.steps[0]}/{t.steps[1]} · {Math.round((t.steps[0] / t.steps[1]) * 100)}%
																</span>
															{:else}
																<span class="font-mono text-xs text-subtle-foreground">no steps</span>
															{/if}
														</span>
													</span>
													<span class="flex items-center gap-3.5 border-t px-4 py-2.5 font-mono text-xs text-muted-foreground">
														<span class={cn('inline-flex items-center gap-1', t.over && 'text-warning')}><Coins class="size-3" />{t.tokens ?? '—'}{t.over ? ' ⚠' : ''}</span>
														<span class="inline-flex items-center gap-1"><MessageSquare class="size-3" />{t.messages}</span>
														{#if t.model}<Badge variant="mono" class="text-2xs"><Cpu />{t.model}</Badge>{/if}
														<span class="ml-auto inline-flex items-center gap-1"><Timer class="size-3" />{t.run ? `Run ${t.run}` : '—'}</span>
													</span>
											</KanbanCard>
										{:else}
											<p class="rounded-lg border border-dashed p-4 text-center text-xs text-subtle-foreground">비어 있어요</p>
										{/each}
									</div>
								</KanbanColumn>
							{/each}
						</div>
					</DragDropProvider>
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
						}}
						class="bg-canvas"
					>
						<!-- canvas-grid 토큰은 canvas 배경과 거의 같아 점이 안 보인다 → 한 단계 진한 input 색 -->
						<Background variant={BackgroundVariant.Dots} gap={20} size={1.5} patternColor="var(--input)" />
						<Controls position="bottom-right" showLock={false} />
						<Panel position="bottom-left">
							<div class="flex items-center gap-3 rounded-md border bg-card px-2.5 py-1.5 text-xs text-muted-foreground" aria-label="연결선 범례">
								{#each [['contains', 'bg-input', 'h-0.5'], ['delegate', 'bg-primary', 'h-0.5'], ['assigned', 'bg-node-agent', 'h-0.5'], ['interaction (idle)', 'bg-status-review', 'h-0.5'], ['live event', 'bg-primary', 'h-0.75']] as [l, bg, h] (l)}
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
							{#each issues.filter((i) => !i.parent) as i (i.num)}
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
				{@const iss = issueOf(cur.issue)}
				<!-- 태스크 상세: 뷰 위 scrim + 패널. Esc · 닫기 버튼으로 닫는다 -->
				<div class="absolute inset-0 z-10 bg-foreground/5 p-5">
					<div role="dialog" aria-modal="false" aria-label="Task #{cur.num} 상세" class="flex h-full flex-col overflow-hidden rounded-xl border bg-card shadow-lg">
						<header class="flex flex-col gap-3 border-b pt-4 pr-4 pb-3.5 pl-5">
							<div class="flex items-start gap-3">
								<span class="flex size-8 shrink-0 items-center justify-center rounded-md bg-node-task text-on-solid"><SquareCheck class="size-4" /></span>
								<div class="flex min-w-0 flex-1 flex-col gap-0.5">
									<p class="truncate text-xs text-muted-foreground">{project.name} / Issue #{cur.issue} {iss?.title} / <span class="font-mono">TASK #{cur.num}</span></p>
									<h2 class="text-lg font-semibold">{cur.title}</h2>
								</div>
								<Button variant="ghost" size="sm" onclick={() => navigator.clipboard?.writeText(`${page.url.origin}${page.url.pathname}?task=${cur.num}`)}><Link2 />Copy link</Button>
								<Button variant="ghost" size="icon-sm" aria-label="상세 닫기" onclick={() => (detail = false)}><X /></Button>
							</div>
							<div class="flex flex-wrap items-center gap-2 pl-11">
								<StatusSelect bind:value={() => cur.status, (v) => (task(cur.num).status = v)} />
								<Badge variant="outline">{cur.priority}</Badge>
								{#if a}
									<span class="flex items-center gap-1.5 text-xs">
										<RoleAvatar role={a.role} size="sm" />
										<span class="font-medium">{a.name}</span>
										<span class="text-muted-foreground">{roles[a.role].label}</span>
										{#if cur.model}<Badge variant="mono" class="text-2xs">{a.runtime === 'claude' ? 'Claude Code' : 'Codex CLI'} · {cur.model}</Badge>{/if}
									</span>
								{/if}
								{#if cur.run}<span class="text-xs text-muted-foreground">Run {cur.run}</span>{/if}
								<span class="flex-1"></span>
								<Button variant="ghost" size="icon-sm" class="bg-warning-soft text-warning" aria-label="일시정지"><Pause /></Button>
								<Button variant="ghost" size="icon-sm" class="bg-destructive-soft text-destructive" aria-label="중지"><Square /></Button>
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
						</header>
						<div class="flex min-h-0 flex-1">
							<div class="flex min-w-0 flex-1 flex-col gap-6 overflow-y-auto px-6 py-5">
								<section class="flex flex-col gap-2">
									<h3 class="text-body font-semibold">Description</h3>
									<p class="text-body leading-relaxed">{info?.description ?? '설명이 없어요.'}</p>
								</section>
								<section class="flex flex-col gap-1">
									<h3 class="text-body font-semibold">Acceptance criteria</h3>
									{#each info?.criteria ?? [] as c, i (i)}
										<label class="flex items-center gap-2 rounded-md px-1 py-1.5 text-body hover:bg-muted">
											<Checkbox bind:checked={c.done} />
											<span class={cn(c.done && 'text-muted-foreground line-through')}>{c.text}</span>
										</label>
									{:else}
										<p class="text-xs text-muted-foreground">완료 조건이 없어요.</p>
									{/each}
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
								{#if info?.runs.length}
									<section class="flex flex-col">
										<h3 class="mb-1 text-body font-semibold">Runs</h3>
										{#each info.runs as r (r.num)}
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
							<aside class="flex w-85 shrink-0 flex-col gap-5 overflow-y-auto border-l bg-background px-5 py-4" aria-label="속성">
								<section class="flex flex-col">
									<h3 class="mb-1 text-body font-semibold">Properties</h3>
									{#each [['Issue', `#${cur.issue} ${iss?.title ?? ''}`], ['Assignee', a ? `${a.name} · ${roles[a.role].label}` : 'Unassigned'], ['Priority', cur.priority], ['ETA', info?.eta ?? '—'], ['Labels', info?.labels.join(' · ') || '—']] as [k, v] (k)}
										<Item.Root variant="row" size="xs">
											<Item.Content><Item.Description>{k}</Item.Description></Item.Content>
											<Item.Actions class="min-w-0 truncate text-xs font-medium">{v}</Item.Actions>
										</Item.Root>
									{/each}
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
						</div>
					</div>
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
				<div class="absolute inset-0 z-10 bg-foreground/5 p-5">
					<div role="dialog" aria-modal="false" aria-label="Issue #{iss.num} 상세" class="flex h-full flex-col overflow-hidden rounded-xl border bg-card shadow-lg">
						<header class="flex flex-col gap-3 border-b pt-4 pr-4 pb-3.5 pl-5">
							<div class="flex items-start gap-3">
								<span class="flex size-8 shrink-0 items-center justify-center rounded-md bg-node-issue text-on-solid"><CircleDot class="size-4" /></span>
								<div class="flex min-w-0 flex-1 flex-col gap-0.5">
									<p class="truncate text-xs text-muted-foreground">
										{project?.name}{' / '}{#if parent}<button type="button" class="hover:underline" onclick={() => openIssue(parent.num)}>Issue #{parent.num} {parent.title}</button>{' / '}{/if}<span class="font-mono">ISSUE #{iss.num}</span>
									</p>
									<h2 class="text-lg font-semibold">{iss.title}</h2>
								</div>
								<Button variant="ghost" size="sm" onclick={() => navigator.clipboard?.writeText(`${page.url.origin}${page.url.pathname}?issue=${iss.num}`)}><Link2 />Copy link</Button>
								<Button variant="ghost" size="icon-sm" aria-label="상세 닫기" onclick={() => (issueSel = undefined)}><X /></Button>
							</div>
							<div class="flex flex-wrap items-center gap-3 pl-11">
								<span class={cn('flex items-center gap-1.5 text-xs font-medium', icon.text)}><icon.icon class="size-3.5" />{issueLabel[iss.status]}</span>
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
						</header>
						<div class="flex min-h-0 flex-1">
							<div class="flex min-w-0 flex-1 flex-col gap-6 overflow-y-auto px-6 py-5">
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
							<aside class="flex w-85 shrink-0 flex-col gap-5 overflow-y-auto border-l bg-background px-5 py-4" aria-label="속성">
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
						</div>
					</div>
				</div>
			{/if}
			{#if agentSel}
				{@const a = agentSel}
				{@const mine = list.filter((t) => t.agent === a.sn)}
				<!-- 에이전트 카드: 뷰 오른쪽에 뜬다 (.pen Agent Inspector Card, 340px) -->
				<div role="dialog" aria-modal="false" aria-label="{a.name} 에이전트" class="absolute top-4 right-4 bottom-4 z-10 flex w-85 flex-col overflow-hidden rounded-xl border bg-card shadow-lg">
					<header class="flex flex-col gap-3 border-b p-4">
						<div class="flex items-start gap-2.5">
							<RoleAvatar role={a.role} />
							<div class="flex min-w-0 flex-1 flex-col gap-0.5">
								<span class="flex items-center gap-1.5 text-sm font-semibold">{a.name}<RuntimeLogo runtime={a.runtime} class="size-3.5 ring-0" /></span>
								<span class="text-xs text-muted-foreground">{roles[a.role].label}</span>
								<Badge variant="mono" class="w-fit text-2xs">{a.runtime === 'claude' ? 'Claude Code' : 'Codex CLI'}</Badge>
							</div>
							<Button variant="ghost" size="icon-sm" aria-label="에이전트 카드 닫기" onclick={() => (inspect = undefined)}><X /></Button>
						</div>
						<div class="flex items-center gap-2">
							<Badge variant="secondary">{a.activity.split(' · ')[0]}</Badge>
							<span class="flex-1"></span>
							<Button variant="ghost" size="icon-sm" class="bg-warning-soft text-warning" aria-label="일시정지 · 현재 단계 끝나면 멈춤" title="일시정지 · 현재 단계 끝나면 멈춤"><Pause /></Button>
							<Button variant="ghost" size="icon-sm" class="bg-destructive-soft text-destructive" aria-label="중지"><Square /></Button>
						</div>
					</header>
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
							<div class="mt-4 flex items-center justify-between text-xs"><span class="text-muted-foreground">토큰 (오늘)</span><span class="font-mono">{a.tokens}</span></div>
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
						<div class="flex items-center gap-1.5 rounded-md border border-input bg-background py-1.5 pr-1.5 pl-3 focus-within:ring-3 focus-within:ring-ring/50">
							<input bind:value={instruction} placeholder="{a.name}에게 실행 중 지시…" aria-label="{a.name}에게 실행 중 지시" class="min-w-0 flex-1 bg-transparent text-body outline-none placeholder:text-muted-foreground" />
							<Button type="submit" size="icon-sm" aria-label="지시 보내기" disabled={!instruction.trim()}><ArrowUp /></Button>
						</div>
					</form>
				</div>
			{/if}
			</div>

			<!-- 하단 Ops (.pen Workbench/BottomOpsPanel) -->
			<section class={cn('flex shrink-0 flex-col border-t bg-card', opsOpen ? 'h-49' : 'h-10')} aria-label="Ops">
				<Tabs.Root bind:value={opsTab} class="flex min-h-0 flex-1 flex-col gap-0">
					<div class="flex h-10 shrink-0 items-center gap-4 border-b pr-3 pl-4">
						<Tabs.List variant="line" class="h-full flex-1 justify-start border-b-0">
							{#each opsTabs as [v, l] (v)}<Tabs.Trigger value={v} class="h-full">{l}</Tabs.Trigger>{/each}
						</Tabs.List>
						<span class={cn('flex items-center gap-1.5 text-xs font-medium', live ? 'text-success' : 'text-muted-foreground')}>
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
								<div class="flex items-center gap-3 px-2 py-1">
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
			<aside class="flex w-100 shrink-0 flex-col border-l bg-background" aria-label="PM Dock">
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
						<!-- Orch 카드: 판단 대기 요약 (.pen OrchCard) -->
						<div class="flex flex-col gap-2 rounded-lg border bg-card p-3 shadow-xs">
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
							<div class="flex flex-col gap-3 rounded-lg border bg-card p-3 shadow-xs">
								<div class="flex items-center gap-2">
									<span class="font-mono text-2xs font-semibold tracking-wide text-muted-foreground">WORK PROPOSAL</span>
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
							<div class="flex flex-col gap-2 rounded-lg border bg-card p-3 shadow-xs">
								<div class="flex items-center gap-2">
									<span class="font-mono text-2xs font-semibold tracking-wide text-muted-foreground">COMMAND RESULT</span>
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
							class="min-w-0 flex-1 bg-transparent text-body outline-none placeholder:text-muted-foreground"
						/>
						<Button type="submit" size="icon-sm" aria-label="보내기" disabled={!draft.trim()}><ArrowUp /></Button>
					</div>
				</form>
			</aside>
		{/if}
	</div>
	<!-- 판단 대기 패널 (.pen DecisionPanel) — 전체 / 이 태스크만 -->
	<Dialog.Root bind:open={panel}>
		<Dialog.Content class="flex h-4/5 flex-col sm:max-w-5xl">
			<Dialog.Header>
				<Dialog.Title class="flex items-center gap-2">판단 대기 <Badge variant="secondary">{pending.length}</Badge></Dialog.Title>
				<Dialog.Description>{scope === undefined ? `${project.name} · L2 모호한 판단` : `Task #${scope}만`}</Dialog.Description>
			</Dialog.Header>
			<div class="flex min-h-0 flex-1 p-0!">
				<nav class="flex w-72 shrink-0 flex-col overflow-y-auto border-r bg-background p-2" aria-label="판단 대기 목록">
					{#each shownQueue as d (d.id)}
						{@const who = agentOf(d.agent)}
						<button
							type="button"
							aria-pressed={active === d.id}
							onclick={() => ((active = d.id), (pick = undefined), (answer = ''))}
							class={cn('flex flex-col gap-1 rounded-md px-3 py-2.5 text-left outline-none hover:bg-muted focus-visible:ring-3 focus-visible:ring-ring/50', active === d.id && 'bg-accent')}
						>
							<span class="flex items-center gap-1.5 text-sm font-medium">{#if who}<RoleAvatar role={who.role} size="sm" />{who.name}{/if} · #{d.task}</span>
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
								<li class={cn('flex flex-col gap-2', n !== qi && 'opacity-80')}>
									<p class="flex items-center gap-2 text-sm font-semibold">
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
														class={cn('flex flex-col gap-1 rounded-lg border p-3 text-left outline-none hover:bg-muted focus-visible:ring-3 focus-visible:ring-ring/50', pick === o.key && 'option-on')}
													>
														<span class="flex items-center gap-1.5 text-sm font-semibold"><span class="font-mono text-xs text-muted-foreground">{o.key}</span>{o.title}{#if o.rec}<Badge variant="secondary" class="text-2xs">추천</Badge>{/if}</span>
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
			</div>
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
