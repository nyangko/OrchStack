<script lang="ts">
	/// Settings › 권한 · 보안 — 기본 Trust 레벨 · 승인 규칙 · 항상 차단 · 가드 · 비밀 · GitHub 계정 · 감사 로그 (.pen Settings · 권한 · 보안).
	/// 워크스페이스 기본값 — 팀 · 멤버는 더 엄격하게만 바꾼다. 목데이터 (서버 연결 #47).
	import type { Component } from 'svelte';
	import ScrollText from '@lucide/svelte/icons/scroll-text';
	import ShieldX from '@lucide/svelte/icons/shield-x';
	import ShieldCheck from '@lucide/svelte/icons/shield-check';
	import KeyRound from '@lucide/svelte/icons/key-round';
	import Ban from '@lucide/svelte/icons/ban';
	import GitFork from '@lucide/svelte/icons/git-fork';
	import Settings2 from '@lucide/svelte/icons/settings-2';
	import X from '@lucide/svelte/icons/x';
	import { Card, CardHeader, CardTitle, CardDescription, CardContent, CardAction } from '$lib/components/ui/card';
	import { InputGroup, InputGroupAddon, InputGroupInput } from '$lib/components/ui/input-group';
	import { Button } from '$lib/components/ui/button';
	import { PageHeader } from '$lib/components/orch/page-header';
	import { Pill } from '$lib/components/orch/pill';
	import { Switch } from '$lib/components/ui/switch';
	import { KeyValueRow } from '$lib/components/orch/key-value-row';
	import { HistoryRow } from '$lib/components/orch/history-row';
	import TrustPicker from '$lib/components/orch/agent/trust-picker.svelte';
	import ApprovalTable from '$lib/components/orch/agent/approval-table.svelte';
	import { security, auditLog, type AuditLog } from '$lib/mock';
	import LogIn from '@lucide/svelte/icons/log-in';
	import Search from '@lucide/svelte/icons/search';
	import PlugZap from '@lucide/svelte/icons/plug-zap';
	import CircleCheck from '@lucide/svelte/icons/circle-check';
	import LoaderCircle from '@lucide/svelte/icons/loader-circle';
	import Download from '@lucide/svelte/icons/download';
	import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogBody, DialogFooter } from '$lib/components/ui/dialog';
	import { FieldRow } from '$lib/components/ui/field';
	import { Input } from '$lib/components/ui/input';
	import { Select, SelectTrigger, SelectContent, SelectItem } from '$lib/components/ui/select';
	import { Toggle } from '$lib/components/ui/toggle';
	import { Badge } from '$lib/components/ui/badge';
	import { ChoiceCards, ChoiceCard } from '$lib/components/orch/choice-cards';
	import { toast } from 'svelte-sonner';
	import { onDestroy } from 'svelte';
	
	let s = $state(structuredClone(security));
	let log = $state(structuredClone(auditLog));
	/// 처음 값과 달라지면 저장됨 표시 (자동 저장 가정).
	const initial = JSON.stringify(security);
	const saved = $derived(JSON.stringify(s) !== initial);
	let adding = $state(false);
	let pattern = $state('');

	const logIcon: Record<AuditLog['kind'], Component> = { KEY: KeyRound, POLICY: ShieldCheck, BLOCK: Ban, SETTING: Settings2, LOGIN: LogIn };
	const logTone: Record<AuditLog['kind'], string> = { KEY: 'bg-primary-soft text-primary', POLICY: 'bg-review-soft text-status-review', BLOCK: 'bg-destructive-soft text-destructive', SETTING: 'bg-muted text-muted-foreground', LOGIN: 'bg-success-soft text-status-done' };

	// ── 감사 로그 · 모두 보기 (.pen 감사 로그) — 검색 · 종류 필터 · 날짜별 묶음 · 20개씩 ─────────
	let logOpen = $state(false);
	let logQuery = $state('');
	let logKind = $state<AuditLog['kind'] | 'all'>('all');
	let logLimit = $state(20);
	const logKinds = ['KEY', 'POLICY', 'BLOCK', 'SETTING', 'LOGIN'] as const;
	const logShown = $derived(log.filter((l) => (logKind === 'all' || l.kind === logKind) && `${l.who} ${l.text}`.toLowerCase().includes(logQuery.trim().toLowerCase())));
	/// 날짜(when의 앞 단어)별로 묶는다 — 순서는 기록 순서 그대로.
	const logDays = $derived(
		logShown.slice(0, logLimit).reduce<{ day: string; items: AuditLog[] }[]>((acc, l) => {
			const day = l.when.split(' ')[0];
			(acc.at(-1)?.day === day ? acc.at(-1)!.items : (acc.push({ day, items: [] }), acc.at(-1)!.items)).push(l);
			return acc;
		}, [])
	);

	// ── GitHub 계정 (.pen GitHub 계정 변경) ─────────
	let ghOpen = $state(false);
	let gh = $state({ mode: 'bot', account: 'orch-bot', scope: 'orchstack/*' });
	let ghTest = $state<'idle' | 'pending' | 'ok'>('idle');
	let ghTimer: ReturnType<typeof setTimeout> | undefined;
	onDestroy(() => clearTimeout(ghTimer));
	function openGh() {
		gh = { mode: s.github.name === 'orch-bot' ? 'bot' : 'personal', account: s.github.name, scope: 'orchstack/*' };
		ghTest = 'ok';
		ghOpen = true;
	}
	/// 연결 테스트 (목데이터: 잠시 후 성공 · 서버 연결은 실행기 Task).
	function testGh() {
		ghTest = 'pending';
		ghTimer = setTimeout(() => (ghTest = 'ok'), 900);
	}
	function saveGh() {
		const before = s.github.name;
		s.github = { name: gh.account, note: `${gh.mode === 'bot' ? '사람이 한 작업과 구분돼요' : '커밋 · PR이 내 이름으로 남아요'} · Repo 권한 ${gh.scope}` };
		if (before !== gh.account) log.unshift({ kind: 'SETTING', who: '나', when: '오늘 방금', text: `GitHub 작성자 ${before} → ${gh.account}` });
		ghOpen = false;
		toast.success('GitHub 계정을 저장했어요');
	}

	function addPattern() {
		const p = pattern.trim();
		if (!p || s.blocked.some((b) => b.pattern === p)) return;
		s.blocked.push({ pattern: p, desc: '직접 추가' });
		log.unshift({ kind: 'POLICY', who: '나', when: '방금', text: `항상 차단 추가: ${p}` });
		pattern = '';
		adding = false;
	}

	/// 감사 로그 내보내기 — 지금 보이는 기록을 CSV로 받는다.
	function exportLog() {
		const csv = ['kind,who,when,text', ...log.map((l) => [l.kind, l.who, l.when, l.text].map((v) => `"${v.replaceAll('"', '""')}"`).join(','))].join('\n');
		const url = URL.createObjectURL(new Blob([csv], { type: 'text/csv' }));
		const a = Object.assign(document.createElement('a'), { href: url, download: 'orchstack-audit-log.csv' });
		a.click();
		URL.revokeObjectURL(url);
	}
