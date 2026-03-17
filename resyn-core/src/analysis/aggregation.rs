//! Pure aggregation functions for LLM annotation data.
//!
//! These functions operate only on WASM-safe types (no DB, no ssr gate) so they
//! can be unit-tested in resyn-core without any server-side feature flags.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::datamodels::llm_annotation::LlmAnnotation;

/// A ranked open problem: the problem text and how many annotations mention it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RankedProblem {
    pub problem: String,
    pub count: usize,
}

/// A method matrix capturing how often pairs of method categories co-occur within
/// the same paper annotation. Category pairs are stored in alphabetical order
/// (pair.0 <= pair.1) to avoid double-counting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodMatrix {
    /// All distinct category labels (sorted alphabetically).
    pub categories: Vec<String>,
    /// Co-occurrence counts keyed by (row_category, col_category) where row <= col.
    pub pair_counts: HashMap<(String, String), u32>,
}

/// Aggregate open problems across all annotations.
///
/// Counts how many annotations mention each unique open-problem string, then
/// returns the list sorted by count descending (most-mentioned first).
///
/// Pure function — no DB, no async, fully testable.
pub fn aggregate_open_problems(annotations: &[LlmAnnotation]) -> Vec<RankedProblem> {
    let mut counts: HashMap<String, usize> = HashMap::new();

    for ann in annotations {
        for problem in &ann.open_problems {
            *counts.entry(problem.clone()).or_default() += 1;
        }
    }

    let mut ranked: Vec<RankedProblem> = counts
        .into_iter()
        .map(|(problem, count)| RankedProblem { problem, count })
        .collect();

    // Sort descending by count, then alphabetically by problem text for stability.
    ranked.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.problem.cmp(&b.problem)));
    ranked
}

/// Build a method-category co-occurrence matrix from annotations.
///
/// For each annotation, collects all method categories then counts every
/// (category_a, category_b) pair where a <= b (alphabetical normalization).
/// Self-pairs (a == b) count once per annotation that has at least one method
/// in that category.
///
/// Pure function — no DB, no async, fully testable.
pub fn build_method_matrix(annotations: &[LlmAnnotation]) -> MethodMatrix {
    let mut pair_counts: HashMap<(String, String), u32> = HashMap::new();
    let mut all_categories: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    for ann in annotations {
        let cats: Vec<String> = ann.methods.iter().map(|m| m.category.clone()).collect();

        for cat in &cats {
            all_categories.insert(cat.clone());
        }

        // Count every pair (i, j) with i <= j (alphabetical normalization).
        for i in 0..cats.len() {
            for j in i..cats.len() {
                let (a, b) = if cats[i] <= cats[j] {
                    (cats[i].clone(), cats[j].clone())
                } else {
                    (cats[j].clone(), cats[i].clone())
                };
                *pair_counts.entry((a, b)).or_default() += 1;
            }
        }
    }

    let categories: Vec<String> = all_categories.into_iter().collect();

    MethodMatrix {
        categories,
        pair_counts,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datamodels::llm_annotation::{Finding, LlmAnnotation, Method};

    fn make_annotation(arxiv_id: &str, open_problems: Vec<&str>, methods: Vec<(&str, &str)>) -> LlmAnnotation {
        LlmAnnotation {
            arxiv_id: arxiv_id.to_string(),
            paper_type: "theoretical".to_string(),
            methods: methods.into_iter().map(|(name, cat)| Method { name: name.to_string(), category: cat.to_string() }).collect(),
            findings: vec![Finding { text: "some finding".to_string(), strength: "strong".to_string() }],
            open_problems: open_problems.into_iter().map(String::from).collect(),
            provider: "noop".to_string(),
            model_name: "noop".to_string(),
            annotated_at: "2026-03-17T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_aggregate_empty() {
        let result = aggregate_open_problems(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_aggregate_ranking() {
        let annotations = vec![
            make_annotation("a", vec!["dark matter", "quantum gravity"], vec![]),
            make_annotation("b", vec!["quantum gravity", "dark matter"], vec![]),
            make_annotation("c", vec!["quantum gravity"], vec![]),
        ];
        let result = aggregate_open_problems(&annotations);
        assert_eq!(result[0].problem, "quantum gravity");
        assert_eq!(result[0].count, 3);
        assert_eq!(result[1].problem, "dark matter");
        assert_eq!(result[1].count, 2);
    }

    #[test]
    fn test_build_method_matrix_empty() {
        let result = build_method_matrix(&[]);
        assert!(result.categories.is_empty());
        assert!(result.pair_counts.is_empty());
    }

    #[test]
    fn test_build_method_matrix_pair_counts() {
        let annotations = vec![
            make_annotation("a", vec![], vec![("m1", "ML"), ("m2", "Stats")]),
            make_annotation("b", vec![], vec![("m3", "ML"), ("m4", "Physics")]),
        ];
        let result = build_method_matrix(&annotations);
        // ML-ML: 2 (each annotation has one ML method, so i==j self-pair counts once per annotation)
        assert_eq!(*result.pair_counts.get(&("ML".to_string(), "ML".to_string())).unwrap_or(&0), 2);
        // ML-Stats: 1 (only annotation a)
        assert_eq!(*result.pair_counts.get(&("ML".to_string(), "Stats".to_string())).unwrap_or(&0), 1);
        // ML-Physics: 1 (only annotation b)
        assert_eq!(*result.pair_counts.get(&("ML".to_string(), "Physics".to_string())).unwrap_or(&0), 1);
        // Physics-Stats: 0 (no annotation has both)
        assert_eq!(*result.pair_counts.get(&("Physics".to_string(), "Stats".to_string())).unwrap_or(&0), 0);
        // Stats-Stats: 1 (annotation a has one Stats method, self-pair)
        assert_eq!(*result.pair_counts.get(&("Stats".to_string(), "Stats".to_string())).unwrap_or(&0), 1);
    }
}
