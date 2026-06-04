<script lang="ts">
	import { tick } from 'svelte';
	import { DEFAULT_IGNORE_RULES } from '$lib/tauri';
	import {
		ignoreRules,
		patchIgnoreRules,
		toggleDotfiles,
		toggleNodeModules,
		toggleBuildArtifacts,
		addCustomHidden,
		removeCustomHidden,
		addCustomShown,
		removeCustomShown
	} from '$lib/stores/settings';

	const ALWAYS_HIDDEN = [
		'.DS_Store', 'Thumbs.db', 'desktop.ini', '.Spotlight-V100', '.Trashes',
		'$RECYCLE.BIN', 'System Volume Information', '.fseventsd', '.TemporaryItems'
	];

	// Estado dos inputs inline ('hidden' | 'shown' | null).
	let adding = $state<'hidden' | 'shown' | null>(null);
	let inputValue = $state('');
	let inputEl = $state<HTMLInputElement>();

	let warning = $derived.by(() => {
		const v = inputValue.trim();
		if (!v) return '';
		if (adding === 'hidden' && ALWAYS_HIDDEN.includes(v)) return 'Já está sempre oculto';
		if (adding === 'hidden' && $ignoreRules.custom_hidden.includes(v)) return 'Já está na lista';
		if (adding === 'shown' && $ignoreRules.custom_shown.includes(v)) return 'Já está na lista';
		if (adding === 'shown' && !v.startsWith('.')) return 'Não começa com . (ok se intencional)';
		return '';
	});

	let isDefault = $derived(
		!$ignoreRules.show_dotfiles &&
			!$ignoreRules.show_node_modules &&
			!$ignoreRules.show_build_artifacts &&
			$ignoreRules.custom_hidden.length === 0 &&
			$ignoreRules.custom_shown.length === 0
	);

	async function startAdd(list: 'hidden' | 'shown') {
		adding = list;
		inputValue = '';
		await tick();
		inputEl?.focus();
	}
	function cancelAdd() {
		adding = null;
		inputValue = '';
	}
	async function commitAdd() {
		const v = inputValue.trim();
		if (!v) {
			cancelAdd();
			return;
		}
		if (adding === 'hidden') await addCustomHidden(v);
		else if (adding === 'shown') await addCustomShown(v);
		cancelAdd();
	}
	function onInputKey(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			e.preventDefault();
			commitAdd();
		} else if (e.key === 'Escape') {
			e.preventDefault();
			cancelAdd();
		}
	}

	function restore() {
		patchIgnoreRules({ ...DEFAULT_IGNORE_RULES });
	}
</script>

