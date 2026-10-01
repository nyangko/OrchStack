<script lang="ts">
	/// ui/combobox 스파이크 — 태스크와 무관한 데이터(도시) · 하나 고르기 / 여러 개 고르기.
	import * as Combobox from "$lib/components/ui/combobox/index.js";
	import { Button } from "$lib/components/ui/button/index.js";

	const cities = ["서울", "부산", "대구", "제주"];
	let one = $state("서울");
	let many = $state<string[]>([]);
</script>

<main class="flex gap-4 p-8">
	<Combobox.Root>
		<Combobox.Trigger>{#snippet child({ props })}<Button {...props} variant="outline">도시: {one}</Button>{/snippet}</Combobox.Trigger>
		<Combobox.Content>
			<Combobox.Search placeholder="도시 검색" />
			<Combobox.List>
				<Combobox.Empty>없어요</Combobox.Empty>
				<Combobox.Group heading="국내">
					{#each cities as c (c)}<Combobox.Item value={c} selected={one === c} onSelect={() => (one = c)}>{c}</Combobox.Item>{/each}
				</Combobox.Group>
			</Combobox.List>
		</Combobox.Content>
	</Combobox.Root>
	<Combobox.Root>
		<Combobox.Trigger>{#snippet child({ props })}<Button {...props} variant="outline">여럿: {many.join(", ") || "없음"}</Button>{/snippet}</Combobox.Trigger>
		<Combobox.Content>
			<Combobox.List>
				{#each cities as c (c)}<Combobox.Item value={c} selected={many.includes(c)} closeOnSelect={false} onSelect={() => (many = many.includes(c) ? many.filter((x) => x !== c) : [...many, c])}>{c}</Combobox.Item>{/each}
			</Combobox.List>
		</Combobox.Content>
	</Combobox.Root>
	<p data-testid="out">{one} | {many.join(",")}</p>
</main>
