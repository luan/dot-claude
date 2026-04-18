mod ansi;
mod apply_patch;
mod artifact;
mod churn;
mod cli;
mod cochanges;
mod gitcontext;
mod mcp;
mod notify;
mod phases;
mod refs;
mod slug;
mod vault;

use clap::{CommandFactory, Parser};

fn dispatch_artifact(
    kind: artifact::ArtifactKind,
    action: cli::ArtifactAction,
) -> Result<(), Box<dyn std::error::Error>> {
    let cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    match action {
        cli::ArtifactAction::List {
            json,
            all,
            project,
            archived,
            include_dives,
        } => cli::run_artifact_list(kind, &cwd, json, all, project, archived, include_dives),
        cli::ArtifactAction::Create {
            topic,
            project,
            slug,
            source,
            tags,
            dive,
        } => cli::run_artifact_create(cli::ArtifactCreateArgs {
            kind,
            topic,
            project,
            slug,
            source,
            tags,
            dive,
        }),
        cli::ArtifactAction::Read { file, frontmatter } => {
            cli::run_artifact_read(kind, file, frontmatter)
        }
        cli::ArtifactAction::Latest {
            project,
            task_file,
            include_dives,
        } => cli::run_artifact_latest(kind, project, task_file, include_dives),
        cli::ArtifactAction::Archive {
            file,
            batch,
            dry_run,
        } => cli::run_artifact_archive(kind, file, batch, dry_run),
        cli::ArtifactAction::Show { id } => cli::run_artifact_show(kind, &id),
        cli::ArtifactAction::Prune {
            days,
            dry_run,
            project,
        } => cli::run_artifact_prune(kind, days, dry_run, project),
        cli::ArtifactAction::Comments { file, json } => {
            cli::run_artifact_comments(kind, file, json)
        }
        cli::ArtifactAction::Rename { old, new_slug } => {
            cli::run_artifact_rename(kind, old, new_slug)
        }
        cli::ArtifactAction::Retag { file } => cli::run_artifact_retag(kind, file),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = cli::Cli::parse();

    match cli.command {
        None => {
            cli::Cli::command().print_help()?;
            println!();
            Ok(())
        }
        Some(cli::Command::Plan { action }) => {
            dispatch_artifact(artifact::ArtifactKind::Plan, action)
        }
        Some(cli::Command::Spec { action }) => {
            dispatch_artifact(artifact::ArtifactKind::Spec, action)
        }
        Some(cli::Command::Review { action }) => {
            dispatch_artifact(artifact::ArtifactKind::Review, action)
        }
        Some(cli::Command::Report { action }) => {
            dispatch_artifact(artifact::ArtifactKind::Report, action)
        }
        Some(cli::Command::Doc { action }) => {
            dispatch_artifact(artifact::ArtifactKind::Doc, action)
        }
        Some(cli::Command::Vault { action }) => match action {
            cli::VaultAction::Init => {
                vault::cmd_init();
                Ok(())
            }
            cli::VaultAction::Migrate => {
                vault::cmd_migrate();
                Ok(())
            }
            cli::VaultAction::Project => {
                vault::cmd_project();
                Ok(())
            }
            cli::VaultAction::Related {
                project,
                topic,
                archive,
            } => {
                let project = project.unwrap_or_else(artifact::current_project);
                vault::cmd_related(&project, &topic, archive);
                Ok(())
            }
            cli::VaultAction::Check { archive } => {
                vault::cmd_check(archive);
                Ok(())
            }
            cli::VaultAction::Search {
                query,
                json,
                r#type,
                project,
                archive,
            } => {
                let kind = r#type.as_deref().and_then(|k| match k {
                    "spec" => Some(artifact::ArtifactKind::Spec),
                    "plan" => Some(artifact::ArtifactKind::Plan),
                    "review" => Some(artifact::ArtifactKind::Review),
                    "report" => Some(artifact::ArtifactKind::Report),
                    "doc" => Some(artifact::ArtifactKind::Doc),
                    _ => None,
                });
                vault::cmd_search(&query, json, kind, project.as_deref(), archive);
                Ok(())
            }
            cli::VaultAction::Status => {
                vault::cmd_status();
                Ok(())
            }
        },
        Some(cli::Command::Read { file, frontmatter }) => {
            let resolved = match artifact::resolve_stem_universal(&file) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(1);
                }
            };
            artifact::cmd_read_resolved(&resolved, frontmatter);
            Ok(())
        }
        Some(cli::Command::Notify) => notify::run(),
        Some(cli::Command::Mcp { action }) => match action {
            cli::McpAction::Blueprint => mcp::run_blueprint_server(),
            cli::McpAction::ApplyPatch => mcp::run_apply_patch_server(),
        },
        Some(cli::Command::Tool { action }) => match action {
            cli::ToolAction::Slug { words } => cli::run_slug(words),
            cli::ToolAction::Phases { file } => phases::run_phases(file),
            cli::ToolAction::Completion { shell } => cli::run_completion(shell),
            cli::ToolAction::Gitcontext {
                base,
                format,
                max_total,
                max_file,
                stat,
                cochanges,
            } => gitcontext::run(base, format, max_total, max_file, stat, cochanges),
            cli::ToolAction::CheckRefs { file, project_root } => refs::run(file, project_root),
            cli::ToolAction::Cochanges {
                base,
                threshold,
                min_commits,
                max_files,
                num_commits,
            } => cli::run_cochanges(base, threshold, min_commits, max_files, num_commits),
            cli::ToolAction::Churn {
                project_root,
                since,
                min_loc,
            } => churn::run(project_root, since, min_loc),
            cli::ToolAction::ApplyPatch { cwd, dry_run } => cli::run_apply_patch(cwd, dry_run),
        },
        Some(cli::Command::ApplyPatch { cmd }) => match cmd {
            cli::ApplyPatchCmd::Stats { all_projects, days } => {
                cli::run_apply_patch_stats(all_projects, days)
            }
            cli::ApplyPatchCmd::Prune { days } => cli::run_apply_patch_prune(days),
        },
    }
}
