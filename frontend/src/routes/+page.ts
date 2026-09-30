// 홈 = 첫 실행을 안 마쳤으면 /setup, 마쳤으면 첫 프로젝트 Workbench (프로젝트가 없으면 All Projects).
// 첫 실행 여부는 브라우저 저장소에 있어 이 페이지는 브라우저에서만 판단한다.
import { redirect } from '@sveltejs/kit';
import { projects } from '$lib/mock';
import { setupDone } from '$lib/setup';

export const ssr = false;

export function load() {
	if (!setupDone()) redirect(307, '/setup');
	redirect(307, projects.length ? `/p/${projects[0].sn}` : '/p');
}
