use serde::{Deserialize, Serialize};
use unicode_normalization::char::is_combining_mark;
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuzzyMatch {
    pub score: i32,
    pub match_indices: Vec<usize>,
}

const SEPARATORS: &[char] = &[' ', '_', '-', '.', '/', '\\', '('];
const ITER_CAP: u32 = 1000;

/// Normaliza um char: remove acentos (NFD → base) e lowercase.
/// Mantém mapeamento 1:1 com os chars originais (índices preservados).
fn norm_char(c: char) -> char {
    let base = c.nfd().find(|&d| !is_combining_mark(d)).unwrap_or(c);
    base.to_lowercase().next().unwrap_or(base)
}

fn normalize(s: &str) -> Vec<char> {
    s.chars().map(norm_char).collect()
}

/// Pontuação de um conjunto de índices de match.
fn score_match(qn: &[char], tn: &[char], idx: &[usize]) -> i32 {
    let mut s = 0i32;

    // a) Match exato
    if tn == qn {
        s += 100;
    }
    // b) Prefixo
    if tn.starts_with(qn) {
        s += 80;
    }
    // f) Primeiro char do nome
    if idx.first() == Some(&0) {
        s += 40;
    }
    // c) Início de palavra (+50 por char)
    for &i in idx {
        if i == 0 || SEPARATORS.contains(&tn[i - 1]) {
            s += 50;
        }
    }
    // d) Grupos consecutivos (+30 por grupo)
    let mut k = 0;
    while k < idx.len() {
        let start = k;
        while k + 1 < idx.len() && idx[k + 1] == idx[k] + 1 {
            k += 1;
        }
        if k > start {
            s += 30;
        }
        k += 1;
    }
    // e) Penalidade de distância (chars pulados dentro do span)
    if let (Some(&first), Some(&last)) = (idx.first(), idx.last()) {
        let span = last - first + 1;
        let skipped = span.saturating_sub(idx.len());
        s -= skipped as i32;
    }

    s
}

/// Busca recursiva o melhor conjunto de índices (greedy-first + backtracking
/// limitado). A primeira solução completa é o match mais à esquerda.
fn best_indices(qn: &[char], tn: &[char]) -> Option<Vec<usize>> {
    let mut best: Option<(i32, Vec<usize>)> = None;
    let mut cur: Vec<usize> = Vec::with_capacity(qn.len());
    let mut iters: u32 = 0;
    rec(qn, tn, 0, 0, &mut cur, &mut best, &mut iters);
    best.map(|(_, idx)| idx)
}

fn rec(
    qn: &[char],
    tn: &[char],
    qi: usize,
    ti: usize,
    cur: &mut Vec<usize>,
    best: &mut Option<(i32, Vec<usize>)>,
    iters: &mut u32,
) {
    if qi == qn.len() {
        let s = score_match(qn, tn, cur);
        if best.as_ref().map_or(true, |(bs, _)| s > *bs) {
            *best = Some((s, cur.clone()));
        }
        return;
    }
    for i in ti..tn.len() {
        if *iters >= ITER_CAP {
            return;
        }
        if tn[i] == qn[qi] {
            *iters += 1;
            cur.push(i);
            rec(qn, tn, qi + 1, i + 1, cur, best, iters);
            cur.pop();
        }
    }
}

pub fn fuzzy_match(query: &str, text: &str) -> Option<FuzzyMatch> {
    if query.is_empty() {
        return None;
    }
    let qn = normalize(query);
    let tn = normalize(text);
    if qn.len() > tn.len() {
        return None;
    }
    let idx = best_indices(&qn, &tn)?;
    let score = score_match(&qn, &tn, &idx);
    Some(FuzzyMatch {
        score,
        match_indices: idx,
    })
}

fn last_segment(path: &str) -> &str {
    let sep = if path.contains('\\') { '\\' } else { '/' };
    path.rsplit(sep).find(|s| !s.is_empty()).unwrap_or(path)
}

pub fn fuzzy_match_path(query: &str, path: &str) -> Option<FuzzyMatch> {
    let name = last_segment(path);
    if let Some(m) = fuzzy_match(query, name) {
        return Some(m);
    }
    // Fallback: tenta no path completo, score reduzido e sem highlight
    // (índices se referem ao path, não ao name).
    if let Some(mut m) = fuzzy_match(query, path) {
        m.score -= 20;
        m.match_indices = Vec::new();
        return Some(m);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        let m = fuzzy_match("main.rs", "main.rs").unwrap();
        // exato (+100) + prefixo (+80) + primeiro char (+40) + word-starts
        assert!(m.score >= 100, "score exato deveria ser alto: {}", m.score);
        assert_eq!(m.match_indices, vec![0, 1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_prefix_scores_higher() {
        let prefix = fuzzy_match("cat", "category").unwrap();
        let middle = fuzzy_match("cat", "locate").unwrap();
        assert!(
            prefix.score > middle.score,
            "prefixo {} deveria > meio {}",
            prefix.score,
            middle.score
        );
    }

    #[test]
    fn test_consecutive_bonus() {
        let consec = fuzzy_match("abc", "abcxyz").unwrap();
        let spread = fuzzy_match("abc", "axbxcx").unwrap();
        assert!(
            consec.score > spread.score,
            "consecutivo {} deveria > espalhado {}",
            consec.score,
            spread.score
        );
    }

    #[test]
    fn test_no_match_returns_none() {
        assert!(fuzzy_match("xyz", "abc").is_none());
        assert!(fuzzy_match("", "abc").is_none());
    }

    #[test]
    fn test_accent_normalization() {
        assert!(fuzzy_match("cafe", "café").is_some());
        assert!(fuzzy_match("café", "cafe").is_some());
        assert!(fuzzy_match("acao", "ação").is_some());
    }

    #[test]
    fn test_common_filename_patterns() {
        for name in ["main.rs", "main.ts", "main.py"] {
            assert!(fuzzy_match("main", name).is_some(), "deveria casar {name}");
        }
    }

    #[test]
    fn test_path_matching() {
        let m = fuzzy_match_path("src/m", "project/src/main.rs");
        assert!(m.is_some(), "src/m deveria casar no path");
    }
}
