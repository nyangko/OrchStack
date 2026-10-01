<script lang="ts" module>
	import type { TaskStatus } from '$lib/status';
	import type { Priority } from '$lib/priority';
	import type { TaskDetail } from '$lib/mock';

	/// 편집기 초안 — 새 태스크면 num이 없다.
	export type TaskDraft = {
		num?: number;
		title: string;
		body: string;
		issue: number;
		status: TaskStatus;
		agent?: number;
		priority: Priority;
		criteria: TaskDetail['criteria'];
		deps: TaskDetail['deps'];
		eta: string;
		labels: string[];
		files: { name: string; size: number; url?: string }[];
	};

	/// 빈 초안 (QuickAdd도 같은 기본값으로 만든다).
	export const blankDraft = (issue: number, over: Partial<TaskDraft> = {}): TaskDraft => ({ title: '', body: '', issue, status: 'backlog', priority: 'P2', criteria: [], deps: [], eta: '', labels: [], files: [], ...over });
</script>

<script lang="ts">
	/**
	 * **태스크 만들기 · 고치기 창** (.pen XBNVi A · 새 태스크 / A' · 편집).
	 * 경로(이슈 · 하위 이슈) · 제목 · 설명(서식 · 첨부) · 완료 조건 · 속성 칩 · Orch 초안 · 진행 중 Run 경고 · 계속 만들기 · ⌘↵ 제출.
	 * 초안은 이 창이 갖고, 목록 · 상세에 쓰는 일은 데이터를 가진 페이지가 `onsave`로 한다.
	 * 여는 쪽은 `create(over)` · `edit(num)`을 부른다.
	 */
	import SquareCheck from '@lucide/svelte/icons/square-check';
	import CircleDot from '@lucide/svelte/icons/circle-dot';
	import GitBranch from '@lucide/svelte/icons/git-branch';
	import ChevronDown from '@lucide/svelte/icons/chevron-down';
	import Check from '@lucide/svelte/icons/check';
	import Maximize2 from '@lucide/svelte/icons/maximize-2';
	import Minimize2 from '@lucide/svelte/icons/minimize-2';
	import Paperclip from '@lucide/svelte/icons/paperclip';
	import FileText from '@lucide/svelte/icons/file-text';
	import X from '@lucide/svelte/icons/x';
	import Link2 from '@lucide/svelte/icons/link-2';
	import Timer from '@lucide/svelte/icons/timer';
	import Tag from '@lucide/svelte/icons/tag';
	import Sparkles from '@lucide/svelte/icons/sparkles';
	import WandSparkles from '@lucide/svelte/icons/wand-sparkles';
	import Radio from '@lucide/svelte/icons/radio';
	import UserRoundX from '@lucide/svelte/icons/user-round-x';
	import Bold from '@lucide/svelte/icons/bold';
	import Italic from '@lucide/svelte/icons/italic';
	import List from '@lucide/svelte/icons/list';
	import ListChecks from '@lucide/svelte/icons/list-checks';
	import Code from '@lucide/svelte/icons/code';
	import LinkIcon from '@lucide/svelte/icons/link';
	import AtSign from '@lucide/svelte/icons/at-sign';
	import { toast } from 'svelte-sonner';
	import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogBody, DialogFooter, DialogClose } from '$lib/components/ui/dialog';
	import { DropdownMenu, DropdownMenuTrigger, DropdownMenuContent, DropdownMenuItem } from '$lib/components/ui/dropdown-menu';
	import { InputGroup, InputGroupAddon, InputGroupButton, InputGroupText, InputGroupTextarea } from '$lib/components/ui/input-group';
	import { Alert, AlertDescription, AlertAction } from '$lib/components/ui/alert';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Kbd } from '$lib/components/ui/kbd';
	import { Switch } from '$lib/components/ui/switch';
	import { AttachmentGroup, Attachment, AttachmentMedia, AttachmentContent, AttachmentTitle, AttachmentDescription, AttachmentActions, AttachmentAction } from '$lib/components/orch/attachment';
	import { Checklist } from '$lib/components/orch/checklist';
	import { StatusSelect } from '$lib/components/orch/status-select';
	import { applyMd, type MdFormat } from '$lib/components/orch/md-editor';
	import AssigneePicker from './assignee-picker.svelte';
	import DependsPicker from './depends-picker.svelte';
	import PriorityPicker from './priority-picker.svelte';
	import { statuses } from '$lib/status';
	import { priorities } from '$lib/priority';
	import { roles } from '$lib/roles';
	import { recommendFor, orchPick } from '$lib/assign';
	import type { Agent, Issue, Task } from '$lib/mock';

	let {
		/** 열림 — 닫으면 초안을 버린다 */
		open = $bindable(false),
		/** 경로 첫 칸에 보일 프로젝트 이름 */
		project,
		/** 의존 · 담당 부하 계산용 */
		tasks,
		/** 경로의 이슈 · 하위 이슈 */
		issues,
		/** 담당 후보 */
		agents,
		/** 편집할 때 불러올 상세 · 진행 중 Run — 없는 태스크는 빈 값 */
		details,
		/** 저장. 새 태스크면 만든 번호를 돌려준다 — undefined면 저장하지 않은 것(창을 그대로 둔다) */
		onsave,
		/** 새 태스크를 만들고 창을 닫은 뒤 (계속 만들기가 꺼져 있을 때) */
		oncreated
	}: {
		open?: boolean;
		project: string;
		tasks: Task[];
		issues: Issue[];
		agents: Agent[];
		details: Record<number, TaskDetail>;
		onsave: (draft: TaskDraft) => number | undefined;
		oncreated?: (num: number) => void;
	} = $props();

	let draft = $state<TaskDraft>(blankDraft(0));
	let wide = $state(false);
	let more = $state(false);
	let bodyArea = $state<HTMLTextAreaElement | null>(null);
	let fileInput = $state<HTMLInputElement | null>(null);

	/// 새 태스크. over.issue가 없으면 진행 중인 첫 이슈 → 첫 이슈.
	export function create(over: Partial<TaskDraft> = {}) {
		const issue = over.issue ?? (issues.find((i) => i.status === 'in_progress') ?? issues[0])?.num ?? 0;
		draft = blankDraft(issue, { ...over, issue });
		wide = false;
		open = true;
	}
	/// 편집 모드 (Task 상세 ✎ · E).
	export function edit(num: number) {
		const t = tasks.find((x) => x.num === num);
		if (!t) return;
		const d = details[num];
		draft = {
			num, title: t.title, body: d?.description ?? t.description ?? '', issue: t.issue, status: t.status, agent: t.agent, priority: t.priority,
			criteria: structuredClone($state.snapshot(d?.criteria ?? [])), deps: [...(d?.deps ?? [])], eta: d?.eta ?? '', labels: [...(d?.labels ?? [])], files: []
		};
		wide = false;
		open = true;
	}

	/// 제출 (⌘↵). 새 태스크면 만들고(계속 만들기면 같은 경로 · 속성으로 비운 초안), 편집이면 덮어쓴다.
	function save(e?: Event) {
		e?.preventDefault();
		const d = draft;
		if (!d.title.trim()) return;
		const num = onsave(d);
		if (num === undefined) return;
		if (d.num === undefined && more) {
			draft = blankDraft(d.issue, { status: d.status, agent: d.agent, priority: d.priority });
			return;
		}
		open = false;
		if (d.num === undefined) oncreated?.(num);
	}

	const issueOf = (num: number) => issues.find((i) => i.num === num);
	const topIssue = (num: number): number => {
		const i = issueOf(num);
		return i?.parent ? topIssue(i.parent) : num;
	};

	/// 설명 툴바 서식 (MdEditor와 같은 규칙). @는 언급.
	async function format(kind: MdFormat) {
		if (!bodyArea) return;
		const { next, cursor } = applyMd(bodyArea.value, bodyArea.selectionStart, bodyArea.selectionEnd, kind);
		draft.body = next;
		await Promise.resolve();
		bodyArea.focus();
		bodyArea.setSelectionRange(cursor, cursor);
	}
	const bodyTools: { k: MdFormat; icon: typeof Bold; label: string }[] = [
		{ k: 'b', icon: Bold, label: '굵게' },
		{ k: 'i', icon: Italic, label: '기울임' },
		{ k: 'ul', icon: List, label: '목록' },
		{ k: 'task', icon: ListChecks, label: '체크리스트' },
		{ k: 'code', icon: Code, label: '코드' },
		{ k: 'link', icon: LinkIcon, label: '링크' },
		{ k: 'mention', icon: AtSign, label: '언급' }
	];
	/// 첨부 — 이미지는 썸네일, 그 밖은 파일 상자. 목데이터라 브라우저 안에서만 보여 준다.
	function attach(files: FileList | null) {
		if (!files) return;
		for (const f of files) draft.files.push({ name: f.name, size: f.size, url: f.type.startsWith('image/') ? URL.createObjectURL(f) : undefined });
	}
	const fileMeta = (f: { name: string; size: number }) => `${f.name.split('.').pop()?.toUpperCase() ?? 'FILE'} · ${f.size < 1024 ? `${f.size}B` : `${Math.round(f.size / 1024)}KB`}`;

	/// 제목만 적으면 Orch가 설명 · 완료 조건 · 담당 · 의존을 제안 (.pen Orch Draft). 목데이터: 정해진 틀로 채운다.
	function orchDraft() {
		const d = draft;
		if (!d.title.trim()) return toast.warning('제목을 먼저 적어 주세요');
		if (!d.body.trim()) d.body = `${d.title.trim()} — 범위 · 완료 기준을 정리했어요. 필요하면 고쳐 주세요.`;
		if (!d.criteria.length) d.criteria = [{ text: '동작 구현', done: false }, { text: '오류 · 빈 상태 처리', done: false }, { text: '테스트 통과', done: false }];
		d.agent ??= orchPick(agents, tasks, d.title, d.labels);
		toast.success('Orch 초안을 채웠어요');
	}
