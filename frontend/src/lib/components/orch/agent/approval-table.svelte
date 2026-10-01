<script lang="ts">
	/// 승인 규칙 표 (.pen 승인 규칙) — 동작마다 정책 · 승인자 · 알림. 에이전트 권한 탭과 설정 › 권한 · 보안에서 쓴다.
	import * as Select from '$lib/components/ui/select';
	import { Switch } from '$lib/components/ui/switch';
	import type { Approval } from '$lib/mock';

	let {
		approvals,
		base,
	}: {
		approvals: Approval[];
		/** 템플릿 값 — 다른 줄에 점을 찍는다 (멤버 화면에서만). */
		base?: Approval[];
	} = $props();
</script>

<div class="approval-head">
	<span class="flex-1">동작</span><span class="w-36">정책</span><span class="w-28">승인자</span><span class="w-12">알림</span>
</div>
{#each approvals as a, i (a.action)}
	{@const b = base?.[i]}
	<div class="approval-row">
		<span class="approval-action">{a.action}{#if b && JSON.stringify(b) !== JSON.stringify(a)}<span class="size-1.5 rounded-full bg-primary" aria-label="템플릿과 다름"></span>{/if}</span>
		<span class="w-36">
			<!-- 승인 필요가 아니면 승인자는 없다 -->
			<Select.Root type="single" bind:value={() => a.policy, (v) => ((a.policy = v as Approval['policy']), (a.who = v === '승인 필요' ? (a.who === '—' ? '나' : a.who) : '—'))}>
				<Select.Trigger size="sm" class="h-7 w-full" aria-label="{a.action} 정책">{a.policy}</Select.Trigger>
				<Select.Content>
					{#each ['승인 필요', '자동', '차단'] as v (v)}<Select.Item value={v} label={v} />{/each}
				</Select.Content>
			</Select.Root>
		</span>
		<span class="w-28 text-muted-foreground">{a.who}</span>
		<span class="w-12"><Switch bind:checked={a.notify} aria-label="{a.action} 알림" /></span>
	</div>
{/each}
