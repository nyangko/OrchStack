<script lang="ts">
	/// Settings › 실행기 (CLI) — 이 기기의 CLI 감지 · 설치 · 로그인 · 업데이트 · 공통 실행 설정 (.pen Settings · 실행기 (CLI)).
	/// 로그인은 연결 추가 다이얼로그 (제공자 지정). 감지 · 설치 · 업데이트는 목데이터 (서버 연결 #47).
	import { onDestroy, type Component } from 'svelte';
	import ScanSearch from '@lucide/svelte/icons/scan-search';
	import Plus from '@lucide/svelte/icons/plus';
	import Settings2 from '@lucide/svelte/icons/settings-2';
	import Download from '@lucide/svelte/icons/download';
	import LogIn from '@lucide/svelte/icons/log-in';
	import CircleCheck from '@lucide/svelte/icons/circle-check';
	import CircleArrowUp from '@lucide/svelte/icons/circle-arrow-up';
	import CircleDashed from '@lucide/svelte/icons/circle-dashed';
	import LoaderCircle from '@lucide/svelte/icons/loader-circle';
	import Sparkles from '@lucide/svelte/icons/sparkles';
	import Code from '@lucide/svelte/icons/code';
	import MousePointer2 from '@lucide/svelte/icons/mouse-pointer-2';
	import Cloud from '@lucide/svelte/icons/cloud';
	import Layers from '@lucide/svelte/icons/layers';
	import Timer from '@lucide/svelte/icons/timer';
	import FolderGit2 from '@lucide/svelte/icons/folder-git-2';
	import ArrowLeftRight from '@lucide/svelte/icons/arrow-left-right';
	import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '$lib/components/ui/card';
	import { Select, SelectTrigger, SelectContent, SelectItem } from '$lib/components/ui/select';
	import { Button } from '$lib/components/ui/button';
	import { PageHeader } from '$lib/components/orch/page-header';
	import { Pill } from '$lib/components/orch/pill';
	import { RuntimeLogo, type Runtime } from '$lib/components/orch/runtime-logo';
	import { FieldRow, FieldSwitchRow } from '$lib/components/ui/field';
	import { KeyValueRow } from '$lib/components/orch/key-value-row';
	import { AddConnectionDialog, type AddedConnection } from '$lib/components/orch/connection';
	import { runtimeClis, runSettings, type RuntimeCli } from '$lib/mock';
	
	/// busy — 설치 · 업데이트 중 (목데이터 동안만).
	type Cli = RuntimeCli & { busy?: boolean };

	let clis = $state<Cli[]>(structuredClone(runtimeClis));
	let run = $state({ ...runSettings });
	let detecting = $state(false);
	let lastDetect = $state('2분 전');
	let addOpen = $state(false);
	let loginCli = $state<Cli>();
	const timers = new Set<ReturnType<typeof setTimeout>>();
	onDestroy(() => timers.forEach(clearTimeout));
	const later = (ms: number, fn: () => void) => {
		const t = setTimeout(() => (timers.delete(t), fn()), ms);
		timers.add(t);
	};

	const logo: Record<string, Runtime | Component> = { codex: 'codex', claude: 'claude', gemini: Sparkles, opencode: Code, cursor: MousePointer2, kiro: Cloud };
	const stateMeta: Record<Cli['state'], { icon: Component; tone: string; action: { icon: Component; label: string } }> = {
		ok: { icon: CircleCheck, tone: 'text-status-done', action: { icon: Settings2, label: '관리' } },
		gateway: { icon: CircleCheck, tone: 'text-status-done', action: { icon: Settings2, label: '관리' } },
		update: { icon: CircleArrowUp, tone: 'text-status-waiting', action: { icon: Download, label: '업데이트' } },
		login: { icon: LogIn, tone: 'text-status-waiting', action: { icon: LogIn, label: '로그인' } },
		missing: { icon: CircleDashed, tone: 'text-muted-foreground', action: { icon: Plus, label: '설치' } }
	};
	const installedCount = $derived(clis.filter((c) => c.state !== 'missing').length);
	const selects = [
		{ key: 'maxRuns', label: '동시 실행', hint: '기기 전체 최대 Run 수', icon: Layers, options: ['1 Run', '2 Run', '3 Run', '4 Run', '6 Run'], note: 'CPU 8코어 기준 권장' },
		{ key: 'timeout', label: 'Run 제한 시간', hint: '넘으면 연장 승인을 요청해요', icon: Timer, options: ['10분', '20분', '40분', '1시간'], note: '권한 › 승인 규칙과 연결' },
		{ key: 'workspace', label: '작업 공간', hint: 'Run마다 격리된 폴더에서 작업', icon: FolderGit2, options: ['git worktree', '저장소 그대로'], note: '~/.orch/worktrees · 끝나면 정리' }
	] as const;
	const switches = [
		{ key: 'sandbox', label: '네트워크 샌드박스', hint: '허용 도메인만 접근 (docs.svelte.dev 등)' },
		{ key: 'autoUpdate', label: '자동 업데이트', hint: '패치 버전만 · 마이너 이상은 알림 후 수동' }
	] as const;

	/// 실행기 동작 (목데이터) — 업데이트 · 설치는 잠시 뒤 상태가 바뀌고, 로그인은 연결 추가 다이얼로그로.
	function act(c: Cli) {
		if (c.state === 'update') {
			c.busy = true;
			later(1200, () => {
				c.busy = false;
				c.state = 'ok';
				c.version = 'v2.4';
				c.note = '로그인됨 · Anthropic Max · 멤버 3';
			});
		} else if (c.state === 'missing') {
			c.busy = true;
			later(1500, () => {
				c.busy = false;
				c.state = 'login';
				c.version = 'v0.3';
				c.note = `설치됨 · ${c.name.replace(' CLI', '')} 로그인 필요`;
			});
		} else if (c.state === 'login') {
			loginCli = c;
			addOpen = true;
		}
		// 관리 화면은 .pen에 아직 없음
	}

	function onLogin(a: AddedConnection) {
		if (!loginCli) return;
		loginCli.state = 'ok';
		loginCli.note = `로그인됨 · ${a.title} · 멤버 0`;
	}

	/// 다시 감지 — PATH에서 CLI를 다시 찾는다.
	function detect() {
		detecting = true;
		later(1000, () => {
			detecting = false;
			lastDetect = '방금';
		});
	}
