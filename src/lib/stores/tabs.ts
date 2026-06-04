import { writable, derived, get } from 'svelte/store';
import { listDirectory, scanDirectorySummary, type FileEntry, type DirectorySummary } from '$lib/tauri';
import { sortEntries } from '$lib/utils/fileUtils';
import { DEFAULT_FILTER, type FilterState } from '$lib/utils/filterUtils';

export interface Tab {
	id: string; // uuid gerado no frontend
	path: string; // pasta atual da tab
	title: string; // nome da pasta (último segmento do path)
	history: string[]; // histórico de navegação desta tab
	historyIndex: number; // posição atual no histórico
	viewMode: 'list' | 'grid' | 'columns'; // modo de visualização independente
	sortBy: 'name' | 'size' | 'modified' | 'type';
	sortDir: 'asc' | 'desc';
	columnWidths: Record<string, number>; // larguras salvas por coluna
	selectedPaths: Set<string>; // arquivos selecionados na tab
	scrollPosition: number; // posição do scroll para restaurar
	isLoading: boolean;
	error: string | null;
	filter: FilterState; // estado dos filtros desta tab
	directorySummary: DirectorySummary | null; // metadados do diretório atual
}

// Âncora de seleção por tab (p/ seleção em range com Shift).
const selectionAnchors = new Map<string, string>();

// ── Stores ──────────────────────────────────────────────────────
export const tabs = writable<Tab[]>([]);
export const activeTabId = writable<string | null>(null);

// Cache de arquivos por tab (writable p/ reatividade do derived currentFiles).
export const filesByTab = writable<Map<string, FileEntry[]>>(new Map());

// Home memorizada p/ fallback do closeTab.
let homePath = '';

// ── Derived ─────────────────────────────────────────────────────
// Tab atualmente ativa.
export const activeTab = derived(
	[tabs, activeTabId],
	([$tabs, $activeTabId]) => $tabs.find((t) => t.id === $activeTabId) ?? null
);

// Arquivos da tab ativa (lidos do cache por tab).
export const activeFiles = derived(
	[activeTab, filesByTab],
	([$activeTab, $filesByTab]) => ($activeTab ? ($filesByTab.get($activeTab.id) ?? []) : [])
);

// ── Helpers ─────────────────────────────────────────────────────
function titleFromPath(path: string): string {
	if (!path) return '';
	const sep = path.includes('\\') ? '\\' : '/';
	const parts = path.split(sep).filter((p) => p.length > 0);
	return parts.length ? parts[parts.length - 1] : path;
}

function patchTab(id: string, patch: Partial<Tab>): void {
	tabs.update((ts) => ts.map((t) => (t.id === id ? { ...t, ...patch } : t)));
}

// Carrega arquivos da tab sem mexer no histórico.
// O resumo do diretório é escaneado em paralelo (falha ignorada).
async function loadTabFiles(id: string, path: string): Promise<void> {
	patchTab(id, { isLoading: true, error: null });
	try {
		const [files, summary] = await Promise.all([
			listDirectory(path),
			scanDirectorySummary(path).catch(() => null)
		]);
		filesByTab.update((m) => {
			const n = new Map(m);
			n.set(id, files);
			return n;
		});
		patchTab(id, { isLoading: false, directorySummary: summary });
	} catch (err) {
		patchTab(id, {
			isLoading: false,
			error: err instanceof Error ? err.message : String(err)
		});
	}
}

// ── Funções ─────────────────────────────────────────────────────
export function createTab(path: string): Tab {
	return {
		id: crypto.randomUUID(),
		path,
		title: titleFromPath(path),
		history: [path],
		historyIndex: 0,
		viewMode: 'list',
		sortBy: 'name',
		sortDir: 'asc',
		columnWidths: {},
		selectedPaths: new Set(),
		scrollPosition: 0,
		isLoading: false,
		error: null,
		filter: { ...DEFAULT_FILTER },
		directorySummary: null
	};
}

export function openTab(path: string): void {
	const tab = createTab(path);
	tabs.update((ts) => [...ts, tab]);
	activeTabId.set(tab.id);
	// History já é [path]; carrega direto p/ não duplicar a entrada.
	loadTabFiles(tab.id, path);
}

