<script lang="ts">
	import { onMount } from 'svelte';
	import type { FileEntry } from '$lib/tauri';
	import { formatSize, formatDate, getFileIcon, getFileType } from '$lib/utils/fileUtils';

	type SortBy = 'name' | 'size' | 'modified' | 'type';
	type SelMode = 'single' | 'toggle' | 'range';

	let {
		entries,
		sortBy,
		sortDir,
		selectedPaths,
		onNavigate,
		onSort,
		onSelect,
		// Extras necessários além da lista mínima de props:
		columnWidths = {},
		onColumnResize = () => {},
		onContextMenu = () => {},
		onClearSelection = () => {}
	}: {
		entries: FileEntry[];
		sortBy: SortBy;
		sortDir: 'asc' | 'desc';
		selectedPaths: Set<string>;
		onNavigate: (path: string) => void;
		onSort: (col: SortBy) => void;
		onSelect: (path: string, mode: SelMode) => void;
		columnWidths?: Record<string, number>;
		onColumnResize?: (col: string, width: number) => void;
		onContextMenu?: (e: { entry: FileEntry; x: number; y: number }) => void;
		onClearSelection?: () => void;
	} = $props();

	const ROW_H = 30;
	const BUFFER = 10;
	const VIRTUAL_THRESHOLD = 200;
	const DEFAULTS: Record<string, number> = { size: 90, type: 110, modified: 140 };

	// ── Redimensionamento de colunas ──────────────────────────────
	let dragCol = $state<string | null>(null);
	let dragWidth = $state(0);
	let resizeStartX = 0;
	let resizeStartW = 0;

	function colW(col: string): number {
		if (dragCol === col) return dragWidth;
		return columnWidths[col] ?? DEFAULTS[col];
	}

	let gridCols = $derived(
		`36px minmax(160px, 1fr) ${colW('size')}px ${colW('type')}px ${colW('modified')}px`
	);

	function startResize(e: MouseEvent, col: string) {
		e.preventDefault();
		e.stopPropagation();
		dragCol = col;
		resizeStartX = e.clientX;
		resizeStartW = colW(col);
		dragWidth = resizeStartW;
		window.addEventListener('mousemove', onResizeMove);
		window.addEventListener('mouseup', endResize);
	}
	function onResizeMove(e: MouseEvent) {
		dragWidth = Math.max(60, resizeStartW + (e.clientX - resizeStartX));
	}
	function endResize() {
		if (dragCol) onColumnResize(dragCol, dragWidth);
		dragCol = null;
		window.removeEventListener('mousemove', onResizeMove);
		window.removeEventListener('mouseup', endResize);
	}

	// ── Virtual scroll ────────────────────────────────────────────
	let bodyEl = $state<HTMLDivElement>();
	let scrollTop = $state(0);
	let viewportH = $state(0);

	let total = $derived(entries.length);
	let virtual = $derived(total > VIRTUAL_THRESHOLD);

	let range = $derived.by(() => {
		if (!virtual) return { start: 0, end: total };
		const start = Math.max(0, Math.floor(scrollTop / ROW_H) - BUFFER);
		const end = Math.min(total, Math.ceil((scrollTop + viewportH) / ROW_H) + BUFFER);
		return { start, end };
	});

	let visible = $derived(entries.slice(range.start, range.end));
	let padTop = $derived(range.start * ROW_H);
	let padBottom = $derived((total - range.end) * ROW_H);

	function onScroll() {
		if (bodyEl) scrollTop = bodyEl.scrollTop;
	}

	onMount(() => {
		if (!bodyEl) return;
		viewportH = bodyEl.clientHeight;
		const ro = new ResizeObserver(() => {
			if (bodyEl) viewportH = bodyEl.clientHeight;
		});
		ro.observe(bodyEl);
		return () => ro.disconnect();
	});

	// ── Interações de linha ───────────────────────────────────────
	function rowClick(e: MouseEvent, entry: FileEntry) {
		const mode: SelMode = e.shiftKey ? 'range' : e.ctrlKey ? 'toggle' : 'single';
		onSelect(entry.path, mode);
	}
	function rowDblClick(entry: FileEntry) {
		if (entry.is_dir) onNavigate(entry.path);
		else onSelect(entry.path, 'single');
	}
	function rowContext(e: MouseEvent, entry: FileEntry) {
		e.preventDefault();
		onContextMenu({ entry, x: e.clientX, y: e.clientY });
	}
	function onRowDragStart(e: DragEvent, entry: FileEntry) {
		if (!entry.is_dir || !e.dataTransfer) return;
		e.dataTransfer.setData('bookmark-drop-path', entry.path);
		e.dataTransfer.setData('explorer-path', entry.path);
		e.dataTransfer.effectAllowed = 'copy';
	}
	function bodyClick(e: MouseEvent) {
		const target = e.target as HTMLElement;
		if (!target.closest('.row')) onClearSelection();
	}

	function arrow(col: SortBy): string {
		if (sortBy !== col) return '';
		return sortDir === 'asc' ? ' ↑' : ' ↓';
	}
