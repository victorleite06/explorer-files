<script lang="ts">
	import {
		gitignoreSettings,
		toggleRespectGitignore,
		toggleGlobalGitignore,
		toggleIgnoreFiles
	} from '$lib/stores/settings';
	import {
		currentGitignoreInfo,
		gitignoreInfoLoading,
		isGitRepo
	} from '$lib/stores/gitignoreInfo';

	let g = $derived($gitignoreSettings);
	let info = $derived($currentGitignoreInfo);
	let mainOff = $derived(!g.respect_gitignore);

	function shortenPath(path: string, maxLen = 40): string {
		if (path.length <= maxLen) return path;
		return '...' + path.slice(path.length - maxLen);
	}

	function fmtCount(n: number): string {
		return n.toLocaleString('pt-BR');
	}
</script>

<div class="editor">
	<div class="sec-head">Gitignore</div>

	{#if $isGitRepo}
		<div class="badge repo">
			<span class="hex">⬡</span>
			<span>
				Repositório git detectado
				{#if info && info.gitignore_files.length}
					<span class="dim">· {info.gitignore_files.length} arquivos .gitignore</span>
				{/if}
			</span>
		</div>
	{:else}
		<div class="badge norepo">
			<span class="hex">○</span>
			<span>Não é um repositório git — .gitignore não tem efeito aqui</span>
		</div>
	{/if}

	<!-- Toggle principal -->
	<button class="toggle-row" disabled={!$isGitRepo} onclick={toggleRespectGitignore}>
		<span class="pill" class:on={g.respect_gitignore} class:dis={!$isGitRepo}><span class="knob"></span></span>
		<span class="txt">
			<span class="t-title">Respeitar .gitignore</span>
			<span class="t-desc">Exclui arquivos do .gitignore da listagem, busca e índice</span>
		</span>
	</button>

	<button
		class="toggle-row"
		disabled={mainOff || !$isGitRepo}
		title={mainOff ? 'Ative o .gitignore primeiro' : ''}
		onclick={toggleGlobalGitignore}
	>
		<span class="pill" class:on={g.respect_global_gitignore} class:dis={mainOff || !$isGitRepo}><span class="knob"></span></span>
		<span class="txt">
			<span class="t-title">Gitignore global do sistema</span>
			<span class="t-desc">~/.config/git/ignore ou ~/.gitignore_global</span>
		</span>
	</button>

	<button
		class="toggle-row"
		disabled={mainOff || !$isGitRepo}
		title={mainOff ? 'Ative o .gitignore primeiro' : ''}
		onclick={toggleIgnoreFiles}
	>
		<span class="pill" class:on={g.respect_ignore_files} class:dis={mainOff || !$isGitRepo}><span class="knob"></span></span>
		<span class="txt">
			<span class="t-title">Respeitar arquivos .ignore</span>
			<span class="t-desc">Padrões adicionais fora do controle do git</span>
		</span>
	</button>

	<!-- Lista de .gitignore -->
	{#if $isGitRepo}
		<div class="sec-head">.gitignore ativos</div>
		{#if $gitignoreInfoLoading}
			<div class="dim small">Carregando…</div>
		{:else if info && info.gitignore_files.length}
			<div class="files">
				{#each info.gitignore_files as f (f)}
					<div class="file" title={f}>📄 {shortenPath(f)}</div>
				{/each}
			</div>
		{:else}
			<div class="dim small">Nenhum .gitignore encontrado</div>
		{/if}

		{#if info && info.ignored_count > 0}
			<div class="warn">⚠ ~{fmtCount(info.ignored_count)} arquivos seriam incluídos sem esta configuração</div>
		{/if}
	{/if}
</div>

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
	.badge {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 6px 10px;
		border-radius: 6px;
		font-size: 12px;
	}
	.badge.repo {
		background: #34d39918;
		border: 1px solid #34d39930;
		color: #34d399;
	}
	.badge.norepo {
		background: #55555518;
		border: 1px solid #33333330;
		color: #555;
	}
	.hex {
		flex-shrink: 0;
	}
	.dim {
		color: #555;
	}
	.small {
		font-size: 11px;
	}

	/* Toggle (mesmo do IgnoreRulesEditor) */
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
	.toggle-row:disabled {
		cursor: not-allowed;
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
	.pill.dis {
		opacity: 0.4;
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

	.files {
		display: flex;
		flex-direction: column;
		gap: 3px;
	}
	.file {
		color: #555;
		font-size: 11px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.warn {
		margin-top: 4px;
		color: #f59e0b;
		font-size: 11px;
	}
</style>
