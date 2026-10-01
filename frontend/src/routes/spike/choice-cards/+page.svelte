<script lang="ts">
	/// ui/choice-cards 스파이크 — 태스크와 무관한 데이터: 요금제(하나) · 토핑(여러 개 · 흐림).
	import * as ChoiceCards from "$lib/components/ui/choice-cards/index.js";

	let plan = $state<string | number | null>("pro");
	let toppings = $state<(string | number | null)[]>(["cheese"]);
</script>

<main class="flex max-w-3xl flex-col gap-6 p-8">
	<ChoiceCards.Root aria-label="요금제" class="grid-cols-3" bind:value={plan}>
		{#each [["free", "Free", "개인용"], ["pro", "Pro", "팀 5명"], ["max", "Max", "무제한"]] as [v, t, d] (v)}
			<ChoiceCards.Item value={v}><span class="font-semibold">{t}</span><span class="text-xs text-muted-foreground">{d}</span></ChoiceCards.Item>
		{/each}
	</ChoiceCards.Root>
	<ChoiceCards.Root type="multiple" aria-label="토핑" class="grid-cols-2" bind:value={toppings}>
		{#each [["cheese", "치즈"], ["olive", "올리브"], ["onion", "양파"]] as [v, t] (v)}
			<ChoiceCards.Item value={v} layout="row" tone="dim">{t}</ChoiceCards.Item>
		{/each}
	</ChoiceCards.Root>
	<p data-testid="out">{plan} | {(toppings as string[]).join(",")}</p>
</main>
