/// 메뉴 항목 데이터 — DropdownMenu.Entries · ContextMenu.Entries가 같은 목록을 그린다.
/// "sep"는 구분선, sub가 있으면 하위 메뉴, shortcut은 오른쪽 단축키 표시.
import type { Component } from "svelte";

export type MenuEntry =
	| "sep"
	| {
			label: string;
			icon?: Component;
			/** 글자 · 아이콘 색 (예: text-destructive · text-primary). 없으면 기본 · 아이콘은 회색 */
			tone?: string;
			shortcut?: string;
			disabled?: boolean;
			onSelect?: () => void;
			sub?: { label: string; icon?: Component; tone?: string; checked?: boolean; shortcut?: string; onSelect: () => void }[];
			/** 하위 메뉴 머리 (예: "연결 종류 · #130 QA"). */
			subLabel?: string;
	  };
