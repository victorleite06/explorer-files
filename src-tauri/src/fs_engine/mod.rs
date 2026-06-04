use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::SystemTime;

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::error::AppError;

pub mod ignore_rules;
use ignore_rules::{should_hide, should_hide_entry, IgnoreRules};

const MAX_DEPTH: u8 = 5;
const MAX_EXTENSIONS: usize = 50;
const NO_EXT: &str = "__no_ext__";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<String>,
    pub created_at: Option<String>,
    pub extension: Option<String>,
    pub is_hidden: bool,
    pub is_symlink: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TreeNode {
    pub name: String,
    pub path: String,
    pub children: Vec<TreeNode>,
    pub is_expanded: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExtensionSummary {
    pub extension: String,
    pub count: u32,
    pub total_size: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DirectorySummary {
    pub total_files: u32,
    pub total_dirs: u32,
    pub total_size: u64,
    pub smallest_file: u64,
    pub largest_file: u64,
    pub oldest_modified: Option<String>,
    pub newest_modified: Option<String>,
    pub extensions: Vec<ExtensionSummary>,
}

/// Format a SystemTime as an ISO 8601 / RFC 3339 string.
fn to_iso8601(time: SystemTime) -> Option<String> {
    let dur = time.duration_since(SystemTime::UNIX_EPOCH).ok()?;
    Some(chrono::DateTime::<chrono::Utc>::from(SystemTime::UNIX_EPOCH + dur).to_rfc3339())
}

#[cfg(target_os = "windows")]
fn is_hidden(name: &str, meta: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
    meta.file_attributes() & FILE_ATTRIBUTE_HIDDEN != 0 || name.starts_with('.')
}

#[cfg(not(target_os = "windows"))]
fn is_hidden(name: &str, _meta: &fs::Metadata) -> bool {
    name.starts_with('.')
}

/// Constrói um FileEntry completo para um único path.
/// file_type via symlink_metadata (não segue link); tamanho/datas via
/// metadata (segue link), com fallback para symlink_metadata.
fn get_entry_metadata(path: &Path) -> Result<FileEntry, AppError> {
    let sym = fs::symlink_metadata(path)?;
    let is_symlink = sym.file_type().is_symlink();
    let meta = fs::metadata(path).unwrap_or(sym);

    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned());

    let hidden = is_hidden(&name, &meta);
    let is_dir = meta.is_dir();

    Ok(FileEntry {
        name,
        path: path.to_string_lossy().into_owned(),
        is_dir,
        size: if is_dir { 0 } else { meta.len() },
        modified: meta.modified().ok().and_then(to_iso8601),
        created_at: meta.created().ok().and_then(to_iso8601),
        extension: if is_dir {
            None
        } else {
            path.extension().map(|e| e.to_string_lossy().into_owned())
        },
        is_hidden: hidden,
        is_symlink,
    })
}

/// List immediate children of `path`. Permission errors on individual entries
/// are skipped, not propagated. Result: dirs first, then files, each A→Z.
pub fn list_directory(path: &str, rules: &IgnoreRules) -> Result<Vec<FileEntry>, AppError> {
    let dir = Path::new(path);
    if !dir.exists() {
        return Err(AppError::NotFound(path.to_string()));
    }

    let mut entries: Vec<FileEntry> = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let fe = match get_entry_metadata(&entry.path()) {
            Ok(f) => f,
            Err(_) => continue,
        };
        if should_hide_entry(&fe, rules) {
            continue;
        }
        entries.push(fe);
    }

    // Dirs first, then files; alphabetical (case-insensitive) within each group.
    entries.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    Ok(entries)
}

/// Metadados completos de um único arquivo/pasta.
pub fn get_file_metadata(path: &str) -> Result<FileEntry, AppError> {
    let p = Path::new(path);
    if !p.exists() {
        return Err(AppError::NotFound(path.to_string()));
    }
    get_entry_metadata(p)
}

