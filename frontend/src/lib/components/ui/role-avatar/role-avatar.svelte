<script lang="ts">
	/// 에이전트 아바타 (.pen RoleAvatar/sm·md·lg). shadcn Avatar의 square 모양 + 역할 색·아이콘.
	import * as Avatar from "$lib/components/ui/avatar/index.js";
	import { roles, type Role } from "$lib/roles.js";
	import { cn } from "$lib/utils.js";
	import type { Component, Snippet } from "svelte";

	let {
		role,
		size = "default",
		icon,
		class: className,
		children,
	}: {
		role: Role;
		size?: "sm" | "default" | "lg";
		/** 역할 기본 아이콘 대신 쓸 아이콘 (멤버가 고른 아바타). 색은 역할 색 그대로. */
		icon?: Component;
		class?: string;
		/** 아바타 위에 겹칠 요소 (예: 접속 상태 Avatar.Badge). */
		children?: Snippet;
	} = $props();

	const meta = $derived(roles[role]);
	const Glyph = $derived(icon ?? meta.icon);
</script>

<Avatar.Root shape="square" {size} class={className} aria-label={meta.label}>
	<Avatar.Fallback class={cn(meta.bg, "text-on-solid")}>
		<Glyph aria-hidden="true" />
	</Avatar.Fallback>
	{@render children?.()}
</Avatar.Root>
