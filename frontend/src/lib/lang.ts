/// 화면 언어 이름 ↔ 서버 언어 코드 (워크스페이스 · 연결의 report_language · commit_language).
export const langCode: Record<string, string> = { 한국어: 'ko', English: 'en', 日本語: 'ja' };

/// 서버 코드 → 화면 이름. 모르는 코드는 그대로.
export const langName = (code: string) => Object.keys(langCode).find((k) => langCode[k] === code) ?? code;
