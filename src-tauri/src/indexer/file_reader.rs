use std::fs;
use std::io::Read;
use std::path::Path;

use crate::error::AppError;

pub const MAX_FILE_SIZE_BYTES: u64 = 10 * 1024 * 1024; // 10 MB
pub const MAX_CONTENT_CHARS: usize = 500_000;

pub const INDEXABLE_EXTENSIONS: &[&str] = &[
    "txt", "md", "markdown", "rst", "adoc", "asciidoc", "csv", "tsv", "log",
    "rs", "ts", "tsx", "js", "jsx", "svelte", "vue", "py", "pyw", "go", "rb",
    "php", "java", "kt", "swift", "c", "cpp", "cc", "cxx", "h", "hpp", "cs",
    "fs", "fsx", "lua", "r", "jl", "ex", "exs", "clj", "hs", "elm", "dart",
    "scala", "pl", "pm", "sh", "bash", "zsh", "fish", "ps1", "bat", "cmd",
    "json", "jsonc", "json5", "yaml", "yml", "toml", "ini", "env", "conf",
    "config", "properties", "xml", "html", "htm", "css", "scss", "sass",
    "less", "svg", "tex", "bib",
];

pub const NEVER_INDEX_NAMES: &[&str] = &[
    "package-lock.json",
    "yarn.lock",
    "pnpm-lock.yaml",
    "Cargo.lock",
    "Gemfile.lock",
    "composer.lock",
    ".DS_Store",
    "Thumbs.db",
];

pub struct FileContent {
    pub text: String,
    pub encoding: String,
    pub truncated: bool,
    pub line_count: usize,
}

/// Heurística rápida: presença de byte nulo nos primeiros 8KB = binário.
fn looks_binary(path: &Path) -> bool {
    let mut f = match fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return true,
    };
    let mut buf = [0u8; 8192];
    let n = f.read(&mut buf).unwrap_or(0);
    buf[..n].contains(&0)
}

pub fn is_indexable(path: &Path) -> bool {
    let meta = match fs::metadata(path) {
        Ok(m) => m,
        Err(_) => return false,
    };
    if !meta.is_file() {
        return false;
    }
    if meta.len() > MAX_FILE_SIZE_BYTES {
        return false;
    }

    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    if NEVER_INDEX_NAMES.iter().any(|n| n.to_lowercase() == name) {
        return false;
    }

    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase());
    match ext {
        Some(e) if INDEXABLE_EXTENSIONS.contains(&e.as_str()) => {}
        _ => return false,
    }

    !looks_binary(path)
}

/// Decodifica bytes detectando BOM / UTF-8 / fallback Windows-1252.
fn decode(bytes: &[u8]) -> (String, &'static str) {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        let (cow, _, _) = encoding_rs::UTF_8.decode(&bytes[3..]);
        return (cow.into_owned(), "UTF-8");
    }
    if bytes.starts_with(&[0xFF, 0xFE]) {
        let (cow, _, _) = encoding_rs::UTF_16LE.decode(&bytes[2..]);
        return (cow.into_owned(), "UTF-16LE");
    }
    if bytes.starts_with(&[0xFE, 0xFF]) {
        let (cow, _, _) = encoding_rs::UTF_16BE.decode(&bytes[2..]);
        return (cow.into_owned(), "UTF-16BE");
    }
    match std::str::from_utf8(bytes) {
        Ok(s) => (s.to_string(), "UTF-8"),
        Err(_) => {
            let (cow, _, _) = encoding_rs::WINDOWS_1252.decode(bytes);
            (cow.into_owned(), "Windows-1252")
        }
    }
}

/// Normaliza: \n, sem nulos, sem controles (exceto \n \t), espaços colapsados,
/// trim por linha.
fn normalize(raw: &str) -> String {
    let unified = raw.replace("\r\n", "\n").replace('\r', "\n");
    let mut out = String::with_capacity(unified.len());
    let mut first = true;
    for line in unified.split('\n') {
        if !first {
            out.push('\n');
        }
        first = false;

        let mut prev_space = false;
        for c in line.chars() {
            if c == '\0' {
                continue;
            }
            if c.is_control() && c != '\t' {
                continue;
            }
            let is_space = c == ' ' || c == '\t';
            if is_space {
                if prev_space {
                    continue;
                }
                prev_space = true;
                out.push(' ');
            } else {
                prev_space = false;
                out.push(c);
            }
        }
        // trim da linha recém-escrita
        let trimmed_len = out.trim_end_matches(' ').len();
        out.truncate(trimmed_len);
    }
    out
}

/// Trunca em limite de palavra se exceder MAX_CONTENT_CHARS.
fn truncate_words(text: &str) -> (String, bool) {
    if text.chars().count() <= MAX_CONTENT_CHARS {
        return (text.to_string(), false);
    }
    // Pega os primeiros MAX_CONTENT_CHARS chars, recua até whitespace.
    let mut end_byte = text
        .char_indices()
        .nth(MAX_CONTENT_CHARS)
        .map(|(i, _)| i)
        .unwrap_or(text.len());
    if let Some(ws) = text[..end_byte].rfind(char::is_whitespace) {
        if ws > 0 {
            end_byte = ws;
        }
    }
    (text[..end_byte].to_string(), true)
}

