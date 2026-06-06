use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant};

use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tauri::Emitter;

use crate::error::AppError;
use crate::fs_engine::ignore_rules::IgnoreRules;
use crate::indexer::walker::is_path_gitignored;
use crate::indexer::FileIndexer;

const DEBOUNCE: Duration = Duration::from_millis(300);
const DRAIN_TIMEOUT: Duration = Duration::from_millis(50);

pub struct IndexWatcher {
    watcher: RecommendedWatcher,
    watched_paths: Vec<PathBuf>,
    root_paths: Arc<RwLock<Vec<PathBuf>>>,
}

fn to_app_err(e: notify::Error) -> AppError {
    AppError::Index(e.to_string())
}

/// Raiz de repo git mais próxima de `path` entre as raízes observadas.
fn git_root_for(path: &Path, roots: &[PathBuf]) -> Option<PathBuf> {
    roots
        .iter()
        .filter(|r| path.starts_with(r))
        .max_by_key(|r| r.components().count())
        .cloned()
}

impl IndexWatcher {
    pub fn new(
        indexer: Arc<Mutex<FileIndexer>>,
        rules: Arc<RwLock<IgnoreRules>>,
        respect_gitignore: bool,
        app_handle: tauri::AppHandle,
    ) -> Result<Self, AppError> {
        let (tx, rx) = mpsc::channel::<notify::Result<notify::Event>>();

        let watcher = notify::recommended_watcher(move |res| {
            let _ = tx.send(res);
        })
        .map_err(to_app_err)?;

        let root_paths: Arc<RwLock<Vec<PathBuf>>> = Arc::new(RwLock::new(Vec::new()));
        let roots_for_thread = Arc::clone(&root_paths);

        // Thread de processamento com debounce.
        thread::spawn(move || {
            let mut pending: HashMap<PathBuf, EventKind> = HashMap::new();
            let mut last_flush = Instant::now();

            loop {
                match rx.recv_timeout(DRAIN_TIMEOUT) {
                    Ok(Ok(event)) => {
                        for p in event.paths.iter() {
                            pending.insert(p.clone(), event.kind);
                        }
                    }
                    Ok(Err(_)) => {}
                    Err(RecvTimeoutError::Timeout) => {}
                    Err(RecvTimeoutError::Disconnected) => break,
                }

                if last_flush.elapsed() >= DEBOUNCE && !pending.is_empty() {
                    let batch: Vec<(PathBuf, EventKind)> = pending.drain().collect();
                    let roots = roots_for_thread.read().map(|r| r.clone()).unwrap_or_default();
                    for (path, kind) in batch {
                        handle_event(&path, kind, &indexer, &rules, respect_gitignore, &roots);
                        let _ = app_handle.emit("index:file-updated", path.to_string_lossy().to_string());
                    }
                    last_flush = Instant::now();
                }
            }
        });

        Ok(Self {
            watcher,
            watched_paths: Vec::new(),
            root_paths,
        })
    }

    pub fn watch(&mut self, path: &Path) -> Result<(), AppError> {
        self.watcher
            .watch(path, RecursiveMode::Recursive)
            .map_err(to_app_err)?;
        self.watched_paths.push(path.to_path_buf());
        if let Ok(mut roots) = self.root_paths.write() {
            roots.push(path.to_path_buf());
        }
        Ok(())
    }

    pub fn unwatch(&mut self, path: &Path) -> Result<(), AppError> {
        self.watcher.unwatch(path).map_err(to_app_err)?;
        self.watched_paths.retain(|p| p != path);
        if let Ok(mut roots) = self.root_paths.write() {
            roots.retain(|p| p != path);
        }
        Ok(())
    }

    pub fn watched_paths(&self) -> &[PathBuf] {
        &self.watched_paths
    }
}

fn handle_event(
    path: &Path,
    kind: EventKind,
    indexer: &Arc<Mutex<FileIndexer>>,
    rules: &Arc<RwLock<IgnoreRules>>,
    respect_gitignore: bool,
    roots: &[PathBuf],
) {
    match kind {
        EventKind::Remove(_) => {
            if let Ok(idx) = indexer.lock() {
                let _ = idx.delete_file(path);
            }
        }
        EventKind::Create(_) | EventKind::Modify(_) => {
            if path.exists() {
                if path.is_dir() {
                    // Reindexa o diretório numa thread (descarta progresso).
                    let idx = Arc::clone(indexer);
                    let rl = rules.read().map(|r| r.clone()).unwrap_or_default();
                    let p = path.to_path_buf();
                    thread::spawn(move || {
                        if let Ok(indexer) = idx.lock() {
                            let (tx, _rx) = mpsc::channel();
                            let _ = indexer.index_directory(&p.to_string_lossy(), &rl, respect_gitignore, tx);
                        }
                    });
                } else {
                    // Se gitignore ativo e o arquivo virou ignorado → remove.
                    let ignored = respect_gitignore
                        && git_root_for(path, roots)
                            .map(|root| is_path_gitignored(path, &root))
                            .unwrap_or(false);
                    if let Ok(indexer) = indexer.lock() {
                        if ignored {
                            let _ = indexer.delete_file(path);
                        } else {
                            let _ = indexer.index_file(path);
                        }
                    }
                }
            } else if let Ok(indexer) = indexer.lock() {
                // Sumiu (ex: rename origem) → remove do índice.
                let _ = indexer.delete_file(path);
            }
        }
        _ => {}
    }
}
