<script lang="ts">
	import { slide } from 'svelte/transition';
	import { indexingProgress } from '$lib/stores/contentSearch';
	import { formatEta } from '$lib/utils/previewUtils';

	let dismissed = $state(false);

	let pct = $derived.by(() => {
		const p = $indexingProgress;
		if (!p || p.total === 0) return 0;
		return Math.min(100, Math.round((p.processed / p.total) * 100));
	});
</script>

{#if !dismissed && $indexingProgress}
	<div class="bar" transition:slide={{ duration: 200 }}>
		<svg class="spin" viewBox="0 0 24 24" width="13" height="13">
			<circle cx="12" cy="12" r="9" fill="none" stroke="#a78bfa" stroke-width="3"
				stroke-dasharray="42" stroke-linecap="round" />
		</svg>
		<span class="status">Indexando conteúdo…</span>
		<span class="file">{$indexingProgress.current_file}</span>

		<div class="track"><div class="fill" style="width: {pct}%"></div></div>
		<span class="pct">{pct}%</span>
		<span class="eta">{formatEta($indexingProgress.eta_seconds)}</span>

		<button class="close" onclick={() => (dismissed = true)} aria-label="Ocultar" title="Ocultar (não cancela)">×</button>
	</div>
{/if}

<style>
	.bar {
		display: flex;
		align-items: center;
		gap: 10px;
		height: 28px;
		padding: 0 12px;
		background: #0e0e1c;
		border-bottom: 1px solid #1e1e35;
		font-family: 'DM Sans', system-ui, sans-serif;
		flex-shrink: 0;
	}
	.spin {
		animation: spin 1s linear infinite;
		flex-shrink: 0;
	}
	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}
	.status {
		color: #666;
		font-size: 11px;
		flex-shrink: 0;
	}
	.file {
		color: #888;
		font-size: 12px;
		max-width: 200px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.track {
		width: 120px;
		height: 4px;
		background: #1e1e35;
		border-radius: 2px;
		overflow: hidden;
		flex-shrink: 0;
		margin-left: auto;
	}
	.fill {
		height: 100%;
		background: linear-gradient(90deg, #7c3aed, #a78bfa);
		transition: width 300ms ease;
	}
	.pct {
		color: #a78bfa;
		font-family: 'DM Mono', ui-monospace, monospace;
		font-size: 11px;
		flex-shrink: 0;
	}
	.eta {
		color: #444;
		font-size: 11px;
		flex-shrink: 0;
	}
	.close {
		width: 20px;
		height: 20px;
		border: none;
		border-radius: 4px;
		background: none;
		color: #444;
		font-size: 14px;
		line-height: 1;
		cursor: pointer;
		flex-shrink: 0;
	}
	.close:hover {
		background: #1a1a2e;
		color: #c8c8dc;
	}
</style>
