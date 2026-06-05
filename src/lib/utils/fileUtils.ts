import type { FileEntry } from '$lib/tauri';

// ── Tamanho ─────────────────────────────────────────────────────
export function formatSize(bytes: number): string {
	if (bytes === 0) return '-';
	if (bytes < 1024) return `${bytes} B`;
	if (bytes < 1_048_576) return `${(bytes / 1024).toFixed(1)} KB`;
	if (bytes < 1_073_741_824) return `${(bytes / 1_048_576).toFixed(1)} MB`;
	return `${(bytes / 1_073_741_824).toFixed(2)} GB`;
}

// ── Data ────────────────────────────────────────────────────────
const timeFmt = new Intl.DateTimeFormat('pt-BR', { hour: '2-digit', minute: '2-digit' });
const monthFmt = new Intl.DateTimeFormat('pt-BR', { month: 'short' });
const fullFmt = new Intl.DateTimeFormat('pt-BR', {
	day: '2-digit',
	month: '2-digit',
	year: 'numeric'
});

function sameDay(a: Date, b: Date): boolean {
	return (
		a.getFullYear() === b.getFullYear() &&
		a.getMonth() === b.getMonth() &&
		a.getDate() === b.getDate()
	);
}

export function formatDate(iso: string | undefined): string {
	if (!iso) return '-';
	const d = new Date(iso);
	if (isNaN(d.getTime())) return '-';

	const now = new Date();
	const yesterday = new Date(now);
	yesterday.setDate(now.getDate() - 1);

	const time = timeFmt.format(d);

	if (sameDay(d, now)) return `Hoje, ${time}`;
	if (sameDay(d, yesterday)) return `Ontem, ${time}`;

	if (d.getFullYear() === now.getFullYear()) {
		// ex: '12 Jun, 14:30'
		let month = monthFmt.format(d).replace('.', '');
		month = month.charAt(0).toUpperCase() + month.slice(1);
		const day = String(d.getDate()).padStart(2, '0');
		return `${day} ${month}, ${time}`;
	}

	return fullFmt.format(d); // DD/MM/YYYY
}

// ── Ícone ───────────────────────────────────────────────────────
const ICON_BY_EXT: Record<string, string> = {};
function reg(icon: string, exts: string[]) {
	for (const e of exts) ICON_BY_EXT[e] = icon;
}
reg('🖼️', ['jpg', 'jpeg', 'png', 'gif', 'webp', 'svg', 'ico']);
reg('🎬', ['mp4', 'mkv', 'avi', 'mov', 'webm']);
reg('🎵', ['mp3', 'wav', 'flac', 'ogg', 'm4a']);
reg('📝', ['rs', 'ts', 'js', 'tsx', 'jsx', 'svelte', 'py', 'go', 'cpp', 'c', 'h', 'cs', 'java', 'rb', 'php', 'swift']);
reg('🌐', ['html', 'css', 'scss', 'less']);
reg('📊', ['json', 'yaml', 'yml', 'toml', 'xml', 'csv', 'sql', 'ppt', 'pptx', 'odp']);
reg('📋', ['pdf']);
reg('📃', ['doc', 'docx', 'odt']);
reg('📈', ['xls', 'xlsx', 'ods']);
reg('📦', ['zip', 'tar', 'gz', 'rar', '7z']);
reg('⚙️', ['exe', 'msi', 'dmg', 'deb', 'appimage']);
reg('🔤', ['ttf', 'otf', 'woff', 'woff2']);

type IconLike = { is_dir?: boolean; extension?: string | null };

export function getFileIcon(entry: IconLike): string {
	if (entry.is_dir) return '📁';
	const ext = (entry.extension ?? '').toLowerCase();
	return ICON_BY_EXT[ext] ?? '📄';
}

