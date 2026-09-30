<script lang="ts">
	import * as Empty from '$lib/components/ui/empty';
	import LayoutGrid from '@lucide/svelte/icons/layout-grid';
	import Plus from '@lucide/svelte/icons/plus';
	import { Button } from '$lib/components/ui/button';
</script>

<svelte:head><title>All Projects · OrchStack</title></svelte:head>

<Empty.Root class="h-full">
	<Empty.Header>
		<Empty.Media variant="icon"><LayoutGrid /></Empty.Media>
		<Empty.Title>All Projects</Empty.Title>
		<Empty.Description>프로젝트 목록 · 보관 · 삭제는 준비 중이에요.</Empty.Description>
	</Empty.Header>
	<!-- 새 프로젝트 다이얼로그는 Workbench 셸(p/+layout)에 있다 -->
	<Empty.Content><Button href="/p?new"><Plus />새 프로젝트</Button></Empty.Content>
</Empty.Root>