</script>

<div class="listview" class:resizing={dragCol !== null}>
	<!-- CABEÇALHO -->
	<div class="head" style="grid-template-columns: {gridCols}">
		<div class="hcell ico"></div>
		<button class="hcell name" onclick={() => onSort('name')}>Nome{arrow('name')}</button>
		<div class="hcol">
			<button type="button" class="resizer" aria-label="Redimensionar coluna Tamanho"
				onmousedown={(e) => startResize(e, 'size')}></button>
			<button class="hcell size" onclick={() => onSort('size')}>Tamanho{arrow('size')}</button>
		</div>
		<div class="hcol">
			<button type="button" class="resizer" aria-label="Redimensionar coluna Tipo"
				onmousedown={(e) => startResize(e, 'type')}></button>
			<button class="hcell type" onclick={() => onSort('type')}>Tipo{arrow('type')}</button>
		</div>
		<div class="hcol">
			<button type="button" class="resizer" aria-label="Redimensionar coluna Modificado"
				onmousedown={(e) => startResize(e, 'modified')}></button>
			<button class="hcell mod" onclick={() => onSort('modified')}>Modificado{arrow('modified')}</button>
		</div>
	</div>

	<!-- CORPO -->
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<div class="body" role="grid" tabindex="-1" bind:this={bodyEl} onscroll={onScroll} onclick={bodyClick}>
		<div class="spacer" style="height: {padTop}px"></div>
		{#each visible as entry, i (entry.path)}
			<div
				class="row"
				class:even={(range.start + i) % 2 === 1}
				class:selected={selectedPaths.has(entry.path)}
				class:hiddenrow={entry.is_hidden}
				style="grid-template-columns: {gridCols}; height: {ROW_H}px"
				role="row"
				tabindex="0"
				draggable={entry.is_dir}
				ondragstart={(e) => onRowDragStart(e, entry)}
				onclick={(e) => rowClick(e, entry)}
				ondblclick={() => rowDblClick(entry)}
				oncontextmenu={(e) => rowContext(e, entry)}
				onkeydown={(e) => e.key === 'Enter' && rowDblClick(entry)}
			>
				<div class="cell ico">{getFileIcon(entry)}</div>
				<div class="cell name" title={entry.name}>{entry.name}</div>
				<div class="cell size">{entry.is_dir ? '-' : formatSize(entry.size)}</div>
				<div class="cell type">{getFileType(entry)}</div>
				<div class="cell mod">{formatDate(entry.modified)}</div>
			</div>
		{/each}
		<div class="spacer" style="height: {padBottom}px"></div>
	</div>
</div>

<style>
	.listview {
		display: flex;
		flex-direction: column;
		height: 100%;
		background: transparent;
		font-family: 'DM Sans', system-ui, sans-serif;
	}
	.listview.resizing {
		cursor: col-resize;
		user-select: none;
	}

	/* Cabeçalho */
	.head {
		display: grid;
		align-items: stretch;
		background: #0c0c18;
		border-bottom: 1px solid #12121e;
		flex-shrink: 0;
	}
	.hcol {
		position: relative;
		display: flex;
	}
	.hcell {
		display: flex;
		align-items: center;
		width: 100%;
		padding: 0 10px;
		height: 28px;
		border: none;
		background: none;
		color: #444;
		font-family: inherit;
		font-size: 11px;
		text-transform: uppercase;
		letter-spacing: 0.06em;
		cursor: pointer;
		transition: background 120ms ease;
	}
	button.hcell:hover {
		background: #111120;
		color: #6a6a82;
	}
	.hcell.ico {
		cursor: default;
	}
	.hcell.size {
		justify-content: flex-end;
	}
	.resizer {
		position: absolute;
		left: -2px;
		top: 0;
		bottom: 0;
		width: 4px;
		padding: 0;
		border: none;
		cursor: col-resize;
		background: #12121e;
		z-index: 2;
	}
	.resizer:hover {
		background: #a78bfa;
	}

	/* Corpo */
	.body {
		flex: 1;
		overflow-y: auto;
		overflow-x: hidden;
	}
	.spacer {
		width: 100%;
	}
	.row {
		display: grid;
		align-items: center;
		border-left: 2px solid transparent;
		color: #c8c8dc;
		font-size: 13px;
		cursor: default;
		transition: background 80ms ease;
	}
	.row.even {
		background: #0a0a12;
	}
	.row:hover {
		background: #111120;
	}
	.row.selected {
		background: #1a1a2e;
		border-left: 2px solid #a78bfa;
	}
	.row.hiddenrow {
		opacity: 0.5;
	}
	.cell {
		padding: 0 10px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.cell.ico {
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 0;
		font-size: 18px;
	}
	.cell.size {
		text-align: right;
		font-family: 'DM Mono', ui-monospace, monospace;
		font-size: 12px;
		color: #555;
	}
	.cell.type,
	.cell.mod {
		color: #555;
		font-size: 12px;
	}
</style>
