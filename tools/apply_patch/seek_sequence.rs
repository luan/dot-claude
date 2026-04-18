// Ported from openai/codex apply-patch/src/seek_sequence.rs
// https://github.com/openai/codex/tree/fe7c959e90d46abb8311e4a0b369e6cb32bf337e
// Licensed under Apache License 2.0. See NOTICE at workspace root.

/// Tier at which a pattern matched. Reported so callers can decide whether to
/// echo the applied diff (fuzzy) or stay silent (exact).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchQuality {
    Exact,
    TrimEnd,
    Trim,
    Normalized,
}

impl MatchQuality {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Exact => "exact",
            Self::TrimEnd => "trim_end",
            Self::Trim => "trim",
            Self::Normalized => "normalized",
        }
    }
}

/// Outcome of a sequence search. Ambiguous matches surface every candidate
/// so the caller can tell the user where their patch could have landed.
#[derive(Debug)]
pub(crate) enum SeekOutcome {
    NotFound,
    Unique { idx: usize, quality: MatchQuality },
    Ambiguous { matches: Vec<usize> },
}

/// Find `pattern` in `lines`, starting at or after `start`. Tiers are tried in
/// order (exact → trim trailing → trim both → normalise Unicode); the first
/// tier with any match wins, and that tier's match count decides Unique vs
/// Ambiguous. When `eof` is set, the search is pinned to the end-of-file
/// position — callers pass this for hunks carrying `*** End of File`.
///
/// Special cases:
///  * Empty `pattern` → `Unique { idx: start, quality: Exact }` (no-op).
///  * `pattern.len() > lines.len()` → `NotFound`.
pub(crate) fn seek_sequence(
    lines: &[String],
    pattern: &[String],
    start: usize,
    eof: bool,
) -> SeekOutcome {
    if pattern.is_empty() {
        return SeekOutcome::Unique {
            idx: start,
            quality: MatchQuality::Exact,
        };
    }
    if pattern.len() > lines.len() {
        return SeekOutcome::NotFound;
    }

    let end = lines.len() - pattern.len();
    let search_start = if eof { end } else { start };

    // Tier 1: exact match.
    let hits = find_hits(lines, pattern, search_start, end, |s| s);
    if !hits.is_empty() {
        return classify(hits, MatchQuality::Exact);
    }
    // Tier 2: trim trailing whitespace on both sides.
    let hits = find_hits(lines, pattern, search_start, end, str::trim_end);
    if !hits.is_empty() {
        return classify(hits, MatchQuality::TrimEnd);
    }
    // Tier 3: trim both ends.
    let hits = find_hits(lines, pattern, search_start, end, str::trim);
    if !hits.is_empty() {
        return classify(hits, MatchQuality::Trim);
    }
    // Tier 4: normalise Unicode punctuation (dashes, smart quotes, NBSP) so a
    // diff authored in ASCII can land on a file with typographic variants.
    let norm_lines: Vec<String> = lines.iter().map(|s| normalise(s)).collect();
    let norm_pat: Vec<String> = pattern.iter().map(|s| normalise(s)).collect();
    let hits: Vec<usize> = (search_start..=end)
        .filter(|&i| norm_lines[i..i + pattern.len()] == *norm_pat)
        .collect();
    if !hits.is_empty() {
        return classify(hits, MatchQuality::Normalized);
    }

    SeekOutcome::NotFound
}

fn classify(hits: Vec<usize>, quality: MatchQuality) -> SeekOutcome {
    if hits.len() == 1 {
        SeekOutcome::Unique {
            idx: hits[0],
            quality,
        }
    } else {
        let _ = quality;
        SeekOutcome::Ambiguous { matches: hits }
    }
}

fn find_hits<F: Fn(&str) -> &str>(
    lines: &[String],
    pattern: &[String],
    search_start: usize,
    end: usize,
    trim: F,
) -> Vec<usize> {
    let trimmed_lines: Vec<&str> = lines.iter().map(|s| trim(s)).collect();
    let trimmed_pat: Vec<&str> = pattern.iter().map(|s| trim(s)).collect();
    (search_start..=end)
        .filter(|&i| trimmed_lines[i..i + pattern.len()] == *trimmed_pat)
        .collect()
}

