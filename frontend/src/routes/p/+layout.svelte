<script lang="ts">
	/// Workbench 셸 (.pen Project Tabs). 열린 프로젝트 탭 · All Projects · 새 프로젝트 (.pen 새 프로젝트 만들기 다이얼로그).
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import LayoutGrid from '@lucide/svelte/icons/layout-grid';
	import Plus from '@lucide/svelte/icons/plus';
	import X from '@lucide/svelte/icons/x';
	import FolderPlus from '@lucide/svelte/icons/folder-plus';
	import Folder from '@lucide/svelte/icons/folder';
	import GitFork from '@lucide/svelte/icons/git-fork';
	import Users from '@lucide/svelte/icons/users';
	import Hand from '@lucide/svelte/icons/hand';
	import Timer from '@lucide/svelte/icons/timer';
	import Zap from '@lucide/svelte/icons/zap';
	import Check from '@lucide/svelte/icons/check';
	import * as Dialog from '$lib/components/ui/dialog';
	import * as Select from '$lib/components/ui/select';
	import * as InputGroup from '$lib/components/ui/input-group';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import { Segmented } from '$lib/components/ui/segmented';
	import * as Field from '$lib/components/ui/field';
	import { repos, type OrchPolicy, type ProjectTab } from '$lib/mock';
	import { onMount } from 'svelte';
	import { useMock } from '$lib/api/env';
	import { api } from '$lib/api/client';
	import type { ApiProject } from '$lib/api/types';
	import { roles } from '$lib/roles';
	import { store } from '$lib/teams.svelte';
	import { cn } from '$lib/utils';

	let { children } = $props();

	// 열린 탭은 화면 상태(서버에 저장하지 않음). 닫아도 프로젝트는 그대로다.
	let open = $state(store.projects.map((p) => p.sn));
	/// 서버 행 → 탭. dot은 상태색 (실행 중 여부는 #88 뒤에).
	const toTab = (p: ApiProject): ProjectTab => ({ sn: p.sn, name: p.name, status: p.status, dot: p.status === 'active' ? 'bg-success' : 'bg-subtle-foreground' });
	// 서버 모드면 프로젝트 목록을 API에서 — 탭 · All Projects · Workbench가 같은 목록(store.projects)을 본다
	onMount(async () => {
		if (useMock) return;
		const { data } = await api.GET('/projects');
		if (data) {
			store.projects = data.map(toTab);
			open = store.projects.map((p) => p.sn);
		}
	});
	const tabs = $derived(store.projects.filter((p) => open.includes(p.sn)));
	const current = $derived(Number(page.params.project));

	/// 탭을 닫고, 보고 있던 탭이면 옆 탭(없으면 All Projects)으로 이동한다.
	function close(sn: number) {
		const i = open.indexOf(sn);
		open = open.filter((s) => s !== sn);
		if (sn === current) goto(open.length ? `/p/${open[Math.min(i, open.length - 1)]}` : '/p');
	}

	// 새 프로젝트 — 프로젝트 탭 + 또는 All Projects(?new)에서 연다.
	const squads = $derived(store.crew.filter((t) => !t.orch));
	let creating = $state(false);
	let name = $state('');
	let repo = $state(repos[0].name);
	let teamSn = $state(0);
	let mode = $state<OrchPolicy['mode']>('manual');
	let importIssues = $state(true);
	let firstPlan = $state(true);
	const repoInfo = $derived(repos.find((r) => r.name === repo)!);
	const team = $derived(squads.find((t) => t.sn === teamSn));
	const nameTaken = $derived(store.projects.some((p) => p.name === name.trim()));

	/// 팀 구성 요약 — 멤버 수 · 역할별 수 (예: 멤버 3 · Frontend 2 · Backend 1).
	function teamMeta(t: (typeof squads)[number]) {
		const count = new Map<keyof typeof roles, number>();
		for (const m of t.members) count.set(m.role, (count.get(m.role) ?? 0) + 1);
		const byRole = [...count].map(([r, n]) => `${roles[r].label} ${n}`);
		return [`멤버 ${t.members.length}`, ...byRole].join(' · ');
	}
	function openNew() {
		name = '';
		repo = repos[0].name;
		teamSn = squads[0]?.sn ?? 0;
		// 팀 정책의 진행 방식이 기본값
		mode = store.policies[teamSn]?.mode ?? 'manual';
		importIssues = true;
		firstPlan = true;
		creating = true;
	}
	$effect(() => {
		if (!page.url.searchParams.has('new')) return;
		openNew();
		goto(page.url.pathname, { replaceState: true });
	});

	let submitting = $state(false);
	/// 만들면 탭을 열고 그 Workbench로 간다. 서버 모드는 POST /projects — 실패하면 입력을 그대로 둔다(토스트는 클라이언트가). 팀 · 진행 방식 · 이슈 가져오기는 A-3 · #89 뒤에 보낸다.
	async function create(e: SubmitEvent) {
		e.preventDefault();
		if (!name.trim() || nameTaken || submitting) return;
		let sn: number;
		if (useMock) {
			sn = Math.max(0, ...store.projects.map((p) => p.sn)) + 1;
			store.projects.push({ sn, name: name.trim(), status: 'active', dot: firstPlan ? 'bg-status-waiting' : 'bg-subtle-foreground' });
		} else {
			submitting = true;
			const { data } = await api.POST('/projects', { body: { name: name.trim(), repo_name: repo } });
			submitting = false;
			if (!data) return;
			store.projects.push(toTab(data));
			sn = data.sn;
		}
		open.push(sn);
		creating = false;
		goto(`/p/${sn}`);
	}
</script>