</script>

<Dialog bind:open>
	<!-- ⌘↵ 제출 -->
	<DialogContent size={wide ? 'xl' : 'md'} onkeydown={(e) => (e.metaKey || e.ctrlKey) && e.key === 'Enter' && save(e)}>
		{@const d = draft}
		{@const top = issueOf(topIssue(d.issue))}
		{@const subs = top ? issues.filter((i) => i.parent === top.num) : []}
		{@const leaf = issueOf(d.issue)}
		{@const who = agents.find((a) => a.sn === d.agent)}
		{@const st = statuses[d.status]}
		{@const pr = priorities[d.priority]}
		{@const run = d.num === undefined ? undefined : details[d.num]?.runs.find((r) => r.live)}
		{@const dep = d.deps.filter((x) => x.kind === 'depends')}
		<form onsubmit={save} class="contents">
			<DialogHeader>
				{#snippet lead()}<span class="flex size-5 shrink-0 items-center justify-center rounded-xs bg-node-task text-on-solid"><SquareCheck class="size-3" /></span>{/snippet}
				<DialogTitle class="sr-only">{d.num === undefined ? '새 태스크' : `Task #${d.num} 편집`}</DialogTitle>
				<!-- 경로: 프로젝트 / 이슈 / 하위 이슈 -->
				<div class="flex min-w-0 items-center gap-2 text-xs">
					<span class="text-muted-foreground">{project}</span>
					<span class="text-subtle-foreground">/</span>
					<DropdownMenu>
						<DropdownMenuTrigger>
							{#snippet child({ props })}
								<Button {...props} variant="outline" size="xs"><CircleDot class="text-node-issue" />{top ? `#${top.num} ${top.title}` : '이슈 선택'}<ChevronDown class="text-muted-foreground" /></Button>
							{/snippet}
						</DropdownMenuTrigger>
						<DropdownMenuContent align="start" class="w-64">
							{#each issues.filter((i) => !i.parent) as i (i.num)}
								<DropdownMenuItem onSelect={() => (d.issue = i.num)}><CircleDot class="text-node-issue" />#{i.num} {i.title}{#if top?.num === i.num}<Check class="ml-auto" />{/if}</DropdownMenuItem>
							{/each}
						</DropdownMenuContent>
					</DropdownMenu>
					{#if subs.length}
						<span class="text-subtle-foreground">/</span>
						<DropdownMenu>
							<DropdownMenuTrigger>
								{#snippet child({ props })}
									<Button {...props} variant="outline" size="xs"><GitBranch class="text-node-issue" />{leaf?.parent ? `#${leaf.num} ${leaf.title}` : '하위 이슈'}<ChevronDown class="text-muted-foreground" /></Button>
								{/snippet}
							</DropdownMenuTrigger>
							<DropdownMenuContent align="start" class="w-60">
								<DropdownMenuItem onSelect={() => top && (d.issue = top.num)}>하위 이슈 없음{#if !leaf?.parent}<Check class="ml-auto" />{/if}</DropdownMenuItem>
								{#each subs as i (i.num)}
									<DropdownMenuItem onSelect={() => (d.issue = i.num)}><GitBranch class="text-node-issue" />#{i.num} {i.title}{#if d.issue === i.num}<Check class="ml-auto" />{/if}</DropdownMenuItem>
								{/each}
							</DropdownMenuContent>
						</DropdownMenu>
					{/if}
				</div>
				{#snippet actions()}
					<Button variant="ghost" size="icon-sm" aria-label={wide ? '작게' : '크게'} onclick={() => (wide = !wide)}>{#if wide}<Minimize2 />{:else}<Maximize2 />{/if}</Button>
				{/snippet}
			</DialogHeader>

			<DialogBody class="gap-3.5">
				<!-- svelte-ignore a11y_autofocus -->
				<Input variant="title" autofocus bind:value={d.title} placeholder="태스크 제목" aria-label="제목" />

				<!-- 설명 (.pen Description Editor) -->
				<InputGroup>
					<InputGroupAddon align="block-start" class="border-b bg-muted">
						{#each bodyTools as t (t.k)}
							<InputGroupButton size="icon-xs" aria-label={t.label} title={t.label} onclick={() => format(t.k)}><t.icon /></InputGroupButton>
						{/each}
						<InputGroupButton size="icon-xs" aria-label="첨부" title="첨부" onclick={() => fileInput?.click()}><Paperclip /></InputGroupButton>
						<Input bind:ref={fileInput} type="file" multiple class="hidden" onchange={(e) => attach(e.currentTarget.files)} />
						<InputGroupText class="ml-auto text-caption">Markdown · @ 언급</InputGroupText>
					</InputGroupAddon>
					<InputGroupTextarea bind:ref={bodyArea} bind:value={d.body} placeholder="무엇을 · 왜 · 참고할 것" aria-label="설명" class="field-sizing-content min-h-16" />
					{#if d.files.length}
						<InputGroupAddon align="block-end">
							<AttachmentGroup>
								{#each d.files as f, i (i)}
									<Attachment size="sm">
										{#if f.url}
											<AttachmentMedia variant="image"><img src={f.url} alt={f.name} /></AttachmentMedia>
										{:else}
											<AttachmentMedia><FileText /></AttachmentMedia>
											<AttachmentContent><AttachmentTitle>{f.name}</AttachmentTitle><AttachmentDescription>{fileMeta(f)}</AttachmentDescription></AttachmentContent>
										{/if}
										<AttachmentActions><AttachmentAction aria-label="{f.name} 빼기" onclick={() => d.files.splice(i, 1)}><X /></AttachmentAction></AttachmentActions>
									</Attachment>
								{/each}
							</AttachmentGroup>
						</InputGroupAddon>
					{/if}
				</InputGroup>

				<!-- 완료 조건 -->
				<section class="flex flex-col border gap-0.5 rounded-md p-3" aria-label="완료 조건">
					<h3 class="pb-1 text-xs font-semibold text-muted-foreground">완료 조건</h3>
					<Checklist label="완료 조건" placeholder="조건 추가" bind:items={d.criteria} />
				</section>

				<!-- 속성 칩 (.pen Properties) — 선택 창은 C -->
				<div class="flex flex-wrap items-center gap-1.5">
					<StatusSelect bind:value={d.status}>
						{#snippet trigger(props)}<Button {...props} variant="outline" size="xs"><st.icon class={st.text} />{st.label}</Button>{/snippet}
					</StatusSelect>
					<AssigneePicker bind:value={d.agent} {agents} {tasks} recommend={recommendFor(agents, d.title, d.labels)} onorch={() => (d.agent = orchPick(agents, tasks, d.title, d.labels))}>
						{#snippet trigger(props)}
							{@const Icon = who ? roles[who.role].icon : UserRoundX}
							<Button {...props} variant="outline" size="xs"><Icon class={who ? roles[who.role].text : 'text-muted-foreground'} />{who ? `${who.name} · ${roles[who.role].label}` : 'Unassigned'}</Button>
						{/snippet}
					</AssigneePicker>
					<PriorityPicker bind:value={d.priority}>
						{#snippet trigger(props)}<Button {...props} variant="outline" size="xs"><pr.icon class={pr.text} />{d.priority}</Button>{/snippet}
					</PriorityPicker>
					<DependsPicker bind:value={d.deps} current={d.num} {tasks} depsOf={(n) => details[n]?.deps ?? []}>
						{#snippet trigger(props)}
							<Button {...props} variant="outline" size="xs"><Link2 class="text-muted-foreground" />{dep.length ? `depends on ${dep.map((x) => `#${x.num}`).join(', ')}` : d.deps.length ? `blocks ${d.deps.map((x) => `#${x.num}`).join(', ')}` : 'Depends'}</Button>
						{/snippet}
					</DependsPicker>
					<!-- ETA · 라벨 선택 창은 .pen에 없어 표시만 (#60) -->
					<Badge variant="outline"><Timer />{d.eta || 'ETA'}</Badge>
					<Badge variant="outline"><Tag />{d.labels.length ? d.labels.join(' · ') : 'Labels'}</Badge>
				</div>

				{#if d.num === undefined}
					<Alert variant="primary">
						<Sparkles />
						<AlertDescription>제목만 적으면 Orch가 설명 · 완료 조건 · 담당자 · 의존 관계를 제안해요</AlertDescription>
						<AlertAction><Button variant="outline" size="sm" onclick={orchDraft}><WandSparkles />초안 요청</Button></AlertAction>
					</Alert>
				{:else if run}
					<Alert variant="warning">
						<Radio />
						<AlertDescription>{who?.name ?? '담당'}이 Run #{run.num} 실행 중 — 저장하면 변경 사항이 Runtime Instruction으로 전달돼요</AlertDescription>
					</Alert>
				{/if}
			</DialogBody>

			<DialogFooter>
				{#snippet lead()}
					{#if d.num === undefined}
						<label class="meta-xs gap-1.5"><Switch size="sm" bind:checked={more} aria-label="계속 만들기" />계속 만들기</label>
					{/if}
				{/snippet}
				<DialogClose>{#snippet child({ props })}<Button type="button" variant="ghost" {...props}>취소</Button>{/snippet}</DialogClose>
				<Button type="submit" disabled={!d.title.trim()}>{d.num === undefined ? '태스크 생성' : '저장'}<Kbd class="bg-primary-foreground/15 text-primary-foreground">⌘↵</Kbd></Button>
			</DialogFooter>
		</form>
	</DialogContent>
</Dialog>
