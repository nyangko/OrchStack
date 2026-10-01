<script lang="ts">
	/// Settings › 스킬 소스 — 스킬을 찾고 설치할 곳 · 설치 정책 · 최근 변경 (.pen Settings · 스킬 소스).
	/// Skills & MCP 페이지와 같은 store.sources · store.log를 쓴다 — 여기서 바꾸면 거기도 바뀐다. 동기화는 목데이터 (서버 연결 #47).
	import { onDestroy, type Component } from 'svelte';
	import RefreshCw from '@lucide/svelte/icons/refresh-cw';
	import Plus from '@lucide/svelte/icons/plus';
	import Globe from '@lucide/svelte/icons/globe';
	import GitFork from '@lucide/svelte/icons/git-fork';
	import Folder from '@lucide/svelte/icons/folder';
	import FolderOpen from '@lucide/svelte/icons/folder-open';
	import Settings2 from '@lucide/svelte/icons/settings-2';
	import Terminal from '@lucide/svelte/icons/terminal';
	import Check from '@lucide/svelte/icons/check';
	import Download from '@lucide/svelte/icons/download';
	import Ban from '@lucide/svelte/icons/ban';
	import ToggleRight from '@lucide/svelte/icons/toggle-right';
	import KeyRound from '@lucide/svelte/icons/key-round';
	import LoaderCircle from '@lucide/svelte/icons/loader-circle';
	import * as Card from '$lib/components/ui/card';
	import * as InputGroup from '$lib/components/ui/input-group';
	import { Button } from '$lib/components/ui/button';
	import { PageHeader } from '$lib/components/ui/page-header';
	import { Pill } from '$lib/components/ui/pill';
	import * as Field from '$lib/components/ui/field';
	import { KeyValueRow } from '$lib/components/ui/key-value-row';
	import { HistoryRow } from '$lib/components/ui/history-row';
	import SkillSourcesDialog from '$lib/components/orch/agent/skill-sources-dialog.svelte';
	import type { Skill, SkillLog } from '$lib/mock';
	import { store } from '$lib/teams.svelte';
	import { cn } from '$lib/utils';

	type Source = (typeof store.sources.sources)[number];

	let syncing = $state(false);
	let dialogOpen = $state(false);
	/// 소스 추가 — 입력 줄을 열어 GitHub 저장소(owner/repo) 또는 폴더 경로를 받는다.
	let adding = $state(false);
	let newSource = $state('');
	let timer: ReturnType<typeof setTimeout> | undefined;
	onDestroy(() => clearTimeout(timer));

	const kindIcon: Record<Source['kind'], Component> = { web: Globe, github: GitFork, folder: Folder };
	const logIcon: Record<SkillLog['kind'], Component> = { INSTALL: Download, UPDATE: RefreshCw, BLOCK: Ban, TOGGLE: ToggleRight, AUTH: KeyRound };
	/// 설치된 스킬 — 소스별 개수 (Marketplace · Built-in은 이 화면의 소스가 아니라 뺀다).
	const bySource: { source: Skill['source']; label: string }[] = [
		{ source: 'skills.sh', label: 'skills.sh' },
		{ source: 'Team', label: 'orchstack/team-skills' },
		{ source: 'Local', label: '~/.orch/skills' }
	];
	const installed = $derived(store.library.filter((s) => bySource.some((b) => b.source === s.source)));
	const applyBy = [
		{ cli: 'Codex CLI', how: 'AGENTS.md에 요약 포함' },
		{ cli: 'Claude Code', how: '~/.claude/skills 동기화' },
		{ cli: 'Gemini CLI', how: 'GEMINI.md에 요약 포함' }
	];

	/// 지금 동기화 — 저장소 소스를 다시 읽는다 (목데이터: 1초 뒤 완료).
	function syncAll() {
		syncing = true;
		timer = setTimeout(() => {
			for (const s of store.sources.sources) if (s.kind === 'github') s.sync = '방금 동기화';
			store.log.unshift({ kind: 'UPDATE', who: '나', when: '방금', text: 'team-skills 동기화 · 변경 없음' });
			syncing = false;
		}, 1000);
	}

	function addSource() {
		const name = newSource.trim();
		if (!name || store.sources.sources.some((s) => s.name === name)) return;
		const folder = name.startsWith('~') || name.startsWith('/') || name.startsWith('.');
		store.sources.sources.push(
			folder
				? { kind: 'folder', name, state: '로컬', desc: '로컬 폴더 · 이 기기에서만', sync: '스킬 0' }
				: { kind: 'github', name, state: '연결됨', desc: 'GitHub 저장소 · main 브랜치', sync: '동기화 전' }
		);
		store.log.unshift({ kind: 'INSTALL', who: '나', when: '방금', text: `소스 추가 · ${name}` });
		newSource = '';
		adding = false;
	}
