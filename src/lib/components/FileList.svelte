<script lang="ts">
	import { tick } from 'svelte';
	import { get } from 'svelte/store';
	import { currentFiles, filteredFiles, isLoading, explorerError } from '$lib/stores/explorer';
	import {
		activeTab,
		activeTabId,
		navigateTab,
		refreshTab,
		selectPath,
		clearSelection,
		updateTabSort,
		updateTabViewMode,
		updateTabColumnWidth,
		updateTabFilter,
		resetTabFilter,
		type Tab
	} from '$lib/stores/tabs';
	import { sortEntries } from '$lib/utils/fileUtils';
	import { isFilterActive, countActiveFilters } from '$lib/utils/filterUtils';
	import ViewModeToggle from './ViewModeToggle.svelte';
	import VisibilityToggles from './VisibilityToggles.svelte';
	import ListView from './views/ListView.svelte';
	import GridView from './views/GridView.svelte';
	import ColumnsView from './views/ColumnsView.svelte';
	import ActiveFilters from './filters/ActiveFilters.svelte';
	import FilterPanel from './filters/FilterPanel.svelte';

	type IconSize = 'small' | 'medium' | 'large';

	// iconSize é estado de UI da grade (não por-tab).
	let iconSize = $state<IconSize>('medium');
	let columnsRef = $state<ReturnType<typeof ColumnsView>>();
	let isFilterOpen = $state(false);

	let viewMode = $derived($activeTab?.viewMode ?? 'list');
	let sortBy = $derived<Tab['sortBy']>($activeTab?.sortBy ?? 'name');
	let sortDir = $derived<Tab['sortDir']>($activeTab?.sortDir ?? 'asc');
	let selectedPaths = $derived($activeTab?.selectedPaths ?? new Set<string>());
	let columnWidths = $derived($activeTab?.columnWidths ?? {});

	// Views usam os arquivos já filtrados.
	let sortedEntries = $derived(sortEntries($filteredFiles, sortBy, sortDir));

	let dirs = $derived($filteredFiles.filter((f) => f.is_dir).length);
	let files = $derived($filteredFiles.length - dirs);

	let totalCount = $derived($currentFiles.length); // sem filtro
	let filterActive = $derived($activeTab ? isFilterActive($activeTab.filter) : false);
	let filterCount = $derived($activeTab ? countActiveFilters($activeTab.filter) : 0);

	// ── Handlers ──────────────────────────────────────────────────
	function navigate(path: string) {
		const id = get(activeTabId);
		if (id) navigateTab(id, path);
	}

	function handleSort(col: Tab['sortBy']) {
		const id = get(activeTabId);
		if (!id) return;
		const dir = col === sortBy ? (sortDir === 'asc' ? 'desc' : 'asc') : 'asc';
		updateTabSort(id, col, dir);
	}

	function handleSelect(path: string, mode: 'single' | 'toggle' | 'range') {
		const id = get(activeTabId);
		if (id) selectPath(id, path, mode);
	}

	function handleClear() {
		const id = get(activeTabId);
		if (id) clearSelection(id);
	}

	function handleColumnResize(col: string, width: number) {
		const id = get(activeTabId);
		if (id) updateTabColumnWidth(id, col, width);
	}

	function handleViewChange(mode: Tab['viewMode']) {
		const id = get(activeTabId);
		if (!id) return;
		updateTabViewMode(id, mode);
		if (mode === 'columns') {
			tick().then(() => columnsRef?.reset());
		}
	}

	function handleRemoveFilter(key: string) {
		const id = get(activeTabId);
		const t = get(activeTab);
		if (!id || !t) return;
		if (key.startsWith('extension:')) {
			const ext = key.slice('extension:'.length);
			updateTabFilter(id, { extensions: t.filter.extensions.filter((x) => x !== ext) });
		} else if (key === 'sizeRange') {
			updateTabFilter(id, { sizeRange: null });
		} else if (key === 'dateRange') {
			updateTabFilter(id, { dateRange: null });
		} else if (key === 'onlyDirs') {
			updateTabFilter(id, { onlyDirs: false });
		} else if (key === 'onlyFiles') {
			updateTabFilter(id, { onlyFiles: false });
		} else if (key === 'nameContains') {
			updateTabFilter(id, { nameContains: '' });
		}
	}

	function resetFilters() {
		const id = get(activeTabId);
		if (id) resetTabFilter(id);
	}

	function retry() {
		const id = get(activeTabId);
		if (id) refreshTab(id);
	}
</script>

