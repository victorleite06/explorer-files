<script lang="ts">
	import { fly, fade } from 'svelte/transition';
	import { get } from 'svelte/store';
	import { activeTab, activeTabId, updateTabFilter, resetTabFilter } from '$lib/stores/tabs';
	import { DEFAULT_FILTER, isFilterActive, type FilterState } from '$lib/utils/filterUtils';
	import TypeFilter from './TypeFilter.svelte';
	import ExtensionFilter from './ExtensionFilter.svelte';
	import SizeFilter from './SizeFilter.svelte';
	import DateFilter from './DateFilter.svelte';
	import IgnoreRulesEditor from '../settings/IgnoreRulesEditor.svelte';
	import GitignoreEditor from '../settings/GitignoreEditor.svelte';
	import IndexStatus from '../indexer/IndexStatus.svelte';

	let { isOpen, onClose }: { isOpen: boolean; onClose: () => void } = $props();

	let filter = $derived($activeTab?.filter ?? DEFAULT_FILTER);
	let summary = $derived($activeTab?.directorySummary ?? null);
	let active = $derived(isFilterActive(filter));

	function patch(p: Partial<FilterState>) {
		const id = get(activeTabId);
		if (id) updateTabFilter(id, p);
	}
	function reset() {
		const id = get(activeTabId);
		if (id) resetTabFilter(id);
	}
</script>

{#if isOpen}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<div class="backdrop" role="button" tabindex="-1" aria-label="Fechar filtros"
		transition:fade={{ duration: 150 }} onclick={onClose}></div>

	<aside class="panel" transition:fly={{ x: 280, duration: 200 }}>
		<div class="head">
			<span class="title">FILTROS</span>
			<button class="close" onclick={onClose} aria-label="Fechar">×</button>
		</div>

		<div class="body">
			<section>
				<TypeFilter onlyDirs={filter.onlyDirs} onlyFiles={filter.onlyFiles} onChange={patch} />
			</section>

			<div class="divider"></div>

			<section>
				<div class="sec-title">Extensão</div>
				<ExtensionFilter
					selected={filter.extensions}
					excluded={filter.excludeExtensions}
					{summary}
					onChange={(ext) => patch({ extensions: ext })}
					onExcludeChange={(ext) => patch({ excludeExtensions: ext })}
				/>
			</section>

			<div class="divider"></div>

			<section>
				<div class="sec-title">Tamanho</div>
				<SizeFilter sizeRange={filter.sizeRange} {summary} onChange={(r) => patch({ sizeRange: r })} />
			</section>

			<div class="divider"></div>

			<section>
				<div class="sec-title">Data</div>
				<DateFilter dateRange={filter.dateRange} onChange={(r) => patch({ dateRange: r })} />
			</section>

			<div class="divider"></div>

			<section>
				<IgnoreRulesEditor />
			</section>

			<div class="divider"></div>

			<section>
				<GitignoreEditor />
			</section>

			<div class="divider"></div>

			<section>
				<IndexStatus />
			</section>
		</div>

		{#if active}
			<div class="footer">
				<button class="clear" onclick={reset}>Limpar todos os filtros</button>
			</div>
		{/if}
	</aside>
{/if}

<style>
	.backdrop {
		position: absolute;
		inset: 0;
		z-index: 99;
		background: #00000044;
	}
	.panel {
		position: absolute;
		top: 0;
		right: 0;
		z-index: 100;
		display: flex;
		flex-direction: column;
		width: 280px;
		height: 100%;
		background: #0e0e1c;
		border-left: 1px solid #1e1e35;
		box-shadow: -8px 0 24px #00000044;
		font-family: 'DM Sans', system-ui, sans-serif;
	}
	.head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 12px 16px;
		border-bottom: 1px solid #1e1e35;
		flex-shrink: 0;
	}
	.title {
		font-family: 'Syne', 'DM Sans', system-ui, sans-serif;
		font-size: 13px;
		font-weight: 700;
		letter-spacing: 0.06em;
		color: #dddde8;
	}
	.close {
		width: 24px;
		height: 24px;
		border: none;
		border-radius: 5px;
		background: none;
		color: #666;
		font-size: 16px;
		line-height: 1;
		cursor: pointer;
	}
	.close:hover {
		background: #1a1a2e;
		color: #c8c8dc;
	}
	.body {
		flex: 1;
		overflow-y: auto;
		padding: 12px 16px;
		display: flex;
		flex-direction: column;
		gap: 12px;
	}
	.sec-title {
		color: #444;
		font-size: 10px;
		text-transform: uppercase;
		letter-spacing: 0.06em;
		margin-bottom: 8px;
	}
	.divider {
		height: 1px;
		background: #1a1a2e;
	}
	.footer {
		padding: 10px 16px;
		border-top: 1px solid #1e1e35;
		flex-shrink: 0;
	}
	.clear {
		width: 100%;
		padding: 8px;
		border: 1px solid #1e1e35;
		border-radius: 6px;
		background: none;
		color: #f87171;
		font-family: inherit;
		font-size: 12px;
		cursor: pointer;
		transition: background 120ms ease;
	}
	.clear:hover {
		background: #2e1a1a;
	}
</style>