<div class="flex h-full flex-col">
	<div class="project-tabs">
		<a
			href="/p"
			aria-current={page.url.pathname === '/p' ? 'page' : undefined}
			class={cn(
				'project-tab-all text-muted-foreground',
				page.url.pathname === '/p' && 'relative -mb-px border border-b-0 bg-background text-foreground'
			)}
		>
			<LayoutGrid class="size-3.5" />
			All Projects
			<Badge variant="secondary" class="px-1.5">{store.projects.length}</Badge>
		</a>
		{#each tabs as p (p.sn)}
			{@const on = p.sn === current}
			<div
				class={cn(
					'project-tab',
					on ? 'relative -mb-px border border-b-0 bg-background' : 'text-muted-foreground hover:text-foreground'
				)}
			>
				<a href="/p/{p.sn}" aria-current={on ? 'page' : undefined} class="project-tab-link">
					<span class={cn('size-1.5 rounded-full', p.dot)}></span>
					{p.name}
				</a>
				<button
					type="button"
					aria-label="{p.name} 탭 닫기"
					onclick={() => close(p.sn)}
					class="focus-ring rounded-xs text-muted-foreground hover:text-foreground"
				>
					<X class="size-3.5" />
				</button>
			</div>
		{/each}
		<Button variant="ghost" size="icon-sm" class="mb-0.5 shrink-0 text-muted-foreground" aria-label="새 프로젝트" onclick={openNew}><Plus /></Button>
	</div>
	<div class="min-h-0 flex-1">
		{@render children()}
	</div>
</div>

<!-- 폼 한 줄 제목 (.pen FormRow) -->
<Dialog.Root bind:open={creating}>
	<Dialog.Content size="lg">
		<!-- contents — Header · Body · Footer가 Content의 세로 배치에 그대로 놓이게 -->
		<form onsubmit={create} class="contents">
			<Dialog.Header icon={FolderPlus}>
				<Dialog.Title>새 프로젝트</Dialog.Title>
				<Dialog.Description>저장소를 연결하고 팀 · Orch 진행 방식을 정해요</Dialog.Description>
			</Dialog.Header>

			<Dialog.Body class="py-1">
				<Field.Row label="이름" hint="프로젝트 탭 · 브레드크럼에 보여요" as="label" error={nameTaken ? '같은 이름의 프로젝트가 있어요' : undefined}>
					<InputGroup.Root>
						<InputGroup.Addon><Folder /></InputGroup.Addon>
						<InputGroup.Input bind:value={name} placeholder="예: Checkout Revamp" aria-invalid={nameTaken || undefined} class="text-xs font-medium" />
					</InputGroup.Root>
				</Field.Row>
				<Field.Row label="저장소" hint="GitHub App이 설치된 저장소만 보여요">
					<Select.Root type="single" bind:value={repo}>
						<Select.Trigger class="w-full" aria-label="저장소">
							<span class="row-fill">
								<GitFork class="size-4 text-muted-foreground" />{repo}
								<span class="truncate text-caption font-normal text-muted-foreground">{repoInfo.meta} · 이슈 {repoInfo.issues}개</span>
							</span>
						</Select.Trigger>
						<Select.Content>{#each repos as r (r.name)}<Select.Item value={r.name} label={r.name} />{/each}</Select.Content>
					</Select.Root>
				</Field.Row>
				<Field.Row label="팀" hint="이 프로젝트를 맡을 팀 · 나중에 바꿀 수 있어요">
					<Select.Root type="single" bind:value={() => String(teamSn), (v) => ((teamSn = Number(v)), (mode = store.policies[teamSn]?.mode ?? mode))}>
						<Select.Trigger class="w-full" aria-label="팀">
							<span class="row-fill">
								<Users class="size-4 text-muted-foreground" />{team?.name}
								{#if team}<span class="truncate text-caption font-normal text-muted-foreground">{teamMeta(team)}</span>{/if}
							</span>
						</Select.Trigger>
						<Select.Content>{#each squads as t (t.sn)}<Select.Item value={String(t.sn)} label={t.name} />{/each}</Select.Content>
					</Select.Root>
				</Field.Row>
				<Field.Row label="Orch 진행 방식" hint="팀 정책을 따르거나 이 프로젝트만 바꿔요">
					<Segmented
						aria-label="Orch 진행 방식"
						options={[
							{ value: 'manual', label: 'Manual', icon: Hand },
							{ value: 'timer', label: `Auto · ${store.policies[teamSn]?.timer ?? 5}초`, icon: Timer },
							{ value: 'full', label: 'Full auto', icon: Zap }
						]}
						bind:value={() => mode, (v) => (mode = v as OrchPolicy['mode'])}
					/>
				</Field.Row>
				<Field.SwitchRow label="GitHub 이슈 가져오기" hint={`열린 이슈 ${repoInfo.issues}개 → Backlog · 라벨 bug · feature만`} bind:checked={importIssues} />
				<!-- 가져온 이슈가 있어야 계획을 세운다 -->
				<Field.SwitchRow label="Orch가 첫 계획 세우기" hint="가져온 이슈를 태스크로 나누고 배정안을 제안해요 (승인 후 실행)" bind:checked={() => importIssues && firstPlan, (v) => (firstPlan = v)} disabled={!importIssues} />
			</Dialog.Body>

			<Dialog.Footer note="만든 뒤 Workbench에서 이슈 · 태스크를 바로 볼 수 있어요">
				<Dialog.Close>{#snippet child({ props })}<Button type="button" variant="ghost" {...props}>취소</Button>{/snippet}</Dialog.Close>
				<Button type="submit" disabled={!name.trim() || nameTaken || !team}><Check />프로젝트 만들기</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>