/// Resumo (não recursivo) do diretório: contagens, tamanhos, datas e
/// extensões agrupadas.
pub fn scan_directory_summary(
    path: &str,
    rules: &IgnoreRules,
) -> Result<DirectorySummary, AppError> {
    let dir = Path::new(path);
    if !dir.exists() {
        return Err(AppError::NotFound(path.to_string()));
    }

    let mut summary = DirectorySummary {
        total_files: 0,
        total_dirs: 0,
        total_size: 0,
        smallest_file: 0,
        largest_file: 0,
        oldest_modified: None,
        newest_modified: None,
        extensions: Vec::new(),
    };
    let mut ext_map: HashMap<String, (u32, u64)> = HashMap::new();
    let mut smallest: Option<u64> = None;

    for entry in fs::read_dir(dir)? {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let fe = match get_entry_metadata(&entry.path()) {
            Ok(f) => f,
            Err(_) => continue,
        };
        if should_hide_entry(&fe, rules) {
            continue;
        }

        if fe.is_dir {
            summary.total_dirs += 1;
        } else {
            summary.total_files += 1;
            summary.total_size += fe.size;
            if fe.size > summary.largest_file {
                summary.largest_file = fe.size;
            }
            if fe.size > 0 {
                smallest = Some(smallest.map_or(fe.size, |m| m.min(fe.size)));
            }
            let ext = fe.extension.clone().unwrap_or_else(|| NO_EXT.to_string());
            let slot = ext_map.entry(ext).or_insert((0, 0));
            slot.0 += 1;
            slot.1 += fe.size;
        }

        // Datas: compara strings ISO (rfc3339 ordena cronologicamente).
        if let Some(m) = &fe.modified {
            match &summary.oldest_modified {
                None => summary.oldest_modified = Some(m.clone()),
                Some(o) if m < o => summary.oldest_modified = Some(m.clone()),
                _ => {}
            }
            match &summary.newest_modified {
                None => summary.newest_modified = Some(m.clone()),
                Some(n) if m > n => summary.newest_modified = Some(m.clone()),
                _ => {}
            }
        }
    }

    summary.smallest_file = smallest.unwrap_or(0);

    let mut exts: Vec<ExtensionSummary> = ext_map
        .into_iter()
        .map(|(extension, (count, total_size))| ExtensionSummary {
            extension,
            count,
            total_size,
        })
        .collect();
    // count desc; empate por extensão asc para ordem estável.
    exts.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.extension.cmp(&b.extension)));
    exts.truncate(MAX_EXTENSIONS);
    summary.extensions = exts;

    Ok(summary)
}

/// Build a directory tree containing only subdirectories, up to `depth`
/// (capped at MAX_DEPTH). Permission errors yield an empty `children`.
pub fn get_directory_tree(
    path: &str,
    depth: u8,
    rules: &IgnoreRules,
) -> Result<TreeNode, AppError> {
    let dir = Path::new(path);
    if !dir.exists() {
        return Err(AppError::NotFound(path.to_string()));
    }

    let depth = depth.min(MAX_DEPTH);
    Ok(build_tree(dir, depth, rules))
}

fn build_tree(dir: &Path, depth: u8, rules: &IgnoreRules) -> TreeNode {
    let name = dir
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| dir.to_string_lossy().into_owned());
    let path = dir.to_string_lossy().into_owned();

    let mut node = TreeNode {
        name,
        path,
        children: Vec::new(),
        is_expanded: false,
    };

    if depth == 0 {
        return node;
    }

    let read = match fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return node,
    };

    let subdirs: Vec<std::path::PathBuf> = read
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
        .filter(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            !should_hide(&name, true, rules)
        })
        .map(|e| e.path())
        .collect();

    node.children = if depth >= 2 {
        subdirs
            .par_iter()
            .map(|p| build_tree(p, depth - 1, rules))
            .collect()
    } else {
        subdirs
            .iter()
            .map(|p| build_tree(p, depth - 1, rules))
            .collect()
    };

    node.children
        .sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    node
}

#[cfg(target_os = "windows")]
pub fn get_home_directory() -> Option<String> {
    std::env::var("USERPROFILE").ok()
}

#[cfg(not(target_os = "windows"))]
pub fn get_home_directory() -> Option<String> {
    std::env::var("HOME").ok()
}

#[cfg(target_os = "windows")]
pub fn get_drives() -> Vec<FileEntry> {
    let mut drives = Vec::new();
    for letter in b'A'..=b'Z' {
        let root = format!("{}:\\", letter as char);
        let p = Path::new(&root);
        if p.exists() {
            drives.push(FileEntry {
                name: format!("{}:", letter as char),
                path: root.clone(),
                is_dir: true,
                size: 0,
                modified: None,
                created_at: None,
                extension: None,
                is_hidden: false,
                is_symlink: false,
            });
        }
    }
    drives
}

#[cfg(not(target_os = "windows"))]
pub fn get_drives() -> Vec<FileEntry> {
    vec![FileEntry {
        name: "/".to_string(),
        path: "/".to_string(),
        is_dir: true,
        size: 0,
        modified: None,
        created_at: None,
        extension: None,
        is_hidden: false,
        is_symlink: false,
    }]
}
