use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::datamodels::paper::Paper;
use crate::utils::strip_version_suffix;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub enum ExtractionMethod {
    #[default]
    AbstractOnly,
    Ar5ivHtml,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SectionMap {
    pub abstract_text: Option<String>,
    pub introduction: Option<String>,
    pub methods: Option<String>,
    pub results: Option<String>,
    pub conclusion: Option<String>,
}

impl SectionMap {
    pub fn populated_count(&self) -> usize {
        [
            self.abstract_text.is_some(),
            self.introduction.is_some(),
            self.methods.is_some(),
            self.results.is_some(),
            self.conclusion.is_some(),
        ]
        .iter()
        .filter(|&&x| x)
        .count()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TextExtractionResult {
    pub arxiv_id: String,
    pub extraction_method: ExtractionMethod,
    pub sections: SectionMap,
    pub is_partial: bool,
    pub extracted_at: String,
}

impl TextExtractionResult {
    pub fn from_abstract(paper: &Paper) -> Self {
        TextExtractionResult {
            arxiv_id: strip_version_suffix(&paper.id),
            extraction_method: ExtractionMethod::AbstractOnly,
            sections: SectionMap {
                abstract_text: Some(paper.summary.clone()),
                introduction: None,
                methods: None,
                results: None,
                conclusion: None,
            },
            is_partial: true,
            extracted_at: Utc::now().to_rfc3339(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datamodels::paper::Paper;

    fn make_test_paper(id: &str, summary: &str) -> Paper {
        Paper {
            id: id.to_string(),
            summary: summary.to_string(),
            title: "Test Paper".to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn test_extraction_method_default() {
        let method = ExtractionMethod::default();
        assert_eq!(method, ExtractionMethod::AbstractOnly);
    }

    #[test]
    fn test_from_abstract_sets_is_partial_and_method() {
        let paper = make_test_paper("2301.12345", "This is the abstract text.");
        let result = TextExtractionResult::from_abstract(&paper);
        assert!(result.is_partial);
        assert_eq!(result.extraction_method, ExtractionMethod::AbstractOnly);
        assert_eq!(
            result.sections.abstract_text,
            Some("This is the abstract text.".to_string())
        );
        assert!(result.sections.introduction.is_none());
        assert!(result.sections.methods.is_none());
        assert!(result.sections.results.is_none());
        assert!(result.sections.conclusion.is_none());
    }

    #[test]
    fn test_from_abstract_strips_version_suffix() {
        let paper = make_test_paper("2301.12345v2", "Abstract text.");
        let result = TextExtractionResult::from_abstract(&paper);
        assert_eq!(result.arxiv_id, "2301.12345");
    }

    #[test]
    fn test_section_map_default_all_none() {
        let sections = SectionMap::default();
        assert!(sections.abstract_text.is_none());
        assert!(sections.introduction.is_none());
        assert!(sections.methods.is_none());
        assert!(sections.results.is_none());
        assert!(sections.conclusion.is_none());
    }

    #[test]
    fn test_round_trip_serialization() {
        let paper = make_test_paper("2301.99999", "Abstract content.");
        let result = TextExtractionResult::from_abstract(&paper);
        let json = serde_json::to_string(&result).expect("serialize failed");
        let deserialized: TextExtractionResult =
            serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(deserialized.arxiv_id, result.arxiv_id);
        assert_eq!(deserialized.is_partial, result.is_partial);
        assert_eq!(deserialized.extraction_method, result.extraction_method);
        assert_eq!(
            deserialized.sections.abstract_text,
            result.sections.abstract_text
        );
        assert_eq!(deserialized.extracted_at, result.extracted_at);
    }
}
