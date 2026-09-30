<script lang="ts">
	import { Avatar as AvatarPrimitive } from "bits-ui";
	import { cn } from "$lib/utils.js";

	let {
		ref = $bindable(null),
		loadingStatus = $bindable("loading"),
		size = "default",
		shape = "circle",
		class: className,
		...restProps
	}: AvatarPrimitive.RootProps & {
		size?: "default" | "sm" | "lg";
		/** square = .pen RoleAvatar (에이전트). 크기별 radius-xs · sm · lg. */
		shape?: "circle" | "square";
	} = $props();
</script>

<AvatarPrimitive.Root
	bind:ref
	bind:loadingStatus
	data-slot="avatar"
	data-size={size}
	data-shape={shape}
	class={cn(
		"size-7 rounded-full after:rounded-[inherit] data-[size=lg]:size-10 data-[size=sm]:size-5 data-[shape=square]:rounded-sm data-[shape=square]:data-[size=sm]:rounded-xs data-[shape=square]:data-[size=lg]:rounded-lg data-[shape=square]:after:hidden group/avatar relative flex shrink-0 select-none after:absolute after:inset-0 after:border after:border-border after:mix-blend-darken dark:after:mix-blend-lighten",
		className
	)}
	{...restProps}
/>
