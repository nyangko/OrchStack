<script lang="ts">
	import { cn, type WithElementRef } from "$lib/utils.js";
	import type { HTMLAttributes } from "svelte/elements";
	import type { Snippet } from "svelte";

	let {
		ref = $bindable(null),
		class: className,
		title,
		description,
		action,
		children,
		...restProps
	}: WithElementRef<Omit<HTMLAttributes<HTMLElement>, "title">> & {
		title: string;
		description?: string;
		action?: Snippet;
	} = $props();
</script>

<section
	bind:this={ref}
	data-slot="section-card"
	class={cn("bg-card flex flex-col gap-3 rounded-lg border px-[18px] py-4", className)}
	{...restProps}
>
	<header class="flex items-center gap-2">
		<div class="flex min-w-0 flex-1 flex-col gap-0.5">
			<h3 class="text-sm font-semibold">{title}</h3>
			{#if description}<p class="text-muted-foreground text-xs">{description}</p>{/if}
		</div>
		{#if action}<div class="text-primary text-xs font-medium">{@render action()}</div>{/if}
	</header>
	<div class="flex flex-col gap-2.5">{@render children?.()}</div>
</section>
