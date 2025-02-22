use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct SourceFile {
    pub path: PathBuf,
    pub content: String,
    pub module_path: String,
}

pub struct SourceLoader {
    root_path: PathBuf,
    ignored_patterns: Vec<String>,
}

impl SourceLoader {
    pub fn new<P: AsRef<Path>>(root_path: P, ignored_patterns: Vec<String>) -> Self {
        Self {
            root_path: root_path.as_ref().to_path_buf(),
            ignored_patterns,
        }
    }

    fn is_ignored(&self, entry: &walkdir::DirEntry) -> bool {
        let path = entry.path().to_string_lossy();
        self.ignored_patterns
            .iter()
            .any(|pattern| path.contains(pattern))
    }

    pub fn discover_rust_files(&self) -> Result<Vec<SourceFile>> {
        let mut files = Vec::new();

        for entry in WalkDir::new(&self.root_path)
            .follow_links(true)
            .into_iter()
            .filter_entry(|e| !Self::is_hidden(e) && !self.is_ignored(e))
        {
            let entry = entry.context("Failed to read directory entry")?;
            if !Self::is_rust_file(entry.path()) {
                continue;
            }

            let path = entry.path().to_path_buf();
            let content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read file: {}", path.display()))?;

            let module_path = Self::calculate_module_path(&self.root_path, &path)?;

            files.push(SourceFile {
                path,
                content,
                module_path,
            });
        }

        Ok(files)
    }

    fn is_hidden(entry: &walkdir::DirEntry) -> bool {
        entry
            .file_name()
            .to_str()
            .map(|s| s.starts_with('.'))
            .unwrap_or(false)
    }

    fn is_rust_file(path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext == "rs")
            .unwrap_or(false)
    }

    fn calculate_module_path(root: &Path, file_path: &Path) -> Result<String> {
        let rel_path = file_path.strip_prefix(root)?;
        let mut module_parts: Vec<String> = rel_path
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect();

        // Remove the .rs extension from the last component
        if let Some(last) = module_parts.last_mut() {
            *last = last.trim_end_matches(".rs").to_string();
        }

        // Filter out mod.rs files
        module_parts.retain(|part| part != "mod");

        Ok(module_parts.join("::"))
    }
}
