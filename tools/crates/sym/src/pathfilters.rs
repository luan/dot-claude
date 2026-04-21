use std::path::Path;

pub fn widen_path_filter_limit(limit: usize, has_filters: bool) -> usize {
    if !has_filters {
        return limit;
    }
    if limit == 0 {
        return 500;
    }
    (limit * 5).clamp(100, 1000)
}

pub fn include_path(rel_path: &Path, includes: &[String], excludes: &[String]) -> bool {
    if !includes.is_empty() && !matches_any_path(rel_path, includes) {
        return false;
    }
    if !excludes.is_empty() && matches_any_path(rel_path, excludes) {
        return false;
    }
    true
}

fn matches_any_path(rel_path: &Path, patterns: &[String]) -> bool {
    let rel = rel_path.to_string_lossy().replace('\\', "/");
    patterns
        .iter()
        .map(|pattern| pattern.trim())
        .filter(|pattern| !pattern.is_empty())
        .any(|pattern| glob_match(pattern, &rel) || rel.contains(pattern))
}

fn glob_match(pattern: &str, rel: &str) -> bool {
    let pattern = pattern.as_bytes();
    let text = rel.as_bytes();
    let mut dp = vec![vec![false; text.len() + 1]; pattern.len() + 1];
    dp[0][0] = true;

    for i in 1..=pattern.len() {
        if pattern[i - 1] == b'*' {
            dp[i][0] = dp[i - 1][0];
        }
    }

    for i in 1..=pattern.len() {
        for j in 1..=text.len() {
            dp[i][j] = match pattern[i - 1] {
                b'*' => dp[i - 1][j] || dp[i][j - 1],
                b'?' => dp[i - 1][j - 1],
                ch => ch == text[j - 1] && dp[i - 1][j - 1],
            };
        }
    }

    dp[pattern.len()][text.len()]
}
