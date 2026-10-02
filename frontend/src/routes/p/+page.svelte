<script lang="ts">
	/// All Projects (.pen All Projects · 프로젝트 목록) — 프로젝트 카드 · 검색 · 활성/보관 · 카드 … 메뉴(열기 · 팀 바꾸기 · 보관 · 삭제).
	/// 카드를 누르면 그 프로젝트 Workbench. 새 프로젝트 다이얼로그는 Workbench 셸(p/+layout)의 ?new.
	import { goto } from '$app/navigation';
	import Plus from '@lucide/svelte/icons/plus';
	import Search from '@lucide/svelte/icons/search';
	import Ellipsis from '@lucide/svelte/icons/ellipsis';
	import GitBranch from '@lucide/svelte/icons/git-branch';
	import ActivityIcon from '@lucide/svelte/icons/activity';
	import Archive from '@lucide/svelte/icons/archive';
	import ArchiveRestore from '@lucide/svelte/icons/archive-restore';
	import Trash2 from '@lucide/svelte/icons/trash-2';
	import Users from '@lucide/svelte/icons/users';
	import SquareArrowOutUpRight from '@lucide/svelte/icons/square-arrow-out-up-right';
	import FolderOpen from '@lucide/svelte/icons/folder-open';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Progress } from '$lib/components/ui/progress';
	import { InputGroup, InputGroupAddon, InputGroupInput } from '$lib/components/ui/input-group';
	import { Empty, EmptyHeader, EmptyMedia, EmptyTitle, EmptyDescription } from '$lib/components/ui/empty';
	import { DropdownMenu, DropdownMenuTrigger, DropdownMenuContent, DropdownMenuLabel, DropdownMenuSeparator, DropdownMenuItem, DropdownMenuSub, DropdownMenuSubTrigger, DropdownMenuSubContent, DropdownMenuRadioGroup, DropdownMenuRadioItem } from '$lib/components/ui/dropdown-menu';
	import { AlertDialog, AlertDialogContent, AlertDialogHeader, AlertDialogTitle, AlertDialogDescription, AlertDialogFooter, AlertDialogCancel, AlertDialogAction } from '$lib/components/ui/alert-dialog';
	import { Segmented } from '$lib/components/orch/segmented';
	import { RoleAvatar } from '$lib/components/orch/role-avatar';
	import { AvatarGroup } from '$lib/components/ui/avatar';
	import { toast } from 'svelte-sonner';
	import { api } from '$lib/api/client';
	import { useMock } from '$lib/api/env';
	import { tasks, projectTasks, issues, decisions } from '$lib/mock';
	import { store } from '$lib/teams.svelte';

	let query = $state('');
	let show = $state<'active' | 'archived' | 'all'>('active');
	let removing = $state<(typeof store.projects)[number]>();

	const archived = (p: { status: string }) => p.status === 'archived';
	const shown = $derived(store.projects.filter((p) => (show === 'all' || (show === 'archived') === archived(p)) && p.name.toLowerCase().includes(query.trim().toLowerCase())));
	const modeLabel = { manual: 'Manual', timer: 'Orch Auto · 타이머', full: 'Full auto' } as const;
	// 목데이터 저장소 · 최근 활동 (서버 모드는 프로젝트 행 · 이벤트에서 — #88)
	const mockMeta: Record<number, { repo: string; last: string; tokens: string }> = {
		1: { repo: 'orchstack/app · main', last: '5분 전 · 진 Run #81', tokens: '오늘 186K tok' },
		2: { repo: 'orchstack/checkout · main', last: '20분 전 · 도윤 Run #12', tokens: '오늘 41K tok' },
		3: { repo: 'orchstack/mobile · develop', last: '어제 · 태오', tokens: '오늘 8K tok' }
	};

	/// 카드 숫자 — 태스크 · 진행 중 · 판단 대기 · 막힘, 이슈 진행(최상위 이슈 중 끝난 수).
	function stats(sn: number) {
		const list = useMock ? (sn === 1 ? tasks : projectTasks.filter((t) => t.project === sn)) : [];
		const top = useMock && sn === 1 ? issues.filter((i) => !i.parent) : [];
		return {
			total: list.length,
			running: list.filter((t) => t.status === 'in_progress').length,
			waiting: useMock && sn === 1 ? decisions.filter((d) => !d.decided).length : 0,
			blocked: list.filter((t) => t.status === 'blocked').length,
			issuesDone: top.filter((i) => i.status === 'done' || i.status === 'closed').length,
			issuesAll: top.length
		};
	}
	const teamOf = (name: string) => store.crew.find((t) => t.project === name);

	async function setArchived(p: (typeof store.projects)[number], on: boolean) {
		if (!useMock) {
			const res = await api.PATCH('/projects/{sn}', { params: { path: { sn: p.sn } }, body: { status: on ? 'archived' : 'active' } }).catch(() => undefined);
			if (!res || res.error) return;
		}
		p.status = on ? 'archived' : 'active';
		toast.success(on ? `${p.name}를 보관했어요` : `${p.name}를 다시 열었어요`);
	}
	/// 팀 바꾸기 — 목데이터는 팀의 연결 프로젝트를 옮긴다. 서버는 PATCH team_sn.
	async function setTeam(p: (typeof store.projects)[number], teamSn: number) {
		if (!useMock) {
			const res = await api.PATCH('/projects/{sn}', { params: { path: { sn: p.sn } }, body: { team_sn: teamSn } }).catch(() => undefined);
			if (!res || res.error) return;
		}
		for (const t of store.crew) if (t.project === p.name) t.project = undefined;
		const t = store.crew.find((x) => x.sn === teamSn);
		if (t) t.project = p.name;
		toast.success(`${p.name} → ${t?.name ?? '팀 없음'}`);
	}
	async function remove() {
		const p = removing;
		if (!p) return;
		if (!useMock) {
			const res = await api.DELETE('/projects/{sn}', { params: { path: { sn: p.sn } } }).catch(() => undefined);
			if (!res || res.error) return;
		}
		store.projects = store.projects.filter((x) => x.sn !== p.sn);
		removing = undefined;
		toast.success(`${p.name}를 삭제했어요`);
	}
