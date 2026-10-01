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
	import * as Avatar from '$lib/components/ui/avatar';
	import * as InputGroup from '$lib/components/ui/input-group';
	import { Button } from '$lib/components/ui/button';
	import { Segmented } from '$lib/components/ui/segmented';
	import { RoleAvatar } from '$lib/components/ui/role-avatar';
	import { store, defaultTeam, glyphOf, membersOf } from '$lib/teams.svelte';
	
	let { children } = $props();

	const scope = $derived(page.route.id?.startsWith('/teams/agents') ? 'agents' : 'teams');
	const teamSn = $derived(scope === 'teams' ? Number(page.url.searchParams.get('team')) || defaultTeam().sn : undefined);
	const tplSn = $derived(scope === 'agents' ? Number(page.params.template) : undefined);

	let query = $state('');
	const shown = $derived(store.crew.filter((t) => t.name.toLowerCase().includes(query.trim().toLowerCase())));
	let tplQuery = $state('');
	const tplGroups = $derived(
		[...new Set(store.templates.map((t) => t.group))].map((g) => ({ g, list: store.templates.filter((t) => t.group === g && t.name.toLowerCase().includes(tplQuery.trim().toLowerCase())) }))
	);
</script>

<div class="flex h-full">
<aside class="flex w-70 shrink-0 flex-col gap-1 overflow-y-auto border-r bg-sidebar px-3 py-4">
		<div class="flex items-center px-1 pb-2">
			<h2 class="flex-1 text-sm font-semibold">Workspace</h2>
			<Button variant="ghost" size="icon-sm" class="size-7 text-muted-foreground" aria-label="새 팀"><Plus class="size-3.5" /></Button>
		</div>
		<Segmented
			class="mb-1"
			aria-label="보기"
			options={[{ value: 'teams', label: `Teams · ${store.crew.length}`, icon: Users }, { value: 'agents', label: `Agents · ${store.templates.length}`, icon: LayoutTemplate }]}
			bind:value={() => scope, (v) => goto(v === 'agents' ? `/teams/agents/${tplSn ?? store.templates[0].sn}` : `/teams?team=${teamSn ?? defaultTeam().sn}`)}
		/>
		{#if scope === 'agents'}
			<InputGroup.Root class="h-8">
				<InputGroup.Addon><Search /></InputGroup.Addon>
				<InputGroup.Input bind:value={tplQuery} placeholder="템플릿 검색" aria-label="템플릿 검색" />
			</InputGroup.Root>
			{#each tplGroups as { g, list } (g)}
				{#if list.length}
					<span class="list-label px-1 pt-3.5 pb-1">{g}</span>
					{#each list as t (t.sn)}
						{@const on = t.sn === tplSn}
						{@const n = membersOf(t.name).length}
						<a
							href="/teams/agents/{t.sn}"
							aria-current={on ? 'page' : undefined}
							class={['flex items-center gap-2.5 rounded-md px-2 py-1.75 outline-none hover:bg-accent/60 focus-visible:ring-3 focus-visible:ring-ring/50', on && 'bg-accent']}
						>
							<RoleAvatar role={t.role} />
							<span class="flex min-w-0 flex-1 flex-col gap-0.75">
								<span class="truncate text-xs font-semibold">{t.name}</span>
								<span class="text-caption text-muted-foreground">v{t.version} · {n ? `멤버 ${n}` : '미사용'}{t.draft ? ' · 초안' : ''}</span>
							</span>
						</a>
					{/each}
				{/if}
			{/each}
		{:else}
		<InputGroup.Root class="h-8">
			<InputGroup.Addon><Search /></InputGroup.Addon>
			<InputGroup.Input bind:value={query} placeholder="Filter teams" aria-label="팀 검색" />
		</InputGroup.Root>
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
						<Avatar.Group class={on ? '*:data-[slot=avatar]:ring-accent' : '*:data-[slot=avatar]:ring-sidebar'}>
							{#each t.members.slice(0, 3) as m (m.sn)}<RoleAvatar role={m.role} icon={glyphOf(m)} size="sm" />{/each}
						</Avatar.Group>
						<span class="flex min-w-0 flex-1 flex-col gap-px">
							<span class={['truncate text-body', on ? 'font-semibold' : 'font-medium']}>{t.name}</span>
							<span class="truncate text-caption text-muted-foreground">{t.orch ? '모든 프로젝트 · PM' : t.project}</span>
						</span>
						<span class="flex flex-col items-end gap-0.5 text-muted-foreground">
							<span class="font-mono text-caption">{t.members.length}</span>
							{#if t.members.some((m) => m.status === 'running')}
								<LoaderCircle class="size-2.75 text-status-in-progress" aria-label="실행 중" />
							{:else}
								<CirclePause class="size-2.75" aria-label="쉬는 중" />
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
