<script lang="ts">
	/// 에이전트 도구 (.pen Tools & MCP). MCP 설치 · GitHub 계정 · CLI 기본 도구(권한에서 계산) · 허용 이유.
	import CircleCheck from '@lucide/svelte/icons/circle-check';
	import RefreshCw from '@lucide/svelte/icons/refresh-cw';
	import ShieldCheck from '@lucide/svelte/icons/shield-check';
	import Github from '@lucide/svelte/icons/git-fork';
	import Plug from '@lucide/svelte/icons/plug';
	import Ban from '@lucide/svelte/icons/ban';
	import FileText from '@lucide/svelte/icons/file-text';
	import FilePen from '@lucide/svelte/icons/file-pen';
	import Terminal from '@lucide/svelte/icons/terminal';
	import GitPullRequest from '@lucide/svelte/icons/git-pull-request';
	import Globe from '@lucide/svelte/icons/globe';
	import ShieldX from '@lucide/svelte/icons/shield-x';
	import * as Card from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Pill } from '$lib/components/orch/pill';
	import { Segmented } from '$lib/components/orch/segmented';
	import { mcpServers, type AgentConfig } from '$lib/mock';
	import { scopeText, toggleIn } from '$lib/teams.svelte';
	
	let { config: c, base, teamName }: { config: AgentConfig; base?: AgentConfig; teamName: string } = $props();

	let mcpChecked = $state('2분 전');
	/// CLI 기본 도구 — 권한 설정에서 계산한다 (.pen CLI 기본 도구).
	const cliTools = $derived.by(() => {
		const pr = c.approvals.find((a) => a.action === 'PR 생성')?.policy ?? '승인 필요';
		return [
			{ icon: FileText, name: 'read', scope: '모든 파일', v: '허용' },
			{ icon: FilePen, name: 'edit / write', scope: c.trust >= 3 ? scopeText(c) : 'diff 제안만', v: c.trust >= 3 ? '허용' : '제안' },
			{ icon: Terminal, name: 'shell', scope: 'lint · test · dev, git status · diff · commit', v: c.trust >= 2 ? '허용 목록' : '차단' },
			{ icon: GitPullRequest, name: 'git push / PR', scope: pr, v: pr === '자동' ? '허용' : pr === '차단' ? '차단' : '승인' },
			{ icon: Globe, name: 'web fetch', scope: '허용 도메인만 (docs 등)', v: '허용 목록' },
			{ icon: ShieldX, name: 'git push --force · reset --hard', scope: '항상 차단', v: '차단' },
		];
	});
</script>

