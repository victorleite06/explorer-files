<script lang="ts">
	import Highlight from '$lib/components/ui/Highlight.svelte';
	import { extractQueryTerms } from '$lib/utils/highlightUtils';
	import { getFileIcon } from '$lib/utils/fileUtils';
	import {
		searchSuggestions,
		suggestionsLoading,
		searchQuery,
		recentSearches,
		selectSuggestion,
		updateQuery,
		executeSearch,
		removeRecentSearch
	} from '$lib/stores/search';

	function shortenPath(path: string, maxLen: number): string {
		if (path.length <= maxLen) return path;
		return '...' + path.slice(path.length - maxLen);
	}

	let {
		anchorEl,
		selectedIndex
	}: { anchorEl: HTMLElement | null; selectedIndex: number } = $props();

	let visible = $derived($searchSuggestions.slice(0, 8));
	let showRecents = $derived($searchQuery.length === 0 && $recentSearches.length > 0);

	// Posicionamento abaixo do anchor.
	let pos = $state({ left: 0, top: 0, width: 320 });
	$effect(() => {
		void $searchSuggestions;
		void $suggestionsLoading;
		if (!anchorEl) return;
		const r = anchorEl.getBoundingClientRect();
		pos = { left: r.left, top: r.bottom, width: Math.max(320, r.width) };
	});

	function runRecent(q: string) {
		updateQuery(q);
		executeSearch(q);
	}
</script>

<div class="dropdown" style="left: {pos.left}px; top: {pos.top}px; width: {pos.width}px" role="listbox">
	{#if $suggestionsLoading}
		<div class="loading">
			<svg class="spin" viewBox="0 0 24 24" width="12" height="12">
				<circle cx="12" cy="12" r="9" fill="none" stroke="#a78bfa" stroke-width="3"
					stroke-dasharray="42" stroke-linecap="round" />
			</svg>
			<span>Buscando…</span>
		</div>
	{:else if visible.length > 0}
		{#each visible as res, i (res.path)}
			<button
				class="item"
				class:sel={i === selectedIndex}
				onclick={() => selectSuggestion(res)}
				role="option"
				aria-selected={i === selectedIndex}
			>
				<span class="ico">{getFileIcon(res)}</span>
				<span class="name">
					<Highlight text={res.name} indices={res.match_indices} source="fuzzy" maxChars={40} />
				</span>
				<span class="parent">
					<Highlight
						text={shortenPath(res.parent_path, 30)}
						terms={extractQueryTerms($searchQuery)}
						source="filter"
						highlightClass="hl-filter"
					/>
				</span>
			</button>
		{/each}
	{:else if $searchQuery.length >= 2}
		<div class="empty">Nenhuma sugestão</div>
	{/if}

	{#if showRecents}
		{#if visible.length > 0}<div class="sep"></div>{/if}
		<div class="rec-head">Buscas recentes</div>
		{#each $recentSearches as q (q)}
			<div class="rec">
				<button class="rec-q" onclick={() => runRecent(q)}>
					<span class="clock">🕐</span>
					<span class="rec-text">{q}</span>
				</button>
				<button class="rec-x" onclick={() => removeRecentSearch(q)} aria-label="Remover">×</button>
			</div>
		{/each}
	{/if}
</div>

<style>
	.dropdown {
		position: fixed;
		z-index: 500;
		max-height: 320px;
		overflow-y: auto;
		background: #111120;
		border: 1px solid #1e1e35;
		border-radius: 0 0 8px 8px;
		box-shadow: 0 8px 24px #00000055;
		font-family: 'DM Sans', system-ui, sans-serif;
	}
	.loading,
	.empty {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 12px;
		color: #555;
		font-size: 12px;
	}
	.spin {
		animation: rot 0.8s linear infinite;
	}
	@keyframes rot {
		to {
			transform: rotate(360deg);
		}
	}
	.item {
		display: flex;
		align-items: center;
		gap: 8px;
		width: 100%;
		height: 34px;
		padding: 0 12px;
		border: none;
		border-left: 2px solid transparent;
		background: none;
		color: #c8c8dc;
		font-family: inherit;
		font-size: 13px;
		text-align: left;
		cursor: pointer;
	}
	.item:hover {
		background: #1a1a2e;
	}
	.item.sel {
		background: #1e1e35;
		border-left: 2px solid #a78bfa;
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
	.parent {
		max-width: 140px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		color: #444;
		font-size: 11px;
		flex-shrink: 0;
	}
	.sep {
		height: 1px;
		background: #1e1e35;
		margin: 4px 0;
	}
	.rec-head {
		padding: 6px 12px 2px;
		color: #333;
		font-size: 9px;
		text-transform: uppercase;
		letter-spacing: 0.06em;
	}
	.rec {
		display: flex;
		align-items: center;
	}
	.rec-q {
		display: flex;
		align-items: center;
		gap: 8px;
		flex: 1;
		height: 28px;
		padding: 0 12px;
		border: none;
		background: none;
		color: #888;
		font-family: inherit;
		font-size: 12px;
		text-align: left;
		cursor: pointer;
	}
	.rec-q:hover {
		background: #1a1a2e;
	}
	.clock {
		font-size: 11px;
	}
	.rec-text {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.rec-x {
		width: 24px;
		border: none;
		background: none;
		color: #2a2a42;
		font-size: 13px;
		cursor: pointer;
	}
	.rec-x:hover {
		color: #f87171;
	}
</style>
