use std::collections::HashMap;

use crate::datamodels::analysis::PaperAnalysis;
use crate::datamodels::similarity::{PaperSimilarity, SimilarNeighbor};

/// Computes cosine similarity between two sparse TF-IDF vectors.
///
/// Returns a value in [0.0, 1.0]. Returns 0.0 if either vector is empty or
/// has zero magnitude.
pub fn cosine_similarity(a: &HashMap<String, f32>, b: &HashMap<String, f32>) -> f32 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }

    // Compute dot product by iterating the shorter vector's keys
    let (shorter, longer) = if a.len() <= b.len() { (a, b) } else { (b, a) };

    let dot_product: f32 = shorter
        .iter()
        .filter_map(|(key, val)| longer.get(key).map(|other| val * other))
        .sum();

    let mag_a: f32 = a.values().map(|v| v * v).sum::<f32>().sqrt();
    let mag_b: f32 = b.values().map(|v| v * v).sum::<f32>().sqrt();

    if mag_a == 0.0 || mag_b == 0.0 {
        return 0.0;
    }

    dot_product / (mag_a * mag_b)
}

/// Computes top-K similar neighbors for each paper in the corpus using cosine similarity.
///
/// Returns a `PaperSimilarity` for each paper, with neighbors sorted by descending score.
/// Self-similarity is excluded. When fewer than `top_k` other papers exist, all are included.
/// Shared high-weight terms (min_weight=0.05, top 3) are stored per neighbor.
pub fn compute_top_neighbors(analyses: &[PaperAnalysis], top_k: usize) -> Vec<PaperSimilarity> {
    analyses
        .iter()
        .map(|target| {
            let mut scored: Vec<SimilarNeighbor> = analyses
                .iter()
                .filter(|other| other.arxiv_id != target.arxiv_id)
                .map(|other| {
                    let score =
                        cosine_similarity(&target.tfidf_vector, &other.tfidf_vector);
                    let shared =
                        shared_high_weight_terms(&target.tfidf_vector, &other.tfidf_vector, 0.05);
                    let top_shared = shared.into_iter().take(3).collect();
                    SimilarNeighbor {
                        arxiv_id: other.arxiv_id.clone(),
                        score,
                        shared_terms: top_shared,
                    }
                })
                .collect();

            scored.sort_by(|a, b| {
                b.score
                    .partial_cmp(&a.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            scored.truncate(top_k);

            PaperSimilarity {
                arxiv_id: target.arxiv_id.clone(),
                neighbors: scored,
                corpus_fingerprint: target.corpus_fingerprint.clone(),
                computed_at: chrono::Utc::now().to_rfc3339(),
            }
        })
        .collect()
}

/// Returns terms present in both vectors with weight >= `min_weight` in both.
/// Output is sorted alphabetically for deterministic results.
pub fn shared_high_weight_terms(
    a: &HashMap<String, f32>,
    b: &HashMap<String, f32>,
    min_weight: f32,
) -> Vec<String> {
    let mut terms: Vec<String> = a
        .iter()
        .filter(|(key, val)| **val >= min_weight && b.get(*key).is_some_and(|bv| *bv >= min_weight))
        .map(|(key, _)| key.clone())
        .collect();

    terms.sort();
    terms
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vec_from(pairs: &[(&str, f32)]) -> HashMap<String, f32> {
        pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    }

    #[test]
    fn test_cosine_similarity_identical() {
        let v = vec_from(&[("quantum", 0.8), ("entanglement", 0.6)]);
        let sim = cosine_similarity(&v, &v);
        assert!(
            (sim - 1.0).abs() < 1e-5,
            "identical vectors should return 1.0, got {sim}"
        );
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec_from(&[("quantum", 0.8)]);
        let b = vec_from(&[("topology", 0.5)]);
        let sim = cosine_similarity(&a, &b);
        assert!(
            (sim - 0.0).abs() < 1e-5,
            "orthogonal vectors should return 0.0, got {sim}"
        );
    }

    #[test]
    fn test_cosine_similarity_partial_overlap() {
        // a = (1, 0, 1), b = (1, 1, 0) — normalised dot product = 1/(sqrt(2)*sqrt(2)) = 0.5
        let a = vec_from(&[("x", 1.0), ("z", 1.0)]);
        let b = vec_from(&[("x", 1.0), ("y", 1.0)]);
        let sim = cosine_similarity(&a, &b);
        let expected = 0.5_f32;
        assert!(
            (sim - expected).abs() < 1e-5,
            "partial overlap: expected {expected}, got {sim}"
        );
    }

    #[test]
    fn test_cosine_similarity_empty_vector() {
        let empty: HashMap<String, f32> = HashMap::new();
        let v = vec_from(&[("quantum", 0.8)]);
        assert_eq!(cosine_similarity(&empty, &v), 0.0);
        assert_eq!(cosine_similarity(&v, &empty), 0.0);
        assert_eq!(cosine_similarity(&empty, &empty), 0.0);
    }

    #[test]
    fn test_shared_high_weight_terms_filters_correctly() {
        let a = vec_from(&[("quantum", 0.8), ("noise", 0.05), ("entanglement", 0.6)]);
        let b = vec_from(&[("quantum", 0.7), ("noise", 0.3), ("topology", 0.5)]);
        // "quantum" is in both with weight >= 0.1; "noise" is in both but <0.1 in a; "entanglement" only in a
        let shared = shared_high_weight_terms(&a, &b, 0.1);
        assert_eq!(shared, vec!["quantum".to_string()]);
    }

    #[test]
    fn test_shared_high_weight_terms_empty_when_no_match() {
        let a = vec_from(&[("quantum", 0.8)]);
        let b = vec_from(&[("topology", 0.5)]);
        let shared = shared_high_weight_terms(&a, &b, 0.1);
        assert!(shared.is_empty());
    }

    #[test]
    fn test_shared_high_weight_terms_sorted_alphabetically() {
        let a = vec_from(&[("z_term", 0.5), ("a_term", 0.5), ("m_term", 0.5)]);
        let b = vec_from(&[("z_term", 0.5), ("a_term", 0.5), ("m_term", 0.5)]);
        let shared = shared_high_weight_terms(&a, &b, 0.1);
        assert_eq!(shared, vec!["a_term", "m_term", "z_term"]);
    }

    // --- compute_top_neighbors tests ---

    fn make_analysis(arxiv_id: &str, terms: &[(&str, f32)], fingerprint: &str) -> PaperAnalysis {
        PaperAnalysis {
            arxiv_id: arxiv_id.to_string(),
            tfidf_vector: vec_from(terms),
            top_terms: terms.iter().map(|(k, _)| k.to_string()).collect(),
            top_scores: terms.iter().map(|(_, v)| *v).collect(),
            analyzed_at: "2026-04-08T00:00:00Z".to_string(),
            corpus_fingerprint: fingerprint.to_string(),
        }
    }

    #[test]
    fn test_compute_top_neighbors_excludes_self() {
        let analyses = vec![
            make_analysis("2301.00001", &[("quantum", 0.8), ("spin", 0.6)], "fp1"),
            make_analysis("2301.00002", &[("quantum", 0.7), ("spin", 0.5)], "fp1"),
            make_analysis("2301.00003", &[("topology", 0.9)], "fp1"),
        ];
        let results = compute_top_neighbors(&analyses, 10);
        assert_eq!(results.len(), 3);
        for result in &results {
            assert!(
                result.neighbors.iter().all(|n| n.arxiv_id != result.arxiv_id),
                "Self-similarity must not appear in neighbors for {}",
                result.arxiv_id
            );
        }
    }

    #[test]
    fn test_compute_top_neighbors_sorted_descending() {
        let analyses = vec![
            make_analysis("2301.00001", &[("quantum", 0.8), ("spin", 0.6)], "fp1"),
            make_analysis("2301.00002", &[("quantum", 0.7), ("spin", 0.5)], "fp1"),
            make_analysis("2301.00003", &[("topology", 0.9)], "fp1"),
            make_analysis("2301.00004", &[("spin", 0.6), ("noise", 0.4)], "fp1"),
        ];
        let results = compute_top_neighbors(&analyses, 10);
        for result in &results {
            let scores: Vec<f32> = result.neighbors.iter().map(|n| n.score).collect();
            for window in scores.windows(2) {
                assert!(
                    window[0] >= window[1],
                    "Neighbors must be sorted descending: {} < {} for {}",
                    window[0],
                    window[1],
                    result.arxiv_id
                );
            }
        }
    }

    #[test]
    fn test_compute_top_neighbors_fewer_than_k_returns_all_non_self() {
        // Only 3 papers, top_k=10 — should return 2 neighbors (all non-self)
        let analyses = vec![
            make_analysis("2301.00001", &[("quantum", 0.8)], "fp1"),
            make_analysis("2301.00002", &[("quantum", 0.7)], "fp1"),
            make_analysis("2301.00003", &[("topology", 0.9)], "fp1"),
        ];
        let results = compute_top_neighbors(&analyses, 10);
        for result in &results {
            assert_eq!(
                result.neighbors.len(),
                2,
                "With 3 papers and top_k=10, expect 2 neighbors for {}",
                result.arxiv_id
            );
        }
    }

    #[test]
    fn test_compute_top_neighbors_truncates_to_k() {
        // 5 papers, top_k=3 — should return exactly 3 neighbors
        let analyses = vec![
            make_analysis("2301.00001", &[("quantum", 0.8)], "fp1"),
            make_analysis("2301.00002", &[("quantum", 0.7)], "fp1"),
            make_analysis("2301.00003", &[("quantum", 0.6)], "fp1"),
            make_analysis("2301.00004", &[("quantum", 0.5)], "fp1"),
            make_analysis("2301.00005", &[("quantum", 0.4)], "fp1"),
        ];
        let results = compute_top_neighbors(&analyses, 3);
        for result in &results {
            assert!(
                result.neighbors.len() <= 3,
                "top_k=3 must truncate to 3, got {} for {}",
                result.neighbors.len(),
                result.arxiv_id
            );
        }
    }

    #[test]
    fn test_compute_top_neighbors_shared_terms_populated() {
        // Papers with overlapping high-weight terms
        let analyses = vec![
            make_analysis(
                "2301.00001",
                &[("quantum", 0.8), ("entanglement", 0.7), ("spin", 0.6)],
                "fp1",
            ),
            make_analysis(
                "2301.00002",
                &[("quantum", 0.9), ("entanglement", 0.8), ("spin", 0.5)],
                "fp1",
            ),
            make_analysis("2301.00003", &[("topology", 0.9)], "fp1"),
        ];
        let results = compute_top_neighbors(&analyses, 10);
        // Paper 1 and 2 should share terms with each other
        let paper1 = results.iter().find(|r| r.arxiv_id == "2301.00001").unwrap();
        let neighbor2 = paper1
            .neighbors
            .iter()
            .find(|n| n.arxiv_id == "2301.00002")
            .unwrap();
        // Both have quantum, entanglement, spin >= 0.05 in both
        assert!(
            !neighbor2.shared_terms.is_empty(),
            "Shared terms should be populated for similar papers"
        );
        // At most 3 shared terms
        assert!(
            neighbor2.shared_terms.len() <= 3,
            "shared_terms must be capped at 3"
        );
    }

    #[test]
    fn test_compute_top_neighbors_corpus_fingerprint_propagated() {
        let analyses = vec![
            make_analysis("2301.00001", &[("quantum", 0.8)], "fingerprint_abc"),
            make_analysis("2301.00002", &[("quantum", 0.7)], "fingerprint_abc"),
        ];
        let results = compute_top_neighbors(&analyses, 10);
        for result in &results {
            assert_eq!(
                result.corpus_fingerprint, "fingerprint_abc",
                "corpus_fingerprint must be propagated from analysis"
            );
        }
    }
}
