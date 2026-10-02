<script lang="ts">
	/// 템플릿 화면 (.pen Teams › Agents / 템플릿) — 헤더 · 메뉴 · Overview · Instructions · Skills · Harness · Tools & MCP · 권한 · Revisions.
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import ChevronRight from '@lucide/svelte/icons/chevron-right';
	import Sparkles from '@lucide/svelte/icons/sparkles';
	import BookOpen from '@lucide/svelte/icons/book-open';
	import Library from '@lucide/svelte/icons/library';
	import SlidersHorizontal from '@lucide/svelte/icons/sliders-horizontal';
	import Wrench from '@lucide/svelte/icons/wrench';
	import ShieldCheck from '@lucide/svelte/icons/shield-check';
	import History from '@lucide/svelte/icons/history';
	import LayoutTemplate from '@lucide/svelte/icons/layout-template';
	import Terminal from '@lucide/svelte/icons/terminal';
	import Users from '@lucide/svelte/icons/users';
	import UserPlus from '@lucide/svelte/icons/user-plus';
	import Copy from '@lucide/svelte/icons/copy';
	import Plug from '@lucide/svelte/icons/plug';
	import Heart from '@lucide/svelte/icons/heart';
	import FileText from '@lucide/svelte/icons/file-text';
	import Folder from '@lucide/svelte/icons/folder';
	import ArrowRight from '@lucide/svelte/icons/arrow-right';
	import Upload from '@lucide/svelte/icons/upload';
	import PencilLine from '@lucide/svelte/icons/pencil-line';
	import BadgeCheck from '@lucide/svelte/icons/badge-check';
	import Star from '@lucide/svelte/icons/star';
	import FlaskConical from '@lucide/svelte/icons/flask-conical';
	import Ellipsis from '@lucide/svelte/icons/ellipsis';
	import Pencil from '@lucide/svelte/icons/pencil';
	import GitCompare from '@lucide/svelte/icons/git-compare';
	import Download from '@lucide/svelte/icons/download';
	import Archive from '@lucide/svelte/icons/archive';
	import ArchiveRestore from '@lucide/svelte/icons/archive-restore';
	import Trash2 from '@lucide/svelte/icons/trash-2';
	import FolderGit2 from '@lucide/svelte/icons/folder-git-2';
	import CircleCheck from '@lucide/svelte/icons/circle-check';
	import ListChecks from '@lucide/svelte/icons/list-checks';
	import Coins from '@lucide/svelte/icons/coins';
	import FileDiff from '@lucide/svelte/icons/file-diff';
	import RotateCcw from '@lucide/svelte/icons/rotate-ccw';
	import LoaderCircle from '@lucide/svelte/icons/loader-circle';
	import { toast } from 'svelte-sonner';
	import { useMock } from '$lib/api/env';
	import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogBody, DialogFooter } from '$lib/components/ui/dialog';
	import { DropdownMenu, DropdownMenuTrigger, DropdownMenuContent, DropdownMenuLabel, DropdownMenuSeparator, DropdownMenuEntries, type MenuEntry } from '$lib/components/ui/dropdown-menu';
	import { FieldRow } from '$lib/components/ui/field';
	import { Input } from '$lib/components/ui/input';
	import { Textarea } from '$lib/components/ui/textarea';
	import { Progress } from '$lib/components/ui/progress';
	import { ChoiceCards, ChoiceCard } from '$lib/components/orch/choice-cards';
	import { Card, CardHeader, CardTitle, CardContent, CardAction, CardDescription } from '$lib/components/ui/card';
	import { Empty, EmptyHeader, EmptyMedia, EmptyTitle, EmptyDescription } from '$lib/components/ui/empty';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Pill } from '$lib/components/orch/pill';
	import { Segmented } from '$lib/components/orch/segmented';
	import { RoleAvatar } from '$lib/components/orch/role-avatar';
	import { RuntimeLogo } from '$lib/components/orch/runtime-logo';
	import { MdEditor, estimateTokens, type MdFile } from '$lib/components/orch/md-editor';
	import { KeyValueRow } from '$lib/components/orch/key-value-row';
	import SkillsPanel from '$lib/components/orch/agent/skills-panel.svelte';
	import ToolsPanel from '$lib/components/orch/agent/tools-panel.svelte';
	import PermPanel from '$lib/components/orch/agent/perm-panel.svelte';
	import HarnessPanel from '$lib/components/orch/agent/harness-panel.svelte';
	import { store, defaultTeam, glyphOf, membersOf, runtimeName, accountOf, scopeText, liveMap, putDraft } from '$lib/teams.svelte';
	
	const t = $derived(store.templates.find((x) => x.sn === Number(page.params.template)) ?? store.templates[0]);
	/// 템플릿의 팀 관련 문구(추가 대상 · team.md)는 기본 팀 기준.
	const team = $derived(defaultTeam());
	const used = $derived(membersOf(t.name));

	const tplNav = [
		{ group: 'Template', items: [
			{ v: 'overview', label: 'Overview', icon: Sparkles },
			{ v: 'instructions', label: 'Instructions', icon: BookOpen },
			{ v: 'skills', label: 'Skills', icon: Library },
		] },
		{ group: 'Defaults', items: [
			{ v: 'harness', label: 'Harness 기본값', icon: SlidersHorizontal },
			{ v: 'tools', label: 'Tools & MCP', icon: Wrench },
			{ v: 'perm', label: '권한 기본값', icon: ShieldCheck },
		] },
		{ group: 'History', items: [{ v: 'revisions', label: 'Revisions', icon: History }] },
	];
	let tplTab = $state('overview');
	/// 템플릿 편집본 — 초안이 있으면 초안, 없으면 배포 중인 파일에서 시작한다. 템플릿을 바꾸면 다시 만든다.
	let tplWork = $state<MdFile[]>([]);
	let tplOpen = $state<string[]>([]);
	let tplActive = $state(0);
	let tplFor = $state<number>();
	let revFile = $state<string>();
	$effect.pre(() => {
		if (tplFor === t.sn) return;
		tplFor = t.sn;
		tplTab = 'overview';
		resetWork();
	});
	function resetWork() {
		tplWork = structuredClone($state.snapshot(t.draft ?? t.files));
		tplOpen = [tplWork[0].name];
		tplActive = 0;
		revFile = undefined;
	}
	const tplOpened = $derived(tplOpen.map((n) => tplWork.find((f) => f.name === n)!).filter(Boolean));
	const saved = $derived(t.draft ?? t.files);
	const dirty = $derived(tplWork.filter((f) => saved.find((o) => o.name === f.name)?.body !== f.body).length);
	/// 파일 트리에서 열기 — 탭에 없으면 추가한다.
	function openTplFile(name: string) {
		if (!tplOpen.includes(name)) tplOpen = [...tplOpen, name];
		tplActive = tplOpen.indexOf(name);
	}
	function closeTplFile(i: number) {
		tplOpen = tplOpen.filter((_, k) => k !== i);
		tplActive = Math.min(tplActive, tplOpen.length - 1);
	}
	/// 저장 — 다음 버전 초안으로 둔다 (게시는 Revisions에서).
	function saveDraft() {
		putDraft(t, $state.snapshot(tplWork), '나', '지침 수정');
	}
	/// 초안 게시 — 새 버전이 배포되고, 이미 만든 멤버에게는 자동 반영되지 않는다.
	function publish() {
		if (!t.draft) return;
		t.files = t.draft;
		t.draft = undefined;
		for (const r of t.revisions) if (r.state === 'live') r.state = 'old';
		const d = t.revisions.find((r) => r.state === 'draft')!;
		d.state = 'live';
		d.when = '방금';
		t.version = d.v;
		resetWork();
	}
	/// 복제 — 초안 없이 v1로 시작한다.
	function duplicate() {
		const sn = Math.max(...store.templates.map((x) => x.sn)) + 1;
		const s = $state.snapshot(t);
		store.templates.push({ ...structuredClone(s), sn, name: `${s.name} 복사본`, version: 1, members: 0, draft: undefined, created: '나 · 방금', revisions: [{ v: 1, state: 'live', who: '나', when: '방금', note: `${s.name} v${s.version}에서 복제` }] });
		goto(`/teams/agents/${sn}`);
	}
	// ── 머리글 메뉴 (.pen Teams · 머리글 메뉴) — ☆ · 테스트 Run · … ─────────
	/// 이름 · 설명 편집 창.
	let rename = $state<{ name: string; desc: string }>();
	function saveRename() {
		if (!rename?.name.trim()) return;
		t.name = rename.name.trim();
		t.desc = rename.desc.trim();
		rename = undefined;
		toast.success('템플릿 이름 · 설명을 바꿨어요');
	}
	/// JSON 내보내기 — 지침 파일 · 기본값 · 버전 기록을 한 파일로.
	function exportJson() {
		const blob = new Blob([JSON.stringify($state.snapshot(t), null, 2)], { type: 'application/json' });
		const a = Object.assign(document.createElement('a'), { href: URL.createObjectURL(blob), download: `template-${t.name.replace(/\s+/g, '-').toLowerCase()}-v${t.version}.json` });
		a.click();
		URL.revokeObjectURL(a.href);
	}
	/// 삭제 — 이 템플릿으로 만든 멤버가 없을 때만. 알림에서 되돌릴 수 있다.
	function remove() {
		const i = store.templates.indexOf(t);
		const gone = store.templates.splice(i, 1)[0];
		goto(`/teams/agents/${store.templates[Math.max(0, i - 1)].sn}`);
		toast(`${gone.name} 템플릿을 삭제했어요`, { action: { label: '되돌리기', onClick: () => (store.templates.splice(i, 0, gone), goto(`/teams/agents/${gone.sn}`)) } });
	}
	const tplMenu: MenuEntry[] = $derived([
		{ label: '이름 · 설명 편집', icon: Pencil, onSelect: () => (rename = { name: t.name, desc: t.desc }) },
		{ label: '버전 기록', icon: History, onSelect: () => (tplTab = 'revisions') },
		{ label: `멤버와 비교 · ${used.length}명`, icon: GitCompare, disabled: !used.length, sub: used.map(({ m, team: tn }) => ({ label: `${m.name} · ${tn}`, onSelect: () => goto(`/teams?team=${store.crew.find((c) => c.name === tn)?.sn}&member=${m.sn}&tab=instructions`) })) },
		{ label: 'JSON으로 내보내기', icon: Download, onSelect: exportJson },
		'sep',
		t.archived
			? { label: '보관 해제', icon: ArchiveRestore, onSelect: () => ((t.archived = false), toast('보관을 풀었어요 · 멤버 추가에 다시 보여요')) }
			: { label: '보관 · 새 멤버에 숨김', icon: Archive, onSelect: () => ((t.archived = true), toast('보관했어요 · 이미 만든 멤버는 그대로예요')) },
		{ label: '삭제', icon: Trash2, tone: 'text-destructive', disabled: used.length > 0, onSelect: remove }
	]);

	/// 테스트 Run (.pen 테스트 Run 다이얼로그) — 팀에 넣지 않고 샘플 태스크로 한 번 실행. 임시 worktree에서 돌고 끝나면 지운다.
	/// 목데이터: 실행은 흉내만 낸다(실행기 연동 #13 전). 서버 모드에서는 막는다.
	const samples = [
		{ v: 'a11y', t: '로그인 폼 접근성 점검', d: '라벨 · 대비 · 키보드 이동을 고치고 테스트 통과', criteria: 3, tok: 18 },
		{ v: 'variant', t: '버튼 컴포넌트 variant 추가', d: 'shadcn-svelte 규칙대로', criteria: 2, tok: 11 },
		{ v: 'custom', t: '직접 입력', d: '제목과 완료 조건을 적어서 실행', criteria: 0, tok: 15 }
	];
	let test = $state<{ sample: string; title: string; criteria: string; phase: 'setup' | 'running' | 'done'; pct: number }>();
	let testTimer: ReturnType<typeof setInterval> | undefined;
	const testSample = $derived(samples.find((x) => x.v === test?.sample) ?? samples[0]);
	function startTest() {
		if (!test) return;
		test.phase = 'running';
		test.pct = 0;
		clearInterval(testTimer);
		testTimer = setInterval(() => {
			if (!test) return clearInterval(testTimer);
			test.pct = Math.min(100, test.pct + 20);
			if (test.pct >= 100) (clearInterval(testTimer), (test.phase = 'done'));
		}, 400);
	}
	function closeTest() {
		clearInterval(testTimer);
		test = undefined;
	}

	const revChanged = $derived(t.draft ? t.draft.filter((f) => t.files.find((o) => o.name === f.name)?.body !== f.body) : []);
