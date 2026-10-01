<script lang="ts" module>
	export type Side = "top" | "right" | "bottom" | "left";
	/** 시트 너비 (.pen): sm 384 · md 640 · lg 800 · xl 1040 */
	export type SheetSize = "sm" | "md" | "lg" | "xl";
	export const sheetSizes: Record<SheetSize, string> = {
		sm: "data-[side=left]:sm:max-w-sm data-[side=right]:sm:max-w-sm",
		md: "data-[side=left]:sm:max-w-160 data-[side=right]:sm:max-w-160",
		lg: "data-[side=left]:sm:max-w-200 data-[side=right]:sm:max-w-200",
		xl: "data-[side=left]:sm:max-w-260 data-[side=right]:sm:max-w-260",
	};
</script>

<script lang="ts">
	/// 시트 본체 — Header · Body · Footer를 세로로 쌓는다. 닫기 버튼은 Header가 그린다.
	import { Dialog as SheetPrimitive } from "bits-ui";
	import { cn, type WithoutChildrenOrChild } from "$lib/utils.js";
	import SheetOverlay from "./sheet-overlay.svelte";
	import SheetPortal from "./sheet-portal.svelte";
	import type { Snippet } from "svelte";
	import type { ComponentProps } from "svelte";

	let {
		ref = $bindable(null),
		class: className,
		side = "right",
		size = "sm",
		portalProps,
		children,
		...restProps
	}: WithoutChildrenOrChild<SheetPrimitive.ContentProps> & {
		portalProps?: WithoutChildrenOrChild<ComponentProps<typeof SheetPortal>>;
		side?: Side;
		size?: SheetSize;
		children: Snippet;
	} = $props();
</script>

<SheetPortal {...portalProps}>
	<SheetOverlay />
	<SheetPrimitive.Content
		bind:ref
		data-slot="sheet-content"
		data-side={side}
		class={cn(
			"bg-popover text-popover-foreground fixed z-50 flex flex-col gap-0 bg-clip-padding p-0 text-sm shadow-lg transition duration-200 ease-in-out data-[side=bottom]:inset-x-0 data-[side=bottom]:bottom-0 data-[side=bottom]:h-auto data-[side=bottom]:border-t data-[side=left]:inset-y-0 data-[side=left]:left-0 data-[side=left]:h-full data-[side=left]:w-3/4 data-[side=left]:border-r data-[side=right]:inset-y-0 data-[side=right]:right-0 data-[side=right]:h-full data-[side=right]:w-3/4 data-[side=right]:border-l data-[side=top]:inset-x-0 data-[side=top]:top-0 data-[side=top]:h-auto data-[side=top]:border-b data-open:animate-in data-open:fade-in-0 data-[side=bottom]:data-open:slide-in-from-bottom-10 data-[side=left]:data-open:slide-in-from-left-10 data-[side=right]:data-open:slide-in-from-right-10 data-[side=top]:data-open:slide-in-from-top-10 data-closed:animate-out data-closed:fade-out-0 data-[side=bottom]:data-closed:slide-out-to-bottom-10 data-[side=left]:data-closed:slide-out-to-left-10 data-[side=right]:data-closed:slide-out-to-right-10 data-[side=top]:data-closed:slide-out-to-top-10",
			sheetSizes[size],
			className
		)}
		{...restProps}
	>
		{@render children?.()}
	</SheetPrimitive.Content>
</SheetPortal>
