<script lang="ts">
	import type { FileEntry } from '$lib/tauri';
	import { getFileIcon } from '$lib/utils/fileUtils';

	let {
		entry,
		isSelected,
		onSelect
	}: { entry: FileEntry; isSelected: boolean; onSelect: () => void } = $props();
</script>

<button
	class="item"
	class:sel-dir={isSelected && entry.is_dir}
	class:sel-file={isSelected && !entry.is_dir}
	class:hiddenrow={entry.is_hidden}
	title={entry.name}
	onclick={onSelect}
>
	<span class="ico">{getFileIcon(entry)}</span>
	<span class="name">{entry.name}</span>
	{#if entry.is_dir}<span class="arrow">▶</span>{/if}
</button>

<style>
	.item {
		display: flex;
		align-items: center;
		gap: 6px;
		width: 100%;
		height: 26px;
		padding: 0 10px;
		border: none;
		background: transparent;
		color: #9090a8;
		font-family: 'DM Sans', system-ui, sans-serif;
		font-size: 13px;
		text-align: left;
		cursor: pointer;
		transition:
			background 80ms ease,
			color 80ms ease;
	}
	.item:hover {
		background: #111120;
		color: #c8c8dc;
	}
	.item.sel-dir {
		background: #1e1e35;
		color: #dddde8;
	}
	.item.sel-file {
		background: #a78bfa22;
		color: #dddde8;
	}
	.item.hiddenrow {
		opacity: 0.4;
	}
	.ico {
		font-size: 14px;
		flex-shrink: 0;
	}
	.name {
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.arrow {
		flex-shrink: 0;
		font-size: 8px;
		color: #555;
	}
	.item.sel-dir .arrow {
		color: #a78bfa;
	}
</style>
