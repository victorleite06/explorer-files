<script lang="ts">
	import type { FilterState } from '$lib/utils/filterUtils';

	let {
		onlyDirs,
		onlyFiles,
		onChange,
		onConfigureVisibility = () => {}
	}: {
		onlyDirs: boolean;
		onlyFiles: boolean;
		onChange: (patch: Partial<FilterState>) => void;
		onConfigureVisibility?: () => void;
	} = $props();

	let mode = $derived(onlyDirs ? 'dirs' : onlyFiles ? 'files' : 'all');
</script>

<div class="typef">
	<div class="title">Tipo</div>
	<div class="chips">
		<button class="chip" class:sel={mode === 'all'} onclick={() => onChange({ onlyDirs: false, onlyFiles: false })}>Todos</button>
		<button class="chip" class:sel={mode === 'dirs'} onclick={() => onChange({ onlyDirs: true, onlyFiles: false })}>Pastas</button>
		<button class="chip" class:sel={mode === 'files'} onclick={() => onChange({ onlyDirs: false, onlyFiles: true })}>Arquivos</button>
	</div>

	<button class="config-link" onclick={onConfigureVisibility}>Configurar visibilidade →</button>
</div>

<style>
	.typef {
		display: flex;
		flex-direction: column;
		gap: 8px;
		font-family: 'DM Sans', system-ui, sans-serif;
	}
	.title {
		color: #444;
		font-size: 10px;
		text-transform: uppercase;
		letter-spacing: 0.06em;
	}
	.chips {
		display: inline-flex;
		gap: 6px;
	}
	.chip {
		padding: 3px 12px;
		border: 1px solid #1e1e35;
		border-radius: 12px;
		background: #111120;
		color: #666;
		font-family: inherit;
		font-size: 11px;
		cursor: pointer;
		transition: all 100ms ease;
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
	.config-link {
		align-self: flex-start;
		border: none;
		background: none;
		color: #555;
		font-family: inherit;
		font-size: 11px;
		cursor: pointer;
		padding: 0;
	}
	.config-link:hover {
		color: #a78bfa;
	}
</style>
