/// 개발 설정 (#91). apiUrl 기본은 '/api' — Vite 프록시(vite.config.ts)가 백엔드로 넘긴다. 다른 출처를 직접 부르려면 VITE_API_URL.
/// VITE_MOCK=1 이면 API를 부르지 않고 $lib/mock.ts 값을 그대로 쓴다. 백엔드 없이 화면만 볼 때.
export const apiUrl: string = import.meta.env.VITE_API_URL ?? '/api';
export const useMock: boolean = import.meta.env.VITE_MOCK === '1';