<div class="flex items-center gap-2.5">
	<h2 class="text-lg font-semibold">Tools & MCP</h2>
	{#if base}
		<Pill class={(!!base && (c.mcp.join() !== base.mcp.join() || c.github !== base.github)) ? 'bg-primary-soft text-primary' : ''}>{(!!base && (c.mcp.join() !== base.mcp.join() || c.github !== base.github)) ? '템플릿과 다름' : '템플릿과 동일'}</Pill>
	{/if}
	<span class="flex items-center gap-1 text-xs text-status-done"><CircleCheck class="size-3.5" />MCP {c.mcp.length}/{c.mcp.length} 정상 · {mcpChecked} 점검</span>
	<span class="flex-1"></span>
	<Button variant="outline" size="sm" onclick={() => (mcpChecked = '방금')}><RefreshCw />다시 점검</Button>
</div>
<p class="info-line"><ShieldCheck class="size-3.5 shrink-0 text-primary" /><span><span class="font-medium text-foreground">적용 권한</span> · 목록에 없는 도구는 차단 · Instructions는 목록을 줄일 수만 있어요 (늘릴 수 없음)</span></p>
<div class="card flex items-center gap-3 p-3.5 rounded-md">
	<Github class="size-4.5 shrink-0" />
	<span class="flex min-w-0 flex-1 flex-col gap-0.5">
		<span class="text-body font-medium">GitHub 계정 · {c.github === 'bot' ? '전용 계정 (orch-bot) 사용 중' : '내 GitHub 사용 중'}</span>
		<span class="text-xs text-muted-foreground">{c.github === 'bot' ? '커밋 · PR 작성자가 orch-bot 으로 기록돼요. 담당자 개인 계정으로 바꾸면 사람이 한 작업과 구분이 안 돼요.' : '커밋 · PR이 내 계정으로 기록돼요. 사람이 한 작업과 구분이 안 돼요.'}</span>
	</span>
	<Segmented class="w-56" aria-label="GitHub 계정" options={[{ value: 'mine', label: '내 GitHub' }, { value: 'bot', label: '전용 계정' }]} bind:value={() => c.github, (v) => (c.github = v as AgentConfig['github'])} />
</div>
<div class="flex items-start gap-4">
	<div class="flex min-w-0 flex-1 flex-col gap-4">
		<Card.Root size="sm">
			<Card.Header>
				<Card.Title>MCP 서버</Card.Title>
				<Card.Description>설치됨 = 매 Run 컨텍스트에 도구가 추가됨 · 접근 가능 = 허용만, 비용 없음</Card.Description>
			</Card.Header>
			<Card.Content class="gap-0">
				{#each mcpServers as srv (srv.name)}
					{@const on = c.mcp.includes(srv.name)}
					<div class="list-row">
						<Plug class={['size-4 shrink-0', on ? 'text-primary' : 'text-muted-foreground']} />
						<span class="flex min-w-0 flex-1 flex-col gap-0.5">
							<span class="row-title">
								{srv.name}
								<Pill class={on ? 'bg-primary-soft text-primary' : ''}>{on ? '설치됨' : '접근 가능'}</Pill>
								{#if srv.auth}<Pill class="bg-warning-soft text-status-waiting">인증 필요</Pill>{/if}
								{#if base && base.mcp.includes(srv.name) !== on}<span class="size-1.5 rounded-full bg-primary" aria-label="템플릿과 다름"></span>{/if}
							</span>
							<span class="text-xs text-muted-foreground">{srv.desc} · {srv.tools} tools{on ? ` · +${srv.tok} tok` : ' · 필요 시 설치'}</span>
						</span>
						{#if on}<span class="text-caption text-status-done">정상</span>{/if}
						<Button variant={on ? 'ghost' : 'outline'} size="sm" disabled={!on && srv.auth} title={!on && srv.auth ? 'Settings › 모델 연결에서 인증 후 설치' : undefined} onclick={() => toggleIn(c.mcp, srv.name)}>{on ? '제거' : '설치'}</Button>
					</div>
				{/each}
			</Card.Content>
		</Card.Root>
		<Card.Root size="sm">
			<Card.Header>
				<Card.Title>CLI 기본 도구</Card.Title>
				<Card.Description>권한 설정(Trust · 파일 범위 · 승인 규칙)에서 계산돼요</Card.Description>
			</Card.Header>
			<Card.Content class="gap-0">
				{#each cliTools as tool (tool.name)}
					<div class="flex items-center gap-3 border-t py-2 text-xs first:border-t-0">
						<tool.icon class="size-3.5 text-muted-foreground" />
						<span class="w-52 font-mono font-medium">{tool.name}</span>
						<span class="flex-1 truncate text-muted-foreground">{tool.scope}</span>
						<Pill class={tool.v === '차단' ? 'bg-destructive-soft text-destructive' : tool.v === '승인' || tool.v === '제안' ? 'bg-warning-soft text-status-waiting' : 'bg-success-soft text-status-done'}>{tool.v}</Pill>
					</div>
				{/each}
			</Card.Content>
		</Card.Root>
	</div>
	<Card.Root size="sm" class="w-72 shrink-0">
		<Card.Header><Card.Title>이 도구들이 허용된 이유</Card.Title></Card.Header>
		<Card.Content class="gap-2.5 text-xs">
			{#each [['접근 프로필', `Trust ${c.trust} · ${scopeText(c)} 쓰기`], ['팀 정책', `${teamName} · PR 생성은 ${c.approvals.find((a) => a.action === 'PR 생성')?.policy}, Destructive git 항상 차단`], ['Instructions', "AGENT.md 의 '새 의존성 추가 금지' → pnpm add 제외 (좁히기만 가능)"]] as [k, v] (k)}
				<span class="flex flex-col gap-0.5"><span class="font-medium">{k}</span><span class="text-muted-foreground">{v}</span></span>
			{/each}
			<span class="border-t pt-2.5 font-medium">사용할 수 없는 도구</span>
			{#each ['pnpm add · npm install', ...c.exclude.slice(0, 2).map((g) => `${g} 접근`), '원격 브랜치 삭제'] as b (b)}
				<span class="flex items-center gap-1.5 text-muted-foreground"><Ban class="size-3 text-destructive" />{b}</span>
			{/each}
		</Card.Content>
	</Card.Root>
</div>