</script>

<svelte:head><title>All Projects · OrchStack</title></svelte:head>

<main class="flex h-full flex-col gap-5 overflow-y-auto bg-muted px-8 py-7">
	<header class="flex items-end gap-3">
		<div class="flex min-w-0 flex-1 flex-col gap-1">
			<h1 class="text-2xl font-bold whitespace-nowrap">All Projects</h1>
			<p class="text-xs text-muted-foreground">
				프로젝트 {store.projects.length}개 · 진행 중 {store.projects.filter((p) => !archived(p)).length} · 보관 {store.projects.filter(archived).length} · 카드를 누르면 그 프로젝트 Workbench로
			</p>
		</div>
		<InputGroup class="h-9 w-60 bg-card">
			<InputGroupAddon><Search /></InputGroupAddon>
			<InputGroupInput bind:value={query} placeholder="프로젝트 검색" aria-label="프로젝트 검색" />
		</InputGroup>
		<Segmented
			class="w-72 shrink-0"
			aria-label="보기"
			options={[{ value: 'active', label: `활성 ${store.projects.filter((p) => !archived(p)).length}` }, { value: 'archived', label: `보관 ${store.projects.filter(archived).length}` }, { value: 'all', label: `전체 ${store.projects.length}` }]}
			bind:value={() => show, (v) => (show = (v ?? 'active') as typeof show)}
		/>
		<Button href="/p?new"><Plus />새 프로젝트</Button>
	</header>

	{#if shown.length}
		<div class="grid grid-cols-3 gap-4">
			{#each shown as p (p.sn)}
				{@const st = stats(p.sn)}
				{@const team = teamOf(p.name)}
				{@const meta = mockMeta[p.sn]}
				<!-- 카드 전체가 링크(제목 링크를 카드 크기로 늘림) · … 메뉴는 그 위에 -->
				<article class={['card relative flex flex-col gap-3.5 rounded-lg p-4.5 transition-shadow hover:shadow-md', archived(p) && 'opacity-60']}>
					<div class="flex items-center gap-2">
						<span class={['size-2 rounded-full', p.dot]}></span>
						<a href="/p/{p.sn}" class="text-base font-semibold outline-none after:absolute after:inset-0 after:rounded-lg focus-visible:after:ring-3 focus-visible:after:ring-ring/50">{p.name}</a>
						<Badge variant="secondary">{archived(p) ? '보관됨' : 'Active'}</Badge>
						<span class="flex-1"></span>
						<DropdownMenu>
							<DropdownMenuTrigger>
								{#snippet child({ props })}<Button {...props} variant="ghost" size="icon-xs" class="relative z-10" aria-label="{p.name} 메뉴"><Ellipsis /></Button>{/snippet}
							</DropdownMenuTrigger>
							<DropdownMenuContent align="end" class="w-56">
								<DropdownMenuLabel>{p.name}</DropdownMenuLabel>
								<DropdownMenuItem onSelect={() => goto(`/p/${p.sn}`)}><SquareArrowOutUpRight />Workbench 열기</DropdownMenuItem>
								<DropdownMenuSub>
									<DropdownMenuSubTrigger><Users />팀 바꾸기</DropdownMenuSubTrigger>
									<DropdownMenuSubContent class="w-48">
										<DropdownMenuRadioGroup value={String(team?.sn ?? '')} onValueChange={(v) => setTeam(p, Number(v))}>
											{#each store.crew.filter((t) => !t.orch) as t (t.sn)}<DropdownMenuRadioItem value={String(t.sn)}>{t.name}</DropdownMenuRadioItem>{/each}
										</DropdownMenuRadioGroup>
									</DropdownMenuSubContent>
								</DropdownMenuSub>
								<DropdownMenuSeparator />
								{#if archived(p)}
									<DropdownMenuItem onSelect={() => setArchived(p, false)}><ArchiveRestore />보관 해제</DropdownMenuItem>
								{:else}
									<DropdownMenuItem onSelect={() => setArchived(p, true)}><Archive />보관</DropdownMenuItem>
								{/if}
								<DropdownMenuItem variant="destructive" onSelect={() => (removing = p)}><Trash2 />삭제</DropdownMenuItem>
							</DropdownMenuContent>
						</DropdownMenu>
					</div>
					<span class="meta-line gap-1.5 font-mono"><GitBranch class="size-3" />{meta?.repo ?? '—'}</span>
					<div class="flex items-center gap-2 text-xs">
						{#if team}
							<AvatarGroup>{#each team.members.slice(0, 4) as m (m.sn)}<RoleAvatar role={m.role} size="sm" />{/each}</AvatarGroup>
							<span class="font-medium">{team.name} · {team.members.length}명</span>
						{:else}
							<span class="text-muted-foreground">팀 없음</span>
						{/if}
						<span class="flex-1"></span>
						<span class="text-caption text-muted-foreground">{archived(p) ? '보관됨' : team ? modeLabel[store.policies[team.sn]?.mode ?? 'manual'] : ''}</span>
					</div>
					<div class="flex gap-4">
						{#each [['태스크', st.total, ''], ['진행 중', st.running, ''], ['판단 대기', st.waiting, st.waiting ? 'text-status-waiting' : ''], ['막힘', st.blocked, st.blocked ? 'text-status-blocked' : '']] as [l, v, tone] (l)}
							<span class="flex flex-col gap-0.5"><span class={['font-mono text-base font-semibold', tone]}>{v}</span><span class="text-caption text-muted-foreground">{l}</span></span>
						{/each}
					</div>
					<div class="flex flex-col gap-1.5">
						<span class="flex text-caption"><span class="flex-1 text-muted-foreground">이슈 진행</span>{#if st.issuesAll}<span class="font-mono font-semibold">{st.issuesDone} / {st.issuesAll}</span>{:else}<span class="text-muted-foreground">이슈 없음</span>{/if}</span>
						<Progress value={st.issuesAll ? (st.issuesDone / st.issuesAll) * 100 : 0} class="h-1.5" aria-label="{p.name} 이슈 진행" tip={`${p.name} 이슈 진행\n${st.issuesAll ? `${st.issuesDone} / ${st.issuesAll} 완료` : "이슈 없음"}`} />
					</div>
					<div class="flex items-center gap-1.5 border-t pt-3 text-caption text-muted-foreground">
						{#if archived(p)}<Archive class="size-3" />{:else}<ActivityIcon class="size-3" />{/if}
						<span class="flex-1">{meta?.last ?? '—'}</span>
						<span class="font-mono">{meta?.tokens ?? ''}</span>
					</div>
				</article>
			{/each}
		</div>
	{:else}
		<Empty class="card rounded-lg py-16">
			<EmptyHeader>
				<EmptyMedia variant="icon"><FolderOpen /></EmptyMedia>
				<EmptyTitle>{store.projects.length ? '조건에 맞는 프로젝트가 없어요' : '아직 프로젝트가 없어요'}</EmptyTitle>
				<EmptyDescription>{store.projects.length ? '검색어나 활성 · 보관 보기를 바꿔 보세요.' : '저장소를 연결해 첫 프로젝트를 만들어요.'}</EmptyDescription>
			</EmptyHeader>
		</Empty>
	{/if}
</main>

<AlertDialog bind:open={() => removing !== undefined, (v) => !v && (removing = undefined)}>
	<AlertDialogContent>
		<AlertDialogHeader>
			<AlertDialogTitle>{removing?.name}를 삭제할까요?</AlertDialogTitle>
			<AlertDialogDescription>이슈 · 태스크 · Run 기록이 함께 지워지고 되돌릴 수 없어요. 기록을 남기려면 보관하세요.</AlertDialogDescription>
		</AlertDialogHeader>
		<AlertDialogFooter>
			<AlertDialogCancel>취소</AlertDialogCancel>
			<AlertDialogAction variant="destructive" onclick={remove}>삭제</AlertDialogAction>
		</AlertDialogFooter>
	</AlertDialogContent>
</AlertDialog>
