pub mod background;
pub mod file_reader;
pub mod schema;
pub mod searcher;
pub mod walker;
pub mod watcher;

#[cfg(test)]
mod walker_tests;

use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tantivy::collector::TopDocs;
use tantivy::query::TermQuery;
use tantivy::schema::{IndexRecordOption, Value};
use tantivy::{Index, IndexWriter, TantivyDocument, Term};
use tauri::Manager;

use crate::error::AppError;
use crate::fs_engine::ignore_rules::IgnoreRules;
use file_reader::{is_indexable, read_file_content};
use schema::{build_schema, IndexSchema};
use walker::{find_gitignore_files, AppWalker};

const WRITER_HEAP: usize = 15_000_000;
const MAX_INDEX_FILES: usize = 50_000;

impl From<tantivy::TantivyError> for AppError {
    fn from(e: tantivy::TantivyError) -> Self {
        AppError::Index(e.to_string())
    }
}

pub struct FileIndexer {
    index: Index,
    schema: IndexSchema,
    index_path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IndexStats {
    pub total_documents: u64,
    pub index_size_bytes: u64,
    pub last_updated: Option<String>,
    pub is_indexing: bool,
}

#[derive(Debug, Clone)]
pub struct IndexProgress {
    pub total: usize,
    pub processed: usize,
    pub current_file: String,
    pub errors: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GitignoreInfo {
    pub is_git_repo: bool,
    pub gitignore_files: Vec<String>,
    pub ignored_count: u64,
    pub active: bool,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct IndexMeta {
    last_updated: Option<String>,
}

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn unix_secs(t: SystemTime) -> u64 {
    t.duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

impl FileIndexer {
    pub fn new(app_handle: &tauri::AppHandle) -> Result<Self, AppError> {
        let base = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| AppError::Index(format!("app_data_dir: {e}")))?;
        Self::open_in(base.join("search_index"))
    }

    /// Abre/cria o índice num diretório específico (usado por new e testes).
    pub fn open_in(index_path: PathBuf) -> Result<Self, AppError> {
        std::fs::create_dir_all(&index_path)?;
        let schema = build_schema();
        let index = match Index::open_in_dir(&index_path) {
            Ok(i) => i,
            Err(_) => Index::create_in_dir(&index_path, schema.schema.clone())?,
        };
        Ok(Self {
            index,
            schema,
            index_path,
        })
    }

    pub fn index(&self) -> &Index {
        &self.index
    }
    pub fn schema(&self) -> &IndexSchema {
        &self.schema
    }

    fn meta_path(&self) -> PathBuf {
        self.index_path.join("metadata.json")
    }

    fn read_meta(&self) -> IndexMeta {
        std::fs::read_to_string(self.meta_path())
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    fn write_meta(&self, meta: &IndexMeta) {
        if let Ok(json) = serde_json::to_string_pretty(meta) {
            let _ = std::fs::write(self.meta_path(), json);
        }
    }

    pub fn get_stats(&self) -> Result<IndexStats, AppError> {
        let reader = self.index.reader()?;
        let total_documents = reader.searcher().num_docs();

        let mut index_size_bytes = 0u64;
        if let Ok(rd) = std::fs::read_dir(&self.index_path) {
            for entry in rd.filter_map(|e| e.ok()) {
                if let Ok(m) = entry.metadata() {
                    if m.is_file() {
                        index_size_bytes += m.len();
                    }
                }
            }
        }

        Ok(IndexStats {
            total_documents,
            index_size_bytes,
            last_updated: self.read_meta().last_updated,
            is_indexing: false,
        })
    }

    /// Modified timestamp já indexado para este path (se existir).
    fn indexed_modified(&self, path_str: &str) -> Result<Option<u64>, AppError> {
        let reader = self.index.reader()?;
        let searcher = reader.searcher();
        let term = Term::from_field_text(self.schema.f_path, path_str);
        let query = TermQuery::new(term, IndexRecordOption::Basic);
        let hits = searcher.search(&query, &TopDocs::with_limit(1))?;
        if let Some((_, addr)) = hits.first() {
            let doc: TantivyDocument = searcher.doc(*addr)?;
            let m = doc.get_first(self.schema.f_modified).and_then(|v| v.as_u64());
            return Ok(m);
        }
        Ok(None)
    }

    /// Adiciona/atualiza um documento usando um writer compartilhado
    /// (sem commit — o chamador faz um commit único).
    pub fn index_file_with_writer(
        &self,
        path: &Path,
        writer: &IndexWriter,
    ) -> Result<(), AppError> {
        if !is_indexable(path) {
            return Ok(());
        }

        let meta = std::fs::metadata(path)?;
        let size = meta.len();
        let modified = meta.modified().ok().map(unix_secs).unwrap_or(0);
        let path_str = path.to_string_lossy().into_owned();

        // Já indexado e sem mudança → não re-indexa.
        if let Ok(Some(prev)) = self.indexed_modified(&path_str) {
            if prev == modified {
                return Ok(());
            }
        }

        let content = read_file_content(path)?;
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path_str.clone());
        let ext = path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        // Remove versão antiga (se houver) antes de inserir.
        writer.delete_term(Term::from_field_text(self.schema.f_path, &path_str));

        let mut doc = TantivyDocument::default();
        doc.add_text(self.schema.f_path, &path_str);
        doc.add_text(self.schema.f_name, &name);
        doc.add_text(self.schema.f_content, &content.text);
        doc.add_text(self.schema.f_ext, &ext);
        doc.add_u64(self.schema.f_size, size);
        doc.add_u64(self.schema.f_modified, modified);
        doc.add_u64(self.schema.f_indexed, unix_secs(SystemTime::now()));

        writer.add_document(doc)?;
        Ok(())
    }

    pub fn index_file(&self, path: &Path) -> Result<(), AppError> {
        let mut writer: IndexWriter = self.index.writer(WRITER_HEAP)?;
        self.index_file_with_writer(path, &writer)?;
        writer.commit()?;
        Ok(())
    }

    pub fn delete_file(&self, path: &Path) -> Result<(), AppError> {
        let path_str = path.to_string_lossy().into_owned();
        let mut writer: IndexWriter = self.index.writer(WRITER_HEAP)?;
        writer.delete_term(Term::from_field_text(self.schema.f_path, &path_str));
        writer.commit()?;
        Ok(())
    }

    pub fn index_directory(
        &self,
        root_path: &str,
        rules: &IgnoreRules,
        respect_gitignore: bool,
        progress_tx: Sender<IndexProgress>,
    ) -> Result<(), AppError> {
        // FASE 1: coleta (respeita git + IgnoreRules do app).
        let walker = AppWalker::new(root_path, rules.clone()).respect_gitignore(respect_gitignore);

        let mut paths: Vec<PathBuf> = Vec::new();
        for entry in walker.walk().filter_map(|e| e.ok()) {
            if entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                let p = entry.path().to_path_buf();
                if is_indexable(&p) {
                    paths.push(p);
                    if paths.len() >= MAX_INDEX_FILES {
                        log::warn!(
                            "index_directory: limite de {MAX_INDEX_FILES} arquivos atingido em {root_path}"
                        );
                        break;
                    }
                }
            }
        }

        let total = paths.len();
        let mut errors = 0usize;

        // FASE 2: indexação com writer único + commit único ao final.
        let mut writer: IndexWriter = self.index.writer(WRITER_HEAP)?;
        for (i, p) in paths.iter().enumerate() {
            let _ = progress_tx.send(IndexProgress {
                total,
                processed: i,
                current_file: p.to_string_lossy().into_owned(),
                errors,
            });
            if let Err(e) = self.index_file_with_writer(p, &writer) {
                log::warn!("index_file falhou {}: {e}", p.display());
                errors += 1;
            }
        }
        writer.commit()?;

        self.write_meta(&IndexMeta {
            last_updated: Some(now_iso()),
        });

        let _ = progress_tx.send(IndexProgress {
            total,
            processed: total,
            current_file: String::new(),
            errors,
        });

        Ok(())
    }

    /// Info sobre gitignore para a UI.
    pub fn get_gitignore_info(&self, path: &str, active: bool) -> GitignoreInfo {
        let root = Path::new(path);
        let is_git_repo = root.join(".git").exists();

        let gitignore_files = find_gitignore_files(root, 4)
            .into_iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect();

        // Estimativa: arquivos que seriam ignorados (cap 1000).
        let mut ignored_count = 0u64;
        if is_git_repo {
            let with = AppWalker::new(root, IgnoreRules::default()).respect_gitignore(true);
            let without = AppWalker::new(root, IgnoreRules::default()).respect_gitignore(false);
            let count = |w: &AppWalker| -> u64 {
                w.walk()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
                    .take(1000)
                    .count() as u64
            };
            ignored_count = count(&without).saturating_sub(count(&with));
        }

        GitignoreInfo {
            is_git_repo,
            gitignore_files,
            ignored_count,
            active,
        }
    }

    pub fn clear_index(&self) -> Result<(), AppError> {
        let mut writer: IndexWriter = self.index.writer(WRITER_HEAP)?;
        writer.delete_all_documents()?;
        writer.commit()?;
        self.write_meta(&IndexMeta {
            last_updated: Some(now_iso()),
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::searcher::{ContentSearcher, ContentSearchQuery};
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_index() -> (FileIndexer, TempDir) {
        let dir = TempDir::new().unwrap();
        let idx = FileIndexer::open_in(dir.path().join("idx")).unwrap();
        (idx, dir)
    }

    fn write_file(dir: &std::path::Path, name: &str, content: &[u8]) -> PathBuf {
        let p = dir.join(name);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        let mut f = std::fs::File::create(&p).unwrap();
        f.write_all(content).unwrap();
        p
    }

    fn search(idx: &FileIndexer, q: &str, root: Option<String>) -> Vec<searcher::ContentSearchResult> {
        let query = ContentSearchQuery {
            query: q.to_string(),
            root_path: root,
            include_previews: false,
            ..Default::default()
        };
        ContentSearcher
            .search_content(&query, idx.index(), idx.schema())
            .unwrap()
    }

    #[test]
    fn test_index_and_search_file() {
        let (idx, dir) = create_test_index();
        let f = write_file(dir.path(), "main.rs", b"fn supercalifragilistic() {}");
        idx.index_file(&f).unwrap();
        let res = search(&idx, "supercalifragilistic", None);
        assert!(res.iter().any(|r| r.name == "main.rs"));
    }

    #[test]
    fn test_skip_non_indexable() {
        let (idx, dir) = create_test_index();
        let f = write_file(dir.path(), "app.exe", b"MZ\x00\x00binary");
        idx.index_file(&f).unwrap();
        assert_eq!(idx.get_stats().unwrap().total_documents, 0);
    }

    #[test]
    fn test_delete_removes_from_results() {
        let (idx, dir) = create_test_index();
        let f = write_file(dir.path(), "notes.txt", b"uniquetoken alpha");
        idx.index_file(&f).unwrap();
        assert!(!search(&idx, "uniquetoken", None).is_empty());
        idx.delete_file(&f).unwrap();
        assert!(search(&idx, "uniquetoken", None).is_empty());
    }

    #[test]
    fn test_reindex_updates_content() {
        let (idx, dir) = create_test_index();
        let f = write_file(dir.path(), "doc.md", b"firstcontent token");
        idx.index_file(&f).unwrap();
        // mtime precisa mudar (skip incremental compara segundos)
        std::thread::sleep(std::time::Duration::from_millis(1100));
        write_file(dir.path(), "doc.md", b"secondcontent token");
        idx.index_file(&f).unwrap();
        assert!(search(&idx, "secondcontent", None).iter().any(|r| r.name == "doc.md"));
        assert!(search(&idx, "firstcontent", None).is_empty());
    }

    #[test]
    fn test_large_file_truncation() {
        let (_idx, dir) = create_test_index();
        let big = vec![b'a'; (file_reader::MAX_FILE_SIZE_BYTES + 1) as usize];
        let f = write_file(dir.path(), "big.txt", &big);
        assert!(!file_reader::is_indexable(&f));
    }

    #[test]
    fn test_search_with_root_path_filter() {
        let (idx, dir) = create_test_index();
        let a = write_file(dir.path(), "alpha/file.txt", b"sharedterm here");
        let b = write_file(dir.path(), "beta/file.txt", b"sharedterm there");
        idx.index_file(&a).unwrap();
        idx.index_file(&b).unwrap();

        let root = dir.path().join("alpha").to_string_lossy().into_owned();
        let res = search(&idx, "sharedterm", Some(root));
        assert!(!res.is_empty());
        assert!(res.iter().all(|r| r.path.contains("alpha")));
        assert!(res.iter().all(|r| !r.path.contains("beta")));
    }
}
