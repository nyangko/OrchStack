// Settings 첫 화면 = 일반.
import { redirect } from '@sveltejs/kit';

export function load() {
	redirect(307, '/settings/general');
}
