<script lang="ts">
	import { onMount } from 'svelte';
	import { convertFileSrc } from '@tauri-apps/api/core';
	import type { FileEntry } from '$lib/tauri';
	import { isTauri } from '$lib/tauri';
	import { getFileIcon } from '$lib/utils/fileUtils';

	type SelMode = 'single' | 'toggle' | 'range';
	type IconSize = 'small' | 'medium' | 'large';

	let {
		entries,
		selectedPaths,
		onNavigate,
		onSelect,
		iconSize = 'medium',
		onContextMenu = () => {},
		onClearSelection = () => {}
	}: {
		entries: FileEntry[];
		selectedPaths: Set<string>;
		onNavigate: (path: string) => void;
		onSelect: (path: string, mode: SelMode) => void;
		iconSize?: IconSize;
		onContextMenu?: (e: { entry: FileEntry; x: number; y: number }) => void;
		onClearSelection?: () => void;
	} = $props();

	const GAP = 4;
	const BUFFER_ROWS = 3;
	const VIRTUAL_THRESHOLD = 500;
	const IMG_EXT = new Set(['jpg', 'jpeg', 'png', 'gif', 'webp']);

	const SIZES: Record<IconSize, { card: number; icon: number; font: number }> = {
		small: { card: 72, icon: 28, font: 10 },
		medium: { card: 96, icon: 40, font: 11 },
		large: { card: 128, icon: 56, font: 12 }
	};
	let dim = $derived(SIZES[iconSize]);
	let rowH = $derived(dim.card + GAP);

	// ── Container / medidas ───────────────────────────────────────
	let containerEl = $state<HTMLDivElement>();
	let scrollTop = $state(0);
	let viewportH = $state(0);
	let containerW = $state(0);

	let cols = $derived(Math.max(1, Math.floor((containerW + GAP) / (dim.card + GAP))));
	let total = $derived(entries.length);
	let virtual = $derived(total > VIRTUAL_THRESHOLD);
	let totalRows = $derived(Math.ceil(total / cols));

	let range = $derived.by(() => {
		if (!virtual) return { start: 0, end: total, padTop: 0, padBottom: 0 };
		const startRow = Math.max(0, Math.floor(scrollTop / rowH) - BUFFER_ROWS);
		const endRow = Math.min(totalRows, Math.ceil((scrollTop + viewportH) / rowH) + BUFFER_ROWS);
		return {
			start: startRow * cols,
			end: Math.min(total, endRow * cols),
			padTop: startRow * rowH,
			padBottom: (totalRows - endRow) * rowH
		};
	});

	let visible = $derived(entries.slice(range.start, range.end));

	function onScroll() {
		if (containerEl) scrollTop = containerEl.scrollTop;
	}

	onMount(() => {
		if (!containerEl) return;
		const measure = () => {
			if (containerEl) {
				viewportH = containerEl.clientHeight;
				containerW = containerEl.clientWidth;
			}
		};
		measure();
		const ro = new ResizeObserver(measure);
		ro.observe(containerEl);
		return () => ro.disconnect();
	});

	// ── Thumbnails ────────────────────────────────────────────────
	let failedThumbs = $state<Set<string>>(new Set());

	function thumbSrc(entry: FileEntry): string | null {
		if (!isTauri() || entry.is_dir) return null;
		const ext = (entry.extension ?? '').toLowerCase();
		if (!IMG_EXT.has(ext)) return null;
		if (failedThumbs.has(entry.path)) return null;
		return convertFileSrc(entry.path);
	}
	function onThumbError(path: string) {
		failedThumbs.add(path);
		failedThumbs = new Set(failedThumbs);
	}

	// ── Interações ────────────────────────────────────────────────
	let focusedIndex = $state(-1);

	function cardClick(e: MouseEvent, idx: number, entry: FileEntry) {
		focusedIndex = idx;
		const mode: SelMode = e.shiftKey ? 'range' : e.ctrlKey ? 'toggle' : 'single';
		onSelect(entry.path, mode);
	}
	function cardDblClick(entry: FileEntry) {
		if (entry.is_dir) onNavigate(entry.path);
	}
	function cardContext(e: MouseEvent, entry: FileEntry) {
		e.preventDefault();
		onContextMenu({ entry, x: e.clientX, y: e.clientY });
	}
	function cardDragStart(e: DragEvent, entry: FileEntry) {
		if (!entry.is_dir || !e.dataTransfer) return;
		e.dataTransfer.setData('bookmark-drop-path', entry.path);
		e.dataTransfer.setData('explorer-path', entry.path);
		e.dataTransfer.effectAllowed = 'copy';
	}

	// Navegação por teclado (move foco visual, não a seleção).
	function onKeydown(e: KeyboardEvent) {
		if (!total) return;
		let next = focusedIndex < 0 ? 0 : focusedIndex;
		if (e.key === 'ArrowRight') next = Math.min(total - 1, next + 1);
		else if (e.key === 'ArrowLeft') next = Math.max(0, next - 1);
		else if (e.key === 'ArrowDown') next = Math.min(total - 1, next + cols);
		else if (e.key === 'ArrowUp') next = Math.max(0, next - cols);
		else if (e.key === 'Enter') {
			if (focusedIndex >= 0) {
				const en = entries[focusedIndex];
				if (en.is_dir) onNavigate(en.path);
			}
			return;
		} else if (e.key === ' ') {
			e.preventDefault();
			if (focusedIndex >= 0) onSelect(entries[focusedIndex].path, 'toggle');
			return;
		} else {
			return;
		}
		e.preventDefault();
		focusedIndex = next;
		// Mantém o card focado visível (virtual scroll).
		if (containerEl && virtual) {
			const rowTop = Math.floor(next / cols) * rowH;
			if (rowTop < scrollTop) containerEl.scrollTop = rowTop;
			else if (rowTop + dim.card > scrollTop + viewportH)
				containerEl.scrollTop = rowTop + dim.card - viewportH;
		}
	}

	// ── Rubber band (seleção por arrasto) ─────────────────────────
	let rubber = $state<{ active: boolean; x0: number; y0: number; x1: number; y1: number }>({
		active: false,
		x0: 0,
		y0: 0,
		x1: 0,
		y1: 0
	});

	let rubberRect = $derived({
		left: Math.min(rubber.x0, rubber.x1),
		top: Math.min(rubber.y0, rubber.y1),
		width: Math.abs(rubber.x1 - rubber.x0),
		height: Math.abs(rubber.y1 - rubber.y0)
	});

	function onContainerMouseDown(e: MouseEvent) {
		if (e.button !== 0) return;
		const t = e.target as HTMLElement;
		if (t.closest('.card')) return; // card cuida do próprio clique
		rubber = { active: true, x0: e.clientX, y0: e.clientY, x1: e.clientX, y1: e.clientY };
		window.addEventListener('mousemove', onRubberMove);
		window.addEventListener('mouseup', onRubberUp);
		window.addEventListener('keydown', onRubberKey);
	}
	function onRubberMove(e: MouseEvent) {
		rubber = { ...rubber, x1: e.clientX, y1: e.clientY };
	}
	function cleanupRubber() {
		window.removeEventListener('mousemove', onRubberMove);
		window.removeEventListener('mouseup', onRubberUp);
		window.removeEventListener('keydown', onRubberKey);
		rubber = { ...rubber, active: false };
	}
	function onRubberKey(e: KeyboardEvent) {
		if (e.key === 'Escape') cleanupRubber(); // cancela sem selecionar
	}
	function onRubberUp() {
		const r = rubberRect;
		// Clique simples (sem arrasto) → apenas limpa.
		if (r.width < 4 && r.height < 4) {
			onClearSelection();
			cleanupRubber();
			return;
		}
		// Intersecta cards renderizados com o retângulo.
		onClearSelection();
		const cards = containerEl?.querySelectorAll<HTMLElement>('.card[data-path]') ?? [];
		cards.forEach((el) => {
			const b = el.getBoundingClientRect();
			const hit =
				b.left < r.left + r.width &&
				b.right > r.left &&
				b.top < r.top + r.height &&
				b.bottom > r.top;
			if (hit) {
				const p = el.dataset.path;
				if (p) onSelect(p, 'toggle');
			}
		});
		cleanupRubber();
	}
