import { writable, get } from 'svelte/store';
import {
	getHomeDirectory,
	listDirectory,
	getBookmarks,
	addBookmark,
	removeBookmark,
	reorderBookmarks,
	renameBookmark,
	setBookmarkIcon,
	type Bookmark
} from '$lib/tauri';

// ── Stores ──────────────────────────────────────────────────────
export const bookmarks = writable<Bookmark[]>([]);
export const bookmarksLoading = writable<boolean>(false);

export interface QuickAccessItem {
	id: string;
	label: string;
	icon: string;
	path: string;
}

export const quickAccessItems = writable<QuickAccessItem[]>([]);

// ── Toasts ──────────────────────────────────────────────────────
interface Toast {
	id: string;
	message: string;
	type: 'success' | 'error' | 'info';
	duration: number;
}

export const toasts = writable<Toast[]>([]);

export function showToast(
	message: string,
	type: Toast['type'] = 'info',
	duration = 3000
): void {
	const id = crypto.randomUUID();
	toasts.update((ts) => [...ts, { id, message, type, duration }]);
	setTimeout(() => {
		toasts.update((ts) => ts.filter((t) => t.id !== id));
	}, duration);
}

// ── Quick access ────────────────────────────────────────────────
function join(base: string, sub: string): string {
	const sep = base.includes('\\') ? '\\' : '/';
	return `${base}${sep}${sub}`;
}

async function resolveQuickAccess(home: string): Promise<QuickAccessItem[]> {
	const candidates: QuickAccessItem[] = [
		{ id: 'home', label: 'Home', icon: '🏠', path: home },
		{ id: 'desktop', label: 'Área de Trabalho', icon: '🖥️', path: join(home, 'Desktop') },
		{ id: 'downloads', label: 'Downloads', icon: '⬇️', path: join(home, 'Downloads') },
		{ id: 'documents', label: 'Documentos', icon: '📄', path: join(home, 'Documents') },
		{ id: 'pictures', label: 'Imagens', icon: '🖼️', path: join(home, 'Pictures') }
	];

	// Mantém só os paths que existem (listDirectory falha em inexistentes).
	const checked = await Promise.all(
		candidates.map(async (item) => {
			if (item.id === 'home') return item; // home sempre existe
			try {
				await listDirectory(item.path);
				return item;
			} catch {
				return null;
			}
		})
	);

	return checked.filter((x): x is QuickAccessItem => x !== null);
}

// ── Ações ───────────────────────────────────────────────────────
export async function initBookmarks(): Promise<void> {
	bookmarksLoading.set(true);
	try {
		const home = await getHomeDirectory();
		if (home) {
			quickAccessItems.set(await resolveQuickAccess(home));
		}
		bookmarks.set(await getBookmarks());
	} catch {
		// Falha silenciosa: bookmarks vazios não quebram o app.
		bookmarks.set([]);
	} finally {
		bookmarksLoading.set(false);
	}
}

export async function addBookmarkFromPath(path: string, name?: string): Promise<void> {
	try {
		const created = await addBookmark(path, name);
		bookmarks.update((bs) => [...bs, created]);
		showToast('Bookmark adicionado', 'success');
	} catch (err) {
		const msg = String(err);
		if (msg.includes('já existe') || msg.toLowerCase().includes('exists')) {
			showToast('Este local já está nos bookmarks', 'info');
		} else {
			showToast(`Falha ao adicionar bookmark: ${msg}`, 'error');
		}
	}
}

export async function removeBookmarkById(id: string): Promise<void> {
	try {
		await removeBookmark(id);
		bookmarks.update((bs) => bs.filter((b) => b.id !== id));
	} catch (err) {
		showToast(`Falha ao remover bookmark: ${String(err)}`, 'error');
	}
}

export async function reorderBookmarksLocally(newOrder: Bookmark[]): Promise<void> {
	const previous = get(bookmarks);
	bookmarks.set(newOrder); // optimistic
	try {
		const updated = await reorderBookmarks(newOrder.map((b) => b.id));
		bookmarks.set(updated);
	} catch (err) {
		bookmarks.set(previous); // reverte
		showToast(`Falha ao reordenar: ${String(err)}`, 'error');
	}
}

export async function renameBookmarkById(id: string, newName: string): Promise<void> {
	const previous = get(bookmarks);
	bookmarks.update((bs) => bs.map((b) => (b.id === id ? { ...b, name: newName } : b)));
	try {
		const updated = await renameBookmark(id, newName);
		bookmarks.update((bs) => bs.map((b) => (b.id === id ? updated : b)));
	} catch (err) {
		bookmarks.set(previous); // reverte
		showToast(`Falha ao renomear: ${String(err)}`, 'error');
	}
}

export async function setBookmarkIconById(id: string, icon: string | null): Promise<void> {
	const previous = get(bookmarks);
	bookmarks.update((bs) => bs.map((b) => (b.id === id ? { ...b, icon } : b)));
	try {
		const updated = await setBookmarkIcon(id, icon);
		bookmarks.update((bs) => bs.map((b) => (b.id === id ? updated : b)));
	} catch (err) {
		bookmarks.set(previous); // reverte
		showToast(`Falha ao alterar ícone: ${String(err)}`, 'error');
	}
}

export function isBookmarked(path: string): boolean {
	return get(bookmarks).some((b) => b.path === path);
}
