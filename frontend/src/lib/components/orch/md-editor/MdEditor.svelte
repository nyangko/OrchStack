<script lang="ts" module>
	export type MdFile = { name: string; body: string };

	/// 토큰 어림값 (한글 · 영문 섞인 지침 기준 약 2.5자 = 1토큰). 정확한 값은 서버(TokenLedger)가 준다.
	export const estimateTokens = (text: string) => Math.round(text.length / 2.5);

	export type DiffRow = { left?: { n: number; text: string }; right?: { n: number; text: string }; kind: "same" | "del" | "add" | "change" };

	/// 줄 단위 비교 (LCS). 지워진 줄과 추가된 줄이 이어지면 한 행에 나란히 둔다.
	export function lineDiff(a: string, b: string): DiffRow[] {
		const x = a.split("\n");
		const y = b.split("\n");
		const lcs = Array.from({ length: x.length + 1 }, () => new Array<number>(y.length + 1).fill(0));
		for (let i = x.length - 1; i >= 0; i--)
			for (let j = y.length - 1; j >= 0; j--) lcs[i][j] = x[i] === y[j] ? lcs[i + 1][j + 1] + 1 : Math.max(lcs[i + 1][j], lcs[i][j + 1]);
		// 뒤에서부터 LCS를 따라가며 같음(=) · 삭제(-) · 추가(+)를 만든다.
		const ops: ["=" | "-" | "+", number, number][] = [];
		let i = 0;
		let j = 0;
		while (i < x.length && j < y.length) {
			if (x[i] === y[j]) ops.push(["=", i++, j++]);
			else if (lcs[i + 1][j] >= lcs[i][j + 1]) ops.push(["-", i++, j]);
			else ops.push(["+", i, j++]);
		}
		while (i < x.length) ops.push(["-", i++, j]);
		while (j < y.length) ops.push(["+", i, j++]);
		const rows: DiffRow[] = [];
		for (let k = 0; k < ops.length; ) {
			const [op, a, b] = ops[k];
			if (op === "=") {
				rows.push({ kind: "same", left: { n: a + 1, text: x[a] }, right: { n: b + 1, text: y[b] } });
				k++;
				continue;
			}
			const dels: NonNullable<DiffRow["left"]>[] = [];
			const adds: NonNullable<DiffRow["right"]>[] = [];
			for (; k < ops.length && ops[k][0] !== "="; k++) {
				const [o, p, q] = ops[k];
				if (o === "-") dels.push({ n: p + 1, text: x[p] });
				else adds.push({ n: q + 1, text: y[q] });
			}
			for (let t = 0; t < Math.max(dels.length, adds.length); t++)
				rows.push({ left: dels[t], right: adds[t], kind: dels[t] && adds[t] ? "change" : dels[t] ? "del" : "add" });
		}
		return rows;
	}

	export type MdFormat = "h" | "b" | "i" | "ul" | "task" | "code" | "link" | "quote" | "var" | "include" | "mention";

	/// 선택 영역(s~e)을 감싸거나(굵게 등) 줄 앞에 붙인다(제목 · 목록 등). 바뀐 글과 커서 위치를 돌려준다.
	/// MdEditor 툴바와 Task Editor 설명 툴바가 같이 쓴다.
	export function applyMd(value: string, s: number, e: number, kind: MdFormat): { next: string; cursor: number } {
		const sel = value.slice(s, e);
		const wrap: Partial<Record<MdFormat, [string, string]>> = { b: ["**", "**"], i: ["*", "*"], code: ["`", "`"], link: ["[", "](url)"], var: ["{{", "}}"], mention: ["@", ""] };
		const prefix: Partial<Record<MdFormat, string>> = { h: "## ", ul: "- ", task: "- [ ] ", quote: "> ", include: "@include " };
		const w = wrap[kind];
		if (w) {
			const [l, r] = w;
			const inner = sel || (kind === "var" ? "team.name" : "");
			return { next: value.slice(0, s) + l + inner + r + value.slice(e), cursor: s + l.length + inner.length };
		}
		const lineStart = value.lastIndexOf("\n", s - 1) + 1;
		const pre = prefix[kind] ?? "";
		return { next: value.slice(0, lineStart) + pre + value.slice(lineStart), cursor: e + pre.length };
	}
