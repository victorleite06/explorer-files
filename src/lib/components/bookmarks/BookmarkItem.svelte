<script lang="ts">
	import { onMount, tick } from 'svelte';
	import { get } from 'svelte/store';
	import { activeTabId, navigateTab, openTab } from '$lib/stores/tabs';
	import {
		removeBookmarkById,
		renameBookmarkById,
		setBookmarkIconById
	} from '$lib/stores/bookmarks';
	import type { Bookmark } from '$lib/tauri';
	import EmojiPicker from './EmojiPicker.svelte';

	let {
		bookmark,
		isActive,
		isDragging,
		isDragOver,
		onDragStartItem,
		onDragOverItem,
		onDropItem,
		onDragEndItem
	}: {
		bookmark: Bookmark;
		isActive: boolean;
		isDragging: boolean;
		isDragOver: boolean;
		onDragStartItem: (id: string) => void;
		onDragOverItem: (id: string) => void;
		onDropItem: (targetId: string) => void;
		onDragEndItem: () => void;
	} = $props();

	let editing = $state(false);
	let editValue = $state('');
	let inputEl = $state<HTMLInputElement>();

	let menu = $state<{ visible: boolean; x: number; y: number }>({ visible: false, x: 0, y: 0 });
	let showPicker = $state(false);

	// ── Navegação ─────────────────────────────────────────────────
	function open(e: MouseEvent) {
		if (editing) return;
		if (e.ctrlKey) openTab(bookmark.path);
		else {
			const id = get(activeTabId);
			if (id) navigateTab(id, bookmark.path);
		}
	}
	function remove(e: MouseEvent) {
		e.stopPropagation();
		removeBookmarkById(bookmark.id);
	}

	// ── Drag & drop (reordenação) ─────────────────────────────────
	function onDragStart(e: DragEvent) {
		if (!e.dataTransfer) return;
		e.dataTransfer.setData('bookmark-id', bookmark.id);
		e.dataTransfer.effectAllowed = 'move';
		onDragStartItem(bookmark.id);
	}
	function onDragOver(e: DragEvent) {
		e.preventDefault();
		if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
		onDragOverItem(bookmark.id);
	}
	function onDrop(e: DragEvent) {
		e.preventDefault();
		e.stopPropagation();
		onDropItem(bookmark.id);
	}

	// ── Rename inline ─────────────────────────────────────────────
	async function startEdit() {
		editing = true;
		editValue = bookmark.name;
		closeMenu();
		await tick();
		inputEl?.focus();
		inputEl?.select();
	}
	function commitEdit() {
		const v = editValue.trim();
		if (v && v !== bookmark.name) renameBookmarkById(bookmark.id, v);
		editing = false;
	}
	function cancelEdit() {
		editing = false;
	}
	function onEditKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			e.preventDefault();
			commitEdit();
		} else if (e.key === 'Escape') {
			e.preventDefault();
			cancelEdit();
		}
	}

	// ── Menu de contexto ──────────────────────────────────────────
	function openMenu(e: MouseEvent) {
		e.preventDefault();
		menu = { visible: true, x: e.clientX, y: e.clientY };
	}
	function closeMenu() {
		menu = { ...menu, visible: false };
	}
	function act(fn: () => void) {
		fn();
		closeMenu();
	}
	function copyPath() {
		navigator.clipboard?.writeText(bookmark.path).catch(() => {});
		closeMenu();
	}
	function navigate() {
		const id = get(activeTabId);
		if (id) navigateTab(id, bookmark.path);
	}

	onMount(() => {
		document.addEventListener('click', closeMenu);
		return () => document.removeEventListener('click', closeMenu);
	});
</script>

