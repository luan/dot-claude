use clap::CommandFactory;
use clap_complete::{Shell, generate};

use super::Cli;

pub fn run_slug(words: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    if words.is_empty() {
        return Ok(());
    }
    let input = words.join(" ");
    let result = crate::slug::slug(&input);
    if !result.is_empty() {
        println!("{result}");
    }
    Ok(())
}

pub fn run_completion(shell: Shell) -> Result<(), Box<dyn std::error::Error>> {
    generate(shell, &mut Cli::command(), "ct", &mut std::io::stdout());
    Ok(())
}

pub fn run_cochanges(
    base: String,
    threshold: f64,
    min_commits: usize,
    max_files_str: String,
    num_commits: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let max_files = if max_files_str.to_lowercase() == "all" {
        None
    } else {
        let n: usize = max_files_str
            .parse()
            .map_err(|_| format!("invalid max-files: {max_files_str}"))?;
        if n == 0 {
            return Err("max-files must be positive or 'all'".into());
        }
        Some(n)
    };
    crate::cochanges::run(base, threshold, min_commits, max_files, num_commits)
}

pub fn run_apply_patch(
    cwd: Option<String>,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::IsTerminal;
    use std::io::Read;
    use std::path::PathBuf;

    let cwd_path = match cwd {
        Some(s) => PathBuf::from(s),
        None => std::env::current_dir()?,
    };
    if !cwd_path.is_dir() {
        eprintln!(
            "apply-patch: cwd is not a directory: {}",
            cwd_path.display()
        );
        std::process::exit(1);
    }

    if std::io::stdin().is_terminal() {
        eprintln!("apply-patch: expected patch on stdin");
        std::process::exit(1);
    }
    let limit = crate::apply_patch::MAX_PATCH_SIZE_BYTES as u64 + 1;
    let mut patch = String::new();
    std::io::stdin()
        .lock()
        .take(limit)
        .read_to_string(&mut patch)?;
    if patch.len() > crate::apply_patch::MAX_PATCH_SIZE_BYTES {
        eprintln!(
            "apply-patch: patch exceeds {} byte limit",
            crate::apply_patch::MAX_PATCH_SIZE_BYTES
        );
        std::process::exit(1);
    }

    let changes = match crate::apply_patch::apply(&patch, &cwd_path, dry_run) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };

    if dry_run {
        let mut first = true;
        for change in &changes {
            if !first {
                println!();
            }
            first = false;
            print!("{}", change.unified_diff);
        }
    } else {
        for change in &changes {
            match change.kind {
                crate::apply_patch::ChangeType::Add => println!("A {}", change.path),
                crate::apply_patch::ChangeType::Update => println!("M {}", change.path),
                crate::apply_patch::ChangeType::Delete => println!("D {}", change.path),
                crate::apply_patch::ChangeType::Move => {
                    let dest = change.move_path.as_deref().unwrap_or("");
                    println!("R {} \u{2192} {}", change.path, dest);
                }
            }
        }
    }
    Ok(())
}
