<script lang="ts" module>
	/// 결정 한 건 — 내가 답했거나(me) 시간이 지나 Orch가 사용자 대신 정한 것(orch).
	export type DecisionRecordData = {
		by: 'me' | 'orch';
		task: number;
		time: string;
		q: string;
		a: string;
		/** 한 줄 요약 (예: #133 재전송 → 30초 쿨다운) */
		summary: string;
		/** 전달 받은 쪽 (예: 진에게 전달됨 · Run #84) */
		sent: string;
		/** Orch가 대신 정한 근거 */
		why?: string;
		/** Orch 결정을 사용자가 확인했나 */
		confirmed?: boolean;
	};
</script>

<script lang="ts">
	/**
	 * **결정 기록** (.pen Decision Record · DecisionRecord/compact · compact-orch).
	 * 타임라인에서는 한 줄, 누르면 Q · A(· 근거) 카드로 펼친다. Task 상세 · 멤버 Activity가 같이 쓴다.
	 */
	import BookMarked from '@lucide/svelte/icons/book-marked';
	import User from '@lucide/svelte/icons/user';
	import Sparkles from '@lucide/svelte/icons/sparkles';
	import Send from '@lucide/svelte/icons/send';
	import MessageSquareText from '@lucide/svelte/icons/message-square-text';
	import RotateCcw from '@lucide/svelte/icons/rotate-ccw';
	import Check from '@lucide/svelte/icons/check';
	import { Button } from '$lib/components/ui/button';

	let {
		record,
		/** 판단 패널에서 다시 보기 (내 결정 "답변 전문" · Orch 결정 "재검토") — 없으면 그 버튼을 숨긴다 */
		onreview
	}: { record: DecisionRecordData; onreview?: () => void } = $props();

	let open = $state(false);
	const orch = $derived(record.by === 'orch');
</script>

{#if !open}
	<button
		type="button"
		aria-expanded="false"
		onclick={() => (open = true)}
		class={['flex w-full items-center gap-2.5 rounded-md border px-3 py-2.5 text-left text-xs outline-none focus-visible:ring-3 focus-visible:ring-ring/50', orch ? 'border-status-waiting bg-warning-soft' : 'bg-card']}
	>
		<BookMarked class={['size-3.5 shrink-0', orch ? 'text-status-waiting' : 'text-primary']} />
		<span class="font-semibold">{orch ? 'Orch' : '나'}</span>
		<span class={['rounded-xs px-1.5 py-px text-caption font-medium', orch ? 'bg-status-waiting text-on-solid' : 'bg-muted text-muted-foreground']}>{orch ? '대신 결정' : '결정'}</span>
		<span class="min-w-0 flex-1 truncate">{record.summary}</span>
		<span class="font-mono text-caption text-subtle-foreground">{record.time}</span>
	</button>
{:else}
	<article class={['flex flex-col gap-2.5 rounded-lg border p-3.5 text-xs', orch ? 'border-status-waiting bg-warning-soft' : 'border-primary bg-card']}>
		<button type="button" aria-expanded="true" onclick={() => (open = false)} class="flex items-center gap-2 text-left outline-none focus-visible:underline">
			<BookMarked class={['size-4 shrink-0', orch ? 'text-status-waiting' : 'text-primary']} />
			<span class={['rounded-xs px-1.75 py-px text-caption font-semibold text-on-solid', orch ? 'bg-status-waiting' : 'bg-primary']}>{orch ? '대신 결정' : '결정'}</span>
			<span class={['flex size-5 items-center justify-center rounded-xs text-on-solid', orch ? 'bg-primary' : 'bg-foreground']}>{#if orch}<Sparkles class="size-3" />{:else}<User class="size-3" />{/if}</span>
			<span class="flex-1 font-semibold">{orch ? 'Orch · 사용자 대신' : '나'}</span>
			<span class="font-mono text-caption text-subtle-foreground">#{record.task} · {record.time}</span>
		</button>
		<p class="flex gap-1.5 text-muted-foreground"><span class="font-mono font-bold">Q</span>{record.q}</p>
		<p class="flex gap-1.5 text-body font-medium"><span class={['font-mono text-xs font-bold', orch ? 'text-status-waiting' : 'text-primary']}>A</span>{record.a}</p>
		{#if orch}
			{#if record.why}
				<div class="flex flex-col gap-1 rounded-sm bg-card p-2.5">
					<span class="text-caption font-semibold text-muted-foreground">근거</span>
					<span>{record.why}</span>
				</div>
			{/if}
			<div class="flex items-center gap-2">
				<span class="flex-1 text-muted-foreground">{record.sent} · 되돌리려면 재검토</span>
				{#if !record.confirmed}
					{#if onreview}<Button variant="outline" onclick={onreview}><RotateCcw />재검토</Button>{/if}
					<Button onclick={() => (record.confirmed = true)}><Check />확인</Button>
				{/if}
			</div>
		{:else}
			<div class="flex items-center gap-3 border-t pt-2">
				<span class="flex items-center gap-1.25 text-muted-foreground"><Send class="size-3.5" />{record.sent}</span>
				{#if onreview}<button type="button" class="flex items-center gap-1.25 font-medium text-primary outline-none hover:underline focus-visible:underline" onclick={onreview}><MessageSquareText class="size-3.5" />답변 전문</button>{/if}
			</div>
		{/if}
	</article>
{/if}
