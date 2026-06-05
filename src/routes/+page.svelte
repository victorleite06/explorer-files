<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { listen } from '@tauri-apps/api/event';
	import TabBar from '$lib/components/TabBar.svelte';
	import SidePanel from '$lib/components/SidePanel.svelte';
	import TopBar from '$lib/components/TopBar.svelte';
	import { fade } from 'svelte/transition';
	import FileList from '$lib/components/FileList.svelte';
	import SearchResults from '$lib/components/search/SearchResults.svelte';
	import ContentSearchResults from '$lib/components/search/ContentSearchResults.svelte';
	import IndexingProgress from '$lib/components/indexer/IndexingProgress.svelte';
	import ToastContainer from '$lib/components/ToastContainer.svelte';
	import { initExplorer } from '$lib/stores/explorer';
	import { activeTabId } from '$lib/stores/tabs';
	import { isSearchMode, initRecentSearches } from '$lib/stores/search';
	import { isContentMode, isIndexing, destroyContentSearch } from '$lib/stores/contentSearch';
	import { isTauri } from '$lib/tauri';

	onDestroy(destroyContentSearch);

	onMount(() => {
		// initExplorer chama initTabs(home) internamente.
		initExplorer();
		initRecentSearches();

		// Listener de drag-drop (placeholder — integração futura).
		let unlisten: (() => void) | undefined;
		if (isTauri()) {
			listen('tauri://drag-drop', (event) => {
				console.log('drag-drop:', event.payload);
			}).then((fn) => {
				unlisten = fn;
			});
		}

		return () => unlisten?.();
	});
</script>

<div class="app">
	<TabBar />
	<TopBar />
	<div class="body">
		<div class="side">
			<SidePanel />
		</div>
		<div class="main">
			{#if $isIndexing}
				<IndexingProgress />
			{/if}
			<div class="panel-area">
				{#if $isSearchMode}
					<div class="fill" transition:fade={{ duration: 150 }}>
						<SearchResults />
					</div>
				{:else if $isContentMode}
					<div class="fill" transition:fade={{ duration: 150 }}>
						<ContentSearchResults />
					</div>
				{:else}
					{#key $activeTabId}
						<FileList />
					{/key}
				{/if}
			</div>
		</div>
	</div>
</div>

<ToastContainer />

<style>
	.app {
		display: flex;
		flex-direction: column;
		height: 100vh;
		overflow: hidden;
	}
	.body {
		display: flex;
		flex: 1;
		overflow: hidden;
	}
	.side {
		width: 240px;
		flex-shrink: 0;
		overflow-y: auto;
		border-right: 1px solid #12121e;
	}
	.main {
		flex: 1;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}
	.panel-area {
		flex: 1;
		overflow: hidden;
	}
	.fill {
		height: 100%;
	}
</style>
