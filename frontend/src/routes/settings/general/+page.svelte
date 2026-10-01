<script lang="ts">
	/// Settings › 일반 — 워크스페이스 이름 · 언어 · 시간대 · 테마 · 데이터 보관 (.pen Settings · 일반).
	/// 바꾸면 바로 저장(목데이터)되고 헤더에 저장됨이 뜬다. 서버 연결은 #47.
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
	import { FieldRow } from '$lib/components/ui/field';

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
	let confirmName = $state('');

	/// 목데이터라 바로 반영 — 서버 연결 후 PATCH 성공 시 표시.
	function set(key: Key, v: string) {
		s[key] = v;
		saved = true;
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
						<!-- 바로 적용 · 이 브라우저에 저장 (워크스페이스 설정 API #47 전) -->
						<Segmented
							aria-label="테마"
							options={[
								{ value: 'light', label: '라이트', icon: Sun },
								{ value: 'dark', label: '다크', icon: Moon },
								{ value: 'system', label: '시스템', icon: Monitor }
							]}
							bind:value={() => userPrefersMode.current, (v) => (setMode(v ?? 'light'), (saved = true))}
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
					<!-- 내보내기 · 삭제는 서버 연결(#47) 후 실제 동작 -->
					<Button variant="outline" class="w-full"><Download />워크스페이스 내보내기 (.zip)</Button>
					<Button variant="destructive" class="w-full" onclick={() => ((confirmName = ''), (deleting = true))}><Trash2 />워크스페이스 삭제</Button>
				</CardContent>
			</Card>
		</div>
	</div>
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
