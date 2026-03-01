use std::collections::HashSet;
use std::process::{self, Command};

use crate::cochanges;

struct Context {
    branch: String,
    commits: Vec<(String, String)>,
    files: Vec<String>,
    diff: String,
    truncated: bool,
    truncated_files: Vec<String>,
    cochanges: Vec<(String, f64)>,
}

fn fatal(msg: &str) -> ! {
    eprintln!("gitcontext: {msg}");
    process::exit(1);
}

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

fn gather(
    base: &str,
    max_total: usize,
    max_file: usize,
    stat: bool,
    include_cochanges: bool,
) -> Context {
    git(&["rev-parse", "--git-dir"]).unwrap_or_else(|_| fatal("not in a git repository"));

    git(&["rev-parse", "--verify", base])
        .unwrap_or_else(|_| fatal(&format!("base branch \"{base}\" does not exist")));

    let branch = git(&["branch", "--show-current"])
        .unwrap_or_else(|e| fatal(&format!("getting branch: {e}")))
        .trim()
        .to_string();

    let log_out = git(&["log", &format!("{base}..HEAD"), "--format=%h %s"]).unwrap_or_default();
    let commits: Vec<(String, String)> = log_out
        .lines()
        .filter(|l| !l.is_empty())
        .filter_map(|l| {
            let (hash, subject) = l.split_once(' ')?;
            Some((hash.to_string(), subject.to_string()))
        })
        .collect();

    let files_out = git(&["diff", &format!("{base}...HEAD"), "--name-only"]).unwrap_or_default();
    let files: Vec<String> = files_out
        .lines()
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect();

    let (diff, truncated, truncated_files) = if stat {
        let stat_out = git(&["diff", &format!("{base}...HEAD"), "--stat"]).unwrap_or_default();
        (stat_out, false, vec![])
    } else {
        let diff_raw = git(&["diff", &format!("{base}...HEAD")]).unwrap_or_default();
        let diff_lines = diff_raw.lines().count();
        if diff_lines > max_total {
            truncate_diff(&diff_raw, max_file)
        } else {
            (diff_raw, false, vec![])
        }
    };

    let cochange_list = if include_cochanges {
        let changed_set: HashSet<String> = files.iter().cloned().collect();
        if changed_set.is_empty() {
            vec![]
        } else {
            let commit_data = cochanges::get_commits_with_files(10000).unwrap_or_default();
            let associations = cochanges::calculate_file_associations(&commit_data, 0.3, 5);
            cochanges::collect_changed_associations(&associations, &changed_set, Some(20), base)
        }
    } else {
        vec![]
    };

    Context {
        branch,
        commits,
        files,
        diff,
        truncated,
        truncated_files,
        cochanges: cochange_list,
    }
}

fn split_diff(raw: &str) -> Vec<&str> {
    let marker = "diff --git ";
    let mut sections: Vec<&str> = Vec::new();
    let mut indices: Vec<usize> = Vec::new();

    let mut start = 0;
    while let Some(pos) = raw[start..].find(marker) {
        indices.push(start + pos);
        start = start + pos + marker.len();
    }

    for (i, &idx) in indices.iter().enumerate() {
        let end = indices.get(i + 1).copied().unwrap_or(raw.len());
        sections.push(&raw[idx..end]);
    }

    sections
}

fn extract_filename(header_line: &str) -> String {
    let trimmed = header_line.trim();
    if let Some(after) = trimmed.strip_prefix("diff --git ")
        && let Some((a_path, _)) = after.split_once(' ')
    {
        return a_path.strip_prefix("a/").unwrap_or(a_path).to_string();
    }
    "<unknown>".to_string()
}

fn truncate_diff(raw: &str, max_file: usize) -> (String, bool, Vec<String>) {
    let sections = split_diff(raw);
    let mut truncated_files = Vec::new();
    let mut result = String::new();
    let mut any_truncated = false;

    for section in sections {
        let lines: Vec<&str> = section.lines().collect();

        if lines.len() <= max_file {
            result.push_str(section);
            if !section.ends_with('\n') {
                result.push('\n');
            }
            continue;
        }

        any_truncated = true;
        let fname = extract_filename(lines.first().unwrap_or(&""));
        truncated_files.push(fname);

        let keep = 50;
        let omitted = lines.len() - 2 * keep;

        for line in &lines[..keep] {
            result.push_str(line);
            result.push('\n');
        }
        result.push_str(&format!(
            "... [truncated: {omitted} lines omitted, use Read tool for full file] ...\n"
        ));
        for line in &lines[lines.len() - keep..] {
            result.push_str(line);
            result.push('\n');
        }
    }

    (result, any_truncated, truncated_files)
}

fn print_text(ctx: &Context) {
    println!("## Branch\n{}\n", ctx.branch);

    println!("## Commits");
    for (hash, subject) in &ctx.commits {
        println!("{hash} {subject}");
    }
    println!();

    println!("## Changed Files");
    for f in &ctx.files {
        println!("{f}");
    }
    println!();

    println!("## Diff");
    print!("{}", ctx.diff);

    if !ctx.cochanges.is_empty() {
        println!("\n## Cochanges");
        for (path, fraction) in &ctx.cochanges {
            println!("{fraction:.1} {path}");
        }
    }
}

fn print_json(ctx: &Context) {
    let json_str = |s: &str| -> String {
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    };

    println!("{{");
    println!("  \"branch\": \"{}\",", json_str(&ctx.branch));

    println!("  \"commits\": [");
    for (i, (hash, subject)) in ctx.commits.iter().enumerate() {
        let comma = if i < ctx.commits.len() - 1 { "," } else { "" };
        println!(
            "    {{\"hash\": \"{}\", \"subject\": \"{}\"}}{comma}",
            json_str(hash),
            json_str(subject)
        );
    }
    println!("  ],");

    println!("  \"files\": [");
    for (i, f) in ctx.files.iter().enumerate() {
        let comma = if i < ctx.files.len() - 1 { "," } else { "" };
        println!("    \"{}\"{comma}", json_str(f));
    }
    println!("  ],");

    println!("  \"diff\": \"{}\",", json_str(&ctx.diff));
    println!("  \"truncated\": {},", ctx.truncated);

    println!("  \"truncated_files\": [");
    for (i, f) in ctx.truncated_files.iter().enumerate() {
        let comma = if i < ctx.truncated_files.len() - 1 {
            ","
        } else {
            ""
        };
        println!("    \"{}\"{comma}", json_str(f));
    }
    println!("  ],");

    println!("  \"cochanges\": [");
    for (i, (path, fraction)) in ctx.cochanges.iter().enumerate() {
        let comma = if i < ctx.cochanges.len() - 1 { "," } else { "" };
        println!(
            "    {{\"file\": \"{}\", \"score\": {:.1}}}{comma}",
            json_str(path),
            fraction
        );
    }
    println!("  ]");

    println!("}}");
}

pub fn run(
    base: String,
    format: String,
    max_total: usize,
    max_file: usize,
    stat: bool,
    include_cochanges: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if format != "text" && format != "json" {
        eprintln!("invalid format \"{format}\": must be \"text\" or \"json\"");
        std::process::exit(1);
    }

    let ctx = gather(&base, max_total, max_file, stat, include_cochanges);

    match format.as_str() {
        "json" => print_json(&ctx),
        _ => print_text(&ctx),
    }

    Ok(())
}
