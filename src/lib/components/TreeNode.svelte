<script lang="ts">
	import type { TreeNode } from '$lib/tauri';
	import {
		currentPath,
		expandedPaths,
		navigateTo,
		expandNode,
		collapseNode
	} from '$lib/stores/explorer';
	import Self from './TreeNode.svelte';

	let { node, depth = 0 }: { node: TreeNode; depth?: number } = $props();

	let loading = $state(false);

	let isExpanded = $derived($expandedPaths.has(node.path));
	let isSelected = $derived(node.path === $currentPath);
	// Diretório: mostra seta se já sabemos que tem filhos, ou se ainda não
	// foi expandido (carga sob demanda — não dá pra saber antes).
	let hasArrow = $derived(node.children.length > 0 || !isExpanded);

	async function toggle(e: MouseEvent) {
		e.stopPropagation();
		if (isExpanded) {
			collapseNode(node.path);
			return;
		}
		loading = true;
		try {
			await expandNode(node.path);
		} finally {
			loading = false;
		}
	}

	function select() {
		navigateTo(node.path);
	}
</script>

<div class="node">
	<div
		class="row"
		class:selected={isSelected}
		style="padding-left: {8 + depth * 16}px"
		onclick={select}
		role="button"
		tabindex="0"
		onkeydown={(e) => e.key === 'Enter' && select()}
	>
		<span
			class="arrow"
			class:open={isExpanded}
			class:hidden={!hasArrow}
			onclick={toggle}
			role="button"
			tabindex="-1"
			onkeydown={(e) => e.key === 'Enter' && toggle(e as unknown as MouseEvent)}
		>
			▶
		</span>
		<span class="icon">{isExpanded ? '📂' : '📁'}</span>
		<span class="name">{node.name}</span>
		{#if loading}
			<span class="spinner"></span>
		{/if}
	</div>

	{#if isExpanded}
		<div class="children">
			{#each node.children as child (child.path)}
				<Self node={child} depth={depth + 1} />
			{/each}
		</div>
	{/if}
</div>

<style>
	.row {
		display: flex;
		align-items: center;
		height: 28px;
		gap: 4px;
		color: #9090a8;
		font-family: 'DM Sans', system-ui, sans-serif;
		font-size: 13px;
		cursor: pointer;
		user-select: none;
		border-left: 2px solid transparent;
		transition: background 150ms ease;
	}
	.row:hover {
		background: #16162a;
	}
	.row.selected {
		background: #1e1e35;
		border-left: 2px solid #a78bfa;
	}
	.arrow {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 14px;
		font-size: 9px;
		color: #6a6a82;
		transition: transform 150ms ease;
	}
	.arrow.open {
		transform: rotate(90deg);
	}
	.arrow.hidden {
		visibility: hidden;
	}
	.icon {
		font-size: 13px;
		transition: transform 150ms ease;
	}
	.name {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.spinner {
		width: 10px;
		height: 10px;
		margin-left: auto;
		margin-right: 8px;
		border: 1.5px solid #2a2a44;
		border-top-color: #a78bfa;
		border-radius: 50%;
		animation: spin 600ms linear infinite;
	}
	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}
</style>
