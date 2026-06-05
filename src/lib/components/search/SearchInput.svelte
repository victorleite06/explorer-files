<script lang="ts">
	import { onMount } from 'svelte';
	import { get } from 'svelte/store';
	import { activeTab } from '$lib/stores/tabs';
	import {
		searchQuery,
		searchSuggestions,
		showSuggestions,
		isSearchMode,
		searchScope,
		searchMode,
		updateQuery,
		executeSearch,
		exitSearchMode,
		selectSuggestion,
		toggleSearchMode
	} from '$lib/stores/search';
	import SearchSuggestions from './SearchSuggestions.svelte';

	let isContent = $derived($searchMode === 'content');
	function setMode(m: 'name' | 'content') {
		if ($searchMode !== m) toggleSearchMode();
	}

	let inputEl = $state<HTMLInputElement>();
	let wrapEl = $state<HTMLDivElement>();
	let focused = $state(false);
	let selectedIndex = $state(-1);
	let blurTimer: ReturnType<typeof setTimeout> | undefined;

	let folderName = $derived.by(() => {
		const p = $activeTab?.path ?? '';
		const sep = p.includes('\\') ? '\\' : '/';
		const parts = p.split(sep).filter(Boolean);
		return parts.length ? parts[parts.length - 1] : 'pasta';
	});

	let visibleCount = $derived(Math.min(8, $searchSuggestions.length));

	function onInput(e: Event) {
		const v = (e.target as HTMLInputElement).value;
		if (isContent) {
			// Modo conteúdo: sem sugestões em tempo real, só busca no Enter.
			searchQuery.set(v);
			showSuggestions.set(false);
		} else {
			updateQuery(v);
		}
		selectedIndex = -1;
	}

	function onKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			e.preventDefault();
			if (selectedIndex >= 0 && selectedIndex < visibleCount) {
				selectSuggestion($searchSuggestions[selectedIndex]);
			} else {
				executeSearch();
				showSuggestions.set(false);
			}
		} else if (e.key === 'Escape') {
			e.preventDefault();
			if (get(showSuggestions)) {
				showSuggestions.set(false);
			} else if (get(isSearchMode)) {
				exitSearchMode();
			} else {
				inputEl?.blur();
			}
		} else if (e.key === 'ArrowDown') {
			e.preventDefault();
			if (visibleCount > 0) selectedIndex = (selectedIndex + 1) % visibleCount;
		} else if (e.key === 'ArrowUp') {
			e.preventDefault();
			if (visibleCount > 0) selectedIndex = (selectedIndex - 1 + visibleCount) % visibleCount;
		}
	}

	function onClear() {
		exitSearchMode();
		selectedIndex = -1;
	}

	function onFocus() {
		focused = true;
		if (blurTimer) clearTimeout(blurTimer);
	}
	function onBlur() {
		focused = false;
		// Atraso para permitir clique nas sugestões.
		blurTimer = setTimeout(() => showSuggestions.set(false), 150);
	}

	function onGlobalKey(e: KeyboardEvent) {
		if (e.ctrlKey && e.key.toLowerCase() === 'f') {
			e.preventDefault();
			inputEl?.focus();
			inputEl?.select();
		}
	}

	onMount(() => {
		document.addEventListener('keydown', onGlobalKey);
		return () => document.removeEventListener('keydown', onGlobalKey);
	});
</script>

<div class="search" class:focused bind:this={wrapEl}>
	<span class="icon">{isContent ? '📄' : '🔍'}</span>
	<input
		bind:this={inputEl}
		class="input"
		type="text"
		placeholder={isContent ? 'Buscar no conteúdo dos arquivos…' : `Buscar em ${folderName}…`}
		value={$searchQuery}
		spellcheck="false"
		oninput={onInput}
		onkeydown={onKeydown}
		onfocus={onFocus}
		onblur={onBlur}
	/>

	{#if $searchQuery.length > 0 || focused}
		<div class="mode">
			<button class="m" class:on={!isContent} onclick={() => setMode('name')}>Nome</button>
			<button class="m" class:on={isContent} onclick={() => setMode('content')}>Conteúdo</button>
		</div>
		<select class="scope" bind:value={$searchScope} title="Escopo da busca">
			<option value="current">{isContent ? 'Nesta pasta' : 'Pasta atual'}</option>
			<option value="recursive">Recursivo</option>
		</select>
	{/if}

	{#if $searchQuery.length > 0 || $isSearchMode}
		<button class="clear" onclick={onClear} aria-label="Limpar busca">×</button>
	{:else if !focused}
		<span class="kbd">Ctrl+F</span>
	{/if}
</div>

{#if $showSuggestions}
	<SearchSuggestions anchorEl={wrapEl ?? null} {selectedIndex} />
{/if}

<style>
	.search {
		display: flex;
		align-items: center;
		gap: 6px;
		width: 220px;
		height: 30px;
		padding: 0 8px;
		background: #111120;
		border: 1px solid #1e1e35;
		border-radius: 6px;
		font-family: 'DM Sans', system-ui, sans-serif;
		transition:
			width 200ms ease,
			border-color 200ms ease;
		flex-shrink: 0;
	}
	.search.focused {
		width: 340px;
		border-color: #2a2a3a;
	}
	.icon {
		font-size: 13px;
		color: #333;
		flex-shrink: 0;
	}
	.input {
		flex: 1;
		min-width: 0;
		border: none;
		background: none;
		color: #c8c8dc;
		font-family: inherit;
		font-size: 13px;
		outline: none;
	}
	.input::placeholder {
		color: #444;
	}
	.mode {
		display: flex;
		flex-shrink: 0;
		border-radius: 5px;
		overflow: hidden;
	}
	.m {
		border: none;
		background: transparent;
		color: #444;
		font-family: inherit;
		font-size: 10px;
		padding: 3px 6px;
		cursor: pointer;
	}
	.m.on {
		background: #1e1e35;
		color: #a78bfa;
	}
	.scope {
		flex-shrink: 0;
		border: none;
		background: transparent;
		color: #555;
		font-family: inherit;
		font-size: 11px;
		outline: none;
		cursor: pointer;
	}
	.clear {
		flex-shrink: 0;
		width: 18px;
		height: 18px;
		border: none;
		border-radius: 4px;
		background: none;
		color: #555;
		font-size: 14px;
		line-height: 1;
		cursor: pointer;
	}
	.clear:hover {
		background: #1e1e35;
		color: #c8c8dc;
	}
	.kbd {
		flex-shrink: 0;
		color: #2a2a42;
		font-size: 10px;
	}
</style>
