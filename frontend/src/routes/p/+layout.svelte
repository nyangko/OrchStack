<script lang="ts">
	/// Workbench 셸 (.pen Project Tabs). 열린 프로젝트 탭 · All Projects · 새 프로젝트.
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import LayoutGrid from '@lucide/svelte/icons/layout-grid';
	import X from '@lucide/svelte/icons/x';
	import Plus from '@lucide/svelte/icons/plus';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import { projects } from '$lib/mock';
	import { cn } from '$lib/utils';

	let { children } = $props();

	// 열린 탭은 화면 상태(서버에 저장하지 않음). 닫아도 프로젝트는 그대로다.
	let open = $state(projects.map((p) => p.sn));
	const tabs = $derived(projects.filter((p) => open.includes(p.sn)));
	const current = $derived(Number(page.params.project));

	/// 탭을 닫고, 보고 있던 탭이면 옆 탭(없으면 All Projects)으로 이동한다.
	function close(sn: number) {
		const i = open.indexOf(sn);
		open = open.filter((s) => s !== sn);
		if (sn === current) goto(open.length ? `/p/${open[Math.min(i, open.length - 1)]}` : '/p');
	}
</script>

<div class="flex h-full flex-col">
	<div class="flex h-10 shrink-0 items-end gap-0.5 overflow-x-auto border-b bg-muted px-3">
		<a
			href="/p"
			aria-current={page.url.pathname === '/p' ? 'page' : undefined}
			class={cn(
				'flex h-8 shrink-0 items-center gap-1.5 rounded-t-lg px-3 text-body font-medium text-muted-foreground hover:text-foreground',
				page.url.pathname === '/p' && 'relative -mb-px border border-b-0 bg-background text-foreground'
			)}
		>
			<LayoutGrid class="size-3.5" />
			All Projects
			<Badge variant="secondary" class="px-1.5">{projects.length}</Badge>
		</a>
		{#each tabs as p (p.sn)}
			{@const on = p.sn === current}
			<div
				class={cn(
					'flex h-8 shrink-0 items-center gap-2 rounded-t-lg pr-2 pl-3',
					on ? 'relative -mb-px border border-b-0 bg-background' : 'text-muted-foreground hover:text-foreground'
				)}
			>
				<a href="/p/{p.sn}" aria-current={on ? 'page' : undefined} class="flex items-center gap-2 text-body font-medium outline-none focus-visible:underline">
					<span class={cn('size-1.5 rounded-full', p.dot)}></span>
					{p.name}
				</a>
				<button
					type="button"
					aria-label="{p.name} 탭 닫기"
					onclick={() => close(p.sn)}
					class="rounded-xs text-muted-foreground outline-none hover:text-foreground focus-visible:ring-3 focus-visible:ring-ring/50"
				>
					<X class="size-3.5" />
				</button>
			</div>
		{/each}
		<Button variant="ghost" size="icon-sm" class="mb-0.5 shrink-0 text-muted-foreground" aria-label="새 프로젝트"><Plus /></Button>
	</div>
	<div class="min-h-0 flex-1">
		{@render children()}
	</div>
</div>