<div class="filelist">
	<!-- TOOLBAR -->
	<div class="toolbar">
		<div class="left">
			<ViewModeToggle
				{viewMode}
				onChange={handleViewChange}
				{iconSize}
				onIconSizeChange={(s) => (iconSize = s)}
			/>
			<span class="vdiv"></span>
			<VisibilityToggles />
			<span class="vdiv"></span>
			<button
				class="filter-btn"
				class:open={isFilterOpen}
				title="Filtros"
				onclick={() => (isFilterOpen = !isFilterOpen)}
			>
				<span class="hex">⬡</span>
				<span>Filtros</span>
				{#if filterCount > 0}<span class="badge">{filterCount}</span>{/if}
			</button>
		</div>
		<div class="counter">
			{dirs} pasta{dirs !== 1 ? 's' : ''}, {files} arquivo{files !== 1 ? 's' : ''}
		</div>
	</div>

	{#if filterActive && $activeTab}
		<ActiveFilters
			filter={$activeTab.filter}
			filteredCount={$filteredFiles.length}
			{totalCount}
			onRemove={handleRemoveFilter}
			onReset={resetFilters}
		/>
	{/if}

	<!-- CONTEÚDO -->
	<div class="content">
		{#if $explorerError}
			<div class="center error">
				<span class="big">⚠️</span>
				<span class="msg">{$explorerError}</span>
				<button class="retry" type="button" onclick={retry}>Tentar novamente</button>
			</div>
		{:else if viewMode !== 'columns' && $filteredFiles.length === 0 && !$isLoading}
			<div class="center muted">
				<span class="big">📂</span>
				<span>{filterActive ? 'Nenhum item corresponde aos filtros' : 'Esta pasta está vazia'}</span>
			</div>
		{:else if viewMode === 'list'}
			<ListView
				entries={sortedEntries}
				{sortBy}
				{sortDir}
				{selectedPaths}
				{columnWidths}
				onNavigate={navigate}
				onSort={handleSort}
				onSelect={handleSelect}
				onColumnResize={handleColumnResize}
				onClearSelection={handleClear}
			/>
		{:else if viewMode === 'grid'}
			<GridView
				entries={sortedEntries}
				{selectedPaths}
				{iconSize}
				onNavigate={navigate}
				onSelect={handleSelect}
				onClearSelection={handleClear}
			/>
		{:else}
			<ColumnsView bind:this={columnsRef} rootPath={$activeTab?.path ?? ''} onNavigate={navigate} />
		{/if}

		<!-- Skeleton sobreposto durante o loading -->
		<div class="skeleton" class:show={$isLoading}>
			{#each Array(12) as _, i (i)}
				<div class="sk-line"></div>
			{/each}
		</div>

		<!-- Painel de filtros deslizante -->
		<FilterPanel isOpen={isFilterOpen} onClose={() => (isFilterOpen = false)} />
	</div>
</div>

<style>
	.filelist {
		display: flex;
		flex-direction: column;
		height: 100%;
		background: #080810;
		color: #c8c8dc;
		font-family: 'DM Sans', system-ui, sans-serif;
		font-size: 13px;
		overflow: hidden;
	}

	/* Toolbar */
	.toolbar {
		display: flex;
		justify-content: space-between;
		align-items: center;
		height: 36px;
		padding: 0 12px;
		background: #0c0c18;
		border-bottom: 1px solid #12121e;
		flex-shrink: 0;
	}
	.left {
		display: flex;
		align-items: center;
		gap: 10px;
	}
	.vdiv {
		width: 1px;
		height: 16px;
		background: #1e1e35;
		margin: 0 8px;
	}
	.filter-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		height: 28px;
		padding: 0 10px;
		border: 1px solid #1e1e35;
		border-radius: 6px;
		background: #111120;
		color: #6a6a82;
		font-family: inherit;
		font-size: 12px;
		cursor: pointer;
		transition:
			background 120ms ease,
			color 120ms ease,
			border-color 120ms ease;
	}
	.filter-btn:hover {
		color: #888;
		border-color: #2a2a42;
	}
	.filter-btn.open {
		background: #1e1e35;
		border-color: #a78bfa44;
		color: #a78bfa;
	}
	.hex {
		font-size: 13px;
	}
	.badge {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 16px;
		height: 16px;
		border-radius: 50%;
		background: #a78bfa;
		color: #fff;
		font-size: 10px;
		font-weight: 700;
	}
	.counter {
		color: #444;
		font-size: 11px;
	}

	/* Conteúdo */
	.content {
		position: relative;
		flex: 1;
		overflow: hidden;
	}

	/* Estados centrais */
	.center {
		position: absolute;
		inset: 0;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 10px;
	}
	.muted {
		color: #444;
	}
	.error {
		color: #e06a6a;
	}
	.big {
		font-size: 64px;
	}
	.msg {
		max-width: 80%;
		text-align: center;
	}
	.retry {
		margin-top: 6px;
		padding: 6px 14px;
		border: 1px solid #1e1e35;
		border-radius: 6px;
		background: #111120;
		color: #9090a8;
		font-family: inherit;
		font-size: 12px;
		cursor: pointer;
	}
	.retry:hover {
		background: #1a1a2e;
		color: #c8c8dc;
	}

	/* Skeleton overlay */
	.skeleton {
		position: absolute;
		inset: 0;
		padding: 12px;
		display: flex;
		flex-direction: column;
		gap: 12px;
		background: #080810;
		pointer-events: none;
		opacity: 0;
		transition: opacity 200ms ease;
	}
	.skeleton.show {
		opacity: 1;
		pointer-events: auto;
	}
	.sk-line {
		height: 2px;
		border-radius: 2px;
		background: linear-gradient(90deg, #111 0%, #1e1e35 50%, #111 100%);
		background-size: 200% 100%;
		animation: shimmer 1.5s infinite;
	}
	@keyframes shimmer {
		to {
			background-position: -200% 0;
		}
	}
</style>
