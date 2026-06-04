<script lang="ts">
	import type { DirectorySummary } from '$lib/tauri';

	let {
		selected,
		summary,
		onChange,
		excluded = [],
		onExcludeChange = () => {}
	}: {
		selected: string[];
		summary: DirectorySummary | null;
		onChange: (extensions: string[]) => void;
		excluded?: string[];
		onExcludeChange?: (extensions: string[]) => void;
	} = $props();

	const QUICK: Record<string, string[]> = {
		'Selecionar imagens': ['jpg', 'jpeg', 'png', 'gif', 'webp', 'svg', 'ico'],
		'Selecionar código': ['rs', 'ts', 'js', 'tsx', 'jsx', 'svelte', 'py', 'go', 'c', 'cpp'],
		'Selecionar docs': ['pdf', 'doc', 'docx', 'odt', 'md', 'txt'],
		'Selecionar vídeos': ['mp4', 'mkv', 'avi', 'mov', 'webm']
	};

	let query = $state('');

	interface Avail {
		ext: string;
		count?: number;
	}
	let available = $derived.by<Avail[]>(() => {
		if (summary) {
			return summary.extensions
				.filter((e) => e.extension !== '__no_ext__')
				.map((e) => ({ ext: e.extension, count: e.count }));
		}
		// Sem summary: mostra apenas o que já está selecionado/excluído.
		return [...new Set([...selected, ...excluded])].map((ext) => ({ ext }));
	});

	let filtered = $derived(
		query.trim()
			? available.filter((a) => a.ext.toLowerCase().includes(query.trim().toLowerCase()))
			: available
	);

	let grouped = $derived(!!summary && !query.trim());
	let top = $derived(grouped ? filtered.slice(0, 8) : filtered);
	let others = $derived(grouped ? filtered.slice(8) : []);

	function chipState(ext: string): 'sel' | 'exc' | 'none' {
		if (excluded.includes(ext)) return 'exc';
		if (selected.includes(ext)) return 'sel';
		return 'none';
	}

	function toggle(ext: string, shift: boolean) {
		if (shift) {
			onExcludeChange(
				excluded.includes(ext) ? excluded.filter((x) => x !== ext) : [...excluded, ext]
			);
		} else {
			onChange(selected.includes(ext) ? selected.filter((x) => x !== ext) : [...selected, ext]);
		}
	}

	function onSearchKey(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			const q = query.trim().toLowerCase().replace(/^\./, '');
			if (q && !selected.includes(q)) onChange([...selected, q]);
			query = '';
		}
	}
</script>

<div class="extf">
	<input
		class="search"
		type="text"
		placeholder="Buscar extensão…"
		bind:value={query}
		onkeydown={onSearchKey}
		spellcheck="false"
	/>

	{#if grouped}
		{#if top.length}
			<div class="grp-label">Mais usadas</div>
			<div class="chips">
				{#each top as a (a.ext)}
					{@render chip(a)}
				{/each}
			</div>
		{/if}
		{#if others.length}
			<div class="grp-label">Outras</div>
			<div class="chips">
				{#each others as a (a.ext)}
					{@render chip(a)}
				{/each}
			</div>
		{/if}
	{:else}
		<div class="chips">
			{#each filtered as a (a.ext)}
				{@render chip(a)}
			{/each}
		</div>
		{#if filtered.length === 0}
			<div class="hint">Enter para adicionar "{query}"</div>
		{/if}
	{/if}

	<div class="quick">
		{#each Object.entries(QUICK) as [label, exts] (label)}
			<button class="q" onclick={() => onChange(exts)}>{label}</button>
		{/each}
		<button class="q" onclick={() => onChange([])}>Limpar seleção</button>
	</div>
</div>

{#snippet chip(a: Avail)}
	<button
		class="chip {chipState(a.ext)}"
		onclick={(e) => toggle(a.ext, e.shiftKey)}
		title="Clique: incluir · Shift+clique: excluir"
	>
		<span class="ext">{a.ext}</span>
		{#if a.count !== undefined}<span class="cnt">{a.count}</span>{/if}
	</button>
{/snippet}

<style>
	.extf {
		display: flex;
		flex-direction: column;
		gap: 8px;
		font-family: 'DM Sans', system-ui, sans-serif;
	}
	.search {
		width: 100%;
		height: 28px;
		padding: 0 10px;
		background: #111120;
		border: 1px solid #1e1e35;
		border-radius: 6px;
		color: #c8c8dc;
		font-family: inherit;
		font-size: 12px;
		outline: none;
	}
	.search:focus {
		border-color: #a78bfa;
	}
	.grp-label {
		color: #444;
		font-size: 10px;
		text-transform: uppercase;
		letter-spacing: 0.06em;
		margin-top: 4px;
	}
	.chips {
		display: inline-flex;
		flex-wrap: wrap;
		gap: 6px;
	}
	.chip {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		padding: 3px 10px;
		border: 1px solid #1e1e35;
		border-radius: 12px;
		background: #111120;
		color: #666;
		font-family: inherit;
		font-size: 11px;
		cursor: pointer;
		transition:
			border-color 100ms ease,
			color 100ms ease,
			background 100ms ease;
	}
	.chip:hover {
		border-color: #2a2a42;
		color: #888;
	}
	.chip.sel {
		background: #1e1e35;
		border-color: #a78bfa55;
		color: #a78bfa;
	}
	.chip.exc {
		border-color: #f8717155;
		color: #f87171;
	}
	.chip.exc .ext {
		text-decoration: line-through;
	}
	.cnt {
		color: #444;
		font-size: 10px;
	}
	.chip.sel .cnt {
		color: #a78bfa55;
	}
	.quick {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
		margin-top: 4px;
	}
	.q {
		border: none;
		background: none;
		color: #444;
		font-family: inherit;
		font-size: 10px;
		cursor: pointer;
		padding: 0;
	}
	.q:hover {
		color: #a78bfa;
	}
	.hint {
		color: #444;
		font-size: 11px;
	}
</style>
