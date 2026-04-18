// Ported from openai/codex apply-patch/src/seek_sequence.rs
// https://github.com/openai/codex/tree/fe7c959e90d46abb8311e4a0b369e6cb32bf337e
// Licensed under Apache License 2.0. See NOTICE at workspace root.

/// Attempt to find the sequence of `pattern` lines within `lines` beginning at or after `start`.
/// Returns the starting index of the match or `None` if not found. Matches are attempted with
/// decreasing strictness: exact match, then ignoring trailing whitespace, then ignoring leading
/// and trailing whitespace. When `eof` is true, we first try starting at the end-of-file (so that
/// patterns intended to match file endings are applied at the end), and fall back to searching
/// from `start` if needed.
///
/// Special cases handled defensively:
///  • Empty `pattern` → returns `Some(start)` (no-op match)
///  • `pattern.len() > lines.len()` → returns `None` (cannot match, avoids
///    out‑of‑bounds panic that occurred pre‑2025‑04‑12)
pub(crate) fn seek_sequence(
    lines: &[String],
    pattern: &[String],
    start: usize,
    eof: bool,
) -> Option<usize> {
    if pattern.is_empty() {
        return Some(start);
    }

    // When the pattern is longer than the available input there is no possible
    // match. Early‑return to avoid the out‑of‑bounds slice that would occur in
    // the search loops below (previously caused a panic when
    // `pattern.len() > lines.len()`).
    if pattern.len() > lines.len() {
        return None;
    }
    let search_start = if eof && lines.len() >= pattern.len() {
        lines.len() - pattern.len()
    } else {
        start
    };
    let end = lines.len().saturating_sub(pattern.len());

    // Pass 1: exact match.
    for i in search_start..=end {
        if lines[i..i + pattern.len()] == *pattern {
            return Some(i);
        }
    }

    // Passes 2 and 3 use pure-slice trims, so pre-compute a Vec<&str> view of
    // each side once instead of re-trimming on every window. trim/trim_end
    // return slices of the original string, no allocation.
    if let Some(i) = find_with_trim(lines, pattern, search_start, end, str::trim_end) {
        return Some(i);
    }
    if let Some(i) = find_with_trim(lines, pattern, search_start, end, str::trim) {
        return Some(i);
    }

    // ------------------------------------------------------------------
    // Final, most permissive pass – attempt to match after *normalising*
    // common Unicode punctuation to their ASCII equivalents so that diffs
    // authored with plain ASCII characters can still be applied to source
    // files that contain typographic dashes / quotes, etc.  This mirrors the
    // fuzzy behaviour of `git apply` which ignores minor byte-level
    // differences when locating context lines. Pre-normalise both sides once
    // up-front so the inner loop is a pointer-and-length compare.
    // ------------------------------------------------------------------
    let norm_lines: Vec<String> = lines.iter().map(|s| normalise(s)).collect();
    let norm_pat: Vec<String> = pattern.iter().map(|s| normalise(s)).collect();
    (search_start..=end).find(|&i| norm_lines[i..i + pattern.len()] == *norm_pat)
}

fn find_with_trim<F: Fn(&str) -> &str>(
    lines: &[String],
    pattern: &[String],
    search_start: usize,
    end: usize,
    trim: F,
) -> Option<usize> {
    let trimmed_lines: Vec<&str> = lines.iter().map(|s| trim(s)).collect();
    let trimmed_pat: Vec<&str> = pattern.iter().map(|s| trim(s)).collect();
    (search_start..=end).find(|&i| trimmed_lines[i..i + pattern.len()] == *trimmed_pat)
}

fn normalise(s: &str) -> String {
    s.trim()
        .chars()
        .map(|c| match c {
            // Various dash / hyphen code-points → ASCII '-'
            '\u{2010}' | '\u{2011}' | '\u{2012}' | '\u{2013}' | '\u{2014}' | '\u{2015}'
            | '\u{2212}' => '-',
            // Fancy single quotes → '\''
            '\u{2018}' | '\u{2019}' | '\u{201A}' | '\u{201B}' => '\'',
            // Fancy double quotes → '"'
            '\u{201C}' | '\u{201D}' | '\u{201E}' | '\u{201F}' => '"',
            // Non-breaking space and other odd spaces → normal space
            '\u{00A0}' | '\u{2002}' | '\u{2003}' | '\u{2004}' | '\u{2005}' | '\u{2006}'
            | '\u{2007}' | '\u{2008}' | '\u{2009}' | '\u{200A}' | '\u{202F}' | '\u{205F}'
            | '\u{3000}' => ' ',
            other => other,
        })
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::seek_sequence;
    use std::string::ToString;

    fn to_vec(strings: &[&str]) -> Vec<String> {
        strings.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn test_exact_match_finds_sequence() {
        let lines = to_vec(&["foo", "bar", "baz"]);
        let pattern = to_vec(&["bar", "baz"]);
        assert_eq!(
            seek_sequence(&lines, &pattern, /*start*/ 0, /*eof*/ false),
            Some(1)
        );
    }

    #[test]
    fn test_rstrip_match_ignores_trailing_whitespace() {
        let lines = to_vec(&["foo   ", "bar\t\t"]);
        // Pattern omits trailing whitespace.
        let pattern = to_vec(&["foo", "bar"]);
        assert_eq!(
            seek_sequence(&lines, &pattern, /*start*/ 0, /*eof*/ false),
            Some(0)
        );
    }

    #[test]
    fn test_trim_match_ignores_leading_and_trailing_whitespace() {
        let lines = to_vec(&["    foo   ", "   bar\t"]);
        // Pattern omits any additional whitespace.
        let pattern = to_vec(&["foo", "bar"]);
        assert_eq!(
            seek_sequence(&lines, &pattern, /*start*/ 0, /*eof*/ false),
            Some(0)
        );
    }

    #[test]
    fn test_pattern_longer_than_input_returns_none() {
        let lines = to_vec(&["just one line"]);
        let pattern = to_vec(&["too", "many", "lines"]);
        // Should not panic – must return None when pattern cannot possibly fit.
        assert_eq!(
            seek_sequence(&lines, &pattern, /*start*/ 0, /*eof*/ false),
            None
        );
    }
}
