<script lang="ts">
	import { tick } from 'svelte';
	import type { FileEntry } from '$lib/tauri';
	import {
		columns,
		initColumns,
		selectInColumn,
		setColumnWidth
	} from '$lib/stores/columnsView';
	import ColumnItem from './ColumnItem.svelte';
	import ColumnsPreview from './ColumnsPreview.svelte';

	let { rootPath, onNavigate }: { rootPath: string; onNavigate: (path: string) => void } = $props();

	let containerEl = $state<HTMLDivElement>();
	let focusedCol = $state(0);

	// Marca navegação iniciada internamente (não deve reinicializar colunas).
	let selfNav = false;
	let lastRoot = '';

	// Reinicializa as colunas só quando o rootPath muda por fonte externa
	// (breadcrumb, ← →). Navegação interna nas colunas é ignorada.
	$effect(() => {
		const rp = rootPath;
		if (!rp) return;
		if (selfNav) {
			selfNav = false;
			lastRoot = rp;
			return;
		}
		if (rp === lastRoot) return;
		lastRoot = rp;
		focusedCol = 0;
		initColumns(rp);
	});

	// Auto-scroll p/ a direita quando o número de colunas muda.
	$effect(() => {
		void $columns.length;
		if (!containerEl) return;
		tick().then(() => {
			containerEl?.scrollTo({ left: containerEl.scrollWidth, behavior: 'smooth' });
		});
	});

	// Método público: reinicializa as colunas no rootPath atual.
	export function reset() {
		focusedCol = 0;
		lastRoot = rootPath;
		if (rootPath) initColumns(rootPath);
	}

	async function handleSelect(idx: number, entry: FileEntry) {
		await selectInColumn(idx, entry);
		if (entry.is_dir) {
			focusedCol = idx + 1;
			selfNav = true;
			onNavigate(entry.path); // sincroniza path/histórico da tab
		} else {
			focusedCol = idx;
		}
	}

	// ── Redimensionamento ─────────────────────────────────────────
	let dragIdx = $state<number | null>(null);
	let startX = 0;
	let startW = 0;

	function startResize(e: MouseEvent, idx: number) {
		e.preventDefault();
		dragIdx = idx;
		startX = e.clientX;
		startW = $columns[idx]?.width ?? 220;
		window.addEventListener('mousemove', onResizeMove);
		window.addEventListener('mouseup', endResize);
	}
	function onResizeMove(e: MouseEvent) {
		if (dragIdx === null) return;
		setColumnWidth(dragIdx, startW + (e.clientX - startX));
	}
	function endResize() {
		dragIdx = null;
		window.removeEventListener('mousemove', onResizeMove);
		window.removeEventListener('mouseup', endResize);
	}

	// ── Teclado ───────────────────────────────────────────────────
	function moveSelection(delta: number) {
		const col = $columns[focusedCol];
		if (!col || col.previewEntry || !col.entries.length) return;
		const cur = col.entries.findIndex((e) => e.path === col.selectedPath);
		const ni =
			cur < 0
				? delta > 0
					? 0
					: col.entries.length - 1
				: Math.min(col.entries.length - 1, Math.max(0, cur + delta));
		handleSelect(focusedCol, col.entries[ni]);
	}

	function onKeydown(e: KeyboardEvent) {
		const cs = $columns;
		if (e.key === 'ArrowDown') {
			e.preventDefault();
			moveSelection(1);
		} else if (e.key === 'ArrowUp') {
			e.preventDefault();
			moveSelection(-1);
		} else if (e.key === 'ArrowLeft') {
			e.preventDefault();
			focusedCol = Math.max(0, focusedCol - 1);
		} else if (e.key === 'ArrowRight') {
			e.preventDefault();
			const col = cs[focusedCol];
			const sel = col?.entries.find((x) => x.path === col.selectedPath);
			if (sel?.is_dir) focusedCol = Math.min(cs.length - 1, focusedCol + 1);
		} else if (e.key === 'Enter') {
			e.preventDefault();
			const col = cs[focusedCol];
			const sel = col?.entries.find((x) => x.path === col.selectedPath);
			if (sel) handleSelect(focusedCol, sel);
		} else if (e.key === 'Escape') {
			e.preventDefault();
			focusedCol = Math.max(0, focusedCol - 1);
		}
	}

	function headerLabel(path: string): string {
		const sep = path.includes('\\') ? '\\' : '/';
		const parts = path.split(sep).filter((p) => p.length > 0);
		return parts.length ? parts[parts.length - 1] : path;
	}
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<div
	class="columns"
	class:resizing={dragIdx !== null}
	role="tree"
	tabindex="0"
	bind:this={containerEl}
	onkeydown={onKeydown}
>
	{#each $columns as col, idx (col.id)}
		{#if col.previewEntry}
			<div class="column preview-col">
				<div class="col-head">{col.previewEntry.name}</div>
				<div class="col-body preview-body">
					<ColumnsPreview entry={col.previewEntry} />
				</div>
			</div>
		{:else}
			<div class="column" style="width: {col.width}px" class:focused={idx === focusedCol}>
				<div class="col-head">{headerLabel(col.path)}</div>
				<div class="col-body">
					{#if col.isLoading}
						<div class="loading">Carregando…</div>
					{:else}
						{#each col.entries as entry (entry.path)}
							<ColumnItem
								{entry}
								isSelected={col.selectedPath === entry.path}
								onSelect={() => handleSelect(idx, entry)}
							/>
						{/each}
					{/if}
				</div>
				<!-- divisor de resize -->
				<button
					type="button"
					class="resizer"
					aria-label="Redimensionar coluna"
					onmousedown={(e) => startResize(e, idx)}
				></button>
			</div>
		{/if}
	{/each}
</div>

<style>
	.columns {
		display: flex;
		flex-direction: row;
		height: 100%;
		overflow-x: auto;
		overflow-y: hidden;
		scroll-behavior: smooth;
		outline: none;
		background: #080810;
	}
	.columns.resizing {
		cursor: col-resize;
		user-select: none;
		scroll-behavior: auto;
	}

	.column {
		position: relative;
		display: flex;
		flex-direction: column;
		flex-shrink: 0;
		height: 100%;
		border-right: 1px solid #12121e;
	}
	.column:last-child {
		border-right: none;
	}
	.preview-col {
		min-width: 200px;
		max-width: 320px;
		flex: 1;
	}

	.col-head {
		display: flex;
		align-items: center;
		height: 28px;
		padding: 0 10px;
		border-bottom: 1px solid #12121e;
		color: #444;
		font-size: 11px;
		text-transform: uppercase;
		letter-spacing: 0.04em;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		flex-shrink: 0;
	}
	.column.focused .col-head {
		color: #6a6a82;
	}
	.col-body {
		flex: 1;
		overflow-y: auto;
		padding: 4px 0;
	}
	.preview-body {
		padding: 16px;
	}
	.loading {
		padding: 10px;
		color: #444;
		font-size: 12px;
	}

	.resizer {
		position: absolute;
		right: -2px;
		top: 0;
		bottom: 0;
		width: 4px;
		padding: 0;
		border: none;
		background: transparent;
		cursor: col-resize;
		z-index: 2;
	}
	.resizer:hover {
		background: #a78bfa;
	}
</style>
