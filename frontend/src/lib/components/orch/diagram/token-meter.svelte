<script lang="ts" module>
	/// 토큰 K 표기 — 18.4 → "18.4K", 22 → "22.0K", 0 → "0".
	export const tokK = (n: number) => (n ? `${n.toFixed(1)}K` : "0");
</script>

<script lang="ts">
	/// 토큰 표시 (.pen TokenMeter). 토큰은 Run별로 저장하고 화면에서만 합친다.
	/// runner 합계를 주면 lead(자기 + runner = 합계), included면 sub · fork(리드 합계에 포함), 아니면 자기 사용량만.
	import Coins from "@lucide/svelte/icons/coins";
	import { cn } from "$lib/utils";

	let {
		self = 0,
		runner,
		included = false,
		class: className,
	}: {
		/** 이 Run 자기 사용량 (K). */
		self?: number;
		/** 하위 runner Run 합계 (K) — 리드일 때만. */
		runner?: number;
		/** sub · fork — 리드 Run 안에서 돌아 따로 세지 않는다. */
		included?: boolean;
		class?: string;
	} = $props();
</script>

<span class={cn("flex items-center gap-1 text-caption text-muted-foreground", className)}>
	<Coins class="size-3 shrink-0" />
	{#if included}
		리드 합계에 포함
	{:else}
		<span class="font-mono font-medium">{tokK(self)}</span>
		{#if runner !== undefined}
			<span class="font-mono">+ runner {tokK(runner)}</span>
			<span class="font-mono font-semibold text-foreground">= {tokK(self + runner)}</span>
		{/if}
	{/if}
</span>
