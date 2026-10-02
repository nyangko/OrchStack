/// 첫 실행(인트로)을 마쳤는지. 서버 모드는 워크스페이스 is_onboarded(#47), 목데이터 모드는 브라우저 저장소.
import { api } from '$lib/api/client';
import { useMock } from '$lib/api/env';

/// 목데이터 모드에서 완료 표시를 기억하는 브라우저 저장소 키.
export const SETUP_KEY = 'orch.setup.done';

/// 첫 실행을 마쳤는지. 서버를 못 읽으면 마친 것으로 본다 — 인트로를 다시 돌려 팀 · 프로젝트를 또 만들지 않게.
export async function setupDone(): Promise<boolean> {
	if (useMock) return typeof localStorage !== 'undefined' && localStorage.getItem(SETUP_KEY) === '1';
	const { data } = await api.GET('/workspace').catch(() => ({ data: undefined }));
	return data ? data.is_onboarded === 1 : true;
}

/// 첫 실행 완료 표시.
export async function markSetupDone() {
	if (useMock) return localStorage.setItem(SETUP_KEY, '1');
	await api.PATCH('/workspace', { body: { is_onboarded: 1 } }).catch(() => undefined);
}
