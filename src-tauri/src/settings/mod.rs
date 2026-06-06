use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::Manager;

use crate::error::AppError;
use crate::fs_engine::ignore_rules::IgnoreRules;

const VERSION: u8 = 1;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GitignoreSettings {
    pub respect_gitignore: bool,
    pub respect_global_gitignore: bool,
    pub respect_ignore_files: bool,
}

impl Default for GitignoreSettings {
    fn default() -> Self {
        Self {
            respect_gitignore: true,
            respect_global_gitignore: true,
            respect_ignore_files: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub version: u8,
    pub ignore_rules: IgnoreRules,
    #[serde(default)]
    pub gitignore: GitignoreSettings,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            version: VERSION,
            ignore_rules: IgnoreRules::default(),
            gitignore: GitignoreSettings::default(),
        }
    }
}

fn get_settings_path(app_handle: &tauri::AppHandle) -> PathBuf {
    let dir = app_handle
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."));
    let _ = fs::create_dir_all(&dir);
    dir.join("settings.json")
}

/// Carrega settings; nunca falha (default em ausência/erro).
pub fn load_settings(app_handle: &tauri::AppHandle) -> AppSettings {
    let path = get_settings_path(app_handle);
    let raw = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => return AppSettings::default(),
    };
    match serde_json::from_str::<AppSettings>(&raw) {
        Ok(s) => s,
        Err(e) => {
            log::warn!("settings.json inválido ({e}); usando padrão");
            AppSettings::default()
        }
    }
}

pub fn save_settings(app_handle: &tauri::AppHandle, settings: &AppSettings) -> Result<(), AppError> {
    let path = get_settings_path(app_handle);
    let json = serde_json::to_string_pretty(settings)?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, json)?;
    fs::rename(&tmp, &path)?;
    Ok(())
}

pub fn update_ignore_rules(
    app_handle: &tauri::AppHandle,
    rules: IgnoreRules,
) -> Result<AppSettings, AppError> {
    let mut settings = load_settings(app_handle);
    settings.ignore_rules = rules;
    save_settings(app_handle, &settings)?;
    Ok(settings)
}

pub fn update_gitignore_settings(
    app_handle: &tauri::AppHandle,
    gitignore: GitignoreSettings,
) -> Result<AppSettings, AppError> {
    let mut settings = load_settings(app_handle);
    settings.gitignore = gitignore;
    save_settings(app_handle, &settings)?;
    Ok(settings)
}
