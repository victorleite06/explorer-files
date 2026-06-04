<script lang="ts">
	import { convertFileSrc } from '@tauri-apps/api/core';
	import type { FileEntry } from '$lib/tauri';
	import { isTauri } from '$lib/tauri';
	import { formatSize, formatDate, getFileIcon, getFileType } from '$lib/utils/fileUtils';

	let { entry }: { entry: FileEntry } = $props();

	const IMG = new Set(['jpg', 'jpeg', 'png', 'gif', 'webp', 'svg']);
	const TEXT = new Set([
		'txt', 'md', 'rs', 'ts', 'js', 'jsx', 'tsx', 'svelte', 'py', 'go', 'c', 'cpp', 'h', 'cs',
		'java', 'rb', 'php', 'json', 'toml', 'yaml', 'yml', 'xml', 'html', 'css', 'scss', 'sh', 'sql'
	]);

	let ext = $derived((entry.extension ?? '').toLowerCase());
	let category = $derived.by<'image' | 'text' | 'pdf' | 'other'>(() => {
		if (IMG.has(ext)) return 'image';
		if (TEXT.has(ext)) return 'text';
		if (ext === 'pdf') return 'pdf';
		return 'other';
	});

	// ── Leitura de texto (primeiras 2000 chars) ───────────────────
	let content = $state('');
	let textLoading = $state(false);

	$effect(() => {
		const e = entry;
		if (category !== 'text' || !isTauri()) {
			content = '';
			return;
		}
		textLoading = true;
		import('@tauri-apps/plugin-fs')
			.then(({ readTextFile }) => readTextFile(e.path))
			.then((txt) => {
				content = txt.slice(0, 2000);
			})
			.catch(() => {
				content = '';
			})
			.finally(() => {
				textLoading = false;
			});
	});

	// ── Highlight básico (tokenizer leve, sem @html) ──────────────
	const KEYWORDS = new Set([
		'const', 'let', 'var', 'function', 'fn', 'return', 'if', 'else', 'for', 'while', 'match',
		'import', 'export', 'from', 'class', 'struct', 'enum', 'impl', 'pub', 'use', 'async', 'await',
		'def', 'pub', 'type', 'interface', 'new', 'this', 'self', 'true', 'false', 'null', 'None', 'Some'
	]);

	type Tok = { t: string; c: 'cmt' | 'str' | 'kw' | 'id' | 'tx' };
	let tokens = $derived.by<Tok[]>(() => {
		if (!content) return [];
		const out: Tok[] = [];
		const re =
			/(\/\/[^\n]*|#[^\n]*)|("(?:[^"\\]|\\.)*"|'(?:[^'\\]|\\.)*'|`(?:[^`\\]|\\.)*`)|([A-Za-z_]\w*)|(\s+|[^\sA-Za-z_]+)/g;
		let m: RegExpExecArray | null;
		while ((m = re.exec(content))) {
			if (m[1]) out.push({ t: m[1], c: 'cmt' });
			else if (m[2]) out.push({ t: m[2], c: 'str' });
			else if (m[3]) out.push({ t: m[3], c: KEYWORDS.has(m[3]) ? 'kw' : 'id' });
			else out.push({ t: m[0], c: 'tx' });
		}
		return out;
	});

	let imgSrc = $derived(isTauri() ? convertFileSrc(entry.path) : '');
	let transparent = $derived(ext === 'svg' || ext === 'png' || ext === 'gif' || ext === 'webp');
</script>

<div class="preview">
	{#if category === 'image'}
		<div class="img-box" class:checker={transparent}>
			<img src={imgSrc} alt={entry.name} />
		</div>
	{:else if category === 'text'}
		{#if textLoading}
			<div class="hint">Carregando…</div>
		{:else if content}
			<pre class="code">{#each tokens as tok, i (i)}<span class={tok.c}>{tok.t}</span>{/each}</pre>
		{:else}
			<div class="hint">Sem preview disponível</div>
		{/if}
	{:else if category === 'pdf'}
		<div class="centered">
			<span class="big-ico">📋</span>
			<span class="fname">{entry.name}</span>
			<button class="ext-btn" type="button">Abrir externamente</button>
		</div>
	{:else}
		<div class="centered">
			<span class="big-ico">{getFileIcon(entry)}</span>
			<span class="fname">{entry.name}</span>
			<span class="sub">{formatSize(entry.size)}</span>
			<span class="sub">{formatDate(entry.modified)}</span>
		</div>
	{/if}

	<!-- METADADOS -->
	<div class="meta">
		<div class="meta-row"><span class="k">Tamanho</span><span class="v">{formatSize(entry.size)}</span></div>
		<div class="meta-row"><span class="k">Tipo</span><span class="v">{getFileType(entry)}</span></div>
		<div class="meta-row"><span class="k">Modificado</span><span class="v">{formatDate(entry.modified)}</span></div>
		<div class="meta-row"><span class="k">Criado</span><span class="v">—</span></div>
	</div>
</div>

<style>
	.preview {
		display: flex;
		flex-direction: column;
		gap: 12px;
		height: 100%;
		font-family: 'DM Sans', system-ui, sans-serif;
		color: #c8c8dc;
	}

	.img-box {
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 8px;
		border-radius: 6px;
	}
	.img-box img {
		max-width: 100%;
		max-height: 200px;
		object-fit: contain;
	}
	.checker {
		background-image:
			linear-gradient(45deg, #1a1a2e 25%, transparent 25%),
			linear-gradient(-45deg, #1a1a2e 25%, transparent 25%),
			linear-gradient(45deg, transparent 75%, #1a1a2e 75%),
			linear-gradient(-45deg, transparent 75%, #1a1a2e 75%);
		background-size: 16px 16px;
		background-position: 0 0, 0 8px, 8px -8px, -8px 0;
		background-color: #0e0e1c;
	}

	.code {
		margin: 0;
		padding: 8px;
		max-height: 360px;
		overflow: auto;
		background: #0a0a12;
		border: 1px solid #12121e;
		border-radius: 6px;
		font-family: 'DM Mono', ui-monospace, monospace;
		font-size: 11px;
		line-height: 1.5;
		white-space: pre-wrap;
		word-break: break-word;
		color: #c8c8dc;
	}
	.code .cmt {
		color: #555;
		font-style: italic;
	}
	.code .str {
		color: #34d399;
	}
	.code .kw {
		color: #a78bfa;
	}

	.centered {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
		padding: 20px 0;
	}
	.big-ico {
		font-size: 64px;
	}
	.fname {
		font-size: 13px;
		text-align: center;
		word-break: break-word;
	}
	.sub {
		font-size: 12px;
		color: #555;
	}
	.ext-btn {
		margin-top: 6px;
		padding: 6px 12px;
		border: 1px solid #1e1e35;
		border-radius: 6px;
		background: #111120;
		color: #9090a8;
		font-family: inherit;
		font-size: 12px;
		cursor: pointer;
	}
	.ext-btn:hover {
		background: #1a1a2e;
		color: #c8c8dc;
	}

	.hint {
		padding: 20px;
		text-align: center;
		color: #444;
		font-size: 12px;
	}

	.meta {
		margin-top: auto;
		display: flex;
		flex-direction: column;
		gap: 4px;
		padding-top: 12px;
		border-top: 1px solid #12121e;
	}
	.meta-row {
		display: flex;
		justify-content: space-between;
		gap: 8px;
	}
	.k {
		color: #444;
		font-size: 10px;
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}
	.v {
		color: #888;
		font-family: 'DM Mono', ui-monospace, monospace;
		font-size: 12px;
		text-align: right;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
</style>