export function closeTab(tabId: string): void {
	const ts = get(tabs);

	// Única tab: não fecha — reabre na home.
	if (ts.length <= 1) {
		const home = createTab(homePath);
		tabs.set([home]);
		filesByTab.set(new Map());
		activeTabId.set(home.id);
		loadTabFiles(home.id, home.path);
		return;
	}

	const idx = ts.findIndex((t) => t.id === tabId);
	if (idx < 0) return;

	const wasActive = get(activeTabId) === tabId;
	const next = ts.filter((t) => t.id !== tabId);
	tabs.set(next);
	filesByTab.update((m) => {
		const n = new Map(m);
		n.delete(tabId);
		return n;
	});

	if (wasActive) {
		// Esquerda; senão direita (que tomou o índice idx); senão primeira.
		const neighbor = next[idx - 1] ?? next[idx] ?? next[0];
		activateTab(neighbor.id);
	}
}

export function activateTab(tabId: string): void {
	activeTabId.set(tabId);
	const t = get(tabs).find((x) => x.id === tabId);
	if (!t) return;

	// Restaura scroll via evento customizado (FileList escuta).
	if (typeof window !== 'undefined') {
		window.dispatchEvent(
			new CustomEvent('tab-restore-scroll', {
				detail: { tabId, position: t.scrollPosition }
			})
		);
	}

	// Carrega se ainda não houver arquivos em cache p/ esta tab.
	if (!get(filesByTab).has(tabId)) {
		loadTabFiles(tabId, t.path);
	}
}

export function navigateTab(tabId: string, path: string): void {
	const t = get(tabs).find((x) => x.id === tabId);
	if (!t) return;

	// Sem duplicar entrada igual consecutiva no histórico.
	if (t.history[t.historyIndex] === path) return;

	// Descarta entradas após historyIndex e empurra novo destino.
	const history = [...t.history.slice(0, t.historyIndex + 1), path];
	patchTab(tabId, {
		path,
		title: titleFromPath(path),
		history,
		historyIndex: history.length - 1
	});
	loadTabFiles(tabId, path);
}

export function tabGoBack(tabId: string): void {
	const t = get(tabs).find((x) => x.id === tabId);
	if (!t || t.historyIndex <= 0) return;
	const idx = t.historyIndex - 1;
	const path = t.history[idx];
	patchTab(tabId, { historyIndex: idx, path, title: titleFromPath(path) });
	loadTabFiles(tabId, path);
}

export function tabGoForward(tabId: string): void {
	const t = get(tabs).find((x) => x.id === tabId);
	if (!t || t.historyIndex >= t.history.length - 1) return;
	const idx = t.historyIndex + 1;
	const path = t.history[idx];
	patchTab(tabId, { historyIndex: idx, path, title: titleFromPath(path) });
	loadTabFiles(tabId, path);
}

export function duplicateTab(tabId: string): void {
	const ts = get(tabs);
	const idx = ts.findIndex((x) => x.id === tabId);
	if (idx < 0) return;

	const orig = ts[idx];
	const clone: Tab = {
		...orig,
		id: crypto.randomUUID(),
		history: [...orig.history],
		columnWidths: { ...orig.columnWidths },
		selectedPaths: new Set(orig.selectedPaths),
		filter: structuredClone(orig.filter)
	};
	tabs.update((list) => [...list.slice(0, idx + 1), clone, ...list.slice(idx + 1)]);

	// Copia o cache de arquivos p/ não recarregar.
	filesByTab.update((m) => {
		const n = new Map(m);
		const f = n.get(orig.id);
		if (f) n.set(clone.id, [...f]);
		return n;
	});

	activeTabId.set(clone.id);
}

export function updateTabScroll(tabId: string, position: number): void {
	patchTab(tabId, { scrollPosition: position });
}

export function updateTabViewMode(tabId: string, viewMode: Tab['viewMode']): void {
	patchTab(tabId, { viewMode });
}

export function updateTabSort(tabId: string, sortBy: Tab['sortBy'], sortDir: Tab['sortDir']): void {
	patchTab(tabId, { sortBy, sortDir });
}

export function updateTabColumnWidth(tabId: string, column: string, width: number): void {
	const t = get(tabs).find((x) => x.id === tabId);
	if (!t) return;
	patchTab(tabId, { columnWidths: { ...t.columnWidths, [column]: width } });
}

