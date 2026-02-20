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

pub fn get_commits_with_files(n: usize) -> Vec<HashSet<String>> {
    let output = git(&[
        "log",
        &format!("-n{n}"),
        "--name-only",
        "--pretty=format:%H",
        "--no-merges",
        "--diff-filter=ACDMRTUXB*",
    ])
    .unwrap_or_default();

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

    commits
}

pub fn calculate_file_associations(
    commits: &[HashSet<String>],
    threshold: f64,
    min_commits: usize,
) -> HashMap<String, HashMap<String, f64>> {
    let mut file_commit_count: HashMap<String, usize> = HashMap::new();
    let mut file_pair_count: HashMap<String, HashMap<String, usize>> = HashMap::new();

    for commit_files in commits {
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

pub fn get_changed_files(base: &str) -> HashSet<String> {
    git(&["diff", "--name-only", base])
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect()
}

fn file_exists_on_branch(path: &str, base: &str) -> bool {
    let spec = format!("{base}:{path}");
    Command::new("git")
        .args(["cat-file", "-e", &spec])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn output_changed_associations(
    associations: &HashMap<String, HashMap<String, f64>>,
    changed_files: &HashSet<String>,
    max_files: Option<usize>,
    base: &str,
) {
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
        if !Path::new(path).exists() && !file_exists_on_branch(path, base) {
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

    let commits = get_commits_with_files(num_commits);
    if commits.is_empty() {
        eprintln!("No commits found or no files changed in the analyzed commits.");
        return Ok(());
    }

    let associations = calculate_file_associations(&commits, threshold, min_commits);
    let changed_files = get_changed_files(&base);

    if changed_files.is_empty() {
        eprintln!("No files changed compared to {base}.");
        return Ok(());
    }

    output_changed_associations(&associations, &changed_files, max_files, &base);

    Ok(())
}
