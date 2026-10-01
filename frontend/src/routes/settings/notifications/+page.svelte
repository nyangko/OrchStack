<script lang="ts">
	/// Settings › 알림 — 이벤트 × 채널 · 채널 상태 · 방해 금지 · 일일 요약 (.pen Settings · 알림).
	/// 바꾸면 바로 저장(목데이터)되고 헤더에 저장됨이 뜬다. 실제 발송은 서버 연결(#47) 후.
	import { onDestroy, type Component } from 'svelte';
	import Send from '@lucide/svelte/icons/send';
	import Bell from '@lucide/svelte/icons/bell';
	import Monitor from '@lucide/svelte/icons/monitor';
	import Mail from '@lucide/svelte/icons/mail';
	import Settings2 from '@lucide/svelte/icons/settings-2';
	import Moon from '@lucide/svelte/icons/moon';
	import Clock from '@lucide/svelte/icons/clock';
	import CircleCheck from '@lucide/svelte/icons/circle-check';
	import * as Card from '$lib/components/ui/card';
	import * as Select from '$lib/components/ui/select';
	import { Button } from '$lib/components/ui/button';
	import { PageHeader } from '$lib/components/ui/page-header';
	import { Pill } from '$lib/components/ui/pill';
	import { Switch } from '$lib/components/ui/switch';
	import * as Field from '$lib/components/ui/field';
	import { notifyEvents, notifyChannels, notifyQuiet, type NotifyChannel } from '$lib/mock';

	let events = $state(structuredClone(notifyEvents));
	let quiet = $state({ ...notifyQuiet });
	let saved = $state(false);
	/// 테스트 알림을 보낸 채널 (잠깐 "보냈어요" 표시).
	let sent = $state<NotifyChannel | 'all'>();
	let timer: ReturnType<typeof setTimeout> | undefined;
	onDestroy(() => clearTimeout(timer));

	const icon: Record<NotifyChannel, Component> = { app: Bell, desktop: Monitor, telegram: Send, email: Mail };
	/// 채널 버튼 — 앱은 늘 켜져 있어 버튼이 없다. 변경 · 연결 화면은 .pen에 아직 없음.
	const action: Partial<Record<NotifyChannel, string>> = { desktop: '테스트', telegram: '변경', email: '연결' };
	const channels = notifyChannels.map((c) => c.key);
	const digestOn = $derived(events.flatMap((g) => g.items).find((e) => e.name === '일일 요약'));

	function test(key: NotifyChannel | 'all') {
		sent = key;
		clearTimeout(timer);
		timer = setTimeout(() => (sent = undefined), 2000);
	}
</script>

<svelte:head><title>알림 · Settings · OrchStack</title></svelte:head>

