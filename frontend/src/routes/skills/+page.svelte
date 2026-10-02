<script lang="ts">
	/// Skills & MCP — 워크스페이스에 설치된 스킬 · MCP 서버와 쓰이는 곳 (.pen Skills & MCP · 워크스페이스 라이브러리).
	/// 켜고 끄기는 워크스페이스 단위, 에이전트별 설정은 템플릿 · 멤버 Skills 탭에서. 소스 설정은 여기와 Settings에서.
	import Layers from '@lucide/svelte/icons/layers';
	import Sparkles from '@lucide/svelte/icons/sparkles';
	import Plug from '@lucide/svelte/icons/plug';
	import CircleArrowUp from '@lucide/svelte/icons/circle-arrow-up';
	import KeyRound from '@lucide/svelte/icons/key-round';
	import ShieldBan from '@lucide/svelte/icons/shield-ban';
	import SlidersHorizontal from '@lucide/svelte/icons/sliders-horizontal';
	import Plus from '@lucide/svelte/icons/plus';
	import Download from '@lucide/svelte/icons/download';
	import RefreshCw from '@lucide/svelte/icons/refresh-cw';
	import Ban from '@lucide/svelte/icons/ban';
	import ToggleRight from '@lucide/svelte/icons/toggle-right';
	import type { Component } from 'svelte';
	import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '$lib/components/ui/card';
	import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogBody } from '$lib/components/ui/dialog';
	import { AvatarGroup } from '$lib/components/ui/avatar';
	import { Button } from '$lib/components/ui/button';
	import { PageHeader } from '$lib/components/orch/page-header';
	import { Switch } from '$lib/components/ui/switch';
	import { Pill } from '$lib/components/orch/pill';
	import { RoleAvatar } from '$lib/components/orch/role-avatar';
	import { KeyValueRow } from '$lib/components/orch/key-value-row';
	import { HistoryRow } from '$lib/components/orch/history-row';
	import SkillBrowser from '$lib/components/orch/agent/skill-browser.svelte';
	import SkillSourcesDialog from '$lib/components/orch/agent/skill-sources-dialog.svelte';
	import { mcpServers, type Skill, type SkillLog, type TeamMember } from '$lib/mock';
	import { store, sourceMeta, installSkill, glyphOf } from '$lib/teams.svelte';
	
	/// 멤버의 실제 설정 — 아직 복사 전이면 원본 템플릿 설정.
	const cfgOf = (m: TeamMember) => m.config ?? store.templates.find((t) => t.name === m.title)?.config;
	const members = $derived(store.crew.flatMap((t) => t.members));
	/// 이 스킬 · MCP를 쓰는 멤버와 템플릿.
	const usersOf = (name: string, kind: 'skill' | 'mcp') => {
		const has = (c?: { skills: string[]; mcp: string[] }) => !!c && (kind === 'skill' ? c.skills : c.mcp).includes(name);
		return { members: members.filter((m) => has(cfgOf(m))), templates: store.templates.filter((t) => has(t.config)) };
	};

	// 좌측 필터 — 종류 · 소스 · 상태 중 하나.
	type Filter = { kind?: 'skill' | 'mcp'; source?: Skill['source']; status?: 'update' | 'auth' | 'blocked' };
	let filter = $state<Filter>({});
	let filterKey = $state('all');
	const skills = $derived(
		store.library.filter(
			(s) =>
				filter.kind !== 'mcp' &&
				(!filter.source || s.source === filter.source) &&
				(!filter.status || (filter.status === 'update' && s.update) || (filter.status === 'blocked' && s.blocked))
		)
	);
	const servers = $derived(mcpServers.filter((m) => filter.kind !== 'skill' && !filter.source && (!filter.status || (filter.status === 'auth' && store.mcp[m.name].auth))));
	const navGroups = $derived([
		{ label: '종류', items: [
			{ key: 'all', label: '전체', icon: Layers, n: store.library.length + mcpServers.length, f: {} },
			{ key: 'skill', label: '스킬', icon: Sparkles, n: store.library.length, f: { kind: 'skill' } },
			{ key: 'mcp', label: 'MCP 서버', icon: Plug, n: mcpServers.length, f: { kind: 'mcp' } },
		] },
		{ label: '소스', items: (Object.keys(sourceMeta) as Skill['source'][]).map((src) => ({ key: `src-${src}`, label: sourceMeta[src].label, icon: sourceMeta[src].icon, n: store.library.filter((s) => s.source === src).length, f: { source: src } })) },
		{ label: '상태', items: [
			{ key: 'update', label: '업데이트 가능', icon: CircleArrowUp, n: store.library.filter((s) => s.update).length, f: { status: 'update' } },
			{ key: 'auth', label: '인증 필요', icon: KeyRound, n: mcpServers.filter((m) => store.mcp[m.name].auth).length, f: { status: 'auth' } },
			{ key: 'blocked', label: '차단됨', icon: ShieldBan, n: store.library.filter((s) => s.blocked).length, f: { status: 'blocked' } },
		] },
	] satisfies { label: string; items: { key: string; label: string; icon: Component; n: number; f: Filter }[] }[]);

	// 선택 — 오른쪽 상세 카드.
	let sel = $state<{ kind: 'skill' | 'mcp'; name: string }>({ kind: 'skill', name: 'svelte-ui' });
	const selSkill = $derived(sel.kind === 'skill' ? store.library.find((s) => s.name === sel.name) : undefined);
	const selMcp = $derived(sel.kind === 'mcp' ? mcpServers.find((m) => m.name === sel.name) : undefined);
	const selUsers = $derived(usersOf(sel.name, sel.kind));

	const activeSkills = $derived(store.library.filter((s) => s.on !== false && !s.blocked));
	const usedTemplates = $derived(store.templates.filter((t) => t.config.skills.some((n) => activeSkills.some((s) => s.name === n))).length);
	// Run당 컨텍스트 — 템플릿별 켜진 스킬 토큰 평균.
	const avgTok = $derived(
		Math.round(store.templates.reduce((sum, t) => sum + t.config.skills.reduce((n, k) => n + (activeSkills.find((s) => s.name === k)?.tok ?? 0), 0), 0) / Math.max(1, store.templates.length))
	);

	const log = (kind: SkillLog['kind'], text: string) => store.log.unshift({ kind, who: '나', when: '방금', text });
	function toggle(s: Skill, on: boolean) {
		s.on = on;
		log('TOGGLE', `${s.name} ${on ? '켬' : '끔'} · 워크스페이스 전체`);
	}
	function update(s: Skill) {
		log('UPDATE', `${s.name} ${s.version} → ${s.update} · ${sourceMeta[s.source].label}`);
		s.version = s.update;
		s.update = undefined;
	}
	function mcpAction(name: string) {
		const st = store.mcp[name];
		if (st.auth) {
			st.auth = false;
			log('AUTH', `${name} 인증 완료`);
		} else if (!st.installed) {
			st.installed = true;
			log('INSTALL', `${name} 설치 · 워크스페이스`);
		}
		sel = { kind: 'mcp', name };
	}

	let adding = $state(false);
	let sourcesOpen = $state(false);
	const logIcon: Record<SkillLog['kind'], Component> = { INSTALL: Download, UPDATE: RefreshCw, BLOCK: Ban, TOGGLE: ToggleRight, AUTH: KeyRound };
	// 스킬 타일 색 — 이름 첫 글자로 고른다.
	const tiles = ['bg-role-frontend', 'bg-role-backend', 'bg-role-qa', 'bg-role-designer', 'bg-role-reviewer', 'bg-primary'];
	const tileOf = (name: string) => tiles[name.charCodeAt(0) % tiles.length];
