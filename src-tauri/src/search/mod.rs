pub mod fuzzy;

use std::path::Path;
use std::time::SystemTime;

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::error::AppError;
use crate::fs_engine::ignore_rules::{should_hide, IgnoreRules};
use fuzzy::fuzzy_match_path;

const MAX_CANDIDATES: usize = 10_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<String>,
    pub extension: Option<String>,
    pub score: i32,
    pub match_indices: Vec<usize>,
    pub parent_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchOptions {
    pub max_results: usize,
    pub include_dirs: bool,
    pub include_files: bool,
    pub recursive: bool,
    pub max_depth: u8,
    pub extensions: Vec<String>,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            max_results: 50,
            include_dirs: true,
            include_files: true,
            recursive: true,
            max_depth: 5,
            extensions: vec![],
        }
    }
}

// Candidato coletado antes do scoring.
struct Candidate {
    path: String,
    name: String,
    is_dir: bool,
    size: u64,
    modified: Option<String>,
    extension: Option<String>,
    parent_path: String,
}

fn to_iso8601(time: SystemTime) -> Option<String> {
    let dur = time.duration_since(SystemTime::UNIX_EPOCH).ok()?;
    Some(chrono::DateTime::<chrono::Utc>::from(SystemTime::UNIX_EPOCH + dur).to_rfc3339())
}

pub fn search_by_name(
    query: &str,
    root_path: &str,
    options: &SearchOptions,
    rules: &IgnoreRules,
) -> Result<Vec<SearchResult>, AppError> {
    // 1. Validação
    let qlen = query.chars().count();
    let short_dir_ok = qlen == 1 && query.ends_with('/');
    if qlen < 2 && !short_dir_ok {
        return Err(AppError::Invalid(
            "Query muito curta, use pelo menos 2 caracteres".into(),
        ));
    }

    let root = Path::new(root_path);
    if !root.exists() {
        return Err(AppError::NotFound(root_path.to_string()));
    }

    // 2. Coleta de candidatos
    let max_depth = if options.recursive {
        options.max_depth as usize
    } else {
        1
    };

    let walker = WalkDir::new(root)
        .max_depth(max_depth)
        .into_iter()
        .filter_entry(|e| {
            if e.depth() == 0 {
                return true; // raiz sempre entra
            }
            let name = e.file_name().to_string_lossy();
            !should_hide(&name, e.file_type().is_dir(), rules)
        });

    let mut candidates: Vec<Candidate> = Vec::new();
    let mut truncated = false;

    for entry in walker.filter_map(|e| e.ok()) {
        if entry.depth() == 0 {
            continue; // não inclui a própria raiz
        }
        let is_dir = entry.file_type().is_dir();
        if is_dir && !options.include_dirs {
            continue;
        }
        if !is_dir && !options.include_files {
            continue;
        }

        let path = entry.path();
        let extension = path
            .extension()
            .map(|e| e.to_string_lossy().into_owned());

        // Filtro de extensão (apenas arquivos)
        if !is_dir && !options.extensions.is_empty() {
            let ext_ok = extension
                .as_deref()
                .map(|e| options.extensions.iter().any(|x| x.eq_ignore_ascii_case(e)))
                .unwrap_or(false);
            if !ext_ok {
                continue;
            }
        }

        let meta = entry.metadata().ok();
        candidates.push(Candidate {
            path: path.to_string_lossy().into_owned(),
            name: entry.file_name().to_string_lossy().into_owned(),
            is_dir,
            size: if is_dir {
                0
            } else {
                meta.as_ref().map(|m| m.len()).unwrap_or(0)
            },
            modified: meta.as_ref().and_then(|m| m.modified().ok()).and_then(to_iso8601),
            extension,
            parent_path: path
                .parent()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_default(),
        });

        if candidates.len() >= MAX_CANDIDATES {
            truncated = true;
            break;
        }
    }

    if truncated {
        log::warn!(
            "search_by_name: limite de {MAX_CANDIDATES} candidatos atingido em {root_path}; resultados parciais"
        );
    }

    // 3. Scoring paralelo
    let mut results: Vec<SearchResult> = candidates
        .par_iter()
        .filter_map(|c| {
            let m = fuzzy_match_path(query, &c.path)?;
            Some(SearchResult {
                path: c.path.clone(),
                name: c.name.clone(),
                is_dir: c.is_dir,
                size: c.size,
                modified: c.modified.clone(),
                extension: c.extension.clone(),
                score: m.score,
                match_indices: m.match_indices,
                parent_path: c.parent_path.clone(),
            })
        })
        .collect();

    // 4. Ordenação: score desc, desempate nome mais curto primeiro
    results.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| a.name.chars().count().cmp(&b.name.chars().count()))
    });

    // 5. Limite
    results.truncate(options.max_results);

    Ok(results)
}