</script>

<script lang="ts">
	/// 마크다운 지침 편집기 (.pen MdEditor). 파일 탭 · Read / Edit / 비교 · 서식 툴바 · 줄 번호 · 상태바.
	/// base를 주면 파일별 변경 표시와 비교 모드가 생긴다 (템플릿 대비 · 버전 대비).
	import FileText from "@lucide/svelte/icons/file-text";
	import Heart from "@lucide/svelte/icons/heart";
	import Heading from "@lucide/svelte/icons/heading";
	import Bold from "@lucide/svelte/icons/bold";
	import Italic from "@lucide/svelte/icons/italic";
	import List from "@lucide/svelte/icons/list";
	import ListChecks from "@lucide/svelte/icons/list-checks";
	import Code from "@lucide/svelte/icons/code";
	import Link from "@lucide/svelte/icons/link";
	import Quote from "@lucide/svelte/icons/quote";
	import Braces from "@lucide/svelte/icons/braces";
	import FileInput from "@lucide/svelte/icons/file-input";
	import X from "@lucide/svelte/icons/x";
	import type { Snippet } from "svelte";
	import { tick } from "svelte";
	import { cn } from "$lib/utils.js";

	type Mode = "read" | "edit" | "diff";

	let {
		files = $bindable(),
		active = $bindable(0),
		mode = $bindable("edit"),
		base,
		baseLabel = "기준",
		currentLabel = "현재",
		readonly = false,
		tabs = true,
		closable = false,
		onclose,
		status,
		class: className,
	}: {
		files: MdFile[];
		active?: number;
		mode?: Mode;
		/** 파일 이름 → 비교 기준 본문. 있으면 변경 점 · 비교 모드가 켜진다. */
		base?: Record<string, string>;
		baseLabel?: string;
		currentLabel?: string;
		/** 읽기 전용 (Edit 모드 · 툴바 숨김). */
		readonly?: boolean;
		/** 파일 탭 표시. 바깥에서 파일을 고를 때는 끈다. */
		tabs?: boolean;
		/** 탭 닫기 버튼 (파일 트리와 같이 쓸 때). */
		closable?: boolean;
		onclose?: (index: number) => void;
		/** 상태바 오른쪽 문구 (예: "초안 · 추가 시 저장"). */
		status?: Snippet;
		class?: string;
	} = $props();

	const file = $derived(files[active]);
	const orig = $derived(file && base ? base[file.name] : undefined);
	const changed = (f: MdFile) => base !== undefined && base[f.name] !== undefined && base[f.name] !== f.body;
	const rows = $derived(file && orig !== undefined ? lineDiff(orig, file.body) : []);
	const plus = $derived(rows.filter((r) => r.kind !== "same" && r.right).length);
	const minus = $derived(rows.filter((r) => r.kind !== "same" && r.left).length);
	const modes = $derived<{ v: Mode; label: string }[]>([
		{ v: "read", label: "Read" },
		...(readonly ? [] : [{ v: "edit" as Mode, label: "Edit" }]),
		...(orig !== undefined ? [{ v: "diff" as Mode, label: `${baseLabel}와 비교` }] : []),
	]);
	$effect.pre(() => {
		if (!modes.some((m) => m.v === mode)) mode = modes[0].v;
	});

	let area = $state<HTMLTextAreaElement>();
	let caret = $state({ ln: 1, col: 1 });
	function track() {
		if (!area) return;
		const before = area.value.slice(0, area.selectionStart).split("\n");
		caret = { ln: before.length, col: before[before.length - 1].length + 1 };
	}

	/// 툴바 서식 적용 (applyMd).
	async function format(kind: MdFormat) {
		if (!area || !file) return;
		const { next, cursor } = applyMd(area.value, area.selectionStart, area.selectionEnd, kind);
		file.body = next;
		await tick();
		area.focus();
		area.setSelectionRange(cursor, cursor);
		track();
	}

	const tools = [
		{ k: "h", icon: Heading, label: "제목" },
		{ k: "b", icon: Bold, label: "굵게" },
		{ k: "i", icon: Italic, label: "기울임" },
		{ k: "ul", icon: List, label: "목록" },
		{ k: "task", icon: ListChecks, label: "체크리스트" },
		{ k: "code", icon: Code, label: "코드" },
		{ k: "link", icon: Link, label: "링크" },
		{ k: "quote", icon: Quote, label: "인용" },
	] as const;

	/// Read 모드 한 줄 강조: 제목 · 변수 · @include · 목록.
	const tone = (line: string) =>
		line.startsWith("#") ? "font-bold text-foreground" : line.startsWith("@include") ? "text-primary" : "text-muted-foreground";
	const parts = (line: string) => line.split(/(\{\{[^}]+\}\}|`[^`]+`|\*\*[^*]+\*\*)/g).filter(Boolean);
</script>

<div data-slot="md-editor" class={cn("flex min-h-0 flex-col overflow-hidden rounded-md border bg-card", className)}>
	{#if tabs}
		<div role="tablist" aria-label="지침 파일" class="flex shrink-0 overflow-x-auto border-b bg-muted">
			{#each files as f, i (f.name)}
				<div class={cn("-mb-px flex shrink-0 items-center gap-1.5 border-r px-3.5 py-2 text-xs", i === active ? "border-b border-b-card bg-card font-medium" : "text-muted-foreground")}>
					<button type="button" role="tab" aria-selected={i === active} onclick={() => (active = i)} class="flex items-center gap-1.5 outline-none focus-visible:underline">
						{#if f.name.endsWith("SOUL.md")}<Heart class={cn("size-3.25", i === active && "text-primary")} />{:else}<FileText class={cn("size-3.25", i === active && "text-primary")} />{/if}
						{f.name.split("/").pop()}
					</button>
					{#if changed(f)}<span class="size-1.5 rounded-full bg-status-waiting" aria-label="변경됨"></span>{/if}
					{#if closable && files.length > 1}
						<button type="button" aria-label="{f.name} 닫기" onclick={() => onclose?.(i)} class="text-subtle-foreground hover:text-foreground"><X class="size-3" /></button>
					{/if}
				</div>
			{/each}
		</div>
	{/if}

	<div class="flex shrink-0 items-center gap-0.5 overflow-x-auto border-b px-3 py-1.5 whitespace-nowrap">
		<div role="radiogroup" aria-label="보기" class="flex gap-0.5 rounded-sm bg-muted p-0.75">
			{#each modes as m (m.v)}
				<button
					type="button"
					role="radio"
					aria-checked={mode === m.v}
					onclick={() => (mode = m.v)}
					class={cn("rounded-xs px-2.5 py-0.75 text-caption outline-none focus-visible:ring-3 focus-visible:ring-ring/50", mode === m.v ? "bg-card font-semibold text-foreground shadow-xs" : "text-muted-foreground")}
				>{m.label}</button>
			{/each}
		</div>
		{#if mode === "edit"}
			<span class="mx-1.5 h-4.5 w-px bg-border"></span>
			{#each tools as t (t.k)}
				<button type="button" aria-label={t.label} title={t.label} onclick={() => format(t.k)} class="flex size-7 items-center justify-center rounded-xs text-muted-foreground outline-none hover:bg-muted hover:text-foreground focus-visible:ring-3 focus-visible:ring-ring/50">
					<t.icon class="size-3.5" />
				</button>
			{/each}
			<span class="mx-1.5 h-4.5 w-px bg-border"></span>
			<button type="button" onclick={() => format("var")} class="flex items-center gap-1.25 rounded-xs border px-2 py-1 text-xs font-medium outline-none hover:bg-muted focus-visible:ring-3 focus-visible:ring-ring/50"><Braces class="size-3.25 text-primary" />변수</button>
			<button type="button" onclick={() => format("include")} class="ml-1 flex items-center gap-1.25 rounded-xs border px-2 py-1 text-xs font-medium outline-none hover:bg-muted focus-visible:ring-3 focus-visible:ring-ring/50"><FileInput class="size-3.25 text-primary" />@include</button>
		{/if}
		<span class="flex-1"></span>
		{#if mode === "diff"}<span class="font-mono text-caption text-muted-foreground">+{plus} −{minus}</span>{/if}
	</div>

	{#if file}
		{#if mode === "edit"}
			<div class="flex min-h-0 flex-1 overflow-y-auto">
				<div aria-hidden="true" class="w-12 shrink-0 py-3 pr-3.5 text-right font-mono text-xs leading-5.5 text-subtle-foreground select-none">
					{#each file.body.split("\n") as _, i (i)}<div>{i + 1}</div>{/each}
				</div>
				<textarea
					bind:this={area}
					bind:value={file.body}
					onkeyup={track}
					onclick={track}
					spellcheck={false}
					aria-label="{file.name} 내용"
					class="field-sizing-content min-h-full flex-1 resize-none bg-transparent py-3 pr-4 pl-2 font-mono text-xs leading-5.5 whitespace-pre outline-none"
				></textarea>
			</div>
		{:else if mode === "read"}
			<div class="min-h-0 flex-1 overflow-y-auto py-3 font-mono text-xs leading-5.5">
				{#each file.body.split("\n") as line, i (i)}
					<div class="flex">
						<span class="w-12 shrink-0 pr-3.5 text-right text-subtle-foreground select-none">{i + 1}</span>
						<span class={cn("pl-2 whitespace-pre-wrap", tone(line))}>
							{#each parts(line) as p, j (j)}
								{#if p.startsWith("{{")}<span class="font-semibold text-primary">{p}</span>{:else if p.startsWith("`")}<span class="rounded-xs bg-muted px-0.5 text-foreground">{p}</span>{:else if p.startsWith("**")}<span class="font-bold text-foreground">{p}</span>{:else}{p}{/if}
							{/each}
						</span>
					</div>
				{/each}
			</div>
		{:else}
			<!-- 좌우를 한 행으로 묶어 줄이 넘쳐도 높이가 맞게 한다 -->
			<div class="min-h-0 flex-1 overflow-y-auto font-mono text-xs leading-5.5">
				<div class="sticky top-0 grid grid-cols-2 border-b bg-muted font-sans text-caption font-medium text-muted-foreground">
					<span class="border-r px-3 py-1.5">{baseLabel}</span>
					<span class="px-3 py-1.5">{currentLabel}</span>
				</div>
				{#each rows as r, i (i)}
					<div class="grid grid-cols-2">
						{#each [r.left, r.right] as cell, side (side)}
							{@const diff = cell && r.kind !== "same"}
							<div class={cn("flex min-h-5.5", side === 0 && "border-r", diff && (side === 0 ? "bg-destructive-soft" : "bg-success-soft"))}>
								<span class="w-10 shrink-0 pr-2.5 text-right text-subtle-foreground select-none">{cell?.n ?? ""}</span>
								<span class={cn("w-4 shrink-0 font-bold", side === 0 ? "text-destructive" : "text-status-done")}>{diff ? (side === 0 ? "−" : "+") : " "}</span>
								<span class="min-w-0 pr-3 break-words whitespace-pre-wrap text-muted-foreground">{cell?.text ?? ""}</span>
							</div>
						{/each}
					</div>
				{/each}
			</div>
		{/if}
		<div class="flex shrink-0 items-center gap-4 border-t bg-muted px-3 py-1.5 font-mono text-2xs text-muted-foreground">
			<span>Markdown</span>
			<span>{estimateTokens(file.body).toLocaleString()} tokens</span>
			{#if mode === "edit"}<span>Ln {caret.ln}, Col {caret.col}</span>{/if}
			<span class="flex-1"></span>
			{#if changed(file)}<span class="font-sans text-status-waiting">● 변경됨 ({baseLabel} 대비 +{plus} −{minus})</span>{/if}
			{#if status}<span class="font-sans">{@render status()}</span>{/if}
		</div>
	{/if}
</div>
