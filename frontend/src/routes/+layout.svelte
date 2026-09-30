<script lang="ts">
	/// 앱 셸 (.pen GlobalNav). 모든 페이지 상단에 한 번만 그린다.
	import '../app.css';
	import favicon from '$lib/assets/logo/favicon.png';
	import faviconDark from '$lib/assets/logo/favicon-dark.png';
	import logo from '$lib/assets/logo/logo.png';
	import { page } from '$app/state';
	import Users from '@lucide/svelte/icons/users';
	import ListChecks from '@lucide/svelte/icons/list-checks';
	import Blocks from '@lucide/svelte/icons/blocks';
	import Settings from '@lucide/svelte/icons/settings';
	import PlugZap from '@lucide/svelte/icons/plug-zap';
	import Sparkles from '@lucide/svelte/icons/sparkles';
	import Terminal from '@lucide/svelte/icons/terminal';
	import FileStack from '@lucide/svelte/icons/file-stack';
	import FileText from '@lucide/svelte/icons/file-text';
	import Shield from '@lucide/svelte/icons/shield';
	import Search from '@lucide/svelte/icons/search';
	import Bell from '@lucide/svelte/icons/bell';
	import Layers from '@lucide/svelte/icons/layers';
	import Plus from '@lucide/svelte/icons/plus';
	import MessageSquarePlus from '@lucide/svelte/icons/message-square-plus';
	import ListFilter from '@lucide/svelte/icons/list-filter';
	import type { Component } from 'svelte';
	import { goto } from '$app/navigation';
	import * as Command from '$lib/components/ui/command';
	import { Button } from '$lib/components/ui/button';
	import { Kbd } from '$lib/components/ui/kbd';
	import { Toggle } from '$lib/components/ui/toggle';
	import { RoleAvatar } from '$lib/components/ui/role-avatar';
	import { tasks, projectTasks, issues, agents, connections } from '$lib/mock';
	import { statuses } from '$lib/status';
	import { roles } from '$lib/roles';
	import { store, glyphOf } from '$lib/teams.svelte';
	import { cn } from '$lib/utils';

	let { children } = $props();

	// 검색 (⌘K) — 태스크 · 이슈 · 멤버 · 설정 · 명령 (.pen Workbench · 검색 (⌘K) 결과). 목데이터에서 찾는다.
	type Scope = '전체' | '태스크' | '이슈' | '멤버' | '설정' | '명령';
	type Hit = { scope: Exclude<Scope, '전체'>; href: string; label: string; meta?: string; icon?: Component; tone?: string; shortcut?: string; member?: (typeof store.crew)[number]['members'][number] };
	let searching = $state(false);
	let q = $state('');
	let scope = $state<Scope>('전체');
	let selected = $state('');
	const settingsPages: [string, string, string, Component][] = [
		['general', '일반', '워크스페이스 · 언어 · 테마 · 데이터 보관', Settings],
		['connections', '모델 연결', '구독 · API 키 · 게이트웨이 · 로컬', PlugZap],
		['skill-sources', '스킬 소스', 'skills.sh · GitHub · 로컬', Sparkles],
		['runtimes', '실행기 (CLI)', 'Codex · Claude Code · Gemini · 공통 실행 설정', Terminal],
		['presets', 'Instruction presets', '역할 · 규칙 · 말투 · 보고서 지침', FileStack],
		['report-forms', '보고서 양식', '태스크 · 이슈 · PR · 일일 요약', FileText],
		['notifications', '알림', '이벤트 · 채널 · 방해 금지', Bell],
		['security', '권한 · 보안', 'Trust · 승인 규칙 · 차단 · 감사 로그', Shield]
	];
	/// 모델 연결 — 로그인이 필요한 연결을 같이 보여준다.
	const loginNeeded = connections.filter((c) => c.state === 'login');
	const has = (...texts: (string | number | undefined)[]) => texts.some((t) => String(t ?? '').toLowerCase().includes(q.trim().toLowerCase()));
	const hits = $derived.by((): Hit[] => {
		const term = q.trim();
		const agentName = (sn?: number) => agents.find((a) => a.sn === sn)?.name;
		const found: Hit[] = [
			...[...tasks, ...projectTasks]
				.filter((t) => has(t.num, t.title))
				.slice(0, 5)
				.map((t) => {
					const s = statuses[t.status];
					const pj = store.projects.find((p) => p.sn === (t.project ?? 1));
					return { scope: '태스크' as const, href: `/p/${pj?.sn ?? 1}?task=${t.num}`, label: `#${t.num} ${t.title}`, meta: [pj?.name, s.label, agentName(t.agent)].filter(Boolean).join(' · '), icon: s.icon, tone: s.text };
				}),
			...issues.filter((i) => has(i.num, i.title)).slice(0, 3).map((i) => ({ scope: '이슈' as const, href: `/p/1?issue=${i.num}`, label: `#${i.num} ${i.title}`, meta: `태스크 ${tasks.filter((t) => t.issue === i.num).length}`, icon: Layers })),
			...store.crew.flatMap((team) => team.members.filter((m) => has(m.name, m.title)).map((m) => ({ scope: '멤버' as const, href: `/teams?team=${team.sn}`, label: m.name, meta: `${roles[m.role].label} · ${team.name}`, member: m }))).slice(0, 4),
			...settingsPages
				.filter(([, label, desc]) => has(label, desc, 'settings'))
				.map(([key, label, desc, icon]) => ({
					scope: '설정' as const,
					href: `/settings/${key}`,
					label: `Settings › ${label}`,
					meta: key === 'connections' && loginNeeded.length ? `로그인 필요 ${loginNeeded.length} (${loginNeeded.map((c) => c.name.split(' ')[0]).join(' · ')})` : desc,
					icon
				}))
		];
		// 명령 — 입력한 말로 바로 할 수 있는 일 (현재 프로젝트 · 없으면 첫 프로젝트)
		const pj = page.params.project ?? store.projects[0]?.sn ?? 1;
		const quoted = term.length > 12 ? `${term.slice(0, 12)}…` : term;
		const commands: Hit[] = term
			? [
					{ scope: '명령', href: `/p/${pj}?ask=${encodeURIComponent(`새 태스크: ${term}`)}`, label: `새 태스크 만들기 "${quoted}"`, icon: Plus, shortcut: '⌘N' },
					{ scope: '명령', href: `/p/${pj}?ask=${encodeURIComponent(term)}`, label: `Orch에게 지시: "${quoted}"`, icon: MessageSquarePlus, shortcut: '⌘⇧O' },
					{ scope: '명령', href: `/tasks?q=${encodeURIComponent(term)}`, label: `Tasks에서 "${quoted}" 필터로 보기`, icon: ListFilter }
				]
			: [];
		return [...found, ...commands];
	});
	const scopes: Scope[] = ['전체', '태스크', '이슈', '멤버', '설정', '명령'];
	/// 결과 id — 같은 팀 멤버처럼 링크가 같아도 항목은 따로 고른다.
	const idOf = (h: Hit) => `${h.scope}:${h.label}`;
	const shown = $derived(hits.filter((h) => scope === '전체' || h.scope === scope));
	const groups = $derived(scopes.slice(1).map((s) => ({ s, list: shown.filter((h) => h.scope === s) })).filter((g) => g.list.length));

	function openSearch() {
		q = '';
		scope = '전체';
		searching = true;
	}
	function go(href: string, newTab = false) {
		searching = false;
		if (newTab) window.open(href, '_blank');
		else goto(href);
	}
	function onkey(e: KeyboardEvent) {
		if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') {
			e.preventDefault();
			if (searching) searching = false;
			else openSearch();
		}
	}

	// .pen에서 Agents 링크는 숨김(에이전트는 Teams 안에서 관리). 라벨은 i18n 도입 시 교체.
	const links = [
		{ href: '/teams', label: 'Teams', icon: Users },
		{ href: '/tasks', label: 'Tasks', icon: ListChecks },
		{ href: '/skills', label: 'Skills & MCP', icon: Blocks },
		{ href: '/settings', label: 'Settings', icon: Settings }
	];
