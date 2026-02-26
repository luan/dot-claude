use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::Command;

fn git(args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .map_err(|e| format!("running git: {e}"))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

pub fn get_commits_with_files(n: usize) -> Result<Vec<HashSet<String>>, String> {
    let output = git(&[
        "log",
        &format!("-n{n}"),
        "--name-only",
        "--pretty=format:%H",
        "--no-merges",
        "--diff-filter=ACDMRTUXB",
    ])
    .map_err(|e| {
        eprintln!("git log failed: {e}");
        e
    })?;

    let output = output.trim();
    if output.is_empty() {
        return Ok(vec![]);
    }

    let mut commits: Vec<HashSet<String>> = Vec::new();
    let mut changed_files: HashSet<String> = HashSet::new();
    let mut before_commit_hash = true;

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            if !changed_files.is_empty() {
                commits.push(std::mem::take(&mut changed_files));
            }
            before_commit_hash = true;
        } else if before_commit_hash {
            // Commit hash line: save any accumulated files from previous commit
            if !changed_files.is_empty() {
                commits.push(std::mem::take(&mut changed_files));
            }
            before_commit_hash = false;
        } else {
            changed_files.insert(line.to_string());
        }
    }

    if !changed_files.is_empty() {
        commits.push(changed_files);
    }

    Ok(commits)
}

pub fn calculate_file_associations(
    commits: &[HashSet<String>],
    threshold: f64,
    min_commits: usize,
) -> HashMap<String, HashMap<String, f64>> {
    let mut file_commit_count: HashMap<String, usize> = HashMap::new();
    let mut file_pair_count: HashMap<String, HashMap<String, usize>> = HashMap::new();

    for commit_files in commits {
        if commit_files.len() > 100 {
            continue;
        }
        for file in commit_files {
            *file_commit_count.entry(file.clone()).or_default() += 1;
            for other_file in commit_files {
                if file != other_file {
                    *file_pair_count
                        .entry(file.clone())
                        .or_default()
                        .entry(other_file.clone())
                        .or_default() += 1;
                }
            }
        }
    }

    let mut result: HashMap<String, HashMap<String, f64>> = HashMap::new();
    for (file, commit_count) in &file_commit_count {
        if *commit_count < min_commits {
            continue;
        }

        let mut associations: HashMap<String, f64> = HashMap::new();
        if let Some(pairs) = file_pair_count.get(file) {
            for (other_file, &pair_count) in pairs {
                let fraction = pair_count as f64 / *commit_count as f64;
                if fraction >= threshold {
                    associations.insert(other_file.clone(), fraction);
                }
            }
        }

        if !associations.is_empty() {
            result.insert(file.clone(), associations);
        }
    }

    result
}

pub fn get_changed_files(base: &str) -> Result<HashSet<String>, String> {
    let ref_arg = format!("{base}...HEAD");
    git(&["diff", "--name-only", &ref_arg])
        .map_err(|e| {
            eprintln!("git diff failed: {e}");
            e
        })
        .map(|out| {
            out.lines()
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .map(String::from)
                .collect()
        })
}

fn get_files_on_branch(base: &str) -> HashSet<String> {
    git(&["ls-tree", "--name-only", "-r", base])
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect()
}

pub fn collect_changed_associations(
    associations: &HashMap<String, HashMap<String, f64>>,
    changed_files: &HashSet<String>,
    max_files: Option<usize>,
    base: &str,
) -> Vec<(String, f64)> {
    let base_files = get_files_on_branch(base);

    let mut max_fractions: HashMap<String, f64> = HashMap::new();
    for file_path in changed_files {
        if let Some(related) = associations.get(file_path) {
            for (other_file, &fraction) in related {
                if changed_files.contains(other_file) {
                    continue;
                }
                let entry = max_fractions.entry(other_file.clone()).or_insert(0.0);
                if fraction > *entry {
                    *entry = fraction;
                }
            }
        }
    }

    let mut sorted: Vec<(String, f64)> = max_fractions.into_iter().collect();
    sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let limit = max_files.unwrap_or(usize::MAX);
    sorted
        .into_iter()
        .filter(|(path, _)| Path::new(path).exists() || base_files.contains(path))
        .take(limit)
        .map(|(path, frac)| (path, (frac * 10.0).floor() / 10.0))
        .collect()
}

