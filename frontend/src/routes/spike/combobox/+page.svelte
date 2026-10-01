<script lang="ts">
	/// ui/combobox 스파이크 — 태스크와 무관한 데이터(도시) · 하나 고르기 / 여러 개 고르기.
	import { Combobox, ComboboxTrigger, ComboboxContent, ComboboxSearch, ComboboxList, ComboboxEmpty, ComboboxGroup, ComboboxItem } from "$lib/components/orch/combobox";
	import { Button } from "$lib/components/ui/button/index.js";

	const cities = ["서울", "부산", "대구", "제주"];
	let one = $state("서울");
	let many = $state<string[]>([]);
</script>

<main class="flex gap-4 p-8">
	<Combobox>
		<ComboboxTrigger>{#snippet child({ props })}<Button {...props} variant="outline">도시: {one}</Button>{/snippet}</ComboboxTrigger>
		<ComboboxContent>
			<ComboboxSearch placeholder="도시 검색" />
			<ComboboxList>
				<ComboboxEmpty>없어요</ComboboxEmpty>
				<ComboboxGroup heading="국내">
					{#each cities as c (c)}<ComboboxItem value={c} selected={one === c} onSelect={() => (one = c)}>{c}</ComboboxItem>{/each}
				</ComboboxGroup>
			</ComboboxList>
		</ComboboxContent>
	</Combobox>
	<Combobox>
		<ComboboxTrigger>{#snippet child({ props })}<Button {...props} variant="outline">여럿: {many.join(", ") || "없음"}</Button>{/snippet}</ComboboxTrigger>
		<ComboboxContent>
			<ComboboxList>
				{#each cities as c (c)}<ComboboxItem value={c} selected={many.includes(c)} closeOnSelect={false} onSelect={() => (many = many.includes(c) ? many.filter((x) => x !== c) : [...many, c])}>{c}</ComboboxItem>{/each}
			</ComboboxList>
		</ComboboxContent>
	</Combobox>
	<p data-testid="out">{one} | {many.join(",")}</p>
</main>
