<script lang="ts">
	import { fly } from 'svelte/transition';
	import { toasts } from '$lib/stores/bookmarks';

	const ICON = { success: '✓', error: '✕', info: 'ℹ' } as const;
</script>

<div class="container">
	{#each $toasts as toast (toast.id)}
		<div
			class="toast {toast.type}"
			role="status"
			in:fly={{ x: 320, duration: 300 }}
			out:fly={{ x: 320, duration: 200 }}
		>
			<span class="ico">{ICON[toast.type]}</span>
			<span class="msg">{toast.message}</span>
		</div>
	{/each}
</div>

<style>
	.container {
		position: fixed;
		bottom: 16px;
		right: 16px;
		z-index: 9999;
		display: flex;
		flex-direction: column-reverse;
		gap: 8px;
		pointer-events: none;
	}
	.toast {
		display: flex;
		align-items: center;
		gap: 10px;
		min-width: 240px;
		max-width: 360px;
		padding: 10px 14px;
		background: #111120;
		border: 1px solid #1e1e35;
		border-left-width: 3px;
		border-radius: 6px;
		color: #c8c8dc;
		font-family: 'DM Sans', system-ui, sans-serif;
		font-size: 13px;
		box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
		pointer-events: auto;
	}
	.toast.success {
		border-left-color: #34d399;
	}
	.toast.error {
		border-left-color: #f87171;
	}
	.toast.info {
		border-left-color: #38bdf8;
	}
	.ico {
		flex-shrink: 0;
		font-size: 13px;
		font-weight: 700;
	}
	.toast.success .ico {
		color: #34d399;
	}
	.toast.error .ico {
		color: #f87171;
	}
	.toast.info .ico {
		color: #38bdf8;
	}
	.msg {
		flex: 1;
		word-break: break-word;
	}
</style>