</script>

<svelte:head><title>실행기 (CLI) · Settings · OrchStack</title></svelte:head>

<main class="page-main">
	<PageHeader title="실행기 (CLI)" desc="에이전트가 도는 CLI · 설치 · 로그인 · 버전 · 공통 실행 설정" status={false}>
		<Button variant="outline" disabled={detecting} onclick={detect}>
			{#if detecting}<LoaderCircle class="animate-spin" />감지 중…{:else}<ScanSearch />다시 감지{/if}
		</Button>
		<!-- 설치 안 된 실행기를 한꺼번에 설치 -->
		<Button disabled={!clis.some((c) => c.state === 'missing' && !c.busy)} onclick={() => clis.filter((c) => c.state === 'missing' && !c.busy).forEach(act)}><Plus />실행기 설치</Button>
	</PageHeader>

	<Card size="sm">
		<CardHeader>
			<CardTitle>설치된 실행기</CardTitle>
			<CardDescription>이 기기에서 감지한 CLI · 에이전트가 실제로 도는 프로그램</CardDescription>
		</CardHeader>
		<CardContent class="grid grid-cols-2 gap-2.5">
			{#each clis as c (c.key)}
				{@const m = stateMeta[c.state]}
				{@const L = logo[c.key]}
				<div class="option-card rounded-md">
					<span class="icon-tile size-9.5">
						{#if typeof L === 'string'}<RuntimeLogo runtime={L} class="size-5 ring-0" />{:else}<L class="size-4.5" />{/if}
					</span>
					<span class="flex min-w-0 flex-1 flex-col gap-1">
						<span class="flex items-center gap-2 text-xs font-semibold">{c.name}<Pill class="font-mono text-2xs">{c.version}</Pill></span>
						<span class={['meta-truncate gap-1.25', c.busy ? 'text-status-in-progress' : m.tone]}>
							{#if c.busy}<LoaderCircle class="size-3 shrink-0 animate-spin" />{c.state === 'update' ? '업데이트 중…' : '설치 중…'}
							{:else}<m.icon class="size-3 shrink-0" />{c.note}{/if}
						</span>
					</span>
					<Button variant="ghost" size="sm" disabled={c.busy} onclick={() => act(c)}><m.action.icon />{m.action.label}</Button>
				</div>
			{/each}
		</CardContent>
	</Card>

	<div class="flex items-start gap-5">
		<Card size="sm" class="min-w-0 flex-1">
			<CardHeader>
				<CardTitle>공통 실행 설정</CardTitle>
				<CardDescription>모든 실행기에 적용 · 팀 정책 · 멤버 권한이 더 좁으면 그쪽이 우선</CardDescription>
			</CardHeader>
			<CardContent>
				{#each selects as r (r.key)}
					<FieldRow label={r.label} hint={r.hint}>
						<Select type="single" bind:value={run[r.key]}>
							<SelectTrigger class="w-full" aria-label={r.label}>
								<span class="flex min-w-0 flex-1 items-center gap-2">
									<r.icon class="size-4 text-muted-foreground" />{run[r.key]}
									<span class="truncate text-caption font-normal text-muted-foreground">{r.note}</span>
								</span>
							</SelectTrigger>
							<SelectContent>
								{#each r.options as o (o)}<SelectItem value={o} label={o} />{/each}
							</SelectContent>
						</Select>
					</FieldRow>
				{/each}
				{#each switches as s (s.key)}
					<FieldSwitchRow label={s.label} hint={s.hint} bind:checked={run[s.key]} />
				{/each}
			</CardContent>
		</Card>

		<div class="flex shrink-0 flex-col w-95 gap-5">
			<Card size="sm">
				<CardHeader>
					<CardTitle>감지</CardTitle>
					<CardDescription>PATH에서 CLI를 찾아요</CardDescription>
				</CardHeader>
				<CardContent>
					{#each [['검색 경로', '~/.local/bin · /opt/homebrew/bin'], ['마지막 감지', lastDetect], ['설치됨', `${installedCount} / ${clis.length}`]] as [l, v] (l)}
						<KeyValueRow label={l} value={v} />
					{/each}
				</CardContent>
			</Card>

			<Card size="sm">
				<CardHeader>
					<CardTitle class="flex items-center gap-1.5">실행기 <ArrowLeftRight class="size-3.5" /> 연결</CardTitle>
					<CardDescription>실행기마다 쓸 수 있는 연결 · 폴백은 <a href="/settings/connections" class="text-primary hover:underline">모델 연결</a>에서</CardDescription>
				</CardHeader>
				<CardContent>
					{#each clis.filter((c) => c.conns) as c (c.key)}
						<KeyValueRow label={c.name} value={c.conns} />
					{/each}
				</CardContent>
			</Card>
		</div>
	</div>
</main>

<AddConnectionDialog bind:open={addOpen} provider={loginCli?.provider} onadd={onLogin} />
