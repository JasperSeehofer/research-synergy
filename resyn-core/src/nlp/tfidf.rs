use std::collections::HashMap;

use sha2::{Digest, Sha256};

use crate::datamodels::extraction::{SectionMap, TextExtractionResult};

use super::preprocessing::{build_stop_words, tokenize};

/// Section weights for TF computation (locked decision from CONTEXT.md).
const ABSTRACT_WEIGHT: f32 = 2.0;
const METHODS_WEIGHT: f32 = 1.5;
const RESULTS_WEIGHT: f32 = 1.0;
const INTRO_WEIGHT: f32 = 0.5;
const CONCLUSION_WEIGHT: f32 = 0.5;

/// Compute weighted TF from a SectionMap.
/// For each section, accumulate weight * (count / n_tokens) per term,
/// then normalize by total weight of populated sections.
pub fn compute_weighted_tf(
    sections: &SectionMap,
    stop_words: &std::collections::HashSet<String>,
) -> HashMap<String, f32> {
    let section_data: Vec<(Option<&str>, f32)> = vec![
        (sections.abstract_text.as_deref(), ABSTRACT_WEIGHT),
        (sections.introduction.as_deref(), INTRO_WEIGHT),
        (sections.methods.as_deref(), METHODS_WEIGHT),
        (sections.results.as_deref(), RESULTS_WEIGHT),
        (sections.conclusion.as_deref(), CONCLUSION_WEIGHT),
    ];

    let mut accumulated: HashMap<String, f32> = HashMap::new();
    let mut total_weight: f32 = 0.0;

    for (text_opt, weight) in &section_data {
        let Some(text) = text_opt else { continue };
        let tokens: Vec<String> = tokenize(text)
            .into_iter()
            .filter(|t| !stop_words.contains(t))
            .collect();

        if tokens.is_empty() {
            continue;
        }

        total_weight += weight;
        let n_tokens = tokens.len() as f32;

        // Count raw token frequencies in this section
        let mut section_counts: HashMap<String, usize> = HashMap::new();
        for token in &tokens {
            *section_counts.entry(token.clone()).or_insert(0) += 1;
        }

        // Accumulate weighted TF contribution: weight * (count / n_tokens)
        for (term, count) in section_counts {
            *accumulated.entry(term).or_insert(0.0) += weight * (count as f32 / n_tokens);
        }
    }

    if total_weight == 0.0 {
        return HashMap::new();
    }

    // Normalize by total weight sum
    accumulated
        .into_iter()
        .map(|(term, score)| (term, score / total_weight))
        .collect()
}

/// Smooth IDF formula: ln((1+N)/(1+df)) + 1 (sklearn default).
pub fn compute_smooth_idf(doc_freq: usize, total_docs: usize) -> f32 {
    let n = total_docs as f32;
    let df = doc_freq as f32;
    ((1.0 + n) / (1.0 + df)).ln() + 1.0
}

/// Corpus fingerprint: sorted paper IDs joined and SHA-256 hashed.
/// Order-independent and deterministic.
pub fn corpus_fingerprint(arxiv_ids: &[String]) -> String {
    let mut sorted = arxiv_ids.to_vec();
    sorted.sort();
    let joined = sorted.join(",");
    let hash = Sha256::digest(joined.as_bytes());
    format!("{:x}", hash)
}

pub struct TfIdfEngine;

impl TfIdfEngine {
    pub fn new() -> Self {
        Self
    }

    /// Compute TF-IDF for all extractions. Returns (arxiv_id, tfidf_vector) pairs.
    /// IDF is computed corpus-wide ONCE per RESEARCH.md anti-pattern warning.
    pub fn compute_corpus(
        extractions: &[TextExtractionResult],
    ) -> Vec<(String, HashMap<String, f32>)> {
        let stop_words = build_stop_words();

        // Step 1: Compute weighted TF for each document
        let tf_docs: Vec<(String, HashMap<String, f32>)> = extractions
            .iter()
            .map(|ext| {
                let tf = compute_weighted_tf(&ext.sections, &stop_words);
                (ext.arxiv_id.clone(), tf)
            })
            .collect();

        // Step 2: Build document-frequency map across all documents
        let total_docs = tf_docs.len();
        let mut doc_freq: HashMap<String, usize> = HashMap::new();
        for (_, tf) in &tf_docs {
            for term in tf.keys() {
                *doc_freq.entry(term.clone()).or_insert(0) += 1;
            }
        }

        // Step 3: Compute IDF for each term (corpus-level, once)
        let idf: HashMap<String, f32> = doc_freq
            .iter()
            .map(|(term, &df)| (term.clone(), compute_smooth_idf(df, total_docs)))
            .collect();

        // Step 4: Apply TF * IDF per document
        tf_docs
            .into_iter()
            .map(|(arxiv_id, tf)| {
                let tfidf: HashMap<String, f32> = tf
                    .into_iter()
                    .map(|(term, tf_score)| {
                        let idf_score = idf.get(&term).copied().unwrap_or(1.0);
                        (term, tf_score * idf_score)
                    })
                    .collect();
                (arxiv_id, tfidf)
            })
            .collect()
    }

