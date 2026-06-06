use std::path::{Path, PathBuf};

use ignore::gitignore::GitignoreBuilder;
use ignore::{DirEntry, WalkBuilder};

use crate::fs_engine::ignore_rules::{should_hide, IgnoreRules};

/// Travessia de diretórios combinando IgnoreRules do app com regras do git.
pub struct AppWalker {
    root: PathBuf,
    rules: IgnoreRules,
    max_depth: Option<usize>,
    respect_gitignore: bool,
    respect_global_gitignore: bool,
    respect_ignore_files: bool,
    follow_symlinks: bool,
    same_filesystem: bool,
}

impl AppWalker {
    pub fn new(root: impl Into<PathBuf>, rules: IgnoreRules) -> Self {
        Self {
            root: root.into(),
            rules,
            max_depth: None,
            respect_gitignore: true,
            respect_global_gitignore: true,
            respect_ignore_files: true,
            follow_symlinks: false,
            same_filesystem: true,
        }
    }

    pub fn max_depth(mut self, depth: usize) -> Self {
        self.max_depth = Some(depth);
        self
    }
    pub fn respect_gitignore(mut self, v: bool) -> Self {
        self.respect_gitignore = v;
        self
    }
    pub fn respect_global_gitignore(mut self, v: bool) -> Self {
        self.respect_global_gitignore = v;
        self
    }
    pub fn respect_ignore_files(mut self, v: bool) -> Self {
        self.respect_ignore_files = v;
        self
    }
    pub fn follow_symlinks(mut self, v: bool) -> Self {
        self.follow_symlinks = v;
        self
    }

    fn builder(&self) -> WalkBuilder {
        let mut b = WalkBuilder::new(&self.root);
        b.max_depth(self.max_depth)
            .git_ignore(self.respect_gitignore)
            .git_global(self.respect_global_gitignore)
            .git_exclude(self.respect_gitignore) // .git/info/exclude
            .ignore(self.respect_ignore_files) // arquivos .ignore
            .hidden(false) // app tem própria lógica de hidden
            .follow_links(self.follow_symlinks)
            .same_file_system(self.same_filesystem)
            .require_git(false); // funciona fora de repos git
        b
    }

    pub fn build(&self) -> ignore::Walk {
        self.builder().build()
    }

    pub fn build_parallel(&self) -> ignore::WalkParallel {
        self.builder().build_parallel()
    }

    /// Iterador aplicando IgnoreRules do app DEPOIS das regras do git.
    pub fn walk(&self) -> impl Iterator<Item = Result<DirEntry, ignore::Error>> + '_ {
        self.build().filter(move |result| match result {
            Ok(entry) => {
                let name = entry.file_name().to_str().unwrap_or("");
                let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                !should_hide(name, is_dir, &self.rules)
            }
            Err(_) => true,
        })
    }
}

/// Verifica se um path seria ignorado pelo gitignore dentro de `root`.
/// fail-open: false se não-git ou em erro.
pub fn is_path_gitignored(path: &Path, root: &Path) -> bool {
    if !root.join(".git").exists() {
        return false;
    }
    let mut b = GitignoreBuilder::new(root);
    b.add(root.join(".gitignore"));
    let _ = b.add(root.join(".git").join("info").join("exclude"));
    let gi = match b.build() {
        Ok(g) => g,
        Err(_) => return false,
    };
    let is_dir = path.is_dir();
    gi.matched_path_or_any_parents(path, is_dir).is_ignore()
}

/// Lista arquivos .gitignore (+ .git/info/exclude) sob `root` até `max_depth`.
/// Pula node_modules e pastas de build para acelerar.
pub fn find_gitignore_files(root: &Path, max_depth: u8) -> Vec<PathBuf> {
    const SKIP: &[&str] = &["node_modules", "target", "dist", "build", ".next"];
    let mut out = Vec::new();

    let walker = WalkBuilder::new(root)
        .max_depth(Some(max_depth as usize))
        .git_ignore(false)
        .ignore(false)
        .hidden(false)
        .require_git(false)
        .filter_entry(|e| {
            let name = e.file_name().to_str().unwrap_or("");
            !(e.file_type().map(|t| t.is_dir()).unwrap_or(false) && SKIP.contains(&name))
        })
        .build();

    for entry in walker.filter_map(|e| e.ok()) {
        if entry.file_type().map(|t| t.is_file()).unwrap_or(false)
            && entry.file_name().to_str() == Some(".gitignore")
        {
            out.push(entry.path().to_path_buf());
        }
    }

    let exclude = root.join(".git").join("info").join("exclude");
    if exclude.exists() {
        out.push(exclude);
    }
    out
}
