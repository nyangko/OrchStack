<script lang="ts">
	/// 범용 컴포넌트 검증용 갤러리 (#33). 각 Task가 아래에 섹션을 추가한다.
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Pill } from '$lib/components/orch/pill';
	import { Toggle } from '$lib/components/ui/toggle';
	import { Kbd } from '$lib/components/ui/kbd';
	import { TooltipProvider, Tooltip, TooltipTrigger, TooltipContent } from '$lib/components/ui/tooltip';
	import { Plus, Ellipsis, Bold, Users, Search, Sparkles } from '@lucide/svelte';
	import { Input } from '$lib/components/ui/input';
	import { InputGroup, InputGroupAddon, InputGroupInput } from '$lib/components/ui/input-group';
	import { Switch } from '$lib/components/ui/switch';
	import { Segmented } from '$lib/components/orch/segmented';
	import { Steps } from '$lib/components/orch/steps';
	import { Select, SelectTrigger, SelectContent, SelectItem } from '$lib/components/ui/select';
	import { FieldGroup, Field, FieldLabel, FieldDescription, FieldError, FieldContent } from '$lib/components/ui/field';
	import { StatusSelect } from '$lib/components/orch/status-select';
	import type { TaskStatus } from '$lib/status';
	import { SidebarProvider, Sidebar, SidebarContent, SidebarGroup, SidebarGroupLabel, SidebarMenu, SidebarMenuItem, SidebarMenuButton } from '$lib/components/ui/sidebar';
	import { Tabs, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
	import { Workflow, KanbanSquare, Sparkles as Spark, Plug } from '@lucide/svelte';
	import { Card, CardHeader, CardTitle, CardDescription, CardAction, CardContent } from '$lib/components/ui/card';
	import { Dialog, DialogTrigger, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogBody, DialogFooter, DialogClose } from '$lib/components/ui/dialog';
	import { DropdownMenu, DropdownMenuTrigger, DropdownMenuContent, DropdownMenuLabel, DropdownMenuItem, DropdownMenuShortcut, DropdownMenuSeparator } from '$lib/components/ui/dropdown-menu';
	import { ContextMenu, ContextMenuTrigger, ContextMenuContent, ContextMenuLabel, ContextMenuItem, ContextMenuSeparator } from '$lib/components/ui/context-menu';
	import { Alert, AlertTitle, AlertDescription } from '$lib/components/ui/alert';
	import { Progress } from '$lib/components/ui/progress';
	import { Spinner } from '$lib/components/ui/spinner';
	import { Empty, EmptyHeader, EmptyMedia, EmptyTitle, EmptyDescription } from '$lib/components/ui/empty';
	import { UserPlus, Play, Trash2, TriangleAlert, ShieldCheck, Inbox, Info, FileCode, Cpu } from '@lucide/svelte';
	import { AvatarGroup, AvatarGroupCount, Avatar, AvatarFallback } from '$lib/components/ui/avatar';
	import { RoleAvatar } from '$lib/components/orch/role-avatar';
	import { RuntimeLogo } from '$lib/components/orch/runtime-logo';
	import { StatusBadge } from '$lib/components/orch/status-badge';
	import { statusOrder } from '$lib/status';
	import { ItemGroup, Item, ItemContent, ItemDescription, ItemActions, ItemMedia, ItemTitle } from '$lib/components/ui/item';
	import { Table, TableHeader, TableRow, TableHead, TableBody, TableCell } from '$lib/components/ui/table';
	import { Puzzle, ArrowRightLeft } from '@lucide/svelte';

	// .pen 색 토큰 확인용. Tailwind 클래스는 동적으로 만들 수 없어 전체 이름으로 나열한다.
	const swatches = [
		{ name: 'background', cls: 'bg-background' },
		{ name: 'card', cls: 'bg-card' },
		{ name: 'muted', cls: 'bg-muted' },
		{ name: 'muted-strong', cls: 'bg-muted-strong' },
		{ name: 'border', cls: 'bg-border' },
		{ name: 'primary', cls: 'bg-primary' },
		{ name: 'primary-soft', cls: 'bg-primary-soft' },
		{ name: 'success', cls: 'bg-success' },
		{ name: 'warning', cls: 'bg-warning' },
		{ name: 'destructive', cls: 'bg-destructive' },
		{ name: 'status-review', cls: 'bg-status-review' },
		{ name: 'brand-claude', cls: 'bg-brand-claude' },
		{ name: 'brand-codex', cls: 'bg-brand-codex' }
	];

	const filters = [
		{ label: 'All', count: 12 },
		{ label: 'Backlog', count: 2 },
		{ label: 'In progress', count: 5 },
		{ label: 'Done', count: 5 }
	];
	let filter = $state('All');

	let on = $state(true);
	let seg = $state('a');
	let runtime = $state('codex');
	let notify = $state(true);
	let status = $state<TaskStatus>('in_progress');
</script>

<main class="mx-auto flex max-w-5xl flex-col gap-10 p-8">
	<header class="flex flex-col gap-1">
		<h1 class="text-2xl font-semibold">Components</h1>
		<p class="text-sm text-muted-foreground">
			범용 UI 컴포넌트 갤러리 — <span class="font-mono">design/OrchStack Alpha UI.pen</span> 과 대조
		</p>
	</header>

	<section class="flex flex-col gap-3">
		<h2 class="text-sm font-semibold text-muted-foreground">Foundation · Colors</h2>
		<div class="grid grid-cols-4 gap-3 sm:grid-cols-6">
			{#each swatches as s (s.name)}
				<div class="flex flex-col gap-1.5">
					<div class="h-10 rounded-md border {s.cls}"></div>
					<span class="font-mono text-xs text-muted-foreground">{s.name}</span>
				</div>
			{/each}
		</div>
	</section>

	<section class="flex flex-col gap-3">
		<h2 class="text-sm font-semibold text-muted-foreground">Foundation · Type & Radius</h2>
		<p class="font-sans text-base">Pretendard — 에이전트 오케스트레이션 0123456789</p>
		<pre class="font-mono text-sm leading-normal">Casquare Mono — 400 <b class="font-semibold">600</b> <b class="font-bold">700</b>
|abcdefgh| src/lib/status.ts:12
|가나다라| src/lib/상태.ts:12 에이전트 로그
|=> != 0O|</pre>
		<div class="flex gap-3">
			<div class="size-12 rounded-xs bg-muted-strong"></div>
			<div class="size-12 rounded-sm bg-muted-strong"></div>
			<div class="size-12 rounded-md bg-muted-strong"></div>
			<div class="size-12 rounded-lg bg-muted-strong"></div>
			<div class="size-12 rounded-xl bg-muted-strong"></div>
		</div>
	</section>

	<section class="flex flex-col gap-4">
		<h2 class="text-sm font-semibold text-muted-foreground">Actions & Tags · #35</h2>
		<div class="flex flex-wrap items-center gap-3">
			<Button><Plus />Default</Button>
			<Button variant="outline"><Plus />Outline</Button>
			<Button variant="ghost"><Plus />Ghost</Button>
			<Button variant="link">Link</Button>
			<Button variant="ghost" size="icon" aria-label="More"><Ellipsis /></Button>
			<Button variant="ghost" size="icon-xs" class="text-muted-foreground" aria-label="Bold"><Bold /></Button>
			<Button disabled>Disabled</Button>
		</div>
		<div class="flex flex-wrap items-center gap-3">
			<Badge variant="secondary"><Users />secondary</Badge>
			<Badge variant="outline">outline</Badge>
			<Pill>Pill</Pill>
			<Pill dot="bg-status-done">Done</Pill>
			<Kbd>⌘K</Kbd>
			<TooltipProvider>
				<Tooltip>
					<TooltipTrigger>
						{#snippet child({ props })}
							<Button variant="outline" size="sm" {...props}>Hover</Button>
						{/snippet}
					</TooltipTrigger>
					<TooltipContent>Agents</TooltipContent>
				</Tooltip>
			</TooltipProvider>
		</div>
		<div class="flex flex-wrap items-center gap-1.5">
			{#each filters as f (f.label)}
				<Toggle variant="chip" count={f.count} pressed={filter === f.label} onPressedChange={() => (filter = f.label)}>{f.label}</Toggle>
			{/each}
		</div>
	</section>
	<section class="flex flex-col gap-4">
		<h2 class="text-sm font-semibold text-muted-foreground">Form Controls · #36</h2>
		<div class="flex flex-wrap items-center gap-3">
			<InputGroup class="w-70">
				<InputGroupAddon><Search /></InputGroupAddon>
				<InputGroupInput placeholder="Search tasks, agents, issues" />
			</InputGroup>
			<Input class="w-60" placeholder="Plain input" />
			<Input class="w-40" placeholder="Invalid" aria-invalid="true" />
			<Switch bind:checked={on} aria-label="Toggle" />
			<Switch aria-label="Off" />
			<StatusSelect bind:value={status} />
		</div>
		<Segmented
			class="w-80"
			bind:value={seg}
			options={[
				{ value: 'a', label: 'Opt1', icon: Sparkles },
				{ value: 'b', label: 'Opt2', icon: Sparkles },
				{ value: 'c', label: 'Opt3', icon: Sparkles }
			]}
		/>
		<Steps steps={['템플릿 선택', '캐릭터', '런타임 · 도구']} current={1} />
		<FieldGroup class="max-w-3xl gap-0">
			<Field class="border-b py-4">
				<FieldLabel for="runtime">실행기</FieldLabel>
				<Select type="single" bind:value={runtime}>
					<SelectTrigger id="runtime" class="w-full">
						{runtime === 'codex' ? 'Codex CLI' : 'Claude Code'}
					</SelectTrigger>
					<SelectContent>
						<SelectItem value="codex" label="Codex CLI" />
						<SelectItem value="claude" label="Claude Code" />
					</SelectContent>
				</Select>
				<FieldDescription>에이전트가 도는 CLI</FieldDescription>
			</Field>
			<Field class="border-b py-4" data-invalid="true">
				<FieldLabel for="name">이름</FieldLabel>
				<Input id="name" aria-invalid="true" />
				<FieldError>이름을 입력하세요</FieldError>
			</Field>
			<Field orientation="horizontal" class="border-b py-2.5">
				<FieldContent>
					<FieldLabel for="notify">알림</FieldLabel>
					<FieldDescription>작업 완료 시 알림을 보냅니다</FieldDescription>
				</FieldContent>
				<Switch id="notify" bind:checked={notify} />
			</Field>
		</FieldGroup>
	</section>
	<section class="flex flex-col gap-4">
		<h2 class="text-sm font-semibold text-muted-foreground">Navigation & Shell · #37</h2>
		<div class="flex gap-8">
			<SidebarProvider class="min-h-0 w-60">
				<Sidebar collapsible="none" class="w-60 border-r">
					<SidebarContent>
						<SidebarGroup>
							<SidebarGroupLabel>보기</SidebarGroupLabel>
							<SidebarMenu>
								{#each [{ l: 'Overview', i: Spark, a: true }, { l: '모델 연결', i: Plug, a: false }, { l: '실행기 (CLI)', i: Workflow, a: false }] as n (n.l)}
									<SidebarMenuItem>
										<SidebarMenuButton isActive={n.a}>
											{#snippet child({ props })}
												<a href="/spike/components" {...props}><n.i />{n.l}</a>
											{/snippet}
										</SidebarMenuButton>
									</SidebarMenuItem>
								{/each}
							</SidebarMenu>
						</SidebarGroup>
					</SidebarContent>
				</Sidebar>
			</SidebarProvider>
			<div class="flex flex-col gap-6">
				<Tabs value="diagram">
					<TabsList>
						<TabsTrigger value="diagram"><Workflow />Diagram</TabsTrigger>
						<TabsTrigger value="kanban"><KanbanSquare />Kanban</TabsTrigger>
						<TabsTrigger value="issues">Issues</TabsTrigger>
					</TabsList>
				</Tabs>
				<Tabs value="overview">
					<TabsList variant="line">
						<TabsTrigger value="overview">Overview</TabsTrigger>
						<TabsTrigger value="installed">설치됨 5</TabsTrigger>
						<TabsTrigger value="runs">Runs</TabsTrigger>
					</TabsList>
				</Tabs>
			</div>
		</div>
	</section>

	<section class="flex flex-col gap-4">
		<h2 class="text-sm font-semibold text-muted-foreground">Containers & Feedback · #38</h2>
		<div class="grid grid-cols-2 gap-4">
			<Card>
				<CardHeader>
					<CardTitle>Card Title</CardTitle>
					<CardDescription>Card description</CardDescription>
				</CardHeader>
			</Card>
			<Card size="sm">
				<CardHeader>
					<CardTitle>Section title</CardTitle>
					<CardDescription>Description</CardDescription>
					<CardAction><Button variant="link" size="xs" class="h-auto p-0">Action</Button></CardAction>
				</CardHeader>
				<CardContent class="text-xs text-subtle-foreground">Content</CardContent>
			</Card>
		</div>
		<div class="flex flex-wrap items-center gap-3">
			<Dialog>
				<DialogTrigger>
					{#snippet child({ props })}<Button variant="outline" {...props}>Open dialog</Button>{/snippet}
				</DialogTrigger>
				<DialogContent size="md">
					<DialogHeader icon={Plug} crumb={['Settings', '모델 연결']}>
						<DialogTitle>연결 추가</DialogTitle>
						<DialogDescription>에이전트가 모델을 부를 경로 · 워크스페이스 전체에서 사용</DialogDescription>
					</DialogHeader>
					<DialogBody><Input placeholder="이름" aria-label="이름" /></DialogBody>
					<DialogFooter note="계정 · 키를 하나 더 추가할 수 있어요">
						<DialogClose>{#snippet child({ props })}<Button variant="ghost" {...props}>취소</Button>{/snippet}</DialogClose>
						<Button>다음</Button>
					</DialogFooter>
				</DialogContent>
			</Dialog>
			<DropdownMenu>
				<DropdownMenuTrigger>
					{#snippet child({ props })}<Button variant="outline" {...props}>Menu</Button>{/snippet}
				</DropdownMenuTrigger>
				<DropdownMenuContent class="w-52">
					<DropdownMenuLabel>Task #428</DropdownMenuLabel>
					<DropdownMenuItem><UserPlus />Assign agent</DropdownMenuItem>
					<DropdownMenuItem><Play />Run<DropdownMenuShortcut>⌘R</DropdownMenuShortcut></DropdownMenuItem>
					<DropdownMenuSeparator />
					<DropdownMenuItem variant="destructive"><Trash2 />Delete</DropdownMenuItem>
				</DropdownMenuContent>
			</DropdownMenu>
			<ContextMenu>
				<ContextMenuTrigger class="flex h-9 items-center rounded-md border border-dashed px-3 text-xs text-muted-foreground">우클릭</ContextMenuTrigger>
				<ContextMenuContent class="w-52">
					<ContextMenuLabel>Task #428</ContextMenuLabel>
					<ContextMenuItem><UserPlus />Assign agent</ContextMenuItem>
					<ContextMenuSeparator />
					<ContextMenuItem variant="destructive"><Trash2 />Delete</ContextMenuItem>
				</ContextMenuContent>
			</ContextMenu>
			<Spinner />
		</div>
		<Alert variant="warning">
			<TriangleAlert />
			<AlertTitle>Repeated context detected</AlertTitle>
			<AlertDescription>3 sources re-sent across the last 4 calls.</AlertDescription>
		</Alert>
		<Alert variant="info">
			<ShieldCheck class="text-status-done" />
			<AlertTitle>적용 권한</AlertTitle>
			<AlertDescription>목록에 없는 도구는 차단 · Instructions는 목록을 줄일 수만 있어요</AlertDescription>
		</Alert>
		<Progress value={68} class="w-50" />
		<Empty class="border">
			<EmptyHeader>
				<EmptyMedia variant="icon"><Inbox /></EmptyMedia>
				<EmptyTitle>태스크가 없어요</EmptyTitle>
				<EmptyDescription>새 태스크를 만들거나 이슈에서 가져오세요.</EmptyDescription>
			</EmptyHeader>
		</Empty>
	</section>

	<section class="flex flex-col gap-4">
		<h2 class="text-sm font-semibold text-muted-foreground">Identity & Status · #39</h2>
		<div class="flex flex-wrap items-center gap-3">
			<RoleAvatar role="frontend" size="sm" />
			<RoleAvatar role="backend" />
			<RoleAvatar role="qa" size="lg" />
			<RoleAvatar role="reviewer" />
			<RoleAvatar role="designer" />
			<RoleAvatar role="agent" />
			<AvatarGroup>
				<RoleAvatar role="backend" size="sm" />
				<RoleAvatar role="frontend" size="sm" />
				<RoleAvatar role="qa" size="sm" />
				<RoleAvatar role="reviewer" size="sm" />
				<AvatarGroupCount>+2</AvatarGroupCount>
			</AvatarGroup>
			<Avatar><AvatarFallback class="bg-primary-soft font-semibold text-primary">S</AvatarFallback></Avatar>
			<RuntimeLogo runtime="codex" />
			<RuntimeLogo runtime="claude" />
			<RuntimeLogo runtime="claude" class="size-8" />
		</div>
		<div class="flex flex-wrap items-center gap-2">
			{#each statusOrder as s (s)}<StatusBadge status={s} />{/each}
		</div>
		<div class="flex flex-wrap items-center gap-2">
			<Badge variant="mono"><FileCode />src/lib/status.ts</Badge>
			<Badge variant="mono"><Cpu />gpt-5</Badge>
		</div>
	</section>

	<section class="flex flex-col gap-4">
		<h2 class="text-sm font-semibold text-muted-foreground">Data Display · #40</h2>
		<div class="grid grid-cols-2 gap-8">
			<ItemGroup>
				<Item variant="row" size="xs">
					<ItemContent><ItemDescription>역할</ItemDescription></ItemContent>
					<ItemActions class="text-body font-medium">프론트엔드 개발</ItemActions>
				</Item>
				<Item variant="row" size="xs">
					<ItemMedia>
						<span class="flex size-5 items-center justify-center rounded-xs bg-node-skill text-on-solid"><Puzzle class="size-3" /></span>
					</ItemMedia>
					<ItemContent><ItemTitle>Rust Backend</ItemTitle></ItemContent>
					<ItemActions class="font-mono text-xs text-muted-foreground">8 calls</ItemActions>
				</Item>
				<Item variant="row" size="xs">
					<ItemMedia>
						<span class="flex size-6 items-center justify-center rounded-full bg-review-soft text-status-review"><ArrowRightLeft class="size-3" /></span>
					</ItemMedia>
					<ItemContent>
						<ItemTitle>진 → 하루 · 전달</ItemTitle>
						<ItemDescription>Login 화면 시안 v2 전달 · figma/login-v2 — 설명이 길어지면 두 줄에서 잘립니다. 설명이 길어지면 두 줄에서 잘립니다.</ItemDescription>
					</ItemContent>
				</Item>
			</ItemGroup>
			<Table>
				<TableHeader>
					<TableRow><TableHead>Task</TableHead><TableHead>Status</TableHead><TableHead class="text-right">Tokens</TableHead></TableRow>
				</TableHeader>
				<TableBody>
					<TableRow><TableCell>Login 화면</TableCell><TableCell><StatusBadge status="review" /></TableCell><TableCell class="text-right font-mono">42.3K</TableCell></TableRow>
					<TableRow><TableCell>토큰 원장</TableCell><TableCell><StatusBadge status="in_progress" /></TableCell><TableCell class="text-right font-mono">1.2K</TableCell></TableRow>
					<TableRow><TableCell>빈 값</TableCell><TableCell><StatusBadge status="todo" /></TableCell><TableCell class="text-right font-mono text-subtle-foreground">—</TableCell></TableRow>
				</TableBody>
			</Table>
		</div>
	</section>
</main>
