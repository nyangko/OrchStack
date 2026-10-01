/// 담당 추천 · Orch 배정 — QuickAdd · Task 상세 · Task Editor가 같은 규칙을 쓴다.
/// 목데이터 단계의 규칙이다. 실제 판단은 서버 Orch(#87).
import { toast } from 'svelte-sonner';
import type { Role } from '$lib/roles';
import type { Agent, Task } from '$lib/mock';

/// 제목 · 라벨에 이 낱말이 있으면 그 역할을 추천한다 (.pen 추천 · 역할 일치).
const roleWords: Partial<Record<Role, string[]>> = {
	frontend: ['ui', 'front', '화면', 'login', '폼'],
	backend: ['api', 'db', 'server', 'token', 'backend', '스키마'],
	qa: ['qa', 'test', '검증', 'e2e'],
	designer: ['design', '디자인', '시안', 'ui'],
	reviewer: ['review', '리뷰']
};

/// 추천 담당 sn 목록.
export function recommendFor(agents: Agent[], title: string, labels: string[]): number[] {
	const words = `${title} ${labels.join(' ')}`.toLowerCase();
	return agents.filter((a) => roleWords[a.role]?.some((w) => words.includes(w))).map((a) => a.sn);
}

/// Orch에게 배정 맡기기 — 추천 중(없으면 전체) 열린 태스크가 가장 적은 에이전트. 고른 결과를 알린다.
export function orchPick(agents: Agent[], tasks: Task[], title: string, labels: string[]): number | undefined {
	const openCount = (sn: number) => tasks.filter((t) => t.agent === sn && !['done', 'cancelled'].includes(t.status)).length;
	const pool = recommendFor(agents, title, labels);
	const best = (pool.length ? agents.filter((a) => pool.includes(a.sn)) : agents).toSorted((a, b) => openCount(a.sn) - openCount(b.sn))[0];
	if (best) toast(`Orch가 ${best.name}에게 배정했어요`, { description: '역할 · 부하 기준' });
	return best?.sn;
}
