use super::archive::{archive, cmd_archive};
use super::crud::{
    Comment, CreateOpts, cmd_retag, create, parse_comments, read, resolve_artifact_path,
    resolve_stem_universal,
};
use super::listing::{latest_artifact, list_archived_artifacts, list_artifacts};
use super::{
    ArtifactKind, CtError, ResolveError, SyncError, artifact_dir_with_base, chrono_rfc3339, env,
    extract_frontmatter_full_from_str, fs, parse_frontmatter, parse_yaml_map, project_name,
    strip_date_prefix, yaml_quote,
};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

// Serialize all tests that mutate CT_BLUEPRINTS_DIR to prevent env-var races.
static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn worktrees_share_repo_name() {
    assert_eq!(project_name("/Users/me/src/repo.git/wt1"), "repo");
    assert_eq!(project_name("/Users/me/src/repo.git/wt2"), "repo");
}

#[test]
fn bare_git_dir_uses_stem() {
    assert_eq!(project_name("/Users/me/src/repo.git"), "repo");
}

#[test]
fn nested_worktree_uses_repo_name() {
    assert_eq!(project_name("/Users/me/src/mono.git/apps/web"), "mono");
}

#[test]
fn normal_path_uses_last_component() {
    assert_eq!(project_name("/Users/me/src/myapp/src/core"), "core");
}

#[test]
fn dots_replaced_with_underscores() {
    assert_eq!(project_name("/Users/me/src/.claude"), "_claude");
    assert_eq!(project_name("/Users/me/src/my.project"), "my_project");
}

