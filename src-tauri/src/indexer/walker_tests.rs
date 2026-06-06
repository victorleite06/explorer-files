use std::fs;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

use crate::fs_engine::ignore_rules::IgnoreRules;
use crate::indexer::walker::{is_path_gitignored, AppWalker};

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

fn setup_repo(dir: &TempDir) -> PathBuf {
    let root = dir.path().to_path_buf();
    fs::create_dir_all(root.join(".git")).unwrap();
    write(&root.join(".gitignore"), "target/\n*.log\nbuild/\n");
    write(&root.join("src").join("main.rs"), "fn main() {}");
    write(&root.join("src").join("lib.rs"), "pub fn x() {}");
    write(&root.join("target").join("debug").join("app.exe"), "bin");
    write(&root.join("app.log"), "log line");
    write(&root.join("build").join("output.js"), "console.log(1)");
    write(&root.join("README.md"), "# readme");
    root
}

/// Caminhos relativos (com '/') retornados pelo walker.
fn rel_names(root: &Path, w: &AppWalker) -> Vec<String> {
    w.walk()
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            e.path()
                .strip_prefix(root)
                .ok()
                .map(|p| p.to_string_lossy().replace('\\', "/"))
        })
        .collect()
}

#[test]
fn test_gitignore_excludes_target() {
    let dir = TempDir::new().unwrap();
    let root = setup_repo(&dir);
    let w = AppWalker::new(&root, IgnoreRules::default()).respect_gitignore(true);
    let names = rel_names(&root, &w);
    assert!(!names.iter().any(|n| n.starts_with("target")), "{names:?}");
    assert!(names.iter().any(|n| n == "src/main.rs"));
}

#[test]
fn test_gitignore_excludes_log_files() {
    let dir = TempDir::new().unwrap();
    let root = setup_repo(&dir);
    let w = AppWalker::new(&root, IgnoreRules::default()).respect_gitignore(true);
    let names = rel_names(&root, &w);
    assert!(!names.iter().any(|n| n == "app.log"), "{names:?}");
}

#[test]
fn test_without_gitignore_includes_all() {
    let dir = TempDir::new().unwrap();
    let root = setup_repo(&dir);
    let w = AppWalker::new(&root, IgnoreRules::default()).respect_gitignore(false);
    let names = rel_names(&root, &w);
    assert!(names.iter().any(|n| n.starts_with("target")));
    assert!(names.iter().any(|n| n == "app.log"));
    assert!(names.iter().any(|n| n.starts_with("build")));
}

#[test]
fn test_nested_gitignore() {
    let dir = TempDir::new().unwrap();
    let root = setup_repo(&dir);
    write(&root.join("src").join(".gitignore"), "secret.rs\n");
    write(&root.join("src").join("secret.rs"), "// secret");
    let w = AppWalker::new(&root, IgnoreRules::default()).respect_gitignore(true);
    let names = rel_names(&root, &w);
    assert!(!names.iter().any(|n| n == "src/secret.rs"), "{names:?}");
    assert!(names.iter().any(|n| n == "src/main.rs"));
}

#[test]
fn test_ignore_file_respected() {
    let dir = TempDir::new().unwrap();
    let root = setup_repo(&dir);
    write(&root.join(".ignore"), "notes.txt\n");
    write(&root.join("notes.txt"), "notes");

    let on = AppWalker::new(&root, IgnoreRules::default())
        .respect_gitignore(true)
        .respect_ignore_files(true);
    assert!(!rel_names(&root, &on).iter().any(|n| n == "notes.txt"));

    let off = AppWalker::new(&root, IgnoreRules::default())
        .respect_gitignore(true)
        .respect_ignore_files(false);
    assert!(rel_names(&root, &off).iter().any(|n| n == "notes.txt"));
}

#[test]
fn test_app_rules_combined_with_gitignore() {
    let dir = TempDir::new().unwrap();
    let root = setup_repo(&dir);
    write(&root.join(".secret"), "hidden");
    // IgnoreRules padrão oculta dotfiles, mesmo com gitignore ativo.
    let w = AppWalker::new(&root, IgnoreRules::default()).respect_gitignore(true);
    let names = rel_names(&root, &w);
    assert!(!names.iter().any(|n| n == ".secret"), "{names:?}");
}

#[test]
fn test_is_path_gitignored() {
    let dir = TempDir::new().unwrap();
    let root = setup_repo(&dir);
    assert!(is_path_gitignored(&root.join("target").join("debug").join("app.exe"), &root));
    assert!(!is_path_gitignored(&root.join("src").join("main.rs"), &root));
}

#[test]
fn test_no_git_repo_does_not_panic() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().to_path_buf();
    write(&root.join("a.txt"), "x");
    write(&root.join("b.rs"), "y");
    let w = AppWalker::new(&root, IgnoreRules::default());
    let names = rel_names(&root, &w);
    assert!(names.iter().any(|n| n == "a.txt"));
    assert!(names.iter().any(|n| n == "b.rs"));
}
