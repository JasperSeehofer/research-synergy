use crate::datamodels::gap_finding::{GapFinding, GapType};

const TRUNCATION_LEN: usize = 60;

/// Format a table of gap findings grouped by type (Contradictions then ABC Bridges).
/// When `verbose` is false, justification is truncated to 60 chars.
/// When `verbose` is true, full justification is shown.
pub fn format_gap_table(findings: &[GapFinding], verbose: bool) -> String {
    let contradictions: Vec<&GapFinding> = findings
        .iter()
        .filter(|f| f.gap_type == GapType::Contradiction)
        .collect();
    let abc_bridges: Vec<&GapFinding> = findings
        .iter()
        .filter(|f| f.gap_type == GapType::AbcBridge)
        .collect();

    let mut output = String::new();

    // --- Contradictions section ---
    output.push_str("Contradictions\n");
    output.push_str(&"-".repeat(80));
    output.push('\n');

    if contradictions.is_empty() {
        output.push_str("(none found)\n");
    } else {
        output.push_str(&format_section(&contradictions, verbose));
    }

    output.push('\n');

    // --- ABC Bridges section ---
    output.push_str("ABC Bridges\n");
    output.push_str(&"-".repeat(80));
    output.push('\n');

    if abc_bridges.is_empty() {
        output.push_str("(none found)\n");
    } else {
        output.push_str(&format_section(&abc_bridges, verbose));
    }

    output
}

fn format_section(findings: &[&GapFinding], verbose: bool) -> String {
    // Compute column widths
    let papers_header = "Papers";
    let terms_header = "Shared Terms";
    let just_header = "Justification";

    let papers_width = findings
        .iter()
        .map(|f| f.paper_ids.join(", ").len())
        .max()
        .unwrap_or(0)
        .max(papers_header.len());

    let terms_width = findings
        .iter()
        .map(|f| f.shared_terms.join(", ").len())
        .max()
        .unwrap_or(0)
        .max(terms_header.len());

    let just_width = if verbose {
        findings
            .iter()
            .map(|f| f.justification.len())
            .max()
            .unwrap_or(0)
            .max(just_header.len())
    } else {
        TRUNCATION_LEN.max(just_header.len())
    };

    let mut output = String::new();

    // Header row
    output.push_str(&format!(
        "{:<papers_width$}  {:<terms_width$}  {:<just_width$}\n",
        papers_header,
        terms_header,
        just_header,
        papers_width = papers_width,
        terms_width = terms_width,
        just_width = just_width,
    ));

    // Separator
    output.push_str(&format!(
        "{}  {}  {}\n",
        "-".repeat(papers_width),
        "-".repeat(terms_width),
        "-".repeat(just_width),
    ));

    // Data rows
    for finding in findings {
        let papers_col = finding.paper_ids.join(", ");
        let terms_col = finding.shared_terms.join(", ");
        let just_col = if verbose {
            finding.justification.clone()
        } else {
            truncate_justification(&finding.justification, TRUNCATION_LEN)
        };

        output.push_str(&format!(
            "{:<papers_width$}  {:<terms_width$}  {}\n",
            papers_col,
            terms_col,
            just_col,
            papers_width = papers_width,
            terms_width = terms_width,
        ));
    }

    output
}

fn truncate_justification(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        // Truncate at max_len - 3 to make room for "..."
        let truncated = &text[..max_len.saturating_sub(3)];
        format!("{truncated}...")
    }
}

