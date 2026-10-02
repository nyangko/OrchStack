<script lang="ts">
	/// Diagram 캔버스 도구 (.pen Workbench/ViewHeader · Canvas Tools) — 자동 정렬 · 격자 정렬 · 확대/축소 · 화면 맞춤.
	/// SvelteFlowProvider 안에서만 쓴다 (뷰 머리글은 캔버스 밖이라 Provider가 둘을 묶는다).
	import { useSvelteFlow, useViewport } from "@xyflow/svelte";
	import Sparkles from "@lucide/svelte/icons/sparkles";
	import LayoutGrid from "@lucide/svelte/icons/layout-grid";
	import Minus from "@lucide/svelte/icons/minus";
	import Plus from "@lucide/svelte/icons/plus";
	import Maximize from "@lucide/svelte/icons/maximize";
	import { Button } from "$lib/components/ui/button";
	import { Separator } from "$lib/components/ui/separator";

	let { grid = false, onauto, ongrid }: { grid?: boolean; onauto: () => void; ongrid: () => void } = $props();
	const flow = useSvelteFlow();
	const viewport = useViewport();

	/// 화면 맞춤 — 캔버스 우클릭 메뉴(Fit View)도 같은 동작을 부른다.
	export function fit() {
		void flow.fitView({ padding: 0.08, duration: 200 });
	}
	/// 1.2배씩 확대 · 축소 (캔버스 최소 0.3 · 최대 2). zoomIn · zoomOut(d3 scaleBy)은 이 화면에서 움직이지 않아 배율을 직접 정한다.
	function zoom(by: number) {
		void flow.setZoom(Math.min(2, Math.max(0.3, viewport.current.zoom * by)), { duration: 150 });
	}
	/// 노드 배치를 바꾼 뒤 그려지면 화면을 맞춘다.
	export function auto() {
		onauto();
		requestAnimationFrame(fit);
	}
</script>

<div class="flex items-center gap-1" role="toolbar" aria-label="캔버스 도구">
	<Button variant="ghost" size="sm" onclick={auto}><Sparkles />Auto Layout</Button>
	<Button variant="ghost" size="sm" aria-pressed={grid} class={grid ? "bg-accent text-accent-foreground" : undefined} title="노드를 20px 격자에 맞추고, 옮길 때도 격자에 붙여요" onclick={ongrid}><LayoutGrid />Grid Layout</Button>
	<Separator orientation="vertical" class="mx-1 h-5" />
	<div class="flex items-center rounded-md border">
		<Button variant="ghost" size="icon-sm" class="size-7.5" aria-label="축소" onclick={() => zoom(1 / 1.2)}><Minus /></Button>
		<button type="button" class="w-10 text-center font-mono text-xs tabular-nums hover:text-primary" title="100%로" onclick={() => void flow.setZoom(1, { duration: 150 })}>{Math.round(viewport.current.zoom * 100)}%</button>
		<Button variant="ghost" size="icon-sm" class="size-7.5" aria-label="확대" onclick={() => zoom(1.2)}><Plus /></Button>
	</div>
	<Button variant="ghost" size="icon-sm" aria-label="화면 맞춤" title="화면 맞춤" onclick={fit}><Maximize /></Button>
</div>
