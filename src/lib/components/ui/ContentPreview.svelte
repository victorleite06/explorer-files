<script lang="ts">
	import Highlight from './Highlight.svelte';

	let {
		previews,
		maxPreviews = 2,
		previewMaxChars = 180,
		font = 'mono'
	}: {
		previews: string[];
		maxPreviews?: number;
		previewMaxChars?: number;
		font?: 'mono' | 'sans';
	} = $props();

	let shown = $derived(previews.slice(0, maxPreviews));
</script>

<div class="previews" class:sans={font === 'sans'}>
	{#each shown as pv (pv)}
		<div class="pv">
			<span class="ell">…</span>
			<Highlight preview={pv} maxChars={previewMaxChars} highlightClass="hl-content" />
			<span class="ell">…</span>
		</div>
	{/each}
</div>

<style>
	.previews {
		display: flex;
		flex-direction: column;
		font-family: 'DM Mono', ui-monospace, monospace;
		font-size: 11px;
		color: #666;
	}
	.previews.sans {
		font-family: 'DM Sans', system-ui, sans-serif;
		font-size: 12px;
	}
	.pv {
		padding: 6px 10px;
		margin-bottom: 4px;
		background: #0a0a12;
		border-radius: 4px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.ell {
		color: #2a2a42;
	}
	.pv :global(.hl-content) {
		background: #a78bfa22;
		color: #a78bfa;
		border-radius: 2px;
		padding: 0 2px;
	}
</style>
