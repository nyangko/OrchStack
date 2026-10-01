<script lang="ts">
	/// 스킬 소스 연동 (.pen Skill Sources Dialog) — 워크스페이스 전체 설정. 사본에서 고치고 저장할 때 반영한다.
	import Plug from '@lucide/svelte/icons/plug';
	import Globe from '@lucide/svelte/icons/globe';
	import Github from '@lucide/svelte/icons/git-fork';
	import Folder from '@lucide/svelte/icons/folder';
	import RefreshCw from '@lucide/svelte/icons/refresh-cw';
	import Terminal from '@lucide/svelte/icons/terminal';
	import Check from '@lucide/svelte/icons/check';
	import * as Dialog from '$lib/components/ui/dialog';
	import { Button } from '$lib/components/ui/button';
	import { Pill } from '$lib/components/orch/pill';
	import * as Field from '$lib/components/ui/field';
	import { skillSources } from '$lib/mock';
	import { store } from '$lib/teams.svelte';

	let { open = $bindable(false) }: { open?: boolean } = $props();

	let sourcesDraft = $state<typeof skillSources>();
	// 열 때마다 현재 설정의 사본으로 시작한다.
	$effect.pre(() => {
		if (open) sourcesDraft = structuredClone($state.snapshot(store.sources));
	});
</script>

<!-- 스킬 소스 연동 (.pen Skill Sources Dialog) — 워크스페이스 전체 설정 -->
<Dialog.Root bind:open={() => open, (v) => (open = v)}>
	<Dialog.Content size="md">
		{#if sourcesDraft}
			{@const d = sourcesDraft}
			<Dialog.Header icon={Plug}>
				<Dialog.Title>스킬 소스 연동</Dialog.Title>
				<Dialog.Description>스킬을 찾고 설치할 곳 · 워크스페이스 전체에 적용</Dialog.Description>
			</Dialog.Header>
			<Dialog.Body class="gap-4">
				<section class="flex flex-col gap-2">
					<span class="list-label">소스</span>
					{#each d.sources as src (src.name)}
						{@const Icon = src.kind === 'web' ? Globe : src.kind === 'github' ? Github : Folder}
						<div class="flex items-center rounded-md border gap-3 p-3">
							<Icon class="size-4 shrink-0" />
							<span class="flex min-w-0 flex-1 flex-col gap-0.5">
								<span class="row-title">{src.name}<Pill class={src.state === '연결됨' ? 'bg-success-soft text-status-done' : ''}>{src.state}</Pill></span>
								<span class="text-xs text-muted-foreground">{src.desc}</span>
							</span>
							<span class="text-caption text-subtle-foreground">{src.sync}</span>
							{#if src.kind === 'github'}<Button variant="ghost" size="sm" onclick={() => (src.sync = '방금 동기화')}><RefreshCw />동기화</Button>{/if}
						</div>
					{/each}
				</section>
				<section class="flex items-center gap-2 text-xs">
					<span class="list-label flex-1">설치 도구</span>
					<Terminal class="size-3.5" /><span class="font-mono">{d.tool.name}</span><span class="text-muted-foreground">{d.tool.version}</span>
					<Pill class="bg-success-soft text-status-done"><Check />설치됨</Pill>
				</section>
				<section class="flex flex-col gap-1">
					<span class="list-label pb-1">설치 정책</span>
					{#each d.policy as pol (pol.name)}
						<Field.SwitchRow label={pol.name} hint={pol.desc} bind:checked={pol.on} />
					{/each}
				</section>
			</Dialog.Body>
			<Dialog.Footer note="API 한도 600회/분 · 캐시 15분">
				<Button variant="ghost" size="sm" onclick={() => (open = false)}>취소</Button>
				<Button size="sm" onclick={() => ((store.sources = d), (open = false))}>저장</Button>
			</Dialog.Footer>
		{/if}
	</Dialog.Content>
</Dialog.Root>
