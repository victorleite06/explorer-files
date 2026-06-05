mod bookmarks;
mod error;
mod fs_engine;
mod git_engine;
mod indexer;
mod search;
mod settings;

use std::sync::{Arc, Mutex, RwLock};

use tauri::{Manager, State};

use bookmarks::Bookmark;
use fs_engine::ignore_rules::IgnoreRules;
use fs_engine::{DirectorySummary, FileEntry, TreeNode};
use indexer::background;
use indexer::searcher::{ContentSearchQuery, ContentSearchResult, ContentSearcher};
use indexer::watcher::IndexWatcher;
use indexer::{FileIndexer, IndexStats};
use search::{SearchOptions, SearchResult};
use settings::AppSettings;

/// Estado compartilhado: settings + indexador + watcher.
pub struct AppState {
    pub settings: Mutex<AppSettings>,
    pub indexer: Arc<Mutex<FileIndexer>>,
    pub watcher: Mutex<Option<IndexWatcher>>,
}

impl AppState {
    pub fn new(app_handle: &tauri::AppHandle) -> Self {
        let settings = settings::load_settings(app_handle);
        let indexer = FileIndexer::new(app_handle).expect("Falha ao inicializar indexador");
        Self {
            settings: Mutex::new(settings),
            indexer: Arc::new(Mutex::new(indexer)),
            watcher: Mutex::new(None),
        }
    }
}

fn current_rules(state: &State<'_, AppState>) -> IgnoreRules {
    state.settings.lock().unwrap().ignore_rules.clone()
}

#[tauri::command]
async fn list_directory(
    state: State<'_, AppState>,
    path: String,
) -> Result<Vec<FileEntry>, String> {
    let rules = current_rules(&state);
    fs_engine::list_directory(&path, &rules).map_err(Into::into)
}

#[tauri::command]
async fn get_directory_tree(
    state: State<'_, AppState>,
    path: String,
    depth: u8,
) -> Result<TreeNode, String> {
    let rules = current_rules(&state);
    fs_engine::get_directory_tree(&path, depth, &rules).map_err(Into::into)
}

#[tauri::command]
async fn get_home_directory() -> Option<String> {
    fs_engine::get_home_directory()
}

#[tauri::command]
async fn get_drives() -> Vec<FileEntry> {
    fs_engine::get_drives()
}

#[tauri::command]
async fn scan_directory_summary(
    state: State<'_, AppState>,
    path: String,
) -> Result<DirectorySummary, String> {
    let rules = current_rules(&state);
    fs_engine::scan_directory_summary(&path, &rules).map_err(Into::into)
}

// ── Settings ────────────────────────────────────────────────────
#[tauri::command]
async fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    Ok(state.settings.lock().unwrap().clone())
}

#[tauri::command]
async fn update_ignore_rules(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
    rules: IgnoreRules,
) -> Result<AppSettings, String> {
    let updated = settings::update_ignore_rules(&app_handle, rules).map_err(|e| e.to_string())?;
    *state.settings.lock().unwrap() = updated.clone();
    Ok(updated)
}

#[tauri::command]
async fn get_file_metadata(path: String) -> Result<FileEntry, String> {
    fs_engine::get_file_metadata(&path).map_err(Into::into)
}

#[tauri::command]
async fn get_git_status(path: String) -> Result<serde_json::Value, String> {
    // TODO: real status from git_engine. Placeholder empty map.
    let _ = path;
    Ok(serde_json::json!({}))
}

#[tauri::command]
async fn search_files(
    state: State<'_, AppState>,
    query: String,
    path: String,
    options: Option<SearchOptions>,
) -> Result<Vec<SearchResult>, String> {
    let rules = current_rules(&state);
    let opts = options.unwrap_or_default();
    search::search_by_name(&query, &path, &opts, &rules).map_err(Into::into)
}

#[tauri::command]
async fn search_files_quick(
    state: State<'_, AppState>,
    query: String,
    path: String,
) -> Result<Vec<SearchResult>, String> {
    let rules = current_rules(&state);
    let opts = SearchOptions {
        max_results: 10,
        max_depth: 3,
        recursive: true,
        ..Default::default()
    };
    search::search_by_name(&query, &path, &opts, &rules).map_err(Into::into)
}