</script>

<svelte:head><title>Skills & MCP · OrchStack</title></svelte:head>


{#snippet kpi(label: string, value: string, sub: string)}
	<div class="stat-card">
		<span class="text-xs text-muted-foreground">{label}</span>
		<span class="font-mono text-xl font-semibold">{value}</span>
		<span class="truncate text-caption text-subtle-foreground">{sub}</span>
	</div>
{/snippet}

<!-- 쓰는 멤버 · 템플릿 -->
{#snippet usage(u: { members: TeamMember[]; templates: { name: string; version: number }[] })}
	<div class="flex flex-col gap-2 border-t pt-2.5 text-xs">
		<span class="text-muted-foreground">사용 중</span>
		<span class="flex items-center gap-2">
			<AvatarGroup>{#each u.members.slice(0, 5) as m (m.sn)}<RoleAvatar role={m.role} icon={glyphOf(m)} size="sm" />{/each}</AvatarGroup>
			<span>멤버 {u.members.length}{u.members.length ? ` · ${u.members.map((m) => m.name).join(' · ')}` : ''}</span>
		</span>
		<span>템플릿 {u.templates.length}{u.templates.length ? ` · ${u.templates.map((t) => `${t.name} v${t.version}`).join(' · ')}` : ''}</span>
	</div>
{/snippet}

<div class="flex h-full">
	<nav aria-label="필터" class="side-nav w-52 px-3 py-5">
		{#each navGroups as g (g.label)}
			<span class="list-label px-2 pt-3 pb-1.5 first:pt-0">{g.label}</span>
			{#each g.items as it (it.key)}
				<button
					type="button"
					aria-current={filterKey === it.key ? 'page' : undefined}
					onclick={() => ((filterKey = it.key), (filter = it.f))}
					class={['side-nav-item h-8', filterKey === it.key ? 'bg-accent font-semibold text-foreground' : 'text-muted-foreground']}
				>
					<it.icon class="size-4 shrink-0" />
					<span class="flex-1 truncate text-left">{it.label}</span>
					<span class="font-mono text-caption">{it.n}</span>
				</button>
			{/each}
		{/each}
	</nav>

	<main class="flex min-w-0 flex-1 flex-col overflow-y-auto px-8 py-7 *:shrink-0 gap-4">
		<PageHeader title="Skills & MCP" desc="워크스페이스에 설치된 스킬 · MCP 서버와 어디서 쓰이는지 · 켜고 끄기는 템플릿 · 멤버에서, 소스 설정은 Settings에서" status={false}>
			<Button variant="outline" size="sm" onclick={() => (sourcesOpen = true)}><SlidersHorizontal />소스 설정</Button>
			<Button size="sm" onclick={() => (adding = true)}><Plus />스킬 · MCP 추가</Button>
		</PageHeader>

		<div class="flex gap-4">
			{@render kpi('스킬', String(store.library.length), `활성 ${activeSkills.length} · 템플릿 ${usedTemplates}개에서 사용`)}
			{@render kpi('MCP 서버', String(mcpServers.length), `정상 ${mcpServers.filter((m) => store.mcp[m.name].installed).length} · 인증 필요 ${mcpServers.filter((m) => store.mcp[m.name].auth).length}`)}
			{@render kpi('Run당 컨텍스트', `+${(avgTok / 1000).toFixed(1)}K`, '활성 스킬 평균 · 권장 8K 이하')}
			{@render kpi('업데이트 · 차단', `${store.library.filter((s) => s.update).length} · ${store.library.filter((s) => s.blocked).length}`, [...store.library.filter((s) => s.update).map((s) => `${s.name} ${s.update}`), ...store.library.filter((s) => s.blocked).map((s) => `${s.name} 차단`)].join(' · ') || '없음')}
		</div>

		<div class="flex items-start gap-4">
			<div class="flex min-w-0 flex-1 flex-col gap-4">
				{#if filter.kind !== 'mcp'}
					<Card size="sm">
						<CardHeader>
							<CardTitle>스킬 · {skills.length}</CardTitle>
							<CardDescription>스위치 = 워크스페이스에서 허용 · 끄면 모든 멤버에서 빠져요</CardDescription>
						</CardHeader>
						<CardContent class="gap-0">
							{#each skills as s (s.name)}
								{@const on = sel.kind === 'skill' && sel.name === s.name}
								{@const src = sourceMeta[s.source]}
								<div class={['list-row -mx-2 rounded-md px-2', on && 'bg-primary-soft', (s.on === false || s.blocked) && 'opacity-60']}>
									<button type="button" onclick={() => (sel = { kind: 'skill', name: s.name })} class="row-link">
										<span class={['flex size-8 shrink-0 items-center justify-center rounded-md font-mono text-xs font-semibold text-on-solid uppercase', tileOf(s.name)]}>{s.name[0]}</span>
										<span class="flex min-w-0 flex-1 flex-col gap-0.5">
											<span class="font-mono text-body font-medium">{s.name}</span>
											<span class="truncate text-xs text-muted-foreground">{s.blocked ? s.blocked + ' → 차단됨' : s.desc}</span>
											<span class="subtle-meta gap-1">
												<src.icon class="size-3" />{src.label}{s.version ? ` · ${s.version}` : ''}
												{#if s.update}<span class="text-status-waiting">→ {s.update} 업데이트</span>{/if}
												<span class="ml-2 font-mono">{s.blocked ? '—' : `+${s.tok} tok`}</span>
											</span>
										</span>
									</button>
									<Switch checked={s.on !== false && !s.blocked} disabled={!!s.blocked} onCheckedChange={(v) => toggle(s, v)} aria-label="{s.name} 워크스페이스에서 허용" />
								</div>
							{:else}
								<p class="text-xs text-muted-foreground">조건에 맞는 스킬이 없어요.</p>
							{/each}
						</CardContent>
					</Card>
				{/if}
				{#if servers.length}
					<Card size="sm">
						<CardHeader>
							<CardTitle>MCP 서버 · {servers.length}</CardTitle>
							<CardDescription>설치됨 = 매 Run 컨텍스트에 도구 추가 · 접근 가능 = 허용만</CardDescription>
						</CardHeader>
						<CardContent class="gap-0">
							{#each servers as m (m.name)}
								{@const st = store.mcp[m.name]}
								{@const n = usersOf(m.name, 'mcp').members.length}
								{@const on = sel.kind === 'mcp' && sel.name === m.name}
								<div class={['list-row -mx-2 rounded-md px-2', on && 'bg-primary-soft']}>
									<button type="button" onclick={() => (sel = { kind: 'mcp', name: m.name })} class="row-link">
										<Plug class={['size-4 shrink-0', st.installed ? 'text-primary' : 'text-muted-foreground']} />
										<span class="flex min-w-0 flex-1 flex-col gap-0.5">
											<span class="row-title font-mono">
												{m.name}
												<Pill class={st.installed ? 'bg-success-soft font-sans text-status-done' : 'font-sans'}>{st.installed ? '설치됨' : '접근 가능'}</Pill>
												{#if st.auth}<Pill class="bg-warning-soft font-sans text-status-waiting">인증 필요</Pill>{/if}
											</span>
											<span class="text-xs text-muted-foreground">{m.desc} · {m.tools} tools{st.installed ? ` · +${m.tok} tok` : ' · 필요 시 설치'} · 멤버 {n}</span>
										</span>
									</button>
									<span class={['flex items-center gap-1.5 text-caption', st.auth ? 'text-status-waiting' : st.installed ? 'text-status-done' : 'text-muted-foreground']}>
										<span class={['size-1.5 rounded-full', st.auth ? 'bg-status-waiting' : st.installed ? 'bg-success' : 'bg-subtle-foreground']}></span>
										{st.auth ? '인증 필요' : st.installed ? '정상' : '미설치'}
									</span>
									{#if st.auth || !st.installed}
										<Button variant="outline" size="sm" onclick={() => mcpAction(m.name)}>{st.auth ? '인증' : '설치'}</Button>
									{/if}
								</div>
							{/each}
						</CardContent>
					</Card>
				{/if}
			</div>

			<aside class="flex shrink-0 flex-col w-80 gap-4">
				<Card size="sm">
					{#if selSkill}
						{@const s = selSkill}
						<CardHeader>
							<CardTitle class="font-mono">{s.name}</CardTitle>
							<CardDescription>선택한 스킬 · {sourceMeta[s.source].label}</CardDescription>
						</CardHeader>
						<CardContent class="gap-0">
							<KeyValueRow label="버전"><span class={['text-body font-medium', s.update ? 'text-status-waiting' : undefined]}>{s.update ? `${s.version} → ${s.update} 업데이트 가능` : (s.version ?? '—')}</span></KeyValueRow>
							<KeyValueRow label="소스" value={sourceMeta[s.source].label} />
							<KeyValueRow label="컨텍스트" value={s.blocked ? '—' : `+${s.tok} tok / Run`} />
							<KeyValueRow label="보안 검사"><span class={['text-body font-medium', s.blocked ? 'text-destructive' : undefined]}>{s.blocked ? `실패 · ${s.blocked}` : '통과 · sha256 고정'}</span></KeyValueRow>
							<KeyValueRow label="Codex CLI" value="AGENTS.md에 요약 포함" />
							<KeyValueRow label="Claude Code" value="~/.claude/skills 동기화" />
							{@render usage(selUsers)}
							{#if s.update}
								<Button size="sm" class="mt-3 w-fit" onclick={() => update(s)}><Download />{s.update}로 업데이트</Button>
							{/if}
						</CardContent>
					{:else if selMcp}
						{@const st = store.mcp[selMcp.name]}
						<CardHeader>
							<CardTitle class="font-mono">{selMcp.name}</CardTitle>
							<CardDescription>선택한 MCP 서버</CardDescription>
						</CardHeader>
						<CardContent class="gap-0">
							<KeyValueRow label="상태"><span class={['text-body font-medium', st.auth ? 'text-status-waiting' : undefined]}>{st.auth ? '인증 필요' : st.installed ? '설치됨 · 정상' : '접근 가능 · 미설치'}</span></KeyValueRow>
							<KeyValueRow label="도구" value={`${selMcp.tools} tools`} />
							<KeyValueRow label="컨텍스트" value={st.installed ? `+${selMcp.tok} tok / Run` : '미설치 · 비용 없음'} />
							<KeyValueRow label="설명" value={selMcp.desc} />
							{@render usage(selUsers)}
							{#if st.auth || !st.installed}
								<Button size="sm" class="mt-3 w-fit" onclick={() => mcpAction(selMcp.name)}>{st.auth ? '인증하기' : '설치'}</Button>
							{/if}
						</CardContent>
					{/if}
				</Card>
				<Card size="sm">
					<CardHeader>
						<CardTitle>최근 변경</CardTitle>
						<CardDescription>7일</CardDescription>
					</CardHeader>
					<CardContent class="gap-0">
						{#each store.log.slice(0, 6) as l, i (i)}
							{@const Icon = logIcon[l.kind]}
							<HistoryRow icon={Icon} tone={l.kind === 'BLOCK' ? 'bg-destructive-soft text-destructive' : 'bg-primary-soft text-primary'} kind={l.kind} who={l.who} when={l.when} text={l.text} class="first:border-t-0" />
						{/each}
					</CardContent>
				</Card>
			</aside>
		</div>
	</main>
</div>

<!-- 스킬 · MCP 추가 — skills.sh에서 워크스페이스 라이브러리로 -->
<Dialog bind:open={adding}>
	<DialogContent size="xl" tall>
		<DialogHeader>
			<DialogTitle>스킬 · MCP 추가</DialogTitle>
			<DialogDescription>워크스페이스 라이브러리에 추가해요. 에이전트별로 켜는 건 템플릿 · 멤버 Skills 탭에서 해요.</DialogDescription>
		</DialogHeader>
		<DialogBody>
			<SkillBrowser added={(n) => store.library.some((s) => s.name === n)} onadd={(h) => (installSkill(h), (sel = { kind: 'skill', name: h.name }))} target="워크스페이스" />
		</DialogBody>
	</DialogContent>
</Dialog>

<SkillSourcesDialog bind:open={sourcesOpen} />
