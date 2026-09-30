<script lang="ts">
	import * as Empty from '$lib/components/ui/empty';
	import LayoutGrid from '@lucide/svelte/icons/layout-grid';
</script>

<svelte:head><title>All Projects · OrchStack</title></svelte:head>

<Empty.Root class="h-full">
	<Empty.Header>
		<Empty.Media variant="icon"><LayoutGrid /></Empty.Media>
		<Empty.Title>All Projects</Empty.Title>
		<Empty.Description>프로젝트 목록 · 보관 · 삭제는 준비 중이에요.</Empty.Description>
	</Empty.Header>
</Empty.Root>
