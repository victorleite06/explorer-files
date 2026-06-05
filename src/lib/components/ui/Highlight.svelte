<script lang="ts">
	import {
		buildSegmentsFromIndices,
		buildSegmentsFromTerms,
		parseDelimitedPreview,
		truncateWithHighlight,
		type HighlightSource,
		type HighlightOptions,
		type TextSegment
	} from '$lib/utils/highlightUtils';

	let {
		text = '',
		indices,
		terms,
		preview,
		source,
		maxChars,
		padding,
		highlightClass = 'hl',
		options
	}: {
		text?: string;
		indices?: number[];
		terms?: string[];
		preview?: string;
		source?: HighlightSource;
		maxChars?: number;
		padding?: number;
		highlightClass?: string;
		options?: Partial<HighlightOptions>;
	} = $props();

	let segments = $derived.by<TextSegment[]>(() => {
		let segs: TextSegment[];

		if (preview !== undefined) {
			segs = parseDelimitedPreview(preview, source ?? 'content');
		} else if (indices !== undefined && indices.length > 0) {
			segs = buildSegmentsFromIndices(text, indices, source ?? 'fuzzy');
		} else if (terms !== undefined && terms.length > 0) {
			segs = buildSegmentsFromTerms(text, terms, options, source ?? 'filter');
		} else {
			segs = [{ text, highlighted: false, source: 'filter' }];
		}

		if (maxChars !== undefined) {
			segs = truncateWithHighlight(segs, maxChars, padding ?? 20);
		}
		return segs;
	});
</script>

<span class="highlight-root">
	{#each segments as seg (seg)}
		{#if seg.isEllipsis}<span class="hl-ellipsis">{seg.text}</span>{:else if seg.highlighted}<mark
				class={highlightClass}>{seg.text}</mark>{:else}{seg.text}{/if}
	{/each}
</span>

<style>
	.highlight-root {
		white-space: pre-wrap;
		word-break: break-word;
	}
	:global(.hl) {
		background: transparent;
		color: #a78bfa;
		font-weight: 600;
		border-radius: 2px;
		padding: 0 1px;
	}
	:global(.hl-content) {
		background: #a78bfa18;
		color: #c4b5fd;
		font-weight: 500;
		border-radius: 2px;
		padding: 0 2px;
	}
	:global(.hl-filter) {
		background: #38bdf818;
		color: #38bdf8;
		font-weight: 500;
		border-radius: 2px;
		padding: 0 1px;
	}
</style>
