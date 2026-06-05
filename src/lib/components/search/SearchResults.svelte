<script lang="ts">
	import { onMount } from 'svelte';
	import { get } from 'svelte/store';
	import Highlight from '$lib/components/ui/Highlight.svelte';
	import { extractQueryTerms } from '$lib/utils/highlightUtils';
	import { getFileIcon } from '$lib/utils/fileUtils';
	import { activeTabId, navigateTab } from '$lib/stores/tabs';
	import {
		searchResults,
		searchQuery,
		searchScope,
		searchLoading,
		searchError,
		searchOptions,
		executeSearch,
		exitSearchMode
	} from '$lib/stores/search';
	import type { SearchResult } from '$lib/tauri';

	let viewMode = $state<'list' | 'grid'>('list');
	let typeFilter = $state<'all' | 'dirs' | 'files'>('all');
	let extFilter = $state('');
	let selectedPath = $state<string | null>(null);
	let menu = $state<{ visible: boolean; x: number; y: number; res: SearchResult | null }>({
		visible: false, x: 0, y: 0, res: null
	});

	// Filtra resultados em memória (sem relançar a busca).
	let displayed = $derived.by(() => {
		let r = $searchResults;
		if (typeFilter === 'dirs') r = r.filter((x) => x.is_dir);
		else if (typeFilter === 'files') r = r.filter((x) => !x.is_dir);
		const ext = extFilter.trim().toLowerCase().replace(/^\./, '');
		if (ext) r = r.filter((x) => (x.extension ?? '').toLowerCase() === ext);
		return r;
	});

	function scoreColor(s: number): string {
		if (s > 80) return '#34d399';
		if (s > 50) return '#a78bfa';
		return '#555';
	}

	function setType(t: 'all' | 'dirs' | 'files') {
		typeFilter = t;
		searchOptions.update((o) => ({
			...o,
			include_dirs: t !== 'files',
			include_files: t !== 'dirs'
		}));
	}

	function openResult(res: SearchResult) {
		const id = get(activeTabId);
		if (!id) return;
		navigateTab(id, res.is_dir ? res.path : res.parent_path);
		exitSearchMode();
	}

	function openMenu(e: MouseEvent, res: SearchResult) {
		e.preventDefault();
		menu = { visible: true, x: e.clientX, y: e.clientY, res };
	}
	function closeMenu() {
		menu = { ...menu, visible: false };
	}
	function menuNavigate() {
		if (menu.res) {
			const id = get(activeTabId);
			if (id) navigateTab(id, menu.res.is_dir ? menu.res.path : menu.res.parent_path);
			exitSearchMode();
		}
		closeMenu();
	}
	function menuCopy() {
		if (menu.res) navigator.clipboard?.writeText(menu.res.path).catch(() => {});
		closeMenu();
	}
	function menuNewTab() {
		// openTab importado sob demanda evita ciclo; usa store diretamente.
		if (menu.res) {
			import('$lib/stores/tabs').then(({ openTab }) => {
				openTab(menu.res!.is_dir ? menu.res!.path : menu.res!.parent_path);
			});
		}
		closeMenu();
	}

	function searchRecursive() {
		searchScope.set('recursive');
		executeSearch();
	}

	onMount(() => {
		document.addEventListener('click', closeMenu);
		return () => document.removeEventListener('click', closeMenu);
	});
</script>

