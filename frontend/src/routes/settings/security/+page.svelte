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
	import CircleCheck from '@lucide/svelte/icons/circle-check';
	import X from '@lucide/svelte/icons/x';
	import * as Card from '$lib/components/ui/card';
	import * as InputGroup from '$lib/components/ui/input-group';
	import { Button } from '$lib/components/ui/button';
	import { Pill } from '$lib/components/ui/pill';
	import { Switch } from '$lib/components/ui/switch';
	import TrustPicker from '$lib/components/orch/agent/trust-picker.svelte';
	import ApprovalTable from '$lib/components/orch/agent/approval-table.svelte';
	import { security, auditLog, type AuditLog } from '$lib/mock';
	import { cn } from '$lib/utils';

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

<main class="flex flex-col gap-5 px-8 py-7">
	<header class="flex items-end gap-3">
		<div class="flex flex-1 flex-col gap-1">
			<h1 class="text-2xl font-semibold">권한 · 보안</h1>
			<p class="text-xs text-muted-foreground">기본 Trust 레벨 · 승인 규칙 · 차단 명령 · 비밀 · 감사 로그</p>
		</div>
		<span class="flex items-center gap-1 text-xs font-medium text-status-done" role="status">
			{#if saved}<CircleCheck class="size-3.25" />저장됨{/if}
		</span>
		<Button variant="outline" onclick={exportLog}><ScrollText />감사 로그 내보내기</Button>
	</header>

	<div class="flex items-start gap-5">
		<div class="flex min-w-0 flex-1 flex-col gap-5">
			<Card.Root size="sm">
				<Card.Header>
					<Card.Title>기본 Trust 레벨</Card.Title>
					<Card.Description>새 멤버 · 템플릿의 기본값 · 팀 정책에서 더 낮출 수 있어요</Card.Description>
				</Card.Header>
				<Card.Content><TrustPicker bind:value={s.trust} /></Card.Content>
			</Card.Root>

			<Card.Root size="sm">
				<Card.Header>
					<Card.Title>승인 규칙</Card.Title>
					<Card.Description>모든 팀에 적용 · 팀 · 멤버는 더 엄격하게만 바꿀 수 있어요</Card.Description>
				</Card.Header>
				<Card.Content class="gap-0"><ApprovalTable approvals={s.approvals} /></Card.Content>
			</Card.Root>

			<Card.Root size="sm">
				<Card.Header>
					<Card.Title>항상 차단</Card.Title>
					<Card.Description>어떤 Trust 레벨 · 승인으로도 실행되지 않아요</Card.Description>
					<Card.Action><button type="button" class="text-xs font-medium text-primary hover:underline" onclick={() => (adding = true)}>패턴 추가</button></Card.Action>
				</Card.Header>
				<Card.Content class="gap-0">
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
							<InputGroup.Root class="flex-1">
								<InputGroup.Addon><ShieldX /></InputGroup.Addon>
								<!-- 패턴 추가를 눌러 연 입력 줄이라 바로 입력하게 한다 -->
								<!-- svelte-ignore a11y_autofocus -->
								<InputGroup.Input bind:value={pattern} autofocus placeholder="명령 패턴 · 예: npm publish" aria-label="차단할 명령 패턴" class="font-mono text-xs" />
							</InputGroup.Root>
							<Button type="submit" size="sm" disabled={!pattern.trim()}>추가</Button>
							<Button type="button" variant="ghost" size="sm" onclick={() => ((adding = false), (pattern = ''))}>취소</Button>
						</form>
					{/if}
				</Card.Content>
			</Card.Root>
		</div>

		<div class="flex w-95 shrink-0 flex-col gap-5">
			<Card.Root size="sm">
				<Card.Header>
					<Card.Title>가드 트리거</Card.Title>
					<Card.Description>Run 도중 위험 신호를 감지하면 멈추고 알려요</Card.Description>
				</Card.Header>
				<Card.Content class="gap-0">
					{#each s.guards as g (g.name)}
						<label class="flex items-center gap-3 py-2">
							<span class="flex flex-1 flex-col gap-0.5"><span class="text-xs font-semibold">{g.name}</span><span class="text-caption text-muted-foreground">{g.desc}</span></span>
							<Switch bind:checked={g.on} aria-label={g.name} />
						</label>
					{/each}
				</Card.Content>
			</Card.Root>

			<Card.Root size="sm">
				<Card.Header><Card.Title>비밀 · 키</Card.Title></Card.Header>
				<Card.Content class="gap-0">
					{#each s.secrets as [l, v] (l)}
						<div class="flex h-9 items-center border-t text-xs">
							<span class="flex-1 text-muted-foreground">{l}</span><span>{v}</span>
						</div>
					{/each}
				</Card.Content>
			</Card.Root>

			<Card.Root size="sm">
				<Card.Header>
					<Card.Title>GitHub 계정</Card.Title>
					<Card.Description>커밋 · PR 작성자</Card.Description>
				</Card.Header>
				<Card.Content>
					<div class="flex items-center gap-3 rounded-md border px-3.5 py-3">
						<span class="flex size-9 shrink-0 items-center justify-center rounded-md bg-muted"><GitFork class="size-4" /></span>
						<span class="flex min-w-0 flex-1 flex-col gap-1">
							<span class="flex items-center gap-2 text-body font-semibold">{s.github.name}<Pill dot="bg-status-done" class="bg-success-soft text-status-done">전용 계정</Pill></span>
							<span class="text-caption text-muted-foreground">{s.github.note}</span>
						</span>
						<!-- 계정 변경 화면은 .pen에 아직 없음 -->
						<Button variant="outline" size="sm"><Settings2 />변경</Button>
					</div>
				</Card.Content>
			</Card.Root>

			<Card.Root size="sm">
				<Card.Header>
					<Card.Title>감사 로그</Card.Title>
					<Card.Description>권한 · 연결 · 설정 변경 기록</Card.Description>
					<!-- 전체 목록 화면은 .pen에 아직 없음 -->
					<Card.Action><span class="text-xs font-medium text-primary">모두 보기</span></Card.Action>
				</Card.Header>
				<Card.Content class="gap-0">
					{#each log.slice(0, 5) as l, i (i)}
						{@const Icon = logIcon[l.kind]}
						<div class="flex gap-2.5 border-t py-2.5">
							<span class={cn('flex size-6 shrink-0 items-center justify-center rounded-full', l.kind === 'BLOCK' ? 'bg-destructive-soft text-destructive' : 'bg-primary-soft text-primary')}><Icon class="size-3" /></span>
							<span class="flex min-w-0 flex-1 flex-col gap-0.5 text-xs">
								<span class="flex items-center gap-1.5"><span class="font-mono text-caption font-semibold text-muted-foreground">{l.kind}</span><span class="text-muted-foreground">{l.who}</span><span class="ml-auto text-caption text-subtle-foreground">{l.when}</span></span>
								<span>{l.text}</span>
							</span>
						</div>
					{/each}
				</Card.Content>
			</Card.Root>
		</div>
	</div>
</main>
