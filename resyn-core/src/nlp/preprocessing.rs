use std::collections::HashSet;

use stop_words::{LANGUAGE, get};

/// Tokenize text into lowercase tokens with length > 2, stripping punctuation.
pub fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|s| s.len() > 2)
        .map(|s| s.to_lowercase())
        .collect()
}

/// Build a stop-words set for English plus academic boilerplate terms.
pub fn build_stop_words() -> HashSet<String> {
    let mut words: HashSet<String> = get(LANGUAGE::English)
        .into_iter()
        .map(|s| s.to_lowercase())
        .collect();

    // Academic boilerplate per CONTEXT.md locked decision
    for word in &[
        "paper", "study", "result", "show", "figure", "also", "using",
    ] {
        words.insert(word.to_string());
    }

    words
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_basic() {
        let tokens = tokenize("Hello, World! This is a test.");
        // After lowercasing, punctuation stripping, len>2 filter
        // "Hello" -> "hello" (len 5, keep), "World" -> "world" (len 5, keep)
        // "This" -> "this" (len 4, keep), "is" -> "is" (len 2, NOT >2, drop)
        // "a" -> "a" (len 1, drop), "test" -> "test" (len 4, keep)
        assert!(tokens.contains(&"hello".to_string()), "Missing 'hello'");
        assert!(tokens.contains(&"world".to_string()), "Missing 'world'");
        assert!(tokens.contains(&"this".to_string()), "Missing 'this'");
        assert!(tokens.contains(&"test".to_string()), "Missing 'test'");
        // "is" and "a" should be filtered out (len <= 2)
        assert!(
            !tokens.contains(&"is".to_string()),
            "'is' should be filtered (len 2)"
        );
        assert!(!tokens.contains(&"a".to_string()), "'a' should be filtered");
    }

    #[test]
    fn test_build_stop_words_contains_common() {
        let stop = build_stop_words();
        assert!(stop.contains("the"), "Missing 'the'");
        assert!(stop.contains("and"), "Missing 'and'");
        assert!(
            stop.contains("paper"),
            "Missing academic boilerplate 'paper'"
        );
        assert!(
            stop.contains("study"),
            "Missing academic boilerplate 'study'"
        );
        assert!(
            stop.contains("result"),
            "Missing academic boilerplate 'result'"
        );
        assert!(stop.contains("show"), "Missing academic boilerplate 'show'");
        assert!(
            stop.contains("figure"),
            "Missing academic boilerplate 'figure'"
        );
    }
}
