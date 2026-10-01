/// Task 우선순위 → 라벨 · 아이콘 · 색 (.pen Priority Menu). Kanban 카드 · Task 상세 · Task Editor · QuickAdd가 같이 쓴다.
import type { Component } from "svelte";
import Signal from "@lucide/svelte/icons/signal";
import SignalHigh from "@lucide/svelte/icons/signal-high";
import SignalMedium from "@lucide/svelte/icons/signal-medium";
import SignalLow from "@lucide/svelte/icons/signal-low";

export type Priority = "P0" | "P1" | "P2" | "P3";
export type PriorityMeta = { label: string; icon: Component; text: string };

export const priorities: Record<Priority, PriorityMeta> = {
	P0: { label: "긴급", icon: Signal, text: "text-destructive" },
	P1: { label: "높음", icon: SignalHigh, text: "text-destructive" },
	P2: { label: "보통", icon: SignalMedium, text: "text-status-waiting" },
	P3: { label: "낮음", icon: SignalLow, text: "text-muted-foreground" },
};

export const priorityOrder = Object.keys(priorities) as Priority[];
