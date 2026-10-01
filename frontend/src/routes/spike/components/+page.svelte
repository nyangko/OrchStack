<script lang="ts">
	/// 범용 컴포넌트 검증용 갤러리 (#33). 각 Task가 아래에 섹션을 추가한다.
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Pill } from '$lib/components/ui/pill';
	import { Toggle } from '$lib/components/ui/toggle';
	import { Kbd } from '$lib/components/ui/kbd';
	import * as Tooltip from '$lib/components/ui/tooltip';
	import { Plus, Ellipsis, Bold, Users, Search, Sparkles } from '@lucide/svelte';
	import { Input } from '$lib/components/ui/input';
	import * as InputGroup from '$lib/components/ui/input-group';
	import { Switch } from '$lib/components/ui/switch';
	import { Segmented } from '$lib/components/ui/segmented';
	import { Steps } from '$lib/components/ui/steps';
	import * as Select from '$lib/components/ui/select';
	import * as Field from '$lib/components/ui/field';
	import { StatusSelect } from '$lib/components/ui/status-select';
	import type { TaskStatus } from '$lib/status';
	import * as Sidebar from '$lib/components/ui/sidebar';
	import * as Tabs from '$lib/components/ui/tabs';
	import { Workflow, KanbanSquare, Sparkles as Spark, Plug } from '@lucide/svelte';
	import * as Card from '$lib/components/ui/card';
	import * as Dialog from '$lib/components/ui/dialog';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
	import * as ContextMenu from '$lib/components/ui/context-menu';
	import * as Alert from '$lib/components/ui/alert';
	import { Progress } from '$lib/components/ui/progress';
	import { Spinner } from '$lib/components/ui/spinner';
	import * as Empty from '$lib/components/ui/empty';
	import { UserPlus, Play, Trash2, TriangleAlert, ShieldCheck, Inbox, Info, FileCode, Cpu } from '@lucide/svelte';
	import * as Avatar from '$lib/components/ui/avatar';
	import { RoleAvatar } from '$lib/components/ui/role-avatar';
	import { RuntimeLogo } from '$lib/components/ui/runtime-logo';
	import { StatusBadge } from '$lib/components/ui/status-badge';
	import { statusOrder } from '$lib/status';
	import * as Item from '$lib/components/ui/item';
	import * as Table from '$lib/components/ui/table';
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
			<Tooltip.Provider>
				<Tooltip.Root>
					<Tooltip.Trigger>
						{#snippet child({ props })}
							<Button variant="outline" size="sm" {...props}>Hover</Button>
						{/snippet}
					</Tooltip.Trigger>
					<Tooltip.Content>Agents</Tooltip.Content>
				</Tooltip.Root>
			</Tooltip.Provider>
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
			<InputGroup.Root class="w-70">
				<InputGroup.Addon><Search /></InputGroup.Addon>
				<InputGroup.Input placeholder="Search tasks, agents, issues" />
			</InputGroup.Root>
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
		<Field.Group class="max-w-3xl gap-0">
			<Field.Field class="border-b py-4">
				<Field.Label for="runtime">실행기</Field.Label>
				<Select.Root type="single" bind:value={runtime}>
					<Select.Trigger id="runtime" class="w-full">
						{runtime === 'codex' ? 'Codex CLI' : 'Claude Code'}
					</Select.Trigger>
					<Select.Content>
						<Select.Item value="codex" label="Codex CLI" />
						<Select.Item value="claude" label="Claude Code" />
					</Select.Content>
				</Select.Root>
				<Field.Description>에이전트가 도는 CLI</Field.Description>
			</Field.Field>
			<Field.Field class="border-b py-4" data-invalid="true">
				<Field.Label for="name">이름</Field.Label>
				<Input id="name" aria-invalid="true" />
				<Field.Error>이름을 입력하세요</Field.Error>
			</Field.Field>
			<Field.Field orientation="horizontal" class="border-b py-2.5">
				<Field.Content>
					<Field.Label for="notify">알림</Field.Label>
					<Field.Description>작업 완료 시 알림을 보냅니다</Field.Description>
				</Field.Content>
				<Switch id="notify" bind:checked={notify} />
			</Field.Field>
		</Field.Group>
	</section>
	<section class="flex flex-col gap-4">
		<h2 class="text-sm font-semibold text-muted-foreground">Navigation & Shell · #37</h2>
		<div class="flex gap-8">
			<Sidebar.Provider class="min-h-0 w-60">
				<Sidebar.Root collapsible="none" class="w-60 border-r">
					<Sidebar.Content>
						<Sidebar.Group>
							<Sidebar.GroupLabel>보기</Sidebar.GroupLabel>
							<Sidebar.Menu>
								{#each [{ l: 'Overview', i: Spark, a: true }, { l: '모델 연결', i: Plug, a: false }, { l: '실행기 (CLI)', i: Workflow, a: false }] as n (n.l)}
									<Sidebar.MenuItem>
										<Sidebar.MenuButton isActive={n.a}>
											{#snippet child({ props })}
												<a href="/spike/components" {...props}><n.i />{n.l}</a>
											{/snippet}
										</Sidebar.MenuButton>
									</Sidebar.MenuItem>
								{/each}
							</Sidebar.Menu>
						</Sidebar.Group>
					</Sidebar.Content>
				</Sidebar.Root>
			</Sidebar.Provider>
			<div class="flex flex-col gap-6">
				<Tabs.Root value="diagram">
					<Tabs.List>
						<Tabs.Trigger value="diagram"><Workflow />Diagram</Tabs.Trigger>
						<Tabs.Trigger value="kanban"><KanbanSquare />Kanban</Tabs.Trigger>
						<Tabs.Trigger value="issues">Issues</Tabs.Trigger>
					</Tabs.List>
				</Tabs.Root>
				<Tabs.Root value="overview">
					<Tabs.List variant="line">
						<Tabs.Trigger value="overview">Overview</Tabs.Trigger>
						<Tabs.Trigger value="installed">설치됨 5</Tabs.Trigger>
						<Tabs.Trigger value="runs">Runs</Tabs.Trigger>
					</Tabs.List>
				</Tabs.Root>
			</div>
		</div>
	</section>

	<section class="flex flex-col gap-4">
		<h2 class="text-sm font-semibold text-muted-foreground">Containers & Feedback · #38</h2>
		<div class="grid grid-cols-2 gap-4">
			<Card.Root>
				<Card.Header>
					<Card.Title>Card Title</Card.Title>
					<Card.Description>Card description</Card.Description>
				</Card.Header>
			</Card.Root>
			<Card.Root size="sm">
				<Card.Header>
					<Card.Title>Section title</Card.Title>
					<Card.Description>Description</Card.Description>
					<Card.Action><Button variant="link" size="xs" class="h-auto p-0">Action</Button></Card.Action>
				</Card.Header>
				<Card.Content class="text-xs text-subtle-foreground">Content</Card.Content>
			</Card.Root>
		</div>
		<div class="flex flex-wrap items-center gap-3">
			<Dialog.Root>
				<Dialog.Trigger>
					{#snippet child({ props })}<Button variant="outline" {...props}>Open dialog</Button>{/snippet}
				</Dialog.Trigger>
				<Dialog.Content size="md">
					<Dialog.Header icon={Plug} crumb={['Settings', '모델 연결']}>
						<Dialog.Title>연결 추가</Dialog.Title>
						<Dialog.Description>에이전트가 모델을 부를 경로 · 워크스페이스 전체에서 사용</Dialog.Description>
					</Dialog.Header>
					<Dialog.Body><Input placeholder="이름" aria-label="이름" /></Dialog.Body>
					<Dialog.Footer note="계정 · 키를 하나 더 추가할 수 있어요">
						<Dialog.Close>{#snippet child({ props })}<Button variant="ghost" {...props}>취소</Button>{/snippet}</Dialog.Close>
						<Button>다음</Button>
					</Dialog.Footer>
				</Dialog.Content>
			</Dialog.Root>
			<DropdownMenu.Root>
				<DropdownMenu.Trigger>
					{#snippet child({ props })}<Button variant="outline" {...props}>Menu</Button>{/snippet}
				</DropdownMenu.Trigger>
				<DropdownMenu.Content class="w-52">
					<DropdownMenu.Label>Task #428</DropdownMenu.Label>
					<DropdownMenu.Item><UserPlus />Assign agent</DropdownMenu.Item>
					<DropdownMenu.Item><Play />Run<DropdownMenu.Shortcut>⌘R</DropdownMenu.Shortcut></DropdownMenu.Item>
					<DropdownMenu.Separator />
					<DropdownMenu.Item variant="destructive"><Trash2 />Delete</DropdownMenu.Item>
				</DropdownMenu.Content>
			</DropdownMenu.Root>
			<ContextMenu.Root>
				<ContextMenu.Trigger class="flex h-9 items-center rounded-md border border-dashed px-3 text-xs text-muted-foreground">우클릭</ContextMenu.Trigger>
				<ContextMenu.Content class="w-52">
					<ContextMenu.Label>Task #428</ContextMenu.Label>
					<ContextMenu.Item><UserPlus />Assign agent</ContextMenu.Item>
					<ContextMenu.Separator />
					<ContextMenu.Item variant="destructive"><Trash2 />Delete</ContextMenu.Item>
				</ContextMenu.Content>
			</ContextMenu.Root>
			<Spinner />
		</div>
		<Alert.Root variant="warning">
			<TriangleAlert />
			<Alert.Title>Repeated context detected</Alert.Title>
			<Alert.Description>3 sources re-sent across the last 4 calls.</Alert.Description>
		</Alert.Root>
		<Alert.Root variant="info">
			<ShieldCheck class="text-status-done" />
			<Alert.Title>적용 권한</Alert.Title>
			<Alert.Description>목록에 없는 도구는 차단 · Instructions는 목록을 줄일 수만 있어요</Alert.Description>
		</Alert.Root>
		<Progress value={68} class="w-50" />
		<Empty.Root class="border">
			<Empty.Header>
				<Empty.Media variant="icon"><Inbox /></Empty.Media>
				<Empty.Title>태스크가 없어요</Empty.Title>
				<Empty.Description>새 태스크를 만들거나 이슈에서 가져오세요.</Empty.Description>
			</Empty.Header>
		</Empty.Root>
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
			<Avatar.Group>
				<RoleAvatar role="backend" size="sm" />
				<RoleAvatar role="frontend" size="sm" />
				<RoleAvatar role="qa" size="sm" />
				<RoleAvatar role="reviewer" size="sm" />
				<Avatar.GroupCount>+2</Avatar.GroupCount>
			</Avatar.Group>
			<Avatar.Root><Avatar.Fallback class="bg-primary-soft font-semibold text-primary">S</Avatar.Fallback></Avatar.Root>
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
			<Item.Group>
				<Item.Root variant="row" size="xs">
					<Item.Content><Item.Description>역할</Item.Description></Item.Content>
					<Item.Actions class="text-body font-medium">프론트엔드 개발</Item.Actions>
				</Item.Root>
				<Item.Root variant="row" size="xs">
					<Item.Media>
						<span class="flex size-5 items-center justify-center rounded-xs bg-node-skill text-on-solid"><Puzzle class="size-3" /></span>
					</Item.Media>
					<Item.Content><Item.Title>Rust Backend</Item.Title></Item.Content>
					<Item.Actions class="font-mono text-xs text-muted-foreground">8 calls</Item.Actions>
				</Item.Root>
				<Item.Root variant="row" size="xs">
					<Item.Media>
						<span class="flex size-6 items-center justify-center rounded-full bg-review-soft text-status-review"><ArrowRightLeft class="size-3" /></span>
					</Item.Media>
					<Item.Content>
						<Item.Title>진 → 하루 · 전달</Item.Title>
						<Item.Description>Login 화면 시안 v2 전달 · figma/login-v2 — 설명이 길어지면 두 줄에서 잘립니다. 설명이 길어지면 두 줄에서 잘립니다.</Item.Description>
					</Item.Content>
				</Item.Root>
			</Item.Group>
			<Table.Root>
				<Table.Header>
					<Table.Row><Table.Head>Task</Table.Head><Table.Head>Status</Table.Head><Table.Head class="text-right">Tokens</Table.Head></Table.Row>
				</Table.Header>
				<Table.Body>
					<Table.Row><Table.Cell>Login 화면</Table.Cell><Table.Cell><StatusBadge status="review" /></Table.Cell><Table.Cell class="text-right font-mono">42.3K</Table.Cell></Table.Row>
					<Table.Row><Table.Cell>토큰 원장</Table.Cell><Table.Cell><StatusBadge status="in_progress" /></Table.Cell><Table.Cell class="text-right font-mono">1.2K</Table.Cell></Table.Row>
					<Table.Row><Table.Cell>빈 값</Table.Cell><Table.Cell><StatusBadge status="todo" /></Table.Cell><Table.Cell class="text-right font-mono text-subtle-foreground">—</Table.Cell></Table.Row>
				</Table.Body>
			</Table.Root>
		</div>
	</section>
</main>
