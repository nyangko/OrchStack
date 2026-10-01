<script lang="ts">
	/// 에이전트 스킬 (.pen Skills · 스킬 추가 · 스킬 상세 · 스킬 소스 연동). 템플릿 기본값과 멤버 설정에 같이 쓴다.
	/// base가 있으면(멤버) 템플릿과 다른 항목을 표시한다. 바꾸면 바로 저장된다.
	import type { Component } from 'svelte';
	import Search from '@lucide/svelte/icons/search';
	import Plus from '@lucide/svelte/icons/plus';
	import ArrowLeft from '@lucide/svelte/icons/arrow-left';
	import CircleCheck from '@lucide/svelte/icons/circle-check';
	import Plug from '@lucide/svelte/icons/plug';
	import Layers from '@lucide/svelte/icons/layers';
	import * as Card from '$lib/components/ui/card';
	import * as Tabs from '$lib/components/ui/tabs';
	import * as InputGroup from '$lib/components/ui/input-group';
	import { Button } from '$lib/components/ui/button';
	import { Pill } from '$lib/components/ui/pill';
	import { Switch } from '$lib/components/ui/switch';
	import { LimitRow } from '$lib/components/ui/limit-row';
	import { RuntimeLogo, type Runtime } from '$lib/components/ui/runtime-logo';
	import { mcpServers, type AgentConfig } from '$lib/mock';
	import { store, cfgTok, toggleIn, installSkill, sourceMeta } from '$lib/teams.svelte';
	import SkillBrowser from './skill-browser.svelte';
	import SkillSourcesDialog from './skill-sources-dialog.svelte';
	import { cn } from '$lib/utils';

	let {
		config: c,
		base,
		runtime,
		target,
	}: {
		config: AgentConfig;
		/** 비교 기준 (멤버의 원본 템플릿). */
		base?: AgentConfig;
		runtime: Runtime;
		/** 추가 대상 이름 (예: "Frontend Developer 템플릿", "진"). */
		target: string;
	} = $props();

	let skillQuery = $state('');
	let skillAdd = $state(false);
	let shTab = $state('sh');
	let sourcesOpen = $state(false);
	const q = $derived(skillQuery.trim().toLowerCase());
	const active = $derived([...c.skills, ...c.mcp].filter((k) => k.includes(q)));
	const avail = $derived(store.library.filter((k) => !c.skills.includes(k.name) && k.name.includes(q)));
	const tok = $derived(cfgTok(c));
</script>

