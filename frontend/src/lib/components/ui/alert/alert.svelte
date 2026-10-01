<script lang="ts" module>
	import { type VariantProps, tv } from "tailwind-variants";

	export const alertVariants = tv({
		base: "grid gap-0.5 rounded-lg border px-4 py-3 text-left text-sm has-data-[slot=alert-action]:relative has-data-[slot=alert-action]:pr-18 has-[>svg]:grid-cols-[auto_1fr] has-[>svg]:gap-x-2.5 *:[svg]:row-span-2 *:[svg]:translate-y-0.5 *:[svg]:text-current *:[svg:not([class*='size-'])]:size-4 group/alert relative w-full",
		variants: {
			variant: {
				default: "bg-card text-card-foreground",
				warning: "bg-warning-soft border-warning px-3 text-warning **:data-[slot=alert-title]:text-body **:data-[slot=alert-title]:font-semibold **:data-[slot=alert-description]:text-xs **:data-[slot=alert-description]:text-foreground",
				// .pen TaskEditor · Orch Draft: 파란 띠 · 테두리 없음 · 12px (오른쪽 버튼은 Alert.Action)
				primary: "bg-primary-soft border-transparent px-3 py-2.5 items-center **:data-[slot=alert-description]:text-xs **:data-[slot=alert-description]:text-foreground *:[svg]:size-3.75 *:[svg]:text-primary has-data-[slot=alert-action]:pr-28",
				// .pen InfoStrip: 회색 띠 · 12px
				info: "bg-muted border-border px-3.5 py-2 **:data-[slot=alert-title]:text-xs **:data-[slot=alert-title]:font-semibold **:data-[slot=alert-description]:text-xs *:[svg]:size-3.5",
				// .pen Quota Alert: 붉은 띠 · 제목은 본문색
				destructive: "bg-destructive-soft border-destructive px-3.5 py-2.5 text-destructive **:data-[slot=alert-title]:text-body **:data-[slot=alert-title]:font-semibold **:data-[slot=alert-title]:text-foreground **:data-[slot=alert-description]:text-xs **:data-[slot=alert-description]:text-muted-foreground *:data-[slot=button]:text-foreground",
			},
		},
		defaultVariants: {
			variant: "default",
		},
	});

	export type AlertVariant = VariantProps<typeof alertVariants>["variant"];
</script>

<script lang="ts">
	import { cn, type WithElementRef } from "$lib/utils.js";
	import type { HTMLAttributes } from "svelte/elements";

	let {
		ref = $bindable(null),
		class: className,
		variant = "default",
		children,
		...restProps
	}: WithElementRef<HTMLAttributes<HTMLDivElement>> & {
		variant?: AlertVariant;
	} = $props();
</script>

<div
	bind:this={ref}
	data-slot="alert"
	role="alert"
	class={cn(alertVariants({ variant }), className)}
	{...restProps}
>
	{@render children?.()}
</div>
