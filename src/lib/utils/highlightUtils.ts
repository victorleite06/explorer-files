export type HighlightSource = 'fuzzy' | 'content' | 'filter';

export interface TextSegment {
	text: string;
	highlighted: boolean;
	source: HighlightSource;
	isEllipsis?: boolean;
}

export interface HighlightOptions {
	caseSensitive: boolean;
	normalizeAccents: boolean;
	maxSegments: number;
}

export const DEFAULT_HIGHLIGHT_OPTIONS: HighlightOptions = {
	caseSensitive: false,
	normalizeAccents: true,
	maxSegments: 500
};

interface Range {
	start: number;
	end: number; // inclusivo
}

// ── 1. Índices (fuzzy) ──────────────────────────────────────────
export function buildSegmentsFromIndices(
	text: string,
	indices: number[],
	source: HighlightSource = 'fuzzy'
): TextSegment[] {
	const chars = [...text];
	const n = chars.length;
	if (n === 0) return [];
	if (indices.length === 0) return [{ text, highlighted: false, source }];

	const valid = [...new Set(indices.filter((i) => i >= 0 && i < n))].sort((a, b) => a - b);
	if (valid.length === 0) return [{ text, highlighted: false, source }];

	// Agrupa índices consecutivos em ranges inclusivos.
	const ranges: Range[] = [];
	let start = valid[0];
	let prev = valid[0];
	for (let k = 1; k < valid.length; k++) {
		if (valid[k] === prev + 1) {
			prev = valid[k];
		} else {
			ranges.push({ start, end: prev });
			start = valid[k];
			prev = valid[k];
		}
	}
	ranges.push({ start, end: prev });

	const segments: TextSegment[] = [];
	let i = 0;
	let ri = 0;
	while (i < n) {
		const inRange = ri < ranges.length && i >= ranges[ri].start && i <= ranges[ri].end;
		const segStart = i;
		if (inRange) {
			while (i <= ranges[ri].end) i++;
			ri++;
		} else {
			const nextStart = ri < ranges.length ? ranges[ri].start : n;
			i = nextStart;
		}
		const slice = chars.slice(segStart, i).join('');
		if (slice.length > 0) segments.push({ text: slice, highlighted: inRange, source });
	}

	if (segments.length > DEFAULT_HIGHLIGHT_OPTIONS.maxSegments) {
		return [{ text, highlighted: false, source }];
	}
	return segments;
}

// ── 2. Termos ───────────────────────────────────────────────────
function normChar(c: string, lower: boolean): string {
	const d = c.normalize('NFD');
	let base = '';
	for (const ch of d) {
		if (!/[\u0300-\u036f]/.test(ch)) {
			base = ch;
			break;
		}
	}
	base = base || c;
	return lower ? base.toLowerCase() : base;
}

// Normaliza mantendo mapeamento 1:1 de chars (índices preservados).
function normalizeKeepLen(s: string, normalizeAccents: boolean, lower: boolean): string {
	return [...s]
		.map((c) => (normalizeAccents ? normChar(c, lower) : lower ? c.toLowerCase() : c))
		.join('');
}

function mergeRanges(ranges: Range[]): Range[] {
	if (ranges.length === 0) return [];
	const sorted = [...ranges].sort((a, b) => a.start - b.start);
	const out: Range[] = [sorted[0]];
	for (let k = 1; k < sorted.length; k++) {
		const last = out[out.length - 1];
		const cur = sorted[k];
		// Sobrepostos ou adjacentes (cur.start <= last.end + 1).
		if (cur.start <= last.end + 1) {
			last.end = Math.max(last.end, cur.end);
		} else {
			out.push({ ...cur });
		}
	}
	return out;
}

