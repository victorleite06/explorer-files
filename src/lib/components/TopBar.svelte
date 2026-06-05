<script lang="ts">
	import { get } from 'svelte/store';
	import { activeTab, activeTabId, tabGoBack, tabGoForward, navigateTab } from '$lib/stores/tabs';
	import NavButtons from './NavButtons.svelte';
	import Breadcrumb from './Breadcrumb.svelte';
	import SearchInput from './search/SearchInput.svelte';

	let breadcrumbRef = $state<ReturnType<typeof Breadcrumb>>();

	let canGoBack = $derived(($activeTab?.historyIndex ?? 0) > 0);
	let canGoForward = $derived(
		$activeTab ? $activeTab.historyIndex < ($activeTab.history.length ?? 0) - 1 : false
	);

	function onBack() {
		const id = get(activeTabId);
		if (id) tabGoBack(id);
	}

	function onForward() {
		const id = get(activeTabId);
		if (id) tabGoForward(id);
	}

	function onNavigate(p: string) {
		const id = get(activeTabId);
		if (id) navigateTab(id, p);
	}

	// Ao trocar de tab: se o Breadcrumb estava editando, cancela.
	$effect(() => {
		void $activeTabId;
		breadcrumbRef?.cancelEdit();
	});
</script>

<div class="topbar">
	<NavButtons {canGoBack} {canGoForward} {onBack} {onForward} />

	<div class="crumb-area">
		<Breadcrumb bind:this={breadcrumbRef} path={$activeTab?.path ?? ''} {onNavigate} />
	</div>

	<SearchInput />

	<button class="opts" title="Opções" aria-label="Opções">···</button>
</div>

<style>
	.topbar {
		display: flex;
		align-items: center;
		gap: 8px;
		height: 44px;
		padding: 0 12px;
		background: #0c0c18;
		border-bottom: 1px solid #12121e;
		flex-shrink: 0;
	}
	.crumb-area {
		display: flex;
		align-items: center;
		flex: 1;
		min-width: 0;
	}
	.opts {
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
		font-size: 14px;
		line-height: 1;
		cursor: pointer;
		transition:
			background 150ms ease,
			border-color 150ms ease;
	}
	.opts:hover {
		background: #1a1a2e;
		border-color: #2a2a42;
		color: #c8c8dc;
	}
</style>
