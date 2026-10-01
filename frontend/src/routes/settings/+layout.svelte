<script lang="ts">
	/// Settings 셸 — 좌측 워크스페이스 설정 메뉴 (.pen SettingsNav). 메뉴마다 /settings/<메뉴> 라우트.
	import { page } from '$app/state';
	import Settings from '@lucide/svelte/icons/settings';
	import PlugZap from '@lucide/svelte/icons/plug-zap';
	import Sparkles from '@lucide/svelte/icons/sparkles';
	import Terminal from '@lucide/svelte/icons/terminal';
	import FileStack from '@lucide/svelte/icons/file-stack';
	import FileText from '@lucide/svelte/icons/file-text';
	import Bell from '@lucide/svelte/icons/bell';
	import Shield from '@lucide/svelte/icons/shield';
	import { cn } from '$lib/utils';

	let { children } = $props();

	const menus = [
		{ href: '/settings/general', label: '일반', icon: Settings },
		{ href: '/settings/connections', label: '모델 연결', icon: PlugZap },
		{ href: '/settings/skill-sources', label: '스킬 소스', icon: Sparkles },
		{ href: '/settings/runtimes', label: '실행기 (CLI)', icon: Terminal },
		{ href: '/settings/presets', label: 'Instruction presets', icon: FileStack },
		{ href: '/settings/report-forms', label: '보고서 양식', icon: FileText },
		{ href: '/settings/notifications', label: '알림', icon: Bell },
		{ href: '/settings/security', label: '권한 · 보안', icon: Shield }
	];
</script>

<div class="flex h-full">
	<nav class="side-nav w-60 px-3 py-5" aria-label="워크스페이스 설정">
		<h2 class="section-label px-2.5 pb-1.5">워크스페이스 설정</h2>
		{#each menus as m (m.href)}
			{@const on = page.url.pathname.startsWith(m.href)}
			<a
				href={m.href}
				aria-current={on ? 'page' : undefined}
				class={cn(
					'side-nav-item h-8.5 text-muted-foreground',
					on && 'bg-accent font-semibold text-foreground'
				)}
			>
				<m.icon class="size-3.75" />{m.label}
			</a>
		{/each}
	</nav>
	<div class="min-w-0 flex-1 overflow-y-auto">
		{@render children()}
	</div>
</div>