<div class="editor">
	<!-- VISIBILIDADE -->
	<div class="sec-head">Visibilidade</div>

	<button class="toggle-row" onclick={toggleDotfiles}>
		<span class="pill" class:on={$ignoreRules.show_dotfiles}><span class="knob"></span></span>
		<span class="txt">
			<span class="t-title">Mostrar arquivos ocultos (dotfiles)</span>
			<span class="t-desc">Arquivos e pastas começando com ponto. Ex: .git, .env, .gitignore</span>
		</span>
	</button>

	<button class="toggle-row" onclick={toggleNodeModules}>
		<span class="pill" class:on={$ignoreRules.show_node_modules}><span class="knob"></span></span>
		<span class="txt">
			<span class="t-title">Mostrar node_modules</span>
			<span class="t-desc">Dependências JavaScript/TypeScript. Pode conter milhares de arquivos</span>
		</span>
	</button>

	<button class="toggle-row" onclick={toggleBuildArtifacts}>
		<span class="pill" class:on={$ignoreRules.show_build_artifacts}><span class="knob"></span></span>
		<span class="txt">
			<span class="t-title">Mostrar artefatos de build</span>
			<span class="t-desc">target, dist, build, .next, __pycache__…</span>
		</span>
	</button>

	<!-- SEMPRE OCULTAR -->
	<div class="sec-head">Sempre ocultar</div>
	<div class="chips">
		{#each $ignoreRules.custom_hidden as name (name)}
			<span class="chip">
				<span>{name}</span>
				<button class="x" onclick={() => removeCustomHidden(name)} aria-label="Remover">×</button>
			</span>
		{/each}
		{#if adding === 'hidden'}
			{@render addInput()}
		{:else}
			<button class="add" onclick={() => startAdd('hidden')}>+ adicionar</button>
		{/if}
	</div>

	<!-- SEMPRE MOSTRAR -->
	<div class="sec-head">Sempre mostrar</div>
	<div class="chips">
		{#each $ignoreRules.custom_shown as name (name)}
			<span class="chip">
				<span>{name}</span>
				<button class="x" onclick={() => removeCustomShown(name)} aria-label="Remover">×</button>
			</span>
		{/each}
		{#if adding === 'shown'}
			{@render addInput()}
		{:else}
			<button class="add" onclick={() => startAdd('shown')}>+ adicionar</button>
		{/if}
	</div>

	{#if !isDefault}
		<button class="restore" onclick={restore}>Restaurar padrões</button>
	{/if}
</div>

{#snippet addInput()}
	<span class="add-wrap">
		<input
			bind:this={inputEl}
			bind:value={inputValue}
			class="add-input"
			class:warn={warning}
			placeholder="nome…"
			spellcheck="false"
			onkeydown={onInputKey}
			onblur={commitAdd}
		/>
		{#if warning}<span class="warn-text">{warning}</span>{/if}
	</span>
{/snippet}

<style>
	.editor {
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
		margin-top: 6px;
	}

	/* Toggle */
	.toggle-row {
		display: flex;
		align-items: flex-start;
		gap: 10px;
		border: none;
		background: none;
		padding: 4px 0;
		text-align: left;
		cursor: pointer;
	}
	.pill {
		flex-shrink: 0;
		display: inline-flex;
		align-items: center;
		width: 32px;
		height: 18px;
		padding: 2px;
		border-radius: 9px;
		background: #1e1e35;
		transition: background 150ms ease;
	}
	.knob {
		width: 14px;
		height: 14px;
		border-radius: 50%;
		background: #444;
		transition:
			transform 150ms ease,
			background 150ms ease;
	}
	.pill.on {
		background: #a78bfa55;
	}
	.pill.on .knob {
		background: #a78bfa;
		transform: translateX(14px);
	}
	.txt {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}
	.t-title {
		color: #c8c8dc;
		font-size: 13px;
	}
	.t-desc {
		color: #444;
		font-size: 11px;
	}

	/* Chips */
	.chips {
		display: inline-flex;
		flex-wrap: wrap;
		gap: 6px;
	}
	.chip {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		padding: 3px 8px;
		border: 1px solid #1e1e35;
		border-radius: 12px;
		background: #111120;
		color: #888;
		font-size: 11px;
	}
	.x {
		border: none;
		background: none;
		color: #2a2a42;
		font-size: 12px;
		line-height: 1;
		cursor: pointer;
		padding: 0;
	}
	.x:hover {
		color: #f87171;
	}
	.add {
		border: 1px dashed #1e1e35;
		border-radius: 12px;
		background: none;
		color: #444;
		font-family: inherit;
		font-size: 11px;
		padding: 3px 8px;
		cursor: pointer;
	}
	.add:hover {
		color: #a78bfa;
		border-color: #a78bfa44;
	}
	.add-wrap {
		display: inline-flex;
		flex-direction: column;
		gap: 2px;
	}
	.add-input {
		width: 110px;
		height: 24px;
		padding: 0 8px;
		background: #111120;
		border: 1px solid #a78bfa55;
		border-radius: 12px;
		color: #c8c8dc;
		font-family: inherit;
		font-size: 11px;
		outline: none;
	}
	.add-input.warn {
		border-color: #f8a05a;
	}
	.warn-text {
		color: #f8a05a;
		font-size: 10px;
		padding-left: 4px;
	}

	.restore {
		align-self: flex-start;
		margin-top: 8px;
		border: 1px solid #1e1e35;
		border-radius: 6px;
		background: none;
		color: #f87171;
		font-family: inherit;
		font-size: 12px;
		padding: 6px 12px;
		cursor: pointer;
	}
	.restore:hover {
		background: #2e1a1a;
	}
</style>