</script>

<svelte:head>
	<link rel="icon" type="image/png" href={favicon} media="(prefers-color-scheme: light)" />
	<link rel="icon" type="image/png" href={faviconDark} media="(prefers-color-scheme: dark)" />
	<link rel="apple-touch-icon" href="/apple-touch-icon.png" />
	<title>OrchStack</title>
</svelte:head>

<div class="flex h-dvh flex-col">
	<!-- 첫 실행(/setup)은 상단 메뉴 없이 단독 화면 -->
	{#if !page.url.pathname.startsWith('/setup')}
	<header class="flex h-13 shrink-0 items-center gap-6 border-b bg-card px-4">
		<a href="/" class="flex shrink-0 items-center">
			<img src={logo} alt="OrchStack" class="h-5 w-auto" />
		</a>
		<nav class="flex min-w-0 flex-1 items-center gap-0.5 overflow-x-auto" aria-label="주요 메뉴">
			{#each links as l (l.href)}
				{@const on = page.url.pathname.startsWith(l.href)}
				<a
					href={l.href}
					aria-current={on ? 'page' : undefined}
					class={cn(
						'flex h-8 shrink-0 items-center gap-2 rounded-md px-2 text-sm whitespace-nowrap outline-none hover:bg-sidebar-accent focus-visible:ring-3 focus-visible:ring-ring/50',
						on && 'bg-sidebar-accent font-medium'
					)}
				>
					<l.icon class="size-4" />
					{l.label}
				</a>
			{/each}
		</nav>
		<!-- 검색창 모양의 버튼 — 누르거나 ⌘K로 검색 팔레트를 연다 -->
		<button type="button" onclick={openSearch} class="flex h-8 w-75 items-center gap-2 rounded-md border bg-card px-2.5 text-xs text-muted-foreground outline-none hover:bg-muted focus-visible:ring-3 focus-visible:ring-ring/50">
			<Search class="size-4" /><span class="flex-1 text-left">Search tasks, agents, issues</span><Kbd>⌘K</Kbd>
		</button>
		<Button variant="ghost" size="icon" aria-label="알림"><Bell /></Button>
		<span class="flex size-7 items-center justify-center rounded-full bg-primary-soft text-xs font-semibold text-primary" aria-label="사용자">S</span>
	</header>
	{/if}
	<div class="min-h-0 flex-1">
		{@render children()}
	</div>
</div>

<svelte:window onkeydown={onkey} />

<Command.Dialog bind:open={searching} bind:value={selected} shouldFilter={false} title="검색" description="태스크 · 이슈 · 멤버 · 설정 · 명령" class="*:px-0! *:py-0! sm:max-w-180">
	<!-- ⌘↵ 새 탭 · ↵ 열기 · ↑↓ 이동은 Command가 처리 -->
	<div class="contents" role="presentation" onkeydown={(e) => {
		const h = hits.find((x) => idOf(x) === selected);
		if ((e.metaKey || e.ctrlKey) && e.key === 'Enter' && h) {
			e.preventDefault();
			go(h.href, true);
		}
	}}>
		<div class="flex items-center gap-2 border-b p-3">
			<div class="flex-1"><Command.Input bind:value={q} placeholder="태스크 · 이슈 · 멤버 · 설정 검색" /></div>
			<Kbd>esc</Kbd>
		</div>
		<div class="flex gap-1.5 px-3 pt-3" role="group" aria-label="범위">
			{#each scopes as s (s)}
				<Toggle variant="chip" count={s === '전체' ? hits.length : hits.filter((h) => h.scope === s).length} bind:pressed={() => scope === s, (v) => { if (v) scope = s; }}>{s}</Toggle>
			{/each}
		</div>
		<Command.List class="max-h-110 px-1.5 py-1">
			<Command.Empty>{q.trim() ? '찾는 결과가 없어요' : '검색어를 입력하세요'}</Command.Empty>
			{#each groups as g (g.s)}
				<Command.Group heading={g.s}>
					{#each g.list as h (idOf(h))}
						<Command.Item value={idOf(h)} onSelect={() => go(h.href)} class="gap-2.5 py-2">
							{#if h.member}
								<RoleAvatar role={h.member.role} icon={glyphOf(h.member)} size="sm" />
								<span class="font-medium">{h.label}</span><span class="text-caption text-muted-foreground">{h.meta}</span>
							{:else}
								{#if h.icon}<h.icon class={cn('size-4', h.tone ?? 'text-muted-foreground')} />{/if}
								<span class="truncate">{h.label}{h.meta ? ` · ${h.meta}` : ''}</span>
							{/if}
							{#if h.shortcut}<Command.Shortcut>{h.shortcut}</Command.Shortcut>{/if}
						</Command.Item>
					{/each}
				</Command.Group>
			{/each}
		</Command.List>
		<div class="flex items-center gap-4 border-t px-4 py-2.5 text-caption text-muted-foreground">
			{#each [['↑↓', '이동'], ['↵', '열기'], ['⌘↵', '새 탭'], ['esc', '닫기']] as [k, l] (k)}<span class="flex items-center gap-1.5"><Kbd>{k}</Kbd>{l}</span>{/each}
		</div>
	</div>
</Command.Dialog>
