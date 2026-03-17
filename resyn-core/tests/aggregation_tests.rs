//! Integration tests for aggregation logic in resyn_core::analysis::aggregation.
//!
//! These tests verify the behavior contract specified in Plan 08-04 Task 3:
//! - aggregate_open_problems: ranking by recurrence, edge cases
//! - build_method_matrix: category pair counting, alphabetical normalization

use resyn_core::analysis::aggregation::{aggregate_open_problems, build_method_matrix};
use resyn_core::datamodels::llm_annotation::{Finding, LlmAnnotation, Method};

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

fn make_annotation(arxiv_id: &str, open_problems: &[&str], methods: &[(&str, &str)]) -> LlmAnnotation {
    LlmAnnotation {
        arxiv_id: arxiv_id.to_string(),
        paper_type: "theoretical".to_string(),
        methods: methods
            .iter()
            .map(|(name, cat)| Method {
                name: name.to_string(),
                category: cat.to_string(),
            })
            .collect(),
        findings: vec![Finding {
            text: "some finding".to_string(),
            strength: "strong".to_string(),
        }],
        open_problems: open_problems.iter().map(|s| s.to_string()).collect(),
        provider: "noop".to_string(),
        model_name: "noop".to_string(),
        annotated_at: "2026-03-17T00:00:00Z".to_string(),
    }
}

// ---------------------------------------------------------------------------
// aggregate_open_problems tests
// ---------------------------------------------------------------------------

/// Empty annotation list returns empty vec.
#[test]
fn test_aggregate_open_problems_empty() {
    let result = aggregate_open_problems(&[]);
    assert!(result.is_empty(), "Expected empty result for empty input");
}

/// Ranking: "quantum gravity" in all 3 annotations, "dark matter" in 2.
/// Expected: [("quantum gravity", 3), ("dark matter", 2)]
#[test]
fn test_aggregate_open_problems_ranking() {
    let annotations = vec![
        make_annotation("a", &["dark matter", "quantum gravity"], &[]),
        make_annotation("b", &["quantum gravity", "dark matter"], &[]),
        make_annotation("c", &["quantum gravity"], &[]),
    ];
    let result = aggregate_open_problems(&annotations);

    assert_eq!(result.len(), 2, "Expected 2 distinct problems");
    assert_eq!(result[0].problem, "quantum gravity", "Most common problem first");
    assert_eq!(result[0].count, 3, "quantum gravity appears 3 times");
    assert_eq!(result[1].problem, "dark matter", "Second most common");
    assert_eq!(result[1].count, 2, "dark matter appears 2 times");
}

/// Single annotation with 3 problems: all returned with count 1.
#[test]
fn test_aggregate_open_problems_single_annotation() {
    let annotations = vec![make_annotation(
        "a",
        &["problem A", "problem B", "problem C"],
        &[],
    )];
    let result = aggregate_open_problems(&annotations);

    assert_eq!(result.len(), 3, "Expected 3 distinct problems");
    assert!(
        result.iter().all(|r| r.count == 1),
        "All counts should be 1 when each problem appears once"
    );
}

// ---------------------------------------------------------------------------
// build_method_matrix tests
// ---------------------------------------------------------------------------

/// Empty annotations returns empty categories and cells.
#[test]
fn test_build_method_matrix_empty() {
    let result = build_method_matrix(&[]);
    assert!(result.categories.is_empty(), "Expected empty categories");
    assert!(result.pair_counts.is_empty(), "Expected empty pair counts");
}

/// Given 2 annotations:
///   - annotation "a": methods in ["ML", "Stats"]
///   - annotation "b": methods in ["ML", "Physics"]
///
/// Expected pair counts:
///   ("ML",      "ML")      = 2  (both annotations have exactly one ML method → self-pair × 2)
///   ("ML",      "Physics") = 1  (only annotation b)
///   ("ML",      "Stats")   = 1  (only annotation a)
///   ("Physics", "Stats")   = 0  (no annotation has both)
///   ("Stats",   "Stats")   = 1  (annotation a has exactly one Stats method → self-pair × 1)
///   ("Physics", "Physics") = 1  (annotation b has exactly one Physics method → self-pair × 1)
#[test]
fn test_build_method_matrix_pair_counts() {
    let annotations = vec![
        make_annotation("a", &[], &[("m1", "ML"), ("m2", "Stats")]),
        make_annotation("b", &[], &[("m3", "ML"), ("m4", "Physics")]),
    ];
    let result = build_method_matrix(&annotations);

    let get = |a: &str, b: &str| -> u32 {
        let key = if a <= b {
            (a.to_string(), b.to_string())
        } else {
            (b.to_string(), a.to_string())
        };
        *result.pair_counts.get(&key).unwrap_or(&0)
    };

    assert_eq!(get("ML", "ML"), 2, "ML-ML: both annotations contribute a self-pair");
    assert_eq!(get("ML", "Stats"), 1, "ML-Stats: only annotation a");
    assert_eq!(get("ML", "Physics"), 1, "ML-Physics: only annotation b");
    assert_eq!(get("Physics", "Stats"), 0, "Physics-Stats: no annotation has both");
    assert_eq!(get("Stats", "Stats"), 1, "Stats-Stats: annotation a self-pair");
    assert_eq!(get("Physics", "Physics"), 1, "Physics-Physics: annotation b self-pair");
}

/// Category pairs are normalized alphabetically (row <= col).
/// Passing ("Stats", "ML") should give the same result as ("ML", "Stats").
#[test]
fn test_build_method_matrix_alphabetical_normalization() {
    let annotations = vec![
        make_annotation("a", &[], &[("m1", "ML"), ("m2", "Stats")]),
    ];
    let result = build_method_matrix(&annotations);

    // ML < Stats alphabetically, so the key must be ("ML", "Stats"), not ("Stats", "ML").
    assert!(
        result.pair_counts.contains_key(&("ML".to_string(), "Stats".to_string())),
        "Pair key should be alphabetically normalized (ML before Stats)"
    );
    assert!(
        !result.pair_counts.contains_key(&("Stats".to_string(), "ML".to_string())),
        "Reverse order key should not exist"
    );
}
