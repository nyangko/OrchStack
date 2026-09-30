/// 첫 실행(인트로)을 마쳤는지 기억하는 브라우저 저장소 키.
/// 서버 연결(#45 · #47) 후에는 워크스페이스 설정 상태 API로 바뀐다.
export const SETUP_KEY = 'orch.setup.done';

export const setupDone = () => typeof localStorage !== 'undefined' && localStorage.getItem(SETUP_KEY) === '1';
