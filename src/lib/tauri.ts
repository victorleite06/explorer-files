import { invoke } from '@tauri-apps/api/core';

export interface FileEntry {
	name: string;
	path: string;
	is_dir: boolean;
	size: number;
	modified?: string;
	created_at?: string;
	extension?: string;
	is_hidden: boolean;
	is_symlink: boolean;
}

export interface TreeNode {
	name: string;
	path: string;
	children: TreeNode[];
	is_expanded: boolean;
}

export const isTauri = (): boolean =>
	typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

export async function listDirectory(path: string): Promise<FileEntry[]> {
	if (!isTauri()) return [];
	try {
		return await invoke<FileEntry[]>('list_directory', { path });
	} catch (err) {
		throw new Error(`Falha ao listar "${path}": ${String(err)}`);
	}
}

export async function getDirectoryTree(path: string, depth = 2): Promise<TreeNode> {
	if (!isTauri()) {
		return { name: path, path, children: [], is_expanded: false };
	}
	try {
		return await invoke<TreeNode>('get_directory_tree', { path, depth });
	} catch (err) {
		throw new Error(`Falha ao montar árvore de "${path}": ${String(err)}`);
	}
}

export async function getHomeDirectory(): Promise<string | null> {
	if (!isTauri()) return null;
	try {
		return await invoke<string | null>('get_home_directory');
	} catch (err) {
		throw new Error(`Falha ao obter home: ${String(err)}`);
	}
}

export async function getDrives(): Promise<FileEntry[]> {
	if (!isTauri()) return [];
	try {
		return await invoke<FileEntry[]>('get_drives');
	} catch (err) {
		throw new Error(`Falha ao listar drives: ${String(err)}`);
	}
}

// ── Directory summary (filtros) ─────────────────────────────────
export interface ExtensionSummary {
	extension: string;
	count: number;
	total_size: number;
}

export interface DirectorySummary {
	total_files: number;
	total_dirs: number;
	total_size: number;
	smallest_file: number;
	largest_file: number;
	oldest_modified: string | null;
	newest_modified: string | null;
	extensions: ExtensionSummary[];
}

export async function scanDirectorySummary(path: string): Promise<DirectorySummary> {
	return invoke<DirectorySummary>('scan_directory_summary', { path });
}

// ── Settings / ignore rules ─────────────────────────────────────
export interface IgnoreRules {
	show_dotfiles: boolean;
	show_node_modules: boolean;
	show_build_artifacts: boolean;
	custom_hidden: string[];
	custom_shown: string[];
}

export interface AppSettings {
	version: number;
	ignore_rules: IgnoreRules;
}

export const DEFAULT_IGNORE_RULES: IgnoreRules = {
	show_dotfiles: false,
	show_node_modules: false,
	show_build_artifacts: false,
	custom_hidden: [],
	custom_shown: []
};

export async function getSettings(): Promise<AppSettings> {
	return invoke<AppSettings>('get_settings');
}

export async function updateIgnoreRules(rules: IgnoreRules): Promise<AppSettings> {
	return invoke<AppSettings>('update_ignore_rules', { rules });
}

// ── Bookmarks ───────────────────────────────────────────────────
export interface Bookmark {
	id: string;
	path: string;
	name: string;
	icon: string | null;
	created_at: string;
	position: number;
}

export async function getBookmarks(): Promise<Bookmark[]> {
	if (!isTauri()) return [];
	return invoke<Bookmark[]>('get_bookmarks');
}

export async function addBookmark(path: string, name?: string): Promise<Bookmark> {
	return invoke<Bookmark>('add_bookmark', { path, name: name ?? null });
}

export async function removeBookmark(id: string): Promise<void> {
	return invoke('remove_bookmark', { id });
}

export async function reorderBookmarks(orderedIds: string[]): Promise<Bookmark[]> {
	return invoke<Bookmark[]>('reorder_bookmarks', { orderedIds });
}

export async function renameBookmark(id: string, newName: string): Promise<Bookmark> {
	return invoke<Bookmark>('rename_bookmark', { id, newName });
}

export async function setBookmarkIcon(id: string, icon: string | null): Promise<Bookmark> {
	return invoke<Bookmark>('set_bookmark_icon', { id, icon });
}

export async function bookmarkExists(path: string): Promise<boolean> {
	if (!isTauri()) return false;
	return invoke<boolean>('bookmark_exists', { path });
}
