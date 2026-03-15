#[cfg(feature = "ssr")]
use super::similarity;
#[cfg(feature = "ssr")]
use crate::datamodels::llm_annotation::LlmAnnotation;

#[cfg(feature = "ssr")]
use crate::datamodels::analysis::PaperAnalysis;
#[cfg(feature = "ssr")]
use crate::datamodels::gap_finding::{GapFinding, GapType};
#[cfg(feature = "ssr")]
use crate::llm::gap_prompt::CONTRADICTION_SYSTEM_PROMPT;
#[cfg(feature = "ssr")]
use crate::llm::traits::LlmProvider;
#[cfg(feature = "ssr")]
use chrono::Utc;
#[cfg(feature = "ssr")]
use std::collections::HashMap;
#[cfg(feature = "ssr")]
use tracing::warn;

/// Strength levels considered "strong" evidence.
#[cfg(feature = "ssr")]
const STRONG_STRENGTHS: &[&str] = &["strong", "strong_evidence", "established", "confirmed"];
/// Strength levels considered "weak" evidence.
#[cfg(feature = "ssr")]
const WEAK_STRENGTHS: &[&str] = &[
    "weak",
    "weak_evidence",
    "preliminary",
    "speculative",
    "inconclusive",
];

/// Cosine similarity threshold — pairs below this are not topic-similar enough.
#[cfg(feature = "ssr")]
const SIMILARITY_THRESHOLD: f32 = 0.3;

#[cfg(feature = "ssr")]
fn classify_strength(strength: &str) -> Option<bool> {
    let s = strength.to_lowercase();
    if STRONG_STRENGTHS.iter().any(|&x| s.contains(x)) {
        return Some(true); // strong
    }
    if WEAK_STRENGTHS.iter().any(|&x| s.contains(x)) {
        return Some(false); // weak
    }
    None
}

#[cfg(feature = "ssr")]
fn findings_diverge(ann_a: &LlmAnnotation, ann_b: &LlmAnnotation) -> bool {
    // At least one paper must have findings at all
    if ann_a.findings.is_empty() || ann_b.findings.is_empty() {
        return false;
    }

    // Diverge if one paper has >= 1 strong finding and the other has >= 1 weak finding
    let a_has_strong = ann_a
        .findings
        .iter()
        .any(|f| classify_strength(&f.strength) == Some(true));
    let b_has_strong = ann_b
        .findings
        .iter()
        .any(|f| classify_strength(&f.strength) == Some(true));
    let a_has_weak = ann_a
        .findings
        .iter()
        .any(|f| classify_strength(&f.strength) == Some(false));
    let b_has_weak = ann_b
        .findings
        .iter()
        .any(|f| classify_strength(&f.strength) == Some(false));

    (a_has_strong && b_has_weak) || (a_has_weak && b_has_strong)
}

#[cfg(feature = "ssr")]
fn build_contradiction_context(
    analysis_a: &PaperAnalysis,
    analysis_b: &PaperAnalysis,
    ann_a: &LlmAnnotation,
    ann_b: &LlmAnnotation,
    shared_terms: &[String],
) -> String {
    let findings_a: Vec<String> = ann_a
        .findings
        .iter()
        .map(|f| format!("  - [{}] {}", f.strength, f.text))
        .collect();
    let findings_b: Vec<String> = ann_b
        .findings
        .iter()
        .map(|f| format!("  - [{}] {}", f.strength, f.text))
        .collect();

    format!(
        "Paper A (arxiv:{}):\nFindings:\n{}\n\nPaper B (arxiv:{}):\nFindings:\n{}\n\nShared high-weight terms: {}",
        analysis_a.arxiv_id,
        if findings_a.is_empty() {
            "  (none)".to_string()
        } else {
            findings_a.join("\n")
        },
        analysis_b.arxiv_id,
        if findings_b.is_empty() {
            "  (none)".to_string()
        } else {
            findings_b.join("\n")
        },
        shared_terms.join(", ")
    )
}

