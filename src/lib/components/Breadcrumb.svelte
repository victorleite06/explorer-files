<script lang="ts">
	import { onMount, onDestroy, tick } from 'svelte';
	import { listDirectory, type FileEntry } from '$lib/tauri';
	import { getHomePath } from '$lib/stores/tabs';

	let { path, onNavigate }: { path: string; onNavigate: (path: string) => void } = $props();

	interface Seg {
		icon: string;
		label: string;
		path: string;
	}

	type Mode = 'display' | 'edit';
	let mode = $state<Mode>('display');
	let expanded = $state(false);

	// Edição
	let editValue = $state('');
	let invalid = $state(false);
	let inputEl = $state<HTMLInputElement>();
	let suggestions = $state<FileEntry[]>([]);
	let activeSuggestion = $state(-1);

	let debounceTimer: ReturnType<typeof setTimeout> | undefined;
	let invalidTimer: ReturnType<typeof setTimeout> | undefined;

	// ── Segmentos do path ─────────────────────────────────────────
	let segments = $derived.by<Seg[]>(() => {
		if (!path) return [];
		const sep = path.includes('\\') ? '\\' : '/';
		const isWin = sep === '\\';
		const home = getHomePath();
		const parts = path.split(sep).filter((p) => p.length > 0);

		const segs: Seg[] = [];
		if (!isWin) segs.push({ icon: '🖥️', label: 'Raiz', path: '/' });

		let acc = '';
		parts.forEach((part, i) => {
			if (isWin) {
				acc = i === 0 ? part + sep : (acc.endsWith(sep) ? acc : acc + sep) + part;
			} else {
				acc = `${acc}/${part}`;
			}
			let icon = '📁';
			let label = part;
			if (isWin && i === 0) icon = '💾';
			if (home && acc === home) {
				icon = '🏠';
				label = 'Home';
			}
			segs.push({ icon, label, path: acc });
		});
		return segs;
	});

	// Recolhe o meio quando há muitos segmentos.
	let collapsed = $derived(segments.length > 4 && !expanded);

	type Cell = { type: 'seg'; seg: Seg; i: number } | { type: 'ellipsis' };
	let cells = $derived.by<Cell[]>(() => {
		if (!collapsed) return segments.map((seg, i) => ({ type: 'seg', seg, i }));
		const n = segments.length;
		return [
			{ type: 'seg', seg: segments[0], i: 0 },
			{ type: 'ellipsis' },
			{ type: 'seg', seg: segments[n - 2], i: n - 2 },
			{ type: 'seg', seg: segments[n - 1], i: n - 1 }
		];
	});

	// ── Modo edição ───────────────────────────────────────────────
	async function enterEdit() {
		if (mode === 'edit') return;
		mode = 'edit';
		editValue = path;
		invalid = false;
		suggestions = [];
		activeSuggestion = -1;
		await tick();
		inputEl?.focus();
		inputEl?.select();
	}

	function exitEdit() {
		mode = 'display';
		editValue = path;
		invalid = false;
		suggestions = [];
		activeSuggestion = -1;
		if (debounceTimer) clearTimeout(debounceTimer);
	}

	// Método público (chamado pelo TopBar ao trocar de tab).
	export function cancelEdit() {
		if (mode === 'edit') exitEdit();
	}

	function navigateSeg(e: MouseEvent, segPath: string) {
		e.stopPropagation();
		onNavigate(segPath);
	}

	function expand(e: MouseEvent) {
		e.stopPropagation();
		expanded = true;
	}

	// ── Sugestões ─────────────────────────────────────────────────
	function splitParentFragment(value: string): { parent: string; fragment: string } {
		const sep = value.includes('\\') ? '\\' : '/';
		const last = value.lastIndexOf(sep);
		if (last < 0) return { parent: value, fragment: '' };
		return { parent: value.slice(0, last + 1), fragment: value.slice(last + 1) };
	}

	async function fetchSuggestions(value: string) {
		const { parent, fragment } = splitParentFragment(value);
		try {
			const items = await listDirectory(parent);
			suggestions = items
				.filter((e) => e.is_dir && e.name.toLowerCase().startsWith(fragment.toLowerCase()))
				.slice(0, 8);
			activeSuggestion = -1;
		} catch {
			suggestions = [];
		}
	}

	function onInput() {
		if (debounceTimer) clearTimeout(debounceTimer);
		debounceTimer = setTimeout(() => fetchSuggestions(editValue), 300);
	}

	function pickSuggestion(s: FileEntry) {
		editValue = s.path;
		inputEl?.focus();
		// Lista os filhos da pasta escolhida.
		fetchSuggestions(s.path);
	}

	// ── Confirmar / validar ───────────────────────────────────────
	async function confirm() {
		const value = editValue;
		try {
			await listDirectory(value);
			onNavigate(value);
			mode = 'display';
			suggestions = [];
		} catch {
			invalid = true;
			if (invalidTimer) clearTimeout(invalidTimer);
			invalidTimer = setTimeout(() => (invalid = false), 2000);
		}
	}

	function onInputKeydown(e: KeyboardEvent) {
		if (e.key === 'ArrowDown') {
			e.preventDefault();
			if (suggestions.length) activeSuggestion = (activeSuggestion + 1) % suggestions.length;
		} else if (e.key === 'ArrowUp') {
			e.preventDefault();
			if (suggestions.length)
				activeSuggestion = (activeSuggestion - 1 + suggestions.length) % suggestions.length;
		} else if (e.key === 'Tab') {
			e.preventDefault();
			const pick = suggestions[activeSuggestion >= 0 ? activeSuggestion : 0];
			if (pick) pickSuggestion(pick);
		} else if (e.key === 'Enter') {
			e.preventDefault();
			confirm();
		} else if (e.key === 'Escape') {
			e.preventDefault();
			exitEdit();
		}
	}

	// ── Evento externo (F4 / Ctrl+L do NavButtons) ────────────────
	function onEditRequest() {
		enterEdit();
	}

	onMount(() => {
		window.addEventListener('breadcrumb:edit', onEditRequest);
		return () => window.removeEventListener('breadcrumb:edit', onEditRequest);
	});

	onDestroy(() => {
		if (debounceTimer) clearTimeout(debounceTimer);
		if (invalidTimer) clearTimeout(invalidTimer);
	});
