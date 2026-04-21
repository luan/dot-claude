use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::{Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use walkdir::WalkDir;

use crate::lang;

const DEFAULT_SKIP_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "vendor",
    ".venv",
    "venv",
    "__pycache__",
    ".tox",
    ".mypy_cache",
    "dist",
    "build",
    ".next",
    ".nuxt",
    "target",
    ".idea",
    ".vscode",
];

pub(crate) fn is_skipped_dir_name(name: &str) -> bool {
    name.starts_with('.') || DEFAULT_SKIP_DIRS.contains(&name)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileEntry {
    pub path: PathBuf,
    pub rel_path: PathBuf,
    pub size: u64,
    pub language: String,
    pub modified: SystemTime,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct TreeNode {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub children: Vec<TreeNode>,
}

#[derive(Debug, Clone, Default)]
pub struct WalkOptions {
    pub parseable_only: bool,
    pub ignore: Vec<String>,
}

pub fn walk(root: &Path, options: &WalkOptions) -> Result<Vec<FileEntry>> {
    let matchers = IgnoreMatchers::new(&options.ignore)?;
    let mut files = Vec::new();

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|entry| should_enter(entry.path(), root, &matchers))
    {
        let entry = entry.with_context(|| format!("walking {}", root.display()))?;
        if !entry.file_type().is_file() {
            continue;
        }

        let rel_path = entry
            .path()
            .strip_prefix(root)
            .with_context(|| format!("computing relative path for {}", entry.path().display()))?
            .to_path_buf();
        if matchers.matches(&rel_path) {
            continue;
        }

        let Some(language) = lang::language_for_file(entry.path()) else {
            continue;
        };
        if options.parseable_only && !language.parseable {
            continue;
        }

        let metadata = fs::metadata(entry.path())
            .with_context(|| format!("reading metadata for {}", entry.path().display()))?;
        files.push(FileEntry {
            path: entry.path().to_path_buf(),
            rel_path,
            size: metadata.len(),
            language: language.id.to_string(),
            modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
        });
    }

    files.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
    Ok(files)
}

pub fn build_tree(root: &Path, max_depth: usize) -> Result<TreeNode> {
    let root = root.canonicalize()?;
    build_tree_recursive(&root, &root, 1, max_depth)
}

pub fn print_tree(node: &TreeNode) -> String {
    let mut out = String::new();
    render_tree(node, "", true, &mut out);
    out
}

fn build_tree_recursive(root: &Path, path: &Path, depth: usize, max_depth: usize) -> Result<TreeNode> {
    let metadata = fs::metadata(path).with_context(|| format!("reading {}", path.display()))?;
    let mut node = TreeNode {
        name: path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.display().to_string()),
        path: path.to_path_buf(),
        is_dir: metadata.is_dir(),
        children: Vec::new(),
    };

    if !metadata.is_dir() || (max_depth > 0 && depth > max_depth) {
        return Ok(node);
    }

    let mut entries = fs::read_dir(path)
        .with_context(|| format!("reading directory {}", path.display()))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let child_path = entry.path();
        if should_skip_tree_entry(&child_path, root) {
            continue;
        }
        node.children
            .push(build_tree_recursive(root, &child_path, depth + 1, max_depth)?);
    }

    Ok(node)
}

fn render_tree(node: &TreeNode, prefix: &str, is_root: bool, out: &mut String) {
    if is_root {
        out.push_str(&node.name);
        out.push('\n');
    }

    for (index, child) in node.children.iter().enumerate() {
        let is_last = index + 1 == node.children.len();
        let connector = if is_last { "└── " } else { "├── " };
        out.push_str(prefix);
        out.push_str(connector);
        out.push_str(&child.name);
        out.push('\n');

        let next_prefix = if is_last {
            format!("{prefix}    ")
        } else {
            format!("{prefix}│   ")
        };
        render_tree(child, &next_prefix, false, out);
    }
}

fn should_enter(path: &Path, root: &Path, matchers: &IgnoreMatchers) -> bool {
    if path == root {
        return true;
    }

    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return true;
    };
    if is_skipped_dir_name(name) {
        return false;
    }

    let Ok(rel_path) = path.strip_prefix(root) else {
        return true;
    };
    !matchers.matches(rel_path)
}

fn should_skip_tree_entry(path: &Path, root: &Path) -> bool {
    if path == root {
        return false;
    }

    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    is_skipped_dir_name(name)
}

struct IgnoreMatchers {
    exact_names: Vec<String>,
    globset: GlobSet,
}

impl IgnoreMatchers {
    fn new(patterns: &[String]) -> Result<Self> {
        let mut exact_names = Vec::new();
        let mut builder = GlobSetBuilder::new();

        for pattern in patterns
            .iter()
            .map(|pattern| pattern.trim())
            .filter(|pattern| !pattern.is_empty())
        {
            if !pattern.contains('*') && !pattern.contains('?') && !pattern.contains('[') {
                exact_names.push(pattern.trim_matches('/').to_string());
            }

            let normalized = pattern.trim_start_matches("./").trim_matches('/');
            if normalized.is_empty() {
                continue;
            }

            builder.add(Glob::new(normalized)?);
            builder.add(Glob::new(&format!("{normalized}/**"))?);
        }

        Ok(Self {
            exact_names,
            globset: builder.build()?,
        })
    }

    fn matches(&self, rel_path: &Path) -> bool {
        let rel = rel_path.to_string_lossy().replace('\\', "/");
        if self.globset.is_match(&rel) {
            return true;
        }

        rel_path.components().any(|component| {
            let name = component.as_os_str().to_string_lossy();
            self.exact_names.iter().any(|pattern| pattern == &name)
        })
    }
}
