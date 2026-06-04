<script lang="ts">
	import { onMount } from 'svelte';
	import { listen } from '@tauri-apps/api/event';
	import TabBar from '$lib/components/TabBar.svelte';
	import SidePanel from '$lib/components/SidePanel.svelte';
	import TopBar from '$lib/components/TopBar.svelte';
	import FileList from '$lib/components/FileList.svelte';
	import ToastContainer from '$lib/components/ToastContainer.svelte';
	import { initExplorer } from '$lib/stores/explorer';
	import { activeTabId } from '$lib/stores/tabs';
	import { isTauri } from '$lib/tauri';

	onMount(() => {
		// initExplorer chama initTabs(home) internamente.
		initExplorer();

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
			{#key $activeTabId}
				<FileList />
			{/key}
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
		overflow: hidden;
	}
</style>
