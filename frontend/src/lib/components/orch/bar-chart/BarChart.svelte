<script lang="ts" module>
	/** 계열 하나 — 범례 이름 · 막대 색(bg-* 클래스). 아래부터 쌓는다. */
	export type BarSeries = { label: string; class: string };
	/** 열 하나 — 아래 축 이름 · 계열별 값(series 순서) · 툴팁 제목 · 한 줄 메모. */
	export type BarColumn = { label: string; values: number[]; title?: string; note?: string };
</script>

<script lang="ts">
	/// 쌓인 막대 차트 (.pen BarChart · orch/display/bar-chart). 마우스 · ←→로 열을 고르면 그 열만 진하게 · 툴팁(제목 · 계열별 값 · 합계 · 메모).
	import type { HTMLAttributes } from "svelte/elements";
	import { cn } from "$lib/utils.js";

	let {
		series,
		columns,
		total,
		format = (v: number) => String(v),
		class: className,
		...restProps
	}: HTMLAttributes<HTMLDivElement> & {
		series: BarSeries[];
		columns: BarColumn[];
		/** 범례 오른쪽 요약 (예: "7일 · 31 runs"). */
		total?: string;
		/** 툴팁 값 표기 (예: 달러). */
		format?: (v: number) => string;
	} = $props();

	const max = $derived(Math.max(1, ...columns.map((c) => c.values.reduce((a, b) => a + b, 0))));
	let active = $state<number>();
	const col = $derived(active === undefined ? undefined : columns[active]);
	/// 툴팁을 열 오른쪽에, 오른쪽 끝 두 열은 왼쪽에 둔다.
	const flip = $derived(active !== undefined && active >= columns.length - 2);

	function onkeydown(e: KeyboardEvent) {
		if (e.key !== "ArrowRight" && e.key !== "ArrowLeft") return;
		e.preventDefault();
		const step = e.key === "ArrowRight" ? 1 : -1;
		active = Math.min(columns.length - 1, Math.max(0, (active ?? (step > 0 ? -1 : columns.length)) + step));
	}
</script>

<div data-slot="bar-chart" class={cn("flex flex-col gap-2.5", className)} {...restProps}>
	<div class="meta-line gap-3.5">
		{#each series as s (s.label)}<span class="flex items-center gap-1.5"><span class={["size-2 rounded-xs", s.class]}></span>{s.label}</span>{/each}
		<span class="flex-1"></span>
		{#if total}<span>{total}</span>{/if}
	</div>
	<!-- 열 고르기 = 슬라이더(←→). 화면 읽기는 고른 열의 값을 읽는다. 벗어나면 툴팁을 닫는다 -->
	<div
		class="relative flex h-25 gap-2 rounded-sm border-b outline-none focus-visible:ring-3 focus-visible:ring-ring/50"
		role="slider"
		aria-label="{series.map((s) => s.label).join(' · ')} 차트 · ←→로 값 보기"
		aria-valuemin={0}
		aria-valuemax={columns.length - 1}
		aria-valuenow={active ?? 0}
		aria-valuetext={col ? `${col.title ?? col.label} · ${series.map((s, k) => `${s.label} ${format(col.values[k] ?? 0)}`).join(' · ')}` : total}
		tabindex="0"
		{onkeydown}
		onmouseleave={() => (active = undefined)}
		onblur={() => (active = undefined)}
	>
		{#each columns as c, i (i)}
			<div
				class={["flex flex-1 flex-col-reverse items-center gap-0.5 rounded-sm transition-opacity", active === i && "bg-muted", active !== undefined && active !== i && "opacity-35"]}
				onmouseenter={() => (active = i)}
				role="presentation"
			>
				{#each c.values as v, k (k)}
					{#if v}<span class={["w-4", series[k]?.class, (k === c.values.length - 1 || !c.values.slice(k + 1).some(Boolean)) && "rounded-t-sm"]} style="height: {(v / max) * 88}px"></span>{/if}
				{/each}
			</div>
		{/each}
		{#if col && active !== undefined}
			<!-- 툴팁 (.pen Tooltip · hover) — 위치는 열 번호로 계산한 런타임 값 -->
			<div
				class="pointer-events-none absolute top-0 z-10 flex w-48 flex-col gap-1.5 rounded-md border bg-popover px-3 py-2.5 text-caption shadow-md"
				style={flip ? `right: ${((columns.length - active) / columns.length) * 100}%` : `left: ${((active + 1) / columns.length) * 100}%`}
				role="status"
			>
				<span class="text-xs font-semibold">{col.title ?? col.label}</span>
				{#each series as s, k (s.label)}
					<span class="flex items-center gap-1.5"><span class={["size-2 rounded-xs", s.class]}></span><span class="flex-1 text-muted-foreground">{s.label}</span><span class="font-mono font-semibold">{format(col.values[k] ?? 0)}</span></span>
				{/each}
				{#if series.length > 1}
					<span class="flex border-t pt-1.5 font-semibold"><span class="flex-1">합계</span><span class="font-mono">{format(col.values.reduce((a, b) => a + b, 0))}</span></span>
				{/if}
				{#if col.note}<span class="text-2xs text-muted-foreground">{col.note}</span>{/if}
			</div>
		{/if}
	</div>
	<div class="flex gap-2 text-center text-caption text-muted-foreground">
		{#each columns as c, i (i)}<span class={["flex-1", active === i && "font-semibold text-foreground"]}>{c.label}</span>{/each}
	</div>
</div>