// ── Tipo legível ────────────────────────────────────────────────
const TYPE_BY_EXT: Record<string, string> = {
	jpg: 'Imagem JPEG',
	jpeg: 'Imagem JPEG',
	png: 'Imagem PNG',
	gif: 'Imagem GIF',
	webp: 'Imagem WebP',
	svg: 'Imagem SVG',
	ico: 'Ícone',
	mp4: 'Vídeo MP4',
	mkv: 'Vídeo MKV',
	avi: 'Vídeo AVI',
	mov: 'Vídeo MOV',
	webm: 'Vídeo WebM',
	mp3: 'Áudio MP3',
	wav: 'Áudio WAV',
	flac: 'Áudio FLAC',
	ogg: 'Áudio OGG',
	m4a: 'Áudio M4A',
	rs: 'Script Rust',
	ts: 'Script TypeScript',
	js: 'Script JavaScript',
	tsx: 'Componente TSX',
	jsx: 'Componente JSX',
	svelte: 'Componente Svelte',
	py: 'Script Python',
	go: 'Script Go',
	cpp: 'Código C++',
	c: 'Código C',
	h: 'Cabeçalho C',
	cs: 'Script C#',
	java: 'Código Java',
	rb: 'Script Ruby',
	php: 'Script PHP',
	swift: 'Script Swift',
	html: 'Documento HTML',
	css: 'Folha de Estilo',
	scss: 'Folha de Estilo SCSS',
	less: 'Folha de Estilo LESS',
	json: 'Dados JSON',
	yaml: 'Dados YAML',
	yml: 'Dados YAML',
	toml: 'Dados TOML',
	xml: 'Dados XML',
	csv: 'Dados CSV',
	sql: 'Script SQL',
	pdf: 'Documento PDF',
	doc: 'Documento Word',
	docx: 'Documento Word',
	odt: 'Documento ODT',
	xls: 'Planilha Excel',
	xlsx: 'Planilha Excel',
	ods: 'Planilha ODS',
	ppt: 'Apresentação',
	pptx: 'Apresentação',
	odp: 'Apresentação ODP',
	zip: 'Arquivo ZIP',
	tar: 'Arquivo TAR',
	gz: 'Arquivo GZIP',
	rar: 'Arquivo RAR',
	'7z': 'Arquivo 7-Zip',
	exe: 'Executável',
	msi: 'Instalador',
	dmg: 'Imagem de Disco',
	deb: 'Pacote Debian',
	appimage: 'AppImage',
	ttf: 'Fonte TrueType',
	otf: 'Fonte OpenType',
	woff: 'Fonte Web',
	woff2: 'Fonte Web'
};

export function getFileType(entry: FileEntry): string {
	if (entry.is_dir) return 'Pasta';
	const ext = (entry.extension ?? '').toLowerCase();
	if (!ext) return 'Arquivo';
	return TYPE_BY_EXT[ext] ?? `${ext.toUpperCase()} File`;
}

// ── Ordenação ───────────────────────────────────────────────────
type SortBy = 'name' | 'size' | 'modified' | 'type';
type SortDir = 'asc' | 'desc';

function comparator(by: SortBy): (a: FileEntry, b: FileEntry) => number {
	switch (by) {
		case 'name':
			return (a, b) => a.name.localeCompare(b.name, 'pt-BR');
		case 'size':
			return (a, b) => a.size - b.size;
		case 'modified':
			return (a, b) => (a.modified ?? '').localeCompare(b.modified ?? '');
		case 'type':
			return (a, b) => {
				const t = getFileType(a).localeCompare(getFileType(b), 'pt-BR');
				return t !== 0 ? t : a.name.localeCompare(b.name, 'pt-BR');
			};
	}
}

export function sortEntries(entries: FileEntry[], sortBy: SortBy, sortDir: SortDir): FileEntry[] {
	const cmp = comparator(sortBy);
	const dirs = entries.filter((e) => e.is_dir).sort(cmp);
	const files = entries.filter((e) => !e.is_dir).sort(cmp);
	if (sortDir === 'desc') {
		dirs.reverse();
		files.reverse();
	}
	// Pastas sempre primeiro, independente da direção.
	return [...dirs, ...files];
}
