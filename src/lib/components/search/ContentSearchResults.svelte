<script lang="ts">
	import { onMount } from 'svelte';
	import { get } from 'svelte/store';
	import ContentPreview from '$lib/components/ui/ContentPreview.svelte';
	import Highlight from '$lib/components/ui/Highlight.svelte';
	import { extractQueryTerms } from '$lib/utils/highlightUtils';
	import { getFileIcon } from '$lib/utils/fileUtils';

	const CODE_EXTS = new Set([
		'rs', 'ts', 'js', 'tsx', 'jsx', 'svelte', 'py', 'go', 'rb', 'php', 'java',
		'kt', 'swift', 'c', 'cpp', 'h', 'cs', 'json', 'toml', 'yaml', 'yml', 'sh'
	]);
	function isCodeFile(ext: string | null): boolean {
		return ext ? CODE_EXTS.has(ext.toLowerCase()) : false;
	}
	import { activeTabId, navigateTab, openTab } from '$lib/stores/tabs';
	import {
		contentResults,
		contentQuery,
		contentLoading,
		contentError,
		exitContentMode,
		executeContentSearch,
		reindexCurrentDirectory,
		indexStats
	} from '$lib/stores/contentSearch';
	import type { ContentSearchResult } from '$lib/tauri';

	type SortBy = 'relevance' | 'date' | 'name';
	let sortBy = $state<SortBy>('relevance');
	let extFilter = $state<Set<string>>(new Set());
	let menu = $state<{ visible: boolean; x: number; y: number; res: ContentSearchResult | null }>({
		visible: false, x: 0, y: 0, res: null
	});

	let availableExts = $derived.by(() => {
		const s = new Set<string>();
		for (const r of $contentResults) if (r.extension) s.add(r.extension);
		return [...s].sort();
	});

	let displayed = $derived.by(() => {
		let r = $contentResults;
		if (extFilter.size > 0) r = r.filter((x) => x.extension && extFilter.has(x.extension));
		const arr = [...r];
		if (sortBy === 'relevance') arr.sort((a, b) => b.score - a.score);
		else if (sortBy === 'date') arr.sort((a, b) => (b.modified ?? '').localeCompare(a.modified ?? ''));
		else arr.sort((a, b) => a.name.localeCompare(b.name, 'pt-BR'));
		return arr;
	});

	let indexEmpty = $derived(($indexStats?.total_documents ?? 0) === 0);

	function scoreColor(s: number): string {
		if (s > 0.8) return '#34d399';
		if (s > 0.5) return '#a78bfa';
		return '#555';
	}

	function toggleExt(e: string) {
		const next = new Set(extFilter);
		if (next.has(e)) next.delete(e);
		else next.add(e);
		extFilter = next;
	}

	function open(res: ContentSearchResult) {
		const id = get(activeTabId);
		if (id) navigateTab(id, res.parent_path);
		exitContentMode();
	}

	function openMenu(e: MouseEvent, res: ContentSearchResult) {
		e.preventDefault();
		menu = { visible: true, x: e.clientX, y: e.clientY, res };
	}
	function closeMenu() {
		menu = { ...menu, visible: false };
	}
	function menuGo() {
		if (menu.res) {
			const id = get(activeTabId);
			if (id) navigateTab(id, menu.res.parent_path);
			exitContentMode();
		}
		closeMenu();
	}
	function menuCopy() {
		if (menu.res) navigator.clipboard?.writeText(menu.res.path).catch(() => {});
		closeMenu();
	}
	function menuNewTab() {
		if (menu.res) openTab(menu.res.parent_path);
		closeMenu();
	}

	onMount(() => {
		document.addEventListener('click', closeMenu);
		return () => document.removeEventListener('click', closeMenu);
	});
</script>