</script>

<svelte:head><title>스킬 소스 · Settings · OrchStack</title></svelte:head>

<main class="flex flex-col gap-5 px-8 py-7">
	<PageHeader title="스킬 소스" desc="스킬을 찾고 설치할 곳 · 워크스페이스 전체에 적용" status={false}>
		<Button variant="outline" disabled={syncing} onclick={syncAll}>
			<RefreshCw class={cn(syncing && 'animate-spin')} />{syncing ? '동기화 중…' : '지금 동기화'}
		</Button>
		<Button onclick={() => (adding = true)}><Plus />소스 추가</Button>
	</PageHeader>

	<div class="flex items-start gap-5">
		<div class="flex min-w-0 flex-1 flex-col gap-5">
			<Card.Root size="sm">
				<Card.Header>
					<Card.Title>소스</Card.Title>
					<Card.Description>스킬을 검색 · 설치할 곳 · 위에서부터 우선</Card.Description>
				</Card.Header>
				<Card.Content class="flex flex-col gap-2">
					{#each store.sources.sources as src (src.name)}
						{@const Icon = kindIcon[src.kind]}
						<div class="flex items-center gap-3 rounded-md border px-3.5 py-3">
							<span class="flex size-9 shrink-0 items-center justify-center rounded-md bg-muted"><Icon class="size-4" /></span>
							<span class="flex min-w-0 flex-1 flex-col gap-1">
								<span class="flex items-center gap-2 text-body font-semibold">
									{src.name}<Pill dot={src.state === '연결됨' ? 'bg-status-done' : 'bg-muted-foreground'} class={src.state === '연결됨' ? 'bg-success-soft text-status-done' : ''}>{src.state}</Pill>
								</span>
								<span class="truncate text-caption text-muted-foreground">{src.desc}</span>
							</span>
							<span class="text-caption text-subtle-foreground">{src.sync}</span>
							{#if src.kind === 'web'}
								<Button variant="outline" size="sm" onclick={() => (dialogOpen = true)}><Settings2 />설정</Button>
							{:else if src.kind === 'github'}
								<Button variant="outline" size="sm" disabled={syncing} onclick={syncAll}><RefreshCw />동기화</Button>
							{:else}
								<!-- 브라우저에서 이 기기 폴더를 열 수 없어 서버 연결(#47) 후 동작 -->
								<Button variant="outline" size="sm"><FolderOpen />열기</Button>
							{/if}
						</div>
					{/each}
					{#if adding}
						<form class="flex items-center gap-2" onsubmit={(e) => (e.preventDefault(), addSource())}>
							<InputGroup.Root class="flex-1">
								<InputGroup.Addon><Plus /></InputGroup.Addon>
								<!-- 소스 추가를 눌러 연 입력 줄이라 바로 입력하게 한다 -->
								<!-- svelte-ignore a11y_autofocus -->
								<InputGroup.Input bind:value={newSource} autofocus placeholder="owner/repo 또는 ~/폴더 경로" aria-label="새 소스" class="font-mono text-xs" />
							</InputGroup.Root>
							<Button type="submit" size="sm" disabled={!newSource.trim()}>추가</Button>
							<Button type="button" variant="ghost" size="sm" onclick={() => ((adding = false), (newSource = ''))}>취소</Button>
						</form>
					{:else}
						<button type="button" onclick={() => (adding = true)} class="flex items-center gap-2 rounded-md px-3.5 py-2 text-xs font-medium outline-none hover:bg-muted focus-visible:ring-3 focus-visible:ring-ring/50">
							<Plus class="size-3.5" />소스 추가 · GitHub 저장소 / 폴더
						</button>
					{/if}
				</Card.Content>
			</Card.Root>

			<Card.Root size="sm">
				<Card.Header>
					<Card.Title>설치 정책</Card.Title>
					<Card.Description>워크스페이스의 모든 팀 · 멤버에 적용</Card.Description>
				</Card.Header>
				<Card.Content>
					{#each store.sources.policy as pol (pol.name)}
						<Field.SwitchRow label={pol.name} hint={pol.desc} bind:checked={pol.on} />
					{/each}
				</Card.Content>
			</Card.Root>

			<Card.Root size="sm">
				<Card.Header>
					<Card.Title>최근 설치 · 변경</Card.Title>
					<Card.Description>7일</Card.Description>
					<!-- 감사 로그 화면은 권한 · 보안(S-4)에서 -->
					<Card.Action><span class="text-xs font-medium text-primary">감사 로그</span></Card.Action>
				</Card.Header>
				<Card.Content>
					{#each store.log.slice(0, 6) as l, i (i)}
						{@const Icon = logIcon[l.kind]}
						<HistoryRow icon={Icon} tone={l.kind === 'BLOCK' ? 'bg-destructive-soft text-destructive' : 'bg-primary-soft text-primary'} kind={l.kind} who={l.who} when={l.when} text={l.text} />
					{/each}
				</Card.Content>
			</Card.Root>
		</div>

		<div class="flex w-95 shrink-0 flex-col gap-5">
			<Card.Root size="sm">
				<Card.Header><Card.Title>설치 도구</Card.Title></Card.Header>
				<Card.Content>
					<div class="flex items-center gap-2 text-xs">
						<Terminal class="size-3.5 text-muted-foreground" />
						<span class="flex-1 text-muted-foreground">{store.sources.tool.name}</span>
						<span class="font-mono">{store.sources.tool.version}</span>
						<Pill class="bg-success-soft text-status-done"><Check />설치됨</Pill>
					</div>
				</Card.Content>
			</Card.Root>

			<Card.Root size="sm">
				<Card.Header>
					<Card.Title>설치된 스킬</Card.Title>
					<Card.Description>{installed.length}개 · 활성 {installed.filter((s) => s.on !== false && !s.blocked).length}</Card.Description>
					<Card.Action><Button variant="link" size="xs" href="/skills">모두 보기</Button></Card.Action>
				</Card.Header>
				<Card.Content>
					{#each bySource as b (b.source)}
						{@const list = installed.filter((s) => s.source === b.source)}
						{@const updates = list.filter((s) => s.update).length}
						<KeyValueRow label={b.label}><span class="text-body font-medium">{list.length}개{updates ? ` · 업데이트 ${updates}` : ''}{b.source === 'Local' ? ' · 로컬' : ''}</span></KeyValueRow>
					{/each}
				</Card.Content>
			</Card.Root>

			<Card.Root size="sm">
				<Card.Header><Card.Title>CLI별 적용 방식</Card.Title></Card.Header>
				<Card.Content>
					{#each applyBy as a (a.cli)}
						<KeyValueRow label={a.cli} value={a.how} />
					{/each}
				</Card.Content>
			</Card.Root>
		</div>
	</div>
</main>

<SkillSourcesDialog bind:open={dialogOpen} />
