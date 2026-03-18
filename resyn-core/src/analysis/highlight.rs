/// Find the byte range [start, end) of `snippet` within `section_text`.
///
/// Strategy:
/// 1. Normalize both inputs (collapse whitespace, lowercase).
/// 2. Attempt exact substring match on normalized text.
/// 3. If found, map byte offsets back to original text.
/// 4. Fallback: word-overlap sliding window with min 0.5 overlap ratio.
///
/// Returns `None` if no acceptable match is found.
pub fn find_highlight_range(section_text: &str, snippet: &str) -> Option<(usize, usize)> {
    if snippet.is_empty() || section_text.is_empty() {
        return None;
    }

    let norm_section = normalize_whitespace(section_text);
    let norm_snippet = normalize_whitespace(snippet);

    // --- Strategy 1: exact match on normalized text ---
    if let Some(norm_start) = norm_section.find(&norm_snippet) {
        let norm_end = norm_start + norm_snippet.len();
        // Map back to original offsets via character-position alignment.
        if let Some((orig_start, orig_end)) =
            map_normalized_range_to_original(section_text, &norm_section, norm_start, norm_end)
        {
            return Some((orig_start, orig_end));
        }
    }

    // --- Strategy 2: word-overlap sliding window ---
    let section_words: Vec<&str> = norm_section.split_whitespace().collect();
    let snippet_words: Vec<&str> = norm_snippet.split_whitespace().collect();
    let window = snippet_words.len();

    if window == 0 || section_words.len() < window {
        return None;
    }

    let mut best_overlap: usize = 0;
    let mut best_window_start: usize = 0;

    for i in 0..=(section_words.len() - window) {
        let overlap = section_words[i..i + window]
            .iter()
            .filter(|w| snippet_words.contains(w))
            .count();
        if overlap > best_overlap {
            best_overlap = overlap;
            best_window_start = i;
        }
    }

    let overlap_ratio = best_overlap as f64 / window as f64;
    if overlap_ratio < 0.5 {
        return None;
    }

    // Reconstruct original byte range for the best window.
    let window_words = &section_words[best_window_start..best_window_start + window];
    let window_text = window_words.join(" ");
    // Find this reconstructed window in the normalized section.
    if let Some(norm_start) = norm_section.find(&window_text) {
        let norm_end = norm_start + window_text.len();
        if let Some((orig_start, orig_end)) =
            map_normalized_range_to_original(section_text, &norm_section, norm_start, norm_end)
        {
            return Some((orig_start, orig_end));
        }
    }

    None
}

/// Map byte range `[norm_start, norm_end)` in `norm_text` back to a byte range in `original`.
///
/// The normalization (collapse whitespace, lowercase) may change byte offsets, so we walk
/// through both strings in tandem to find the matching region.
fn map_normalized_range_to_original(
    original: &str,
    norm_text: &str,
    norm_start: usize,
    norm_end: usize,
) -> Option<(usize, usize)> {
    // We need to find the substring of `original` that corresponds to
    // norm_text[norm_start..norm_end]. Walk both with a char-level cursor.
    let norm_prefix = &norm_text[..norm_start];
    let norm_match = &norm_text[norm_start..norm_end];

    // Count whitespace-collapsed "words" consumed by norm_prefix.
    let prefix_words: Vec<&str> = norm_prefix.split_whitespace().collect();

    // Rebuild word positions from original.
    let orig_words: Vec<(usize, usize, &str)> = {
        let mut v = Vec::new();
        let mut in_word = false;
        let mut word_start = 0;
        for (i, c) in original.char_indices() {
            if !c.is_whitespace() {
                if !in_word {
                    word_start = i;
                    in_word = true;
                }
            } else if in_word {
                v.push((word_start, i, &original[word_start..i]));
                in_word = false;
            }
        }
        if in_word {
            v.push((word_start, original.len(), &original[word_start..]));
        }
        v
    };

    // The normalized string (lowercase + collapsed whitespace) interleaves words identically
    // with the original words. So the i-th word in norm corresponds to the i-th word in orig.
    let prefix_word_count = prefix_words.len();
    let match_words: Vec<&str> = norm_match.split_whitespace().collect();
    let match_word_count = match_words.len();

    if prefix_word_count + match_word_count > orig_words.len() {
        return None;
    }

    // Handle the edge case: if norm_prefix ends with a partial word (won't happen with our
    // normalization since split_whitespace always produces complete words).

    // Find start and end in original.
    let start_word_idx = prefix_word_count;
    let end_word_idx = prefix_word_count + match_word_count;

    if end_word_idx == 0 || end_word_idx > orig_words.len() {
        return None;
    }

    let orig_start = orig_words[start_word_idx].0;
    let orig_end = orig_words[end_word_idx - 1].1;

    Some((orig_start, orig_end))
}

/// Collapse all whitespace runs to a single space, trim, and lowercase.
fn normalize_whitespace(s: &str) -> String {
    s.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_highlight_range_exact_match() {
        let section = "The energy gap is non-zero and significant.";
        let snippet = "energy gap is non-zero";
        let result = find_highlight_range(section, snippet);
        assert!(result.is_some(), "Expected Some for exact substring match");
        let (start, end) = result.unwrap();
        assert_eq!(&section[start..end], "energy gap is non-zero");
    }

    #[test]
    fn test_find_highlight_range_whitespace_differences() {
        let section = "We  used   Monte   Carlo  simulation.";
        let snippet = "used Monte Carlo simulation";
        let result = find_highlight_range(section, snippet);
        assert!(
            result.is_some(),
            "Expected Some for match with extra whitespace"
        );
        let (start, end) = result.unwrap();
        let matched = &section[start..end];
        assert!(
            matched.contains("Monte"),
            "Matched text should contain 'Monte'"
        );
    }

    #[test]
    fn test_find_highlight_range_unrelated_returns_none() {
        let section = "The paper discusses quantum mechanics.";
        let snippet = "classical thermodynamics pressure volume";
        let result = find_highlight_range(section, snippet);
        assert!(result.is_none(), "Expected None for completely unrelated snippet");
    }
}