{#snippet sameAs(changed: boolean, base?: AgentConfig)}
	{#if base}
		<Pill class={changed ? 'bg-primary-soft text-primary' : ''}>{changed ? '템플릿과 다름' : '템플릿과 동일'}</Pill>
	{/if}
{/snippet}

<!-- .pen SkillRow — 글자 타일 · 이름 · 설명 · 출처 · 토큰 · 켜기 -->
{#snippet skillRow(name: string, desc: string, source: string, SourceIcon: Component, tok: number, on: boolean, toggle: () => void, changed = false)}
	<div class="list-row">
		<span class="skill-mark-sm">{name[0]}</span>
		<span class="row-text">
			<span class="row-title">{name}{#if changed}<span class="size-1.5 rounded-full bg-primary" aria-label="템플릿과 다름"></span>{/if}</span>
			<span class="truncate text-xs text-muted-foreground">{desc}</span>
			<span class="subtle-meta gap-3">
				<span class="flex items-center gap-1"><SourceIcon class="size-3" />{source}</span>
				<span class="font-mono">+{tok} tok</span>
			</span>
		</span>
		<Switch checked={on} onCheckedChange={toggle} aria-label="{name} 사용" />
	</div>
{/snippet}

<div class="flex items-center gap-2.5">
	<h2 class="text-lg font-semibold">{skillAdd ? '스킬 추가' : 'Skills'}</h2>
	{@render sameAs(!!base && (c.skills.join() !== base.skills.join() || c.mcp.join() !== base.mcp.join()), base)}
	<span class="text-xs text-muted-foreground">{c.skills.length + c.mcp.length} / {store.library.length + c.mcp.length} 활성</span>
	<Pill>다음 Run부터 적용</Pill>
	<span class="saved-note"><CircleCheck class="size-3" />저장됨</span>
	<span class="flex-1"></span>
	{#if skillAdd}
		<Button variant="outline" size="sm" onclick={() => (skillAdd = false)}><ArrowLeft />설치된 스킬 {c.skills.length + c.mcp.length}</Button>
	{:else}
		<InputGroup.Root class="h-8 w-52 bg-card">
			<InputGroup.Addon><Search /></InputGroup.Addon>
			<InputGroup.Input bind:value={skillQuery} placeholder="스킬 검색" aria-label="스킬 검색" />
		</InputGroup.Root>
		<Button variant="outline" size="sm" onclick={() => ((skillAdd = true), (shTab = 'sh'))}><Plus />스킬 추가</Button>
	{/if}
</div>
{#if skillAdd}
	<Tabs.Root bind:value={shTab} class="gap-3.5">
		<div class="flex items-center gap-2 border-b">
			<Tabs.List variant="line">
				<Tabs.Trigger value="installed">설치됨 {store.library.length}</Tabs.Trigger>
				<Tabs.Trigger value="sh">skills.sh 탐색</Tabs.Trigger>
				<Tabs.Trigger value="team">팀 스킬 {store.library.filter((k) => k.source === 'Team').length}</Tabs.Trigger>
			</Tabs.List>
			<span class="flex-1"></span>
			<Button variant="ghost" size="sm" onclick={() => (sourcesOpen = true)}><Plug />소스 연동 · skills.sh {store.sources.sources[0].state}</Button>
		</div>
		<Tabs.Content value="sh" class="flex flex-col gap-3">
			<SkillBrowser added={(n) => c.skills.includes(n)} onadd={(h) => (installSkill(h), c.skills.includes(h.name) || c.skills.push(h.name))} {target} />
		</Tabs.Content>
		{#each ['installed', 'team'] as v (v)}
			<Tabs.Content value={v} class="rounded-lg border bg-card px-4 py-1">
				{#each store.library.filter((k) => v === 'installed' || k.source === 'Team') as k (k.name)}
					{@render skillRow(k.name, k.desc, k.version ? `${sourceMeta[k.source].label} · ${k.version}` : sourceMeta[k.source].label, sourceMeta[k.source].icon, k.tok, c.skills.includes(k.name), () => toggleIn(c.skills, k.name))}
				{/each}
			</Tabs.Content>
		{/each}
	</Tabs.Root>
{:else}
	<div class="flex items-start gap-4">
		<div class="col-fill gap-4">
			<Card.Root size="sm">
				<Card.Header><Card.Title class="flex items-center gap-2">이 에이전트에서 활성 <span class="font-normal text-muted-foreground">{active.length}</span></Card.Title></Card.Header>
				<Card.Content class="gap-0">
					{#each active as k (k)}
						{@const lib = store.library.find((x) => x.name === k)}
						{@const srv = mcpServers.find((x) => x.name === k)}
						{#if srv && c.mcp.includes(k)}
							{@render skillRow(k, `${srv.desc} · ${srv.tools} tools`, 'MCP · Tools & MCP에서 연결', Plug, srv.tok, true, () => toggleIn(c.mcp, k), !!base && !base.mcp.includes(k))}
						{:else if lib}
							{@render skillRow(k, lib.desc, lib.version ? `${sourceMeta[lib.source].label} · ${lib.version}` : sourceMeta[lib.source].label, sourceMeta[lib.source].icon, lib.tok, true, () => toggleIn(c.skills, k), !!base && !base.skills.includes(k))}
						{/if}
					{:else}
						<p class="text-xs text-muted-foreground">켜진 스킬이 없어요.</p>
					{/each}
				</Card.Content>
			</Card.Root>
			<Card.Root size="sm">
				<Card.Header><Card.Title class="flex items-center gap-2">라이브러리에서 사용 가능 <span class="font-normal text-muted-foreground">{avail.length}</span></Card.Title></Card.Header>
				<Card.Content class="gap-0">
					{#each avail as k (k.name)}
						{@render skillRow(k.name, k.desc, k.version ? `${sourceMeta[k.source].label} · ${k.version}` : sourceMeta[k.source].label, sourceMeta[k.source].icon, k.tok, false, () => toggleIn(c.skills, k.name), !!base && base.skills.includes(k.name))}
					{:else}
						<p class="text-xs text-muted-foreground">모두 켜져 있어요. 새 스킬은 ‘스킬 추가’에서 찾아요.</p>
					{/each}
				</Card.Content>
			</Card.Root>
		</div>
		<aside class="aside-col w-72 gap-4">
			<Card.Root size="sm">
				<Card.Header>
					<Card.Title>컨텍스트 영향</Card.Title>
					<Card.Description>활성 스킬은 매 Run 컨텍스트에 추가돼요.</Card.Description>
				</Card.Header>
				<Card.Content><LimitRow icon={Layers} label="활성 스킬 합계" used="{(tok / 1000).toFixed(1)}K" max="/ 권장 8K" value={(tok / 8000) * 100} note="{Math.round((tok / 8000) * 100)}% · {tok < 6000 ? '여유 있음' : '많아요'}" warn={tok >= 6000} /></Card.Content>
			</Card.Root>
			<Card.Root size="sm">
				<Card.Header><Card.Title>CLI별 적용 방식</Card.Title></Card.Header>
				<Card.Content class="gap-2 text-xs">
					{#each [['codex', 'Codex CLI', 'AGENTS.md에 스킬 요약을 포함'], ['claude', 'Claude Code', '~/.claude/skills 로 동기화']] as const as [r, n, d] (r)}
						<div class={cn('skill-suggest', r === runtime && 'bg-primary-soft')}>
							<RuntimeLogo runtime={r} class="size-4 ring-0" />
							<span class="flex flex-col gap-0.5"><span class="font-medium">{n}{r === runtime ? ' · 현재' : ''}</span><span class="text-muted-foreground">{d}</span></span>
						</div>
					{/each}
				</Card.Content>
			</Card.Root>
		</aside>
	</div>
{/if}


<SkillSourcesDialog bind:open={sourcesOpen} />
