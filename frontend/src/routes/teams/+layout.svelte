<script lang="ts">
	/// Teams 셸 — 좌측 Workspace 목록 (Teams · Agents 전환). /teams는 팀, /teams/agents/[template]는 템플릿.
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import Plus from '@lucide/svelte/icons/plus';
	import Search from '@lucide/svelte/icons/search';
	import Users from '@lucide/svelte/icons/users';
	import LayoutTemplate from '@lucide/svelte/icons/layout-template';
	import LoaderCircle from '@lucide/svelte/icons/loader-circle';
	import CirclePause from '@lucide/svelte/icons/circle-pause';
	import { AvatarGroup } from '$lib/components/ui/avatar';
	import { InputGroup, InputGroupAddon, InputGroupInput } from '$lib/components/ui/input-group';
	import { Button } from '$lib/components/ui/button';
	import { Segmented } from '$lib/components/orch/segmented';
	import { RoleAvatar } from '$lib/components/orch/role-avatar';
	import { store, defaultTeam, glyphOf, membersOf, teamsLoad, loadTeams, emptyStats, runtimeName } from '$lib/teams.svelte';
	import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogBody, DialogFooter } from '$lib/components/ui/dialog';
	import { FieldRow } from '$lib/components/ui/field';
	import { Input } from '$lib/components/ui/input';
	import { Select, SelectTrigger, SelectContent, SelectItem } from '$lib/components/ui/select';
	import { ChoiceCards, ChoiceCard } from '$lib/components/orch/choice-cards';
	import { Badge } from '$lib/components/ui/badge';
	import UsersRound from '@lucide/svelte/icons/users-round';
	import { toast } from 'svelte-sonner';
	import { api } from '$lib/api/client';
	import { useMock } from '$lib/api/env';
	import type { TeamMember } from '$lib/mock';
	import { Empty, EmptyHeader, EmptyTitle, EmptyDescription, EmptyContent } from '$lib/components/ui/empty';
	
	let { children } = $props();

	// 서버 모드: 팀 · 멤버를 한 번 읽는다 (A-2 #93). 목데이터 모드는 처음부터 ready.
	$effect(() => {
		if (teamsLoad.state === 'idle') void loadTeams();
	});

	const scope = $derived(page.route.id?.startsWith('/teams/agents') ? 'agents' : 'teams');
	const teamSn = $derived(scope === 'teams' ? Number(page.url.searchParams.get('team')) || defaultTeam()?.sn : undefined);
	const tplSn = $derived(scope === 'agents' ? Number(page.params.template) : undefined);

	// 새 팀 (.pen Teams · 새 팀) — 시작 구성: 추천(템플릿 4개로 멤버) · 빈 팀 · 기본 팀 복제
	const starts = [
		{ v: 'recommend', t: '추천 구성 · 웹 앱', d: 'Frontend · Backend · QA · Reviewer 4명 — 템플릿을 복사해 만들어요' },
		{ v: 'empty', t: '빈 팀', d: '멤버 없이 시작 · ‘멤버 추가’로 하나씩' },
		{ v: 'copy', t: `${defaultTeam()?.name ?? '기본 팀'} 복제`, d: '멤버 · 실행 한도 · 리뷰 · Orch 정책을 그대로' }
	] as const;
	let newTeam = $state<{ name: string; desc: string; start: (typeof starts)[number]['v']; project: string }>();
	let creating = $state(false);
	const recommendRoles = ['frontend', 'backend', 'qa', 'reviewer'];
	/// 템플릿 → 새 멤버 (목데이터). 이름은 템플릿 이름으로 시작해 나중에 바꾼다.
	function fromTemplates(start: number): TeamMember[] {
		return recommendRoles.flatMap((r, i) => {
			const t = store.templates.find((x) => x.role === r);
			return t ? [{ sn: start + i, name: t.name, role: t.role, title: t.name, runtime: t.runtime, model: `${runtimeName(t.runtime)} · ${t.model}`, status: 'idle' as const, work: '배정 대기', context: 0, tokens: 0, load: [], loadNote: '여유' }] : [];
		});
	}
	async function createTeam() {
		const f = newTeam;
		if (!f || !f.name.trim() || creating) return;
		const name = f.name.trim();
		let sn: number;
		if (useMock) {
			sn = Math.max(0, ...store.crew.map((t) => t.sn)) + 1;
			const memberStart = Math.max(0, ...store.crew.flatMap((t) => t.members.map((m) => m.sn))) + 1;
			const base = defaultTeam();
			const members = f.start === 'recommend' ? fromTemplates(memberStart) : f.start === 'copy' ? base.members.map((m, i) => ({ ...structuredClone($state.snapshot(m)), sn: memberStart + i })) : [];
			store.crew.push({ sn, name, desc: f.desc, project: f.project === 'later' ? undefined : f.project, members, stats: structuredClone(emptyStats) });
			store.policies[sn] = structuredClone($state.snapshot(store.policies[base.sn]));
		} else {
			creating = true;
			const res = await api.POST('/teams', { body: { name } }).catch(() => undefined);
			if (!res?.data) return void (creating = false);
			sn = res.data.sn;
			if (f.start === 'recommend')
				for (const r of recommendRoles) {
					const t = store.templates.find((x) => x.role === r);
					if (t) await api.POST('/teams/{sn}/members', { params: { path: { sn } }, body: { name: t.name, role_name: t.name, first_task_mode: 'orch' } }).catch(() => undefined);
				}
			teamsLoad.state = 'idle';
			await loadTeams();
			creating = false;
		}
		newTeam = undefined;
		toast.success(`${name} 팀을 만들었어요`);
		goto(`/teams?team=${sn}`);
	}

	let query = $state('');
	const shown = $derived(store.crew.filter((t) => t.name.toLowerCase().includes(query.trim().toLowerCase())));
	let tplQuery = $state('');
	const tplGroups = $derived(
		[...new Set(store.templates.map((t) => t.group))].map((g) => ({ g, list: store.templates.filter((t) => t.group === g && t.name.toLowerCase().includes(tplQuery.trim().toLowerCase())) }))
	);
