<script lang="ts">
	import { onMount } from 'svelte';
	import { get } from 'svelte/store';
	import {
		tabs,
		activeTabId,
		openTab,
		closeTab,
		activateTab,
		duplicateTab,
		closeOtherTabs,
		closeTabsToRight,
		reorderTab,
		getHomePath
	} from '$lib/stores/tabs';

	let container = $state<HTMLDivElement>();

	// Drag & drop
	let dragId = $state<string | null>(null);
	let dragOverIndex = $state<number | null>(null);

	// Menu de contexto
	let menu = $state<{ visible: boolean; x: number; y: number; tabId: string; tabPath: string }>({
		visible: false,
		x: 0,
		y: 0,
		tabId: '',
		tabPath: ''
	});
	let menuEl = $state<HTMLDivElement>();

	// ── Interações básicas ────────────────────────────────────────
	function newTab() {
		openTab(getHomePath());
	}

	function onTabMouseDown(e: MouseEvent, tabId: string) {
		// Botão do meio fecha a tab.
		if (e.button === 1) {
			e.preventDefault();
			closeTab(tabId);
		}
	}

	function onClose(e: MouseEvent, tabId: string) {
		e.stopPropagation();
		closeTab(tabId);
	}

	// Scroll horizontal com a roda do mouse.
	function onWheel(e: WheelEvent) {
		if (!container) return;
		if (e.deltaY !== 0) {
			e.preventDefault();
			container.scrollLeft += e.deltaY;
		}
	}

	// ── Drag & drop ───────────────────────────────────────────────
	function onDragStart(e: DragEvent, tabId: string) {
		dragId = tabId;
		e.dataTransfer?.setData('text/plain', tabId);
		if (e.dataTransfer) e.dataTransfer.effectAllowed = 'move';
	}

	function onDragOver(e: DragEvent, index: number) {
		e.preventDefault();
		if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
		dragOverIndex = index;
	}

	function onDrop(e: DragEvent, index: number) {
		e.preventDefault();
		if (dragId) reorderTab(dragId, index);
		dragId = null;
		dragOverIndex = null;
	}

	function onDragEnd() {
		dragId = null;
		dragOverIndex = null;
	}

	// ── Menu de contexto ──────────────────────────────────────────
	function openMenu(e: MouseEvent, tabId: string, tabPath: string) {
		e.preventDefault();
		menu = { visible: true, x: e.clientX, y: e.clientY, tabId, tabPath };
	}

	function closeMenu() {
		menu = { ...menu, visible: false };
	}

	// Clampa o menu p/ não sair da tela após renderizar.
	$effect(() => {
		if (menu.visible && menuEl) {
			const rect = menuEl.getBoundingClientRect();
			let { x, y } = menu;
			if (x + rect.width > window.innerWidth) x = window.innerWidth - rect.width - 4;
			if (y + rect.height > window.innerHeight) y = window.innerHeight - rect.height - 4;
			if (x !== menu.x || y !== menu.y) menu = { ...menu, x, y };
		}
	});

	function menuAction(fn: () => void) {
		fn();
		closeMenu();
	}

	async function copyPath(path: string) {
		try {
			await navigator.clipboard.writeText(path);
		} catch {
			/* clipboard pode falhar sem permissão; ignora */
		}
		closeMenu();
	}

	// ── Tab ativa sempre visível ──────────────────────────────────
	$effect(() => {
		const id = $activeTabId;
		if (!id || !container) return;
		const el = container.querySelector<HTMLElement>(`[data-tab-id="${id}"]`);
		el?.scrollIntoView({ behavior: 'smooth', inline: 'nearest', block: 'nearest' });
	});

	// ── Atalhos de teclado ────────────────────────────────────────
	function onKeydown(e: KeyboardEvent) {
		if (!e.ctrlKey) return;

		const list = get(tabs);
		const activeId = get(activeTabId);
		const idx = list.findIndex((t) => t.id === activeId);

		// Ctrl+Tab / Ctrl+Shift+Tab
		if (e.key === 'Tab') {
			e.preventDefault();
			if (list.length === 0) return;
			const next = e.shiftKey
				? (idx - 1 + list.length) % list.length
				: (idx + 1) % list.length;
			activateTab(list[next].id);
			return;
		}

		const k = e.key.toLowerCase();
		if (k === 't') {
			e.preventDefault();
			newTab();
		} else if (k === 'w') {
			e.preventDefault();
			if (activeId) closeTab(activeId);
		} else if (e.key >= '1' && e.key <= '9') {
			e.preventDefault();
			// Ctrl+9 = última; Ctrl+1..8 = índice direto.
			const target = e.key === '9' ? list.length - 1 : Number(e.key) - 1;
			if (target >= 0 && target < list.length) activateTab(list[target].id);
		}
	}

	onMount(() => {
		window.addEventListener('keydown', onKeydown);
		document.addEventListener('click', closeMenu);
		document.addEventListener('contextmenu', handleGlobalContext);
		return () => {
			window.removeEventListener('keydown', onKeydown);
			document.removeEventListener('click', closeMenu);
			document.removeEventListener('contextmenu', handleGlobalContext);
		};
	});

	// Fecha o menu se o clique direito for fora de uma tab.
	function handleGlobalContext(e: MouseEvent) {
		const target = e.target as HTMLElement;
		if (!target.closest('.tab')) closeMenu();
	}
</script>

