use super::{handle_sync_error, truncate_at_char_boundary};
use crate::ansi;

pub fn run_artifact_list(
    kind: crate::artifact::ArtifactKind,
    cwd: &str,
    json: bool,
    all: bool,
    project: Option<String>,
    archived: bool,
    include_dives: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut items = if archived {
        crate::artifact::list_archived_artifacts(kind)
    } else {
        crate::artifact::list_artifacts(kind, include_dives)
    };

    items.retain(|a| !a.project.is_empty());

    if let Some(ref proj) = project {
        let resolved = crate::artifact::resolve_repo_root(proj);
        items.retain(|a| a.project.contains(resolved.as_str()));
    } else if !all {
        let resolved_cwd = crate::artifact::resolve_repo_root(cwd);
        items.retain(|a| resolved_cwd.contains(&a.project));
    }

    let label = kind.dir_name();

    if items.is_empty() {
        if all {
            eprintln!(
                "{}",
                ansi::dim(&format!("No {label}s found in ~/blueprints/"))
            );
        } else {
            eprintln!(
                "{}",
                ansi::dim(&format!(
                    "No {label}s found for current project. Use --all to show all {label}s."
                ))
            );
        }
        return Ok(());
    }

    if json {
        let json_items: Vec<_> = items
            .iter()
            .map(|a| {
                serde_json::json!({
                    "name": a.name,
                    "title": a.title,
                    "project": crate::artifact::project_name(&a.project),
                    "modified": crate::artifact::format_date(a.mod_time),
                    "size": crate::artifact::format_size(a.size),
                    "created": a.created,
                    "source": a.source,
                    "tags": a.tags,
                    "author": a.author,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&json_items)?);
    } else {
        println!(
            "{}",
            ansi::bold(&format!(
                "{:<12} {:<30} {:<42} {:<12} SIZE",
                "PROJECT", "NAME", "TITLE", "MODIFIED"
            ))
        );
        println!("{}", ansi::dim(&"-".repeat(100)));

        for a in &items {
            let proj = crate::artifact::project_name(&a.project);

            let name = if a.name.len() > 28 {
                format!("{}...", truncate_at_char_boundary(&a.name, 25))
            } else {
                a.name.clone()
            };

            let title = if a.title.len() > 40 {
                format!("{}...", truncate_at_char_boundary(&a.title, 37))
            } else {
                a.title.clone()
            };

            let title_col = format!("{:<42}", title);
            println!(
                "{} {} {} {} {}",
                ansi::id(&format!("{:<12}", proj)),
                ansi::dim(&format!("{:<30}", name)),
                title_col,
                ansi::dim(&format!("{:<12}", crate::artifact::format_date(a.mod_time))),
                ansi::dim(&crate::artifact::format_size(a.size))
            );
        }
    }

    Ok(())
}

pub fn run_artifact_show(
    kind: crate::artifact::ArtifactKind,
    id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let items = crate::artifact::list_artifacts(kind, true);
    let label = kind.dir_name();

    if items.is_empty() {
        eprintln!(
            "{}",
            ansi::dim(&format!("No {label}s found in ~/blueprints/"))
        );
        return Ok(());
    }

    let normalized_id = id.strip_suffix(".md").unwrap_or(id);

    let found = items.iter().find(|a| {
        a.name == normalized_id || a.name == id || a.path.file_name().is_some_and(|f| f == id)
    });

    let Some(artifact_ref) = found else {
        eprintln!("{label} not found: {id}");
        return Ok(());
    };

    let content = crate::artifact::load_content(kind, &artifact_ref.path);
    println!("{content}");

    Ok(())
}

pub struct ArtifactCreateArgs {
    pub kind: crate::artifact::ArtifactKind,
    pub topic: String,
    pub project: Option<String>,
    pub slug: Option<String>,
    pub source: Option<String>,
    pub tags: Option<String>,
    pub dive: bool,
}

pub fn run_artifact_create(args: ArtifactCreateArgs) -> Result<(), Box<dyn std::error::Error>> {
    let ArtifactCreateArgs {
        kind,
        topic,
        project,
        slug,
        source,
        tags,
        dive,
    } = args;
    let tag_list: Vec<String> = tags
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let project = project.unwrap_or_else(crate::artifact::current_project);
    match crate::artifact::create(crate::artifact::CreateOpts {
        kind,
        topic: &topic,
        project: &project,
        slug_override: slug.as_deref(),
        source: source.as_deref(),
        user_tags: &tag_list,
        dive,
    }) {
        Ok(outcome) => {
            println!("{}", outcome.path.display());
        }
        Err(crate::artifact::CtError::Sync(e)) => handle_sync_error(e),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
    Ok(())
}

pub fn run_artifact_read(
    kind: crate::artifact::ArtifactKind,
    file: String,
    frontmatter: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    crate::artifact::cmd_read(&file, kind, frontmatter);
    Ok(())
}

pub fn run_artifact_comments(
    kind: crate::artifact::ArtifactKind,
    file: String,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    crate::artifact::cmd_comments(&file, kind, json);
    Ok(())
}

pub fn run_artifact_latest(
    kind: crate::artifact::ArtifactKind,
    project: Option<String>,
    task_file: Option<String>,
    include_dives: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    crate::artifact::cmd_latest(
        kind,
        project.as_deref(),
        task_file.as_deref(),
        include_dives,
    );
    Ok(())
}

pub fn run_artifact_archive(
    kind: crate::artifact::ArtifactKind,
    file: Option<String>,
    batch: Vec<String>,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if !batch.is_empty() {
        if let Err(e) = crate::artifact::cmd_archive_batch(kind, &batch, dry_run) {
            handle_sync_error(e);
        }
    } else if let Some(f) = file {
        if let Err(e) = crate::artifact::cmd_archive(kind, &f, dry_run) {
            handle_sync_error(e);
        }
    } else {
        eprintln!(
            "Usage: ct <type> archive <file> or ct <type> archive --batch <file1> <file2> ..."
        );
        std::process::exit(1);
    }
    Ok(())
}

pub fn run_artifact_prune(
    kind: crate::artifact::ArtifactKind,
    days: u64,
    dry_run: bool,
    project: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let bp = crate::artifact::blueprints_dir();
    let kind_dir = kind.dir_name();
    let threshold = std::time::Duration::from_secs(days * 86400);
    let now = std::time::SystemTime::now();
    let mut archived_count = 0u32;
    let mut sync_errors = 0u32;

    let Ok(project_dirs) = std::fs::read_dir(&bp) else {
        eprintln!(
            "{}",
            ansi::dim(&format!("No {kind_dir} found in ~/blueprints/"))
        );
        return Ok(());
    };

    for dir_entry in project_dirs.flatten() {
        if !dir_entry.path().is_dir() {
            continue;
        }
        let dir_name = dir_entry.file_name().to_string_lossy().to_string();
        if dir_name == "archive" {
            continue;
        }
        if let Some(ref proj) = project {
            let resolved = crate::artifact::project_name(&crate::artifact::resolve_repo_root(proj));
            if !dir_name.contains(resolved.as_str()) {
                continue;
            }
        }

        let mut scan_dirs = vec![dir_entry.path().join(kind_dir)];
        if kind == crate::artifact::ArtifactKind::Spec {
            scan_dirs.push(dir_entry.path().join("dive"));
        }
        for artifact_dir in scan_dirs {
            let Ok(files) = std::fs::read_dir(&artifact_dir) else {
                continue;
            };
            for file_entry in files.flatten() {
                let path = file_entry.path();
                if path.is_dir() || path.extension().is_none_or(|ext| ext != "md") {
                    continue;
                }
                let Ok(meta) = file_entry.metadata() else {
                    continue;
                };
                let Ok(modified) = meta.modified() else {
                    continue;
                };
                let Ok(age) = now.duration_since(modified) else {
                    continue;
                };
                if age < threshold {
                    continue;
                }

                let path_str = path.to_string_lossy().to_string();
                if dry_run {
                    println!("would archive: {path_str}");
                } else {
                    match crate::artifact::cmd_archive(kind, &path_str, false) {
                        Ok(()) => archived_count += 1,
                        Err(e) => {
                            eprintln!("{e}");
                            sync_errors += 1;
                        }
                    }
                }
            }
        }
    }

    if !dry_run && archived_count > 0 {
        println!("Archived {archived_count} {} file(s)", kind.dir_name());
    }
    if sync_errors > 0 {
        eprintln!("{sync_errors} file(s) failed to sync");
        std::process::exit(2);
    }

    Ok(())
}

pub fn run_artifact_rename(
    kind: crate::artifact::ArtifactKind,
    old: String,
    new_slug: String,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Err(e) = crate::artifact::cmd_rename(kind, &old, &new_slug) {
        handle_sync_error(e);
    }
    Ok(())
}

pub fn run_artifact_retag(
    kind: crate::artifact::ArtifactKind,
    file: String,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Err(e) = crate::artifact::cmd_retag(kind, &file) {
        handle_sync_error(e);
    }
    Ok(())
}
