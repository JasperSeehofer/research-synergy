use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PaperAnalysis {
    pub arxiv_id: String,
    pub tfidf_vector: HashMap<String, f32>, // sparse term->score
    pub top_terms: Vec<String>,             // parallel arrays per RESEARCH.md pitfall 5
    pub top_scores: Vec<f32>,               // parallel arrays
    pub analyzed_at: String,
    pub corpus_fingerprint: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnalysisMetadata {
    pub key: String, // e.g. "corpus_tfidf"
    pub paper_count: u64,
    pub corpus_fingerprint: String,
    pub last_analyzed: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_paper_analysis() -> PaperAnalysis {
        let mut tfidf = HashMap::new();
        tfidf.insert("quantum".to_string(), 0.85_f32);
        tfidf.insert("entanglement".to_string(), 0.72_f32);
        tfidf.insert("decoherence".to_string(), 0.61_f32);

        PaperAnalysis {
            arxiv_id: "2301.12345".to_string(),
            tfidf_vector: tfidf,
            top_terms: vec![
                "quantum".to_string(),
                "entanglement".to_string(),
                "decoherence".to_string(),
            ],
            top_scores: vec![0.85_f32, 0.72_f32, 0.61_f32],
            analyzed_at: "2026-03-14T00:00:00Z".to_string(),
            corpus_fingerprint: "abc123def456".to_string(),
        }
    }

    #[test]
    fn test_paper_analysis_roundtrip_serde() {
        let analysis = make_paper_analysis();
        let json = serde_json::to_string(&analysis).unwrap();
        let recovered: PaperAnalysis = serde_json::from_str(&json).unwrap();

        assert_eq!(recovered.arxiv_id, analysis.arxiv_id);
        assert_eq!(recovered.analyzed_at, analysis.analyzed_at);
        assert_eq!(recovered.corpus_fingerprint, analysis.corpus_fingerprint);
        assert_eq!(recovered.top_terms, analysis.top_terms);
        assert_eq!(recovered.top_scores.len(), analysis.top_scores.len());
        for (a, b) in recovered.top_scores.iter().zip(analysis.top_scores.iter()) {
            assert!((a - b).abs() < 1e-6, "scores differ: {a} vs {b}");
        }
        assert_eq!(recovered.tfidf_vector.len(), analysis.tfidf_vector.len());
        for (key, val) in &analysis.tfidf_vector {
            let rv = recovered.tfidf_vector.get(key).expect("key missing");
            assert!((rv - val).abs() < 1e-6, "value differs for {key}");
        }
    }

    #[test]
    fn test_analysis_metadata_roundtrip_serde() {
        let meta = AnalysisMetadata {
            key: "corpus_tfidf".to_string(),
            paper_count: 42,
            corpus_fingerprint: "deadbeef".to_string(),
            last_analyzed: "2026-03-14T10:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&meta).unwrap();
        let recovered: AnalysisMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(recovered.key, meta.key);
        assert_eq!(recovered.paper_count, meta.paper_count);
        assert_eq!(recovered.corpus_fingerprint, meta.corpus_fingerprint);
        assert_eq!(recovered.last_analyzed, meta.last_analyzed);
    }

    #[test]
    fn test_paper_analysis_empty_tfidf_serializes() {
        let analysis = PaperAnalysis {
            arxiv_id: "2301.99999".to_string(),
            tfidf_vector: HashMap::new(),
            top_terms: vec![],
            top_scores: vec![],
            analyzed_at: "2026-03-14T00:00:00Z".to_string(),
            corpus_fingerprint: "empty".to_string(),
        };
        let json = serde_json::to_string(&analysis).unwrap();
        let recovered: PaperAnalysis = serde_json::from_str(&json).unwrap();
        assert!(recovered.tfidf_vector.is_empty());
        assert!(recovered.top_terms.is_empty());
        assert!(recovered.top_scores.is_empty());
    }

    #[test]
    fn test_paper_analysis_top_terms_top_scores_same_length() {
        let analysis = make_paper_analysis();
        assert_eq!(
            analysis.top_terms.len(),
            analysis.top_scores.len(),
            "top_terms and top_scores must have same length"
        );

        // Also verify after roundtrip
        let json = serde_json::to_string(&analysis).unwrap();
        let recovered: PaperAnalysis = serde_json::from_str(&json).unwrap();
        assert_eq!(recovered.top_terms.len(), recovered.top_scores.len());
    }
}
