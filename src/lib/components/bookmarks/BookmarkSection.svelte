<script lang="ts">
	import { get } from 'svelte/store';
	import { isTauri } from '$lib/tauri';
	import { bookmarks, addBookmarkFromPath, reorderBookmarksLocally } from '$lib/stores/bookmarks';
	import { currentPath } from '$lib/stores/explorer';
	import BookmarkItem from './BookmarkItem.svelte';

	let collapsed = $state(false);
	let draggedId = $state<string | null>(null);
	let dragOverId = $state<string | null>(null);
	let dzOver = $state(false);

	// ── Reordenação ───────────────────────────────────────────────
	function reorder(draggedId: string, targetId: string) {
		if (draggedId === targetId) return;
		const list = [...get(bookmarks)];
		const from = list.findIndex((b) => b.id === draggedId);
		const to = list.findIndex((b) => b.id === targetId);
		if (from < 0 || to < 0) return;
		const [moved] = list.splice(from, 1);
		const insertAt = list.findIndex((b) => b.id === targetId);
		list.splice(insertAt, 0, moved);
		reorderBookmarksLocally(list);
	}

	function onDropItem(targetId: string) {
		if (draggedId) reorder(draggedId, targetId);
		draggedId = null;
		dragOverId = null;
	}
	function onDragEndItem() {
		draggedId = null;
		dragOverId = null;
	}

	// ── Drop zone ─────────────────────────────────────────────────
	function dzDragOver(e: DragEvent) {
		e.preventDefault();
		dzOver = true;
	}
	function dzDragLeave() {
		dzOver = false;
	}
	function dzDrop(e: DragEvent) {
		e.preventDefault();
		e.stopPropagation(); // evita o handler global do SidePanel re-adicionar
		dzOver = false;
		const dt = e.dataTransfer;
		if (!dt) return;

		const path = dt.getData('bookmark-drop-path');
		if (path) {
			addBookmarkFromPath(path);
			return;
		}
		// Reordenar para o fim.
		const id = dt.getData('bookmark-id') || draggedId;
		if (id) {
			const list = [...get(bookmarks)];
			const from = list.findIndex((b) => b.id === id);
			if (from >= 0) {
				const [moved] = list.splice(from, 1);
				list.push(moved);
				reorderBookmarksLocally(list);
			}
		}
		draggedId = null;
	}

	// ── Botão + (file picker nativo) ──────────────────────────────
	async function pickFolder() {
		if (!isTauri()) return;
		try {
			const { open } = await import('@tauri-apps/plugin-dialog');
			const sel = await open({ directory: true, multiple: false });
			if (typeof sel === 'string') addBookmarkFromPath(sel);
		} catch {
			/* cancelado ou indisponível */
		}
	}

	let activePath = $derived($currentPath);
</script>

<section class="section">
	<div class="header">
		<button class="title" onclick={() => (collapsed = !collapsed)}>
			<span class="arrow" class:open={!collapsed}>▶</span>
			<span>Favoritos</span>
		</button>
		<button class="add" title="Adicionar pasta" onclick={pickFolder} aria-label="Adicionar pasta">+</button>
	</div>

	{#if !collapsed}
		<div class="body">
			{#if $bookmarks.length > 0}
				{#each $bookmarks as bm (bm.id)}
					<BookmarkItem
						bookmark={bm}
						isActive={bm.path === activePath}
						isDragging={draggedId === bm.id}
						isDragOver={dragOverId === bm.id}
						onDragStartItem={(id) => (draggedId = id)}
						onDragOverItem={(id) => (dragOverId = id)}
						{onDropItem}
						{onDragEndItem}
					/>
				{/each}

				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<div
					class="dropzone"
					class:over={dzOver}
					ondragover={dzDragOver}
					ondragleave={dzDragLeave}
					ondrop={dzDrop}
				>
					··· solte aqui para adicionar ···
				</div>
			{:else}
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<div
					class="empty dropzone"
					class:over={dzOver}
					ondragover={dzDragOver}
					ondragleave={dzDragLeave}
					ondrop={dzDrop}
				>
					<span class="empty-ico">🔖</span>
					<span>Arraste pastas aqui ou clique em +</span>
				</div>
			{/if}
		</div>
	{/if}
</section>

<style>
	.section {
		display: flex;
		flex-direction: column;
		font-family: 'DM Sans', system-ui, sans-serif;
	}
	.header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 8px 12px 4px;
	}
	.title {
		display: flex;
		align-items: center;
		gap: 6px;
		border: none;
		background: none;
		color: #333;
		font-family: inherit;
		font-size: 10px;
		text-transform: uppercase;
		letter-spacing: 0.1em;
		cursor: pointer;
	}
	.title:hover {
		color: #555;
	}
	.arrow {
		font-size: 7px;
		transition: transform 120ms ease;
	}
	.arrow.open {
		transform: rotate(90deg);
	}
	.add {
		width: 18px;
		height: 18px;
		border: none;
		border-radius: 4px;
		background: none;
		color: #444;
		font-size: 14px;
		line-height: 1;
		cursor: pointer;
		transition:
			background 120ms ease,
			color 120ms ease;
	}
	.add:hover {
		background: #1a1a2e;
		color: #a78bfa;
	}

	.body {
		display: flex;
		flex-direction: column;
	}
	.dropzone {
		padding: 8px;
		text-align: center;
		color: #2a2a42;
		font-size: 11px;
		border: 1px solid transparent;
		border-radius: 6px;
		margin: 4px 8px;
		transition:
			background 120ms ease,
			border-color 120ms ease;
	}
	.dropzone.over {
		background: #1a1a2e;
		border: 1px dashed #a78bfa55;
	}
	.empty {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 6px;
		padding: 16px 8px;
		color: #444;
	}
	.empty-ico {
		font-size: 24px;
	}
</style>
