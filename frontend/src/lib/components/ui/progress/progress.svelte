<script lang="ts">
	/// 진행 막대 (.pen Progress). 마우스를 올리거나 포커스하면 툴팁 — tip(1줄 = 무엇, 2줄 = 값 / 한도 · 기준), 없으면 'aria-label · 몇 %'.
	/// 버튼 · 링크 안에 놓일 때는 focusable={false} (포커스 겹침 방지 · 툴팁은 마우스로만).
	import { Progress as ProgressPrimitive } from "bits-ui";
	import { TooltipProvider, Tooltip, TooltipTrigger, TooltipContent } from "$lib/components/ui/tooltip";
	import { cn, type WithoutChildrenOrChild } from "$lib/utils.js";

	let {
		ref = $bindable(null),
		class: className,
		max = 100,
		value,
		indicator,
		tip,
		focusable = true,
		...restProps
	}: WithoutChildrenOrChild<ProgressPrimitive.RootProps> & {
		/** 막대 색 (bg-* 클래스). 잔량 · 경고처럼 값에 따라 색이 바뀔 때 쓴다. */
		indicator?: string;
		/** 툴팁 — 줄바꿈(\n) 앞은 제목, 뒤는 값 · 기준. */
		tip?: string;
		/** Tab으로 포커스할 수 있는지 (기본 true). */
		focusable?: boolean;
	} = $props();

	const pct = $derived(Math.round((100 * (value ?? 0)) / (max || 1)));
	const label = $derived(restProps["aria-label"] as string | undefined);
	const lines = $derived((tip ?? (label ? `${label}\n${pct}%` : `${pct}%`)).split("\n"));
</script>

<TooltipProvider delayDuration={120}>
	<Tooltip>
		<TooltipTrigger>
			{#snippet child({ props })}
				<ProgressPrimitive.Root
					{...props}
					bind:ref
					data-slot="progress"
					class={cn("bg-primary-soft h-2 rounded-full relative flex w-full items-center overflow-x-hidden outline-none focus-visible:ring-2 focus-visible:ring-ring/60", className)}
					{value}
					{max}
					aria-valuetext={lines.join(" · ")}
					tabindex={focusable ? 0 : -1}
					{...restProps}
				>
					<div
						data-slot="progress-indicator"
						class={cn("bg-primary size-full flex-1 transition-all", indicator)}
						style="transform: translateX(-{100 - pct}%)"
					></div>
				</ProgressPrimitive.Root>
			{/snippet}
		</TooltipTrigger>
		<TooltipContent class="flex-col items-start gap-0.5">
			<span class="font-semibold">{lines[0]}</span>
			{#each lines.slice(1) as l, i (i)}<span>{l}</span>{/each}
		</TooltipContent>
	</Tooltip>
</TooltipProvider>
