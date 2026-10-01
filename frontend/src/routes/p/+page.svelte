<script lang="ts">
	import { Empty, EmptyHeader, EmptyMedia, EmptyTitle, EmptyDescription, EmptyContent } from '$lib/components/ui/empty';
	import LayoutGrid from '@lucide/svelte/icons/layout-grid';
	import Plus from '@lucide/svelte/icons/plus';
	import { Button } from '$lib/components/ui/button';
</script>

<svelte:head><title>All Projects · OrchStack</title></svelte:head>

<Empty class="h-full">
	<EmptyHeader>
		<EmptyMedia variant="icon"><LayoutGrid /></EmptyMedia>
		<EmptyTitle>All Projects</EmptyTitle>
		<EmptyDescription>프로젝트 목록 · 보관 · 삭제는 준비 중이에요.</EmptyDescription>
	</EmptyHeader>
	<!-- 새 프로젝트 다이얼로그는 Workbench 셸(p/+layout)에 있다 -->
	<EmptyContent><Button href="/p?new"><Plus />새 프로젝트</Button></EmptyContent>
</Empty>