<div class="results">
	<!-- HEADER -->
	<div class="head">
		<span class="hicon">🔍</span>
		<span class="q">"{$searchQuery}"</span>
		<span class="dot">·</span>
		{#if $searchLoading}
			<span class="muted">buscando…</span>
		{:else if $searchError}
			<span class="err">⚠️ {$searchError}</span>
		{:else}
			<span class="muted">{$searchResults.length} resultados</span>
		{/if}
		<span class="dot">·</span>
		<span class="muted">{$searchScope === 'current' ? 'Pasta atual' : 'Recursivo'}</span>
		<button class="close" onclick={exitSearchMode} aria-label="Fechar busca">×</button>
	</div>

	<!-- TOOLBAR -->
	<div class="toolbar">
		<div class="views">
			<button class:active={viewMode === 'list'} onclick={() => (viewMode = 'list')} title="Lista">≡</button>
			<button class:active={viewMode === 'grid'} onclick={() => (viewMode = 'grid')} title="Grade">⊞</button>
		</div>
		<span class="vdiv"></span>
		<div class="types">
			<button class:sel={typeFilter === 'all'} onclick={() => setType('all')}>Todos</button>
			<button class:sel={typeFilter === 'dirs'} onclick={() => setType('dirs')}>📁 Pastas</button>
			<button class:sel={typeFilter === 'files'} onclick={() => setType('files')}>📄 Arquivos</button>
		</div>
		<input class="ext" placeholder="ext…" bind:value={extFilter} spellcheck="false" />
		<span class="count">{displayed.length} resultados</span>
	</div>

	<!-- CONTEÚDO -->
	<div class="body">
		{#if $searchLoading}
			<div class="skeleton">
				{#each Array(6) as _, i (i)}<div class="sk"></div>{/each}
			</div>
		{:else if $searchError}
			<div class="center err">
				<span class="big">⚠️</span>
				<span>{$searchError}</span>
				<button class="action" onclick={() => executeSearch()}>Tentar novamente</button>
			</div>
		{:else if displayed.length === 0}
			<div class="center muted">
				<span class="big">🔍</span>
				<span>Nenhum resultado para "{$searchQuery}"</span>
				<span class="hint">Tente termos mais curtos ou mude o escopo da busca</span>
				{#if $searchScope === 'current'}
					<button class="action" onclick={searchRecursive}>Buscar recursivamente</button>
				{/if}
			</div>
		{:else if viewMode === 'list'}
			<div class="list">
				{#each displayed as res (res.path)}
					<div
						class="row"
						class:selected={selectedPath === res.path}
						role="button"
						tabindex="0"
						onclick={() => (selectedPath = res.path)}
						ondblclick={() => openResult(res)}
						oncontextmenu={(e) => openMenu(e, res)}
						onkeydown={(e) => e.key === 'Enter' && openResult(res)}
					>
						<div class="r1">
							<span class="ico">{getFileIcon(res)}</span>
							<span class="name">
								<Highlight text={res.name} indices={res.match_indices} source="fuzzy" />
							</span>
							<span class="badge" style="color: {scoreColor(res.score)}; border-color: {scoreColor(res.score)}33">
								{res.score}
							</span>
						</div>
						<div class="r2">
							<Highlight
								text={res.parent_path}
								terms={extractQueryTerms($searchQuery)}
								source="filter"
								highlightClass="hl-filter"
								maxChars={50}
								padding={10}
							/>
						</div>
					</div>
				{/each}
			</div>
		{:else}
			<div class="grid">
				{#each displayed as res (res.path)}
					<button class="card" ondblclick={() => openResult(res)} oncontextmenu={(e) => openMenu(e, res)} title={res.path}>
						<span class="card-ico">{getFileIcon(res)}</span>
						<span class="card-name">
							<Highlight text={res.name} indices={res.match_indices} source="fuzzy" maxChars={24} />
						</span>
						<span class="card-parent" dir="rtl">{res.parent_path}</span>
					</button>
				{/each}
			</div>
		{/if}
	</div>
</div>

{#if menu.visible && menu.res}
	<div class="ctx" style="left: {menu.x}px; top: {menu.y}px" role="menu" tabindex="-1">
		<button role="menuitem" onclick={menuNavigate}>Navegar para esta pasta</button>
		<button role="menuitem" onclick={menuCopy}>Copiar caminho completo</button>
		<button role="menuitem" onclick={menuNewTab}>Abrir em nova tab</button>
	</div>
{/if}

<style>
	.results {
		display: flex;
		flex-direction: column;
		height: 100%;
		background: #080810;
		color: #c8c8dc;
		font-family: 'DM Sans', system-ui, sans-serif;
		font-size: 13px;
		overflow: hidden;
	}
	.head {
		display: flex;
		align-items: center;
		gap: 8px;
		height: 40px;
		padding: 0 12px;
		border-bottom: 1px solid #12121e;
		flex-shrink: 0;
	}
	.hicon {
		font-size: 14px;
	}
	.q {
		color: #dddde8;
		font-weight: 500;
	}
	.dot {
		color: #2a2a42;
	}
	.muted {
		color: #555;
		font-size: 12px;
	}
	.err {
		color: #f87171;
		font-size: 12px;
	}
	.close {
		margin-left: auto;
		width: 24px;
		height: 24px;
		border: none;
		border-radius: 5px;
		background: none;
		color: #666;
		font-size: 16px;
		cursor: pointer;
	}
	.close:hover {
		background: #1a1a2e;
		color: #c8c8dc;
	}

	.toolbar {
		display: flex;
		align-items: center;
		gap: 8px;
		height: 34px;
		padding: 0 12px;
		border-bottom: 1px solid #12121e;
		background: #0c0c18;
		flex-shrink: 0;
	}
	.views {
		display: flex;
		border-radius: 5px;
		overflow: hidden;
	}
	.views button {
		width: 26px;
		height: 24px;
		border: 1px solid #1e1e35;
		background: #111120;
		color: #444;
		cursor: pointer;
		font-size: 13px;
	}
	.views button.active {
		background: #1e1e35;
		color: #a78bfa;
		border-color: #a78bfa44;
	}
	.vdiv {
		width: 1px;
		height: 14px;
		background: #1e1e35;
	}
	.types {
		display: flex;
		gap: 4px;
	}
	.types button {
		padding: 3px 8px;
		border: 1px solid #1e1e35;
		border-radius: 10px;
		background: #111120;
		color: #666;
		font-family: inherit;
		font-size: 11px;
		cursor: pointer;
	}
	.types button.sel {
		background: #1e1e35;
		color: #a78bfa;
		border-color: #a78bfa44;
	}
	.ext {
		width: 70px;
		height: 24px;
		padding: 0 8px;
		background: #111120;
		border: 1px solid #1e1e35;
		border-radius: 5px;
		color: #c8c8dc;
		font-family: inherit;
		font-size: 11px;
		outline: none;
	}
	.count {
		margin-left: auto;
		color: #444;
		font-size: 11px;
	}

	.body {
		flex: 1;
		overflow-y: auto;
	}

	/* Lista */
	.list {
		display: flex;
		flex-direction: column;
	}
	.row {
		display: flex;
		flex-direction: column;
		justify-content: center;
		gap: 2px;
		height: 52px;
		padding: 0 12px;
		border-bottom: 1px solid #0e0e1a;
		cursor: default;
	}
	.row:hover {
		background: #111120;
	}
	.row.selected {
		background: #16162a;
	}
	.r1 {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.ico {
		font-size: 14px;
		flex-shrink: 0;
	}
	.name {
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.name :global(.hl) {
		color: #a78bfa;
		font-weight: 600;
	}
	.badge {
		flex-shrink: 0;
		padding: 1px 7px;
		border: 1px solid;
		border-radius: 9px;
		font-family: 'DM Mono', ui-monospace, monospace;
		font-size: 11px;
	}
	.r2 {
		color: #444;
		font-size: 11px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		padding-left: 22px;
	}

	/* Grade */
	.grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(96px, 1fr));
		gap: 8px;
		padding: 12px;
		align-content: start;
	}
	.card {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 4px;
		height: 80px;
		padding: 8px 4px;
		border: 1px solid transparent;
		border-radius: 8px;
		background: none;
		color: #c8c8dc;
		font-family: inherit;
		font-size: 11px;
		cursor: pointer;
		overflow: hidden;
	}
	.card:hover {
		background: #111120;
		border-color: #1e1e35;
	}
	.card-ico {
		font-size: 28px;
	}
	.card-name {
		width: 100%;
		text-align: center;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.card-name :global(.hl) {
		color: #a78bfa;
		font-weight: 600;
	}
	.card-parent {
		max-width: 100%;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		color: #333;
		font-size: 9px;
	}

	/* Estados */
	.center {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 8px;
		height: 100%;
		text-align: center;
	}
	.big {
		font-size: 48px;
	}
	.hint {
		color: #333;
		font-size: 11px;
	}
	.action {
		margin-top: 8px;
		padding: 6px 14px;
		border: 1px solid #1e1e35;
		border-radius: 6px;
		background: #111120;
		color: #a78bfa;
		font-family: inherit;
		font-size: 12px;
		cursor: pointer;
	}
	.action:hover {
		background: #1a1a2e;
	}
	.skeleton {
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding: 12px;
	}
	.sk {
		height: 40px;
		border-radius: 6px;
		background: linear-gradient(90deg, #0e0e1c 25%, #16162a 50%, #0e0e1c 75%);
		background-size: 200% 100%;
		animation: shimmer 1.4s infinite;
	}
	@keyframes shimmer {
		to {
			background-position: -200% 0;
		}
	}

	/* Menu de contexto */
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
	}
	.ctx button {
		padding: 7px 10px;
		border: none;
		border-radius: 5px;
		background: none;
		color: #c8c8dc;
		font-family: 'DM Sans', system-ui, sans-serif;
		font-size: 13px;
		text-align: left;
		cursor: pointer;
	}
	.ctx button:hover {
		background: #1e1e35;
	}
</style>
