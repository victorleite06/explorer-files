import { describe, it, expect } from 'vitest';
import {
	buildSegmentsFromIndices,
	buildSegmentsFromTerms,
	parseDelimitedPreview,
	truncateWithHighlight,
	segmentsToHtml
} from './highlightUtils';

describe('buildSegmentsFromIndices', () => {
	it('retorna texto completo sem highlight quando indices vazio', () => {
		const segs = buildSegmentsFromIndices('main.rs', []);
		expect(segs).toHaveLength(1);
		expect(segs[0].highlighted).toBe(false);
		expect(segs[0].text).toBe('main.rs');
	});

	it('agrupa índices consecutivos corretamente', () => {
		const segs = buildSegmentsFromIndices('package.json', [0, 4, 5]);
		const highlighted = segs.filter((s) => s.highlighted).map((s) => s.text);
		expect(highlighted).toContain('p');
		expect(highlighted).toContain('ag');
	});

	it('ignora índices fora do range', () => {
		const segs = buildSegmentsFromIndices('abc', [0, 99, -1]);
		expect(segs.every((s) => s.text.length > 0)).toBe(true);
	});

	it('match exato marca o texto inteiro', () => {
		const segs = buildSegmentsFromIndices('main', [0, 1, 2, 3]);
		expect(segs).toHaveLength(1);
		expect(segs[0].highlighted).toBe(true);
		expect(segs[0].text).toBe('main');
	});
});

describe('buildSegmentsFromTerms', () => {
	it('encontra termo no meio do texto', () => {
		const segs = buildSegmentsFromTerms('file_reader.rs', ['reader']);
		const hl = segs.filter((s) => s.highlighted);
		expect(hl).toHaveLength(1);
		expect(hl[0].text).toBe('reader');
	});

	it('busca é case-insensitive por padrão', () => {
		const segs = buildSegmentsFromTerms('Main.rs', ['main']);
		const hl = segs.filter((s) => s.highlighted);
		expect(hl[0].text).toBe('Main');
	});

	it('normaliza acentos', () => {
		const segs = buildSegmentsFromTerms('índice.txt', ['indice']);
		expect(segs.some((s) => s.highlighted)).toBe(true);
	});

	it('múltiplos termos', () => {
		const segs = buildSegmentsFromTerms('async fn main()', ['async', 'main']);
		const hl = segs.filter((s) => s.highlighted).map((s) => s.text);
		expect(hl).toContain('async');
		expect(hl).toContain('main');
	});

	it('mescla intervalos sobrepostos', () => {
		const segs = buildSegmentsFromTerms('asyncfn', ['async', 'asyncfn']);
		const hlText = segs
			.filter((s) => s.highlighted)
			.map((s) => s.text)
			.join('');
		expect(hlText).toBe('asyncfn');
	});
});

describe('parseDelimitedPreview', () => {
	it('parseia delimitadores corretamente', () => {
		const segs = parseDelimitedPreview('let <<<x>>> = <<<1>>>');
		expect(segs.filter((s) => s.highlighted).map((s) => s.text)).toEqual(['x', '1']);
	});

	it('texto sem delimitadores é segmento normal', () => {
		const segs = parseDelimitedPreview('sem highlight aqui');
		expect(segs).toHaveLength(1);
		expect(segs[0].highlighted).toBe(false);
	});

	it('delimitador no início do texto', () => {
		const segs = parseDelimitedPreview('<<<fn>>> main()');
		expect(segs[0].highlighted).toBe(true);
		expect(segs[0].text).toBe('fn');
	});
});

describe('truncateWithHighlight', () => {
	it('não trunca se total <= maxChars', () => {
		const segs = buildSegmentsFromIndices('abc', [1]);
		const result = truncateWithHighlight(segs, 100);
		expect(result.some((s) => s.text === '...')).toBe(false);
	});

	it('adiciona reticências ao truncar', () => {
		const long = 'a'.repeat(200);
		const segs = buildSegmentsFromIndices(long, [100]);
		const result = truncateWithHighlight(segs, 60, 10);
		const texts = result.map((s) => s.text);
		expect(texts.some((t) => t === '...')).toBe(true);
	});

	it('preserva o highlight no resultado truncado', () => {
		const segs = buildSegmentsFromIndices('a'.repeat(200), [150]);
		const result = truncateWithHighlight(segs, 60, 10);
		expect(result.some((s) => s.highlighted)).toBe(true);
	});
});

describe('segmentsToHtml', () => {
	it('escapa caracteres HTML em texto normal', () => {
		const segs = [{ text: '<script>', highlighted: false, source: 'filter' as const }];
		expect(segmentsToHtml(segs)).toBe('&lt;script&gt;');
	});

	it('wraps highlighted em <mark>', () => {
		const segs = [{ text: 'rust', highlighted: true, source: 'fuzzy' as const }];
		expect(segmentsToHtml(segs)).toBe('<mark class="hl">rust</mark>');
	});

	it('usa highlightClass customizada', () => {
		const segs = [{ text: 'rs', highlighted: true, source: 'content' as const }];
		expect(segmentsToHtml(segs, 'hl-content')).toContain('class="hl-content"');
	});

	it('escapa texto highlighted também', () => {
		const segs = [{ text: '<b>bold</b>', highlighted: true, source: 'fuzzy' as const }];
		const html = segmentsToHtml(segs);
		expect(html).toContain('&lt;b&gt;');
		expect(html).not.toContain('<b>');
	});
});
