// 백엔드 API 클라이언트. 경로 · 요청 · 응답 타입은 schema.d.ts(`pnpm api:gen`)에서 온다
import createClient from 'openapi-fetch';
import type { paths } from './schema';

export const api = createClient<paths>({ baseUrl: import.meta.env.VITE_API_URL ?? 'http://127.0.0.1:8080' });