</script>

{#if teamsLoad.state === 'ready'}
<div class="flex h-full">
<aside class="flex w-70 shrink-0 flex-col gap-1 overflow-y-auto border-r bg-sidebar px-3 py-4">
		<div class="flex items-center px-1 pb-2">
			<h2 class="flex-1 text-sm font-semibold">Workspace</h2>
			<Button variant="ghost" size="icon-sm" class="size-7 text-muted-foreground" aria-label="새 팀" onclick={() => (newTeam = { name: '', desc: '', start: 'recommend', project: 'later' })}><Plus class="size-3.5" /></Button>
		</div>
		<Segmented
			class="mb-1"
			aria-label="보기"
			options={[{ value: 'teams', label: `Teams · ${store.crew.length}`, icon: Users }, { value: 'agents', label: `Agents · ${store.templates.length}`, icon: LayoutTemplate }]}
			bind:value={() => scope, (v) => goto(v === 'agents' ? `/teams/agents/${tplSn ?? store.templates[0].sn}` : `/teams?team=${teamSn ?? defaultTeam().sn}`)}
		/>
		{#if scope === 'agents'}
			<InputGroup class="h-8">
				<InputGroupAddon><Search /></InputGroupAddon>
				<InputGroupInput bind:value={tplQuery} placeholder="템플릿 검색" aria-label="템플릿 검색" />
			</InputGroup>
			{#each tplGroups as { g, list } (g)}
				{#if list.length}
					<span class="list-label px-1 pt-3.5 pb-1">{g}</span>
					{#each list as t (t.sn)}
						{@const on = t.sn === tplSn}
						{@const n = membersOf(t.name).length}
						<a
							href="/teams/agents/{t.sn}"
							aria-current={on ? 'page' : undefined}
							class={['flex items-center gap-2.5 rounded-md px-2 py-2 outline-none hover:bg-accent/60 focus-visible:ring-3 focus-visible:ring-ring/50', on && 'bg-accent']}
						>
							<RoleAvatar role={t.role} />
							<span class="flex min-w-0 flex-1 flex-col gap-1">
								<span class="truncate text-xs font-semibold">{t.name}</span>
								<span class="text-caption text-muted-foreground">v{t.version} · {n ? `멤버 ${n}` : '미사용'}{t.draft ? ' · 초안' : ''}</span>
							</span>
						</a>
					{/each}
				{/if}
			{/each}
		{:else}
		<InputGroup class="h-8">
			<InputGroupAddon><Search /></InputGroupAddon>
			<InputGroupInput bind:value={query} placeholder="Filter teams" aria-label="팀 검색" />
		</InputGroup>
		{#each [true, false] as orch (orch)}
			{@const list = shown.filter((t) => !!t.orch === orch)}
			{#if list.length}
				<span class="list-label px-1 pt-3.5 pb-1">{orch ? 'Orchestrator' : 'Project teams'}</span>
				{#each list as t (t.sn)}
					{@const on = t.sn === teamSn}
					<a
						href="/teams?team={t.sn}"
						aria-current={on ? 'page' : undefined}
						class={['flex items-center gap-2.5 rounded-md p-2 outline-none hover:bg-accent/60 focus-visible:ring-3 focus-visible:ring-ring/50', on && 'bg-accent']}
					>
						<AvatarGroup class={on ? '*:data-[slot=avatar]:ring-accent' : '*:data-[slot=avatar]:ring-sidebar'}>
							{#each t.members.slice(0, 3) as m (m.sn)}<RoleAvatar role={m.role} icon={glyphOf(m)} size="sm" />{/each}
						</AvatarGroup>
						<span class="flex min-w-0 flex-1 flex-col gap-px">
							<span class={['truncate text-body', on ? 'font-semibold' : 'font-medium']}>{t.name}</span>
							<span class="truncate text-caption text-muted-foreground">{t.orch ? '모든 프로젝트 · PM' : t.project}</span>
						</span>
						<span class="flex flex-col items-end gap-0.5 text-muted-foreground">
							<span class="font-mono text-caption">{t.members.length}</span>
							{#if t.members.some((m) => m.status === 'running')}
								<LoaderCircle class="size-3 text-status-in-progress" aria-label="실행 중" />
							{:else}
								<CirclePause class="size-3" aria-label="쉬는 중" />
							{/if}
						</span>
					</a>
				{/each}
			{/if}
		{/each}
		{/if}
	</aside>
	{@render children()}
</div>
{:else}
	<Empty class="h-full">
		<EmptyHeader>
			{#if teamsLoad.state === 'error'}
				<EmptyTitle>팀을 불러오지 못했어요</EmptyTitle>
				<EmptyDescription>서버 연결을 확인하고 다시 시도하세요.</EmptyDescription>
			{:else}
				<LoaderCircle class="size-5 animate-spin text-muted-foreground" aria-hidden="true" />
				<EmptyTitle>팀을 불러오는 중…</EmptyTitle>
			{/if}
		</EmptyHeader>
		{#if teamsLoad.state === 'error'}<EmptyContent><Button size="sm" onclick={() => ((teamsLoad.state = 'idle'), loadTeams())}>다시 시도</Button></EmptyContent>{/if}
	</Empty>
{/if}

<!-- 새 팀 (.pen Teams · 새 팀) -->
<Dialog bind:open={() => newTeam !== undefined, (v) => !v && (newTeam = undefined)}>
	<DialogContent size="md">
		{#if newTeam}
			{@const f = newTeam}
			<DialogHeader icon={UsersRound}>
				<DialogTitle>새 팀</DialogTitle>
				<DialogDescription>함께 일하는 에이전트 묶음이에요. 프로젝트에 연결해서 써요.</DialogDescription>
			</DialogHeader>
			<DialogBody>
				<FieldRow label="이름" as="label"><Input bind:value={f.name} placeholder="예: Web Squad" /></FieldRow>
				<FieldRow label="설명" hint="선택" as="label"><Input bind:value={f.desc} placeholder="이 팀이 맡는 일" /></FieldRow>
				<FieldRow label="시작 구성" hint="나중에 Teams에서 언제든 바꿀 수 있어요">
					<ChoiceCards aria-label="시작 구성" class="flex flex-col gap-2" bind:value={() => f.start, (v) => (f.start = v as typeof f.start)}>
						{#each starts as o, i (o.v)}
							<ChoiceCard value={o.v} class="gap-1 rounded-lg px-3.5 py-3">
								<span class="row-title-strong"><span class="font-mono text-caption text-muted-foreground">{'ABC'[i]}</span>{o.t}{#if o.v === 'recommend'}<Badge class="ml-auto">추천</Badge>{/if}</span>
								<span class="text-caption text-muted-foreground">{o.d}</span>
							</ChoiceCard>
						{/each}
					</ChoiceCards>
				</FieldRow>
				<FieldRow label="연결할 프로젝트" hint="나중에 프로젝트 화면에서도 연결돼요">
					<Select type="single" bind:value={f.project}>
						<SelectTrigger class="w-full" aria-label="연결할 프로젝트">{f.project === 'later' ? '나중에 연결' : f.project}</SelectTrigger>
						<SelectContent>
							<SelectItem value="later" label="나중에 연결" />
							{#each store.projects as p (p.sn)}<SelectItem value={p.name} label={p.name} />{/each}
						</SelectContent>
					</Select>
				</FieldRow>
			</DialogBody>
			<DialogFooter note="실행 한도 · 리뷰 기본값으로 시작해요 · 팀 설정에서 바꿀 수 있어요">
				<Button variant="ghost" size="sm" onclick={() => (newTeam = undefined)}>취소</Button>
				<Button size="sm" disabled={!f.name.trim() || creating} onclick={createTeam}>{#if creating}<LoaderCircle class="animate-spin" />{/if}팀 만들기</Button>
			</DialogFooter>
		{/if}
	</DialogContent>
</Dialog>