</script>

{#snippet card(entry: FileEntry, idx: number)}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<div
		class="card"
		class:selected={selectedPaths.has(entry.path)}
		class:focused={idx === focusedIndex}
		class:hiddenrow={entry.is_hidden}
		data-path={entry.path}
		style="height: {dim.card}px"
		role="gridcell"
		tabindex="-1"
		title={entry.name}
		draggable={entry.is_dir}
		ondragstart={(e) => cardDragStart(e, entry)}
		onclick={(e) => cardClick(e, idx, entry)}
		ondblclick={() => cardDblClick(entry)}
		oncontextmenu={(e) => cardContext(e, entry)}
	>
		<div class="thumb" style="height: {dim.icon + 8}px">
			{#if thumbSrc(entry)}
				<img
					class="thumb-img"
					src={thumbSrc(entry)}
					alt={entry.name}
					style="max-width: {dim.icon + 12}px; max-height: {dim.icon + 8}px"
					onerror={() => onThumbError(entry.path)}
				/>
			{:else}
				<span class="emoji" style="font-size: {dim.icon}px">{getFileIcon(entry)}</span>
			{/if}
		</div>
		<span class="name" style="font-size: {dim.font}px">{entry.name}</span>
	</div>
{/snippet}

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_noninteractive_tabindex -->
<div
	class="gridview"
	role="grid"
	tabindex="0"
	bind:this={containerEl}
	onscroll={onScroll}
	onmousedown={onContainerMouseDown}
	onkeydown={onKeydown}
>
	{#if virtual}
		<div class="spacer" style="height: {range.padTop}px"></div>
		<div class="grid" style="grid-template-columns: repeat({cols}, minmax({dim.card}px, 1fr)); gap: {GAP}px">
			{#each visible as entry, i (entry.path)}
				{@render card(entry, range.start + i)}
			{/each}
		</div>
		<div class="spacer" style="height: {range.padBottom}px"></div>
	{:else}
		<div class="grid" style="grid-template-columns: repeat(auto-fill, minmax({dim.card}px, 1fr)); gap: {GAP}px">
			{#each entries as entry, i (entry.path)}
				{@render card(entry, i)}
			{/each}
		</div>
	{/if}

	{#if rubber.active}
		<div
			class="rubber"
			style="left: {rubberRect.left}px; top: {rubberRect.top}px; width: {rubberRect.width}px; height: {rubberRect.height}px"
		></div>
	{/if}
</div>

<style>
	.gridview {
		position: relative;
		height: 100%;
		overflow-y: auto;
		overflow-x: hidden;
		padding: 8px;
		outline: none;
		font-family: 'DM Sans', system-ui, sans-serif;
	}
	.spacer {
		width: 100%;
	}
	.grid {
		display: grid;
		align-content: start;
	}
	.card {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 4px;
		padding: 8px;
		border: 1px solid transparent;
		border-radius: 6px;
		background: transparent;
		color: #c8c8dc;
		cursor: default;
		transition:
			background 80ms ease,
			border-color 80ms ease;
		overflow: hidden;
	}
	.card:hover {
		background: #111120;
		border-color: #1e1e35;
	}
	.card.selected {
		background: #1a1a2e;
		border-color: #a78bfa55;
	}
	.card.selected .name {
		color: #a78bfa;
	}
	.card.focused {
		outline: 1px solid #a78bfa;
		outline-offset: -1px;
	}
	.card.hiddenrow {
		opacity: 0.5;
	}
	.thumb {
		display: flex;
		align-items: center;
		justify-content: center;
	}
	.thumb-img {
		object-fit: contain;
		border-radius: 3px;
	}
	.emoji {
		line-height: 1;
	}
	.name {
		width: 100%;
		text-align: center;
		line-height: 1.2;
		overflow: hidden;
		display: -webkit-box;
		-webkit-line-clamp: 2;
		line-clamp: 2;
		-webkit-box-orient: vertical;
		word-break: break-word;
	}
	.rubber {
		position: fixed;
		z-index: 40;
		background: #a78bfa15;
		border: 1px solid #a78bfa55;
		pointer-events: none;
	}
</style>
