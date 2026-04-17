use similar::ChangeTag;
use similar::TextDiff;

pub fn unified_diff(path: &str, old: &str, new: &str) -> (String, usize, usize) {
    let diff = TextDiff::from_lines(old, new);
    let mut additions = 0usize;
    let mut deletions = 0usize;
    for change in diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Insert => additions += 1,
            ChangeTag::Delete => deletions += 1,
            ChangeTag::Equal => {}
        }
    }
    let unified = diff
        .unified_diff()
        .header(&format!("a/{path}"), &format!("b/{path}"))
        .to_string();
    (unified, additions, deletions)
}
