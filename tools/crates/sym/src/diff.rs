use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::repo;
use crate::resolve;
use crate::store::SymbolResult;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct DiffResult {
    pub symbol: SymbolResult,
    pub base: String,
    pub content: String,
    pub stat: bool,
}

pub fn symbol_diff(cwd: &Path, name: &str, base: &str, stat: bool) -> Result<DiffResult> {
    let symbol = resolve::resolve_symbol(cwd, name)?.symbol;

    let file_path = Path::new(&symbol.file);
    let repo_root = repo::find_git_root(file_path.parent().unwrap_or(cwd))?;
    let rel_path = file_path
        .strip_prefix(&repo_root)
        .with_context(|| format!("computing relative path for {}", file_path.display()))?;

    let content = if stat {
        git_diff(&repo_root, rel_path, base, true)?
    } else {
        let diff = git_diff(&repo_root, rel_path, base, false)?;
        if diff.is_empty() {
            diff
        } else {
            filter_diff_hunks(&diff, symbol.start_line, symbol.end_line)
        }
    };

    Ok(DiffResult {
        symbol,
        base: base.to_string(),
        content,
        stat,
    })
}

fn git_diff(repo_root: &Path, rel_path: &Path, base: &str, stat: bool) -> Result<String> {
    let mut command = Command::new("git");
    command.arg("-C").arg(repo_root).arg("diff");
    if stat {
        command.arg("--stat");
    }
    command.arg(base).arg("--").arg(rel_path);

    let output = command
        .output()
        .with_context(|| format!("running git diff in {}", repo_root.display()))?;
    if !output.status.success() && !output.stderr.is_empty() {
        bail!("git diff: {}", String::from_utf8_lossy(&output.stderr).trim());
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

pub fn filter_diff_hunks(diff_output: &str, start_line: usize, end_line: usize) -> String {
    let lines = diff_output.split_inclusive('\n');

    let mut result = String::new();
    let mut file_headers: Vec<&str> = Vec::new();
    let mut current_hunk: Vec<&str> = Vec::new();
    let mut hunk_overlaps = false;
    let mut wrote_headers = false;

    for line in lines {
        let trimmed = line.trim_end_matches('\n');

        if matches_file_header(trimmed) {
            flush_hunk(
                &mut result,
                &file_headers,
                &current_hunk,
                hunk_overlaps,
                &mut wrote_headers,
            );
            current_hunk.clear();
            hunk_overlaps = false;
            file_headers.push(line);
            continue;
        }

        if trimmed.starts_with("@@") {
            flush_hunk(
                &mut result,
                &file_headers,
                &current_hunk,
                hunk_overlaps,
                &mut wrote_headers,
            );
            current_hunk.clear();
            current_hunk.push(line);
            hunk_overlaps = false;

            let (new_start, new_count) = parse_hunk_header(trimmed);
            if new_start > 0 {
                let mut hunk_end = new_start + new_count.saturating_sub(1);
                if new_count == 0 {
                    hunk_end = new_start;
                }
                if new_start <= end_line && hunk_end >= start_line {
                    hunk_overlaps = true;
                }
            }
            continue;
        }

        if !current_hunk.is_empty() {
            current_hunk.push(line);
        }
    }

    flush_hunk(
        &mut result,
        &file_headers,
        &current_hunk,
        hunk_overlaps,
        &mut wrote_headers,
    );
    result
}

pub fn parse_hunk_header(header: &str) -> (usize, usize) {
    let Some(plus_idx) = header.find('+') else {
        return (0, 0);
    };
    let rest = &header[plus_idx + 1..];
    let Some(end_idx) = rest.find(' ') else {
        return (0, 0);
    };
    let range_str = &rest[..end_idx];
    let mut parts = range_str.splitn(2, ',');
    let Some(start) = parts.next().and_then(|value| value.parse::<usize>().ok()) else {
        return (0, 0);
    };
    let count = parts
        .next()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(1);
    (start, count)
}

fn flush_hunk(
    result: &mut String,
    file_headers: &[&str],
    current_hunk: &[&str],
    hunk_overlaps: bool,
    wrote_headers: &mut bool,
) {
    if !hunk_overlaps || current_hunk.is_empty() {
        return;
    }
    if !*wrote_headers {
        for header in file_headers {
            result.push_str(header);
        }
        *wrote_headers = true;
    }
    for line in current_hunk {
        result.push_str(line);
    }
}

fn matches_file_header(line: &str) -> bool {
    line.starts_with("diff --git ")
        || line.starts_with("index ")
        || line.starts_with("--- ")
        || line.starts_with("+++ ")
}
