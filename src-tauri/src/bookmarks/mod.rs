use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::Manager;

use crate::error::AppError;

const VERSION: u8 = 1;
const MAX_NAME_LEN: usize = 64;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Bookmark {
    pub id: String,
    pub path: String,
    pub name: String,
    pub icon: Option<String>,
    pub created_at: String,
    pub position: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BookmarksFile {
    pub version: u8,
    pub bookmarks: Vec<Bookmark>,
}

/// Último segmento de um path (cross-platform).
fn basename(path: &str) -> String {
    let sep = if path.contains('\\') { '\\' } else { '/' };
    path.split(sep)
        .filter(|s| !s.is_empty())
        .last()
        .unwrap_or(path)
        .to_string()
}

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// Caminho do bookmarks.json no diretório de dados do app.
/// Cria o diretório se ainda não existir.
fn get_bookmarks_path(app_handle: &tauri::AppHandle) -> PathBuf {
    let dir = app_handle
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."));
    let _ = fs::create_dir_all(&dir);
    dir.join("bookmarks.json")
}

pub fn load_bookmarks(app_handle: &tauri::AppHandle) -> Result<Vec<Bookmark>, AppError> {
    let path = get_bookmarks_path(app_handle);

    if !path.exists() {
        return Ok(Vec::new());
    }

    let raw = fs::read_to_string(&path)?;
    let mut list = match serde_json::from_str::<BookmarksFile>(&raw) {
        Ok(file) => file.bookmarks,
        Err(e) => {
            log::warn!("bookmarks.json inválido ({e}); ignorando conteúdo");
            Vec::new()
        }
    };

    list.sort_by_key(|b| b.position);
    Ok(list)
}

pub fn save_bookmarks(
    app_handle: &tauri::AppHandle,
    bookmarks: Vec<Bookmark>,
) -> Result<(), AppError> {
    let path = get_bookmarks_path(app_handle);
    let file = BookmarksFile {
        version: VERSION,
        bookmarks,
    };
    let json = serde_json::to_string_pretty(&file)?;

    // Escrita atômica: grava no .tmp e renomeia.
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, json)?;
    fs::rename(&tmp, &path)?;
    Ok(())
}

pub fn add_bookmark(
    app_handle: &tauri::AppHandle,
    path: String,
    name: Option<String>,
) -> Result<Bookmark, AppError> {
    let p = Path::new(&path);
    if !p.exists() || !p.is_dir() {
        return Err(AppError::Invalid(format!(
            "path não é um diretório válido: {path}"
        )));
    }

    let mut list = load_bookmarks(app_handle)?;
    if list.iter().any(|b| b.path == path) {
        return Err(AppError::Invalid(format!("bookmark já existe: {path}")));
    }

    let position = list.iter().map(|b| b.position).max().map_or(0, |m| m + 1);
    let bookmark = Bookmark {
        id: uuid::Uuid::new_v4().to_string(),
        name: name.unwrap_or_else(|| basename(&path)),
        path,
        icon: None,
        created_at: now_iso(),
        position,
    };

    list.push(bookmark.clone());
    save_bookmarks(app_handle, list)?;
    Ok(bookmark)
}

pub fn remove_bookmark(app_handle: &tauri::AppHandle, id: String) -> Result<(), AppError> {
    let mut list = load_bookmarks(app_handle)?;
    let before = list.len();
    list.retain(|b| b.id != id);
    if list.len() == before {
        return Err(AppError::NotFound(format!("bookmark: {id}")));
    }

    // Renumera positions para fechar lacunas.
    list.sort_by_key(|b| b.position);
    for (i, b) in list.iter_mut().enumerate() {
        b.position = i as u32;
    }

    save_bookmarks(app_handle, list)?;
    Ok(())
}

pub fn reorder_bookmarks(
    app_handle: &tauri::AppHandle,
    ordered_ids: Vec<String>,
) -> Result<Vec<Bookmark>, AppError> {
    let mut list = load_bookmarks(app_handle)?;

    // Aplica a nova position conforme o índice na lista recebida.
    for (idx, id) in ordered_ids.iter().enumerate() {
        if let Some(b) = list.iter_mut().find(|b| &b.id == id) {
            b.position = idx as u32;
        }
    }

    list.sort_by_key(|b| b.position);
    save_bookmarks(app_handle, list.clone())?;
    Ok(list)
}

pub fn rename_bookmark(
    app_handle: &tauri::AppHandle,
    id: String,
    new_name: String,
) -> Result<Bookmark, AppError> {
    let trimmed = new_name.trim();
    if trimmed.is_empty() {
        return Err(AppError::Invalid("nome não pode ser vazio".into()));
    }
    if trimmed.chars().count() > MAX_NAME_LEN {
        return Err(AppError::Invalid(format!(
            "nome excede {MAX_NAME_LEN} caracteres"
        )));
    }

    let mut list = load_bookmarks(app_handle)?;
    let updated = {
        let b = list
            .iter_mut()
            .find(|b| b.id == id)
            .ok_or_else(|| AppError::NotFound(format!("bookmark: {id}")))?;
        b.name = trimmed.to_string();
        b.clone()
    };

    save_bookmarks(app_handle, list)?;
    Ok(updated)
}

pub fn set_bookmark_icon(
    app_handle: &tauri::AppHandle,
    id: String,
    icon: Option<String>,
) -> Result<Bookmark, AppError> {
    let mut list = load_bookmarks(app_handle)?;
    let updated = {
        let b = list
            .iter_mut()
            .find(|b| b.id == id)
            .ok_or_else(|| AppError::NotFound(format!("bookmark: {id}")))?;
        b.icon = icon;
        b.clone()
    };

    save_bookmarks(app_handle, list)?;
    Ok(updated)
}

pub fn bookmark_exists(app_handle: &tauri::AppHandle, path: &str) -> bool {
    load_bookmarks(app_handle)
        .map(|list| list.iter().any(|b| b.path == path))
        .unwrap_or(false)
}
