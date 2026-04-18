use std::collections::HashSet;
use std::path::Path;
use std::process::Command;

use regex::Regex;
use serde::Serialize;

use crate::artifact::{self, ArtifactKind, parse_frontmatter};

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RefKind {
    Path,
    Type,
    Function,
    Crate,
}

impl RefKind {
    fn as_str(&self) -> &'static str {
        match self {
            RefKind::Path => "path",
            RefKind::Type => "type",
            RefKind::Function => "function",
            RefKind::Crate => "crate",
        }
    }
}

#[derive(Debug, Serialize)]
pub struct MissingRef {
    kind: String,
    value: String,
    line: usize,
}

#[derive(Debug, Serialize)]
pub struct RefReport {
    doc: String,
    total_refs: usize,
    valid: usize,
    missing: usize,
    missing_refs: Vec<MissingRef>,
    staleness: f64,
}

// ---------------------------------------------------------------------------
// Extraction
// ---------------------------------------------------------------------------

pub fn extract_refs(body: &str) -> Vec<(RefKind, String, usize)> {
    let path_re =
        Regex::new(r"(?:src|crates|lib|tests|bin)/[a-zA-Z0-9_/.\-]*[a-zA-Z0-9_]").unwrap();
    let backtick_re = Regex::new(r"`([^`]+)`").unwrap();
    let type_re = Regex::new(r"^[A-Z][a-zA-Z0-9_]+$").unwrap();
    let func_re = Regex::new(r"^[a-z_][a-zA-Z0-9_]*\(\)$").unwrap();
    let crate_re = Regex::new(r"crates/([a-zA-Z0-9_\-]+)").unwrap();

    let mut seen = HashSet::new();
    let mut refs = Vec::new();

    for (idx, line) in body.lines().enumerate() {
        let line_num = idx + 1;

        // File paths (no backticks required)
        for m in path_re.find_iter(line) {
            let val = m.as_str().to_string();
            if seen.insert((RefKind::Path, val.clone())) {
                refs.push((RefKind::Path, val, line_num));
            }
        }

        // Crate names from paths like crates/<name>
        for cap in crate_re.captures_iter(line) {
            let name = cap[1].to_string();
            if seen.insert((RefKind::Crate, name.clone())) {
                refs.push((RefKind::Crate, name, line_num));
            }
        }

        // Backtick content: types and functions
        for cap in backtick_re.captures_iter(line) {
            let inner = &cap[1];
            if func_re.is_match(inner) {
                let val = inner.to_string();
                if seen.insert((RefKind::Function, val.clone())) {
                    refs.push((RefKind::Function, val, line_num));
                }
            } else if type_re.is_match(inner) {
                let val = inner.to_string();
                if seen.insert((RefKind::Type, val.clone())) {
                    refs.push((RefKind::Type, val, line_num));
                }
            }
        }
    }

    refs
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

fn search_in_project(pattern: &str, project_root: &Path) -> bool {
    // Try rg first (fast), -F for fixed-string (identifiers aren't regex)
    if let Ok(status) = Command::new("rg")
        .args(["-qF", pattern])
        .arg(project_root)
        .status()
    {
        return status.success();
    }
    // Fallback to grep
    Command::new("grep")
        .args(["-rqF", pattern])
        .arg(project_root)
        .status()
        .is_ok_and(|s| s.success())
}

pub fn validate_refs(
    refs: &[(RefKind, String, usize)],
    project_root: &Path,
    doc_name: &str,
) -> RefReport {
    let total_refs = refs.len();
    let mut missing_refs = Vec::new();

    for (kind, value, line) in refs {
        let valid = match kind {
            RefKind::Path => project_root.join(value).exists(),
            RefKind::Type | RefKind::Function => {
                let pattern = if *kind == RefKind::Function {
                    // Strip trailing () for search
                    value.trim_end_matches("()").to_string()
                } else {
                    value.clone()
                };
                search_in_project(&pattern, project_root)
            }
            RefKind::Crate => {
                project_root
                    .join("crates")
                    .join(value)
                    .join("Cargo.toml")
                    .exists()
                    || project_root.join(value).join("Cargo.toml").exists()
            }
        };
        if !valid {
            missing_refs.push(MissingRef {
                kind: kind.as_str().to_string(),
                value: value.clone(),
                line: *line,
            });
        }
    }

    let missing = missing_refs.len();
    let valid = total_refs - missing;
    let staleness = if total_refs == 0 {
        0.0
    } else {
        missing as f64 / total_refs as f64
    };

    RefReport {
        doc: doc_name.to_string(),
        total_refs,
        valid,
        missing,
        missing_refs,
        staleness,
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub fn run(
    doc_path: String,
    project_root: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let resolved =
        artifact::resolve_artifact_path(&doc_path, ArtifactKind::Doc).map_err(|e| e.to_string())?;
    let content = std::fs::read_to_string(&resolved)
        .map_err(|e| format!("cannot read {}: {e}", resolved.display()))?;

    let (_, body) = parse_frontmatter(&content);

    let root = match project_root {
        Some(r) => std::path::PathBuf::from(r),
        None => {
            let output = Command::new("git")
                .args(["rev-parse", "--show-toplevel"])
                .output()
                .map_err(|e| format!("git rev-parse failed: {e}"))?;
            if !output.status.success() {
                return Err("not inside a git repository".into());
            }
            std::path::PathBuf::from(String::from_utf8_lossy(&output.stdout).trim())
        }
    };

    let doc_name = resolved
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| doc_path.clone());

    // Try to build a richer doc name: project/kind/stem
    let doc_display = resolved
        .parent()
        .and_then(|kind_dir| {
            let kind = kind_dir.file_name()?.to_string_lossy().to_string();
            let project = kind_dir
                .parent()?
                .file_name()?
                .to_string_lossy()
                .to_string();
            Some(format!("{project}/{kind}/{doc_name}"))
        })
        .unwrap_or(doc_name);

    let refs = extract_refs(body);
    let report = validate_refs(&refs, &root, &doc_display);

    let json = serde_json::to_string_pretty(&report)?;
    println!("{json}");

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_file_paths_from_prose() {
        let body =
            "The entry point is src/main.rs and the renderer lives in crates/renderer/src/lib.rs.";
        let refs = extract_refs(body);
        let paths: Vec<&str> = refs
            .iter()
            .filter(|(k, _, _)| *k == RefKind::Path)
            .map(|(_, v, _)| v.as_str())
            .collect();
        assert!(paths.contains(&"src/main.rs"), "should find src/main.rs");
        assert!(
            paths.contains(&"crates/renderer/src/lib.rs"),
            "should find crates/renderer/src/lib.rs"
        );
    }

    #[test]
    fn extracts_backtick_type_and_function() {
        let body = "Use `MyStruct` and call `do_thing()` for setup.";
        let refs = extract_refs(body);
        let types: Vec<&str> = refs
            .iter()
            .filter(|(k, _, _)| *k == RefKind::Type)
            .map(|(_, v, _)| v.as_str())
            .collect();
        let funcs: Vec<&str> = refs
            .iter()
            .filter(|(k, _, _)| *k == RefKind::Function)
            .map(|(_, v, _)| v.as_str())
            .collect();
        assert!(types.contains(&"MyStruct"));
        assert!(funcs.contains(&"do_thing()"));
    }

    #[test]
    fn deduplicates_same_ref_on_multiple_lines() {
        let body = "See src/main.rs for details.\nAlso check src/main.rs again.";
        let refs = extract_refs(body);
        let path_count = refs
            .iter()
            .filter(|(k, v, _)| *k == RefKind::Path && v == "src/main.rs")
            .count();
        assert_eq!(path_count, 1, "duplicate path should be deduplicated");
    }

    #[test]
    fn bare_words_not_extracted_as_types() {
        let body = "MyStruct is mentioned without backticks here.";
        let refs = extract_refs(body);
        let types: Vec<&str> = refs
            .iter()
            .filter(|(k, _, _)| *k == RefKind::Type)
            .map(|(_, v, _)| v.as_str())
            .collect();
        assert!(
            !types.contains(&"MyStruct"),
            "bare word should not be extracted as type"
        );
    }

    #[test]
    fn extracts_crate_names() {
        let body = "The crates/renderer module handles display.";
        let refs = extract_refs(body);
        let crates: Vec<&str> = refs
            .iter()
            .filter(|(k, _, _)| *k == RefKind::Crate)
            .map(|(_, v, _)| v.as_str())
            .collect();
        assert!(crates.contains(&"renderer"));
    }

    #[test]
    fn line_numbers_are_1_based() {
        let body = "first line\nsrc/foo.rs here\nthird line";
        let refs = extract_refs(body);
        let path_ref = refs.iter().find(|(k, _, _)| *k == RefKind::Path).unwrap();
        assert_eq!(path_ref.2, 2, "path on second line should have line_num=2");
    }
}
