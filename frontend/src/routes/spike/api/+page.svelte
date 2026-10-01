<script lang="ts">
	/// API 기반 확인용 스파이크 (#91): 목록 호출 · 404 토스트 · 409 문구 · 연결 실패 토스트.
	import { api, failureOf, type ApiFailure } from '$lib/api/client';
	import { useMock, apiUrl } from '$lib/api/env';
	import { Button } from '$lib/components/ui/button';

	let out = $state('');
	let conflict = $state<ApiFailure>();
	async function list() {
		const { data } = await api.GET('/projects');
		out = JSON.stringify(data ?? null).slice(0, 300);
	}
	async function missing() {
		await api.GET('/projects/{sn}', { params: { path: { sn: 999999 } } });
	}
	async function move() {
		const { error, response } = await api.POST('/tasks/{sn}/move', { params: { path: { sn: 1 } }, body: { status: 'done' } });
		conflict = error ? failureOf(error, response) : undefined;
	}
</script>

<main class="flex flex-col gap-3 p-8 text-sm">
	<p>API {apiUrl} · mock {String(useMock)}</p>
	<div class="flex gap-2">
		<Button onclick={list}>GET /projects</Button>
		<Button variant="outline" onclick={missing}>404 토스트</Button>
		<Button variant="outline" onclick={move}>409 문구</Button>
	</div>
	<pre class="rounded-md bg-muted p-3 text-xs" data-testid="out">{out}</pre>
	{#if conflict}<p class="text-destructive" data-testid="conflict">{conflict.status} · {conflict.message}</p>{/if}
</main>