</script>

<svelte:head><title>{t.name} · Agents · OrchStack</title></svelte:head>


<main class="flex min-w-0 flex-1 flex-col">
	<nav aria-label="Breadcrumb" class="flex items-center gap-1.5 border-b px-6 py-2.5 text-xs text-muted-foreground">
		<span>Teams</span><ChevronRight class="size-3" /><span>Agents › {t.name}</span><ChevronRight class="size-3" />
		<span class="font-medium text-foreground">{tplNav.flatMap((g) => g.items).find((i) => i.v === tplTab)?.label}</span>
	</nav>
	<div class="flex min-h-0 flex-1">
		<nav aria-label="템플릿 메뉴" class="flex w-53 shrink-0 flex-col gap-0.5 border-r bg-sidebar px-2.5 py-3.5">
			{#each tplNav as g, gi (g.group)}
				<span class={['list-label px-2 pb-1.5', gi > 0 ? 'mt-1.5 border-t pt-3.5' : 'pt-0.5']}>{g.group}</span>
				{#each g.items as it (it.v)}
					<button
						type="button"
						aria-current={tplTab === it.v ? 'page' : undefined}
						onclick={() => (tplTab = it.v)}
						class={['side-nav-item h-8.5', tplTab === it.v ? 'bg-accent font-semibold text-foreground' : 'text-muted-foreground']}
					>
						<it.icon class="size-4" />
						<span class="flex-1 text-left">{it.label}</span>
						{#if it.v === 'revisions' && t.draft}<span class="size-1.5 rounded-full bg-status-waiting" aria-label="초안 있음"></span>{/if}
					</button>
				{/each}
			{/each}
		</nav>
		<div class="flex min-w-0 flex-1 flex-col overflow-y-auto *:shrink-0">
			<header class="flex items-center gap-3.5 border-b px-7 py-5">
				<RoleAvatar role={t.role} size="lg" class="overflow-visible">
					<RuntimeLogo runtime={t.runtime} class="absolute -right-1 -bottom-1 size-4" />
				</RoleAvatar>
				<div class="flex min-w-0 flex-1 flex-col gap-1.5">
					<div class="flex items-center gap-2.5">
						<h1 class="text-xl font-bold">{t.name}</h1>
						<Badge variant="secondary" class="gap-1"><LayoutTemplate class="size-3" />템플릿 · v{t.version}</Badge>
						{#if t.draft}<Badge variant="outline" class="text-status-waiting">v{t.revisions.find((r) => r.state === 'draft')?.v} 초안</Badge>{/if}
					</div>
					<div class="meta-xs flex-wrap gap-2">
						<span>역할 · {t.focus}</span>
						<span class="text-subtle-foreground">·</span>
						<span class="flex items-center gap-1"><Terminal class="size-3" />기본 {runtimeName(t.runtime)} · {t.model} · Auto</span>
						<span class="text-subtle-foreground">·</span>
						<Badge variant="secondary" class="gap-1"><Users class="size-3" />이 템플릿으로 만든 멤버 {used.length}</Badge>
					</div>
				</div>
				{#if t.archived}<Badge variant="outline" class="gap-1 text-muted-foreground"><Archive class="size-3" />보관됨</Badge>{/if}
				<Button variant="ghost" size="icon-sm" class={t.favorite ? 'bg-warning-soft text-warning hover:bg-warning-soft hover:text-warning' : 'text-muted-foreground'} aria-pressed={!!t.favorite} aria-label="즐겨찾기" title={t.favorite ? '즐겨찾기 해제' : '즐겨찾기 · 목록 맨 위에 고정'} onclick={() => (t.favorite = !t.favorite)}>
					<Star class={t.favorite ? 'fill-current' : undefined} />
				</Button>
				<Button variant="outline" size="sm" href="/teams?team={team.sn}&add={t.sn}" disabled={t.archived}><UserPlus />{team.name}에 추가</Button>
				<Button variant="outline" size="sm" onclick={duplicate}><Copy />복제</Button>
				<Button variant="outline" size="sm" disabled={!useMock} title={useMock ? undefined : '실행기 연동(#13) 후 쓸 수 있어요'} onclick={() => (test = { sample: samples[0].v, title: '', criteria: '', phase: 'setup', pct: 0 })}><FlaskConical />테스트 Run</Button>
				<DropdownMenu>
					<DropdownMenuTrigger>
						{#snippet child({ props })}<Button {...props} variant="ghost" size="icon-sm" aria-label="템플릿 메뉴"><Ellipsis /></Button>{/snippet}
					</DropdownMenuTrigger>
					<DropdownMenuContent align="end" class="w-63">
						<DropdownMenuLabel class="truncate">{t.name} · 템플릿 v{t.version}</DropdownMenuLabel>
						<DropdownMenuSeparator />
						<DropdownMenuEntries entries={tplMenu} />
						{#if used.length}
							<p class="px-2 pt-1 pb-1.5 text-caption text-muted-foreground">이 템플릿으로 만든 멤버 {used.length}명이 있어 삭제할 수 없어요. 보관하면 새 멤버 추가에서만 숨겨져요.</p>
						{/if}
					</DropdownMenuContent>
				</DropdownMenu>
			</header>

			<div class="flex flex-col gap-4 bg-muted px-7 py-5 *:shrink-0">
				{#if tplTab === 'overview'}
					<div class="flex items-start gap-4">
						<div class="flex min-w-0 flex-1 flex-col gap-4">
							<Card size="sm">
								<CardHeader><CardTitle>설명</CardTitle></CardHeader>
								<CardContent class="gap-2.5">
									<p class="text-body">{t.desc}</p>
									<div class="flex gap-1.5">{#each t.tags as tag (tag)}<Pill>#{tag}</Pill>{/each}</div>
								</CardContent>
							</Card>
							{#each t.files.filter((f) => f.name === 'SOUL.md') as soul (soul.name)}
								<Card size="sm">
									<CardHeader><CardTitle>Soul 미리보기</CardTitle></CardHeader>
									<CardContent class="gap-2">
										<pre class="rounded-sm bg-muted p-3 font-sans text-xs leading-relaxed whitespace-pre-wrap text-muted-foreground">{soul.body}</pre>
										<span class="text-caption text-subtle-foreground">SOUL.md · 멤버마다 따로 다듬어져요</span>
									</CardContent>
								</Card>
							{/each}
							<Card size="sm">
								<CardHeader>
									<CardTitle class="flex items-center gap-2">스킬 <Pill>{t.config.skills.length + t.config.mcp.length} 활성</Pill></CardTitle>
									<CardAction><Button variant="link" size="xs" onclick={() => (tplTab = 'skills')}>관리</Button></CardAction>
								</CardHeader>
								<CardContent class="gap-2">
									<div class="flex flex-wrap gap-1.5">
										{#each t.config.skills as sk (sk)}<Pill><Sparkles />{sk}</Pill>{/each}
										{#each t.config.mcp as mc (mc)}<Pill><Plug />{mc}</Pill>{/each}
									</div>
									<span class="text-caption text-muted-foreground">다음 Run부터 적용 · 컨텍스트 +{((t.config.skills.length + t.config.mcp.length) * 0.24).toFixed(1)}K tok</span>
								</CardContent>
							</Card>
							<Card size="sm">
								<CardHeader><CardTitle>이 템플릿으로 만든 멤버 · {used.length}</CardTitle></CardHeader>
								<CardContent class="gap-0">
									{#each used as { m, team: tn } (m.sn)}
										{@const changedFiles = (m.files ?? t.files).filter((f) => t.files.find((o) => o.name === f.name)?.body !== f.body)}
										<div class="row-divided items-center gap-2.5 py-2">
											<RoleAvatar role={m.role} icon={glyphOf(m)} />
											<span class="flex flex-1 flex-col gap-px">
												<span class="text-xs font-medium">{m.name} · {tn}</span>
												<span class="text-caption text-muted-foreground">변경 {changedFiles.length}{changedFiles.length ? ` · ${changedFiles.map((f) => f.name.split('/').pop()).join(', ')}` : ' · 템플릿과 동일'}</span>
											</span>
										</div>
									{:else}
										<p class="text-xs text-muted-foreground">아직 이 템플릿으로 만든 멤버가 없어요.</p>
									{/each}
									<span class="border-t pt-2 text-caption text-subtle-foreground">멤버는 추가 시점의 복사본이에요. 템플릿을 고쳐도 자동 반영되지 않아요.</span>
								</CardContent>
							</Card>
						</div>
						<div class="flex shrink-0 flex-col w-80 gap-4">
							<Card size="sm">
								<CardHeader><CardTitle>구성 기본값</CardTitle></CardHeader>
								<CardContent class="gap-0"><KeyValueRow label="역할" value={t.focus} /><KeyValueRow label="만든 사람" value={t.created} /></CardContent>
							</Card>
							<Card size="sm">
								<CardHeader><CardTitle>하네스 / 런타임</CardTitle></CardHeader>
								<CardContent class="gap-0">
									<KeyValueRow label="CLI" value={runtimeName(t.runtime)} />
									<KeyValueRow label="모델 · Effort" value={`${t.model} · Auto`} />
									<KeyValueRow label="기본 계정" value={accountOf(t.runtime).plan} />
								</CardContent>
							</Card>
							<Card size="sm">
								<CardHeader><CardTitle>도구 · 권한</CardTitle></CardHeader>
								<CardContent class="gap-0">
									<KeyValueRow label="MCP" value={`${t.config.mcp.length}`} />
									<KeyValueRow label="Trust" value={`${t.config.trust}`} />
									<KeyValueRow label="파일 쓰기" value={scopeText(t.config)} />
									<KeyValueRow label="PR 생성" value="승인 필요" />
								</CardContent>
							</Card>
							<Card size="sm">
								<CardHeader>
									<CardTitle>최근 Revision</CardTitle>
									<CardAction><Button variant="link" size="xs" onclick={() => (tplTab = 'revisions')}>전체</Button></CardAction>
								</CardHeader>
								<CardContent class="gap-2">
									{#each t.revisions.slice(0, 3) as r (r.v)}
										<div class="flex items-center gap-2 text-xs">
											<Pill class={['font-mono', r.state === 'draft' && 'bg-warning-soft text-status-waiting']}>v{r.v}</Pill>
											<span class="flex-1 truncate">{r.note}</span>
											<span class="font-mono text-caption text-subtle-foreground">{r.when}</span>
										</div>
									{/each}
								</CardContent>
							</Card>
						</div>
					</div>
				{:else if tplTab === 'instructions'}
					{@const agent = tplWork.find((f) => f.name === 'AGENT.md')}
					{@const includes = (agent?.body.match(/@include \S+/g) ?? []).map((x) => x.slice(9))}
					{@const incTok = includes.reduce((n, name) => n + estimateTokens(tplWork.find((f) => f.name === name)?.body ?? ''), 0)}
					{@const stack = [['Orch 시스템 지침', '고정', 420], ['AGENT.md', '프로필', estimateTokens(agent?.body ?? '')], [`@include ${includes.length}개`, '프로필', incTok], [`${team.name} · team.md`, '팀', 310], ['태스크 지시 + 완료 조건', '태스크', 620]] as const}
					<div class="flex items-center gap-2.5">
						<p class="flex-1 text-xs text-muted-foreground">CLI 실행 시 AGENTS.md / CLAUDE.md로 변환돼 각 CLI의 기본 시스템 프롬프트 뒤에 전달돼요</p>
						{#if dirty}<span class="text-xs font-medium text-status-waiting">● 저장 안 된 파일 {dirty}개</span>{/if}
						<Button variant="ghost" size="sm" disabled={!dirty} onclick={resetWork}>취소</Button>
						<Button size="sm" disabled={!dirty} onclick={saveDraft}>v{t.draft ? t.revisions.find((r) => r.state === 'draft')?.v : t.version + 1} 초안에 저장</Button>
					</div>
					<div class="flex h-160 gap-3.5">
						<nav aria-label="지침 파일" class="card flex w-56 shrink-0 flex-col gap-0.5 overflow-y-auto p-2 text-xs rounded-md">
							<span class="list-label px-2 pt-1 pb-1.5">파일</span>
							{#each tplWork as f (f.name)}
								{@const depth = f.name.includes('/') ? 1 : 0}
								{@const folder = f.name.split('/')[0]}
								{#if depth && tplWork.find((x) => x.name.startsWith(folder + '/')) === f}
									<span class="menu-line"><Folder class="size-3.5" />{folder}</span>
								{/if}
								{@const changedF = t.files.find((o) => o.name === f.name)?.body !== f.body}
								<button
									type="button"
									aria-current={tplOpened[tplActive]?.name === f.name ? 'true' : undefined}
									onclick={() => openTplFile(f.name)}
									class={['flex items-center gap-1.5 rounded-sm py-1.5 pr-2 text-left outline-none hover:bg-muted focus-visible:ring-3 focus-visible:ring-ring/50', depth ? 'pl-7' : 'pl-2', tplOpened[tplActive]?.name === f.name && 'bg-accent font-medium']}
								>
									{#if f.name === 'SOUL.md'}<Heart class="size-3.5 text-muted-foreground" />{:else}<FileText class="size-3.5 text-muted-foreground" />{/if}
									<span class="flex-1 truncate">{f.name.split('/').pop()}</span>
									{#if f.name === 'AGENT.md'}<Pill class="px-1.5 text-2xs">ENTRY</Pill>{:else if f.name === 'SOUL.md'}<Pill class="px-1.5 text-2xs">성격</Pill>{/if}
									{#if changedF}<span class="size-1.5 rounded-full bg-status-waiting" aria-label="v{t.version} 대비 변경"></span>{/if}
								</button>
							{/each}
							<span class="list-label mt-2 border-t px-2 pt-3 pb-1.5">CLI 출력 (자동 생성)</span>
							<span class="menu-line"><ArrowRight class="size-3" />Codex CLI<span class="ml-auto font-mono text-2xs">AGENTS.md</span></span>
							<span class="menu-line"><ArrowRight class="size-3" />Claude Code<span class="ml-auto font-mono text-2xs">CLAUDE.md</span></span>
						</nav>
						<MdEditor files={tplOpened} bind:active={tplActive} base={liveMap(t.files)} baseLabel="v{t.version}" currentLabel="편집본" closable onclose={closeTplFile} class="min-w-0 flex-1">
							{#snippet status()}{t.draft ? '초안 편집 중' : `v${t.version} 기준`}{/snippet}
						</MdEditor>
						<aside class="flex shrink-0 flex-col w-64 gap-3.5">
							<Card size="sm">
								<CardHeader><CardTitle>Run 시 합성되는 컨텍스트</CardTitle></CardHeader>
								<CardContent class="gap-0">
									{#each stack as [label, kind, n], i (label)}
										<div class="flex items-center gap-2 border-t py-2 text-xs first:border-t-0">
											<span class="flex size-4.5 items-center justify-center rounded-full bg-muted font-mono text-2xs">{i + 1}</span>
											<span class="flex-1 truncate">{label}</span>
											<span class="text-caption text-subtle-foreground">{kind}</span>
											<span class="w-10 text-right font-mono text-caption">{n}</span>
										</div>
									{/each}
									<KeyValueRow label="합계" value={`${stack.reduce((s, x) => s + x[2], 0).toLocaleString()} / 권장 8K`} />
								</CardContent>
							</Card>
							<Card size="sm">
								<CardHeader><CardTitle>사용 가능한 변수</CardTitle></CardHeader>
								<CardContent class="gap-1.5">
									{#each [['{{team.name}}', team.name], ['{{agent.alias}}', t.name], ['{{task.title}}', '태스크 제목'], ['{{task.criteria}}', '완료 조건']] as [k, v] (k)}
										<div class="flex items-center justify-between gap-2 text-xs"><code class="font-mono text-primary">{k}</code><span class="truncate text-muted-foreground">{v}</span></div>
									{/each}
								</CardContent>
							</Card>
						</aside>
					</div>
				{:else if tplTab === 'harness'}
					<HarnessPanel config={t.config} runtime={t.runtime} model={t.model} onchange={(r, md) => ((t.runtime = r), (t.model = md))} />
				{:else if tplTab === 'skills'}
					<SkillsPanel config={t.config} runtime={t.runtime} target="{t.name} 템플릿" />
				{:else if tplTab === 'tools'}
					<ToolsPanel config={t.config} teamName={team.name} />
				{:else if tplTab === 'perm'}
					<PermPanel config={t.config} {used} />
				{:else if tplTab === 'revisions'}
					{@const draftRev = t.revisions.find((r) => r.state === 'draft')}
					{@const cur = revFile ?? revChanged[0]?.name}
					<div class="flex items-center gap-2.5">
						<h2 class="flex-1 text-lg font-semibold">Revisions</h2>
						<Button size="sm" disabled={!t.draft} onclick={publish}><Upload />{draftRev ? `v${draftRev.v} 게시` : '게시할 초안 없음'}</Button>
					</div>
					<div class="flex items-start gap-4">
						<div class="flex shrink-0 flex-col w-80 gap-4">
							<Card size="sm">
								<CardHeader>
									<CardTitle>버전</CardTitle>
									<CardDescription>멤버는 복사본 · 새 버전은 자동 반영되지 않아요</CardDescription>
								</CardHeader>
								<CardContent class="gap-0">
									{#each t.revisions as r (r.v)}
										{@const Icon = r.state === 'draft' ? PencilLine : r.state === 'live' ? BadgeCheck : History}
										<div class="row-divided gap-2.5 py-2.5">
											<Icon class={['mt-0.5 size-4 shrink-0', r.state === 'draft' ? 'text-status-waiting' : r.state === 'live' ? 'text-status-done' : 'text-muted-foreground']} />
											<span class="flex flex-col gap-0.5">
												<span class="flex items-center gap-2 text-xs"><span class="font-semibold">v{r.v}{r.state === 'draft' ? ' · 초안' : r.state === 'live' ? ' · 배포 중' : ''}</span><span class="text-muted-foreground">{r.who}</span><span class="font-mono text-caption text-subtle-foreground">{r.when}</span></span>
												<span class="text-xs text-muted-foreground">{r.note}</span>
											</span>
										</div>
									{/each}
								</CardContent>
							</Card>
							<Card size="sm">
								<CardHeader>
									<CardTitle>이 템플릿을 쓰는 멤버</CardTitle>
									<CardDescription>v{t.version} 기준 · 개별 변경은 유지돼요</CardDescription>
								</CardHeader>
								<CardContent class="gap-0">
									{#each used as { m, team: tn } (m.sn)}
										{@const n = (m.files ?? t.files).filter((f) => t.files.find((o) => o.name === f.name)?.body !== f.body).length}
										<KeyValueRow label={`${m.name} · ${tn}`} value={n ? `개별 변경 ${n}` : '변경 없음'} />
									{:else}
										<p class="text-xs text-muted-foreground">없어요.</p>
									{/each}
								</CardContent>
							</Card>
						</div>
						{#if t.draft && cur}
							<div class="flex min-w-0 flex-1 flex-col gap-2.5">
								<div class="flex items-center gap-2.5">
									<Segmented
										class="w-auto"
										aria-label="비교할 파일"
										options={revChanged.map((f) => ({ value: f.name, label: f.name.split('/').pop()!, icon: f.name === 'SOUL.md' ? Heart : FileText }))}
										bind:value={() => cur, (v) => (revFile = v)}
									/>
									<span class="text-xs text-muted-foreground">v{t.version} → v{draftRev?.v} · 파일 {revChanged.length}개</span>
								</div>
								<MdEditor
									files={t.draft.filter((f) => f.name === cur)}
									base={liveMap(t.files)}
									baseLabel="v{t.version} · 배포 중"
									currentLabel="v{draftRev?.v} · 초안"
									mode="diff"
									readonly
									tabs={false}
									class="h-120"
								/>
							</div>
						{:else}
							<Empty class="flex-1 bg-card">
								<EmptyHeader>
									<EmptyMedia variant="icon"><History /></EmptyMedia>
									<EmptyTitle>게시할 초안이 없어요</EmptyTitle>
									<EmptyDescription>Instructions에서 고치고 저장하면 다음 버전 초안이 생기고, 여기서 배포 중인 버전과 비교해 게시해요.</EmptyDescription>
								</EmptyHeader>
							</Empty>
						{/if}
					</div>
				{:else}
					<Empty class="bg-card">
						<EmptyHeader>
							<EmptyMedia variant="icon"><SlidersHorizontal /></EmptyMedia>
							<EmptyTitle>{tplNav.flatMap((g) => g.items).find((i) => i.v === tplTab)?.label}</EmptyTitle>
							<EmptyDescription>이어서 만들어요. (#65 · #63 T-3b)</EmptyDescription>
						</EmptyHeader>
					</Empty>
				{/if}
			</div>
		</div>
	</div>
</main>

<!-- 이름 · 설명 편집 (템플릿 … 메뉴) -->
<Dialog bind:open={() => rename !== undefined, (v) => !v && (rename = undefined)}>
	<DialogContent size="sm">
		{#if rename}
			{@const f = rename}
			<DialogHeader icon={Pencil}>
				<DialogTitle>이름 · 설명 편집</DialogTitle>
				<DialogDescription>바꿔도 이미 만든 멤버의 이름은 그대로예요.</DialogDescription>
			</DialogHeader>
			<DialogBody>
				<FieldRow label="이름" as="label"><Input bind:value={f.name} /></FieldRow>
				<FieldRow label="설명" as="label"><Textarea bind:value={f.desc} rows={3} /></FieldRow>
			</DialogBody>
			<DialogFooter>
				<Button variant="ghost" size="sm" onclick={() => (rename = undefined)}>취소</Button>
				<Button size="sm" disabled={!f.name.trim()} onclick={saveRename}>저장</Button>
			</DialogFooter>
		{/if}
	</DialogContent>
</Dialog>

<!-- 테스트 Run (.pen Teams · 머리글 메뉴 · 테스트 Run 다이얼로그) — 실행 전 / 실행 중 / 결과 -->
<Dialog bind:open={() => test !== undefined, (v) => !v && closeTest()}>
	<DialogContent size="md">
		{#if test}
			{@const f = test}
			{@const acc = accountOf(t.runtime)}
			<DialogHeader icon={FlaskConical}>
				<DialogTitle>테스트 Run · {t.name} v{t.version}</DialogTitle>
				<DialogDescription>팀에 넣지 않고 샘플 태스크로 한 번 실행해 봐요</DialogDescription>
			</DialogHeader>
			{#if f.phase === 'setup'}
				<DialogBody>
					<FieldRow label="샘플 태스크" hint="템플릿 역할에 맞는 예시 · 완료 조건까지 함께 실행">
						<ChoiceCards aria-label="샘플 태스크" class="flex flex-col gap-2" bind:value={() => f.sample, (v) => (f.sample = v)}>
							{#each samples as o, i (o.v)}
								<ChoiceCard value={o.v} class="gap-1 rounded-lg px-3.5 py-3">
									<span class="row-title-strong"><span class="font-mono text-caption text-muted-foreground">{'ABC'[i]}</span>{o.t}</span>
									<span class="text-caption text-muted-foreground">{o.d}{o.criteria ? ` · 완료 조건 ${o.criteria}` : ''}</span>
								</ChoiceCard>
							{/each}
						</ChoiceCards>
					</FieldRow>
					{#if f.sample === 'custom'}
						<FieldRow label="제목" as="label"><Input bind:value={f.title} placeholder="예: 검색 입력에 지우기 버튼 추가" /></FieldRow>
						<FieldRow label="완료 조건" hint="한 줄에 하나" as="label"><Textarea bind:value={f.criteria} rows={3} placeholder="버튼을 누르면 입력이 비워진다" /></FieldRow>
					{/if}
					<FieldRow label="실행 환경">
						<span class="flex h-9 items-center gap-2 rounded-md border px-3 text-body"><Terminal class="size-3.5 text-muted-foreground" />{runtimeName(t.runtime)} · {t.model} · Auto<span class="ml-auto text-caption text-muted-foreground">템플릿 기본값</span></span>
					</FieldRow>
					<FieldRow label="작업 폴더">
						<span class="flex items-center gap-2 rounded-md bg-muted px-2.5 py-2 text-xs"><FolderGit2 class="size-3.5 text-muted-foreground" />임시 worktree · 끝나면 삭제 · 커밋 · PR 없음</span>
					</FieldRow>
					<div class="flex flex-col">
						<KeyValueRow label="예상 토큰" value="~{testSample.tok}K tok" />
						<KeyValueRow label="{acc.plan} 주간 잔량" value="{acc.week}% 남음" />
						<KeyValueRow label="호환성" value="{runtimeName(t.runtime)} · 모두 호환" />
					</div>
				</DialogBody>
				<DialogFooter note="토큰은 워크스페이스 한도에 포함돼요">
					<Button variant="ghost" size="sm" onclick={closeTest}>취소</Button>
					<Button size="sm" disabled={f.sample === 'custom' && (!f.title.trim() || !f.criteria.trim())} onclick={startTest}><FlaskConical />테스트 시작</Button>
				</DialogFooter>
			{:else}
				{@const name = f.sample === 'custom' ? f.title : testSample.t}
				{@const total = f.sample === 'custom' ? f.criteria.split('\n').filter((x) => x.trim()).length : testSample.criteria}
				<DialogBody>
					{#if f.phase === 'running'}
						<p class="flex items-center gap-2 text-body font-semibold"><LoaderCircle class="size-4 animate-spin text-status-in-progress" />{name} 실행 중</p>
						<Progress value={f.pct} class="h-1.5" aria-label="테스트 Run 진행" tip={`테스트 Run\n${f.pct}% · 임시 worktree`} />
						<p class="text-caption text-muted-foreground">임시 worktree에서 실행 중이에요. 닫으면 멈추고 worktree를 지워요.</p>
					{:else}
						<p class="flex items-center gap-2 text-body font-semibold"><CircleCheck class="size-4 text-success" /><span class="flex-1">테스트 통과 · {name}</span><span class="font-mono text-xs font-normal text-muted-foreground">4m 12s</span></p>
						<div class="flex flex-col">
							<KeyValueRow label="완료 조건"><span class="flex items-center gap-1.5 font-medium text-success"><ListChecks class="size-3.5" />{total} / {total} 통과</span></KeyValueRow>
							<KeyValueRow label="토큰"><span class="flex items-center gap-1.5 font-medium"><Coins class="size-3.5 text-muted-foreground" />{(testSample.tok * 0.9).toFixed(1)}K tok</span></KeyValueRow>
							<KeyValueRow label="변경 파일"><span class="flex items-center gap-1.5 font-medium"><FileDiff class="size-3.5 text-muted-foreground" />4개 · worktree와 함께 삭제됨</span></KeyValueRow>
						</div>
					{/if}
				</DialogBody>
				<DialogFooter>
					{#if f.phase === 'done'}
						<Button variant="outline" size="sm" disabled title="실행기 연동(#13) 후 로그를 볼 수 있어요"><Terminal />로그 보기</Button>
						<Button variant="outline" size="sm" onclick={startTest}><RotateCcw />다시 실행</Button>
						<Button size="sm" onclick={closeTest}>닫기</Button>
					{:else}
						<Button variant="ghost" size="sm" onclick={closeTest}>멈추고 닫기</Button>
					{/if}
				</DialogFooter>
			{/if}
		{/if}
	</DialogContent>
</Dialog>
