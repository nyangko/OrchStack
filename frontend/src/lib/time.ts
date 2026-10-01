/// 서버 시각(UTC 'YYYY-MM-DD HH:MM:SS') → 상대 표시 ('just now' · '3m ago' · '2h ago' · '5d ago').
export function ago(at?: string | null): string {
	if (!at) return '—';
	const t = Date.parse(at.includes('T') ? at : at.replace(' ', 'T') + 'Z');
	if (Number.isNaN(t)) return at;
	const s = Math.max(0, (Date.now() - t) / 1000);
	if (s < 60) return 'just now';
	if (s < 3600) return `${Math.floor(s / 60)}m ago`;
	if (s < 86400) return `${Math.floor(s / 3600)}h ago`;
	return `${Math.floor(s / 86400)}d ago`;
}
