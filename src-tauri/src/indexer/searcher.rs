use std::path::Path;

use serde::{Deserialize, Serialize};
use tantivy::collector::{Count, TopDocs};
use tantivy::query::{BooleanQuery, Occur, Query, QueryParser, TermQuery};
use tantivy::schema::{IndexRecordOption, Value};
use tantivy::{Index, TantivyDocument, Term};

use crate::error::AppError;
use crate::indexer::file_reader::{extract_text_preview, read_file_content};
use crate::indexer::schema::IndexSchema;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSearchResult {
    pub path: String,
    pub name: String,
    pub extension: Option<String>,
    pub size: u64,
    pub modified: Option<String>,
    pub score: f32,
    pub previews: Vec<String>,
    pub match_count: usize,
    pub parent_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContentSearchQuery {
    pub query: String,
    pub root_path: Option<String>,
    pub extensions: Vec<String>,
    pub max_results: usize,
    pub include_previews: bool,
    pub preview_chars: usize,
}

impl Default for ContentSearchQuery {
    fn default() -> Self {
        Self {
            query: String::new(),
            root_path: None,
            extensions: vec![],
            max_results: 50,
            include_previews: true,
            preview_chars: 200,
        }
    }
}

pub struct ContentSearcher;

fn ts_to_iso(secs: u64) -> Option<String> {
    chrono::DateTime::from_timestamp(secs as i64, 0).map(|d| d.to_rfc3339())
}

impl ContentSearcher {
    pub fn search_content(
        &self,
        query: &ContentSearchQuery,
        index: &Index,
        schema: &IndexSchema,
    ) -> Result<Vec<ContentSearchResult>, AppError> {
        if query.query.trim().is_empty() {
            return Ok(vec![]);
        }

        // 1. Parse
        let parser = QueryParser::for_index(index, vec![schema.f_name, schema.f_content]);
        let main_query = parser
            .parse_query(&query.query)
            .map_err(|e| AppError::NotFound(e.to_string()))?;

        // 2. + Filtro de extensões (BooleanQuery)
        let final_query: Box<dyn Query> = if query.extensions.is_empty() {
            main_query
        } else {
            let ext_clauses: Vec<(Occur, Box<dyn Query>)> = query
                .extensions
                .iter()
                .map(|e| {
                    let term = Term::from_field_text(schema.f_ext, &e.to_lowercase());
                    (
                        Occur::Should,
                        Box::new(TermQuery::new(term, IndexRecordOption::Basic)) as Box<dyn Query>,
                    )
                })
                .collect();
            let ext_query = BooleanQuery::new(ext_clauses);
            Box::new(BooleanQuery::new(vec![
                (Occur::Must, main_query),
                (Occur::Must, Box::new(ext_query) as Box<dyn Query>),
            ]))
        };

        // 3. Execução
        let reader = index.reader()?;
        let searcher = reader.searcher();
        let top_docs = searcher.search(&final_query, &TopDocs::with_limit(query.max_results * 2))?;

        let query_terms: Vec<String> = query
            .query
            .split_whitespace()
            .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase())
            .filter(|s| !s.is_empty())
            .collect();

        let mut results: Vec<ContentSearchResult> = Vec::new();
        for (score, addr) in top_docs {
            let doc: TantivyDocument = searcher.doc(addr)?;

            let path = doc
                .get_first(schema.f_path)
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();

            // 2b. Filtro por root_path (pós-filtro no campo raw).
            if let Some(root) = &query.root_path {
                if !path.starts_with(root.as_str()) {
                    continue;
                }
            }

            let name = doc
                .get_first(schema.f_name)
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let extension = doc
                .get_first(schema.f_ext)
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());
            let size = doc.get_first(schema.f_size).and_then(|v| v.as_u64()).unwrap_or(0);
            let modified = doc
                .get_first(schema.f_modified)
                .and_then(|v| v.as_u64())
                .and_then(ts_to_iso);

            let parent_path = Path::new(&path)
                .parent()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_default();

            // 5. Previews (reler arquivo; falha não aborta).
            let mut previews = Vec::new();
            let mut match_count = 0usize;
            if query.include_previews {
                if let Ok(content) = read_file_content(Path::new(&path)) {
                    let terms: Vec<&str> = query_terms.iter().map(|s| s.as_str()).collect();
                    previews = extract_text_preview(&content.text, &terms, query.preview_chars);
                    let lower = content.text.to_lowercase();
                    for t in &query_terms {
                        match_count += lower.matches(t.as_str()).count();
                    }
                }
            }

            results.push(ContentSearchResult {
                path,
                name,
                extension,
                size,
                modified,
                score,
                previews,
                match_count,
                parent_path,
            });

            if results.len() >= query.max_results {
                break;
            }
        }

        // 6. Já vem ordenado por score do TopDocs; garante desc.
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        Ok(results)
    }

    pub fn get_term_frequency(&self, term: &str, index: &Index, schema: &IndexSchema) -> u64 {
        let result = (|| -> Result<u64, AppError> {
            let reader = index.reader()?;
            let searcher = reader.searcher();
            let t = Term::from_field_text(schema.f_content, &term.to_lowercase());
            let q = TermQuery::new(t, IndexRecordOption::Basic);
            let count = searcher.search(&q, &Count)?;
            Ok(count as u64)
        })();
        result.unwrap_or(0)
    }
}