</script>

<svelte:head><title>권한 · 보안 · Settings · OrchStack</title></svelte:head>

<main class="page-main">
	<PageHeader title="권한 · 보안" desc="기본 Trust 레벨 · 승인 규칙 · 차단 명령 · 비밀 · 감사 로그" status={saved && '저장됨'}>
		<Button variant="outline" onclick={exportLog}><ScrollText />감사 로그 내보내기</Button>
	</PageHeader>

	<div class="flex items-start gap-5">
		<div class="flex min-w-0 flex-1 flex-col gap-5">
			<Card size="sm">
				<CardHeader>
					<CardTitle>기본 Trust 레벨</CardTitle>
					<CardDescription>새 멤버 · 템플릿의 기본값 · 팀 정책에서 더 낮출 수 있어요</CardDescription>
				</CardHeader>
				<CardContent><TrustPicker bind:value={s.trust} /></CardContent>
			</Card>

			<Card size="sm">
				<CardHeader>
					<CardTitle>승인 규칙</CardTitle>
					<CardDescription>모든 팀에 적용 · 팀 · 멤버는 더 엄격하게만 바꿀 수 있어요</CardDescription>
				</CardHeader>
				<CardContent class="gap-0"><ApprovalTable approvals={s.approvals} /></CardContent>
			</Card>

			<Card size="sm">
				<CardHeader>
					<CardTitle>항상 차단</CardTitle>
					<CardDescription>어떤 Trust 레벨 · 승인으로도 실행되지 않아요</CardDescription>
					<CardAction><Button variant="link" size="xs" onclick={() => (adding = true)}>패턴 추가</Button></CardAction>
				</CardHeader>
				<CardContent class="gap-0">
					{#each s.blocked as b, i (b.pattern)}
						<div class="flex items-center gap-3 border-t py-2.5 text-xs">
							<ShieldX class="size-3.5 shrink-0 text-destructive" />
							<span class="w-60 truncate font-medium"><span class="font-mono">{b.pattern}</span>{#if b.scope}{' '}({b.scope}){/if}</span>
							<span class="flex-1 truncate text-muted-foreground">{b.desc}</span>
							<Pill class="bg-destructive-soft text-destructive">차단</Pill>
							{#if b.desc === '직접 추가'}
								<button type="button" aria-label="{b.pattern} 삭제" class="text-subtle-foreground hover:text-foreground" onclick={() => s.blocked.splice(i, 1)}><X class="size-3.5" /></button>
							{/if}
						</div>
					{/each}
					{#if adding}
						<form class="flex items-center gap-2 border-t pt-2.5" onsubmit={(e) => (e.preventDefault(), addPattern())}>
							<InputGroup class="flex-1">
								<InputGroupAddon><ShieldX /></InputGroupAddon>
								<!-- 패턴 추가를 눌러 연 입력 줄이라 바로 입력하게 한다 -->
								<!-- svelte-ignore a11y_autofocus -->
								<InputGroupInput bind:value={pattern} autofocus placeholder="명령 패턴 · 예: npm publish" aria-label="차단할 명령 패턴" class="font-mono text-xs" />
							</InputGroup>
							<Button type="submit" size="sm" disabled={!pattern.trim()}>추가</Button>
							<Button type="button" variant="ghost" size="sm" onclick={() => ((adding = false), (pattern = ''))}>취소</Button>
						</form>
					{/if}
				</CardContent>
			</Card>
		</div>

		<div class="flex shrink-0 flex-col w-95 gap-5">
			<Card size="sm">
				<CardHeader>
					<CardTitle>가드 트리거</CardTitle>
					<CardDescription>Run 도중 위험 신호를 감지하면 멈추고 알려요</CardDescription>
				</CardHeader>
				<CardContent class="gap-0">
					{#each s.guards as g (g.name)}
						<label class="flex items-center gap-3 py-2">
							<span class="flex flex-1 flex-col gap-0.5"><span class="text-xs font-semibold">{g.name}</span><span class="text-caption text-muted-foreground">{g.desc}</span></span>
							<Switch bind:checked={g.on} aria-label={g.name} />
						</label>
					{/each}
				</CardContent>
			</Card>

			<Card size="sm">
				<CardHeader><CardTitle>비밀 · 키</CardTitle></CardHeader>
				<CardContent class="gap-0">
					{#each s.secrets as [l, v] (l)}
						<KeyValueRow label={l} value={v} />
					{/each}
				</CardContent>
			</Card>

			<Card size="sm">
				<CardHeader>
					<CardTitle>GitHub 계정</CardTitle>
					<CardDescription>커밋 · PR 작성자</CardDescription>
				</CardHeader>
				<CardContent>
					<div class="option-card rounded-md">
						<span class="icon-tile"><GitFork class="size-4" /></span>
						<span class="flex min-w-0 flex-1 flex-col gap-1">
							<span class="row-title-strong">{s.github.name}<Pill dot="bg-status-done" class="bg-success-soft text-status-done">전용 계정</Pill></span>
							<span class="text-caption text-muted-foreground">{s.github.note}</span>
						</span>
						<Button variant="outline" size="sm" onclick={openGh}><Settings2 />변경</Button>
					</div>
				</CardContent>
			</Card>

			<Card size="sm">
				<CardHeader>
					<CardTitle>감사 로그</CardTitle>
					<CardDescription>권한 · 연결 · 설정 변경 기록</CardDescription>
					<CardAction><Button variant="link" size="xs" onclick={() => ((logLimit = 20), (logOpen = true))}>모두 보기</Button></CardAction>
				</CardHeader>
				<CardContent class="gap-0">
					{#each log.slice(0, 5) as l, i (i)}
						{@const Icon = logIcon[l.kind]}
						<HistoryRow icon={Icon} tone={logTone[l.kind]} kind={l.kind} who={l.who} when={l.when} text={l.text} />
					{/each}
				</CardContent>
			</Card>
		</div>
	</div>
</main>

<!-- 감사 로그 · 모두 보기 (.pen 감사 로그) -->
<Dialog bind:open={logOpen}>
	<DialogContent size="md" tall>
		<DialogHeader icon={ScrollText}>
			<DialogTitle>감사 로그</DialogTitle>
			<DialogDescription>권한 · 연결 · 설정 변경 기록 · 90일 보관</DialogDescription>
			{#snippet sub()}
				<div class="flex flex-wrap items-center gap-2">
					<InputGroup class="h-8 w-56">
						<InputGroupAddon><Search /></InputGroupAddon>
						<InputGroupInput bind:value={logQuery} placeholder="누가 · 무엇을 검색" aria-label="감사 로그 검색" />
					</InputGroup>
					<div class="flex flex-wrap gap-1" role="group" aria-label="종류">
						<Toggle variant="chip" count={log.length} pressed={logKind === 'all'} onPressedChange={() => (logKind = 'all')}>전체</Toggle>
						{#each logKinds as k (k)}
							<Toggle variant="chip" count={log.filter((l) => l.kind === k).length} pressed={logKind === k} onPressedChange={() => (logKind = k)}>{k}</Toggle>
						{/each}
					</div>
				</div>
			{/snippet}
		</DialogHeader>
		<DialogBody class="gap-1">
			{#each logDays as g (g.day)}
				<span class="pt-2 text-caption font-semibold text-muted-foreground">{g.day}</span>
				{#each g.items as l, i (i)}
					<HistoryRow icon={logIcon[l.kind]} tone={logTone[l.kind]} kind={l.kind} who={l.who} when={l.when.split(' ').slice(1).join(' ')} text={l.text} />
				{/each}
			{:else}
				<p class="py-10 text-center text-xs text-muted-foreground">조건에 맞는 기록이 없어요.</p>
			{/each}
			{#if logShown.length > logLimit}
				<Button variant="link" size="xs" class="w-fit" onclick={() => (logLimit += 20)}>이전 기록 더 보기 · {logShown.length - logLimit}개</Button>
			{/if}
		</DialogBody>
		<DialogFooter note="보관 기간은 일반 › 데이터 보관에서 바꿔요">
			<Button variant="ghost" size="sm" onclick={() => (logOpen = false)}>닫기</Button>
			<Button size="sm" onclick={exportLog}><Download />CSV 내보내기</Button>
		</DialogFooter>
	</DialogContent>
</Dialog>

<!-- GitHub 계정 변경 (.pen GitHub 계정 변경) -->
<Dialog bind:open={ghOpen}>
	<DialogContent size="md">
		<DialogHeader icon={GitFork}>
			<DialogTitle>GitHub 계정</DialogTitle>
			<DialogDescription>커밋 · PR을 누구 이름으로 만들지 정해요</DialogDescription>
		</DialogHeader>
		<DialogBody>
			<FieldRow label="작성자" hint="사람 작업과 구분하려면 bot 권장">
				<ChoiceCards aria-label="작성자" class="flex flex-col gap-2" bind:value={() => gh.mode, (v) => ((gh.mode = v as string), (gh.account = v === 'bot' ? 'orch-bot' : 'pixel'))}>
					<ChoiceCard value="bot" class="gap-1 rounded-lg px-3.5 py-3">
						<span class="row-title-strong"><span class="font-mono text-caption text-muted-foreground">A</span>전용 bot 계정<Badge class="ml-auto">추천</Badge></span>
						<span class="text-caption text-muted-foreground">커밋 · PR이 bot 이름으로 남아 사람 작업과 구분돼요</span>
					</ChoiceCard>
					<ChoiceCard value="personal" class="gap-1 rounded-lg px-3.5 py-3">
						<span class="row-title-strong"><span class="font-mono text-caption text-muted-foreground">B</span>내 개인 계정</span>
						<span class="text-caption text-muted-foreground">커밋 · PR이 내 이름으로 남아요 · 혼자 쓸 때</span>
					</ChoiceCard>
				</ChoiceCards>
			</FieldRow>
			<FieldRow label="계정" hint="GitHub App이 설치된 계정만">
				<Select type="single" bind:value={gh.account}>
					<SelectTrigger class="w-full" aria-label="계정"><GitFork class="size-4" />{gh.account}</SelectTrigger>
					<SelectContent>
						<SelectItem value="orch-bot" label="orch-bot · orchstack 조직" />
						<SelectItem value="pixel" label="pixel · 개인 계정" />
					</SelectContent>
				</Select>
			</FieldRow>
			<FieldRow label="저장소 범위" hint="이 범위 밖 저장소엔 push 안 해요" as="label"><Input class="font-mono" bind:value={gh.scope} /></FieldRow>
			<div class={['mt-4 flex items-center gap-2 rounded-md px-3 py-2.5 text-caption', ghTest === 'ok' ? 'bg-success-soft' : 'bg-muted']}>
				{#if ghTest === 'pending'}<LoaderCircle class="size-3.5 animate-spin text-muted-foreground" />{:else}<CircleCheck class="size-3.5 text-status-done" />{/if}
				<span class="flex-1 font-medium">{ghTest === 'pending' ? '확인 중…' : `${gh.account} 연결됨 · 권한 contents · pull_requests`}</span>
				<Button variant="outline" size="sm" disabled={ghTest === 'pending'} onclick={testGh}><PlugZap />연결 테스트</Button>
			</div>
		</DialogBody>
		<DialogFooter note="토큰은 이 기기 키체인에만 있어요" noteIcon={KeyRound}>
			<Button variant="ghost" size="sm" onclick={() => (ghOpen = false)}>취소</Button>
			<Button size="sm" disabled={!gh.scope.trim() || ghTest === 'pending'} onclick={saveGh}>저장</Button>
		</DialogFooter>
	</DialogContent>
</Dialog>
