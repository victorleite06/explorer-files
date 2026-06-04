<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { get } from 'svelte/store';
	import { getDrives, type FileEntry } from '$lib/tauri';
	import { treeRoot, navigateTo } from '$lib/stores/explorer';
	import { activeTab, openTab } from '$lib/stores/tabs';
	import { quickAccessItems, addBookmarkFromPath } from '$lib/stores/bookmarks';
	import TreeNode from './TreeNode.svelte';
	import QuickAccessItem from './bookmarks/QuickAccessItem.svelte';
	import BookmarkSection from './bookmarks/BookmarkSection.svelte';

	let drives = $state<FileEntry[]>([]);
	let favArea = $state<HTMLDivElement>();

	// ── Colapso por seção (persistido em localStorage) ────────────
	function loadCollapsed(key: string): boolean {
		if (typeof localStorage === 'undefined') return false;
		return localStorage.getItem(`sidebar-section-${key}`) === '1';
	}
	let collapsed = $state<Record<string, boolean>>({
		quick: loadCollapsed('quick'),
		devices: loadCollapsed('devices'),
		folders: loadCollapsed('folders')
	});
	function toggle(key: string) {
		collapsed[key] = !collapsed[key];
		if (typeof localStorage !== 'undefined') {
			localStorage.setItem(`sidebar-section-${key}`, collapsed[key] ? '1' : '0');
		}
	}

	// ── Dispositivos ──────────────────────────────────────────────
	function driveClick(e: MouseEvent, path: string) {
		if (e.ctrlKey) openTab(path);
		else navigateTo(path);
	}

	// ── Drag global p/ favoritos (drops fora da drop zone) ────────
	function hasBookmarkPath(e: DragEvent): boolean {
		return e.dataTransfer?.types.includes('bookmark-drop-path') ?? false;
	}
	function overFav(e: DragEvent): boolean {
		return !!favArea && e.target instanceof Node && favArea.contains(e.target);
	}
	function handleGlobalDragOver(e: DragEvent) {
		if (hasBookmarkPath(e) && overFav(e)) e.preventDefault(); // permite o drop
	}
	function handleGlobalDrop(e: DragEvent) {
		if (!hasBookmarkPath(e) || !overFav(e)) return;
		// A drop zone do BookmarkSection já faz stopPropagation; aqui só
		// pegamos drops na área de favoritos fora da zona dedicada.
		const path = e.dataTransfer?.getData('bookmark-drop-path');
		if (path) {
			e.preventDefault();
			addBookmarkFromPath(path);
		}
	}

	onMount(async () => {
		drives = await getDrives();
		document.addEventListener('dragover', handleGlobalDragOver);
		document.addEventListener('drop', handleGlobalDrop);
	});
	onDestroy(() => {
		document.removeEventListener('dragover', handleGlobalDragOver);
		document.removeEventListener('drop', handleGlobalDrop);
	});

	let activePath = $derived($activeTab?.path ?? '');
</script>

{#snippet sectionHeader(label: string, key: string)}
	<button class="sec-head" onclick={() => toggle(key)}>
		<span class="sec-arrow" class:open={!collapsed[key]}>▶</span>
		<span>{label}</span>
	</button>
{/snippet}

<aside class="panel">
	<!-- ACESSO RÁPIDO -->
	<section>
		{@render sectionHeader('Acesso Rápido', 'quick')}
		{#if !collapsed.quick}
			{#each $quickAccessItems as item (item.id)}
				<QuickAccessItem {item} isActive={item.path === activePath} />
			{/each}
		{/if}
	</section>

	<div class="sep"></div>

	<!-- DISPOSITIVOS -->
	<section>
		{@render sectionHeader('Dispositivos', 'devices')}
		{#if !collapsed.devices}
			{#each drives as drive (drive.path)}
				<button
					class="drive"
					class:active={drive.path === activePath}
					title={drive.path}
					onclick={(e) => driveClick(e, drive.path)}
				>
					<span class="ico">{drive.name === '/' ? '🖥️' : '💾'}</span>
					<span class="label">{drive.name === '/' ? 'Raiz' : drive.name}</span>
				</button>
			{/each}
		{/if}
	</section>

	<div class="sep"></div>

	<!-- FAVORITOS -->
	<div class="fav-area" bind:this={favArea}>
		<BookmarkSection />
	</div>

	<div class="sep"></div>

	<!-- PASTAS -->
	<section>
		{@render sectionHeader('Pastas', 'folders')}
		{#if !collapsed.folders}
			{#if $treeRoot}
				<TreeNode node={$treeRoot} depth={0} />
			{:else}
				<div class="skeleton">
					<div class="sk-line"></div>
					<div class="sk-line"></div>
					<div class="sk-line"></div>
				</div>
			{/if}
		{/if}
	</section>
</aside>

<style>
	.panel {
		width: 240px;
		height: 100%;
		background: #0e0e1c;
		overflow-y: auto;
		overflow-x: hidden;
		display: flex;
		flex-direction: column;
		font-family: 'DM Sans', system-ui, sans-serif;
	}

	.sec-head {
		display: flex;
		align-items: center;
		gap: 6px;
		width: 100%;
		padding: 8px 12px 4px;
		border: none;
		background: none;
		color: #333;
		font-family: inherit;
		font-size: 10px;
		text-transform: uppercase;
		letter-spacing: 0.1em;
		text-align: left;
		cursor: pointer;
		transition: color 120ms ease;
	}
	.sec-head:hover {
		color: #555;
	}
	.sec-arrow {
		font-size: 7px;
		transition: transform 120ms ease;
	}
	.sec-arrow.open {
		transform: rotate(90deg);
	}

	.sep {
		height: 1px;
		background: #1a1a2e;
		margin: 4px 0;
	}

	.drive {
		display: flex;
		align-items: center;
		gap: 8px;
		width: 100%;
		height: 28px;
		padding: 0 12px;
		border: none;
		border-left: 2px solid transparent;
		background: none;
		color: #666;
		font-family: inherit;
		font-size: 13px;
		text-align: left;
		cursor: pointer;
		transition:
			background 100ms ease,
			color 100ms ease;
	}
	.drive:hover {
		background: #111120;
		color: #9090a8;
	}
	.drive.active {
		background: #16162e;
		color: #dddde8;
		border-left: 2px solid #a78bfa;
	}
	.ico {
		font-size: 16px;
		width: 16px;
		text-align: center;
	}
	.label {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.fav-area {
		display: flex;
		flex-direction: column;
	}

	.skeleton {
		padding: 4px 12px;
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.sk-line {
		height: 14px;
		border-radius: 4px;
		background: linear-gradient(90deg, #16162a 25%, #1e1e35 50%, #16162a 75%);
		background-size: 200% 100%;
		animation: shimmer 1.2s linear infinite;
	}
	.sk-line:nth-child(2) {
		width: 80%;
	}
	.sk-line:nth-child(3) {
		width: 60%;
	}
	@keyframes shimmer {
		to {
			background-position: -200% 0;
		}
	}
</style>
