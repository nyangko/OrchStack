/// 개발 설정 (#91). 지금은 디자인 기준 라우트 완성 단계라 **목데이터가 기본**이다. 서버에 붙여 볼 때만 VITE_MOCK=0.
/// apiUrl 기본은 '/api' — Vite 프록시(vite.config.ts)가 백엔드로 넘긴다. 다른 출처를 직접 부르려면 VITE_API_URL.
export const apiUrl: string = import.meta.env.VITE_API_URL ?? '/api';
export const useMock: boolean = import.meta.env.VITE_MOCK !== '0';
