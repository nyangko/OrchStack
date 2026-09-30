<script lang="ts">
	/// 앱 셸 (.pen GlobalNav). 모든 페이지 상단에 한 번만 그린다.
	import '../app.css';
	import favicon from '$lib/assets/logo/favicon.png';
	import faviconDark from '$lib/assets/logo/favicon-dark.png';
	import logo from '$lib/assets/logo/logo.png';
	import { page } from '$app/state';
	import Users from '@lucide/svelte/icons/users';
	import ListChecks from '@lucide/svelte/icons/list-checks';
	import Blocks from '@lucide/svelte/icons/blocks';
	import Settings from '@lucide/svelte/icons/settings';
	import Search from '@lucide/svelte/icons/search';
	import Bell from '@lucide/svelte/icons/bell';
	import * as InputGroup from '$lib/components/ui/input-group';
	import { Button } from '$lib/components/ui/button';
	import { Kbd } from '$lib/components/ui/kbd';
	import { cn } from '$lib/utils';

	let { children } = $props();

	// .pen에서 Agents 링크는 숨김(에이전트는 Teams 안에서 관리). 라벨은 i18n 도입 시 교체.
	const links = [
		{ href: '/teams', label: 'Teams', icon: Users },
		{ href: '/tasks', label: 'Tasks', icon: ListChecks },
		{ href: '/skills', label: 'Skills & MCP', icon: Blocks },
		{ href: '/settings', label: 'Settings', icon: Settings }
	];
</script>

<svelte:head>
	<link rel="icon" type="image/png" href={favicon} media="(prefers-color-scheme: light)" />
	<link rel="icon" type="image/png" href={faviconDark} media="(prefers-color-scheme: dark)" />
	<link rel="apple-touch-icon" href="/apple-touch-icon.png" />
	<title>OrchStack</title>
</svelte:head>

<div class="flex h-dvh flex-col">
	<!-- 첫 실행(/setup)은 상단 메뉴 없이 단독 화면 -->
	{#if !page.url.pathname.startsWith('/setup')}
	<header class="flex h-13 shrink-0 items-center gap-6 border-b bg-card px-4">
		<a href="/" class="flex shrink-0 items-center">
			<img src={logo} alt="OrchStack" class="h-5 w-auto" />
		</a>
		<nav class="flex min-w-0 flex-1 items-center gap-0.5 overflow-x-auto" aria-label="주요 메뉴">
			{#each links as l (l.href)}
				{@const on = page.url.pathname.startsWith(l.href)}
				<a
					href={l.href}
					aria-current={on ? 'page' : undefined}
					class={cn(
						'flex h-8 shrink-0 items-center gap-2 rounded-md px-2 text-sm whitespace-nowrap outline-none hover:bg-sidebar-accent focus-visible:ring-3 focus-visible:ring-ring/50',
						on && 'bg-sidebar-accent font-medium'
					)}
				>
					<l.icon class="size-4" />
					{l.label}
				</a>
			{/each}
		</nav>
		<InputGroup.Root class="w-75">
			<InputGroup.Addon><Search /></InputGroup.Addon>
			<InputGroup.Input placeholder="Search tasks, agents, issues" aria-label="검색" />
			<InputGroup.Addon align="inline-end"><Kbd>⌘K</Kbd></InputGroup.Addon>
		</InputGroup.Root>
		<Button variant="ghost" size="icon" aria-label="알림"><Bell /></Button>
		<span class="flex size-7 items-center justify-center rounded-full bg-primary-soft text-xs font-semibold text-primary" aria-label="사용자">S</span>
	</header>
	{/if}
	<div class="min-h-0 flex-1">
		{@render children()}
	</div>
</div>