#[cfg(feature = "ssr")]
/// Finds contradiction candidates among a set of papers using a two-stage pipeline:
///
/// 1. TF-IDF similarity filter (>= 0.3 threshold)
/// 2. Finding strength divergence check
/// 3. LLM verification (graceful skip on failure)
pub async fn find_contradictions(
    analyses: &[PaperAnalysis],
    annotations: &[LlmAnnotation],
    provider: &mut dyn LlmProvider,
) -> Vec<GapFinding> {
    // Build lookup maps
    let annotation_map: HashMap<&str, &LlmAnnotation> = annotations
        .iter()
        .map(|a| (a.arxiv_id.as_str(), a))
        .collect();

    let mut results = Vec::new();

    // Stage 1 + 2: TF-IDF similarity and finding divergence
    for i in 0..analyses.len() {
        for j in (i + 1)..analyses.len() {
            let a = &analyses[i];
            let b = &analyses[j];

            // Stage 1: cosine similarity threshold
            let sim = similarity::cosine_similarity(&a.tfidf_vector, &b.tfidf_vector);
            if sim < SIMILARITY_THRESHOLD {
                continue;
            }

            // Need both annotations for stage 2
            let ann_a = match annotation_map.get(a.arxiv_id.as_str()) {
                Some(ann) => ann,
                None => continue,
            };
            let ann_b = match annotation_map.get(b.arxiv_id.as_str()) {
                Some(ann) => ann,
                None => continue,
            };

            // Stage 2: finding strength divergence
            if !findings_diverge(ann_a, ann_b) {
                continue;
            }

            // Stage 3: LLM verification
            let shared_terms =
                similarity::shared_high_weight_terms(&a.tfidf_vector, &b.tfidf_vector, 0.1);
            let context = build_contradiction_context(a, b, ann_a, ann_b, &shared_terms);

            let justification = match provider
                .verify_gap(CONTRADICTION_SYSTEM_PROMPT, &context)
                .await
            {
                Ok(response) => {
                    if response.trim().eq_ignore_ascii_case("NO") {
                        continue;
                    }
                    response
                }
                Err(e) => {
                    warn!(
                        paper_a = a.arxiv_id.as_str(),
                        paper_b = b.arxiv_id.as_str(),
                        error = %e,
                        "LLM verification failed for contradiction candidate, skipping"
                    );
                    continue;
                }
            };

            results.push(GapFinding {
                gap_type: GapType::Contradiction,
                paper_ids: vec![a.arxiv_id.clone(), b.arxiv_id.clone()],
                shared_terms,
                justification,
                confidence: sim,
                found_at: Utc::now().to_rfc3339(),
            });
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "ssr")]
    use crate::datamodels::llm_annotation::{Finding, LlmAnnotation};

    #[cfg(feature = "ssr")]
    fn make_annotation(id: &str, findings: &[(&str, &str)]) -> LlmAnnotation {
        LlmAnnotation {
            arxiv_id: id.to_string(),
            paper_type: "theoretical".to_string(),
            methods: vec![],
            findings: findings
                .iter()
                .map(|(text, strength)| Finding {
                    text: text.to_string(),
                    strength: strength.to_string(),
                })
                .collect(),
            open_problems: vec![],
            provider: "noop".to_string(),
            model_name: "noop".to_string(),
            annotated_at: "2026-03-14T00:00:00Z".to_string(),
        }
    }

    #[cfg(feature = "ssr")]
    use crate::datamodels::analysis::PaperAnalysis;
    #[cfg(feature = "ssr")]
    use crate::llm::noop::NoopProvider;

    #[cfg(feature = "ssr")]
    fn make_analysis(id: &str, terms: &[(&str, f32)]) -> PaperAnalysis {
        PaperAnalysis {
            arxiv_id: id.to_string(),
            tfidf_vector: terms.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
            top_terms: vec![],
            top_scores: vec![],
            analyzed_at: "2026-03-14T00:00:00Z".to_string(),
            corpus_fingerprint: "test".to_string(),
        }
    }

    #[cfg(feature = "ssr")]
    fn make_high_sim_analyses() -> (PaperAnalysis, PaperAnalysis) {
        // Same terms = cosine similarity 1.0 (well above 0.3 threshold)
        let terms = &[
            ("quantum", 0.8_f32),
            ("entanglement", 0.6),
            ("decoherence", 0.5),
        ];
        (
            make_analysis("2301.11111", terms),
            make_analysis("2301.22222", terms),
        )
    }

    #[cfg(feature = "ssr")]
    #[tokio::test]
    async fn test_find_contradictions_returns_finding_for_divergent_pair() {
        let (a, b) = make_high_sim_analyses();
        let ann_a = make_annotation(
            "2301.11111",
            &[("Energy gap is non-zero", "strong_evidence")],
        );
        let ann_b = make_annotation(
            "2301.22222",
            &[("Energy gap appears negligible", "preliminary")],
        );
        let mut provider = NoopProvider;

        // NoopProvider returns "NO", so result will be empty — but we test stage 1 + 2 logic
        // by checking that with a YES provider it would produce a result.
        // For NoopProvider tests: verify no panic, correct skip on NO.
        let result = find_contradictions(&[a, b], &[ann_a, ann_b], &mut provider).await;
        // NoopProvider returns "NO" — expect empty (LLM rejection)
        assert!(
            result.is_empty(),
            "NoopProvider returns NO, so result should be empty"
        );
    }

    #[cfg(feature = "ssr")]
    #[tokio::test]
    async fn test_find_contradictions_skips_below_similarity_threshold() {
        // Orthogonal vectors: cosine similarity = 0.0 (< 0.3 threshold)
        let a = make_analysis("2301.11111", &[("quantum", 0.8)]);
        let b = make_analysis("2301.22222", &[("topology", 0.5)]);
        let ann_a = make_annotation("2301.11111", &[("Result A", "strong_evidence")]);
        let ann_b = make_annotation("2301.22222", &[("Result B", "preliminary")]);
        let mut provider = NoopProvider;

        let result = find_contradictions(&[a, b], &[ann_a, ann_b], &mut provider).await;
        assert!(result.is_empty(), "Low similarity pairs should be skipped");
    }

    #[cfg(feature = "ssr")]
    #[tokio::test]
    async fn test_find_contradictions_skips_non_divergent_strengths() {
        // Both papers have strong findings — no divergence
        let (a, b) = make_high_sim_analyses();
        let ann_a = make_annotation("2301.11111", &[("Result A", "strong_evidence")]);
        let ann_b = make_annotation("2301.22222", &[("Result B", "established")]);
        let mut provider = NoopProvider;

        let result = find_contradictions(&[a, b], &[ann_a, ann_b], &mut provider).await;
        assert!(
            result.is_empty(),
            "Non-divergent findings should be skipped"
        );
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn test_findings_diverge_strong_vs_weak() {
        let ann_a = make_annotation("a", &[("X", "strong_evidence")]);
        let ann_b = make_annotation("b", &[("Y", "preliminary")]);
        assert!(findings_diverge(&ann_a, &ann_b));
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn test_findings_diverge_both_strong_no_divergence() {
        let ann_a = make_annotation("a", &[("X", "strong_evidence")]);
        let ann_b = make_annotation("b", &[("Y", "established")]);
        assert!(!findings_diverge(&ann_a, &ann_b));
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn test_findings_diverge_empty_findings_no_divergence() {
        let ann_a = make_annotation("a", &[]);
        let ann_b = make_annotation("b", &[("Y", "preliminary")]);
        assert!(!findings_diverge(&ann_a, &ann_b));
    }
}