<main class="page-main">
	<PageHeader title="알림" desc="어떤 일을 · 어디로 · 언제 알릴지" status={sent === 'all' ? '연결된 채널로 보냈어요' : saved && '저장됨'} statusIcon={sent === 'all' ? Send : CircleCheck}>
		<Button variant="outline" onclick={() => test('all')}><Send />테스트 알림 보내기</Button>
	</PageHeader>

	<div class="flex items-start gap-5">
		<Card.Root size="sm" class="min-w-0 flex-1">
			<Card.Header>
				<Card.Title>이벤트별 알림</Card.Title>
				<Card.Description>무엇을 어디로 보낼지 · 멤버 · 연결별 설정이 있으면 그쪽이 우선</Card.Description>
			</Card.Header>
			<Card.Content class="gap-0">
				<div class="matrix-head">
					<span class="flex-1">이벤트</span>
					{#each notifyChannels as c (c.key)}<span class="w-18 text-center">{c.name}</span>{/each}
				</div>
				{#each events as g (g.group)}
					<h3 class="section-label pt-4 pb-1">{g.group}</h3>
					{#each g.items as ev (ev.name)}
						<div class="flex items-center border-b py-2.5">
							<span class="flex flex-1 flex-col gap-0.5">
								<span class="text-xs font-semibold">{ev.name}</span>
								{#if ev.desc}<span class="text-caption text-muted-foreground">{ev.desc}</span>{/if}
							</span>
							{#each channels as ch (ch)}
								<span class="flex w-18 justify-center">
									<Switch bind:checked={() => ev.on[ch], (v) => ((ev.on[ch] = v), (saved = true))} aria-label="{ev.name} · {notifyChannels.find((c) => c.key === ch)?.name}" />
								</span>
							{/each}
						</div>
					{/each}
				{/each}
			</Card.Content>
		</Card.Root>

		<div class="aside-col w-95 gap-5">
			<Card.Root size="sm">
				<Card.Header><Card.Title>채널</Card.Title></Card.Header>
				<Card.Content class="flex flex-col gap-2">
					{#each notifyChannels as c (c.key)}
						{@const Icon = icon[c.key]}
						<div class="option-card rounded-md">
							<span class="icon-tile"><Icon class="size-4" /></span>
							<span class="col-fill gap-1">
								<span class="row-title-strong">
									{c.name}<Pill dot={c.ok ? 'bg-status-done' : 'bg-muted-foreground'} class={c.ok ? 'bg-success-soft text-status-done' : ''}>{c.state}</Pill>
								</span>
								<span class="truncate text-caption text-muted-foreground">{sent === c.key ? '테스트 알림을 보냈어요' : c.desc}</span>
							</span>
							{#if action[c.key]}
								<Button variant="outline" size="sm" onclick={() => c.key === 'desktop' && test('desktop')}><Settings2 />{action[c.key]}</Button>
							{/if}
						</div>
					{/each}
				</Card.Content>
			</Card.Root>

			<Card.Root size="sm">
				<Card.Header><Card.Title>방해 금지</Card.Title></Card.Header>
				<Card.Content>
					<Field.Row label="시간" hint="이 시간에는 앱 배지만 쌓여요">
						<Select.Root type="single" bind:value={() => quiet.hours, (v) => ((quiet.hours = v), (saved = true))}>
							<Select.Trigger class="w-full" aria-label="방해 금지 시간">
								<span class="row-fill">
									<Moon class="size-4 text-muted-foreground" />{quiet.hours}
									<span class="truncate text-caption font-normal text-muted-foreground">{quiet.weekend ? '주말 전체 포함' : ''}</span>
								</span>
							</Select.Trigger>
							<Select.Content>
								{#each ['22:00 – 08:00', '23:00 – 07:00', '끄기'] as o (o)}<Select.Item value={o} label={o} />{/each}
							</Select.Content>
						</Select.Root>
					</Field.Row>
					<Field.SwitchRow label="L3 이상은 방해 금지 무시" hint="외부 영향 · 위험 판단, 연결 오류" bind:checked={() => quiet.urgentBypass, (v) => ((quiet.urgentBypass = v), (saved = true))} />
				</Card.Content>
			</Card.Root>

			<Card.Root size="sm">
				<Card.Header><Card.Title>일일 요약</Card.Title></Card.Header>
				<Card.Content>
					<Field.Row label="보내는 시각" hint="완료 · 실패 · 토큰 · 비용 요약">
						<Select.Root type="single" bind:value={() => quiet.digest, (v) => ((quiet.digest = v), (saved = true))}>
							<Select.Trigger class="w-full" aria-label="일일 요약 시각">
								<span class="row-fill">
									<Clock class="size-4 text-muted-foreground" />{quiet.digest}
									<!-- 일일 요약 이벤트에서 켠 채널 -->
									<span class="truncate text-caption font-normal text-muted-foreground">
										{notifyChannels.filter((c) => digestOn?.on[c.key]).map((c) => c.name).join(' + ')}
									</span>
								</span>
							</Select.Trigger>
							<Select.Content>
								{#each ['매일 09:00', '매일 18:00', '평일 18:00', '보내지 않음'] as o (o)}<Select.Item value={o} label={o} />{/each}
							</Select.Content>
						</Select.Root>
					</Field.Row>
				</Card.Content>
			</Card.Root>
		</div>
	</div>
</main>
