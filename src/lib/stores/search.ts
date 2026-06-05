import { writable, get } from 'svelte/store';
import { searchFiles, searchFilesQuick, type SearchOptions, type SearchResult } from '$lib/tauri';
import { activeTab, activeTabId, navigateTab } from './tabs';
import {
	executeContentSearch,
	exitContentMode,
	contentResults
} from './contentSearch';

const RECENT_KEY = 'fe-pro-recent-searches';
const MAX_RECENT = 10;

export type SearchMode = 'name' | 'content';
export const searchMode = writable<SearchMode>('name');

// ── Stores ──────────────────────────────────────────────────────
export const searchQuery = writable<string>('');
export const searchResults = writable<SearchResult[]>([]);
export const searchLoading = writable<boolean>(false);
export const searchError = writable<string | null>(null);
export const isSearchMode = writable<boolean>(false);

export const searchSuggestions = writable<SearchResult[]>([]);
export const suggestionsLoading = writable<boolean>(false);
export const showSuggestions = writable<boolean>(false);

export const recentSearches = writable<string[]>([]);

export const searchScope = writable<'current' | 'recursive'>('recursive');

export const searchOptions = writable<SearchOptions>({
	max_results: 50,
	include_dirs: true,
	include_files: true,
	recursive: true,
	max_depth: 5,
	extensions: []
});

// ── Debounce ────────────────────────────────────────────────────
function createDebounce<A extends unknown[]>(fn: (...args: A) => void, delay: number) {
	let timer: ReturnType<typeof setTimeout> | undefined;
	const debounced = (...args: A) => {
		if (timer) clearTimeout(timer);
		timer = setTimeout(() => fn(...args), delay);
	};
	const cancel = () => {
		if (timer) clearTimeout(timer);
		timer = undefined;
	};
	return { debounced, cancel };
}

async function fetchSuggestions(query: string, path: string): Promise<void> {
	if (!path) {
		suggestionsLoading.set(false);
		return;
	}
	try {
		const results = await searchFilesQuick(query, path);
		searchSuggestions.set(results);
	} catch {
		searchSuggestions.set([]); // falha silenciosa
	} finally {
		suggestionsLoading.set(false);
	}
}

const { debounced: debouncedSuggest, cancel: cancelSuggest } = createDebounce(
	(query: string, path: string) => fetchSuggestions(query, path),
	200
);

// ── Ações ───────────────────────────────────────────────────────
export async function updateQuery(query: string): Promise<void> {
	searchQuery.set(query);
	if (query.length < 2) {
		cancelSuggest();
		searchSuggestions.set([]);
		showSuggestions.set(false);
		suggestionsLoading.set(false);
		return;
	}
	suggestionsLoading.set(true);
	showSuggestions.set(true);
	debouncedSuggest(query, get(activeTab)?.path ?? '');
}

function pushRecent(query: string): void {
	recentSearches.update((list) => {
		const next = [query, ...list.filter((q) => q !== query)].slice(0, MAX_RECENT);
		if (typeof localStorage !== 'undefined') {
			localStorage.setItem(RECENT_KEY, JSON.stringify(next));
		}
		return next;
	});
}

export async function executeSearch(query?: string): Promise<void> {
	const q = query ?? get(searchQuery);
	if (q.length < 2) {
		searchError.set('Digite pelo menos 2 caracteres');
		return;
	}

	searchQuery.set(q);
	showSuggestions.set(false);
	cancelSuggest();
	pushRecent(q);

	// Modo conteúdo: delega para o store de busca full-text.
	if (get(searchMode) === 'content') {
		await executeContentSearch(q);
		return;
	}

	searchLoading.set(true);
	searchError.set(null);
	isSearchMode.set(true);

	const root = get(activeTab)?.path ?? '';
	const base = get(searchOptions);
	const options: SearchOptions =
		get(searchScope) === 'current'
			? { ...base, recursive: false, max_depth: 1 }
			: { ...base };

	try {
		const results = await searchFiles(q, root, options);
		searchResults.set(results);
	} catch (err) {
		searchError.set(err instanceof Error ? err.message : String(err));
	} finally {
		searchLoading.set(false);
	}
}

export function exitSearchMode(): void {
	isSearchMode.set(false);
	searchQuery.set('');
	searchResults.set([]);
	searchSuggestions.set([]);
	showSuggestions.set(false);
	searchError.set(null);
	cancelSuggest();
	exitContentMode();
}

/** Alterna entre busca por nome e por conteúdo; limpa resultados. */
export function toggleSearchMode(): void {
	searchMode.update((m) => (m === 'name' ? 'content' : 'name'));
	searchResults.set([]);
	contentResults.set([]);
	searchError.set(null);
}

export function selectSuggestion(result: SearchResult): void {
	const id = get(activeTabId);
	if (id) {
		const target = result.is_dir ? result.path : result.parent_path;
		navigateTab(id, target);
	}
	exitSearchMode();
}

// ── Histórico ───────────────────────────────────────────────────
export function initRecentSearches(): void {
	if (typeof localStorage === 'undefined') return;
	try {
		const raw = localStorage.getItem(RECENT_KEY);
		const parsed = raw ? JSON.parse(raw) : [];
		recentSearches.set(Array.isArray(parsed) ? parsed : []);
	} catch {
		recentSearches.set([]);
	}
}

export function clearRecentSearches(): void {
	recentSearches.set([]);
	if (typeof localStorage !== 'undefined') localStorage.removeItem(RECENT_KEY);
}

export function removeRecentSearch(query: string): void {
	recentSearches.update((list) => {
		const next = list.filter((q) => q !== query);
		if (typeof localStorage !== 'undefined') {
			localStorage.setItem(RECENT_KEY, JSON.stringify(next));
		}
		return next;
	});
}