#[test]
fn task_file_returns_specified_path() {
    let tmp = std::env::temp_dir().join(format!("ck-latest-test-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();

    let plan = tmp.join("my-plan.md");
    std::fs::write(&plan, "# plan\n").unwrap();

    let result = latest_artifact(ArtifactKind::Plan, Some(plan.to_str().unwrap()), "", false);
    assert!(result.is_ok(), "expected Ok, got {result:?}");
    assert_eq!(
        result.unwrap().canonicalize().unwrap(),
        plan.canonicalize().unwrap(),
        "--task-file should return the specified path"
    );

    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn task_file_flag_errors_when_file_missing() {
    let result = latest_artifact(
        ArtifactKind::Plan,
        Some("/nonexistent/path/plan.md"),
        "",
        false,
    );
    assert!(result.is_err(), "expected Err for missing task-file");
    let msg = result.unwrap_err();
    assert!(
        msg.contains("task-file not found"),
        "error message should mention task-file, got: {msg}"
    );
}

#[test]
fn frontmatter_has_no_status_field() {
    let tmp = std::env::temp_dir().join(format!("ck-test-{}", std::process::id()));
    let project_path = "/some/project";

    let project_dir = artifact_dir_with_base(project_path, "plans", &tmp);
    std::fs::create_dir_all(&project_dir).unwrap();

    let slug = crate::slug::slug("Test Topic");
    let file_path = project_dir.join(format!("{slug}.md"));

    let now = chrono_rfc3339();
    let mut buf = String::new();
    buf.push_str("---\n");
    buf.push_str(&format!("topic: {}\n", yaml_quote("Test Topic")));
    buf.push_str(&format!("created: {now}\n"));
    buf.push_str("---\n");
    std::fs::write(&file_path, &buf).unwrap();

    let content = std::fs::read_to_string(&file_path).unwrap();

    let (yaml, _) = parse_frontmatter(&content);
    let yaml = yaml.expect("frontmatter must be present");
    let keys: Vec<_> = parse_yaml_map(yaml).into_iter().map(|(k, _)| k).collect();
    assert!(
        !keys.contains(&"status".to_string()),
        "frontmatter must not contain a 'status' field, got keys: {keys:?}"
    );

    std::fs::remove_dir_all(&tmp).ok();
}

fn create_artifact_file(base: &Path, project: &str, kind: ArtifactKind, stem: &str) -> PathBuf {
    let dir = base.join(project).join(kind.dir_name());
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join(format!("{stem}.md"));
    std::fs::write(&file, "# test\n").unwrap();
    file
}

#[test]
fn universal_resolve_picks_highest_priority_kind() {
    let tmp = std::env::temp_dir().join(format!("ct-univ-prio-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();

    let _spec = create_artifact_file(&tmp, "myproj", ArtifactKind::Spec, "widget");
    let doc = create_artifact_file(&tmp, "myproj", ArtifactKind::Doc, "widget");

    with_blueprints_dir(&tmp, || {
        let result = resolve_stem_universal("widget").expect("resolve widget");
        assert_eq!(result, doc, "Doc should take priority over Spec");
    });
    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn universal_resolve_single_match() {
    let tmp = std::env::temp_dir().join(format!("ct-univ-single-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();

    let plan = create_artifact_file(&tmp, "myproj", ArtifactKind::Plan, "deploy");

    with_blueprints_dir(&tmp, || {
        let result = resolve_stem_universal("deploy").expect("resolve deploy");
        assert_eq!(result, plan);
    });
    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn universal_resolve_report_over_plan() {
    let tmp = std::env::temp_dir().join(format!("ct-univ-rp-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();

    let _plan = create_artifact_file(&tmp, "myproj", ArtifactKind::Plan, "auth");
    let report = create_artifact_file(&tmp, "myproj", ArtifactKind::Report, "auth");

    with_blueprints_dir(&tmp, || {
        let result = resolve_stem_universal("auth").expect("resolve auth");
        assert_eq!(result, report, "Report should take priority over Plan");
    });
    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn frontmatter_full_all_fields() {
    let content = "\
---
topic: \"My Widget\"
project: myproj
created: 2026-01-15T10:30:00Z
source: \"[[some-spec]]\"
tags:
  - type/plan
  - domain/combat
  - stage/implementing
author: \"Luan\"
---
# Body
";
    let (title, project, created, source, tags, author) =
        extract_frontmatter_full_from_str(content);
    assert_eq!(title, "My Widget");
    assert_eq!(project, "myproj");
    assert_eq!(created.as_deref(), Some("2026-01-15T10:30:00Z"));
    assert_eq!(source.as_deref(), Some("some-spec"));
    assert_eq!(
        tags,
        vec!["type/plan", "domain/combat", "stage/implementing"]
    );
    assert_eq!(author.as_deref(), Some("Luan"));
}

#[test]
fn frontmatter_full_optional_fields_missing() {
    let content = "\
---
topic: Minimal
project: proj
---
";
    let (title, project, created, source, tags, author) =
        extract_frontmatter_full_from_str(content);
    assert_eq!(title, "Minimal");
    assert_eq!(project, "proj");
    assert!(created.is_none());
    assert!(source.is_none());
    assert!(tags.is_empty());
    assert!(author.is_none());
}

#[test]
fn frontmatter_full_tags_list() {
    let content = "\
---
topic: Tags
project: p
tags:
  - alpha
  - \"beta\"
  - 'gamma'
---
";
    let (_, _, _, _, tags, _) = extract_frontmatter_full_from_str(content);
    assert_eq!(tags, vec!["alpha", "beta", "gamma"]);
}

#[test]
fn frontmatter_full_source_wiki_link_brackets() {
    let content = "\
---
topic: Linked
project: p
source: \"[[my-source-spec]]\"
---
";
    let (_, _, _, source, _, _) = extract_frontmatter_full_from_str(content);
    assert_eq!(source.as_deref(), Some("my-source-spec"));
}

#[test]
fn frontmatter_full_source_without_brackets() {
    let content = "\
---
topic: Plain
project: p
source: plain-ref
---
";
    let (_, _, _, source, _, _) = extract_frontmatter_full_from_str(content);
    assert_eq!(source.as_deref(), Some("plain-ref"));
}

#[test]
fn frontmatter_full_falls_back_to_h1() {
    let content = "# Heading Title\nsome body\n";
    let (title, project, created, source, tags, author) =
        extract_frontmatter_full_from_str(content);
    assert_eq!(title, "Heading Title");
    assert!(project.is_empty());
    assert!(created.is_none());
    assert!(source.is_none());
    assert!(tags.is_empty());
    assert!(author.is_none());
}

#[test]
fn parse_comments_with_highlight() {
    let comments = parse_comments("==foo==<!--bar-->");
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0].line, 1);
    assert_eq!(comments[0].highlight.as_deref(), Some("foo"));
    assert_eq!(comments[0].text, "bar");
}

#[test]
fn parse_comments_without_highlight() {
    let comments = parse_comments("<!--bar-->");
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0].line, 1);
    assert_eq!(comments[0].highlight, None);
    assert_eq!(comments[0].text, "bar");
}

#[test]
fn parse_comments_multiple_on_one_line() {
    let comments = parse_comments("<!--a--> <!--b-->");
    assert_eq!(comments.len(), 2);
    assert_eq!(comments[0].line, 1);
    assert_eq!(comments[0].text, "a");
    assert_eq!(comments[1].line, 1);
    assert_eq!(comments[1].text, "b");
}

#[test]
fn parse_comments_no_comments() {
    let comments = parse_comments("just text");
    assert!(comments.is_empty());
}

#[test]
fn parse_comments_highlight_without_comment() {
    let comments = parse_comments("==foo==");
    assert!(comments.is_empty());
}

#[test]
fn parse_comments_on_later_line() {
    let comments = parse_comments("line1\nline2\n<!--here-->");
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0].line, 3);
    assert_eq!(comments[0].text, "here");
}

fn with_blueprints_dir<F: FnOnce()>(tmp: &std::path::Path, f: F) {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let prev = env::var("CT_BLUEPRINTS_DIR").ok();
    unsafe { env::set_var("CT_BLUEPRINTS_DIR", tmp) };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
    match prev {
        Some(v) => unsafe { env::set_var("CT_BLUEPRINTS_DIR", v) },
        None => unsafe { env::remove_var("CT_BLUEPRINTS_DIR") },
    }
    if let Err(payload) = result {
        std::panic::resume_unwind(payload);
    }
}

#[test]
fn dive_create_routes_to_dive_folder_with_spec_tag() {
    let tmp = std::env::temp_dir().join(format!("ct-dive-create-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();

    let project = tmp.join("myproj");
    std::fs::create_dir_all(&project).unwrap();

    with_blueprints_dir(&tmp, || {
        let _ = create(CreateOpts {
            kind: ArtifactKind::Spec,
            topic: "Sub Topic A",
            project: project.to_str().unwrap(),
            slug_override: Some("hub-sub-topic-a"),
            source: Some("20260411-hub"),
            user_tags: &[],
            dive: true,
        });

        let dive_dir = tmp.join("myproj").join("dive");
        let spec_dir = tmp.join("myproj").join("spec");

        let dive_files: Vec<_> = fs::read_dir(&dive_dir)
            .expect("dive/ directory must exist")
            .flatten()
            .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("md"))
            .collect();
        assert_eq!(dive_files.len(), 1, "exactly one file in dive/");

        let spec_has_files = fs::read_dir(&spec_dir)
            .map(|d| {
                d.flatten()
                    .any(|e| e.path().extension().and_then(|x| x.to_str()) == Some("md"))
            })
            .unwrap_or(false);
        assert!(!spec_has_files, "spec/ must not contain the dive file");

        let content = fs::read_to_string(dive_files[0].path()).unwrap();
        assert!(
            content.contains("type/spec"),
            "dive file must have type/spec tag"
        );
        assert!(
            content.contains("source: \"[[20260411-hub]]\""),
            "source must be singly wrapped: got\n{content}"
        );
        assert!(
            !content.contains("[[[["),
            "source must not be double-wrapped: got\n{content}"
        );
    });
    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn include_dives_flag_toggles_list_visibility() {
    let tmp = std::env::temp_dir().join(format!("ct-dive-list-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();

    let spec_dir = tmp.join("myproj").join("spec");
    std::fs::create_dir_all(&spec_dir).unwrap();
    std::fs::write(spec_dir.join("20260411-hub.md"), "---\ntopic: Hub\n---\n").unwrap();

    let dive_dir = tmp.join("myproj").join("dive");
    std::fs::create_dir_all(&dive_dir).unwrap();
    std::fs::write(
        dive_dir.join("20260411-hub-sub.md"),
        "---\ntopic: Sub\n---\n",
    )
    .unwrap();

    with_blueprints_dir(&tmp, || {
        let without = list_artifacts(ArtifactKind::Spec, false);
        assert_eq!(
            without.len(),
            1,
            "list without --include-dives should show 1 artifact"
        );
        assert!(
            without[0].path.to_string_lossy().contains("spec/"),
            "should be the spec hub"
        );

        let with_dives = list_artifacts(ArtifactKind::Spec, true);
        assert_eq!(
            with_dives.len(),
            2,
            "list with --include-dives should show 2 artifacts"
        );
    });
    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn archive_dive_lands_in_archive_dive_not_archive_spec() {
    let tmp = std::env::temp_dir().join(format!("ct-dive-archive-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();

    let dive_dir = tmp.join("myproj").join("dive");
    std::fs::create_dir_all(&dive_dir).unwrap();
    let dive_file = dive_dir.join("20260411-hub-sub.md");
    std::fs::write(&dive_file, "---\ntopic: Sub\n---\n").unwrap();

    with_blueprints_dir(&tmp, || {
        let _ = cmd_archive(ArtifactKind::Spec, dive_file.to_str().unwrap(), false);

        let expected = tmp
            .join("myproj")
            .join("archive")
            .join("dive")
            .join("20260411-hub-sub.md");
        let wrong = tmp
            .join("myproj")
            .join("archive")
            .join("spec")
            .join("20260411-hub-sub.md");

        assert!(expected.exists(), "archived dive must be at archive/dive/");
        assert!(
            !wrong.exists(),
            "archived dive must NOT be at archive/spec/"
        );
    });
    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn dive_rejected_on_non_spec_kinds() {
    let tmp = std::env::temp_dir().join(format!("ct-dive-nonspec-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();

    let project = tmp.join("myproj");
    std::fs::create_dir_all(&project).unwrap();

    with_blueprints_dir(&tmp, || {
        let result = create(CreateOpts {
            kind: ArtifactKind::Plan,
            topic: "Some Plan",
            project: project.to_str().unwrap(),
            slug_override: None,
            source: Some("foo"),
            user_tags: &[],
            dive: true,
        });
        assert!(
            result.is_err(),
            "create with dive=true on a non-Spec kind must return Err"
        );
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("dive is only valid for spec artifacts"),
            "error message must mention dive restriction; got: {msg}"
        );
    });
    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn dive_requires_source() {
    let tmp = std::env::temp_dir().join(format!("ct-dive-no-source-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();

    let project = tmp.join("myproj");
    std::fs::create_dir_all(&project).unwrap();

    with_blueprints_dir(&tmp, || {
        let result = create(CreateOpts {
            kind: ArtifactKind::Spec,
            topic: "Orphan Dive",
            project: project.to_str().unwrap(),
            slug_override: None,
            source: None,
            user_tags: &[],
            dive: true,
        });
        assert!(
            result.is_err(),
            "create with dive=true and source=None must return Err"
        );
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("dive requires source"),
            "error message must mention source requirement; got: {msg}"
        );
    });
    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn archived_dive_is_listable() {
    let tmp = std::env::temp_dir().join(format!("ct-dive-archived-list-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();

    let project = tmp.join("myproj");
    std::fs::create_dir_all(&project).unwrap();

    with_blueprints_dir(&tmp, || {
        let dive_dir = tmp.join("myproj").join("dive");
        std::fs::create_dir_all(&dive_dir).unwrap();
        let dive_file = dive_dir.join("20260411-hub-sub.md");
        std::fs::write(&dive_file, "---\ntopic: Sub\ntags:\n  - type/spec\n---\n").unwrap();

        let _ = cmd_archive(ArtifactKind::Spec, dive_file.to_str().unwrap(), false);

        let archived = list_archived_artifacts(ArtifactKind::Spec);
        assert!(
            archived
                .iter()
                .any(|a| a.path.to_string_lossy().contains("archive/dive")),
            "archived dive must be visible in list_archived_artifacts; got: {:?}",
            archived
                .iter()
                .map(|a| a.path.display().to_string())
                .collect::<Vec<_>>()
        );
    });
    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn resolve_artifact_path_finds_dive_by_bare_stem() {
    let tmp = std::env::temp_dir().join(format!("ct-dive-resolve-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();

    let dive_dir = tmp.join("myproj").join("dive");
    std::fs::create_dir_all(&dive_dir).unwrap();
    let dive_file = dive_dir.join("20260411-hub-detail.md");
    std::fs::write(&dive_file, "---\ntopic: Detail\n---\n").unwrap();

    with_blueprints_dir(&tmp, || {
        let resolved = resolve_artifact_path("20260411-hub-detail", ArtifactKind::Spec)
            .expect("resolve dive stem");
        assert!(
            resolved.to_string_lossy().contains("dive/"),
            "resolved path must be inside dive/; got: {}",
            resolved.display()
        );
        assert_eq!(
            resolved.canonicalize().unwrap(),
            dive_file.canonicalize().unwrap(),
            "resolved path must match the dive file"
        );
    });
    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn strip_date_prefix_current_format() {
    assert_eq!(
        strip_date_prefix("20260411-07-kdl-derive-reference"),
        "kdl-derive-reference"
    );
}

#[test]
fn strip_date_prefix_legacy_format() {
    assert_eq!(strip_date_prefix("20260408-foo"), "foo");
}

#[test]
fn strip_date_prefix_no_prefix() {
    assert_eq!(strip_date_prefix("no-prefix-here"), "no-prefix-here");
}

#[test]
fn strip_date_prefix_short() {
    assert_eq!(strip_date_prefix("short"), "short");
}

#[test]
fn strip_date_prefix_empty() {
    assert_eq!(strip_date_prefix(""), "");
}

#[test]
fn resolve_artifact_path_fuzzy_strips_candidate_prefix() {
    let tmp = std::env::temp_dir().join(format!("ct-fuzzy-cand-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();

    let file = create_artifact_file(
        &tmp,
        "myproj",
        ArtifactKind::Doc,
        "20260411-07-kdl-derive-reference",
    );

    with_blueprints_dir(&tmp, || {
        let result = resolve_artifact_path("kdl-derive-reference", ArtifactKind::Doc)
            .expect("resolve kdl stem");
        assert_eq!(result, file);
    });
    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn resolve_artifact_path_exact_takes_priority_over_fuzzy() {
    let tmp = std::env::temp_dir().join(format!("ct-fuzzy-exact-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();

    let exact = create_artifact_file(
        &tmp,
        "myproj",
        ArtifactKind::Doc,
        "20260411-07-kdl-derive-reference",
    );
    let _other = create_artifact_file(
        &tmp,
        "other",
        ArtifactKind::Doc,
        "20260412-07-kdl-derive-reference",
    );

    with_blueprints_dir(&tmp, || {
        let result = resolve_artifact_path("20260411-07-kdl-derive-reference", ArtifactKind::Doc)
            .expect("resolve kdl stem exact");
        assert_eq!(result, exact);
    });
    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn resolve_artifact_path_fuzzy_strips_query_prefix() {
    let tmp = std::env::temp_dir().join(format!("ct-fuzzy-query-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();

    let file = create_artifact_file(&tmp, "myproj", ArtifactKind::Doc, "foo");

    with_blueprints_dir(&tmp, || {
        let result = resolve_artifact_path("20260411-07-foo", ArtifactKind::Doc)
            .expect("resolve foo stem fuzzy");
        assert_eq!(result, file);
    });
    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn universal_resolve_fuzzy_strips_candidate_prefix() {
    let tmp = std::env::temp_dir().join(format!("ct-univ-fuzzy-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();

    let file = create_artifact_file(
        &tmp,
        "myproj",
        ArtifactKind::Doc,
        "20260411-07-kdl-derive-reference",
    );

    with_blueprints_dir(&tmp, || {
        let result = resolve_stem_universal("kdl-derive-reference").expect("universal resolve kdl");
        assert_eq!(result, file);
    });
    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn universal_resolve_fuzzy_strips_query_prefix() {
    let tmp = std::env::temp_dir().join(format!("ct-univ-fuzzy-q-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();

    let file = create_artifact_file(&tmp, "myproj", ArtifactKind::Plan, "bar");

    with_blueprints_dir(&tmp, || {
        let result = resolve_stem_universal("20260411-07-bar").expect("universal resolve bar");
        assert_eq!(result, file);
    });
    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn retag_fixes_wrong_auto_derived_tags() {
    let tmp = std::env::temp_dir().join(format!("ct-retag-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();

    let docs_dir = tmp.join("myproj").join("docs");
    std::fs::create_dir_all(&docs_dir).unwrap();

    let file = docs_dir.join("widget.md");
    let content = "---\ntopic: Widget Guide\ntags:\n  - type/spec\n  - project/wrong\n  - domain/ui\n---\n# Widget\n";
    std::fs::write(&file, content).unwrap();

    with_blueprints_dir(&tmp, || {
        let _ = cmd_retag(ArtifactKind::Doc, file.to_str().unwrap());

        let result = std::fs::read_to_string(&file).unwrap();
        assert!(
            result.contains("  - type/docs"),
            "type tag should be corrected to type/docs; got:\n{result}"
        );
        assert!(
            result.contains("  - project/myproj"),
            "project tag should be corrected to project/myproj; got:\n{result}"
        );
        assert!(
            result.contains("  - domain/ui"),
            "non-auto-derived tags should be preserved; got:\n{result}"
        );
        assert!(
            !result.contains("  - type/spec"),
            "old type tag should be removed; got:\n{result}"
        );
        assert!(
            !result.contains("  - project/wrong"),
            "old project tag should be removed; got:\n{result}"
        );
    });
    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn retag_noop_when_tags_correct() {
    let tmp = std::env::temp_dir().join(format!("ct-retag-noop-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();

    let docs_dir = tmp.join("myproj").join("docs");
    std::fs::create_dir_all(&docs_dir).unwrap();

    let file = docs_dir.join("correct.md");
    let content =
        "---\ntopic: Already Correct\ntags:\n  - type/docs\n  - project/myproj\n---\n# Correct\n";
    std::fs::write(&file, content).unwrap();

    with_blueprints_dir(&tmp, || {
        let result = cmd_retag(ArtifactKind::Doc, file.to_str().unwrap());
        assert!(result.is_ok(), "should return Ok when no changes needed");

        let after = std::fs::read_to_string(&file).unwrap();
        assert_eq!(after, content, "file should be unchanged");
    });
    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn read_returns_structured_frontmatter_and_body() {
    let tmp = std::env::temp_dir().join(format!("ct-read-core-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();
    let file = tmp.join("sample.md");
    std::fs::write(
        &file,
        "---\ntopic: My Topic\nauthor: me\ncreated: 2026-04-16\nsource: \"[[ref]]\"\ntags:\n  - type/spec\n  - domain/x\n---\nbody line 1\n<!--todo-->\n",
    )
    .unwrap();

    let outcome = read(&file).expect("read ok");
    assert_eq!(outcome.path, file);
    assert_eq!(outcome.body, "body line 1\n<!--todo-->\n");
    assert_eq!(outcome.comments.len(), 1, "one inline comment");
    let fm = outcome.frontmatter.expect("frontmatter present");
    assert_eq!(fm.topic.as_deref(), Some("My Topic"));
    assert_eq!(fm.author.as_deref(), Some("me"));
    assert_eq!(fm.created.as_deref(), Some("2026-04-16"));
    assert_eq!(fm.source.as_deref(), Some("ref"));
    assert_eq!(fm.tags, vec!["type/spec", "domain/x"]);

    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn create_returns_outcome_with_populated_fields() {
    let tmp = std::env::temp_dir().join(format!("ct-create-core-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();

    let project = tmp.join("myproj");
    std::fs::create_dir_all(&project).unwrap();

    with_blueprints_dir(&tmp, || {
        let outcome = create(CreateOpts {
            kind: ArtifactKind::Plan,
            topic: "New Plan",
            project: project.to_str().unwrap(),
            slug_override: Some("new-plan"),
            source: None,
            user_tags: &[],
            dive: false,
        });
        let plan_dir = tmp.join("myproj").join("plan");
        let files: Vec<_> = fs::read_dir(&plan_dir)
            .expect("plan/ exists")
            .flatten()
            .collect();
        assert_eq!(files.len(), 1, "exactly one plan file written");

        if let Ok(o) = outcome {
            assert_eq!(o.kind, ArtifactKind::Plan);
            assert_eq!(o.project, "myproj");
            assert!(o.pushed);
            assert!(o.path.exists());
        }
    });
    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn archive_core_moves_file_and_returns_destination() {
    let tmp = std::env::temp_dir().join(format!("ct-archive-core-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();

    let spec_dir = tmp.join("myproj").join("spec");
    std::fs::create_dir_all(&spec_dir).unwrap();
    let spec_file = spec_dir.join("20260411-target.md");
    std::fs::write(&spec_file, "---\ntopic: T\n---\n").unwrap();

    with_blueprints_dir(&tmp, || {
        let _ = archive(ArtifactKind::Spec, &spec_file);
        let expected = tmp
            .join("myproj")
            .join("archive")
            .join("spec")
            .join("20260411-target.md");
        assert!(expected.exists(), "file moved to archive/spec/");
        assert!(!spec_file.exists(), "source file removed");
    });
    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn resolve_artifact_path_returns_not_found_error() {
    let tmp = std::env::temp_dir().join(format!("ct-nf-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();

    with_blueprints_dir(&tmp, || {
        let err = resolve_artifact_path("nonexistent-stem", ArtifactKind::Plan)
            .expect_err("no match -> Err");
        assert!(matches!(err, ResolveError::NotFound(_)));
    });
    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn resolve_stem_universal_returns_ambiguous_when_same_kind_multiple_projects() {
    let tmp = std::env::temp_dir().join(format!("ct-univ-ambig-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();

    let _a = create_artifact_file(&tmp, "p1", ArtifactKind::Plan, "duplicated");
    let _b = create_artifact_file(&tmp, "p2", ArtifactKind::Plan, "duplicated");

    with_blueprints_dir(&tmp, || {
        let err = resolve_stem_universal("duplicated").expect_err("ambiguous");
        let ResolveError::Ambiguous(matches) = err else {
            panic!("expected Ambiguous");
        };
        assert_eq!(matches.len(), 2, "both duplicated matches returned");
    });
    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn ct_error_wraps_component_errors() {
    // Just exercise the From impls and Display — keeps the CtError variants
    // off the dead-code list and guards against their Display drifting.
    let sync: CtError = SyncError::Push("x".to_string()).into();
    assert!(sync.to_string().contains("push"));
    let resolve: CtError = ResolveError::NotFound("y".to_string()).into();
    assert!(resolve.to_string().contains("not found"));
    let io: CtError = std::io::Error::other("z").into();
    assert!(io.to_string().contains('z'));
    let val = CtError::Validation("bad".to_string());
    assert_eq!(val.to_string(), "bad");
}

// ── security regression tests ───────────────────────────────────────────

#[test]
fn resolve_stem_universal_rejects_path_traversal() {
    // Pre-fix this returned /etc/passwd because exists() was the only gate.
    let tmp = std::env::temp_dir().join(format!("ct-sec-traverse-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();
    with_blueprints_dir(&tmp, || {
        let err = resolve_stem_universal("../../../etc/passwd")
            .expect_err("path traversal must be rejected");
        assert!(
            matches!(err, ResolveError::NotFound(_)),
            "expected NotFound, got {err:?}"
        );
    });
    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn resolve_artifact_path_rejects_path_traversal() {
    let tmp = std::env::temp_dir().join(format!("ct-sec-artpath-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();
    with_blueprints_dir(&tmp, || {
        let err = resolve_artifact_path("../../../etc/passwd", ArtifactKind::Spec)
            .expect_err("path traversal must be rejected");
        assert!(
            matches!(err, ResolveError::NotFound(_)),
            "expected NotFound, got {err:?}"
        );
    });
    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn create_sanitizes_slug_override_with_path_separator() {
    let tmp = std::env::temp_dir().join(format!("ct-sec-slug-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();
    let project = tmp.join("myproj");
    std::fs::create_dir_all(&project).unwrap();
    with_blueprints_dir(&tmp, || {
        let outcome = create(CreateOpts {
            kind: ArtifactKind::Spec,
            topic: "Harmless Topic",
            project: project.to_str().unwrap(),
            slug_override: Some("../evil"),
            source: None,
            user_tags: &[],
            dive: false,
        });
        match outcome {
            Err(CtError::Validation(_)) => return,
            Ok(_) | Err(CtError::Sync(_)) => {}
            Err(e) => panic!("unexpected error variant: {e:?}"),
        }
        let spec_dir = tmp.join("myproj").join("spec");
        let files: Vec<PathBuf> = fs::read_dir(&spec_dir)
            .map(|d| d.flatten().map(|e| e.path()).collect())
            .unwrap_or_default();
        assert!(
            !files.is_empty(),
            "sanitized slug should have produced a file under {}",
            spec_dir.display()
        );
        for path in &files {
            assert!(
                path.starts_with(&spec_dir),
                "file escaped project/spec: {}",
                path.display()
            );
            let fname = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            assert!(!fname.contains('/'), "slug leaked / into {fname}");
            assert!(!fname.contains('\\'), "slug leaked \\ into {fname}");
            assert!(!fname.contains(".."), "slug leaked .. into {fname}");
        }
        let bad = tmp.join("etc");
        assert!(!bad.exists(), "traversal produced {}", bad.display());
    });
    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn create_rejects_whitespace_slug_override() {
    let tmp = std::env::temp_dir().join(format!("ct-sec-ws-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();
    let project = tmp.join("myproj");
    std::fs::create_dir_all(&project).unwrap();
    with_blueprints_dir(&tmp, || {
        let outcome = create(CreateOpts {
            kind: ArtifactKind::Spec,
            topic: "Topic",
            project: project.to_str().unwrap(),
            slug_override: Some("   "),
            source: None,
            user_tags: &[],
            dive: false,
        });
        assert!(
            matches!(outcome, Err(CtError::Validation(_))),
            "expected Validation for whitespace slug, got {outcome:?}"
        );
    });
    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn create_rejects_duplicate_same_hour() {
    let tmp = std::env::temp_dir().join(format!("ct-sec-dupe-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();
    let project = tmp.join("myproj");
    std::fs::create_dir_all(&project).unwrap();
    with_blueprints_dir(&tmp, || {
        let first = create(CreateOpts {
            kind: ArtifactKind::Plan,
            topic: "Some Plan",
            project: project.to_str().unwrap(),
            slug_override: Some("fixed-slug"),
            source: None,
            user_tags: &[],
            dive: false,
        });
        let path_exists = match &first {
            Ok(o) => o.path.exists(),
            Err(CtError::Sync(_)) => {
                let plan_dir = tmp.join("myproj").join("plan");
                fs::read_dir(&plan_dir)
                    .map(|d| d.flatten().next().is_some())
                    .unwrap_or(false)
            }
            Err(e) => panic!("unexpected first-create error: {e:?}"),
        };
        assert!(path_exists, "first create should have written the file");

        let second = create(CreateOpts {
            kind: ArtifactKind::Plan,
            topic: "Some Plan",
            project: project.to_str().unwrap(),
            slug_override: Some("fixed-slug"),
            source: None,
            user_tags: &[],
            dive: false,
        });
        assert!(
            matches!(second, Err(CtError::Validation(ref m)) if m.contains("already exists")),
            "second create must error with 'already exists', got {second:?}"
        );
    });
    std::fs::remove_dir_all(&tmp).ok();
}

// Unused-variable guard
fn _use(_: &Comment) {}
