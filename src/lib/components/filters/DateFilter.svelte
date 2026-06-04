<script lang="ts">
	import type { DateField, DatePreset, DateRange } from '$lib/utils/filterUtils';

	let {
		dateRange,
		onChange
	}: { dateRange: DateRange | null; onChange: (range: DateRange | null) => void } = $props();

	const PRESETS: { preset: DatePreset; label: string }[] = [
		{ preset: 'today', label: 'Hoje' },
		{ preset: 'yesterday', label: 'Ontem' },
		{ preset: 'last7days', label: '7 dias' },
		{ preset: 'last30days', label: '30 dias' },
		{ preset: 'last3months', label: '3 meses' },
		{ preset: 'thisYear', label: 'Este ano' }
	];
	const PRESET_FULL: Record<DatePreset, string> = {
		today: 'hoje',
		yesterday: 'ontem',
		last7days: 'nos últimos 7 dias',
		last30days: 'nos últimos 30 dias',
		last3months: 'nos últimos 3 meses',
		thisYear: 'neste ano',
		custom: 'período personalizado'
	};

	let field = $derived<DateField>(dateRange?.field ?? 'modified');

	function setField(e: Event) {
		const f = (e.target as HTMLSelectElement).value as DateField;
		if (dateRange) onChange({ ...dateRange, field: f });
	}

	function setPreset(preset: DatePreset) {
		onChange({ preset, field, customFrom: null, customTo: null });
	}

	let fromVal = $derived(dateRange?.customFrom ? dateRange.customFrom.slice(0, 10) : '');
	let toVal = $derived(dateRange?.customTo ? dateRange.customTo.slice(0, 10) : '');

	function setCustom(from: string, to: string) {
		const customFrom = from ? new Date(from).toISOString() : null;
		const customTo = to ? new Date(`${to}T23:59:59`).toISOString() : null;
		if (!customFrom && !customTo) {
			onChange(null);
			return;
		}
		onChange({ preset: 'custom', field, customFrom, customTo });
	}

	let invalid = $derived(!!fromVal && !!toVal && new Date(fromVal) > new Date(toVal));

	function fmt(iso: string | null): string {
		if (!iso) return '';
		const d = new Date(iso);
		return isNaN(d.getTime()) ? '' : d.toLocaleDateString('pt-BR');
	}

	let summaryText = $derived.by(() => {
		if (!dateRange) return '';
		const fieldLabel = field === 'created' ? 'Criado' : 'Modificado';
		if (dateRange.preset === 'custom') {
			const a = fmt(dateRange.customFrom);
			const b = fmt(dateRange.customTo);
			if (a && b) return `${fieldLabel} entre ${a} e ${b}`;
			if (a) return `${fieldLabel} a partir de ${a}`;
			if (b) return `${fieldLabel} até ${b}`;
			return '';
		}
		return `${fieldLabel} ${PRESET_FULL[dateRange.preset]}`;
	});
</script>

<div class="datef">
	<label class="field">
		<span>Campo</span>
		<select value={field} onchange={setField}>
			<option value="modified">Modificado</option>
			<option value="created">Criado</option>
		</select>
	</label>

	<div class="chips">
		<button class="chip" class:sel={dateRange === null} onclick={() => onChange(null)}>Qualquer</button>
		{#each PRESETS as p (p.preset)}
			<button
				class="chip"
				class:sel={dateRange?.preset === p.preset}
				onclick={() => setPreset(p.preset)}
			>
				{p.label}
			</button>
		{/each}
	</div>

	<div class="divider">personalizado</div>

	<div class="custom">
		<label>
			<span>De</span>
			<input type="date" class:invalid value={fromVal} onchange={(e) => setCustom((e.target as HTMLInputElement).value, toVal)} />
		</label>
		<label>
			<span>Até</span>
			<input type="date" value={toVal} onchange={(e) => setCustom(fromVal, (e.target as HTMLInputElement).value)} />
		</label>
	</div>
	{#if invalid}
		<div class="err">Data inicial maior que a final</div>
	{/if}

	{#if summaryText}
		<div class="summary">{summaryText}</div>
	{/if}
</div>

<style>
	.datef {
		display: flex;
		flex-direction: column;
		gap: 8px;
		font-family: 'DM Sans', system-ui, sans-serif;
	}
	.field {
		display: flex;
		flex-direction: column;
		gap: 3px;
		color: #555;
		font-size: 10px;
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
	select,
	input[type='date'] {
		height: 26px;
		padding: 0 6px;
		background: #111120;
		border: 1px solid #1e1e35;
		border-radius: 5px;
		color: #c8c8dc;
		font-family: inherit;
		font-size: 12px;
		outline: none;
		color-scheme: dark;
	}
	input.invalid {
		border-color: #f87171;
	}
	.err {
		color: #f87171;
		font-size: 11px;
	}
	.summary {
		color: #555;
		font-size: 11px;
	}
</style>
