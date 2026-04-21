use std::fs;

use anyhow::Result;
use sym::source_context::{RefLine, dedup_ref_lines, read_source_context};

#[test]
fn read_context_clamps_at_file_boundaries() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("sample.rs");
    fs::write(&path, "line1\nline2\nline3\nline4\n")?;

    // Near start: ctx=2 around line 1 must not underflow to line 0 or negative.
    let (lines, start) = read_source_context(&path, 1, 2);
    assert_eq!(start, 1);
    assert_eq!(lines, vec!["line1", "line2", "line3"]);

    // Near end: ctx=2 around line 4 must stop at last line.
    let (lines, start) = read_source_context(&path, 4, 2);
    assert_eq!(start, 2);
    assert_eq!(lines, vec!["line2", "line3", "line4"]);

    // ctx=0 returns just the target line.
    let (lines, start) = read_source_context(&path, 3, 0);
    assert_eq!(start, 3);
    assert_eq!(lines, vec!["line3"]);

    Ok(())
}

#[test]
fn dedup_groups_same_call_site_across_multiple_lines() {
    let refs = vec![
        RefLine {
            rel_path: "src/a.rs".into(),
            line: 10,
            text: "foo.bar()".into(),
            context_lines: vec!["foo.bar()".into()],
            context_start: 10,
        },
        RefLine {
            rel_path: "src/a.rs".into(),
            line: 20,
            text: "foo.bar()".into(),
            context_lines: vec!["foo.bar()".into()],
            context_start: 20,
        },
        RefLine {
            rel_path: "src/b.rs".into(),
            line: 5,
            text: "foo.bar()".into(),
            context_lines: vec!["foo.bar()".into()],
            context_start: 5,
        },
    ];
    let (lines, group_count) = dedup_ref_lines(&refs);
    assert_eq!(group_count, 2, "same (path, text) across 2 lines collapses to 1 group");
    assert!(
        lines.iter().any(|s| s.starts_with("src/a.rs (2 sites):")),
        "multi-line group must render as '(N sites):', got: {lines:?}"
    );
    assert!(lines.iter().any(|s| s.starts_with("src/b.rs:5:")));
}

#[test]
fn dedup_renders_context_with_caret_on_call_line() {
    let refs = vec![RefLine {
        rel_path: "src/a.rs".into(),
        line: 10,
        text: "foo.bar()".into(),
        context_lines: vec!["before".into(), "foo.bar()".into(), "after".into()],
        context_start: 9,
    }];
    let (lines, _) = dedup_ref_lines(&refs);
    assert_eq!(lines[0], "src/a.rs:10:");
    assert_eq!(lines[1], "    before");
    assert_eq!(lines[2], "  > foo.bar()");
    assert_eq!(lines[3], "    after");
}
