<script lang="ts">
	/// Trust 레벨 선택 (.pen 기본 Trust 레벨) — 1 읽기 전용 ~ 4 자율. 에이전트 권한 탭과 설정 › 권한 · 보안에서 쓴다.
	import type { AgentConfig } from '$lib/mock';
	import { cn } from '$lib/utils';

	let {
		value = $bindable(),
		base,
	}: {
		value: AgentConfig['trust'];
		/** 템플릿 기본값 — 다르면 그 칸에 표시한다 (멤버 화면에서만). */
		base?: AgentConfig['trust'];
	} = $props();

	const levels = [
		{ n: 1, t: '읽기 전용', d: '코드 읽기 · 분석만' },
		{ n: 2, t: '제안만', d: '변경은 diff로 제출, 사람이 적용' },
		{ n: 3, t: '워크스페이스 쓰기', d: '허용 범위 안에서 파일 수정 · 커밋' },
		{ n: 4, t: '자율', d: 'push · PR 까지 자동 (Review 필수)' },
	] as const;
</script>

<div role="radiogroup" aria-label="Trust 레벨" class="grid grid-cols-4 gap-2.5">
	{#each levels as l (l.n)}
		{@const on = value === l.n}
		<button
			type="button"
			role="radio"
			aria-checked={on}
			onclick={() => (value = l.n)}
			class={cn('option-press rounded-md', on && 'option-on')}
		>
			<span class="row-title-strong">
				<span class={cn('level-num', on ? 'bg-primary text-on-solid' : 'bg-muted text-muted-foreground')}>{l.n}</span>
				{l.t}
			</span>
			<span class="text-xs text-muted-foreground">{l.d}</span>
			{#if base === l.n && !on}<span class="text-caption text-primary">템플릿 기본값</span>{/if}
		</button>
	{/each}
</div>
