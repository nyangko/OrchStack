/// Task 상태 → 라벨·아이콘·색 매핑. StatusSelect·StatusBadge 등이 이 한 곳을 공유한다.
import type { Component } from "svelte";
import CircleDashed from "@lucide/svelte/icons/circle-dashed";
import Circle from "@lucide/svelte/icons/circle";
import CircleDot from "@lucide/svelte/icons/circle-dot";
import Hourglass from "@lucide/svelte/icons/hourglass";
import OctagonX from "@lucide/svelte/icons/octagon-x";
import Eye from "@lucide/svelte/icons/eye";
import CircleCheck from "@lucide/svelte/icons/circle-check";
import CircleSlash from "@lucide/svelte/icons/circle-slash";
import CircleX from "@lucide/svelte/icons/circle-x";

// data/sqlite.sql tbl_task.status 값. waiting은 저장하지 않고 의존 관계로 계산되는 표시 전용 상태.
// 백엔드 OpenAPI 타입(#43)이 나오면 그 union으로 교체한다.
export type TaskStatus =
	| "backlog"
	| "todo"
	| "in_progress"
	| "waiting"
	| "blocked"
	| "review"
	| "done"
	| "failed"
	| "cancelled";

/// 상태 1개의 표시 정보. 클래스는 Tailwind가 찾을 수 있게 전체 이름으로 둔다.
export type StatusMeta = { label: string; icon: Component; text: string; soft: string };

export const statuses: Record<TaskStatus, StatusMeta> = {
	backlog: { label: "Backlog", icon: CircleDashed, text: "text-status-backlog", soft: "bg-muted" },
	todo: { label: "Todo", icon: Circle, text: "text-status-todo", soft: "bg-muted" },
	in_progress: { label: "In Progress", icon: CircleDot, text: "text-status-in-progress", soft: "bg-primary-soft" },
	waiting: { label: "Waiting", icon: Hourglass, text: "text-status-waiting", soft: "bg-warning-soft" },
	blocked: { label: "Blocked", icon: OctagonX, text: "text-status-blocked", soft: "bg-destructive-soft" },
	review: { label: "Review", icon: Eye, text: "text-status-review", soft: "bg-review-soft" },
	done: { label: "Done", icon: CircleCheck, text: "text-status-done", soft: "bg-success-soft" },
	// .pen에 failed 색이 없어 blocked와 같은 위험 색을 쓴다.
	failed: { label: "Failed", icon: CircleX, text: "text-status-blocked", soft: "bg-destructive-soft" },
	cancelled: { label: "Cancelled", icon: CircleSlash, text: "text-status-cancelled", soft: "bg-muted" },
};

/// 메뉴 표시 순서.
export const statusOrder = Object.keys(statuses) as TaskStatus[];
