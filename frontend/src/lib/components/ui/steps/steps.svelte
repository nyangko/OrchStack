<script lang="ts">
	import Check from "@lucide/svelte/icons/check";
	import { cn } from "$lib/utils.js";

	let {
		steps,
		current = 0,
		class: className,
	}: { steps: string[]; current?: number; class?: string } = $props();
</script>

<ol data-slot="steps" class={cn("flex items-center gap-2", className)}>
	{#each steps as label, i (label)}
		{@const done = i < current}
		{@const active = i === current}
		<li class="flex items-center gap-1.5" aria-current={active ? "step" : undefined}>
			<span
				class={cn(
					"flex size-[22px] items-center justify-center rounded-full text-[11px] font-bold",
					done && "bg-success text-on-solid",
					active && "bg-primary text-primary-foreground",
					!done && !active && "bg-secondary text-muted-foreground"
				)}
			>
				{#if done}<Check class="size-3" strokeWidth={3} />{:else}{i + 1}{/if}
			</span>
			<span
				class={cn(
					"text-[13px]",
					active ? "text-foreground font-semibold" : done ? "text-foreground" : "text-muted-foreground"
				)}>{label}</span
			>
		</li>
		{#if i < steps.length - 1}
			<li aria-hidden="true" class={cn("h-[1.5px] w-10", done ? "bg-primary" : "bg-border")}></li>
		{/if}
	{/each}
</ol>
