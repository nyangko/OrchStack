/// 프로젝트 현재 상태 (#91): snapshot(Issue · Task · Member)을 한 곳에 두고 Kanban · Issue Board · Diagram · Quick Panel이 같은 값을 본다.
/// 이벤트가 오면 행 단위로 갱신한다 — Created는 payload가 행 전체라 그대로 넣고, 나머지는 그 행을 다시 읽는다(payload 해석에 의존하지 않아 종류가 늘어도 안전).
import { api } from '$lib/api/client';
import { ProjectStream, type ApiEvent, type StreamState } from '$lib/api/stream.svelte';
import type { ApiIssue, ApiMember, ApiTask } from '$lib/api/types';

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
