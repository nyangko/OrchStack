<script lang="ts">
	/// API 기반 확인용 스파이크 (#91): 목록 호출 · 404 토스트 · 409 문구 · 연결 실패 토스트.
	import { api, failureOf, type ApiFailure } from '$lib/api/client';
	import { useMock, apiUrl } from '$lib/api/env';
	import { Button } from '$lib/components/ui/button';
	import { project, openProject } from '$lib/project.svelte';
	import { onMount } from 'svelte';

	import { page } from '$app/state';

	// ?wd= 끊김 감지 시간(ms) — 검증용
	onMount(() => void openProject(1, Number(page.url.searchParams.get('wd')) || undefined));

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

	<!-- 스트림 확인: snapshot 수 · 연결 상태 · 마지막 이벤트 -->
	<p data-testid="stream">
		<span data-testid="state">{project.state}</span> · issues <span data-testid="issues">{project.issues.length}</span> · tasks <span data-testid="tasks">{project.tasks.length}</span> · members {project.members.length} · last <span data-testid="last">{project.last?.sn ?? 0} {project.last?.event_type ?? ''}</span>
	</p>
	<ul class="text-xs" data-testid="tasklist">
		{#each project.tasks as t (t.sn)}<li>#{t.num} {t.title} · {t.status}</li>{/each}
	</ul>
</main>
