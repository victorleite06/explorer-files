use serde::{Deserialize, Serialize};

use super::FileEntry;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IgnoreRules {
    pub show_dotfiles: bool,
    pub show_node_modules: bool,
    pub show_build_artifacts: bool,
    pub custom_hidden: Vec<String>,
    pub custom_shown: Vec<String>,
}

impl Default for IgnoreRules {
    fn default() -> Self {
        Self {
            show_dotfiles: false,
            show_node_modules: false,
            show_build_artifacts: false,
            custom_hidden: vec![],
            custom_shown: vec![],
        }
    }
}

pub const DOTFILE_PATTERN: &str = ".";

pub const NODE_MODULES_NAMES: &[&str] = &["node_modules"];

pub const BUILD_ARTIFACT_NAMES: &[&str] = &[
    "target",
    "dist",
    "build",
    "out",
    ".next",
    ".nuxt",
    ".svelte-kit",
    "__pycache__",
    ".pytest_cache",
    "venv",
    ".venv",
    "vendor",
    ".gradle",
    ".m2",
    "bin",
    "obj",
    ".dart_tool",
    ".flutter-plugins",
];

pub const ALWAYS_HIDDEN_NAMES: &[&str] = &[
    ".DS_Store",
    "Thumbs.db",
    "desktop.ini",
    ".Spotlight-V100",
    ".Trashes",
    "$RECYCLE.BIN",
    "System Volume Information",
    ".fseventsd",
    ".TemporaryItems",
];

// Comparação de nomes: case-sensitive no Unix, case-insensitive no Windows.
#[cfg(target_os = "windows")]
fn name_eq(a: &str, b: &str) -> bool {
    a.to_lowercase() == b.to_lowercase()
}
#[cfg(not(target_os = "windows"))]
fn name_eq(a: &str, b: &str) -> bool {
    a == b
}

fn in_static(name: &str, list: &[&str]) -> bool {
    list.iter().any(|x| name_eq(name, x))
}
fn in_owned(name: &str, list: &[String]) -> bool {
    list.iter().any(|x| name_eq(name, x))
}

/// Decide se uma entrada deve ser ocultada (true = esconder).
pub fn should_hide(name: &str, is_dir: bool, rules: &IgnoreRules) -> bool {
    // 1. Sempre ocultar
    if in_static(name, ALWAYS_HIDDEN_NAMES) {
        return true;
    }
    // 2. Exceções explícitas do usuário (nunca esconde)
    if in_owned(name, &rules.custom_shown) {
        return false;
    }
    // 3. Regras de ocultação do usuário
    if in_owned(name, &rules.custom_hidden) {
        return true;
    }
    // 4. node_modules
    if !rules.show_node_modules && is_dir && in_static(name, NODE_MODULES_NAMES) {
        return true;
    }
    // 5. Artefatos de build
    if !rules.show_build_artifacts && is_dir && in_static(name, BUILD_ARTIFACT_NAMES) {
        return true;
    }
    // 6. Dotfiles
    if !rules.show_dotfiles && name.starts_with(DOTFILE_PATTERN) {
        return true;
    }
    // 7. Mostrar
    false
}

pub fn should_hide_entry(entry: &FileEntry, rules: &IgnoreRules) -> bool {
    should_hide(&entry.name, entry.is_dir, rules)
}