export function buildSegmentsFromTerms(
	text: string,
	terms: string[],
	options: Partial<HighlightOptions> = {},
	source: HighlightSource = 'filter'
): TextSegment[] {
	const opts = { ...DEFAULT_HIGHLIGHT_OPTIONS, ...options };
	const chars = [...text];
	const n = chars.length;
	if (n === 0) return [];

	const lower = !opts.caseSensitive;
	const normText = normalizeKeepLen(text, opts.normalizeAccents, lower);

	// Termos mais longos primeiro (evita overlaps parciais).
	const normTerms = terms
		.map((t) => normalizeKeepLen(t, opts.normalizeAccents, lower))
		.filter((t) => t.length > 0)
		.sort((a, b) => b.length - a.length);

	const ranges: Range[] = [];
	for (const t of normTerms) {
		let from = 0;
		let idx = normText.indexOf(t, from);
		while (idx !== -1) {
			ranges.push({ start: idx, end: idx + t.length - 1 });
			from = idx + t.length;
			idx = normText.indexOf(t, from);
		}
	}

	const merged = mergeRanges(ranges);
	if (merged.length === 0) return [{ text, highlighted: false, source }];

	const segments: TextSegment[] = [];
	let i = 0;
	let ri = 0;
	while (i < n) {
		const inRange = ri < merged.length && i >= merged[ri].start && i <= merged[ri].end;
		const segStart = i;
		if (inRange) {
			while (i <= merged[ri].end) i++;
			ri++;
		} else {
			i = ri < merged.length ? merged[ri].start : n;
		}
		const slice = chars.slice(segStart, i).join('');
		if (slice.length > 0) segments.push({ text: slice, highlighted: inRange, source });
	}

	if (segments.length > opts.maxSegments) {
		return [{ text, highlighted: false, source }];
	}
	return segments;
}

// ── 3. Preview delimitado <<<termo>>> ───────────────────────────
export function parseDelimitedPreview(
	raw: string,
	source: HighlightSource = 'content'
): TextSegment[] {
	const parts = raw.split(/<<<(.*?)>>>/g);
	const segments: TextSegment[] = [];
	parts.forEach((part, idx) => {
		if (part.length === 0) return;
		segments.push({ text: part, highlighted: idx % 2 === 1, source });
	});
	return segments;
}

// ── 4. HTML ─────────────────────────────────────────────────────
function escapeHtml(s: string): string {
	return s
		.replace(/&/g, '&amp;')
		.replace(/</g, '&lt;')
		.replace(/>/g, '&gt;')
		.replace(/"/g, '&quot;');
}

export function segmentsToHtml(segments: TextSegment[], highlightClass = 'hl'): string {
	let out = '';
	for (const seg of segments) {
		const safe = escapeHtml(seg.text);
		out += seg.highlighted ? `<mark class="${highlightClass}">${safe}</mark>` : safe;
	}
	return out;
}

// ── 5. Truncar ao redor do 1º highlight ─────────────────────────
export function truncateWithHighlight(
	segments: TextSegment[],
	maxChars: number,
	padding = 20
): TextSegment[] {
	const total = segments.reduce((acc, s) => acc + [...s.text].length, 0);
	if (total <= maxChars) return segments;

	const source = segments[0]?.source ?? 'filter';

	// Offset do 1º highlight.
	let offset = 0;
	let firstHl = -1;
	for (const s of segments) {
		if (s.highlighted) {
			firstHl = offset;
			break;
		}
		offset += [...s.text].length;
	}
	if (firstHl < 0) firstHl = 0;

	const start = Math.max(0, firstHl - padding);
	const end = start + maxChars;

	const out: TextSegment[] = [];
	if (start > 0) out.push({ text: '...', highlighted: false, source, isEllipsis: true });

	let pos = 0;
	for (const s of segments) {
		const sChars = [...s.text];
		const sStart = pos;
		const sEnd = pos + sChars.length; // exclusivo
		pos = sEnd;

		if (sEnd <= start || sStart >= end) continue; // fora da janela

		const from = Math.max(0, start - sStart);
		const to = Math.min(sChars.length, end - sStart);
		const slice = sChars.slice(from, to).join('');
		if (slice.length > 0) out.push({ text: slice, highlighted: s.highlighted, source: s.source });
	}

	if (end < total) out.push({ text: '...', highlighted: false, source, isEllipsis: true });
	return out;
}

// ── 6. Termos da query ──────────────────────────────────────────
export function extractQueryTerms(query: string): string[] {
	const ops = new Set(['and', 'or', 'not']);
	const seen = new Set<string>();
	const out: string[] = [];
	for (const raw of query.split(/\s+/)) {
		const t = raw.trim().toLowerCase();
		if (t.length < 2 || ops.has(t) || seen.has(t)) continue;
		seen.add(t);
		out.push(t);
	}
	return out;
}
