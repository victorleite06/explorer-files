import { writable, get } from 'svelte/store';
import { listDirectory, type FileEntry } from '$lib/tauri';

export interface Column {
	id: string;
	path: string; // pasta que esta coluna representa
	entries: FileEntry[]; // conteúdo da pasta
	selectedPath: string | null; // item selecionado nesta coluna
	isLoading: boolean;
	width: number; // largura em px, default 220
	previewEntry?: FileEntry | null; // se preenchido, coluna é preview de arquivo
}

const MIN_W = 160;
const MAX_W = 400;
const DEFAULT_W = 220;

export const columns = writable<Column[]>([]);

function mkColumn(path: string): Column {
	return {
		id: crypto.randomUUID(),
		path,
		entries: [],
		selectedPath: null,
		isLoading: false,
		width: DEFAULT_W,
		previewEntry: null
	};
}

function patchColumn(id: string, patch: Partial<Column>): void {
	columns.update((cs) => cs.map((c) => (c.id === id ? { ...c, ...patch } : c)));
}

async function loadColumn(id: string, path: string): Promise<void> {
	patchColumn(id, { isLoading: true });
	try {
		const entries = await listDirectory(path);
		patchColumn(id, { entries, isLoading: false });
	} catch {
		patchColumn(id, { entries: [], isLoading: false });
	}
}

export async function initColumns(rootPath: string): Promise<void> {
	const col = mkColumn(rootPath);
	columns.set([col]);
	await loadColumn(col.id, rootPath);
}

export async function selectInColumn(columnIndex: number, entry: FileEntry): Promise<void> {
	const cs = get(columns);
	if (columnIndex < 0 || columnIndex >= cs.length) return;

	// Trunca tudo após a coluna e marca a seleção.
	const base = cs
		.slice(0, columnIndex + 1)
		.map((c, i) => (i === columnIndex ? { ...c, selectedPath: entry.path } : c));

	if (entry.is_dir) {
		const next = mkColumn(entry.path);
		columns.set([...base, next]);
		await loadColumn(next.id, entry.path);
	} else {
		// Coluna de preview do arquivo.
		const preview: Column = { ...mkColumn(entry.path), previewEntry: entry };
		columns.set([...base, preview]);
	}
}

export function setColumnWidth(columnIndex: number, width: number): void {
	const w = Math.max(MIN_W, Math.min(MAX_W, width));
	columns.update((cs) => cs.map((c, i) => (i === columnIndex ? { ...c, width: w } : c)));
}

export function getColumnPath(columnIndex: number): string {
	return get(columns)[columnIndex]?.path ?? '';
}