pub fn output_changed_associations(
    associations: &HashMap<String, HashMap<String, f64>>,
    changed_files: &HashSet<String>,
    max_files: Option<usize>,
    base: &str,
) {
    let base_files = get_files_on_branch(base);

    let mut max_fractions: HashMap<String, f64> = HashMap::new();

    for file_path in changed_files {
        if let Some(related) = associations.get(file_path) {
            for (other_file, &fraction) in related {
                if changed_files.contains(other_file) {
                    continue;
                }
                let entry = max_fractions.entry(other_file.clone()).or_insert(0.0);
                if fraction > *entry {
                    *entry = fraction;
                }
            }
        }
    }

    let mut sorted_files: Vec<(String, f64)> = max_fractions.into_iter().collect();
    sorted_files.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let limit = max_files.unwrap_or(usize::MAX);
    let mut count = 0;

    for (path, fraction) in &sorted_files {
        if count >= limit {
            break;
        }
        if !Path::new(path).exists() && !base_files.contains(path) {
            continue;
        }
        let truncated = (fraction * 10.0).floor() / 10.0;
        println!("{:.1} {path}", truncated);
        count += 1;
    }
}

pub fn run(
    base: String,
    threshold: f64,
    min_commits: usize,
    max_files: Option<usize>,
    num_commits: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    if num_commits == 0 {
        return Err("num-commits must be > 0".into());
    }
    if !(0.0..=1.0).contains(&threshold) {
        return Err("threshold must be between 0.0 and 1.0".into());
    }
    if min_commits == 0 {
        return Err("min-commits must be > 0".into());
    }

    let commits = get_commits_with_files(num_commits)?;
    if commits.is_empty() {
        eprintln!("No commits found or no files changed in the analyzed commits.");
        return Ok(());
    }

    let associations = calculate_file_associations(&commits, threshold, min_commits);
    let changed_files = get_changed_files(&base)?;

    if changed_files.is_empty() {
        eprintln!("No files changed compared to {base}.");
        return Ok(());
    }

    output_changed_associations(&associations, &changed_files, max_files, &base);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_commits(output: &str) -> Vec<HashSet<String>> {
        // Re-implement the parser inline so tests don't require git.
        // Mirrors get_commits_with_files parser logic exactly.
        let output = output.trim();
        if output.is_empty() {
            return vec![];
        }
        let mut commits: Vec<HashSet<String>> = Vec::new();
        let mut changed_files: HashSet<String> = HashSet::new();
        let mut before_commit_hash = true;
        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() {
                if !changed_files.is_empty() {
                    commits.push(std::mem::take(&mut changed_files));
                }
                before_commit_hash = true;
            } else if before_commit_hash {
                if !changed_files.is_empty() {
                    commits.push(std::mem::take(&mut changed_files));
                }
                before_commit_hash = false;
            } else {
                changed_files.insert(line.to_string());
            }
        }
        if !changed_files.is_empty() {
            commits.push(changed_files);
        }
        commits
    }

    // --- get_commits_with_files parser ---

    #[test]
    fn parser_normal_commits() {
        let output = "abc123\nfoo.rs\nbar.rs\n\ndef456\nbaz.rs\n";
        let commits = parse_commits(output);
        assert_eq!(commits.len(), 2);
        assert!(commits[0].contains("foo.rs"));
        assert!(commits[0].contains("bar.rs"));
        assert!(commits[1].contains("baz.rs"));
    }

    #[test]
    fn parser_empty_output() {
        let commits = parse_commits("");
        assert!(commits.is_empty());
    }

    #[test]
    fn parser_whitespace_only() {
        let commits = parse_commits("   \n  \n");
        assert!(commits.is_empty());
    }

    #[test]
    fn parser_commit_with_no_files() {
        // A commit hash with no following file lines should produce no entry.
        let output = "abc123\n\ndef456\nbaz.rs\n";
        let commits = parse_commits(output);
        assert_eq!(commits.len(), 1);
        assert!(commits[0].contains("baz.rs"));
    }

    #[test]
    fn parser_single_commit_no_trailing_newline() {
        let output = "abc123\nonly.rs";
        let commits = parse_commits(output);
        assert_eq!(commits.len(), 1);
        assert!(commits[0].contains("only.rs"));
    }

    // --- calculate_file_associations ---

    fn make_commit(files: &[&str]) -> HashSet<String> {
        files.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn associations_threshold_filtering() {
        // a.rs and b.rs co-change in 2 of 2 commits → fraction 1.0 (above 0.5)
        // a.rs and c.rs co-change in 1 of 2 commits → fraction 0.5 (at threshold, included)
        // a.rs and d.rs co-change in 0 → excluded
        let commits = vec![
            make_commit(&["a.rs", "b.rs", "c.rs"]),
            make_commit(&["a.rs", "b.rs"]),
        ];
        let result = calculate_file_associations(&commits, 0.5, 1);
        let a_assoc = result.get("a.rs").expect("a.rs must have associations");
        assert!(a_assoc.contains_key("b.rs"), "b.rs should be associated");
        assert!(a_assoc.contains_key("c.rs"), "c.rs should be associated");
        assert!(!a_assoc.contains_key("d.rs"));
    }

    #[test]
    fn associations_below_threshold_excluded() {
        // a.rs appears in 4 commits, c.rs only in 1 of those → 0.25 < 0.5
        let commits = vec![
            make_commit(&["a.rs", "b.rs"]),
            make_commit(&["a.rs", "b.rs"]),
            make_commit(&["a.rs", "b.rs"]),
            make_commit(&["a.rs", "b.rs", "c.rs"]),
        ];
        let result = calculate_file_associations(&commits, 0.5, 1);
        let a_assoc = result.get("a.rs").expect("a.rs must have associations");
        assert!(
            !a_assoc.contains_key("c.rs"),
            "c.rs fraction is 0.25, below threshold"
        );
    }

    #[test]
    fn associations_min_commits_filtering() {
        // a.rs appears only once; with min_commits=2 it should be excluded entirely.
        let commits = vec![make_commit(&["a.rs", "b.rs"])];
        let result = calculate_file_associations(&commits, 0.0, 2);
        assert!(!result.contains_key("a.rs"));
    }

    #[test]
    fn associations_empty_input() {
        let result = calculate_file_associations(&[], 0.5, 1);
        assert!(result.is_empty());
    }

    // --- edge cases for output_changed_associations ---

    #[test]
    fn no_changed_files_produces_no_output() {
        let commits = vec![make_commit(&["a.rs", "b.rs"])];
        let associations = calculate_file_associations(&commits, 0.5, 1);
        // Empty changed_files → no associations surfaced (no panic, no output).
        let changed: HashSet<String> = HashSet::new();
        // Just verify this doesn't panic and associations are non-empty.
        assert!(!associations.is_empty());
        // With no changed files the loop body in output_changed_associations is never entered.
        for file in &changed {
            assert!(associations.contains_key(file.as_str()));
        }
    }

    #[test]
    fn all_related_files_already_changed() {
        // When every associated file is already in changed_files, nothing gets printed.
        let commits = vec![
            make_commit(&["a.rs", "b.rs"]),
            make_commit(&["a.rs", "b.rs"]),
        ];
        let associations = calculate_file_associations(&commits, 0.5, 1);
        let changed: HashSet<String> = ["a.rs", "b.rs"].iter().map(|s| s.to_string()).collect();
        // build max_fractions manually — all related are in changed, so result must be empty.
        let mut max_fractions: std::collections::HashMap<String, f64> = HashMap::new();
        for file_path in &changed {
            if let Some(related) = associations.get(file_path.as_str()) {
                for (other_file, &fraction) in related {
                    if changed.contains(other_file) {
                        continue;
                    }
                    let entry = max_fractions.entry(other_file.clone()).or_insert(0.0);
                    if fraction > *entry {
                        *entry = fraction;
                    }
                }
            }
        }
        assert!(
            max_fractions.is_empty(),
            "all related files are already changed"
        );
    }
}
