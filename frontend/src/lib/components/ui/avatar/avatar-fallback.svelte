<script lang="ts">
	import { Avatar as AvatarPrimitive } from "bits-ui";
	import { cn } from "$lib/utils.js";

	let {
		ref = $bindable(null),
		class: className,
		...restProps
	}: AvatarPrimitive.FallbackProps = $props();
</script>

<AvatarPrimitive.Fallback
	bind:ref
	data-slot="avatar-fallback"
	class={cn(
		"bg-muted text-muted-foreground rounded-[inherit] flex size-full items-center justify-center text-xs group-data-[size=sm]/avatar:text-2xs [&>svg]:size-4 group-data-[size=sm]/avatar:[&>svg]:size-3 group-data-[size=lg]/avatar:[&>svg]:size-5.5 group-data-[size=lg]/avatar:text-sm",
		className
	)}
	{...restProps}
/>
