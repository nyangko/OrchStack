<script lang="ts">
	/// 메뉴 항목 목록(MenuEntry[])을 그린다 — 구분선 · 하위 메뉴 · 단축키 · 비활성.
	import Check from "@lucide/svelte/icons/check";
	import Item from "./dropdown-menu-item.svelte";
	import Separator from "./dropdown-menu-separator.svelte";
	import Label from "./dropdown-menu-label.svelte";
	import Shortcut from "./dropdown-menu-shortcut.svelte";
	import Sub from "./dropdown-menu-sub.svelte";
	import SubTrigger from "./dropdown-menu-sub-trigger.svelte";
	import SubContent from "./dropdown-menu-sub-content.svelte";
	import type { MenuEntry } from "./menu-entry.js";

	let { entries, subClass = "w-48" }: { entries: MenuEntry[]; subClass?: string } = $props();
</script>

{#each entries as m, i (i)}
	{#if m === "sep"}
		<Separator />
	{:else if m.sub}
		<Sub>
			<SubTrigger disabled={m.disabled}>{#if m.icon}<m.icon class="text-muted-foreground" />{/if}{m.label}</SubTrigger>
			<SubContent class={subClass}>
				{#if m.subLabel}<Label class="truncate">{m.subLabel}</Label><Separator />{/if}
				{#each m.sub as x (x.label)}
					<Item onSelect={x.onSelect}>{#if x.icon}<x.icon class={x.tone} />{/if}<span class="flex-1">{x.label}</span>{#if x.checked}<Check />{/if}{#if x.shortcut}<Shortcut>{x.shortcut}</Shortcut>{/if}</Item>
				{/each}
			</SubContent>
		</Sub>
	{:else}
		<Item disabled={m.disabled} onSelect={m.onSelect} class={m.tone}>
			{#if m.icon}<m.icon class={m.tone ?? "text-muted-foreground"} />{/if}{m.label}
			{#if m.shortcut}<Shortcut>{m.shortcut}</Shortcut>{/if}
		</Item>
	{/if}
{/each}
