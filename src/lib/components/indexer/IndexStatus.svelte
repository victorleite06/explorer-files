<script lang="ts">
	import {
		indexStats,
		isIndexing,
		indexingProgress,
		reindexCurrentDirectory,
		clearSearchIndex
	} from '$lib/stores/contentSearch';
	import { formatIndexSize } from '$lib/utils/previewUtils';
	import { formatDate } from '$lib/utils/fileUtils';

	let total = $derived($indexStats?.total_documents ?? 0);
	let empty = $derived(total === 0);

	let pct = $derived.by(() => {
		const p = $indexingProgress;
		if (!p || p.total === 0) return 0;
		return Math.min(100, Math.round((p.processed / p.total) * 100));
	});
</script>

<div class="status">
	<div class="sec-head">Índice de Conteúdo</div>

	{#if $isIndexing}
		<div class="indexing">
			<svg class="spin" viewBox="0 0 24 24" width="12" height="12">
				<circle cx="12" cy="12" r="9" fill="none" stroke="#a78bfa" stroke-width="3"
					stroke-dasharray="42" stroke-linecap="round" />
			</svg>
			<span class="txt">Indexando… {pct}%</span>
			{#if $indexingProgress}
				<span class="sub">{$indexingProgress.processed}/{$indexingProgress.total}</span>
			{/if}
		</div>
	{:else if empty}
		<div class="info muted">Índice vazio — nenhum arquivo indexado</div>
	{:else}
		<div class="info">
			<span class="big">📚</span>
			<span>{total.toLocaleString('pt-BR')} arquivos indexados</span>
		</div>
		<div class="meta">
			Tamanho: {formatIndexSize($indexStats?.index_size_bytes ?? 0)}
			{#if $indexStats?.last_updated}
				· Atualizado: {formatDate($indexStats.last_updated)}
			{/if}
		</div>
	{/if}

	<div class="actions">
		<button class="reindex" disabled={$isIndexing} onclick={reindexCurrentDirectory}>
			⟳ {empty ? 'Indexar pasta atual' : 'Reindexar pasta atual'}
		</button>
		{#if total > 0 && !$isIndexing}
			<button class="clear" onclick={clearSearchIndex}>🗑️ Limpar índice</button>
		{/if}
	</div>
</div>

<style>
	.status {
		display: flex;
		flex-direction: column;
		gap: 8px;
		font-family: 'DM Sans', system-ui, sans-serif;
	}
	.sec-head {
		color: #444;
		font-size: 10px;
		text-transform: uppercase;
		letter-spacing: 0.06em;
	}
	.info {
		display: flex;
		align-items: center;
		gap: 8px;
		color: #c8c8dc;
		font-size: 13px;
	}
	.muted {
		color: #555;
		font-size: 12px;
	}
	.big {
		font-size: 16px;
	}
	.meta {
		color: #555;
		font-size: 11px;
	}
	.indexing {
		display: flex;
		align-items: center;
		gap: 8px;
		color: #a78bfa;
		font-size: 12px;
	}
	.spin {
		animation: spin 1s linear infinite;
	}
	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}
	.sub {
		color: #555;
		font-family: 'DM Mono', ui-monospace, monospace;
		font-size: 11px;
	}
	.actions {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
	}
	.reindex {
		padding: 6px 10px;
		border: 1px solid #a78bfa33;
		border-radius: 6px;
		background: #1e1e35;
		color: #a78bfa;
		font-family: inherit;
		font-size: 12px;
		cursor: pointer;
	}
	.reindex:disabled {
		opacity: 0.4;
		cursor: default;
	}
	.reindex:hover:not(:disabled) {
		background: #25253f;
	}
	.clear {
		padding: 6px 10px;
		border: 1px solid #f8717133;
		border-radius: 6px;
		background: none;
		color: #f87171;
		font-family: inherit;
		font-size: 12px;
		cursor: pointer;
	}
	.clear:hover {
		background: #2e1a1a;
	}
</style>
