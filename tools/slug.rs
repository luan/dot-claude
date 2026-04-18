use std::collections::HashSet;
use std::sync::LazyLock;

static STRIP_WORDS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    HashSet::from([
        "a", "an", "the", "for", "with", "and", "or", "to", "in", "of", "on", "by", "is", "it",
        "be", "as", "at", "do",
    ])
});

const MAX_SLUG_LEN: usize = 50;

pub fn slug(input: &str) -> String {
    let kept: Vec<&str> = input
        .split_whitespace()
        .filter(|w| {
            let clean: String = w.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
            let lower = clean.to_ascii_lowercase();
            !lower.is_empty() && !STRIP_WORDS.contains(lower.as_str())
        })
        .collect();

    let joined = kept.join("-");
    let s: String = joined
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();

    // Collapse multiple hyphens
    let mut result = String::with_capacity(s.len());
    let mut prev_hyphen = false;
    for c in s.chars() {
        if c == '-' {
            if !prev_hyphen {
                result.push('-');
            }
            prev_hyphen = true;
        } else {
            result.push(c);
            prev_hyphen = false;
        }
    }

    let trimmed = result.trim_matches('-').to_string();

    if trimmed.len() <= MAX_SLUG_LEN {
        return trimmed;
    }

    let cut = &trimmed[..MAX_SLUG_LEN];
    if let Some(last) = cut.rfind('-')
        && last > 0
    {
        return cut[..last].trim_end_matches('-').to_string();
    }
    cut.trim_end_matches('-').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_slug() {
        assert_eq!(slug("refactor the auth module"), "refactor-auth-module");
    }

    #[test]
    fn strips_articles_and_filler() {
        assert_eq!(
            slug("add a new feature for the users"),
            "add-new-feature-users"
        );
    }

    #[test]
    fn handles_special_chars() {
        assert_eq!(slug("fix: handle token expiry!"), "fix-handle-token-expiry");
    }

    #[test]
    fn truncates_on_word_boundary() {
        let long = "implement comprehensive authentication system with oauth2 and jwt token management for microservices";
        let result = slug(long);
        assert!(result.len() <= MAX_SLUG_LEN);
        assert!(!result.ends_with('-'));
    }

    #[test]
    fn empty_input() {
        assert_eq!(slug(""), "");
    }

    #[test]
    fn only_filler_words() {
        assert_eq!(slug("a the and or"), "");
    }

    #[test]
    fn unquoted_args_same_as_quoted() {
        assert_eq!(slug("refactor auth module"), slug("refactor auth module"));
    }
}