pub fn read_file_content(path: &Path) -> Result<FileContent, AppError> {
    let bytes = fs::read(path).map_err(|e| match e.kind() {
        std::io::ErrorKind::PermissionDenied => {
            AppError::Permission(path.to_string_lossy().into_owned())
        }
        _ => AppError::Io(e),
    })?;

    let (decoded, encoding) = decode(&bytes);
    let normalized = normalize(&decoded);
    let line_count = normalized.split('\n').count();
    let (text, truncated) = truncate_words(&normalized);

    Ok(FileContent {
        text,
        encoding: encoding.to_string(),
        truncated,
        line_count,
    })
}

/// Extrai até 3 trechos contendo os termos da query, com termos marcados
/// como <<<termo>>>. Fallback: primeiros `preview_chars` chars.
pub fn extract_text_preview(
    content: &str,
    query_terms: &[&str],
    preview_chars: usize,
) -> Vec<String> {
    let lower = content.to_lowercase();
    let mut snippets: Vec<String> = Vec::new();

    'outer: for term in query_terms {
        let t = term.to_lowercase();
        if t.is_empty() {
            continue;
        }
        let mut from = 0;
        while let Some(rel) = lower[from..].find(&t) {
            let pos = from + rel;
            let half = preview_chars / 2;
            let start = pos.saturating_sub(half);
            let end = (pos + t.len() + half).min(content.len());
            // ajusta para limites de char válidos
            let start = floor_char_boundary(content, start);
            let end = ceil_char_boundary(content, end);

            let raw = &content[start..end];
            // marca todas as ocorrências do termo no trecho
            let marked = mark_term(raw, &t);
            snippets.push(marked);

            if snippets.len() >= 3 {
                break 'outer;
            }
            from = pos + t.len();
        }
    }

    if snippets.is_empty() {
        let end = ceil_char_boundary(content, preview_chars.min(content.len()));
        snippets.push(content[..end].to_string());
    }
    snippets
}

fn mark_term(text: &str, term_lower: &str) -> String {
    let lower = text.to_lowercase();
    let mut out = String::with_capacity(text.len() + 16);
    let mut from = 0;
    while let Some(rel) = lower[from..].find(term_lower) {
        let pos = from + rel;
        out.push_str(&text[from..pos]);
        out.push_str("<<<");
        out.push_str(&text[pos..pos + term_lower.len()]);
        out.push_str(">>>");
        from = pos + term_lower.len();
    }
    out.push_str(&text[from..]);
    out
}

fn floor_char_boundary(s: &str, mut i: usize) -> usize {
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}
fn ceil_char_boundary(s: &str, mut i: usize) -> usize {
    while i < s.len() && !s.is_char_boundary(i) {
        i += 1;
    }
    i
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn tmp(name: &str, content: &[u8]) -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("fe_test_{}_{}", std::process::id(), name));
        let mut f = fs::File::create(&p).unwrap();
        f.write_all(content).unwrap();
        p
    }

    #[test]
    fn test_is_indexable_respects_size_limit() {
        let big = vec![b'a'; (MAX_FILE_SIZE_BYTES + 1) as usize];
        let p = tmp("big.txt", &big);
        assert!(!is_indexable(&p));
        let _ = fs::remove_file(&p);
    }

    #[test]
    fn test_is_indexable_rejects_binary() {
        // .exe não está nas extensões indexáveis
        let p = tmp("app.exe", b"MZ\x00\x00binary");
        assert!(!is_indexable(&p));
        let _ = fs::remove_file(&p);
        // arquivo .txt com byte nulo = binário
        let p2 = tmp("nul.txt", b"hello\x00world");
        assert!(!is_indexable(&p2));
        let _ = fs::remove_file(&p2);
    }

    #[test]
    fn test_is_indexable_accepts_rust_file() {
        let p = tmp("lib.rs", b"fn main() { println!(\"hi\"); }");
        assert!(is_indexable(&p));
        let _ = fs::remove_file(&p);
    }

    #[test]
    fn test_content_truncation() {
        let content = "a ".repeat(MAX_CONTENT_CHARS); // ~1M chars
        let p = tmp("long.txt", content.as_bytes());
        let fc = read_file_content(&p).unwrap();
        assert!(fc.truncated);
        assert!(fc.text.chars().count() <= MAX_CONTENT_CHARS);
        let _ = fs::remove_file(&p);
    }

    #[test]
    fn test_preview_extraction() {
        let content = "hello world foo bar baz qux foo again";
        let previews = extract_text_preview(content, &["foo"], 20);
        assert!(!previews.is_empty());
        assert!(previews.iter().any(|s| s.contains("<<<foo>>>")));
    }
}
