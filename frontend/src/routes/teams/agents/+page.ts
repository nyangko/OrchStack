import { redirect } from '@sveltejs/kit';
import { templates } from '$lib/mock';

/// /teams/agents → 첫 템플릿.
export const load = () => {
	redirect(307, `/teams/agents/${templates[0].sn}`);
};
