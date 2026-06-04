<script lang="ts">
	import { onMount } from 'svelte';

	let { onSelect, onClose }: { onSelect: (emoji: string) => void; onClose: () => void } = $props();

	interface Category {
		name: string;
		emojis: string[];
	}
	const CATEGORIES: Category[] = [
		{ name: 'Pastas', emojis: ['📁', '📂', '🗂️', '🗃️', '📦', '📥', '📤'] },
		{ name: 'Trabalho', emojis: ['💼', '📋', '📌', '📍', '🔖', '🏷️', '📎'] },
		{ name: 'Código', emojis: ['💻', '🖥️', '⌨️', '🖱️', '💾', '💿', '🖨️'] },
		{ name: 'Projetos', emojis: ['🚀', '⚡', '🔥', '✨', '🎯', '🏆', '🎨'] },
		{ name: 'Misc', emojis: ['❤️', '⭐', '🌟', '💡', '🔑', '🔒', '🏠'] }
	];

	// Nome de busca por emoji (pt-BR, simplificado).
	const NAMES: Record<string, string> = {
		'📁': 'pasta', '📂': 'pasta aberta', '🗂️': 'organizador', '🗃️': 'arquivo', '📦': 'caixa pacote',
		'📥': 'entrada', '📤': 'saida', '💼': 'maleta trabalho', '📋': 'prancheta', '📌': 'alfinete',
		'📍': 'local', '🔖': 'marcador', '🏷️': 'etiqueta tag', '📎': 'clipe', '💻': 'laptop codigo',
		'🖥️': 'monitor', '⌨️': 'teclado', '🖱️': 'mouse', '💾': 'disquete salvar', '💿': 'disco cd',
		'🖨️': 'impressora', '🚀': 'foguete', '⚡': 'raio energia', '🔥': 'fogo', '✨': 'brilho',
		'🎯': 'alvo', '🏆': 'trofeu', '🎨': 'arte paleta', '❤️': 'coracao', '⭐': 'estrela',
		'🌟': 'estrela brilho', '💡': 'ideia luz', '🔑': 'chave', '🔒': 'cadeado', '🏠': 'casa home'
	};

	let query = $state('');

	let filtered = $derived.by<Category[]>(() => {
		const q = query.trim().toLowerCase();
		if (!q) return CATEGORIES;
		return CATEGORIES.map((c) => ({
			name: c.name,
			emojis: c.emojis.filter((e) => (NAMES[e] ?? '').includes(q))
		})).filter((c) => c.emojis.length > 0);
	});

	function pick(emoji: string) {
		onSelect(emoji);
		onClose();
	}

	function onKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') onClose();
	}
	function onDocClick() {
		onClose();
	}

	onMount(() => {
		// Adia 1 tick p/ não capturar o próprio clique que abriu o picker.
		const id = setTimeout(() => document.addEventListener('click', onDocClick), 0);
		window.addEventListener('keydown', onKeydown);
		return () => {
			clearTimeout(id);
			document.removeEventListener('click', onDocClick);
			window.removeEventListener('keydown', onKeydown);
		};
	});
</script>

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div class="picker" role="dialog" tabindex="-1" onclick={(e) => e.stopPropagation()}>
	<input class="search" type="text" placeholder="Buscar…" bind:value={query} spellcheck="false" />
	<div class="cats">
		{#each filtered as cat (cat.name)}
			<div class="cat">
				<div class="cat-name">{cat.name}</div>
				<div class="grid">
					{#each cat.emojis as emoji (emoji)}
						<button class="emoji" onclick={() => pick(emoji)} title={NAMES[emoji] ?? ''}>{emoji}</button>
					{/each}
				</div>
			</div>
		{/each}
		{#if filtered.length === 0}
			<div class="empty">Nenhum emoji</div>
		{/if}
	</div>
</div>

<style>
	.picker {
		position: absolute;
		z-index: 1000;
		width: 220px;
		padding: 8px;
		background: #111120;
		border: 1px solid #1e1e35;
		border-radius: 8px;
		box-shadow: 0 8px 32px #00000066;
		font-family: 'DM Sans', system-ui, sans-serif;
	}
	.search {
		width: 100%;
		height: 28px;
		margin-bottom: 8px;
		padding: 0 8px;
		background: #0a0a12;
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
	.cats {
		max-height: 240px;
		overflow-y: auto;
	}
	.cat-name {
		color: #444;
		font-size: 10px;
		text-transform: uppercase;
		letter-spacing: 0.06em;
		margin: 6px 2px 4px;
	}
	.grid {
		display: grid;
		grid-template-columns: repeat(7, 1fr);
		gap: 2px;
	}
	.emoji {
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 4px;
		border: none;
		border-radius: 4px;
		background: none;
		font-size: 20px;
		cursor: pointer;
		transition: background 100ms ease;
	}
	.emoji:hover {
		background: #1a1a2e;
	}
	.empty {
		padding: 12px;
		text-align: center;
		color: #444;
		font-size: 12px;
	}
</style>
