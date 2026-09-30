import { createCn } from "cn/config";
import { defaultConfig } from "tailwind-variants";

// app.css의 커스텀 font-size 토큰. 등록하지 않으면 병합기가 색 클래스로 오인해 지운다.
const extend = { classGroups: { "font-size": [{ text: ["2xs", "caption", "body"] }] } };

/// 클래스 병합 (커스텀 토큰 등록).
export const cn = createCn({ extend });

// shadcn 컴포넌트의 tv()도 자체 tailwind-merge로 병합하므로 같은 토큰을 등록한다.
defaultConfig.twMergeConfig = { extend };

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChild<T> = T extends { child?: any } ? Omit<T, "child"> : T;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChildren<T> = T extends { children?: any } ? Omit<T, "children"> : T;
export type WithoutChildrenOrChild<T> = WithoutChildren<WithoutChild<T>>;
export type WithElementRef<T, U extends HTMLElement = HTMLElement> = T & { ref?: U | null };
