use std::env;
use std::process::{self, Command};

struct Context {
    branch: String,
    commits: Vec<(String, String)>, // (hash, subject)
    files: Vec<String>,
    diff: String,
    truncated: bool,
    truncated_files: Vec<String>,
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

fn gather(base: &str, max_total: usize, max_file: usize) -> Context {
    // Verify git repo
    git(&["rev-parse", "--git-dir"]).unwrap_or_else(|_| fatal("not in a git repository"));

    // Verify base exists
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

    let diff_raw = git(&["diff", &format!("{base}...HEAD")]).unwrap_or_default();

    let diff_lines = diff_raw.lines().count();
    let (diff, truncated, truncated_files) = if diff_lines > max_total {
        truncate_diff(&diff_raw, max_file)
    } else {
        (diff_raw, false, vec![])
    };

    Context {
        branch,
        commits,
        files,
        diff,
        truncated,
        truncated_files,
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

    // commits
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

    // files
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
    println!("  ]");

    println!("}}");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut base = "main".to_string();
    let mut format = "text".to_string();
    let mut max_total: usize = 3000;
    let mut max_file: usize = 200;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                println!(
                    "Usage: gitcontext [flags]\n\n\
                     Gather branch context (diff, log, files) for Claude Code skills.\n\n\
                     Flags:\n  \
                     --base <branch>    Base branch for comparison (default: main)\n  \
                     --format <fmt>     Output format: text or json (default: text)\n  \
                     --max-total <n>    Max total diff lines before truncation (default: 3000)\n  \
                     --max-file <n>     Per-file diff line threshold (default: 200)"
                );
                return;
            }
            "--base" => {
                i += 1;
                base = args.get(i).cloned().unwrap_or(base);
            }
            "--format" => {
                i += 1;
                format = args.get(i).cloned().unwrap_or(format);
            }
            "--max-total" => {
                i += 1;
                max_total = args
                    .get(i)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(max_total);
            }
            "--max-file" => {
                i += 1;
                max_file = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(max_file);
            }
            _ => {}
        }
        i += 1;
    }

    if format != "text" && format != "json" {
        fatal(&format!(
            "invalid format \"{format}\": must be \"text\" or \"json\""
        ));
    }

    let ctx = gather(&base, max_total, max_file);

    match format.as_str() {
        "json" => print_json(&ctx),
        _ => print_text(&ctx),
    }
}
