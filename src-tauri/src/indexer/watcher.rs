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
use crate::indexer::FileIndexer;

const DEBOUNCE: Duration = Duration::from_millis(300);
const DRAIN_TIMEOUT: Duration = Duration::from_millis(50);

pub struct IndexWatcher {
    watcher: RecommendedWatcher,
    watched_paths: Vec<PathBuf>,
}

fn to_app_err(e: notify::Error) -> AppError {
    AppError::Index(e.to_string())
}

impl IndexWatcher {
    pub fn new(
        indexer: Arc<Mutex<FileIndexer>>,
        rules: Arc<RwLock<IgnoreRules>>,
        app_handle: tauri::AppHandle,
    ) -> Result<Self, AppError> {
        let (tx, rx) = mpsc::channel::<notify::Result<notify::Event>>();

        let watcher = notify::recommended_watcher(move |res| {
            let _ = tx.send(res);
        })
        .map_err(to_app_err)?;

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
                    for (path, kind) in batch {
                        handle_event(&path, kind, &indexer, &rules);
                        let _ = app_handle.emit("index:file-updated", path.to_string_lossy().to_string());
                    }
                    last_flush = Instant::now();
                }
            }
        });

        Ok(Self {
            watcher,
            watched_paths: Vec::new(),
        })
    }

    pub fn watch(&mut self, path: &Path) -> Result<(), AppError> {
        self.watcher
            .watch(path, RecursiveMode::Recursive)
            .map_err(to_app_err)?;
        self.watched_paths.push(path.to_path_buf());
        Ok(())
    }

    pub fn unwatch(&mut self, path: &Path) -> Result<(), AppError> {
        self.watcher.unwatch(path).map_err(to_app_err)?;
        self.watched_paths.retain(|p| p != path);
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
                            let _ = indexer.index_directory(&p.to_string_lossy(), &rl, tx);
                        }
                    });
                } else if let Ok(indexer) = indexer.lock() {
                    let _ = indexer.index_file(path);
                }
            } else if let Ok(indexer) = indexer.lock() {
                // Sumiu (ex: rename origem) → remove do índice.
                let _ = indexer.delete_file(path);
            }
        }
        _ => {}
    }
}
