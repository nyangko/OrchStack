<script lang="ts">
	/// Settings › Instruction presets — 모델 컨텍스트에 들어가는 재사용 지침 (.pen Settings · Instruction presets).
	/// 기본 제공은 잠겨 있어 복제해서 고친다. 저장할 때마다 새 버전 · 역할 템플릿 · 멤버는 버전을 고정해 쓴다. 목데이터 (서버 연결 #47).
	import type { Component } from 'svelte';
	import Download from '@lucide/svelte/icons/download';
	import Plus from '@lucide/svelte/icons/plus';
	import Layers from '@lucide/svelte/icons/layers';
	import Search from '@lucide/svelte/icons/search';
	import Lock from '@lucide/svelte/icons/lock';
	import UserRound from '@lucide/svelte/icons/user-round';
	import ListChecks from '@lucide/svelte/icons/list-checks';
	import MessageSquare from '@lucide/svelte/icons/message-square';
	import FileText from '@lucide/svelte/icons/file-text';
	import Copy from '@lucide/svelte/icons/copy';
	import Save from '@lucide/svelte/icons/save';
	import Trash2 from '@lucide/svelte/icons/trash-2';
	import Package from '@lucide/svelte/icons/package';
	import CircleCheck from '@lucide/svelte/icons/circle-check';
	import CircleAlert from '@lucide/svelte/icons/circle-alert';
	import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '$lib/components/ui/card';
	import { InputGroup, InputGroupAddon, InputGroupInput } from '$lib/components/ui/input-group';
	import { Button } from '$lib/components/ui/button';
	import { PageHeader } from '$lib/components/orch/page-header';
	import { Pill } from '$lib/components/orch/pill';
	import { Progress } from '$lib/components/ui/progress';
	import { MdEditor } from '$lib/components/orch/md-editor';
	import { KeyValueRow } from '$lib/components/orch/key-value-row';
	import { HistoryRow } from '$lib/components/orch/history-row';
	import { presets, type Preset, type PresetKind } from '$lib/mock';
	import { store, membersOf } from '$lib/teams.svelte';
	
	type Version = { v: number; who: string; when: string; note: string; body: string };
	/// builtIn — OrchStack 기본 제공 (잠김, 복제해서 수정). name은 편집기 파일 이름 — 편집기가 이 항목의 body를 바로 고친다.
	type Item = Preset & { name: string; builtIn: boolean; versions: Version[] };

	/// 토큰 어림값 — 영어 본문은 약 4.4자, 한글이 섞이면 2.5자 = 1토큰 (측정값 기준 보정). 정확한 값은 서버가 저장할 때 잰다.
	const estimate = (text: string) => Math.round(text.length / (/[가-힣]/.test(text) ? 2.5 : 4.4));

	let list = $state<Item[]>(
		presets.map((p) => ({ ...p, name: `${p.key}.md`, builtIn: true, versions: [{ v: 1, who: 'OrchStack', when: '9/30', note: `최초 배포 · ${p.tok} tok`, body: p.body }] }))
	);
	let selKey = $state('frontend');
	let query = $state('');
	let fileInput = $state<HTMLInputElement>();

	const kinds: { kind: PresetKind; label: string; icon: Component }[] = [
		{ kind: 'protocol', label: 'PROTOCOL · 시스템', icon: Lock },
		{ kind: 'role', label: 'ROLE · 역할', icon: UserRound },
		{ kind: 'rule', label: 'RULE · 규칙', icon: ListChecks },
		{ kind: 'style', label: 'STYLE', icon: MessageSquare },
		{ kind: 'report', label: 'REPORT', icon: FileText }
	];
	const sel = $derived(list.find((p) => p.key === selKey) ?? list[0]);
	const last = $derived(sel.versions.at(-1)!);
	const tok = $derived(sel.body === last.body ? sel.tok : estimate(sel.body));
	const lang = $derived(/[가-힣]/.test(sel.body) ? '한국어' : '영어');
	/// {{…}} 변수 개수 — 조합할 때 워크스페이스 값으로 채워진다.
	const vars = $derived((sel.body.match(/\{\{[^}]+\}\}/g) ?? []).length);
	const title = $derived(sel.body.split('\n')[0].replace(/^#\s*(\w+:\s*)?/, ''));

	/// 저장 전 검사 — 상한 · 비밀키 패턴 · 다른 프리셋과 똑같은 규칙 줄.
	const checks = $derived.by(() => {
		const out: string[] = [];
		if (tok > sel.limit) out.push(`상한 ${sel.limit} tok 초과`);
		if (/sk-[A-Za-z0-9]{8,}|ghp_[A-Za-z0-9]{8,}|AKIA[0-9A-Z]{12,}/.test(sel.body)) out.push('비밀키 패턴이 있어요');
		const lines = new Set(sel.body.split('\n').filter((l) => l.startsWith('- ')));
		const dup = list.find((p) => p.key !== sel.key && !sel.versions.some((v) => v.note.includes(p.key)) && p.body.split('\n').some((l) => lines.has(l)));
		if (dup) out.push(`${dup.key}와 겹치는 규칙 줄이 있어요`);
		return out;
	});

	/// 쓰는 곳 — 역할은 같은 역할 템플릿, 공통(protocol · 기본 rule · style · report)은 모든 템플릿. 복제본은 아직 연결 전.
	const roleOf: Record<string, string> = { generalist: 'agent' };
	const usage = $derived.by(() => {
		if (!sel.builtIn || (sel.kind !== 'role' && !sel.default && sel.kind !== 'protocol')) return [];
		const tpls = store.templates.filter((t) => sel.kind !== 'role' || t.role === (roleOf[sel.key] ?? sel.key));
		return tpls.flatMap((t) => [{ label: `${t.name} v${t.version}`, tpl: true }, ...membersOf(t.name).map(({ m, team }) => ({ label: `${m.name} · ${team}`, tpl: false }))]);
	});

	/// 역할 1개당 기본 조합 — protocol + 기본 rule + 기본 style + report + 역할 평균.
	const bundle = $derived.by(() => {
		const sum = (f: (p: Item) => boolean) => list.filter((p) => p.builtIn && f(p)).reduce((n, p) => n + p.tok, 0);
		const roles = list.filter((p) => p.builtIn && p.kind === 'role');
		const parts = {
			protocol: sum((p) => p.kind === 'protocol'),
			rule: sum((p) => p.kind === 'rule' && !!p.default),
			ruleN: list.filter((p) => p.builtIn && p.kind === 'rule' && p.default).length,
			style: sum((p) => p.kind === 'style' && !!p.default),
			report: sum((p) => p.kind === 'report'),
			role: Math.round(roles.reduce((n, p) => n + p.tok, 0) / roles.length)
		};
		return { ...parts, total: parts.protocol + parts.rule + parts.style + parts.report + parts.role };
	});

	const shown = (k: PresetKind) => list.filter((p) => p.kind === k && p.key.includes(query.trim().toLowerCase()));
	const uniqueKey = (base: string) => {
		let key = base;
		for (let n = 2; list.some((p) => p.key === key); n++) key = `${base}-${n}`;
		return key;
	};
	function addCustom(kind: PresetKind, base: string, body: string, note: string, tok = estimate(body)) {
		const key = uniqueKey(base);
		list.push({ key, name: `${key}.md`, kind, limit: sel.limit, tok, body, builtIn: false, versions: [{ v: 1, who: '나', when: '방금', note, body }] });
		selKey = key;
	}
	const clone = () => addCustom(sel.kind, `${sel.key}-copy`, sel.body, `복제 · ${sel.key} v${last.v}`, sel.tok);
	const create = () => addCustom('rule', 'new-rule', '# Rule: new\n- ', '새 프리셋');

	/// .md 가져오기 — 앞머리(kind · key)가 있으면 따르고, 없으면 규칙으로.
	async function importFile(f?: File) {
		if (!f) return;
		const text = await f.text();
		const fm = text.match(/^---\n([\s\S]*?)\n---\n/);
		const meta = Object.fromEntries((fm?.[1] ?? '').split('\n').map((l) => l.split(':').map((s) => s.trim())));
		const kind = (['protocol', 'role', 'rule', 'style', 'report'].includes(meta.kind) && meta.kind !== 'protocol' ? meta.kind : 'rule') as PresetKind;
		addCustom(kind, meta.key || f.name.replace(/\.md$/, '').toLowerCase(), text.slice(fm?.[0].length ?? 0).trim(), `가져오기 · ${f.name}`);
	}

	function save() {
		sel.tok = estimate(sel.body);
		sel.versions.push({ v: last.v + 1, who: '나', when: '방금', note: `수정 · ${sel.tok} tok`, body: sel.body });
	}
	function remove() {
		list = list.filter((p) => p.key !== sel.key);
		selKey = 'frontend';
	}
</script>

<svelte:head><title>Instruction presets · Settings · OrchStack</title></svelte:head>

<main class="page-main">
	<PageHeader title="Instruction presets" desc="모델 컨텍스트에 들어가는 재사용 지침 · 역할 템플릿은 여기서 고르기만 해요 · 수정하면 새 버전" status={false}>
		<input bind:this={fileInput} type="file" accept=".md,text/markdown" class="hidden" onchange={(e) => (importFile(e.currentTarget.files?.[0]), (e.currentTarget.value = ''))} />
		<Button variant="outline" onclick={() => fileInput?.click()}><Download />가져오기 (.md · AGENTS.md)</Button>
		<Button onclick={create}><Plus />새 프리셋</Button>
	</PageHeader>

	<div class="strip strip-primary py-2">
		<Layers class="size-3.5 shrink-0 text-primary" />
		<span class="font-semibold">역할 1개당 기본 조합 ≈ {bundle.total} tok</span>
		<span class="truncate text-muted-foreground">
			protocol {bundle.protocol} + rule {bundle.ruleN}개 {bundle.rule} + style {bundle.style} + report {bundle.report} + role ≈{bundle.role} · 조합 순서: protocol → rule → role → style → report → 저장소 규칙 → 개별 지침
		</span>
	</div>

	<div class="flex items-start gap-5">
		<nav class="list-panel w-72" aria-label="프리셋">
			<InputGroup class="mb-1 h-8">
				<InputGroupAddon><Search /></InputGroupAddon>
				<InputGroupInput bind:value={query} placeholder="프리셋 검색" aria-label="프리셋 검색" />
			</InputGroup>
			{#each kinds as k (k.kind)}
				{@const items = shown(k.kind)}
				{#if items.length}
					<span class="list-label">{k.label}{k.kind === 'protocol' ? '' : ` ${items.length}`}</span>
					{#each items as p (p.key)}
						<button
							type="button"
							aria-current={p.key === sel.key ? 'true' : undefined}
							onclick={() => (selKey = p.key)}
							class={['list-panel-item px-2 py-2', p.key === sel.key ? 'bg-accent font-semibold text-foreground' : 'text-muted-foreground']}
						>
							<k.icon class="size-3.5 shrink-0" />
							<span class="truncate">{p.key} · {p.tok} tok{p.kind === 'style' && p.default ? ' (기본)' : ''}</span>
							{#if !p.builtIn}<Pill class="ml-auto bg-primary-soft text-primary">내 프리셋</Pill>{/if}
						</button>
					{/each}
				{/if}
			{/each}
		</nav>

		<section class="flex min-w-0 flex-1 flex-col gap-3.5">
			<div class="flex items-center gap-2">
				<h2 class="text-lg font-semibold">{title}</h2>
				<Pill class="bg-primary-soft font-mono text-primary uppercase">{sel.kind}</Pill>
				<Pill class="font-mono">v{last.v}</Pill>
				<Pill>{sel.builtIn ? (sel.locked ? '시스템 · 잠김' : '기본 제공') : '내 프리셋'}</Pill>
				<span class="flex-1"></span>
				{#if sel.builtIn}
					{#if !sel.locked}<Button variant="outline" size="sm" onclick={clone}><Copy />복제해서 수정</Button>{/if}
				{:else}
					<!-- 쓰는 곳이 생기면 삭제 대신 보관 (연결은 템플릿 화면에서) -->
					<Button variant="ghost" size="sm" onclick={remove}><Trash2 />삭제</Button>
					<Button size="sm" disabled={sel.body === last.body || checks.length > 0} onclick={save}><Save />저장 · v{last.v + 1}</Button>
				{/if}
			</div>
			<p class="text-xs text-muted-foreground">
				{#if sel.locked}시스템이 쓰는 형식이라 바꿀 수 없어요 · @TASK · @REPORT · @ASK 규격
				{:else if sel.builtIn}기본 제공 프리셋은 잠겨 있어요 · 복제하면 새 프리셋(v1)으로 편집할 수 있어요 · 쓰는 곳이 있으면 삭제 대신 보관
				{:else}저장하면 새 버전이 돼요 · 이미 연결된 곳은 고정한 버전을 계속 써요{/if}
			</p>

			<!-- 프리셋마다 새로 그린다 — 읽기 전용(기본 제공)에서 복제본으로 넘어가면 편집 모드로 -->
			{#key sel.key}
			<MdEditor
				files={[sel]}
				readonly={sel.builtIn}
				tabs={false}
				base={sel.builtIn ? undefined : { [sel.name]: last.body }}
				baseLabel="v{last.v}"
				class="h-80"
			/>
			{/key}

			<div class="meta-xs gap-3">
				<Progress value={Math.min(100, (tok / sel.limit) * 100)} class="h-1.5 w-60 bg-muted" indicator={tok > sel.limit ? 'bg-destructive' : 'bg-status-done'} aria-label="토큰 상한" />
				<span><span class="font-mono">{tok} / {sel.limit} tok</span> · {lang} · {vars ? `변수 ${vars}` : '변수 없음'}</span>
			</div>

			{#if checks.length}
				<div class="strip strip-danger py-2.5">
					<CircleAlert class="size-3.5 text-destructive" /><span class="font-semibold text-destructive">검사 실패</span><span>{checks.join(' · ')}</span>
				</div>
			{:else}
				<div class="strip strip-success py-2.5">
					<CircleCheck class="size-3.5 text-status-done" /><span class="font-semibold">검사 통과</span><span class="text-muted-foreground">상한 이내 · 비밀키 패턴 없음 · 다른 프리셋과 겹치는 규칙 없음</span>
				</div>
			{/if}

			<div class="grid grid-cols-2 items-start gap-5">
				<Card size="sm">
					<CardHeader>
						<CardTitle>사용처</CardTitle>
						<CardDescription>
							{usage.length ? `역할 템플릿 ${usage.filter((u) => u.tpl).length} · 멤버 ${usage.filter((u) => !u.tpl).length}` : '아직 연결된 곳이 없어요 · 템플릿 화면에서 연결해요'}
						</CardDescription>
					</CardHeader>
					<CardContent class="gap-0">
						{#each usage.slice(0, 8) as u (u.label)}
							<KeyValueRow label={u.label}><span class="font-medium">v1 고정</span></KeyValueRow>
						{/each}
						{#if usage.length > 8}<span class="border-t pt-2 text-caption text-subtle-foreground">외 {usage.length - 8}곳</span>{/if}
					</CardContent>
				</Card>
				<Card size="sm">
					<CardHeader>
						<CardTitle>버전</CardTitle>
						<CardDescription>수정할 때마다 새 버전 · 연결은 고정 버전 사용</CardDescription>
					</CardHeader>
					<CardContent class="gap-0">
						{#each sel.versions.toReversed() as v (v.v)}
							<HistoryRow icon={Package} tone="bg-review-soft text-status-review" kind={`v${v.v}`} who={v.who} when={v.when} text={v.note} />
						{/each}
					</CardContent>
				</Card>
			</div>
		</section>
	</div>
</main>
