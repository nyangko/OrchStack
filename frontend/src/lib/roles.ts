/// 에이전트 역할 → 아이콘·색 매핑. RoleAvatar 등이 이 한 곳을 공유한다.
import type { Component } from "svelte";
import Server from "@lucide/svelte/icons/server";
import Monitor from "@lucide/svelte/icons/monitor";
import FlaskConical from "@lucide/svelte/icons/flask-conical";
import ShieldCheck from "@lucide/svelte/icons/shield-check";
import Palette from "@lucide/svelte/icons/palette";
import Bot from "@lucide/svelte/icons/bot";

export type Role = "backend" | "frontend" | "qa" | "reviewer" | "designer" | "agent";

/// 역할 1개의 표시 정보. bg는 Tailwind가 찾을 수 있게 전체 클래스 이름으로 둔다.
export type RoleMeta = { label: string; icon: Component; bg: string };

// designer 아이콘은 .pen에 사용례가 없어 palette로 둔다. agent는 역할 미지정 에이전트.
export const roles: Record<Role, RoleMeta> = {
	backend: { label: "Backend", icon: Server, bg: "bg-role-backend" },
	frontend: { label: "Frontend", icon: Monitor, bg: "bg-role-frontend" },
	qa: { label: "QA", icon: FlaskConical, bg: "bg-role-qa" },
	reviewer: { label: "Reviewer", icon: ShieldCheck, bg: "bg-role-reviewer" },
	designer: { label: "Designer", icon: Palette, bg: "bg-role-designer" },
	agent: { label: "Agent", icon: Bot, bg: "bg-node-agent" },
};
