/// 백엔드 API 클라이언트. 경로 · 요청 · 응답 타입은 schema.d.ts(`pnpm api:gen`)에서 온다.
/// 실패 응답(ErrorBody)은 여기서 한 번 토스트로 띄운다. 409(상태 전이 불가)는 화면이 문구로 보여주므로 토스트 없음.
import createClient, { type Middleware } from 'openapi-fetch';
import { toast } from 'svelte-sonner';
import type { paths } from './schema';
import type { ApiError } from './types';
import { apiUrl } from './env';

/** 실패 응답을 화면용으로 정리한 것. 409면 toasted=false — 호출부가 문구를 그린다. */
export type ApiFailure = { status: number; message: string; code?: string };

/// 응답 본문에서 ErrorBody를 꺼낸다. JSON이 아니면 상태 문구.
async function readFailure(res: Response): Promise<ApiFailure> {
	const body = (await res.clone().json().catch(() => undefined)) as Partial<ApiError> | undefined;
	return { status: res.status, message: body?.message ?? `${res.status} ${res.statusText}`, code: body?.error };
}

const report: Middleware = {
	async onResponse({ response }) {
		if (response.ok || response.status === 409) return;
		const f = await readFailure(response);
		toast.error(f.message, { description: f.code });
	},
	onError({ error }) {
		toast.error('서버에 연결할 수 없어요', { description: String(error).slice(0, 120) });
	}
};

export const api = createClient<paths>({ baseUrl: apiUrl });
api.use(report);

/// openapi-fetch 결과의 error를 화면 문구로. 409 등을 호출부에서 보여줄 때.
export function failureOf(error: unknown, response?: Response): ApiFailure {
	const e = error as Partial<ApiError> | undefined;
	return { status: response?.status ?? 0, message: e?.message ?? '요청에 실패했어요', code: e?.error };
}