<div class="wrap">
	<div
		class="bm"
		class:active={isActive}
		class:dragging={isDragging}
		class:dragover={isDragOver}
		role="button"
		tabindex="0"
		title={bookmark.path}
		draggable={!editing}
		onclick={open}
		onkeydown={(e) => e.key === 'Enter' && !editing && navigate()}
		oncontextmenu={openMenu}
		ondragstart={onDragStart}
		ondragover={onDragOver}
		ondrop={onDrop}
		ondragend={onDragEndItem}
	>
		<span class="ico">{bookmark.icon ?? '📁'}</span>
		{#if editing}
			<input
				bind:this={inputEl}
				bind:value={editValue}
				class="edit"
				spellcheck="false"
				onkeydown={onEditKeydown}
				onblur={commitEdit}
				onclick={(e) => e.stopPropagation()}
			/>
		{:else}
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<span class="label" ondblclick={startEdit}>{bookmark.name}</span>
			<button class="rm" title="Remover" onclick={remove} tabindex="-1">×</button>
		{/if}
	</div>

	{#if showPicker}
		<EmojiPicker
			onSelect={(emoji) => setBookmarkIconById(bookmark.id, emoji)}
			onClose={() => (showPicker = false)}
		/>
	{/if}
</div>

{#if menu.visible}
	<div class="ctx" style="left: {menu.x}px; top: {menu.y}px" role="menu" tabindex="-1">
		<button role="menuitem" onclick={() => act(navigate)}>Abrir</button>
		<button role="menuitem" onclick={() => act(() => openTab(bookmark.path))}>Abrir em nova tab</button>
		<div class="sep"></div>
		<button role="menuitem" onclick={startEdit}>Renomear</button>
		<button role="menuitem" onclick={() => act(() => (showPicker = true))}>Alterar ícone</button>
		<button role="menuitem" onclick={() => act(() => setBookmarkIconById(bookmark.id, null))}>
			Redefinir ícone
		</button>
		<div class="sep"></div>
		<button role="menuitem" onclick={copyPath}>Copiar caminho</button>
		<div class="sep"></div>
		<button role="menuitem" class="danger" onclick={() => act(() => removeBookmarkById(bookmark.id))}>
			Remover dos favoritos
		</button>
	</div>
{/if}

<style>
	.wrap {
		position: relative;
	}
	.bm {
		display: flex;
		align-items: center;
		gap: 8px;
		width: 100%;
		height: 28px;
		padding: 0 12px;
		border-left: 2px solid transparent;
		border-top: 1px solid transparent;
		border-bottom: 1px solid transparent;
		background: transparent;
		color: #666;
		font-family: 'DM Sans', system-ui, sans-serif;
		font-size: 13px;
		cursor: pointer;
		transition:
			background 100ms ease,
			color 100ms ease;
	}
	.bm:hover {
		background: #111120;
		color: #9090a8;
	}
	.bm.active {
		background: #16162e;
		color: #dddde8;
		border-left: 2px solid #a78bfa;
	}
	.bm.dragging {
		opacity: 0.4;
	}
	.bm.dragover {
		background: #1a1a2e;
		border: 1px dashed #a78bfa55;
	}
	.ico {
		font-size: 16px;
		flex-shrink: 0;
	}
	.label {
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.edit {
		flex: 1;
		min-width: 0;
		height: 20px;
		padding: 0 4px;
		border: none;
		border-radius: 3px;
		background: #1a1a2e;
		color: #dddde8;
		font-family: inherit;
		font-size: 13px;
		outline: none;
	}
	.rm {
		flex-shrink: 0;
		border: none;
		background: none;
		color: #2a2a42;
		font-size: 14px;
		line-height: 1;
		cursor: pointer;
		opacity: 0;
		transition:
			opacity 100ms ease,
			color 100ms ease;
	}
	.bm:hover .rm {
		opacity: 1;
	}
	.rm:hover {
		color: #f87171;
	}

	.ctx {
		position: fixed;
		z-index: 1000;
		min-width: 190px;
		padding: 4px;
		background: #14142a;
		border: 1px solid #1e1e35;
		border-radius: 8px;
		box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
		display: flex;
		flex-direction: column;
		font-family: 'DM Sans', system-ui, sans-serif;
	}
	.ctx button {
		padding: 7px 10px;
		border: none;
		border-radius: 5px;
		background: none;
		color: #c8c8dc;
		font-family: inherit;
		font-size: 13px;
		text-align: left;
		cursor: pointer;
	}
	.ctx button:hover {
		background: #1e1e35;
	}
	.ctx .danger:hover {
		background: #2e1a1a;
		color: #f87171;
	}
	.sep {
		height: 1px;
		margin: 4px 6px;
		background: #1e1e35;
	}
</style>
