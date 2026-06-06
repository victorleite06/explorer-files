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

// ── Busca por nome ──────────────────────────────────────────────
export interface SearchResult {
	path: string;
	name: string;
	is_dir: boolean;
	size: number;
	modified?: string;
	extension?: string;
	score: number;
	match_indices: number[];
	parent_path: string;
}

export interface SearchOptions {
	max_results?: number;
	include_dirs?: boolean;
	include_files?: boolean;
	recursive?: boolean;
	max_depth?: number;
	extensions?: string[];
}

export async function searchFiles(
	query: string,
	path: string,
	options?: SearchOptions
): Promise<SearchResult[]> {
	if (!isTauri()) return [];
	return invoke<SearchResult[]>('search_files', { query, path, options: options ?? null });
}

export async function searchFilesQuick(query: string, path: string): Promise<SearchResult[]> {
	if (!isTauri()) return [];
	return invoke<SearchResult[]>('search_files_quick', { query, path });
}

// ── Indexação / busca por conteúdo ──────────────────────────────
export interface IndexStats {
	total_documents: number;
	index_size_bytes: number;
	last_updated: string | null;
	is_indexing: boolean;
}

export interface IndexingProgressEvent {
	session_id: string;
	total: number;
	processed: number;
	current_file: string;
	errors: number;
	is_complete: boolean;
	eta_seconds: number | null;
}

export interface ContentSearchResult {
	path: string;
	name: string;
	extension: string | null;
	size: number;
	modified: string | null;
	score: number;
	previews: string[];
	match_count: number;
	parent_path: string;
}

export interface ContentSearchQuery {
	query: string;
	root_path?: string | null;
	extensions?: string[];
	max_results?: number;
	include_previews?: boolean;
	preview_chars?: number;
}

export async function getIndexStats(): Promise<IndexStats> {
	return invoke<IndexStats>('get_index_stats');
}

export async function startIndexing(path: string): Promise<string> {
	return invoke<string>('start_indexing', { path });
}

export async function watchDirectory(path: string): Promise<void> {
	return invoke('watch_directory', { path });
}

export async function unwatchDirectory(path: string): Promise<void> {
	return invoke('unwatch_directory', { path });
}

export async function searchContent(query: ContentSearchQuery): Promise<ContentSearchResult[]> {
	if (!isTauri()) return [];
	return invoke<ContentSearchResult[]>('search_content', { query });
}

export async function clearIndex(): Promise<void> {
	return invoke('clear_index');
}

// ── Settings / ignore rules ─────────────────────────────────────
export interface IgnoreRules {
	show_dotfiles: boolean;
	show_node_modules: boolean;
	show_build_artifacts: boolean;
	custom_hidden: string[];
	custom_shown: string[];
}

export interface GitignoreSettings {
	respect_gitignore: boolean;
	respect_global_gitignore: boolean;
	respect_ignore_files: boolean;
}

export interface GitignoreInfo {
	is_git_repo: boolean;
	gitignore_files: string[];
	ignored_count: number;
	active: boolean;
}

export const DEFAULT_GITIGNORE_SETTINGS: GitignoreSettings = {
	respect_gitignore: true,
	respect_global_gitignore: true,
	respect_ignore_files: true
};

export interface AppSettings {
	version: number;
	ignore_rules: IgnoreRules;
	gitignore: GitignoreSettings;
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

export async function getGitignoreInfo(path: string): Promise<GitignoreInfo> {
	return invoke<GitignoreInfo>('get_gitignore_info', { path });
}

export async function updateGitignoreSettings(
	gitignore: GitignoreSettings
): Promise<AppSettings> {
	return invoke<AppSettings>('update_gitignore_settings', { gitignore });
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