fn normalise(s: &str) -> String {
    s.trim()
        .chars()
        .map(|c| match c {
            '\u{2010}' | '\u{2011}' | '\u{2012}' | '\u{2013}' | '\u{2014}' | '\u{2015}'
            | '\u{2212}' => '-',
            '\u{2018}' | '\u{2019}' | '\u{201A}' | '\u{201B}' => '\'',
            '\u{201C}' | '\u{201D}' | '\u{201E}' | '\u{201F}' => '"',
            '\u{00A0}' | '\u{2002}' | '\u{2003}' | '\u{2004}' | '\u{2005}' | '\u{2006}'
            | '\u{2007}' | '\u{2008}' | '\u{2009}' | '\u{200A}' | '\u{202F}' | '\u{205F}'
            | '\u{3000}' => ' ',
            other => other,
        })
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn to_vec(strings: &[&str]) -> Vec<String> {
        strings.iter().map(|s| s.to_string()).collect()
    }

    fn unique_idx(outcome: SeekOutcome) -> Option<usize> {
        match outcome {
            SeekOutcome::Unique { idx, .. } => Some(idx),
            _ => None,
        }
    }

    #[test]
    fn exact_match_finds_sequence() {
        let lines = to_vec(&["foo", "bar", "baz"]);
        let pattern = to_vec(&["bar", "baz"]);
        assert_eq!(
            unique_idx(seek_sequence(&lines, &pattern, 0, false)),
            Some(1)
        );
    }

    #[test]
    fn rstrip_match_ignores_trailing_whitespace() {
        let lines = to_vec(&["foo   ", "bar\t\t"]);
        let pattern = to_vec(&["foo", "bar"]);
        let out = seek_sequence(&lines, &pattern, 0, false);
        match out {
            SeekOutcome::Unique { idx, quality } => {
                assert_eq!(idx, 0);
                assert_eq!(quality, MatchQuality::TrimEnd);
            }
            _ => panic!("expected unique trim-end match, got {out:?}"),
        }
    }

    #[test]
    fn trim_match_ignores_leading_and_trailing_whitespace() {
        let lines = to_vec(&["    foo   ", "   bar\t"]);
        let pattern = to_vec(&["foo", "bar"]);
        assert_eq!(
            unique_idx(seek_sequence(&lines, &pattern, 0, false)),
            Some(0)
        );
    }

    #[test]
    fn pattern_longer_than_input_returns_not_found() {
        let lines = to_vec(&["just one line"]);
        let pattern = to_vec(&["too", "many", "lines"]);
        assert!(matches!(
            seek_sequence(&lines, &pattern, 0, false),
            SeekOutcome::NotFound
        ));
    }

    #[test]
    fn ambiguous_exact_match_reports_all_candidates() {
        let lines = to_vec(&["foo", "bar", "foo", "bar"]);
        let pattern = to_vec(&["foo"]);
        match seek_sequence(&lines, &pattern, 0, false) {
            SeekOutcome::Ambiguous { matches } => {
                assert_eq!(matches, vec![0, 2]);
            }
            other => panic!("expected ambiguous, got {other:?}"),
        }
    }

    #[test]
    fn anchor_advances_search_past_first_match() {
        // When the caller advances `start` past a matched anchor, the earlier
        // occurrence is implicitly out of scope — second occurrence is unique.
        let lines = to_vec(&["foo", "bar", "foo", "baz"]);
        let pattern = to_vec(&["foo"]);
        assert_eq!(
            unique_idx(seek_sequence(&lines, &pattern, 1, false)),
            Some(2)
        );
    }

    #[test]
    fn eof_pins_search_to_end_of_file() {
        let lines = to_vec(&["foo", "bar", "foo", "baz"]);
        let pattern = to_vec(&["baz"]);
        assert_eq!(
            unique_idx(seek_sequence(&lines, &pattern, 0, true)),
            Some(3)
        );
    }
}
