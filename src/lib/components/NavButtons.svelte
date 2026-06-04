<script lang="ts">
	import { onMount } from 'svelte';

	let {
		canGoBack,
		canGoForward,
		onBack,
		onForward
	}: {
		canGoBack: boolean;
		canGoForward: boolean;
		onBack: () => void;
		onForward: () => void;
	} = $props();

	function onKeydown(e: KeyboardEvent) {
		// Alt + setas → histórico.
		if (e.altKey && e.key === 'ArrowLeft') {
			e.preventDefault();
			if (canGoBack) onBack();
			return;
		}
		if (e.altKey && e.key === 'ArrowRight') {
			e.preventDefault();
			if (canGoForward) onForward();
			return;
		}
		// F4 ou Ctrl+L → pede edição do breadcrumb.
		if (e.key === 'F4' || (e.ctrlKey && e.key.toLowerCase() === 'l')) {
			e.preventDefault();
			window.dispatchEvent(new CustomEvent('breadcrumb:edit'));
		}
	}

	onMount(() => {
		window.addEventListener('keydown', onKeydown);
		return () => window.removeEventListener('keydown', onKeydown);
	});
</script>

<div class="nav">
	<button
		class="navbtn"
		data-tip="Voltar (Alt + ←)"
		disabled={!canGoBack}
		onclick={onBack}
		aria-label="Voltar"
	>
		<svg viewBox="0 0 24 24" width="16" height="16">
			<path d="M15 18l-6-6 6-6" stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round" />
		</svg>
	</button>

	<button
		class="navbtn"
		data-tip="Avançar (Alt + →)"
		disabled={!canGoForward}
		onclick={onForward}
		aria-label="Avançar"
	>
		<svg viewBox="0 0 24 24" width="16" height="16">
			<path d="M9 18l6-6-6-6" stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round" />
		</svg>
	</button>
</div>

<style>
	.nav {
		display: flex;
		gap: 4px;
	}
	.navbtn {
		position: relative;
		display: flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		border: 1px solid #1e1e35;
		border-radius: 6px;
		background: #111120;
		color: #9090a8;
		cursor: pointer;
		transition:
			background 150ms ease,
			border-color 150ms ease,
			transform 80ms ease;
	}
	.navbtn:hover:not(:disabled) {
		background: #1a1a2e;
		border-color: #2a2a42;
		color: #c8c8dc;
	}
	.navbtn:active:not(:disabled) {
		transform: scale(0.93);
	}
	.navbtn:disabled {
		opacity: 0.25;
		cursor: not-allowed;
	}

	/* Tooltip simples via ::after */
	.navbtn::after {
		content: attr(data-tip);
		position: absolute;
		top: calc(100% + 6px);
		left: 50%;
		transform: translateX(-50%);
		white-space: nowrap;
		padding: 4px 8px;
		background: #14142a;
		border: 1px solid #1e1e35;
		border-radius: 5px;
		color: #c8c8dc;
		font-family: 'DM Sans', system-ui, sans-serif;
		font-size: 11px;
		opacity: 0;
		pointer-events: none;
		transition: opacity 150ms ease;
		z-index: 100;
	}
	.navbtn:hover:not(:disabled)::after {
		opacity: 1;
	}
</style>
