<script lang="ts">
	import { fly } from 'svelte/transition';
	import { isFilterActive, type DatePreset, type FilterState } from '$lib/utils/filterUtils';

	let {
		filter,
		filteredCount,
		totalCount,
		onRemove,
		onReset
	}: {
		filter: FilterState;
		filteredCount: number;
		totalCount: number;
		onRemove: (key: string) => void;
		onReset: () => void;
	} = $props();

	const PRESET_LABEL: Record<DatePreset, string> = {
		today: 'Hoje',
		yesterday: 'Ontem',
		last7days: 'Últimos 7 dias',
		last30days: 'Últimos 30 dias',
		last3months: 'Últimos 3 meses',
		thisYear: 'Este ano',
		custom: 'Data personalizada'
	};

	interface Chip {
		key: string;
		label: string;
	}
	let chips = $derived.by<Chip[]>(() => {
		const cs: Chip[] = [];
		for (const ext of filter.extensions) cs.push({ key: `extension:${ext}`, label: ext });
		if (filter.sizeRange) {
			const { min, max, unit } = filter.sizeRange;
			let l = '';
			if (min !== null && max !== null) l = `${min}–${max} ${unit}`;
			else if (min !== null) l = `> ${min} ${unit}`;
			else if (max !== null) l = `< ${max} ${unit}`;
			cs.push({ key: 'sizeRange', label: l });
		}
		if (filter.dateRange) cs.push({ key: 'dateRange', label: PRESET_LABEL[filter.dateRange.preset] });
		if (filter.onlyDirs) cs.push({ key: 'onlyDirs', label: 'Pastas' });
		if (filter.onlyFiles) cs.push({ key: 'onlyFiles', label: 'Arquivos' });
		if (filter.nameContains) cs.push({ key: 'nameContains', label: `"${filter.nameContains}"` });
		return cs;
	});

	let active = $derived(isFilterActive(filter));
	let filtering = $derived(filteredCount !== totalCount);
</script>

{#if active}
	<div class="bar">
		<span class="lead">⬡ Filtros:</span>
		<div class="chips">
			{#each chips as chip (chip.key)}
				<span class="chip" transition:fly={{ x: -10, duration: 150 }}>
					<span class="lbl">{chip.label}</span>
					<button class="x" onclick={() => onRemove(chip.key)} aria-label="Remover">×</button>
				</span>
			{/each}
		</div>
		<span class="count" class:filtering>{filteredCount} de {totalCount}</span>
		<button class="reset" onclick={onReset}>Limpar tudo</button>
	</div>
{/if}

<style>
	.bar {
		display: flex;
		align-items: center;
		gap: 8px;
		height: 32px;
		padding: 0 12px;
		background: #0a0a14;
		border-bottom: 1px solid #12121e;
		font-family: 'DM Sans', system-ui, sans-serif;
		overflow-x: auto;
		white-space: nowrap;
	}
	.lead {
		color: #444;
		font-size: 11px;
		flex-shrink: 0;
	}
	.chips {
		display: inline-flex;
		gap: 6px;
		flex: 1;
	}
	.chip {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 2px 8px;
		background: #1e1e35;
		border: 1px solid #a78bfa33;
		border-radius: 11px;
		color: #a78bfa;
		font-size: 11px;
	}
	.x {
		border: none;
		background: none;
		color: #a78bfa99;
		font-size: 12px;
		line-height: 1;
		cursor: pointer;
		padding: 0;
	}
	.x:hover {
		color: #fff;
	}
	.count {
		color: #555;
		font-family: 'DM Mono', ui-monospace, monospace;
		font-size: 11px;
		flex-shrink: 0;
	}
	.count.filtering {
		color: #a78bfa;
	}
	.reset {
		border: none;
		background: none;
		color: #444;
		font-family: inherit;
		font-size: 11px;
		cursor: pointer;
		flex-shrink: 0;
	}
	.reset:hover {
		color: #f87171;
	}
</style>
