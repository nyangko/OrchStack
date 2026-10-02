// 홈 = 첫 실행을 안 마쳤으면 /setup, 마쳤으면 첫 프로젝트 Workbench (프로젝트가 없으면 All Projects).
// 첫 실행 여부는 서버 워크스페이스(목데이터 모드는 브라우저 저장소)에서 읽어 이 페이지는 브라우저에서만 판단한다.
import { redirect } from '@sveltejs/kit';
import { store } from '$lib/teams.svelte';
import { setupDone } from '$lib/setup';

export const ssr = false;

export async function load() {
	if (!(await setupDone())) redirect(307, '/setup');
	redirect(307, store.projects.length ? `/p/${store.projects[0].sn}` : '/p');
}
