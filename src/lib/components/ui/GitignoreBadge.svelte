<script lang="ts">
	import { respectsGitignore, toggleRespectGitignore } from '$lib/stores/settings';
	import { isGitRepo, currentGitignoreInfo } from '$lib/stores/gitignoreInfo';

	let count = $derived($currentGitignoreInfo?.ignored_count ?? 0);

	function openPanel() {
		window.dispatchEvent(new CustomEvent('filterpanel:open'));
	}
</script>

{#if $isGitRepo}
	{#if $respectsGitignore}
		<button
			class="badge on"
			title="Respeitando .gitignore — ~{count.toLocaleString('pt-BR')} arquivos ignorados"
			onclick={openPanel}
		>
			<span class="ico">⬡</span><span>.gitignore</span>
		</button>
	{:else}
		<button
			class="badge off"
			title="Clique para ativar respeito ao .gitignore"
			onclick={toggleRespectGitignore}
		>
			<span class="ico">○</span><span>.gitignore off</span>
		</button>
	{/if}
{/if}

<style>
	.badge {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		height: 20px;
		padding: 2px 8px;
		border-radius: 10px;
		font-family: 'DM Sans', system-ui, sans-serif;
		font-size: 10px;
		cursor: pointer;
		flex-shrink: 0;
		animation: fade 200ms ease;
	}
	.badge.on {
		background: #34d39912;
		border: 1px solid #34d39928;
		color: #34d399;
	}
	.badge.off {
		background: #f59e0b12;
		border: 1px solid #f59e0b28;
		color: #f59e0b;
	}
	.ico {
		font-size: 10px;
	}
	@keyframes fade {
		from {
			opacity: 0;
		}
		to {
			opacity: 1;
		}
	}
</style>
