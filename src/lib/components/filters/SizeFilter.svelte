<script lang="ts">
	import type { DirectorySummary } from '$lib/tauri';
	import { sizeToBytes, type SizeRange, type SizeUnit } from '$lib/utils/filterUtils';

	let {
		sizeRange,
		summary,
		onChange
	}: {
		sizeRange: SizeRange | null;
		summary: DirectorySummary | null;
		onChange: (range: SizeRange | null) => void;
	} = $props();

	const UNITS: SizeUnit[] = ['B', 'KB', 'MB', 'GB'];

	interface Preset {
		label: string;
		range: SizeRange | null;
	}
	const PRESETS: Preset[] = [
		{ label: 'Qualquer', range: null },
		{ label: '< 100 KB', range: { min: null, max: 100, unit: 'KB' } },
		{ label: '100 KB – 1 MB', range: { min: 100, max: 1024, unit: 'KB' } },
		{ label: '1 MB – 100 MB', range: { min: 1, max: 100, unit: 'MB' } },
		{ label: '> 100 MB', range: { min: 100, max: null, unit: 'MB' } }
	];

	function sameRange(a: SizeRange | null, b: SizeRange | null): boolean {
		if (a === null && b === null) return true;
		if (!a || !b) return false;
		return a.min === b.min && a.max === b.max && a.unit === b.unit;
	}

	// ── Campos personalizados ─────────────────────────────────────
	let minVal = $state<string>('');
	let maxVal = $state<string>('');
	let minUnit = $state<SizeUnit>('KB');
	let maxUnit = $state<SizeUnit>('MB');
	let debounce: ReturnType<typeof setTimeout> | undefined;

	let invalid = $derived.by(() => {
		const mn = minVal !== '' ? sizeToBytes(Number(minVal), minUnit) : null;
		const mx = maxVal !== '' ? sizeToBytes(Number(maxVal), maxUnit) : null;
		return mn !== null && mx !== null && mn > mx;
	});

	function emitCustom() {
		if (debounce) clearTimeout(debounce);
		debounce = setTimeout(() => {
			if (invalid) return;
			if (minVal === '' && maxVal === '') {
				onChange(null);
				return;
			}
			// Normaliza p/ uma única unidade (a maior fornecida).
			const unit: SizeUnit = maxVal !== '' ? maxUnit : minUnit;
			const min = minVal !== '' ? sizeToBytes(Number(minVal), minUnit) / sizeToBytes(1, unit) : null;
			const max = maxVal !== '' ? sizeToBytes(Number(maxVal), maxUnit) / sizeToBytes(1, unit) : null;
			onChange({ min, max, unit });
		}, 500);
	}

	// ── Histograma (aproximado via média por extensão) ────────────
	const BUCKETS: { label: string; min: number; max: number | null }[] = [
		{ label: '<1KB', min: 0, max: 1024 },
		{ label: '1-10KB', min: 1024, max: 10240 },
		{ label: '10-100KB', min: 10240, max: 102400 },
		{ label: '100KB-1MB', min: 102400, max: 1048576 },
		{ label: '1-10MB', min: 1048576, max: 10485760 },
		{ label: '10-100MB', min: 10485760, max: 104857600 },
		{ label: '100MB-1GB', min: 104857600, max: 1073741824 },
		{ label: '>1GB', min: 1073741824, max: null }
	];

	let histogram = $derived.by<number[]>(() => {
		const counts = new Array(BUCKETS.length).fill(0);
		if (!summary) return counts;
		for (const e of summary.extensions) {
			const avg = e.count > 0 ? e.total_size / e.count : 0;
			const idx = BUCKETS.findIndex((b) => avg >= b.min && (b.max === null || avg < b.max));
			if (idx >= 0) counts[idx] += e.count;
		}
		return counts;
	});
	let maxCount = $derived(Math.max(1, ...histogram));

	function bytesToRange(min: number, max: number | null): SizeRange {
		const ref = max ?? min;
		let unit: SizeUnit = 'B';
		if (ref >= 1_073_741_824) unit = 'GB';
		else if (ref >= 1_048_576) unit = 'MB';
		else if (ref >= 1024) unit = 'KB';
		const div = sizeToBytes(1, unit);
		return { min: min > 0 ? min / div : null, max: max !== null ? max / div : null, unit };
	}
