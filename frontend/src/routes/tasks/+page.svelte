<script lang="ts">
	import { page } from '$app/state';
	/// Tasks — 모든 프로젝트의 태스크 (.pen Tasks · 전체 태스크). 좌측 보기 · KPI · 필터 · 프로젝트별 표. 배정은 각 프로젝트 Orch가 한다.
	import { goto } from '$app/navigation';
	import Inbox from '@lucide/svelte/icons/inbox';
	import List from '@lucide/svelte/icons/list';
	import LoaderCircle from '@lucide/svelte/icons/loader-circle';
	import OctagonAlert from '@lucide/svelte/icons/octagon-alert';
	import Eye from '@lucide/svelte/icons/eye';
	import CircleCheck from '@lucide/svelte/icons/circle-check';
	import Folder from '@lucide/svelte/icons/folder';
	import Bookmark from '@lucide/svelte/icons/bookmark';
	import Plus from '@lucide/svelte/icons/plus';
	import X from '@lucide/svelte/icons/x';
	import Search from '@lucide/svelte/icons/search';
	import User from '@lucide/svelte/icons/user';
	import Flag from '@lucide/svelte/icons/flag';
	import ArrowDownUp from '@lucide/svelte/icons/arrow-down-up';
	import ChevronDown from '@lucide/svelte/icons/chevron-down';
	import TriangleAlert from '@lucide/svelte/icons/triangle-alert';
	import MessageCircleQuestion from '@lucide/svelte/icons/message-circle-question';
	import Cpu from '@lucide/svelte/icons/cpu';
	import ListTodo from '@lucide/svelte/icons/list-todo';
	import Sparkles from '@lucide/svelte/icons/sparkles';
	import type { Component } from 'svelte';
	import { DropdownMenu, DropdownMenuTrigger, DropdownMenuContent, DropdownMenuRadioGroup, DropdownMenuRadioItem, DropdownMenuCheckboxItem } from '$lib/components/ui/dropdown-menu';
	import { Empty, EmptyHeader, EmptyMedia, EmptyTitle, EmptyDescription, EmptyContent } from '$lib/components/ui/empty';
	import { InputGroup, InputGroupAddon, InputGroupInput } from '$lib/components/ui/input-group';
	import { Button } from '$lib/components/ui/button';
	import { Toggle } from '$lib/components/ui/toggle';
	import { Pill } from '$lib/components/orch/pill';
	import { Progress } from '$lib/components/ui/progress';
	import { StatusBadge } from '$lib/components/orch/status-badge';
	import { RoleAvatar } from '$lib/components/orch/role-avatar';
	import { RuntimeLogo } from '$lib/components/orch/runtime-logo';
	import { statuses, type TaskStatus } from '$lib/status';
	import { roles } from '$lib/roles';
	import { tasks, projectTasks, decisions, type Task } from '$lib/mock';
	import { store, glyphOf, teamsLoad, loadTeams, loadProjects, loadAllTasks } from '$lib/teams.svelte';
	import { useMock } from '$lib/api/env';
	import { onMount } from 'svelte';
	
	// 모든 프로젝트의 태스크. 목데이터는 OrchStack이 project가 없어 1로 채운다. 서버 모드는 프로젝트별 목록을 합친다(A-3 · 전체 API #89 대기).
	let all = $state<(Task & { project: number })[]>(useMock ? [...tasks, ...projectTasks].map((t) => ({ ...t, project: t.project ?? 1 })) : []);
	let loadState = $state<'loading' | 'ready' | 'error'>(useMock ? 'ready' : 'loading');
	async function load() {
		loadState = 'loading';
		if (teamsLoad.state !== 'ready') await loadTeams();
		const list = (await loadProjects()) ? await loadAllTasks() : undefined;
		if (!list) return void (loadState = 'error');
		all = list;
		loadState = 'ready';
	}
	onMount(() => {
		if (!useMock) void load();
	});
	// 판단 대기 태스크 — 서버 판단 API(#49) 전에는 서버 모드에서 비운다.
	const pending = new Set(useMock ? decisions.filter((d) => !d.decided).map((d) => d.task) : []);
	/// 담당 — 모든 팀 멤버에서 찾는다.
	const memberOf = (sn?: number) => store.crew.flatMap((t) => t.members).find((m) => m.sn === sn);
	const teamOf = (project: string) => store.crew.find((t) => t.project === project);
	const modeLabel = (sn: number) => {
		const p = store.policies[sn];
		return p.mode === 'manual' ? 'Manual' : p.mode === 'full' ? 'Full auto' : `Auto · ${p.timer}초`;
	};

	/// 목록 조건. status: 'todo+'는 Todo · Backlog, inbox는 판단 대기 · 리뷰 대기.
	type Filter = { status?: TaskStatus | 'todo+'; project?: number; agent?: number; priorities: string[]; runtime?: 'claude' | 'codex'; inbox?: boolean };
	// 좌측 보기 — 누르면 필터를 그 조건으로 바꾼다.
	const views: { key: string; label: string; icon: Component; f: Filter }[] = [
		{ key: 'inbox', label: '내 확인 필요', icon: Inbox, f: { inbox: true, priorities: [] } },
		{ key: 'all', label: '전체', icon: List, f: { priorities: [] } },
		{ key: 'in_progress', label: '진행 중', icon: LoaderCircle, f: { status: 'in_progress', priorities: [] } },
		{ key: 'blocked', label: '막힘', icon: OctagonAlert, f: { status: 'blocked', priorities: [] } },
		{ key: 'review', label: '리뷰 대기', icon: Eye, f: { status: 'review', priorities: [] } },
		{ key: 'done', label: '이번 주 완료', icon: CircleCheck, f: { status: 'done', priorities: [] } },
	];
	// 저장된 보기는 화면 상태 (서버 저장은 #44).
	let saved = $state<{ key: string; label: string; f: Filter }[]>([
		{ key: 's1', label: 'P0–P1 · 이번 주', f: { priorities: ['P0', 'P1'] } },
		{ key: 's2', label: 'Claude Code 담당', f: { runtime: 'claude', priorities: [] } },
	]);
	let view = $state('all');
	let f = $state<Filter>({ priorities: [] });
	// 검색(⌘K)의 "Tasks에서 필터로 보기"는 ?q=로 검색어를 넘긴다.
	let query = $state(page.url.searchParams.get('q') ?? '');
	let sort = $state<'updated' | 'priority' | 'num'>('updated');
	let folded = $state<number[]>([]);

	const match = (t: (typeof all)[number], x: Filter, q = '') => {
		const a = memberOf(t.agent);
		if (x.inbox && !(pending.has(t.num) || t.status === 'review')) return false;
		if (x.status === 'todo+' ? !['todo', 'backlog'].includes(t.status) : x.status && t.status !== x.status) return false;
		if (x.project && t.project !== x.project) return false;
		if (x.agent !== undefined && t.agent !== x.agent) return false;
		if (x.priorities.length && !x.priorities.includes(t.priority)) return false;
		if (x.runtime && a?.runtime !== x.runtime) return false;
		return !q || `#${t.num} ${t.title} ${a?.name ?? ''}`.toLowerCase().includes(q.trim().toLowerCase());
	};
	const count = (x: Filter) => all.filter((t) => match(t, x)).length;
	/// 보기 선택 — 필터를 보기 조건으로 바꾼다.
	function pickView(key: string, x: Filter) {
		view = key;
		f = structuredClone(x);
		query = '';
	}

	// 상태 칩 — 나머지 조건은 두고 상태만 바꾼다.
	const chips = [
		{ v: undefined, l: '전체' },
		{ v: 'in_progress', l: '진행 중' },
		{ v: 'blocked', l: '막힘' },
		{ v: 'review', l: '리뷰' },
		{ v: 'todo+', l: 'Todo' },
	] as const;
	/// 지금 필터를 저장된 보기로 남긴다 (이름은 조건에서 만든다).
	function saveView() {
		const parts = [
			f.inbox && '내 확인 필요',
			f.project && store.projects.find((p) => p.sn === f.project)?.name,
			f.status && chips.find((c) => c.v === f.status)?.l,
			f.agent !== undefined && memberOf(f.agent)?.name,
			f.priorities.join('·'),
			f.runtime && (f.runtime === 'claude' ? 'Claude Code' : 'Codex CLI'),
		].filter(Boolean);
		const key = `s${saved.length + 1}-${Date.now()}`;
		saved.push({ key, label: parts.join(' · ') || '내 보기', f: $state.snapshot(f) });
		view = key;
	}

	// "1h ago" 같은 목데이터 문자열을 분으로 (정렬용).
	const minutes = (s: string) => (s === 'now' ? 0 : Number.parseInt(s) * (s.includes('d') ? 1440 : s.includes('h') ? 60 : 1));
	const rank = { P0: 0, P1: 1, P2: 2, P3: 3 };
	const shown = $derived(
		all
			.filter((t) => match(t, f, query))
			.sort((a, b) => (sort === 'updated' ? minutes(a.updated) - minutes(b.updated) : sort === 'priority' ? rank[a.priority] - rank[b.priority] : b.num - a.num))
	);
	const groups = $derived(store.projects.map((p) => ({ p, team: teamOf(p.name), list: shown.filter((t) => t.project === p.sn) })).filter((g) => g.list.length));
	const heading = $derived(
		f.project ? store.projects.find((p) => p.sn === f.project)!.name : `${[...views, ...saved].find((v) => v.key === view)?.label ?? '전체'}${view === 'all' ? ' 태스크' : ''}`
	);
	const assignees = $derived([...new Set(all.map((t) => t.agent).filter((sn) => sn !== undefined))].map((sn) => memberOf(sn)!).filter(Boolean));

	const inProgress = $derived(all.filter((t) => t.status === 'in_progress'));
	const blockedList = $derived(all.filter((t) => t.status === 'blocked'));
	const reviewList = $derived(all.filter((t) => t.status === 'review'));
	const doneList = $derived(all.filter((t) => t.status === 'done'));
