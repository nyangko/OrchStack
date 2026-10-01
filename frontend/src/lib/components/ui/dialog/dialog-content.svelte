<script lang="ts" module>
	/** 다이얼로그 너비 (.pen): sm 448 · md 672 · lg 800 · xl 1024 · full 1120 */
	export type DialogSize = "sm" | "md" | "lg" | "xl" | "full";
	export const dialogSizes: Record<DialogSize, string> = {
		sm: "sm:max-w-md",
		md: "sm:max-w-2xl",
		lg: "sm:max-w-200",
		xl: "sm:max-w-5xl",
		full: "sm:max-w-280",
	};
</script>

<script lang="ts">
	/// 다이얼로그 본체 — Header · Body · Footer를 세로로 쌓는다. 닫기 버튼은 Header가 그린다.
	/// tall: 화면 높이 4/5로 고정하고 Body만 스크롤한다 (목록 · 다단계 화면).
	import { Dialog as DialogPrimitive } from "bits-ui";
	import { cn, type WithoutChildrenOrChild } from "$lib/utils.js";
	import DialogPortal from "./dialog-portal.svelte";
	import DialogOverlay from "./dialog-overlay.svelte";
	import type { Snippet } from "svelte";
	import type { ComponentProps } from "svelte";

	let {
		ref = $bindable(null),
		class: className,
		portalProps,
		children,
		size = "sm",
		tall = false,
		...restProps
	}: WithoutChildrenOrChild<DialogPrimitive.ContentProps> & {
		portalProps?: WithoutChildrenOrChild<ComponentProps<typeof DialogPortal>>;
		children: Snippet;
		size?: DialogSize;
		tall?: boolean;
	} = $props();
</script>

<DialogPortal {...portalProps}>
	<DialogOverlay />
	<DialogPrimitive.Content
		bind:ref
		data-slot="dialog-content"
		class={cn(
			"bg-popover text-popover-foreground data-open:animate-in data-closed:animate-out data-closed:fade-out-0 data-open:fade-in-0 data-closed:zoom-out-95 data-open:zoom-in-95 ring-foreground/10 fixed top-1/2 left-1/2 z-50 flex w-full max-w-[calc(100%_-_2rem)] -translate-x-1/2 -translate-y-1/2 flex-col gap-0 overflow-hidden rounded-xl p-0 text-sm ring-1 duration-100 outline-none",
			dialogSizes[size],
			tall && "h-4/5",
			className
		)}
		{...restProps}
	>
		{@render children?.()}
	</DialogPrimitive.Content>
</DialogPortal>
