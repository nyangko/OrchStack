/// 프로젝트 현재 상태 (#91): snapshot(Issue · Task · Member)을 한 곳에 두고 Kanban · Issue Board · Diagram · Quick Panel이 같은 값을 본다.
/// 이벤트가 오면 행 단위로 갱신한다 — Created는 payload가 행 전체라 그대로 넣고, 나머지는 그 행을 다시 읽는다(payload 해석에 의존하지 않아 종류가 늘어도 안전).
import { api, failureOf } from '$lib/api/client';
import { ProjectStream, type ApiEvent, type StreamState } from '$lib/api/stream.svelte';
import type { ApiIssue, ApiMember, ApiRun, ApiTask } from '$lib/api/types';
import type { Agent, Issue, Task } from '$lib/mock';
import { statuses, type TaskStatus } from '$lib/status';
import { roles, type Role } from '$lib/roles';
import { ago } from '$lib/time';

export const project = $state({
	sn: 0,
	state: 'idle' as StreamState | 'idle',
	issues: [] as ApiIssue[],
	tasks: [] as ApiTask[],
	members: [] as ApiMember[],
	/** 마지막으로 적용한 이벤트 (디버그 · 스파이크 표시용). */
	last: undefined as ApiEvent | undefined
});

let stream: ProjectStream | undefined;

/// 프로젝트를 연다. 같은 프로젝트면 그대로, 다르면 이전 스트림을 닫고 새로 받는다.
export async function openProject(sn: number, timeout?: number) {
	if (stream?.sn === sn) return;
	stream?.close();
	project.sn = sn;
	project.state = 'loading';
	const s = new ProjectStream(
		sn,
		(snap) => {
			project.issues = snap.issues;
			project.tasks = snap.tasks;
			project.members = snap.members;
		},
		apply,
		timeout
	);
	stream = s;
	$effect.root(() => {
		$effect(() => {
			if (stream === s) project.state = s.state;
		});
	});
	await s.start();
}

export function closeProject() {
	stream?.close();
	stream = undefined;
	project.state = 'idle';
}

function upsert<T extends { sn: number }>(list: T[], row: T) {
	const i = list.findIndex((x) => x.sn === row.sn);
	if (i < 0) list.push(row);
	else list[i] = row;
}
const drop = <T extends { sn: number }>(list: T[], sn: number) => {
	const i = list.findIndex((x) => x.sn === sn);
	if (i >= 0) list.splice(i, 1);
};

async function refetchTask(sn: number) {
	const { data } = await api.GET('/tasks/{sn}', { params: { path: { sn } } });
	if (data) upsert(project.tasks, data);
}

/// 이벤트 1건을 상태에 반영한다. 모르는 종류는 대상(aggregate)만 보고 다시 읽는다.
async function apply(ev: ApiEvent) {
	project.last = ev;
	const deleted = ev.event_type.endsWith('Deleted');
	const created = ev.event_type.endsWith('Created');
	switch (ev.aggregate_type) {
		case 'task':
			if (deleted) drop(project.tasks, ev.aggregate_sn);
			else if (created) upsert(project.tasks, ev.payload as ApiTask);
			else await refetchTask(ev.aggregate_sn);
			return;
		case 'issue':
			if (deleted) drop(project.issues, ev.aggregate_sn);
			else if (created) upsert(project.issues, ev.payload as ApiIssue);
			else {
				const { data } = await api.GET('/issues/{sn}', { params: { path: { sn: ev.aggregate_sn } } });
				if (data) upsert(project.issues, data);
			}
			return;
		case 'member': {
			if (deleted) return drop(project.members, ev.aggregate_sn);
			const row = created ? (ev.payload as ApiMember) : (await api.GET('/members/{sn}', { params: { path: { sn: ev.aggregate_sn } } })).data;
			// 이 프로젝트 팀의 멤버만 둔다 (팀이 없으면 snapshot도 비어 있다)
			const teamSn = project.members[0]?.team_sn;
			if (row && (teamSn === undefined || row.team_sn === teamSn)) upsert(project.members, row);
			return;
		}
		case 'run': {
			// Run 전이는 태스크 상태를 바꾼다 (RunStarted → in_progress 등) — 태스크를 다시 읽는다
			const t = (ev.payload as { task_sn?: number } | null)?.task_sn;
			if (t) await refetchTask(t);
			return;
		}
		default:
			console.debug('[stream] ignored', ev.event_type);
	}
}

