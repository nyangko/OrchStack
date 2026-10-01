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
	import * as Card from '$lib/components/ui/card';
	import * as Dialog from '$lib/components/ui/dialog';
	import * as Avatar from '$lib/components/ui/avatar';
	import { Button } from '$lib/components/ui/button';
	import { Switch } from '$lib/components/ui/switch';
	import { Pill } from '$lib/components/ui/pill';
	import { RoleAvatar } from '$lib/components/ui/role-avatar';
	import SkillBrowser from '$lib/components/orch/agent/skill-browser.svelte';
	import SkillSourcesDialog from '$lib/components/orch/agent/skill-sources-dialog.svelte';
	import { mcpServers, type Skill, type SkillLog, type TeamMember } from '$lib/mock';
	import { store, sourceMeta, installSkill, glyphOf } from '$lib/teams.svelte';
	import { cn } from '$lib/utils';

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

{#snippet kv(label: string, value: string, tone?: string)}
	<div class="flex h-7.5 items-center border-t text-xs">
		<span class="flex-1 text-muted-foreground">{label}</span>
		<span class={cn('font-medium', tone)}>{value}</span>
	</div>
{/snippet}

{#snippet kpi(label: string, value: string, sub: string)}
	<div class="flex flex-1 flex-col gap-1 rounded-md border bg-card px-3.5 py-3">
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
			<Avatar.Group>{#each u.members.slice(0, 5) as m (m.sn)}<RoleAvatar role={m.role} icon={glyphOf(m)} size="sm" />{/each}</Avatar.Group>
			<span>멤버 {u.members.length}{u.members.length ? ` · ${u.members.map((m) => m.name).join(' · ')}` : ''}</span>
		</span>
		<span>템플릿 {u.templates.length}{u.templates.length ? ` · ${u.templates.map((t) => `${t.name} v${t.version}`).join(' · ')}` : ''}</span>
	</div>
{/snippet}

<div class="flex h-full">
	<nav aria-label="필터" class="flex w-52 shrink-0 flex-col gap-0.5 overflow-y-auto border-r bg-sidebar px-3 py-5">
		{#each navGroups as g (g.label)}
			<span class="list-label px-2 pt-3 pb-1.5 first:pt-0">{g.label}</span>
			{#each g.items as it (it.key)}
				<button
					type="button"
					aria-current={filterKey === it.key ? 'page' : undefined}
					onclick={() => ((filterKey = it.key), (filter = it.f))}
					class={cn(
						'flex h-8 items-center gap-2.5 rounded-md px-2.5 text-body text-muted-foreground outline-none hover:bg-accent/60 focus-visible:ring-3 focus-visible:ring-ring/50',
						filterKey === it.key && 'bg-accent font-semibold text-foreground'
					)}
				>
					<it.icon class="size-3.75 shrink-0" />
					<span class="flex-1 truncate text-left">{it.label}</span>
					<span class="font-mono text-caption">{it.n}</span>
				</button>
			{/each}
		{/each}
	</nav>

	<main class="flex min-w-0 flex-1 flex-col gap-4 overflow-y-auto px-8 py-7 *:shrink-0">
		<header class="flex items-end gap-3">
			<div class="flex flex-1 flex-col gap-1">
				<h1 class="text-2xl font-bold">Skills & MCP</h1>
				<p class="text-xs text-muted-foreground">워크스페이스에 설치된 스킬 · MCP 서버와 어디서 쓰이는지 · 켜고 끄기는 템플릿 · 멤버에서, 소스 설정은 Settings에서</p>
			</div>
			<Button variant="outline" size="sm" onclick={() => (sourcesOpen = true)}><SlidersHorizontal />소스 설정</Button>
			<Button size="sm" onclick={() => (adding = true)}><Plus />스킬 · MCP 추가</Button>
		</header>

		<div class="flex gap-4">
			{@render kpi('스킬', String(store.library.length), `활성 ${activeSkills.length} · 템플릿 ${usedTemplates}개에서 사용`)}
			{@render kpi('MCP 서버', String(mcpServers.length), `정상 ${mcpServers.filter((m) => store.mcp[m.name].installed).length} · 인증 필요 ${mcpServers.filter((m) => store.mcp[m.name].auth).length}`)}
			{@render kpi('Run당 컨텍스트', `+${(avgTok / 1000).toFixed(1)}K`, '활성 스킬 평균 · 권장 8K 이하')}
			{@render kpi('업데이트 · 차단', `${store.library.filter((s) => s.update).length} · ${store.library.filter((s) => s.blocked).length}`, [...store.library.filter((s) => s.update).map((s) => `${s.name} ${s.update}`), ...store.library.filter((s) => s.blocked).map((s) => `${s.name} 차단`)].join(' · ') || '없음')}
		</div>

		<div class="flex items-start gap-4">
			<div class="flex min-w-0 flex-1 flex-col gap-4">
				{#if filter.kind !== 'mcp'}
					<Card.Root size="sm">
						<Card.Header>
							<Card.Title>스킬 · {skills.length}</Card.Title>
							<Card.Description>스위치 = 워크스페이스에서 허용 · 끄면 모든 멤버에서 빠져요</Card.Description>
						</Card.Header>
						<Card.Content class="gap-0">
							{#each skills as s (s.name)}
								{@const on = sel.kind === 'skill' && sel.name === s.name}
								{@const src = sourceMeta[s.source]}
								<div class={cn('-mx-2 flex items-center gap-3 rounded-md border-t px-2 py-2.5 first:border-t-0', on && 'bg-primary-soft', (s.on === false || s.blocked) && 'opacity-60')}>
									<button type="button" onclick={() => (sel = { kind: 'skill', name: s.name })} class="flex min-w-0 flex-1 items-center gap-3 text-left outline-none focus-visible:underline">
										<span class={cn('flex size-8 shrink-0 items-center justify-center rounded-md font-mono text-xs font-semibold text-on-solid uppercase', tileOf(s.name))}>{s.name[0]}</span>
										<span class="flex min-w-0 flex-1 flex-col gap-0.5">
											<span class="font-mono text-body font-medium">{s.name}</span>
											<span class="truncate text-xs text-muted-foreground">{s.blocked ? s.blocked + ' → 차단됨' : s.desc}</span>
											<span class="flex items-center gap-1 text-caption text-subtle-foreground">
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
						</Card.Content>
					</Card.Root>
				{/if}
				{#if servers.length}
					<Card.Root size="sm">
						<Card.Header>
							<Card.Title>MCP 서버 · {servers.length}</Card.Title>
							<Card.Description>설치됨 = 매 Run 컨텍스트에 도구 추가 · 접근 가능 = 허용만</Card.Description>
						</Card.Header>
						<Card.Content class="gap-0">
							{#each servers as m (m.name)}
								{@const st = store.mcp[m.name]}
								{@const n = usersOf(m.name, 'mcp').members.length}
								{@const on = sel.kind === 'mcp' && sel.name === m.name}
								<div class={cn('-mx-2 flex items-center gap-3 rounded-md border-t px-2 py-2.5 first:border-t-0', on && 'bg-primary-soft')}>
									<button type="button" onclick={() => (sel = { kind: 'mcp', name: m.name })} class="flex min-w-0 flex-1 items-center gap-3 text-left outline-none focus-visible:underline">
										<Plug class={cn('size-4 shrink-0', st.installed ? 'text-primary' : 'text-muted-foreground')} />
										<span class="flex min-w-0 flex-1 flex-col gap-0.5">
											<span class="flex items-center gap-1.5 font-mono text-body font-medium">
												{m.name}
												<Pill class={st.installed ? 'bg-success-soft font-sans text-status-done' : 'font-sans'}>{st.installed ? '설치됨' : '접근 가능'}</Pill>
												{#if st.auth}<Pill class="bg-warning-soft font-sans text-status-waiting">인증 필요</Pill>{/if}
											</span>
											<span class="text-xs text-muted-foreground">{m.desc} · {m.tools} tools{st.installed ? ` · +${m.tok} tok` : ' · 필요 시 설치'} · 멤버 {n}</span>
										</span>
									</button>
									<span class={cn('flex items-center gap-1.5 text-caption', st.auth ? 'text-status-waiting' : st.installed ? 'text-status-done' : 'text-muted-foreground')}>
										<span class={cn('size-1.5 rounded-full', st.auth ? 'bg-status-waiting' : st.installed ? 'bg-success' : 'bg-subtle-foreground')}></span>
										{st.auth ? '인증 필요' : st.installed ? '정상' : '미설치'}
									</span>
									{#if st.auth || !st.installed}
										<Button variant="outline" size="sm" onclick={() => mcpAction(m.name)}>{st.auth ? '인증' : '설치'}</Button>
									{/if}
								</div>
							{/each}
						</Card.Content>
					</Card.Root>
				{/if}
			</div>

			<aside class="flex w-80 shrink-0 flex-col gap-4">
				<Card.Root size="sm">
					{#if selSkill}
						{@const s = selSkill}
						<Card.Header>
							<Card.Title class="font-mono">{s.name}</Card.Title>
							<Card.Description>선택한 스킬 · {sourceMeta[s.source].label}</Card.Description>
						</Card.Header>
						<Card.Content class="gap-0">
							{@render kv('버전', s.update ? `${s.version} → ${s.update} 업데이트 가능` : (s.version ?? '—'), s.update ? 'text-status-waiting' : undefined)}
							{@render kv('소스', sourceMeta[s.source].label)}
							{@render kv('컨텍스트', s.blocked ? '—' : `+${s.tok} tok / Run`)}
							{@render kv('보안 검사', s.blocked ? `실패 · ${s.blocked}` : '통과 · sha256 고정', s.blocked ? 'text-destructive' : undefined)}
							{@render kv('Codex CLI', 'AGENTS.md에 요약 포함')}
							{@render kv('Claude Code', '~/.claude/skills 동기화')}
							{@render usage(selUsers)}
							{#if s.update}
								<Button size="sm" class="mt-3 w-fit" onclick={() => update(s)}><Download />{s.update}로 업데이트</Button>
							{/if}
						</Card.Content>
					{:else if selMcp}
						{@const st = store.mcp[selMcp.name]}
						<Card.Header>
							<Card.Title class="font-mono">{selMcp.name}</Card.Title>
							<Card.Description>선택한 MCP 서버</Card.Description>
						</Card.Header>
						<Card.Content class="gap-0">
							{@render kv('상태', st.auth ? '인증 필요' : st.installed ? '설치됨 · 정상' : '접근 가능 · 미설치', st.auth ? 'text-status-waiting' : undefined)}
							{@render kv('도구', `${selMcp.tools} tools`)}
							{@render kv('컨텍스트', st.installed ? `+${selMcp.tok} tok / Run` : '미설치 · 비용 없음')}
							{@render kv('설명', selMcp.desc)}
							{@render usage(selUsers)}
							{#if st.auth || !st.installed}
								<Button size="sm" class="mt-3 w-fit" onclick={() => mcpAction(selMcp.name)}>{st.auth ? '인증하기' : '설치'}</Button>
							{/if}
						</Card.Content>
					{/if}
				</Card.Root>
				<Card.Root size="sm">
					<Card.Header>
						<Card.Title>최근 변경</Card.Title>
						<Card.Description>7일</Card.Description>
					</Card.Header>
					<Card.Content class="gap-0">
						{#each store.log.slice(0, 6) as l, i (i)}
							{@const Icon = logIcon[l.kind]}
							<div class="flex gap-2.5 border-t py-2.5 first:border-t-0">
								<span class={cn('flex size-6 shrink-0 items-center justify-center rounded-full', l.kind === 'BLOCK' ? 'bg-destructive-soft text-destructive' : 'bg-primary-soft text-primary')}><Icon class="size-3" /></span>
								<span class="flex min-w-0 flex-1 flex-col gap-0.5 text-xs">
									<span class="flex items-center gap-1.5"><span class="font-mono text-caption font-semibold text-muted-foreground">{l.kind}</span><span class="text-muted-foreground">{l.who}</span><span class="ml-auto font-mono text-caption text-subtle-foreground">{l.when}</span></span>
									<span>{l.text}</span>
								</span>
							</div>
						{/each}
					</Card.Content>
				</Card.Root>
			</aside>
		</div>
	</main>
</div>

<!-- 스킬 · MCP 추가 — skills.sh에서 워크스페이스 라이브러리로 -->
<Dialog.Root bind:open={adding}>
	<Dialog.Content size="xl" tall>
		<Dialog.Header>
			<Dialog.Title>스킬 · MCP 추가</Dialog.Title>
			<Dialog.Description>워크스페이스 라이브러리에 추가해요. 에이전트별로 켜는 건 템플릿 · 멤버 Skills 탭에서 해요.</Dialog.Description>
		</Dialog.Header>
		<Dialog.Body>
			<SkillBrowser added={(n) => store.library.some((s) => s.name === n)} onadd={(h) => (installSkill(h), (sel = { kind: 'skill', name: h.name }))} target="워크스페이스" />
		</Dialog.Body>
	</Dialog.Content>
</Dialog.Root>

<SkillSourcesDialog bind:open={sourcesOpen} />
