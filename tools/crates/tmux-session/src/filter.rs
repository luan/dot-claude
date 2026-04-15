use nucleo_matcher::pattern::{Atom, AtomKind, CaseMatching, Normalization};
use nucleo_matcher::{Config, Matcher, Utf32Str};

/// Return `(original_index, score)` pairs sorted by score descending for all
/// items whose haystack (produced by `haystack_fn`) fuzzy-matches `query`.
pub(crate) fn fuzzy_match<T>(
    items: &[T],
    query: &str,
    haystack_fn: impl Fn(&T) -> String,
) -> Vec<(usize, u16)> {
    if query.is_empty() {
        return Vec::new();
    }

    let mut matcher = Matcher::new(Config::DEFAULT);
    let atom = Atom::new(
        query,
        CaseMatching::Ignore,
        Normalization::Smart,
        AtomKind::Fuzzy,
        false,
    );
    let needle = atom.needle_text();
    let mut buf = Vec::new();
    let mut matches = Vec::new();

    for (idx, item) in items.iter().enumerate() {
        let hay = haystack_fn(item);
        let haystack = Utf32Str::new(&hay, &mut buf);
        let mut indices = Vec::new();
        if let Some(score) = matcher.fuzzy_indices(haystack, needle, &mut indices) {
            matches.push((idx, score));
        }
        buf.clear();
    }

    matches.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    matches
}
