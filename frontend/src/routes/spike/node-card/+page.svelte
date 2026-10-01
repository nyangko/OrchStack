<script lang="ts">
	/// ui/node-card · 메뉴 렌더러 스파이크 — 태스크와 무관한 데이터(배포 단계).
	import Rocket from "@lucide/svelte/icons/rocket";
	import Ellipsis from "@lucide/svelte/icons/ellipsis";
	import Play from "@lucide/svelte/icons/play";
	import * as NodeCard from "$lib/components/ui/node-card/index.js";
	import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
	import { Button } from "$lib/components/ui/button/index.js";
	import { Badge } from "$lib/components/ui/badge/index.js";
	import type { MenuEntry } from "$lib/components/ui/dropdown-menu/index.js";

	let last = $state("");
	let env = $state("staging");
	const entries: MenuEntry[] = [
		{ label: "실행", icon: Play, shortcut: "↵", onSelect: () => (last = "run") },
		"sep",
		{ label: "환경", sub: ["staging", "prod"].map((e) => ({ label: e, checked: env === e, onSelect: () => ((env = e), (last = e)) })) },
		{ label: "삭제", disabled: true }
	];
</script>

<main class="flex gap-4 p-8">
	<NodeCard.Root selected>
		<NodeCard.Header>
			<NodeCard.Kind class="bg-primary"><Rocket /></NodeCard.Kind>
			<NodeCard.Ref>DEPLOY · #12</NodeCard.Ref>
			<DropdownMenu.Root>
				<DropdownMenu.Trigger>{#snippet child({ props })}<Button {...props} variant="ghost" size="icon-xs" aria-label="배포 메뉴"><Ellipsis /></Button>{/snippet}</DropdownMenu.Trigger>
				<DropdownMenu.Content class="w-48"><DropdownMenu.Entries {entries} /></DropdownMenu.Content>
			</DropdownMenu.Root>
		</NodeCard.Header>
		<NodeCard.Title>웹 앱 배포</NodeCard.Title>
		<NodeCard.Footer><Badge variant="secondary">{env}</Badge><span class="mono-meta">3m</span></NodeCard.Footer>
	</NodeCard.Root>
	<NodeCard.Root dashed><NodeCard.Title>점선 노드</NodeCard.Title></NodeCard.Root>
	<p data-testid="out">{last}</p>
</main>