// ---- 화면 모양으로 (A-1 #92). 화면 코드는 목데이터 타입을 그대로 쓰고, 서버 행을 여기서 맞춘다. 스키마에 없는 값은 기본값 · #88 #89 뒤에 채운다.

const issueNum = (sn?: number | null) => project.issues.find((i) => i.sn === sn)?.num;

export function viewIssues(): Issue[] {
	return project.issues.map((i) => ({
		num: i.num, title: i.title, status: i.status as Issue['status'], parent: issueNum(i.parent_sn),
		updated: ago(i.update_at), description: i.body ?? '', labels: []
	}));
}

export function viewTasks(): Task[] {
	return project.tasks.map((t) => ({
		sn: t.sn, num: t.num, project: t.project_sn, title: t.title, description: t.description,
		status: (t.status in statuses ? t.status : 'todo') as TaskStatus,
		priority: `P${Math.min(3, Math.max(0, t.priority))}` as Task['priority'],
		agent: t.member_sn ?? undefined, issue: issueNum(t.issue_sn) ?? 0,
		steps: [0, 0], messages: 0, updated: ago(t.update_at)
	}));
}

/// role_name(예: "Frontend Developer")을 화면 역할로. Orch 멤버는 orch, 못 찾으면 agent.
function roleOf(m: ApiMember): Role {
	if (m.is_orch) return 'orch';
	const n = m.role_name.toLowerCase();
	return (Object.keys(roles) as Role[]).find((r) => r !== 'agent' && n.includes(roles[r].label.toLowerCase())) ?? 'agent';
}
const activityLabel: Record<string, string> = { running: 'Running', waiting: 'Waiting', idle: 'Idle', paused: 'Paused' };

/// 멤버 → 에이전트. runtime은 프로필(runtime_sn)을 풀어야 알 수 있어 아직 claude 고정 (#60).
export function viewAgents(): Agent[] {
	return project.members.filter((m) => m.status !== 'archived').map((m) => ({
		sn: m.sn, name: m.name, role: roleOf(m), runtime: 'claude', activity: activityLabel[m.status] ?? m.status,
		online: m.status !== 'paused', tokens: '—'
	}));
}

/// 상태 이동 command. 성공하면 응답 행을 바로 반영(이벤트로도 다시 온다). 실패하면 문구를 돌려준다 — 409는 토스트 없이 화면이 보여준다.
export async function moveTask(num: number, status: TaskStatus): Promise<string | undefined> {
	const sn = project.tasks.find((t) => t.num === num)?.sn;
	if (sn === undefined) return '태스크를 찾을 수 없어요';
	const { data, error, response } = await api.POST('/tasks/{sn}/move', { params: { path: { sn } }, body: { status } });
	if (data) upsert(project.tasks, data);
	return error ? failureOf(error, response).message : undefined;
}

/// 태스크의 Run 목록 (최근이 앞).
export async function runsOf(num: number): Promise<ApiRun[]> {
	const sn = project.tasks.find((t) => t.num === num)?.sn;
	if (sn === undefined) return [];
	const { data } = await api.GET('/tasks/{sn}/runs', { params: { path: { sn } } });
	return (data ?? []).toSorted((a, b) => b.num - a.num);
}

/// Run 중지 command. 실패 문구를 돌려준다.
export async function stopRun(sn: number): Promise<string | undefined> {
	const { error, response } = await api.POST('/runs/{sn}/stop', { params: { path: { sn } } });
	return error ? failureOf(error, response).message : undefined;
}
