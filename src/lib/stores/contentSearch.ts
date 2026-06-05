import { writable, get } from 'svelte/store';
import {
	getIndexStats,
	startIndexing,
	watchDirectory,
	searchContent,
	clearIndex,
	type IndexStats,
	type IndexingProgressEvent,
	type ContentSearchResult,
	type ContentSearchQuery
} from '$lib/tauri';
import { isTauri } from '$lib/tauri';
import { activeTab } from './tabs';
import { showToast } from './bookmarks';

// ── Stores ──────────────────────────────────────────────────────
export const indexStats = writable<IndexStats | null>(null);
export const isIndexing = writable<boolean>(false);
export const indexingSession = writable<string | null>(null);
export const indexingProgress = writable<IndexingProgressEvent | null>(null);

export const contentQuery = writable<string>('');
export const contentResults = writable<ContentSearchResult[]>([]);
export const contentLoading = writable<boolean>(false);
export const contentError = writable<string | null>(null);
export const isContentMode = writable<boolean>(false);

export const contentSearchOptions = writable<Partial<ContentSearchQuery>>({
	max_results: 50,
	include_previews: true,
	preview_chars: 200,
	extensions: []
});

let progressUnlisten: (() => void) | null = null;
let fileUpdatedUnlisten: (() => void) | null = null;

// ── Init / destroy ──────────────────────────────────────────────
export async function initContentSearch(): Promise<void> {
	await loadIndexStats();
	if (!isTauri()) return;

	const { listen } = await import('@tauri-apps/api/event');

	progressUnlisten = await listen<IndexingProgressEvent>('index:progress', ({ payload }) => {
		indexingProgress.set(payload);
		isIndexing.set(!payload.is_complete);
		if (payload.is_complete) {
			indexingSession.set(null);
			loadIndexStats();
			showToast(`Indexação concluída: ${payload.processed} arquivos`, 'success');
		}
	});

	fileUpdatedUnlisten = await listen<string>('index:file-updated', () => {
		loadIndexStats();
	});
}

export function destroyContentSearch(): void {
	progressUnlisten?.();
	fileUpdatedUnlisten?.();
	progressUnlisten = null;
	fileUpdatedUnlisten = null;
}

// ── Stats ───────────────────────────────────────────────────────
export async function loadIndexStats(): Promise<void> {
	if (!isTauri()) return;
	try {
		indexStats.set(await getIndexStats());
	} catch {
		// silencioso
	}
}

// ── Indexação ───────────────────────────────────────────────────
export async function startDirectoryIndexing(path: string): Promise<void> {
	if (get(isIndexing)) {
		showToast('Indexação já em andamento', 'info');
		return;
	}
	isIndexing.set(true);
	indexingProgress.set(null);
	try {
		const sessionId = await startIndexing(path);
		indexingSession.set(sessionId);
		// Observa o diretório em paralelo (não bloqueia).
		watchDirectory(path).catch(() => {});
	} catch (err) {
		isIndexing.set(false);
		showToast(`Falha ao iniciar indexação: ${String(err)}`, 'error');
	}
}

export async function reindexCurrentDirectory(): Promise<void> {
	const path = get(activeTab)?.path;
	if (!path) return;
	await startDirectoryIndexing(path);
}

export async function clearSearchIndex(): Promise<void> {
	try {
		await clearIndex();
		await loadIndexStats();
		showToast('Índice limpo com sucesso', 'success');
	} catch (err) {
		showToast(`Falha ao limpar índice: ${String(err)}`, 'error');
	}
}

// ── Busca por conteúdo ──────────────────────────────────────────
export async function executeContentSearch(query?: string): Promise<void> {
	const q = query ?? get(contentQuery);
	if (q.length < 2) {
		contentError.set('Digite pelo menos 2 caracteres');
		return;
	}

	contentQuery.set(q);
	contentLoading.set(true);
	contentError.set(null);
	isContentMode.set(true);

	const opts = get(contentSearchOptions);
	try {
		const results = await searchContent({
			query: q,
			root_path: get(activeTab)?.path ?? null,
			...opts
		});
		contentResults.set(results);
	} catch (err) {
		contentError.set(err instanceof Error ? err.message : String(err));
	} finally {
		contentLoading.set(false);
	}
}

export function exitContentMode(): void {
	isContentMode.set(false);
	contentQuery.set('');
	contentResults.set([]);
	contentError.set(null);
}
