<script lang="ts">
	/// Settings › 보고서 양식 — 시스템이 조립하는 사용자 보고서 (.pen Settings · 보고서 양식). 모델 호출 없음(0 tok).
	/// {{…}} 시스템 값 · [[…]] Agent 사람 칸. 잠긴 칸은 지울 수 없고 섹션 순서 · 이름은 바꿀 수 있다. 목데이터 (서버 연결 #47).
	import type { Component } from 'svelte';
	import Eye from '@lucide/svelte/icons/eye';
	import EyeOff from '@lucide/svelte/icons/eye-off';
	import Copy from '@lucide/svelte/icons/copy';
	import ZapOff from '@lucide/svelte/icons/zap-off';
	import SquareCheck from '@lucide/svelte/icons/square-check';
	import Layers from '@lucide/svelte/icons/layers';
	import GitPullRequest from '@lucide/svelte/icons/git-pull-request';
	import Calendar from '@lucide/svelte/icons/calendar';
	import Lock from '@lucide/svelte/icons/lock';
	import Trash2 from '@lucide/svelte/icons/trash-2';
	import { Button } from '$lib/components/ui/button';
	import { PageHeader } from '$lib/components/ui/page-header';
	import { Pill } from '$lib/components/ui/pill';
	import { MdEditor } from '$lib/components/ui/md-editor';
	import { reportForms, reportSample, type ReportForm } from '$lib/mock';
	
	/// base — 복제본이면 원본 양식 key (잠긴 칸 · 아이콘을 원본에서 가져온다). name은 편집기 파일 이름.
	type Form = ReportForm & { name: string; label: string; base?: string };

	let forms = $state<Form[]>(reportForms.map((f) => ({ ...f, label: f.name, name: `${f.key}.md` })));
	let selKey = $state('task-report');
	let preview = $state(true);

	const icon: Record<string, Component> = { 'task-report': SquareCheck, 'issue-report': Layers, 'pr-body': GitPullRequest, 'daily-summary': Calendar };
	/// 지울 수 없는 칸 — 태스크 보고서의 검증(통과 · 미확인), 이슈 보고서의 미확인 합산, PR 본문의 태스크 보고서.
	const locked: Record<string, string[]> = {
		'task-report': ['{{tests.passed_summary}}', '[[unverified]]'],
		'issue-report': ['{{#each unverified}}'],
		'pr-body': ['{{> task-report}}'],
		'daily-summary': []
	};
	const sel = $derived(forms.find((f) => f.key === selKey) ?? forms[0]);
	const origin = $derived(sel.base ?? sel.key);
	const original = $derived(reportForms.find((f) => f.key === origin)!.body);
	const missing = $derived(locked[origin].filter((l) => !sel.body.includes(l)));

	type Ctx = Record<string, unknown>;
	/// 점 경로 값 — 목록의 .count는 개수.
	function get(ctx: Ctx, path: string): unknown {
		return path.split('.').reduce<unknown>((v, k) => (Array.isArray(v) && k === 'count' ? v.length : (v as Ctx | undefined)?.[k]), ctx);
	}
	/// 양식 조립 (서버와 같은 규칙의 축소판) — 조각 {{> 이름}}, {{#each}}, {{#if}}, {{값}}, [[사람 칸]].
	function render(tpl: string, ctx: Ctx): string {
		return tpl
			.replace(/\{\{> ([\w-]+)\}\}/g, (_, n) => render(forms.find((f) => f.key === n)?.body ?? '', ctx))
			.replace(/\{\{#each (\w+)\}\}([\s\S]*?)\{\{\/each\}\}/g, (_, list, inner) => ((get(ctx, list) as Ctx[] | undefined) ?? []).map((item) => render(inner, { ...ctx, ...item })).join(''))
			.replace(/\{\{#if ([\w.]+)\}\}([\s\S]*?)\{\{\/if\}\}/g, (_, path, inner) => (get(ctx, path) ? render(inner, ctx) : ''))
			.replace(/\{\{([\w.]+)\}\}/g, (_, path) => String(get(ctx, path) ?? ''))
			.replace(/\[\[(\w+)\]\]/g, (_, k) => String((ctx.human as Ctx)[k] ?? ''));
	}
	const sysVar = '{{…}}';
	const lines = $derived(render(sel.body, reportSample).split('\n'));

	function clone() {
		let key = `${origin}-copy`;
		for (let n = 2; forms.some((f) => f.key === key); n++) key = `${origin}-copy-${n}`;
		forms.push({ ...sel, key, name: `${key}.md`, label: `${sel.label} 사본`, base: origin });
		selKey = key;
	}
	function remove() {
		forms = forms.filter((f) => f.key !== sel.key);
		selKey = 'task-report';
	}
</script>

<svelte:head><title>보고서 양식 · Settings · OrchStack</title></svelte:head>

<main class="page-main">
	<PageHeader title="보고서 양식" desc="시스템이 조립하는 사용자 보고서 · 모델 호출 없음(0 tok) · 잠긴 칸은 지울 수 없어요" status={false}>
		<Button variant="outline" aria-pressed={preview} onclick={() => (preview = !preview)}>
			{#if preview}<Eye />{:else}<EyeOff />{/if}미리보기 데이터: #{reportSample.task.num}
		</Button>
		<Button onclick={clone}><Copy />양식 복제</Button>
	</PageHeader>

	<div class="strip strip-success py-2">
		<ZapOff class="size-3.5 shrink-0 text-status-done" />
		<span class="font-semibold">0 tok</span>
		<span class="text-muted-foreground">시스템 값 {sysVar} 과 Agent 사람 칸 [[…]] 을 조립만 해요 · 모델을 부르지 않아요 · Agent 작성법은 <a href="/settings/presets" class="text-primary hover:underline">Instruction presets › REPORT</a></span>
	</div>

	<div class="flex items-start gap-5">
		<nav class="list-panel w-52" aria-label="양식">
			{#each forms as f (f.key)}
				{@const Icon = icon[f.base ?? f.key]}
				<button
					type="button"
					aria-current={f.key === sel.key ? 'true' : undefined}
					onclick={() => (selKey = f.key)}
					class={['list-panel-item px-2.5 py-2', f.key === sel.key ? 'bg-accent font-semibold text-foreground' : 'text-muted-foreground']}
				>
					<Icon class="size-3.5 shrink-0" /><span class="truncate">{f.label}</span>
				</button>
			{/each}
		</nav>

		<section class="flex min-w-0 flex-1 flex-col gap-3">
			<div class="flex items-center gap-2">
				<h2 class="shrink-0 text-sm font-semibold">양식 · {sel.label} v1</h2>
				<span class="truncate text-caption text-muted-foreground">{sel.note}</span>
				<span class="flex-1"></span>
				{#if sel.base}<Button variant="ghost" size="sm" onclick={remove}><Trash2 />삭제</Button>{/if}
			</div>
			{#key sel.key}<MdEditor files={[sel]} tabs={false} base={{ [sel.name]: original }} baseLabel="기본 양식" class="h-100" />{/key}
			<div class={['strip border py-2.5', missing.length ? 'border-destructive/40 bg-destructive-soft' : 'bg-muted']}>
				<Lock class={['size-3.5 shrink-0', missing.length && 'text-destructive']} />
				<span class="shrink-0 font-semibold">잠긴 칸</span>
				{#if missing.length}
					<span class="text-destructive">지운 잠긴 칸이 있어요: <span class="font-mono">{missing.join(' · ')}</span> · 되돌려야 저장돼요</span>
				{:else if locked[origin].length}
					<span class="text-muted-foreground"><span class="font-mono">{locked[origin].join(' · ')}</span>는 지울 수 없어요 · 섹션 순서 · 이름은 바꿀 수 있어요</span>
				{:else}
					<span class="text-muted-foreground">없음 · 모든 칸을 바꿀 수 있어요</span>
				{/if}
			</div>
		</section>

		{#if preview}
			<aside class="card flex w-100 shrink-0 flex-col gap-2 p-5 text-xs" aria-label="미리보기">
				<span class="text-caption text-muted-foreground">미리보기 · #{reportSample.task.num} 데이터</span>
				{#each lines as l, i (i)}
					{#if i === 0}<h3 class="text-base font-semibold">{l.replace(/^#+\s+/, '')}</h3>
					{:else if l.startsWith('## ')}<h4 class="pt-2 font-semibold">{l.slice(3)}</h4>
					{:else if l.startsWith('범위') || l.startsWith('OrchStack task') || l.startsWith('토큰')}<p class="pt-1 text-caption text-subtle-foreground">{l}</p>
					{:else if l === '---'}<hr />
					{:else if l.trim()}<p class="leading-relaxed">{l}</p>{/if}
				{/each}
				{#if missing.length}<p class="pt-2 text-destructive">잠긴 칸이 빠져 실제 보고서에는 이 양식을 쓰지 않아요.</p>{/if}
			</aside>
		{/if}
	</div>
</main>