// ── Indexação de conteúdo ───────────────────────────────────────
#[tauri::command]
async fn get_index_stats(state: State<'_, AppState>) -> Result<IndexStats, String> {
    state
        .indexer
        .lock()
        .unwrap()
        .get_stats()
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn start_indexing(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> Result<String, String> {
    let rules = Arc::new(RwLock::new(current_rules(&state)));
    let session = background::start_background_indexing(
        path,
        Arc::clone(&state.indexer),
        rules,
        app_handle,
    );
    Ok(session)
}

#[tauri::command]
async fn watch_directory(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> Result<(), String> {
    let mut guard = state.watcher.lock().unwrap();
    if guard.is_none() {
        let rules = Arc::new(RwLock::new(current_rules(&state)));
        let w = IndexWatcher::new(Arc::clone(&state.indexer), rules, app_handle)
            .map_err(|e| e.to_string())?;
        *guard = Some(w);
    }
    guard
        .as_mut()
        .unwrap()
        .watch(std::path::Path::new(&path))
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn unwatch_directory(state: State<'_, AppState>, path: String) -> Result<(), String> {
    let mut guard = state.watcher.lock().unwrap();
    if let Some(w) = guard.as_mut() {
        w.unwatch(std::path::Path::new(&path)).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn search_content(
    state: State<'_, AppState>,
    query: ContentSearchQuery,
) -> Result<Vec<ContentSearchResult>, String> {
    let idx = state.indexer.lock().unwrap();
    ContentSearcher
        .search_content(&query, idx.index(), idx.schema())
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn clear_index(state: State<'_, AppState>) -> Result<(), String> {
    state
        .indexer
        .lock()
        .unwrap()
        .clear_index()
        .map_err(|e| e.to_string())
}

// ── Bookmarks ───────────────────────────────────────────────────
#[tauri::command]
async fn get_bookmarks(app_handle: tauri::AppHandle) -> Result<Vec<Bookmark>, String> {
    bookmarks::load_bookmarks(&app_handle).map_err(Into::into)
}

#[tauri::command]
async fn add_bookmark(
    app_handle: tauri::AppHandle,
    path: String,
    name: Option<String>,
) -> Result<Bookmark, String> {
    bookmarks::add_bookmark(&app_handle, path, name).map_err(Into::into)
}

#[tauri::command]
async fn remove_bookmark(app_handle: tauri::AppHandle, id: String) -> Result<(), String> {
    bookmarks::remove_bookmark(&app_handle, id).map_err(Into::into)
}

#[tauri::command]
async fn reorder_bookmarks(
    app_handle: tauri::AppHandle,
    ordered_ids: Vec<String>,
) -> Result<Vec<Bookmark>, String> {
    bookmarks::reorder_bookmarks(&app_handle, ordered_ids).map_err(Into::into)
}

#[tauri::command]
async fn rename_bookmark(
    app_handle: tauri::AppHandle,
    id: String,
    new_name: String,
) -> Result<Bookmark, String> {
    bookmarks::rename_bookmark(&app_handle, id, new_name).map_err(Into::into)
}

#[tauri::command]
async fn set_bookmark_icon(
    app_handle: tauri::AppHandle,
    id: String,
    icon: Option<String>,
) -> Result<Bookmark, String> {
    bookmarks::set_bookmark_icon(&app_handle, id, icon).map_err(Into::into)
}

#[tauri::command]
async fn bookmark_exists(app_handle: tauri::AppHandle, path: String) -> bool {
    bookmarks::bookmark_exists(&app_handle, &path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Inicializa estado: settings + indexador + watcher.
            app.manage(AppState::new(&app.handle()));

            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_directory,
            get_directory_tree,
            get_home_directory,
            get_drives,
            scan_directory_summary,
            get_file_metadata,
            get_settings,
            update_ignore_rules,
            get_git_status,
            search_files,
            search_files_quick,
            get_index_stats,
            start_indexing,
            watch_directory,
            unwatch_directory,
            search_content,
            clear_index,
            get_bookmarks,
            add_bookmark,
            remove_bookmark,
            reorder_bookmarks,
            rename_bookmark,
            set_bookmark_icon,
            bookmark_exists
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
