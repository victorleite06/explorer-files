<script lang="ts">
	import { get } from 'svelte/store';
	import { activeTabId, refreshTab } from '$lib/stores/tabs';
	import {
		ignoreRules,
		toggleDotfiles,
		toggleNodeModules,
		toggleBuildArtifacts
	} from '$lib/stores/settings';

	// Recarrega o diretório ativo quando as rules mudam (ignora 1º disparo).
	let initialized = false;
	$effect(() => {
		void $ignoreRules; // dependência
		if (!initialized) {
			initialized = true;
			return;
		}
		const id = get(activeTabId);
		if (id) refreshTab(id);
	});
</script>

<div class="vis">
	<button class="t" class:on={$ignoreRules.show_dotfiles} onclick={toggleDotfiles} title="Mostrar dotfiles">
		<span class="dot">·</span> dotfiles
	</button>
	<button class="t" class:on={$ignoreRules.show_node_modules} onclick={toggleNodeModules} title="Mostrar node_modules">
		node_modules
	</button>
	<button class="t" class:on={$ignoreRules.show_build_artifacts} onclick={toggleBuildArtifacts} title="Mostrar artefatos de build">
		⚙ build
	</button>
</div>

<style>
	.vis {
		display: flex;
		align-items: center;
		gap: 4px;
	}
	.t {
		padding: 4px 10px;
		border: 1px solid #1e1e35;
		border-radius: 5px;
		background: #111120;
		color: #444;
		font-family: 'DM Sans', system-ui, sans-serif;
		font-size: 11px;
		cursor: pointer;
		transition:
			background 120ms ease,
			border-color 120ms ease,
			color 120ms ease;
	}
	.t:hover {
		color: #888;
		border-color: #2a2a42;
	}
	.t.on {
		background: #1e1e35;
		border-color: #a78bfa44;
		color: #a78bfa;
	}
	.dot {
		font-weight: 700;
	}
</style>
