<script lang="ts">
	/// Settings › 일반 — 워크스페이스 이름 · 언어 · 시간대 · 테마 · 데이터 보관 (.pen Settings · 일반).
	/// 바꾸면 바로 저장되고 헤더에 저장됨이 뜬다. 서버 모드는 GET/PATCH /workspace(A-4 #95) — 데이터 보관 칸은 아직 서버 필드가 없어 화면 상태.
	import type { Component } from 'svelte';
	import Building2 from '@lucide/svelte/icons/building-2';
	import FolderGit2 from '@lucide/svelte/icons/folder-git-2';
	import Globe from '@lucide/svelte/icons/globe';
	import Languages from '@lucide/svelte/icons/languages';
	import MessageSquare from '@lucide/svelte/icons/message-square';
	import GitCommitHorizontal from '@lucide/svelte/icons/git-commit-horizontal';
	import Calendar from '@lucide/svelte/icons/calendar';
	import Sun from '@lucide/svelte/icons/sun';
	import Moon from '@lucide/svelte/icons/moon';
	import Monitor from '@lucide/svelte/icons/monitor';
	import ScrollText from '@lucide/svelte/icons/scroll-text';
	import ChartColumn from '@lucide/svelte/icons/chart-column';
	import BookMarked from '@lucide/svelte/icons/book-marked';
	import Download from '@lucide/svelte/icons/download';
	import Trash2 from '@lucide/svelte/icons/trash-2';
	import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '$lib/components/ui/card';
	import { Select, SelectTrigger, SelectContent, SelectItem } from '$lib/components/ui/select';
	import { InputGroup, InputGroupAddon, InputGroupInput } from '$lib/components/ui/input-group';
	import { AlertDialog, AlertDialogContent, AlertDialogHeader, AlertDialogTitle, AlertDialogDescription, AlertDialogFooter, AlertDialogCancel, AlertDialogAction } from '$lib/components/ui/alert-dialog';
	import { Button } from '$lib/components/ui/button';
	import { PageHeader } from '$lib/components/orch/page-header';
	import { Segmented } from '$lib/components/orch/segmented';
	import { setMode, userPrefersMode } from 'mode-watcher';
	import { FieldRow, FieldSwitchRow } from '$lib/components/ui/field';
	import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogBody, DialogFooter } from '$lib/components/ui/dialog';
	import Lock from '@lucide/svelte/icons/lock';
	import { toast } from 'svelte-sonner';
	import { store } from '$lib/teams.svelte';
	import { tasks, issues, decisions } from '$lib/mock';
	import { Empty, EmptyHeader, EmptyMedia, EmptyTitle, EmptyDescription, EmptyContent } from '$lib/components/ui/empty';
	import LoaderCircle from '@lucide/svelte/icons/loader-circle';
	import { onMount } from 'svelte';
	import { api } from '$lib/api/client';
	import { useMock } from '$lib/api/env';
	import type { components } from '$lib/api/schema';
	import { langCode, langName } from '$lib/lang';

	type Key = keyof typeof s;
	/// Select 한 줄 — note는 값 옆 회색 설명 (값에 따라 다를 수 있다).
	type Row = { key: Key; label: string; hint: string; icon: Component; options: string[]; note?: (v: string) => string };

	let s = $state({
		name: 'OrchStack',
		repo: 'orchstack/app',
		timezone: 'Asia/Seoul (UTC+9)',
		uiLang: '한국어',
		reportLang: '한국어',
		commitLang: 'English',
		dateFormat: 'iso',
		runLogs: '90일',
		usage: '1년',
		decisions: '영구 보관'
	});
	let saved = $state(false);
	let deleting = $state(false);
	// 워크스페이스 내보내기 (.pen 워크스페이스 내보내기) — Alpha는 고른 항목을 JSON 한 파일로. Run 로그 · .zip은 서버 연결 후.
	let exportOpen = $state(false);
	let pick = $state({ settings: true, teams: true, projects: true, decisions: true });
	function exportWorkspace() {
		const data = {
			app: 'OrchStack', exportedAt: new Date().toISOString(),
			...(pick.settings && { settings: $state.snapshot(s) }),
			...(pick.teams && { teams: $state.snapshot(store.crew), templates: $state.snapshot(store.templates) }),
			...(pick.projects && { projects: $state.snapshot(store.projects), issues, tasks }),
			...(pick.decisions && { decisions })
		};
		const a = document.createElement('a');
		a.href = URL.createObjectURL(new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' }));
		a.download = `${s.name.toLowerCase().replace(/\s+/g, '-')}-workspace.json`;
		a.click();
		URL.revokeObjectURL(a.href);
		exportOpen = false;
		toast.success('워크스페이스를 내보냈어요');
	}
	let confirmName = $state('');

	let loadState = $state<'loading' | 'ready' | 'error'>(useMock ? 'ready' : 'loading');

	/// 화면 값 ↔ 서버 값. 시간대는 화면 표기의 앞 단어(예: 'Asia/Seoul (UTC+9)' → 'Asia/Seoul').
	const dateCode: Record<string, string> = { iso: 'YYYY-MM-DD', short: 'M/D', long: 'M월 D일' };
	const field: Partial<Record<Key, (v: string) => components['schemas']['WorkspacePatch']>> = {
		name: (v) => ({ name: v }),
		repo: (v) => ({ default_repo: v }),
		timezone: (v) => ({ timezone: v.split(' ')[0] }),
		uiLang: (v) => ({ ui_language: langCode[v] ?? v }),
		reportLang: (v) => ({ report_language: langCode[v] ?? v }),
		commitLang: (v) => ({ commit_language: langCode[v] ?? v }),
		dateFormat: (v) => ({ date_format: dateCode[v] ?? v })
	};

	async function load() {
		loadState = 'loading';
		const { data: w } = await api.GET('/workspace').catch(() => ({ data: undefined }));
		if (!w) return void (loadState = 'error');
		Object.assign(s, {
			name: w.name, repo: w.default_repo ?? '', timezone: workspaceRows[1].options.find((o) => o.split(' ')[0] === w.timezone) ?? w.timezone,
			uiLang: langName(w.ui_language), reportLang: langName(w.report_language), commitLang: langName(w.commit_language),
			dateFormat: Object.keys(dateCode).find((k) => dateCode[k] === w.date_format) ?? 'iso'
		});
		loadState = 'ready';
	}
	onMount(() => {
		if (!useMock) void load();
	});

	/// 바로 반영 — 서버 모드는 PATCH가 성공해야 저장됨. 실패하면 입력은 두고 토스트(클라이언트).
	async function set(key: Key, v: string) {
		s[key] = v;
		const body = field[key];
		if (useMock || !body) return void (saved = true);
		saved = false;
		const res = await api.PATCH('/workspace', { body: body(v) }).catch(() => undefined);
		saved = !!res && !res.error;
	}

	/// 테마는 이 브라우저에 바로 적용(mode-watcher)하고, 서버 모드면 워크스페이스에도 저장.
	function setTheme(v: 'light' | 'dark' | 'system') {
		setMode(v);
		saved = true;
		if (!useMock) void api.PATCH('/workspace', { body: { theme: v } }).catch(() => undefined);
	}

	const langs = ['한국어', 'English', '日本語'];
	const workspaceRows: Row[] = [
		{ key: 'repo', label: '기본 저장소', hint: '새 팀 · 태스크가 기본으로 연결돼요', icon: FolderGit2, options: ['orchstack/app', 'orchstack/docs'], note: () => 'main · GitHub App 연결됨' },
		{ key: 'timezone', label: '시간대', hint: '리셋 시각 · 타이머 · 일일 요약 기준', icon: Globe, options: ['Asia/Seoul (UTC+9)', 'Asia/Tokyo (UTC+9)', 'UTC', 'America/Los_Angeles (UTC-7)'], note: (v) => (v === 'Asia/Seoul (UTC+9)' ? '자동 감지' : '') }
	];
	const langRows: Row[] = [
		{ key: 'uiLang', label: '화면 언어', hint: 'OrchStack 인터페이스', icon: Languages, options: langs, note: () => '브라우저 설정 따름' },
		{ key: 'reportLang', label: '에이전트 보고 언어', hint: '완료 보고 · 요약 · 질문 · Orch 메시지', icon: MessageSquare, options: langs, note: () => '모든 연결 · 멤버 기본값' },
		{ key: 'commitLang', label: '커밋 · PR 언어', hint: '커밋 메시지 · PR 본문 · 코드 주석', icon: GitCommitHorizontal, options: langs, note: () => '저장소 CONTRIBUTING 규칙 우선' }
	];
	const keepRows: Row[] = [
		{ key: 'runLogs', label: 'Run 로그', hint: '단계별 로그 · diff · 스크린샷', icon: ScrollText, options: ['30일', '90일', '1년'], note: () => '약 2.1GB 사용 중' },
		{ key: 'usage', label: '토큰 · 비용 기록', hint: '사용량 차트 · 예산 계산', icon: ChartColumn, options: ['90일', '1년', '영구 보관'] },
		{ key: 'decisions', label: '결정 기록', hint: 'DecisionRecord · 반려 사유', icon: BookMarked, options: ['1년', '영구 보관'] }
	];
</script>

<svelte:head><title>일반 · Settings · OrchStack</title></svelte:head>

<!-- .pen FormRow 제목 -->
<!-- .pen FormRow + Select -->
{#snippet selectRow(r: Row)}
	<!-- label로 감싸면 트리거 클릭이 두 번 전달돼 목록이 다시 열린다 -->
	<FieldRow label={r.label} hint={r.hint}>
		<Select type="single" value={s[r.key]} onValueChange={(v) => set(r.key, v)}>
			<SelectTrigger class="w-full" aria-label={r.label}>
				<span class="flex min-w-0 flex-1 items-center gap-2">
					<r.icon class="size-4 text-muted-foreground" />
					{s[r.key]}
					<span class="truncate text-caption font-normal text-muted-foreground">{r.note?.(s[r.key])}</span>
				</span>
			</SelectTrigger>
			<SelectContent>
				{#each r.options as o (o)}<SelectItem value={o} label={o} />{/each}
			</SelectContent>
		</Select>
	</FieldRow>
{/snippet}

<main class="page-main">
	<PageHeader title="일반" desc="워크스페이스 이름 · 언어 · 시간대 · 테마 · 데이터 보관" status={saved && '저장됨'} />

	{#if loadState !== 'ready'}
		<!-- 서버 모드 불러오기 -->
		<Empty class="card py-16 rounded-lg">
			<EmptyHeader>
				{#if loadState === 'error'}
					<EmptyTitle>설정을 불러오지 못했어요</EmptyTitle>
					<EmptyDescription>서버 연결을 확인하고 다시 시도하세요.</EmptyDescription>
				{:else}
					<EmptyMedia variant="icon"><LoaderCircle class="animate-spin" /></EmptyMedia>
					<EmptyTitle>설정을 불러오는 중…</EmptyTitle>
				{/if}
			</EmptyHeader>
			{#if loadState === 'error'}<EmptyContent><Button variant="outline" onclick={load}>다시 시도</Button></EmptyContent>{/if}
		</Empty>
	{:else}
		<div class="flex items-start gap-5">
			<div class="flex min-w-0 flex-1 flex-col gap-5">
				<Card size="sm">
					<CardHeader>
						<CardTitle>워크스페이스</CardTitle>
						<CardDescription>팀원 모두에게 보이는 기본 정보</CardDescription>
					</CardHeader>
					<CardContent>
						<FieldRow label="이름" as="label">
							<InputGroup>
								<InputGroupAddon><Building2 /></InputGroupAddon>
								<InputGroupInput class="text-xs font-medium" value={s.name} onchange={(e) => set('name', e.currentTarget.value)} />
							</InputGroup>
						</FieldRow>
						{#each workspaceRows as r (r.key)}{@render selectRow(r)}{/each}
					</CardContent>
				</Card>

				<Card size="sm">
					<CardHeader>
						<CardTitle>언어 · 지역</CardTitle>
						<CardDescription>화면과 에이전트 출력 언어의 기본값 · 연결 · 멤버별로 바꿀 수 있어요</CardDescription>
					</CardHeader>
					<CardContent>
						{#each langRows as r (r.key)}{@render selectRow(r)}{/each}
						<FieldRow label="날짜 형식">
							<Segmented
								aria-label="날짜 형식"
								options={[
									{ value: 'iso', label: '2026-09-29', icon: Calendar },
									{ value: 'short', label: '9/29', icon: Calendar },
									{ value: 'long', label: '9월 29일', icon: Calendar }
								]}
								bind:value={() => s.dateFormat, (v) => set('dateFormat', v ?? 'iso')}
							/>
						</FieldRow>
					</CardContent>
				</Card>
			</div>

			<div class="flex shrink-0 flex-col w-95 gap-5">
				<Card size="sm">
					<CardHeader><CardTitle>화면</CardTitle></CardHeader>
					<CardContent>
						<FieldRow label="테마">
							<!-- 바로 적용 · 이 브라우저에 저장 · 서버 모드면 워크스페이스에도 -->
							<Segmented
								aria-label="테마"
								options={[
									{ value: 'light', label: '라이트', icon: Sun },
									{ value: 'dark', label: '다크', icon: Moon },
									{ value: 'system', label: '시스템', icon: Monitor }
								]}
								bind:value={() => userPrefersMode.current, (v) => setTheme(v ?? 'light')}
							/>
						</FieldRow>
					</CardContent>
				</Card>

				<Card size="sm">
					<CardHeader>
						<CardTitle>데이터 보관</CardTitle>
						<CardDescription>오래된 기록은 자동으로 정리돼요</CardDescription>
					</CardHeader>
					<CardContent>
						{#each keepRows as r (r.key)}{@render selectRow(r)}{/each}
					</CardContent>
				</Card>

				<Card size="sm">
					<CardHeader>
						<CardTitle>위험 구역</CardTitle>
						<CardDescription>되돌릴 수 없는 작업</CardDescription>
					</CardHeader>
					<CardContent class="flex flex-col gap-2.5 pt-1">
						<!-- 삭제는 서버 연결(#47) 후 실제 동작 -->
						<Button variant="outline" class="w-full" onclick={() => (exportOpen = true)}><Download />워크스페이스 내보내기</Button>
						<Button variant="destructive" class="w-full" onclick={() => ((confirmName = ''), (deleting = true))}><Trash2 />워크스페이스 삭제</Button>
					</CardContent>
				</Card>
			</div>
		</div>
	{/if}
</main>

<AlertDialog bind:open={deleting}>
	<AlertDialogContent>
		<AlertDialogHeader>
			<AlertDialogTitle>워크스페이스를 삭제할까요?</AlertDialogTitle>
			<AlertDialogDescription>팀 · 태스크 · Run 기록 · 연결이 모두 지워지고 되돌릴 수 없어요. 계속하려면 워크스페이스 이름 <b>{s.name}</b>을 입력하세요.</AlertDialogDescription>
		</AlertDialogHeader>
		<InputGroup>
			<InputGroupInput bind:value={confirmName} placeholder={s.name} aria-label="워크스페이스 이름 확인" />
		</InputGroup>
		<AlertDialogFooter>
			<AlertDialogCancel>취소</AlertDialogCancel>
			<AlertDialogAction variant="destructive" disabled={confirmName !== s.name}>삭제</AlertDialogAction>
		</AlertDialogFooter>
	</AlertDialogContent>
</AlertDialog>

<!-- 워크스페이스 내보내기 (.pen 워크스페이스 내보내기) -->
<Dialog bind:open={exportOpen}>
	<DialogContent size="md">
		<DialogHeader icon={Download}>
			<DialogTitle>워크스페이스 내보내기</DialogTitle>
			<DialogDescription>한 파일로 받아 다른 기기에서 가져올 수 있어요</DialogDescription>
		</DialogHeader>
		<DialogBody>
			<h3 class="pt-2 text-sm font-semibold">포함할 것</h3>
			<FieldSwitchRow label="설정 · 연결" hint="언어 · 알림 · 권한 · 연결 목록 (키 원문 제외)" bind:checked={pick.settings} />
			<FieldSwitchRow label="팀 · 템플릿 · 프리셋" hint="멤버 · Instructions · Skills · 정책" bind:checked={pick.teams} />
			<FieldSwitchRow label="프로젝트 · 이슈 · 태스크" hint="완료 조건 · 의존 · 보고서 포함" bind:checked={pick.projects} />
			<FieldSwitchRow label="결정 기록" hint="판단 · 승인 · Orch 대신 결정" bind:checked={pick.decisions} />
			<FieldSwitchRow label="Run 로그 · diff" hint="단계별 로그 · 스크린샷 · 서버 연결 후 (.zip)" checked={false} disabled />
			<div class="mt-4 flex items-start gap-2.5 rounded-md bg-muted px-3 py-2.5 text-caption">
				<Lock class="mt-0.5 size-3.5 shrink-0 text-muted-foreground" />
				<span><span class="font-semibold">API 키 · 토큰 원문은 들어가지 않아요</span><span class="text-muted-foreground"> · 가져온 뒤 각 연결에서 다시 로그인하거나 키를 넣어요</span></span>
			</div>
		</DialogBody>
		<DialogFooter note="지금은 JSON 한 파일 · Run 로그를 담은 .zip은 서버 연결 후">
			<Button variant="ghost" size="sm" onclick={() => (exportOpen = false)}>취소</Button>
			<Button size="sm" disabled={!Object.values(pick).some(Boolean)} onclick={exportWorkspace}><Download />내보내기</Button>
		</DialogFooter>
	</DialogContent>
</Dialog>