<div class="tabbar">
	<div class="scroll" bind:this={container} onwheel={onWheel} role="tablist" tabindex="-1">
		{#each $tabs as tab, i (tab.id)}
			<div
				class="tab"
				class:active={tab.id === $activeTabId}
				class:dragging={tab.id === dragId}
				class:dropbefore={dragOverIndex === i && dragId !== null && dragId !== tab.id}
				data-tab-id={tab.id}
				draggable="true"
				role="tab"
				tabindex="0"
				aria-selected={tab.id === $activeTabId}
				title={tab.path}
				onclick={() => activateTab(tab.id)}
				onkeydown={(e) => e.key === 'Enter' && activateTab(tab.id)}
				onmousedown={(e) => onTabMouseDown(e, tab.id)}
				oncontextmenu={(e) => openMenu(e, tab.id, tab.path)}
				ondragstart={(e) => onDragStart(e, tab.id)}
				ondragover={(e) => onDragOver(e, i)}
				ondrop={(e) => onDrop(e, i)}
				ondragend={onDragEnd}
			>
				<span class="tab-ico">📁</span>
				<span class="tab-title">{tab.title || tab.path}</span>
				<button
					class="tab-close"
					title="Fechar"
					onclick={(e) => onClose(e, tab.id)}
					tabindex="-1"
				>
					×
				</button>
			</div>
		{/each}
	</div>

	<button class="new-tab" title="Nova tab (Ctrl+T)" onclick={newTab}>+</button>
</div>

{#if menu.visible}
	<div
		class="ctx-menu"
		bind:this={menuEl}
		style="left: {menu.x}px; top: {menu.y}px"
		role="menu"
		tabindex="-1"
	>
		<button role="menuitem" onclick={() => menuAction(() => closeTab(menu.tabId))}>Fechar tab</button>
		<button role="menuitem" onclick={() => menuAction(() => closeOtherTabs(menu.tabId))}>
			Fechar outras tabs
		</button>
		<button role="menuitem" onclick={() => menuAction(() => closeTabsToRight(menu.tabId))}>
			Fechar tabs à direita
		</button>
		<button role="menuitem" onclick={() => menuAction(() => duplicateTab(menu.tabId))}>
			Duplicar tab
		</button>
		<button role="menuitem" onclick={() => menuAction(() => openTab(menu.tabPath))}>
			Abrir nova tab aqui
		</button>
		<div class="ctx-sep"></div>
		<button role="menuitem" onclick={() => copyPath(menu.tabPath)}>Copiar caminho</button>
	</div>
{/if}

<style>
	.tabbar {
		--tab-height: 36px;
		--tab-bg-active: #0e0e1c;
		--tab-bg-hover: #0c0c18;
		--tab-accent: #a78bfa;
		--tab-text-active: #dddde8;
		--tab-text-inactive: #555;
		--tab-border: #12121e;

		display: flex;
		align-items: stretch;
		height: var(--tab-height);
		background: #0a0a14;
		border-bottom: 1px solid var(--tab-border);
		font-family: 'DM Sans', system-ui, sans-serif;
		flex-shrink: 0;
	}

	.scroll {
		display: flex;
		align-items: stretch;
		flex: 1;
		overflow-x: auto;
		overflow-y: hidden;
		scrollbar-width: none; /* Firefox */
	}
	.scroll::-webkit-scrollbar {
		display: none; /* Chrome/Edge/WebView2 */
	}

	.tab {
		position: relative;
		display: flex;
		align-items: center;
		gap: 6px;
		min-width: 120px;
		max-width: 200px;
		flex: 1 1 0;
		padding: 0 10px;
		background: transparent;
		border-top: 2px solid transparent;
		color: var(--tab-text-inactive);
		font-size: 13px;
		cursor: pointer;
		user-select: none;
		transition:
			color 150ms ease,
			background 150ms ease;
	}
	.tab:hover {
		background: var(--tab-bg-hover);
		color: #888;
	}
	.tab.active {
		background: var(--tab-bg-active);
		border-top: 2px solid var(--tab-accent);
		color: var(--tab-text-active);
	}
	.tab.dragging {
		opacity: 0.4;
	}
	/* Indicador de drop: linha vertical 2px à esquerda da tab-alvo. */
	.tab.dropbefore::before {
		content: '';
		position: absolute;
		left: -1px;
		top: 0;
		bottom: 0;
		width: 2px;
		background: var(--tab-accent);
	}

	.tab-ico {
		font-size: 12px;
		flex-shrink: 0;
	}
	.tab-title {
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.tab-close {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 16px;
		height: 16px;
		flex-shrink: 0;
		border: none;
		border-radius: 4px;
		background: none;
		color: inherit;
		font-size: 14px;
		line-height: 1;
		cursor: pointer;
		opacity: 0;
		transition:
			background 150ms ease,
			color 150ms ease,
			opacity 150ms ease;
	}
	.tab.active .tab-close,
	.tab:hover .tab-close {
		opacity: 1;
	}
	.tab-close:hover {
		background: #f8717130;
		color: #f87171;
	}

	.new-tab {
		flex-shrink: 0;
		width: 32px;
		border: none;
		border-left: 1px solid var(--tab-border);
		background: #0a0a14;
		color: #6a6a82;
		font-size: 18px;
		cursor: pointer;
		transition:
			background 150ms ease,
			color 150ms ease;
	}
	.new-tab:hover {
		background: var(--tab-bg-hover);
		color: var(--tab-accent);
	}

	/* Menu de contexto */
	.ctx-menu {
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
	.ctx-menu button {
		display: block;
		width: 100%;
		padding: 7px 10px;
		border: none;
		border-radius: 5px;
		background: none;
		color: #c8c8dc;
		font-family: inherit;
		font-size: 13px;
		text-align: left;
		cursor: pointer;
		transition: background 120ms ease;
	}
	.ctx-menu button:hover {
		background: #1e1e35;
		color: #fff;
	}
	.ctx-sep {
		height: 1px;
		margin: 4px 6px;
		background: #1e1e35;
	}
</style>
