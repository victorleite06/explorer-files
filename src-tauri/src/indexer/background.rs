use std::sync::mpsc;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use tauri::Emitter;

use crate::fs_engine::ignore_rules::IgnoreRules;
use crate::indexer::{FileIndexer, IndexProgress};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexingProgressEvent {
    pub session_id: String,
    pub total: usize,
    pub processed: usize,
    pub current_file: String,
    pub errors: usize,
    pub is_complete: bool,
    pub eta_seconds: Option<u32>,
}

fn extract_filename(path: &str) -> String {
    if path.is_empty() {
        return String::new();
    }
    let sep = if path.contains('\\') { '\\' } else { '/' };
    path.rsplit(sep).find(|s| !s.is_empty()).unwrap_or(path).to_string()
}

pub fn start_background_indexing(
    root_path: String,
    indexer: Arc<Mutex<FileIndexer>>,
    rules: Arc<RwLock<IgnoreRules>>,
    app_handle: tauri::AppHandle,
) -> String {
    let session_id = uuid::Uuid::new_v4().to_string();
    let session_for_thread = session_id.clone();

    thread::spawn(move || {
        let (tx, rx) = mpsc::channel::<IndexProgress>();
        let start_time = Instant::now();

        // Indexação numa thread separada.
        let indexer_clone = Arc::clone(&indexer);
        let rules_read = rules.read().map(|r| r.clone()).unwrap_or_default();
        let path_clone = root_path.clone();
        let tx_clone = tx.clone();
        thread::spawn(move || {
            if let Ok(idx) = indexer_clone.lock() {
                let _ = idx.index_directory(&path_clone, &rules_read, tx_clone);
            }
        });
        drop(tx); // fecha o canal quando a thread de indexação terminar

        for progress in rx {
            let elapsed = start_time.elapsed().as_secs_f64();
            let rate = if elapsed > 0.0 {
                progress.processed as f64 / elapsed
            } else {
                1.0
            };
            let remaining = progress.total.saturating_sub(progress.processed);
            let eta = if rate > 0.0 {
                Some((remaining as f64 / rate) as u32)
            } else {
                None
            };

            let is_complete = progress.processed >= progress.total;
            let event = IndexingProgressEvent {
                session_id: session_for_thread.clone(),
                total: progress.total,
                processed: progress.processed,
                current_file: extract_filename(&progress.current_file),
                errors: progress.errors,
                is_complete,
                eta_seconds: eta,
            };

            let _ = app_handle.emit("index:progress", &event);

            if is_complete {
                break;
            }
        }
    });

    session_id
}
