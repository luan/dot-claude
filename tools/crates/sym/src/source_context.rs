use std::fs;
use std::path::Path;

/// A single reference to render, with optional surrounding source context.
pub struct RefLine {
    pub rel_path: String,
    pub line: usize,
    pub text: String,
    pub context_lines: Vec<String>,
    pub context_start: usize,
}

/// Read a single line from a file, right-trimmed. Returns empty string on
/// any IO or out-of-range error — consumers format "" as a blank line.
pub fn read_source_line(path: &Path, line_num: usize) -> String {
    let Ok(contents) = fs::read_to_string(path) else {
        return String::new();
    };
    contents
        .lines()
        .nth(line_num.saturating_sub(1))
        .map(|line| line.trim_end_matches([' ', '\t']).to_string())
        .unwrap_or_default()
}

/// Read lines `[line_num - ctx, line_num + ctx]` (clamped to file bounds),
/// right-trimmed. Returns `(lines, first_line_1_based)`. `ctx == 0` returns
/// just the target line.
pub fn read_source_context(path: &Path, line_num: usize, ctx: usize) -> (Vec<String>, usize) {
    if ctx == 0 {
        return (vec![read_source_line(path, line_num)], line_num);
    }
    let Ok(contents) = fs::read_to_string(path) else {
        return (vec![String::new()], line_num);
    };

    let start = line_num.saturating_sub(ctx).max(1);
    let end = line_num.saturating_add(ctx);

    let lines = contents
        .lines()
        .enumerate()
        .filter_map(|(idx, text)| {
            let n = idx + 1;
            (n >= start && n <= end).then(|| text.trim_end_matches([' ', '\t']).to_string())
        })
        .collect();
    (lines, start)
}

/// Group refs by (path, text) and format for human output. Identical call
/// sites across multiple lines collapse into a single "path (N sites)"
/// header with one set of context lines. Returns (formatted_lines, group_count).
pub fn dedup_ref_lines(refs: &[RefLine]) -> (Vec<String>, usize) {
    struct Group {
        path: String,
        text: String,
        lines: Vec<usize>,
        context_lines: Vec<String>,
        context_start: usize,
        call_line: usize,
    }

    let mut groups: Vec<Group> = Vec::new();
    let mut index: std::collections::BTreeMap<(String, String), usize> = Default::default();

    for ref_line in refs {
        let key = (ref_line.rel_path.clone(), ref_line.text.clone());
        if let Some(&i) = index.get(&key) {
            groups[i].lines.push(ref_line.line);
        } else {
            index.insert(key, groups.len());
            groups.push(Group {
                path: ref_line.rel_path.clone(),
                text: ref_line.text.clone(),
                lines: vec![ref_line.line],
                context_lines: ref_line.context_lines.clone(),
                context_start: ref_line.context_start,
                call_line: ref_line.line,
            });
        }
    }

    let mut out = Vec::new();
    for group in &groups {
        let has_context = group.context_lines.len() > 1;
        let header = if group.lines.len() == 1 {
            format!("{}:{}:", group.path, group.lines[0])
        } else {
            format!("{} ({} sites):", group.path, group.lines.len())
        };

        if !has_context {
            out.push(format!("{header} {}", group.text));
            continue;
        }

        out.push(header);
        for (i, line_text) in group.context_lines.iter().enumerate() {
            let line_no = group.context_start + i;
            if line_no == group.call_line {
                out.push(format!("  > {line_text}"));
            } else {
                out.push(format!("    {line_text}"));
            }
        }
    }
    (out, groups.len())
}
