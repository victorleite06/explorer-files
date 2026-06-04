import type { FileEntry } from '$lib/tauri';

// ── Tipos ───────────────────────────────────────────────────────
export type SizeUnit = 'B' | 'KB' | 'MB' | 'GB';

export interface SizeRange {
	min: number | null;
	max: number | null;
	unit: SizeUnit;
}

export type DateField = 'modified' | 'created';

export type DatePreset =
	| 'today'
	| 'yesterday'
	| 'last7days'
	| 'last30days'
	| 'last3months'
	| 'thisYear'
	| 'custom';

export interface DateRange {
	preset: DatePreset;
	field: DateField;
	customFrom: string | null;
	customTo: string | null;
}

export interface FilterState {
	extensions: string[];
	excludeExtensions: string[];
	sizeRange: SizeRange | null;
	dateRange: DateRange | null;
	onlyDirs: boolean;
	onlyFiles: boolean;
	nameContains: string;
}

export const DEFAULT_FILTER: FilterState = {
	extensions: [],
	excludeExtensions: [],
	sizeRange: null,
	dateRange: null,
	onlyDirs: false,
	onlyFiles: false,
	nameContains: ''
};

// ── Helpers ─────────────────────────────────────────────────────
export function sizeToBytes(value: number, unit: SizeUnit): number {
	switch (unit) {
		case 'B':
			return value;
		case 'KB':
			return value * 1024;
		case 'MB':
			return value * 1_048_576;
		case 'GB':
			return value * 1_073_741_824;
	}
}

function startOfDay(d: Date): Date {
	const x = new Date(d);
	x.setHours(0, 0, 0, 0);
	return x;
}
function endOfDay(d: Date): Date {
	const x = new Date(d);
	x.setHours(23, 59, 59, 999);
	return x;
}

export function getPresetDateRange(preset: DatePreset): { from: Date; to: Date } {
	const now = new Date();
	switch (preset) {
		case 'today':
			return { from: startOfDay(now), to: endOfDay(now) };
		case 'yesterday': {
			const y = new Date(now);
			y.setDate(now.getDate() - 1);
			return { from: startOfDay(y), to: endOfDay(y) };
		}
		case 'last7days': {
			const f = new Date(now);
			f.setDate(now.getDate() - 7);
			return { from: f, to: now };
		}
		case 'last30days': {
			const f = new Date(now);
			f.setDate(now.getDate() - 30);
			return { from: f, to: now };
		}
		case 'last3months': {
			const f = new Date(now);
			f.setMonth(now.getMonth() - 3);
			return { from: f, to: now };
		}
		case 'thisYear':
			return { from: new Date(now.getFullYear(), 0, 1), to: now };
		case 'custom':
			throw new Error('getPresetDateRange não deve ser chamado para preset custom');
	}
}

// Remove acentos e normaliza p/ comparação case-insensitive.
function normalize(s: string): string {
	return s
		.normalize('NFD')
		.replace(/[\u0300-\u036f]/g, '')
		.toLowerCase();
}

// ── Aplicação dos filtros ───────────────────────────────────────
export function applyFilters(entries: FileEntry[], filter: FilterState): FileEntry[] {
	let out = entries;

	// (Ocultos agora são filtrados no backend via IgnoreRules.)

	// Apenas pastas / apenas arquivos
	if (filter.onlyDirs) {
		out = out.filter((e) => e.is_dir);
	} else if (filter.onlyFiles) {
		out = out.filter((e) => !e.is_dir);
	}

	// 3. Nome contém (sem acento, case-insensitive)
	if (filter.nameContains) {
		const needle = normalize(filter.nameContains);
		out = out.filter((e) => normalize(e.name).includes(needle));
	}

	// 4. Extensões incluídas (pastas sempre passam)
	if (filter.extensions.length > 0) {
		const set = new Set(filter.extensions.map((x) => x.toLowerCase()));
		out = out.filter((e) => e.is_dir || (e.extension != null && set.has(e.extension.toLowerCase())));
	}

	// 5. Extensões excluídas (pastas nunca são excluídas)
	if (filter.excludeExtensions.length > 0) {
		const set = new Set(filter.excludeExtensions.map((x) => x.toLowerCase()));
		out = out.filter((e) => e.is_dir || e.extension == null || !set.has(e.extension.toLowerCase()));
	}

	// 6. Tamanho (pastas sempre passam)
	if (filter.sizeRange) {
		const { min, max, unit } = filter.sizeRange;
		const minB = min !== null ? sizeToBytes(min, unit) : null;
		const maxB = max !== null ? sizeToBytes(max, unit) : null;
		out = out.filter((e) => {
			if (e.is_dir) return true;
			if (minB !== null && e.size < minB) return false;
			if (maxB !== null && e.size > maxB) return false;
			return true;
		});
	}

	// 7. Data (pastas sempre passam; sem data = passa)
	if (filter.dateRange) {
		const { field, preset, customFrom, customTo } = filter.dateRange;
		let from: Date | null = null;
		let to: Date | null = null;
		if (preset === 'custom') {
			from = customFrom ? new Date(customFrom) : null;
			to = customTo ? new Date(customTo) : null;
		} else {
			const r = getPresetDateRange(preset);
			from = r.from;
			to = r.to;
		}
		out = out.filter((e) => {
			if (e.is_dir) return true;
			const raw = field === 'created' ? e.created_at : e.modified;
			if (!raw) return true;
			const d = new Date(raw);
			if (isNaN(d.getTime())) return true;
			if (from && d < from) return false;
			if (to && d > to) return false;
			return true;
		});
	}

	return out;
}

// ── Estado / resumo ─────────────────────────────────────────────
export function isFilterActive(filter: FilterState): boolean {
	return (
		filter.extensions.length > 0 ||
		filter.excludeExtensions.length > 0 ||
		filter.sizeRange !== null ||
		filter.dateRange !== null ||
		filter.onlyDirs ||
		filter.onlyFiles ||
		filter.nameContains !== ''
	);
}

export function countActiveFilters(filter: FilterState): number {
	let n = 0;
	if (filter.extensions.length > 0) n += 1;
	if (filter.sizeRange !== null) n += 1;
	if (filter.dateRange !== null) n += 1;
	if (filter.onlyDirs || filter.onlyFiles) n += 1;
	if (filter.nameContains !== '') n += 1;
	return n;
}

const PRESET_LABEL: Record<DatePreset, string> = {
	today: 'Hoje',
	yesterday: 'Ontem',
	last7days: 'Últimos 7 dias',
	last30days: 'Últimos 30 dias',
	last3months: 'Últimos 3 meses',
	thisYear: 'Este ano',
	custom: 'Período personalizado'
};

export function filterSummary(filter: FilterState): string {
	const parts: string[] = [];

	if (filter.extensions.length > 0) {
		const n = filter.extensions.length;
		parts.push(`${n} ${n === 1 ? 'extensão' : 'extensões'}`);
	}

	if (filter.sizeRange) {
		const { min, max, unit } = filter.sizeRange;
		if (min !== null && max !== null) parts.push(`${min}–${max} ${unit}`);
		else if (min !== null) parts.push(`> ${min} ${unit}`);
		else if (max !== null) parts.push(`< ${max} ${unit}`);
	}

	if (filter.dateRange) {
		parts.push(PRESET_LABEL[filter.dateRange.preset]);
	}

	if (filter.onlyDirs) parts.push('Apenas pastas');
	else if (filter.onlyFiles) parts.push('Apenas arquivos');

	if (filter.nameContains) parts.push(`nome contém "${filter.nameContains}"`);

	return parts.join(' · ');
}
