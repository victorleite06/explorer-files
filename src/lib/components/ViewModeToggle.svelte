<script lang="ts">
	import { onMount } from 'svelte';

	type ViewMode = 'list' | 'grid' | 'columns';
	type IconSize = 'small' | 'medium' | 'large';

	let {
		viewMode,
		onChange,
		iconSize,
		onIconSizeChange
	}: {
		viewMode: ViewMode;
		onChange: (mode: ViewMode) => void;
		iconSize: IconSize;
		onIconSizeChange: (size: IconSize) => void;
	} = $props();

	const SIZES: IconSize[] = ['small', 'medium', 'large'];
	const SIZE_LABEL: Record<IconSize, string> = {
		small: 'Pequeno',
		medium: 'Médio',
		large: 'Grande'
	};

	let sizeIndex = $derived(SIZES.indexOf(iconSize));

	function onSlider(e: Event) {
		const v = Number((e.target as HTMLInputElement).value);
		onIconSizeChange(SIZES[v] ?? 'medium');
	}

	function onKeydown(e: KeyboardEvent) {
		if (!e.ctrlKey) return;
		if (e.key === '1') {
			e.preventDefault();
			onChange('list');
		} else if (e.key === '2') {
			e.preventDefault();
			onChange('grid');
		} else if (e.key === '3') {
			e.preventDefault();
			onChange('columns');
		}
	}

	onMount(() => {
		window.addEventListener('keydown', onKeydown);
		return () => window.removeEventListener('keydown', onKeydown);
	});
</script>

<div class="toggle">
	<div class="group">
		<button
			class="btn"
			class:active={viewMode === 'list'}
			title="Lista (Ctrl+1)"
			aria-label="Lista"
			onclick={() => onChange('list')}
		>
			<svg viewBox="0 0 24 24" width="15" height="15">
				<g stroke="currentColor" stroke-width="2" stroke-linecap="round">
					<line x1="4" y1="7" x2="20" y2="7" />
					<line x1="4" y1="12" x2="20" y2="12" />
					<line x1="4" y1="17" x2="20" y2="17" />
				</g>
			</svg>
		</button>

		<button
			class="btn"
			class:active={viewMode === 'grid'}
			title="Grade (Ctrl+2)"
			aria-label="Grade"
			onclick={() => onChange('grid')}
		>
			<svg viewBox="0 0 24 24" width="15" height="15">
				<g fill="none" stroke="currentColor" stroke-width="2">
					<rect x="4" y="4" width="7" height="7" rx="1" />
					<rect x="13" y="4" width="7" height="7" rx="1" />
					<rect x="4" y="13" width="7" height="7" rx="1" />
					<rect x="13" y="13" width="7" height="7" rx="1" />
				</g>
			</svg>
		</button>

		<button
			class="btn"
			class:active={viewMode === 'columns'}
			title="Colunas (Ctrl+3)"
			aria-label="Colunas"
			onclick={() => onChange('columns')}
		>
			<svg viewBox="0 0 24 24" width="15" height="15">
				<g fill="none" stroke="currentColor" stroke-width="2">
					<rect x="4" y="4" width="4" height="16" rx="1" />
					<rect x="10" y="4" width="4" height="16" rx="1" />
					<rect x="16" y="4" width="4" height="16" rx="1" />
				</g>
			</svg>
		</button>
	</div>

	{#if viewMode === 'grid'}
		<div class="size">
			<input
				type="range"
				min="0"
				max="2"
				step="1"
				value={sizeIndex}
				oninput={onSlider}
				aria-label="Tamanho dos ícones"
				title={SIZE_LABEL[iconSize]}
			/>
			<span class="size-label">{SIZE_LABEL[iconSize]}</span>
		</div>
	{/if}
</div>

<style>
	.toggle {
		display: flex;
		align-items: center;
		gap: 10px;
	}
	.group {
		display: flex;
		border-radius: 6px;
		overflow: hidden;
	}
	.btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		border: none;
		border-right: 1px solid #1e1e35;
		background: #111120;
		color: #444;
		cursor: pointer;
		transition:
			background 120ms ease,
			color 120ms ease;
	}
	.btn:last-child {
		border-right: none;
	}
	.btn:hover {
		color: #888;
	}
	.btn.active {
		background: #1e1e35;
		border: 1px solid #a78bfa44;
		color: #a78bfa;
	}

	.size {
		display: flex;
		align-items: center;
		gap: 8px;
		animation: fade 150ms ease;
	}
	@keyframes fade {
		from {
			opacity: 0;
			transform: translateX(-4px);
		}
		to {
			opacity: 1;
			transform: none;
		}
	}
	.size-label {
		color: #555;
		font-family: 'DM Sans', system-ui, sans-serif;
		font-size: 11px;
		min-width: 48px;
	}

	/* Range estilizado */
	input[type='range'] {
		-webkit-appearance: none;
		appearance: none;
		width: 80px;
		height: 3px;
		background: #1e1e35;
		border-radius: 2px;
		outline: none;
		cursor: pointer;
	}
	input[type='range']::-webkit-slider-thumb {
		-webkit-appearance: none;
		appearance: none;
		width: 12px;
		height: 12px;
		background: #a78bfa;
		border-radius: 50%;
		cursor: pointer;
	}
	input[type='range']::-moz-range-thumb {
		width: 12px;
		height: 12px;
		background: #a78bfa;
		border: none;
		border-radius: 50%;
		cursor: pointer;
	}
</style>