export function selectPath(
	tabId: string,
	path: string,
	mode: 'single' | 'toggle' | 'range'
): void {
	const t = get(tabs).find((x) => x.id === tabId);
	if (!t) return;
	const sel = new Set(t.selectedPaths);

	if (mode === 'single') {
		sel.clear();
		sel.add(path);
		selectionAnchors.set(tabId, path);
	} else if (mode === 'toggle') {
		if (sel.has(path)) sel.delete(path);
		else sel.add(path);
		selectionAnchors.set(tabId, path);
	} else {
		// range: do âncora até este, na ordem em que a lista está exibida.
		const files = get(filesByTab).get(tabId) ?? [];
		const ordered = sortEntries(files, t.sortBy, t.sortDir).map((e) => e.path);
		const anchor = selectionAnchors.get(tabId) ?? path;
		const a = ordered.indexOf(anchor);
		const b = ordered.indexOf(path);
		if (a >= 0 && b >= 0) {
			const [lo, hi] = a < b ? [a, b] : [b, a];
			sel.clear();
			for (let i = lo; i <= hi; i++) sel.add(ordered[i]);
		} else {
			sel.add(path);
		}
		// âncora não muda no range.
	}

	patchTab(tabId, { selectedPaths: sel });
}

export function clearSelection(tabId: string): void {
	patchTab(tabId, { selectedPaths: new Set() });
	selectionAnchors.delete(tabId);
}

export function updateTabFilter(tabId: string, patch: Partial<FilterState>): void {
	const t = get(tabs).find((x) => x.id === tabId);
	if (!t) return;
	// Filtro aplicado no frontend → não recarrega arquivos.
	patchTab(tabId, { filter: { ...t.filter, ...patch } });
}

export function resetTabFilter(tabId: string): void {
	patchTab(tabId, { filter: { ...DEFAULT_FILTER } });
}

export function updateTabSummary(tabId: string, summary: DirectorySummary): void {
	patchTab(tabId, { directorySummary: summary });
}

/** Recarrega os arquivos da tab sem alterar o histórico (ex: toggle ocultos). */
export function refreshTab(tabId: string): void {
	const t = get(tabs).find((x) => x.id === tabId);
	if (t) loadTabFiles(tabId, t.path);
}

export function initTabs(homePathArg: string): void {
	homePath = homePathArg;
	tabs.set([]);
	activeTabId.set(null);
	filesByTab.set(new Map());
	openTab(homePathArg);
}

/** Home memorizada (p/ botão "+" e Ctrl+T). */
export function getHomePath(): string {
	return homePath;
}

/** Fecha todas as tabs exceto a informada. */
export function closeOtherTabs(tabId: string): void {
	const ts = get(tabs);
	const keep = ts.find((t) => t.id === tabId);
	if (!keep) return;
	tabs.set([keep]);
	filesByTab.update((m) => {
		const n = new Map<string, FileEntry[]>();
		const f = m.get(tabId);
		if (f) n.set(tabId, f);
		return n;
	});
	activeTabId.set(tabId);
}

/** Fecha as tabs à direita da informada. */
export function closeTabsToRight(tabId: string): void {
	const ts = get(tabs);
	const idx = ts.findIndex((t) => t.id === tabId);
	if (idx < 0) return;
	const kept = ts.slice(0, idx + 1);
	const keepIds = new Set(kept.map((t) => t.id));
	tabs.set(kept);
	filesByTab.update((m) => {
		const n = new Map<string, FileEntry[]>();
		for (const [k, v] of m) if (keepIds.has(k)) n.set(k, v);
		return n;
	});
	if (!keepIds.has(get(activeTabId) ?? '')) activeTabId.set(tabId);
}

/** Reordena a tab arrastada para a posição-alvo. */
export function reorderTab(fromId: string, toIndex: number): void {
	tabs.update((ts) => {
		const from = ts.findIndex((t) => t.id === fromId);
		if (from < 0) return ts;
		const arr = [...ts];
		const [moved] = arr.splice(from, 1);
		const target = from < toIndex ? toIndex - 1 : toIndex;
		arr.splice(Math.max(0, Math.min(target, arr.length)), 0, moved);
		return arr;
	});
}