</script>

<div class="breadcrumb">
	{#if mode === 'display'}
		<!-- Área vazia clicável entra em edição; segmentos param a propagação. -->
		<div
			class="display"
			role="button"
			tabindex="0"
			title="Clique para editar o caminho"
			onclick={enterEdit}
			onkeydown={(e) => e.key === 'Enter' && enterEdit()}
		>
			{#each cells as cell, idx (idx)}
				{#if idx > 0}<span class="sep">›</span>{/if}
				{#if cell.type === 'ellipsis'}
					<button class="seg ellipsis" onclick={expand} title="Expandir caminho">···</button>
				{:else}
					<button
						class="seg"
						class:last={cell.i === segments.length - 1}
						style="animation-delay: {idx * 30}ms"
						onclick={(e) => navigateSeg(e, cell.seg.path)}
					>
						<span class="seg-ico">{cell.seg.icon}</span>
						<span class="seg-label">{cell.seg.label}</span>
					</button>
				{/if}
			{/each}
		</div>
	{:else}
		<div class="edit">
			<div class="input-wrap" class:invalid>
				<input
					bind:this={inputEl}
					bind:value={editValue}
					class="path-input"
					spellcheck="false"
					autocomplete="off"
					oninput={onInput}
					onkeydown={onInputKeydown}
				/>
				{#if invalid}
					<span class="tooltip">Caminho não encontrado</span>
				{/if}
			</div>
			<button class="act confirm" title="Confirmar" onclick={confirm} aria-label="Confirmar">↵</button>
			<button class="act cancel" title="Cancelar" onclick={exitEdit} aria-label="Cancelar">✕</button>

			{#if suggestions.length}
				<ul class="suggestions" role="listbox">
					{#each suggestions as s, i (s.path)}
						<li>
							<button
								class="suggestion"
								class:active={i === activeSuggestion}
								onclick={() => pickSuggestion(s)}
							>
								<span class="sug-ico">📁</span>
								<span class="sug-name">{s.name}</span>
							</button>
						</li>
					{/each}
				</ul>
			{/if}
		</div>
	{/if}
</div>

<style>
	.breadcrumb {
		position: relative;
		display: flex;
		align-items: center;
		height: 44px;
		flex: 1;
		min-width: 0;
		font-family: 'DM Sans', system-ui, sans-serif;
	}

	/* ── Display ── */
	.display {
		display: flex;
		align-items: center;
		gap: 2px;
		flex: 1;
		min-width: 0;
		height: 100%;
		overflow: hidden;
		white-space: nowrap;
		cursor: text;
		animation: fade 150ms ease;
	}
	.seg {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 2px 6px;
		border: none;
		border-radius: 4px;
		background: transparent;
		color: #555;
		font-family: inherit;
		font-size: 13px;
		cursor: pointer;
		transition:
			color 150ms ease,
			background 150ms ease;
		animation: segin 150ms ease both;
	}
	.seg:hover {
		color: #888;
		background: #16162a;
	}
	.seg.last {
		color: #dddde8;
		font-weight: 500;
	}
	.seg.ellipsis {
		color: #6a6a82;
		letter-spacing: 1px;
	}
	.seg-ico {
		font-size: 12px;
	}
	.seg-label {
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.sep {
		padding: 0 2px;
		color: #2a2a42;
		pointer-events: none;
	}

	@keyframes segin {
		from {
			opacity: 0;
			transform: translateX(-4px);
		}
		to {
			opacity: 1;
			transform: none;
		}
	}
	@keyframes fade {
		from {
			opacity: 0;
		}
		to {
			opacity: 1;
		}
	}

	/* ── Edição ── */
	.edit {
		position: relative;
		display: flex;
		align-items: center;
		gap: 6px;
		flex: 1;
		min-width: 0;
		animation: fade 150ms ease;
	}
	.input-wrap {
		position: relative;
		flex: 1;
		min-width: 0;
	}
	.path-input {
		width: 100%;
		height: 30px;
		padding: 0 10px;
		background: #111120;
		border: 1px solid #a78bfa;
		border-radius: 6px;
		color: #dddde8;
		font-family: 'DM Mono', ui-monospace, monospace;
		font-size: 13px;
		outline: none;
	}
	.input-wrap.invalid .path-input {
		border-color: #f87171;
	}
	.tooltip {
		position: absolute;
		top: calc(100% + 4px);
		left: 0;
		padding: 4px 8px;
		background: #2a1414;
		border: 1px solid #f87171;
		border-radius: 5px;
		color: #f87171;
		font-size: 11px;
		white-space: nowrap;
		z-index: 50;
	}
	.act {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		flex-shrink: 0;
		border: 1px solid #1e1e35;
		border-radius: 6px;
		background: #111120;
		color: #9090a8;
		font-size: 13px;
		cursor: pointer;
		transition:
			background 150ms ease,
			color 150ms ease;
	}
	.act.confirm:hover {
		background: #1a2e1a;
		color: #86efac;
		border-color: #2a422a;
	}
	.act.cancel:hover {
		background: #2e1a1a;
		color: #f87171;
		border-color: #422a2a;
	}

	/* ── Sugestões ── */
	.suggestions {
		position: absolute;
		top: calc(100% + 2px);
		left: 0;
		right: 60px;
		z-index: 60;
		margin: 0;
		padding: 4px;
		list-style: none;
		background: #14142a;
		border: 1px solid #1e1e35;
		border-radius: 8px;
		box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
		max-height: 280px;
		overflow-y: auto;
	}
	.suggestion {
		display: flex;
		align-items: center;
		gap: 8px;
		width: 100%;
		padding: 6px 8px;
		border: none;
		border-radius: 5px;
		background: none;
		color: #c8c8dc;
		font-family: 'DM Mono', ui-monospace, monospace;
		font-size: 12px;
		text-align: left;
		cursor: pointer;
	}
	.suggestion:hover,
	.suggestion.active {
		background: #1a1a2e;
	}
	.sug-name {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
</style>