</script>

<svelte:head><title>Tasks · OrchStack</title></svelte:head>

{#snippet kpi(label: string, value: number, sub: string, tone?: string)}
	<div class="stat-card">
		<span class="text-xs text-muted-foreground">{label}</span>
		<span class={['font-mono text-xl font-semibold', tone]}>{value}</span>
		<span class="truncate text-caption text-subtle-foreground">{sub}</span>
	</div>
{/snippet}

{#snippet navItem(key: string, label: string, Icon: Component, x: Filter)}
	<button
		type="button"
		aria-current={view === key ? 'page' : undefined}
		onclick={() => pickView(key, x)}
		class={['side-nav-item h-8.5 w-full', view === key ? 'bg-accent font-semibold text-foreground' : 'text-muted-foreground']}
	>
		<Icon class="size-4 shrink-0" />
		<span class="flex-1 truncate text-left">{label}</span>
		<span class="font-mono text-caption">{count(x)}</span>
	</button>
{/snippet}

<div class="flex h-full">
	<nav aria-label="보기" class="side-nav w-65 px-3 py-5">
		<span class="list-label px-2 pt-3 pb-1.5">보기</span>
		{#each views as v (v.key)}{@render navItem(v.key, v.label, v.icon, v.f)}{/each}
		<span class="list-label px-2 pt-3 pb-1.5">프로젝트</span>
		{#each store.projects as p (p.sn)}{@render navItem(`p${p.sn}`, p.name, Folder, { project: p.sn, priorities: [] })}{/each}
		<span class="list-label px-2 pt-3 pb-1.5">저장된 보기</span>
		{#each saved as v (v.key)}
			<div class="group relative">
				{@render navItem(v.key, v.label, Bookmark, v.f)}
				<button
					type="button"
					aria-label="{v.label} 보기 삭제"
					onclick={() => ((saved = saved.filter((x) => x.key !== v.key)), view === v.key && pickView('all', { priorities: [] }))}
					class="absolute top-2.5 right-8 rounded-xs text-subtle-foreground opacity-0 outline-none group-hover:opacity-100 hover:text-foreground focus-visible:opacity-100"
				><X class="size-3.5" /></button>
			</div>
		{/each}
		<button type="button" onclick={saveView} class="side-nav-item h-8.5 text-muted-foreground">
			<Plus class="size-4" />지금 필터로 보기 저장
		</button>
	</nav>

	<main class="flex min-w-0 flex-1 flex-col overflow-y-auto px-8 py-7 *:shrink-0 gap-4.5">
		<header class="flex flex-col gap-1">
			<h1 class="text-2xl font-bold">{heading}</h1>
			<p class="text-xs text-muted-foreground">
				{f.project ? `${teamOf(heading)?.name ?? '팀 없음'} · 태스크 ${shown.length}개` : `모든 프로젝트 · ${shown.length}개 · 프로젝트를 넘나드는 목록이에요. 배정은 각 프로젝트의 Orch가 해요.`}
			</p>
		</header>

		<div class="flex gap-4">
			{@render kpi('진행 중', inProgress.length, `Run ${inProgress.filter((t) => t.run).length}개 실행 중`)}
			{@render kpi('막힘', blockedList.length, blockedList.map((t) => `#${t.num} ${t.title}`).join(' · ') || '없음', blockedList.length ? 'text-status-blocked' : undefined)}
			{@render kpi('리뷰 대기', reviewList.length, reviewList.map((t) => `#${t.num} ${memberOf(t.agent)?.name ?? ''}`).join(' · ') || '없음')}
			{@render kpi('이번 주 완료', doneList.length, `판단 대기 ${pending.size} · PM Dock에서 답해요`)}
		</div>

		<div class="flex flex-wrap items-center gap-2">
			<InputGroup class="h-8 w-64 bg-card">
				<InputGroupAddon><Search /></InputGroupAddon>
				<InputGroupInput bind:value={query} placeholder="태스크 · 이슈 · 담당 검색" aria-label="태스크 검색" />
			</InputGroup>
			{#each chips as c (c.l)}
				<Toggle variant="chip" count={count({ ...f, status: c.v })} pressed={f.status === c.v} onPressedChange={() => (f.status = c.v)}>{c.l}</Toggle>
			{/each}
			<span class="flex-1"></span>
			<DropdownMenu>
				<DropdownMenuTrigger>
					{#snippet child({ props })}<Button variant="outline" size="sm" {...props}><Folder />프로젝트: {f.project ? store.projects.find((p) => p.sn === f.project)?.name : '전체'}</Button>{/snippet}
				</DropdownMenuTrigger>
				<DropdownMenuContent align="end">
					<DropdownMenuRadioGroup bind:value={() => String(f.project ?? ''), (v) => (f.project = v ? Number(v) : undefined)}>
						<DropdownMenuRadioItem value="">전체</DropdownMenuRadioItem>
						{#each store.projects as p (p.sn)}<DropdownMenuRadioItem value={String(p.sn)}>{p.name}</DropdownMenuRadioItem>{/each}
					</DropdownMenuRadioGroup>
				</DropdownMenuContent>
			</DropdownMenu>
			<DropdownMenu>
				<DropdownMenuTrigger>
					{#snippet child({ props })}<Button variant="outline" size="sm" {...props}><User />담당: {f.agent !== undefined ? memberOf(f.agent)?.name : '전체'}</Button>{/snippet}
				</DropdownMenuTrigger>
				<DropdownMenuContent align="end">
					<DropdownMenuRadioGroup bind:value={() => String(f.agent ?? ''), (v) => (f.agent = v ? Number(v) : undefined)}>
						<DropdownMenuRadioItem value="">전체</DropdownMenuRadioItem>
						{#each assignees as m (m.sn)}<DropdownMenuRadioItem value={String(m.sn)}>{m.name} · {roles[m.role].label}</DropdownMenuRadioItem>{/each}
					</DropdownMenuRadioGroup>
				</DropdownMenuContent>
			</DropdownMenu>
			<DropdownMenu>
				<DropdownMenuTrigger>
					{#snippet child({ props })}<Button variant="outline" size="sm" {...props}><Flag />우선순위{f.priorities.length ? `: ${f.priorities.join('·')}` : ''}</Button>{/snippet}
				</DropdownMenuTrigger>
				<DropdownMenuContent align="end">
					{#each ['P0', 'P1', 'P2', 'P3'] as pr (pr)}
						<DropdownMenuCheckboxItem
							checked={f.priorities.includes(pr)}
							onCheckedChange={(on) => (f.priorities = on ? [...f.priorities, pr] : f.priorities.filter((x) => x !== pr))}
						>{pr}</DropdownMenuCheckboxItem>
					{/each}
				</DropdownMenuContent>
			</DropdownMenu>
			<DropdownMenu>
				<DropdownMenuTrigger>
					{#snippet child({ props })}<Button variant="outline" size="sm" {...props}><ArrowDownUp />{sort === 'updated' ? '업데이트순' : sort === 'priority' ? '우선순위순' : '번호순'}</Button>{/snippet}
				</DropdownMenuTrigger>
				<DropdownMenuContent align="end">
					<DropdownMenuRadioGroup bind:value={sort}>
						<DropdownMenuRadioItem value="updated">업데이트순</DropdownMenuRadioItem>
						<DropdownMenuRadioItem value="priority">우선순위순</DropdownMenuRadioItem>
						<DropdownMenuRadioItem value="num">번호순</DropdownMenuRadioItem>
					</DropdownMenuRadioGroup>
				</DropdownMenuContent>
			</DropdownMenu>
		</div>

		{#if loadState !== 'ready'}
			<!-- 서버 모드 불러오기 -->
			<Empty class="card py-16 rounded-lg">
				<EmptyHeader>
					{#if loadState === 'error'}
						<EmptyTitle>태스크를 불러오지 못했어요</EmptyTitle>
						<EmptyDescription>서버 연결을 확인하고 다시 시도하세요.</EmptyDescription>
					{:else}
						<EmptyMedia variant="icon"><LoaderCircle class="animate-spin" /></EmptyMedia>
						<EmptyTitle>태스크를 불러오는 중…</EmptyTitle>
					{/if}
				</EmptyHeader>
				{#if loadState === 'error'}<EmptyContent><Button variant="outline" onclick={load}>다시 시도</Button></EmptyContent>{/if}
			</Empty>
		{:else if groups.length}
			<div class="card overflow-hidden rounded-lg">
				<div class="flex h-9 items-center gap-3 bg-muted px-4 text-caption font-medium text-muted-foreground">
					<span class="w-3.5"></span><span class="w-12">ID</span><span class="flex-1">Title</span><span class="w-28">Status</span><span class="w-36">Progress</span><span class="w-60">담당</span><span class="w-16 text-right">Updated</span>
				</div>
				{#each groups as { p, team, list } (p.sn)}
					{@const open = !folded.includes(p.sn)}
					<div class="flex items-center gap-2 border-t bg-muted px-4 py-2.5 text-xs">
						<button
							type="button"
							aria-expanded={open}
							aria-label="{p.name} {open ? '접기' : '펼치기'}"
							onclick={() => (folded = open ? [...folded, p.sn] : folded.filter((x) => x !== p.sn))}
							class="focus-ring rounded-xs text-muted-foreground hover:text-foreground"
						>
							<ChevronDown class={['size-3.5 transition-transform', !open && '-rotate-90']} />
						</button>
						<Folder class="size-3.5 text-muted-foreground" />
						<span class="font-semibold">{p.name}</span>
						<span class="font-mono text-muted-foreground">{list.length}</span>
						{#if team}<span class="text-muted-foreground">· {team.name} · Orch {modeLabel(team.sn)}</span>{/if}
						<span class="flex-1"></span>
						<a href="/p/{p.sn}" class="font-medium text-primary hover:underline">Workbench 열기 →</a>
					</div>
					{#if open}
						{#each list as t (t.num)}
							{@const st = statuses[t.status]}
							{@const a = memberOf(t.agent)}
							<!-- 행 클릭은 마우스 편의, 키보드는 제목 링크로 연다 -->
							<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
							<div class="flex h-13 cursor-pointer items-center gap-3 border-t px-4 text-xs hover:bg-muted/50" onclick={() => goto(`/p/${t.project}?task=${t.num}`)}>
								<st.icon class={['size-3.5 shrink-0', st.text]} aria-hidden="true" />
								<span class="w-12 font-mono text-muted-foreground">#{t.num}</span>
								<span class="flex min-w-0 flex-1 items-center gap-2">
									<a href="/p/{t.project}?task={t.num}" onclick={(e) => e.stopPropagation()} class="truncate text-body font-medium outline-none hover:underline focus-visible:underline">{t.title}</a>
									{#if t.over}<Pill class="shrink-0 bg-warning-soft text-status-waiting"><TriangleAlert />토큰 초과</Pill>{/if}
									{#if pending.has(t.num)}<Pill class="shrink-0 bg-warning-soft text-status-waiting"><MessageCircleQuestion />판단 대기</Pill>{/if}
								</span>
								<span class="w-28"><StatusBadge status={t.status} /></span>
								<span class="flex w-36 items-center gap-2">
									{#if t.steps[1]}
										<Progress value={(t.steps[0] / t.steps[1]) * 100} class="h-1.5" aria-label="#{t.num} 완료 조건" tip={`#${t.num} 완료 조건\n${t.steps[0]} / ${t.steps[1]} 완료`} />
										<span class="font-mono text-caption text-muted-foreground">{t.steps[0]}/{t.steps[1]}</span>
									{:else}
										<span class="text-caption text-muted-foreground">{t.priority} · 조건 없음</span>
									{/if}
								</span>
								<span class="flex w-60 items-center gap-2">
									{#if a}
										<RoleAvatar role={a.role} icon={glyphOf(a)} size="sm" class="overflow-visible">
											<RuntimeLogo runtime={a.runtime} class="absolute -right-1 -bottom-1 size-2.5 ring-[1.5px]" />
										</RoleAvatar>
										<span class="font-medium">{a.name}</span>
										<span class="text-muted-foreground">{roles[a.role].label}</span>
										{#if t.model}<span class="code-tag truncate"><Cpu class="size-2.5 shrink-0" />{t.model}</span>{/if}
									{:else}
										<span class="text-muted-foreground">미배정</span>
									{/if}
								</span>
								<span class="w-16 text-right font-mono text-caption text-subtle-foreground">{t.updated}</span>
							</div>
						{/each}
					{/if}
				{/each}
			</div>
		{:else}
			<!-- 빈 상태 (.pen 빈 상태 · 새 프로젝트에 태스크 없음) -->
			{@const noTasks = f.project !== undefined && !all.some((t) => t.project === f.project)}
			<Empty class="card py-16 rounded-lg">
				<EmptyHeader>
					<EmptyMedia variant="icon"><ListTodo /></EmptyMedia>
					<EmptyTitle>{noTasks ? '아직 태스크가 없어요' : '조건에 맞는 태스크가 없어요'}</EmptyTitle>
					<EmptyDescription>{noTasks ? 'Orch에게 목표를 알려주면 태스크로 나누고 배정안을 제안해요.' : '필터를 바꾸거나 전체 보기로 돌아가세요.'}</EmptyDescription>
				</EmptyHeader>
				<EmptyContent>
					{#if noTasks}
						<Button href="/p/{f.project}"><Sparkles />Orch에게 계획 요청</Button>
					{:else}
						<Button variant="outline" onclick={() => pickView('all', { priorities: [] })}>전체 보기</Button>
					{/if}
				</EmptyContent>
			</Empty>
		{/if}
	</main>
</div>
