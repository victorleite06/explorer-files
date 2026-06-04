import { writable, derived, get } from 'svelte/store';
import { getDirectoryTree, getHomeDirectory, type TreeNode } from '$lib/tauri';
import { activeTab, activeTabId, filesByTab, navigateTab, initTabs } from './tabs';
import { initBookmarks } from './bookmarks';
import { initSettings } from './settings';
import { applyFilters } from '$lib/utils/filterUtils';

// ── currentPath / currentFiles agora derivam da tab ativa ───────
export const currentPath = derived(activeTab, ($t) => $t?.path ?? '');

export const currentFiles = derived([activeTab, filesByTab], ([$t, $m]) =>
	$t ? ($m.get($t.id) ?? []) : []
);

// Arquivos da tab ativa já com os filtros aplicados (frontend).
export const filteredFiles = derived([activeTab, filesByTab], ([$t, $m]) => {
	if (!$t) return [];
	const files = $m.get($t.id) ?? [];
	return applyFilters(files, $t.filter);
});

// Loading e erro também passam a vir da tab ativa.
export const isLoading = derived(activeTab, ($t) => $t?.isLoading ?? false);
export const explorerError = derived(activeTab, ($t) => $t?.error ?? null);

// ── Tree view (estado global, não por-tab) ──────────────────────
export const treeRoot = writable<TreeNode | null>(null);
export const expandedPaths = writable<Set<string>>(new Set());

/** Navega para uma pasta. Sem tabId usa a tab ativa. */
export function navigateTo(path: string, tabId?: string): void {
	const id = tabId ?? get(activeTabId);
	if (!id) return;
	navigateTab(id, path);
}

/** Inicializa o explorer: home → tabs → tree. */
export async function initExplorer(): Promise<void> {
	const [home] = await Promise.all([getHomeDirectory(), initSettings(), initBookmarks()]);
	const root = home ?? '/';

	// Abre a primeira tab na home (settings/bookmarks já carregados).
	initTabs(root);

	// Monta a tree do painel esquerdo.
	try {
		const tree = await getDirectoryTree(root, 2);
		treeRoot.set(tree);
		expandedPaths.update((s) => {
			s.add(root);
			return new Set(s);
		});
	} catch {
		// Tree é secundária; falha aqui não bloqueia a navegação.
	}
}

// ── Tree: merge / lookup ────────────────────────────────────────
function mergeChildren(node: TreeNode, path: string, children: TreeNode[]): boolean {
	if (node.path === path) {
		node.children = children;
		node.is_expanded = true;
		return true;
	}
	for (const child of node.children) {
		if (mergeChildren(child, path, children)) return true;
	}
	return false;
}

function findNode(node: TreeNode, path: string): TreeNode | null {
	if (node.path === path) return node;
	for (const child of node.children) {
		const found = findNode(child, path);
		if (found) return found;
	}
	return null;
}

/** Expande nó na tree: carrega filhos sob demanda e faz merge no treeRoot. */
export async function expandNode(path: string): Promise<void> {
	expandedPaths.update((s) => {
		s.add(path);
		return new Set(s);
	});

	const root = get(treeRoot);
	if (!root) return;

	const target = findNode(root, path);
	if (target && target.children.length > 0) return;

	try {
		const subtree = await getDirectoryTree(path, 1);
		treeRoot.update((r) => {
			if (!r) return r;
			mergeChildren(r, path, subtree.children);
			return { ...r };
		});
	} catch {
		// Falha ao expandir não deve quebrar a tree.
	}
}

/** Colapsa nó na tree (remove do Set de expandidos). */
export function collapseNode(path: string): void {
	expandedPaths.update((s) => {
		s.delete(path);
		return new Set(s);
	});
}