    /// Get top N terms by descending TF-IDF score.
    /// Returns parallel (top_terms, top_scores) arrays.
    pub fn get_top_n(tfidf: &HashMap<String, f32>, n: usize) -> (Vec<String>, Vec<f32>) {
        let mut pairs: Vec<(&String, &f32)> = tfidf.iter().collect();
        // Sort descending by score; break ties by term name for determinism
        pairs.sort_by(|a, b| {
            b.1.partial_cmp(a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.0.cmp(b.0))
        });
        pairs.truncate(n);

        let top_terms = pairs.iter().map(|(t, _)| (*t).clone()).collect();
        let top_scores = pairs.iter().map(|(_, s)| **s).collect();
        (top_terms, top_scores)
    }
}

impl Default for TfIdfEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::datamodels::extraction::SectionMap;

    use super::*;

    fn make_sections(
        abstract_text: Option<&str>,
        introduction: Option<&str>,
        methods: Option<&str>,
        results: Option<&str>,
        conclusion: Option<&str>,
    ) -> SectionMap {
        SectionMap {
            abstract_text: abstract_text.map(String::from),
            introduction: introduction.map(String::from),
            methods: methods.map(String::from),
            results: results.map(String::from),
            conclusion: conclusion.map(String::from),
        }
    }

    fn empty_stop_words() -> std::collections::HashSet<String> {
        std::collections::HashSet::new()
    }

    // Test 3: compute_weighted_tf with abstract="quantum entanglement" and methods="quantum lattice"
    // produces higher TF for "quantum" (appears in two weighted sections) than either "entanglement" or "lattice"
    #[test]
    fn test_weighted_tf_cross_section_accumulation() {
        let sections = make_sections(
            Some("quantum entanglement"),
            None,
            Some("quantum lattice"),
            None,
            None,
        );
        let tf = compute_weighted_tf(&sections, &empty_stop_words());

        let quantum = tf.get("quantum").copied().unwrap_or(0.0);
        let entanglement = tf.get("entanglement").copied().unwrap_or(0.0);
        let lattice = tf.get("lattice").copied().unwrap_or(0.0);

        assert!(
            quantum > entanglement,
            "quantum ({quantum}) should be > entanglement ({entanglement})"
        );
        assert!(
            quantum > lattice,
            "quantum ({quantum}) should be > lattice ({lattice})"
        );
    }

    // Test 4: abstract weight 2.0 > conclusion weight 0.5
    #[test]
    fn test_weighted_tf_abstract_outweighs_conclusion() {
        let sections = make_sections(Some("uniqueterm"), None, None, None, Some("otherterm"));
        let tf = compute_weighted_tf(&sections, &empty_stop_words());

        let abstract_score = tf.get("uniqueterm").copied().unwrap_or(0.0);
        let conclusion_score = tf.get("otherterm").copied().unwrap_or(0.0);

        assert!(
            abstract_score > conclusion_score,
            "Abstract term ({abstract_score}) should score higher than conclusion term ({conclusion_score})"
        );
    }

    // Test 5: compute_smooth_idf(1, 10) returns ln(11/2)+1
    #[test]
    fn test_smooth_idf_formula() {
        let idf = compute_smooth_idf(1, 10);
        let expected = ((11.0_f32) / (2.0_f32)).ln() + 1.0;
        assert!(
            (idf - expected).abs() < 1e-5,
            "IDF {idf} != expected {expected}"
        );
    }

    // Test 6: term in 1 doc has higher IDF than term in all 3 docs
    #[test]
    fn test_tfidf_corpus_relative_scores() {
        let sections1 = make_sections(
            Some("rare unique alpha beta gamma delta epsilon"),
            None,
            None,
            None,
            None,
        );
        let sections2 = make_sections(
            Some("common alpha beta gamma delta epsilon"),
            None,
            None,
            None,
            None,
        );
        let sections3 = make_sections(
            Some("common alpha beta gamma delta epsilon"),
            None,
            None,
            None,
            None,
        );

        let extractions = vec![
            crate::datamodels::extraction::TextExtractionResult {
                arxiv_id: "doc1".to_string(),
                sections: sections1,
                is_partial: false,
                extraction_method: crate::datamodels::extraction::ExtractionMethod::Ar5ivHtml,
                extracted_at: "2026-03-14".to_string(),
            },
            crate::datamodels::extraction::TextExtractionResult {
                arxiv_id: "doc2".to_string(),
                sections: sections2,
                is_partial: false,
                extraction_method: crate::datamodels::extraction::ExtractionMethod::Ar5ivHtml,
                extracted_at: "2026-03-14".to_string(),
            },
            crate::datamodels::extraction::TextExtractionResult {
                arxiv_id: "doc3".to_string(),
                sections: sections3,
                is_partial: false,
                extraction_method: crate::datamodels::extraction::ExtractionMethod::Ar5ivHtml,
                extracted_at: "2026-03-14".to_string(),
            },
        ];

        let results = TfIdfEngine::compute_corpus(&extractions);
        let doc1_tfidf = results
            .iter()
            .find(|(id, _)| id == "doc1")
            .map(|(_, v)| v)
            .unwrap();

        // "rare" appears only in doc1, "common" appears in doc2 and doc3
        // IDF("rare") > IDF("common"), so TF-IDF for "rare" should be higher than "common" in doc1
        let rare_score = doc1_tfidf.get("rare").copied().unwrap_or(0.0);
        let common_score = doc1_tfidf.get("common").copied().unwrap_or(0.0);

        assert!(
            rare_score > common_score,
            "rare ({rare_score}) should have higher TF-IDF than common ({common_score}) in doc1"
        );
    }

    // Test 7: get_top_n(5) returns exactly 5 terms sorted descending
    #[test]
    fn test_get_top_n_returns_n_sorted() {
        let mut tfidf = HashMap::new();
        tfidf.insert("alpha".to_string(), 0.9_f32);
        tfidf.insert("beta".to_string(), 0.7_f32);
        tfidf.insert("gamma".to_string(), 0.5_f32);
        tfidf.insert("delta".to_string(), 0.3_f32);
        tfidf.insert("epsilon".to_string(), 0.1_f32);
        tfidf.insert("zeta".to_string(), 0.05_f32);
        tfidf.insert("eta".to_string(), 0.02_f32);

        let (terms, scores) = TfIdfEngine::get_top_n(&tfidf, 5);
        assert_eq!(terms.len(), 5, "Should return exactly 5 terms");
        assert_eq!(scores.len(), 5, "Should return exactly 5 scores");

        // Check sorted descending
        for i in 0..scores.len() - 1 {
            assert!(
                scores[i] >= scores[i + 1],
                "Scores not sorted descending at position {i}"
            );
        }
        assert_eq!(terms[0], "alpha");
        assert_eq!(terms[1], "beta");
    }

    // Test 8: get_top_n on fewer than N terms returns all available
    #[test]
    fn test_get_top_n_fewer_than_n() {
        let mut tfidf = HashMap::new();
        tfidf.insert("alpha".to_string(), 0.9_f32);
        tfidf.insert("beta".to_string(), 0.5_f32);

        let (terms, scores) = TfIdfEngine::get_top_n(&tfidf, 5);
        assert_eq!(terms.len(), 2, "Should return all 2 available terms");
        assert_eq!(scores.len(), 2);
    }

    // Test 9: Abstract-only paper produces valid TF-IDF scores
    #[test]
    fn test_abstract_only_paper_produces_scores() {
        let sections = make_sections(
            Some("quantum computing entanglement decoherence lattice"),
            None,
            None,
            None,
            None,
        );
        let extraction = crate::datamodels::extraction::TextExtractionResult {
            arxiv_id: "abstract-only".to_string(),
            sections,
            is_partial: true,
            extraction_method: crate::datamodels::extraction::ExtractionMethod::AbstractOnly,
            extracted_at: "2026-03-14".to_string(),
        };

        let results = TfIdfEngine::compute_corpus(&[extraction]);
        let tfidf = &results[0].1;
        assert!(
            !tfidf.is_empty(),
            "Abstract-only paper should produce non-empty TF-IDF"
        );
        assert!(
            tfidf.values().any(|&v| v > 0.0),
            "Scores should be positive"
        );
    }

    // Test 10: corpus_fingerprint same for same IDs regardless of order
    #[test]
    fn test_corpus_fingerprint_order_independent() {
        let ids1 = vec![
            "2301.12345".to_string(),
            "2301.99999".to_string(),
            "2302.00001".to_string(),
        ];
        let ids2 = vec![
            "2302.00001".to_string(),
            "2301.12345".to_string(),
            "2301.99999".to_string(),
        ];

        assert_eq!(
            corpus_fingerprint(&ids1),
            corpus_fingerprint(&ids2),
            "Fingerprint should be order-independent"
        );
    }

    // Test 11: corpus_fingerprint changes when a paper ID is added
    #[test]
    fn test_corpus_fingerprint_changes_on_addition() {
        let ids1 = vec!["2301.12345".to_string(), "2301.99999".to_string()];
        let ids2 = vec![
            "2301.12345".to_string(),
            "2301.99999".to_string(),
            "2302.00001".to_string(),
        ];

        assert_ne!(
            corpus_fingerprint(&ids1),
            corpus_fingerprint(&ids2),
            "Fingerprint should change when paper ID is added"
        );
    }
}
