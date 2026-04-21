use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

pub fn load_ignore_patterns(root: &Path, cli_patterns: &[String]) -> Result<Vec<String>> {
    let mut patterns = Vec::new();
    let ignore_path = root.join(".symignore");

    if ignore_path.exists() {
        let contents = fs::read_to_string(&ignore_path)
            .with_context(|| format!("reading {}", ignore_path.display()))?;
        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            patterns.push(line.to_string());
        }
    }

    patterns.extend(cli_patterns.iter().cloned());
    Ok(patterns)
}
