<script lang="ts">
	/// 에이전트 아바타 (.pen RoleAvatar/sm·md·lg). shadcn Avatar의 square 모양 + 역할 색·아이콘.
	import * as Avatar from "$lib/components/ui/avatar/index.js";
	import { roles, type Role } from "$lib/roles.js";
	import { cn } from "$lib/utils.js";

	let {
		role,
		size = "default",
		class: className,
	}: {
		role: Role;
		size?: "sm" | "default" | "lg";
		class?: string;
	} = $props();

	const meta = $derived(roles[role]);
</script>

<Avatar.Root shape="square" {size} class={className} aria-label={meta.label}>
	<Avatar.Fallback class={cn(meta.bg, "text-on-solid")}>
		<meta.icon aria-hidden="true" />
	</Avatar.Fallback>
</Avatar.Root>