</script>

<div class="sizef">
	<div class="title">Tamanho</div>

	<div class="chips">
		{#each PRESETS as p (p.label)}
			<button class="chip" class:sel={sameRange(sizeRange, p.range)} onclick={() => onChange(p.range)}>
				{p.label}
			</button>
		{/each}
	</div>

	<div class="divider">ou personalizado</div>

	<div class="custom">
		<label>
			<span>Mínimo</span>
			<span class="row">
				<input class="num" class:invalid type="number" min="0" bind:value={minVal} oninput={emitCustom} />
				<select bind:value={minUnit} onchange={emitCustom}>
					{#each UNITS as u (u)}<option value={u}>{u}</option>{/each}
				</select>
			</span>
		</label>
		<label>
			<span>Máximo</span>
			<span class="row">
				<input class="num" type="number" min="0" bind:value={maxVal} oninput={emitCustom} />
				<select bind:value={maxUnit} onchange={emitCustom}>
					{#each UNITS as u (u)}<option value={u}>{u}</option>{/each}
				</select>
			</span>
		</label>
	</div>
	{#if invalid}
		<div class="err">Mínimo maior que máximo</div>
	{/if}

	{#if summary}
		<div class="hist" title="Distribuição aproximada por tamanho">
			{#each BUCKETS as b, i (b.label)}
				<button
					class="bar-wrap"
					title="{b.label}: {histogram[i]}"
					onclick={() => onChange(bytesToRange(b.min, b.max))}
				>
					<span class="bar" style="height: {(histogram[i] / maxCount) * 32}px"></span>
				</button>
			{/each}
		</div>
	{/if}
</div>

<style>
	.sizef {
		display: flex;
		flex-direction: column;
		gap: 8px;
		font-family: 'DM Sans', system-ui, sans-serif;
	}
	.title {
		color: #444;
		font-size: 10px;
		text-transform: uppercase;
		letter-spacing: 0.06em;
	}
	.chips {
		display: inline-flex;
		flex-wrap: wrap;
		gap: 6px;
	}
	.chip {
		padding: 3px 10px;
		border: 1px solid #1e1e35;
		border-radius: 12px;
		background: #111120;
		color: #666;
		font-family: inherit;
		font-size: 11px;
		cursor: pointer;
		transition: all 100ms ease;
	}
	.chip:hover {
		border-color: #2a2a42;
		color: #888;
	}
	.chip.sel {
		background: #1e1e35;
		border-color: #a78bfa55;
		color: #a78bfa;
	}
	.divider {
		color: #2a2a42;
		font-size: 10px;
		text-align: center;
		margin: 2px 0;
	}
	.custom {
		display: flex;
		gap: 10px;
	}
	.custom label {
		display: flex;
		flex-direction: column;
		gap: 3px;
		color: #555;
		font-size: 10px;
	}
	.row {
		display: flex;
		gap: 4px;
	}
	.num {
		width: 56px;
		height: 26px;
		padding: 0 6px;
		background: #111120;
		border: 1px solid #1e1e35;
		border-radius: 5px;
		color: #c8c8dc;
		font-family: inherit;
		font-size: 12px;
		outline: none;
	}
	.num.invalid {
		border-color: #f87171;
	}
	select {
		height: 26px;
		background: #111120;
		border: 1px solid #1e1e35;
		border-radius: 5px;
		color: #c8c8dc;
		font-family: inherit;
		font-size: 12px;
		outline: none;
	}
	.err {
		color: #f87171;
		font-size: 11px;
	}
	.hist {
		display: flex;
		align-items: flex-end;
		gap: 2px;
		height: 36px;
		width: 100%;
		margin-top: 4px;
	}
	.bar-wrap {
		flex: 1;
		display: flex;
		align-items: flex-end;
		justify-content: center;
		height: 100%;
		border: none;
		background: none;
		cursor: pointer;
		padding: 0;
	}
	.bar {
		width: 100%;
		min-height: 2px;
		background: #1e1e35;
		border-radius: 2px 2px 0 0;
		transition: background 100ms ease;
	}
	.bar-wrap:hover .bar {
		background: #a78bfa55;
	}
</style>
