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
	
	let s = $state(structuredClone(security));
	let log = $state(structuredClone(auditLog));
	/// 처음 값과 달라지면 저장됨 표시 (자동 저장 가정).
	const initial = JSON.stringify(security);
	const saved = $derived(JSON.stringify(s) !== initial);
	let adding = $state(false);
	let pattern = $state('');

	const logIcon: Record<AuditLog['kind'], Component> = { KEY: KeyRound, POLICY: ShieldCheck, BLOCK: Ban };

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
						<!-- 계정 변경 화면은 .pen에 아직 없음 -->
						<Button variant="outline" size="sm"><Settings2 />변경</Button>
					</div>
				</CardContent>
			</Card>

			<Card size="sm">
				<CardHeader>
					<CardTitle>감사 로그</CardTitle>
					<CardDescription>권한 · 연결 · 설정 변경 기록</CardDescription>
					<!-- 전체 목록 화면은 .pen에 아직 없음 -->
					<CardAction><span class="text-xs font-medium text-primary">모두 보기</span></CardAction>
				</CardHeader>
				<CardContent class="gap-0">
					{#each log.slice(0, 5) as l, i (i)}
						{@const Icon = logIcon[l.kind]}
						<HistoryRow icon={Icon} tone={l.kind === 'BLOCK' ? 'bg-destructive-soft text-destructive' : 'bg-primary-soft text-primary'} kind={l.kind} who={l.who} when={l.when} text={l.text} />
					{/each}
				</CardContent>
			</Card>
		</div>
	</div>
</main>