/// Summarize gap findings with counts of each type and unique paper count.
/// Returns: "Gap analysis: N contradictions, M ABC-bridges found across P papers"
pub fn format_gap_summary(findings: &[GapFinding]) -> String {
    let n_contradictions = findings
        .iter()
        .filter(|f| f.gap_type == GapType::Contradiction)
        .count();
    let m_abc_bridges = findings
        .iter()
        .filter(|f| f.gap_type == GapType::AbcBridge)
        .count();

    let mut unique_papers: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for finding in findings {
        for pid in &finding.paper_ids {
            unique_papers.insert(pid.as_str());
        }
    }
    let p_papers = unique_papers.len();

    format!(
        "Gap analysis: {n_contradictions} contradictions, {m_abc_bridges} ABC-bridges found across {p_papers} papers"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datamodels::gap_finding::{GapFinding, GapType};

    fn make_contradiction(
        paper_ids: Vec<&str>,
        shared_terms: Vec<&str>,
        justification: &str,
    ) -> GapFinding {
        GapFinding {
            gap_type: GapType::Contradiction,
            paper_ids: paper_ids.into_iter().map(String::from).collect(),
            shared_terms: shared_terms.into_iter().map(String::from).collect(),
            justification: justification.to_string(),
            confidence: 0.8,
            found_at: "2026-03-14T10:00:00Z".to_string(),
        }
    }

    fn make_abc_bridge(
        paper_ids: Vec<&str>,
        shared_terms: Vec<&str>,
        justification: &str,
    ) -> GapFinding {
        GapFinding {
            gap_type: GapType::AbcBridge,
            paper_ids: paper_ids.into_iter().map(String::from).collect(),
            shared_terms: shared_terms.into_iter().map(String::from).collect(),
            justification: justification.to_string(),
            confidence: 0.6,
            found_at: "2026-03-14T10:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_format_gap_table_empty_has_none_found_sections() {
        let result = format_gap_table(&[], false);
        assert!(
            result.contains("Contradictions"),
            "Should have Contradictions section"
        );
        assert!(
            result.contains("ABC Bridges"),
            "Should have ABC Bridges section"
        );
        assert_eq!(
            result.matches("(none found)").count(),
            2,
            "Both sections should show (none found)"
        );
    }

    #[test]
    fn test_format_gap_summary_empty_has_zero_counts() {
        let result = format_gap_summary(&[]);
        assert_eq!(
            result,
            "Gap analysis: 0 contradictions, 0 ABC-bridges found across 0 papers"
        );
    }

    #[test]
    fn test_format_gap_table_one_contradiction_has_headers() {
        let finding = make_contradiction(
            vec!["2301.11111", "2301.22222"],
            vec!["quantum", "entanglement"],
            "Paper A claims strong correlation while Paper B finds none.",
        );
        let result = format_gap_table(&[finding], false);

        // Section header present
        assert!(
            result.contains("Contradictions"),
            "Should have Contradictions section"
        );
        // Table headers present
        assert!(
            result.contains("Papers"),
            "Should have Papers column header"
        );
        assert!(
            result.contains("Shared Terms"),
            "Should have Shared Terms column header"
        );
        assert!(
            result.contains("Justification"),
            "Should have Justification column header"
        );
        // Paper IDs in result
        assert!(
            result.contains("2301.11111"),
            "Should contain first paper ID"
        );
        assert!(
            result.contains("2301.22222"),
            "Should contain second paper ID"
        );
        // Shared terms in result
        assert!(result.contains("quantum"), "Should contain shared term");
        // ABC section shows none
        assert!(
            result.contains("(none found)"),
            "ABC Bridges section should show (none found)"
        );
    }

    #[test]
    fn test_format_gap_table_one_abc_bridge_has_headers() {
        let finding = make_abc_bridge(
            vec!["2301.11111", "2301.33333"],
            vec!["topological", "lattice", "symmetry"],
            "Paper A's methods connect to Paper C via topological concepts.",
        );
        let result = format_gap_table(&[finding], false);

        assert!(
            result.contains("ABC Bridges"),
            "Should have ABC Bridges section"
        );
        assert!(
            result.contains("Papers"),
            "Should have Papers column header"
        );
        assert!(
            result.contains("Shared Terms"),
            "Should have Shared Terms column header"
        );
        assert!(result.contains("topological"), "Should contain shared term");
        assert!(
            result.contains("(none found)"),
            "Contradictions section should show (none found)"
        );
    }

    #[test]
    fn test_format_gap_table_truncates_justification_at_60_chars_non_verbose() {
        // A justification longer than 60 chars should be truncated
        let long_just = "This is a very long justification that exceeds sixty characters in length for testing.";
        assert!(
            long_just.len() > 60,
            "Test prerequisite: justification must be > 60 chars"
        );

        let finding =
            make_contradiction(vec!["2301.11111", "2301.22222"], vec!["quantum"], long_just);
        let result = format_gap_table(&[finding], false);

        // Should contain "..." as truncation marker
        assert!(
            result.contains("..."),
            "Truncated justification should end with ..."
        );
        // Should NOT contain the full text
        assert!(
            !result.contains(long_just),
            "Full justification should not appear in non-verbose output"
        );
    }

    #[test]
    fn test_format_gap_table_shows_full_justification_verbose() {
        let long_just = "This is a very long justification that exceeds sixty characters in length for testing.";
        let finding =
            make_contradiction(vec!["2301.11111", "2301.22222"], vec!["quantum"], long_just);
        let result = format_gap_table(&[finding], true);

        // Full justification should appear
        assert!(
            result.contains(long_just),
            "Full justification should appear in verbose mode"
        );
    }

    #[test]
    fn test_format_gap_summary_correct_counts() {
        let findings = vec![
            make_contradiction(vec!["A", "B"], vec!["x"], "just1"),
            make_contradiction(vec!["B", "C"], vec!["y"], "just2"),
            make_abc_bridge(vec!["A", "C"], vec!["z"], "just3"),
        ];
        let result = format_gap_summary(&findings);
        assert_eq!(
            result,
            "Gap analysis: 2 contradictions, 1 ABC-bridges found across 3 papers"
        );
    }

    #[test]
    fn test_format_gap_summary_counts_unique_papers() {
        // Papers A and B appear in multiple findings — should count as 2 unique papers
        let findings = vec![
            make_contradiction(vec!["A", "B"], vec!["x"], "just1"),
            make_contradiction(vec!["A", "B"], vec!["y"], "just2"),
        ];
        let result = format_gap_summary(&findings);
        assert_eq!(
            result,
            "Gap analysis: 2 contradictions, 0 ABC-bridges found across 2 papers"
        );
    }

    #[test]
    fn test_truncate_justification_short_text_unchanged() {
        let short = "Short text.";
        assert!(short.len() <= 60);
        let result = truncate_justification(short, 60);
        assert_eq!(result, short);
    }

    #[test]
    fn test_truncate_justification_exactly_60_unchanged() {
        let exactly_60 = "a".repeat(60);
        let result = truncate_justification(&exactly_60, 60);
        assert_eq!(result, exactly_60);
    }

    #[test]
    fn test_truncate_justification_61_chars_truncated() {
        let text = "a".repeat(61);
        let result = truncate_justification(&text, 60);
        assert!(result.ends_with("..."), "Should end with ...");
        assert!(result.len() <= 60, "Should be at most 60 chars");
    }
}
