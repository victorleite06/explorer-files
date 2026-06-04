<script lang="ts">
	import { onMount } from 'svelte';
	import { get } from 'svelte/store';
	import { activeTabId, navigateTab, openTab } from '$lib/stores/tabs';
	import { bookmarks, addBookmarkFromPath, type QuickAccessItem } from '$lib/stores/bookmarks';

	let { item, isActive }: { item: QuickAccessItem; isActive: boolean } = $props();

	let menu = $state<{ visible: boolean; x: number; y: number }>({ visible: false, x: 0, y: 0 });
	let already = $derived($bookmarks.some((b) => b.path === item.path));

	function open(e: MouseEvent) {
		if (e.ctrlKey) openTab(item.path);
		else {
			const id = get(activeTabId);
			if (id) navigateTab(id, item.path);
		}
	}

	function onDragStart(e: DragEvent) {
		if (!e.dataTransfer) return;
		e.dataTransfer.setData('bookmark-drop-path', item.path);
		e.dataTransfer.effectAllowed = 'copy';
		window.dispatchEvent(
			new CustomEvent('quickaccess:drag', {
				detail: { path: item.path, label: item.label, icon: item.icon }
			})
		);
	}

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
		navigator.clipboard?.writeText(item.path).catch(() => {});
		closeMenu();
	}

	onMount(() => {
		document.addEventListener('click', closeMenu);
		return () => document.removeEventListener('click', closeMenu);
	});
</script>

<button
	class="qa"
	class:active={isActive}
	title={item.path}
	draggable="true"
	onclick={open}
	ondragstart={onDragStart}
	oncontextmenu={openMenu}
>
	<span class="ico">{item.icon}</span>
	<span class="label">{item.label}</span>
</button>

{#if menu.visible}
	<div class="ctx" style="left: {menu.x}px; top: {menu.y}px" role="menu" tabindex="-1">
		<button role="menuitem" onclick={() => act(() => openTab(item.path))}>Abrir em nova tab</button>
		<button role="menuitem" onclick={copyPath}>Copiar caminho</button>
		<button
			role="menuitem"
			disabled={already}
			onclick={() => act(() => addBookmarkFromPath(item.path, item.label))}
		>
			Adicionar aos favoritos
		</button>
	</div>
{/if}

<style>
	.qa {
		display: flex;
		align-items: center;
		gap: 8px;
		width: 100%;
		height: 28px;
		padding: 0 12px;
		border: none;
		border-left: 2px solid transparent;
		background: transparent;
		color: #666;
		font-family: 'DM Sans', system-ui, sans-serif;
		font-size: 13px;
		text-align: left;
		cursor: pointer;
		transition:
			background 100ms ease,
			color 100ms ease;
	}
	.qa:hover {
		background: #111120;
		color: #9090a8;
	}
	.qa.active {
		background: #16162e;
		color: #dddde8;
		border-left: 2px solid #a78bfa;
	}
	.ico {
		font-size: 16px;
		flex-shrink: 0;
	}
	.label {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.ctx {
		position: fixed;
		z-index: 1000;
		min-width: 180px;
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
	.ctx button:hover:not(:disabled) {
		background: #1e1e35;
	}
	.ctx button:disabled {
		opacity: 0.4;
		cursor: default;
	}
</style>