<div class="content-res">
	<!-- HEADER -->
	<div class="head">
		<span class="hicon">📄</span>
		<span class="q">"{$contentQuery}"</span>
		<span class="dot">·</span>
		{#if $contentLoading}
			<span class="muted">buscando…</span>
		{:else}
			<span class="muted">{$contentResults.length} resultados</span>
		{/if}
		<span class="pill">Conteúdo</span>
		<button class="close" onclick={exitContentMode} aria-label="Fechar">×</button>
	</div>

	<!-- TOOLBAR -->
	{#if !indexEmpty && $contentResults.length > 0}
		<div class="toolbar">
			<div class="exts">
				{#each availableExts as e (e)}
					<button class="ext" class:sel={extFilter.has(e)} onclick={() => toggleExt(e)}>{e}</button>
				{/each}
			</div>
			<select class="sort" bind:value={sortBy}>
				<option value="relevance">Relevância</option>
				<option value="date">Data</option>
				<option value="name">Nome</option>
			</select>
			<span class="count">{displayed.length} resultados</span>
		</div>
	{/if}

	<!-- CONTEÚDO -->
	<div class="body">
		{#if $contentError}
			<div class="center err">
				<span class="big">⚠️</span>
				<span>{$contentError}</span>
				<button class="action" onclick={() => executeContentSearch()}>Tentar novamente</button>
			</div>
		{:else if $contentLoading}
			<div class="skeleton">
				{#each Array(5) as _, i (i)}<div class="sk"></div>{/each}
			</div>
		{:else if indexEmpty}
			<div class="center muted">
				<span class="big">📄</span>
				<span>Índice vazio — indexe a pasta para buscar no conteúdo</span>
				<button class="action" onclick={reindexCurrentDirectory}>Indexar agora</button>
			</div>
		{:else if displayed.length === 0}
			<div class="center muted">
				<span class="big">📄</span>
				<span>Nenhum resultado para "{$contentQuery}" no conteúdo dos arquivos</span>
				<span class="hint">Certifique-se de que a pasta foi indexada</span>
				<button class="action" onclick={reindexCurrentDirectory}>Indexar pasta atual</button>
			</div>
		{:else}
			<div class="list">
				{#each displayed as res (res.path)}
					<div
						class="card"
						role="button"
						tabindex="0"
						onclick={() => open(res)}
						ondblclick={() => open(res)}
						oncontextmenu={(e) => openMenu(e, res)}
						onkeydown={(e) => e.key === 'Enter' && open(res)}
					>
						<div class="r1">
							<span class="ico">{getFileIcon(res)}</span>
							<span class="name">
								<Highlight
									text={res.name}
									terms={extractQueryTerms($contentQuery)}
									source="content"
									highlightClass="hl-content"
								/>
							</span>
							<span class="parent">
								<Highlight
									text={res.parent_path}
									terms={extractQueryTerms($contentQuery)}
									source="filter"
									highlightClass="hl-filter"
									maxChars={50}
									padding={10}
								/>
							</span>
							<span class="badge" style="color: {scoreColor(res.score)}; border-color: {scoreColor(res.score)}33">
								{res.score.toFixed(2)}
							</span>
						</div>
						{#if res.previews.length}
							<ContentPreview
								previews={res.previews}
								maxPreviews={2}
								previewMaxChars={180}
								font={isCodeFile(res.extension) ? 'mono' : 'sans'}
							/>
						{/if}
					</div>
				{/each}
			</div>
		{/if}
	</div>
</div>

{#if menu.visible && menu.res}
	<div class="ctx" style="left: {menu.x}px; top: {menu.y}px" role="menu" tabindex="-1">
		<button role="menuitem" onclick={menuGo}>Ir para a pasta</button>
		<button role="menuitem" onclick={menuCopy}>Copiar caminho</button>
		<button role="menuitem" onclick={menuNewTab}>Abrir em nova tab</button>
	</div>
{/if}

<style>
	.content-res {
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
	.pill {
		padding: 2px 8px;
		border-radius: 10px;
		background: #38bdf820;
		border: 1px solid #38bdf830;
		color: #38bdf8;
		font-size: 11px;
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
		min-height: 34px;
		padding: 4px 12px;
		border-bottom: 1px solid #12121e;
		background: #0c0c18;
		flex-shrink: 0;
		flex-wrap: wrap;
	}
	.exts {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
	}
	.ext {
		padding: 2px 8px;
		border: 1px solid #1e1e35;
		border-radius: 10px;
		background: #111120;
		color: #666;
		font-family: inherit;
		font-size: 11px;
		cursor: pointer;
	}
	.ext.sel {
		background: #1e1e35;
		color: #a78bfa;
		border-color: #a78bfa44;
	}
	.sort {
		height: 24px;
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
		padding: 8px;
	}
	.card {
		padding: 12px;
		margin-bottom: 4px;
		background: #0e0e1c;
		border: 1px solid #1a1a26;
		border-radius: 6px;
		cursor: pointer;
		transition: border-color 120ms ease;
	}
	.card:hover {
		border-color: #2a2a38;
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
		font-size: 13px;
		font-weight: 600;
		color: #dddde8;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.parent {
		flex: 1;
		text-align: right;
		color: #444;
		font-size: 11px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.badge {
		flex-shrink: 0;
		padding: 1px 7px;
		border: 1px solid;
		border-radius: 9px;
		font-family: 'DM Mono', ui-monospace, monospace;
		font-size: 11px;
	}

	.center {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 8px;
		height: 100%;
		text-align: center;
	}
	.err {
		color: #f87171;
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
	}
	.sk {
		height: 64px;
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
