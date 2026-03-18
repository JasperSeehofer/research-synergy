use crate::datamodels::extraction::TextExtractionResult;

pub const SYSTEM_PROMPT: &str = r#"You are a scientific paper analyst. Extract structured information from the paper text below.
The paper text is organized by section. Sections marked [EMPTY] were not available.
For each finding and method you extract, include:
- source_section: the section name where this was found (e.g., "results", "methods", "abstract")
- source_snippet: a verbatim 1-2 sentence quote from that section supporting this finding/method
Extract:
- paper_type: experimental | theoretical | review | computational
- methods: array of {name, category, source_section, source_snippet}
- findings: array of {text, strength, source_section, source_snippet}
- open_problems: array of strings
Respond ONLY with a JSON object."#;

pub const RETRY_NUDGE: &str = "IMPORTANT: respond ONLY with valid JSON, nothing else.";

pub const LLM_ANNOTATION_SCHEMA: &str = r#"{
  "type": "object",
  "properties": {
    "arxiv_id": { "type": "string" },
    "paper_type": {
      "type": "string",
      "enum": ["experimental", "theoretical", "review", "computational", "unknown"]
    },
    "methods": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "name": { "type": "string" },
          "category": {
            "type": "string",
            "enum": ["experimental", "theoretical", "computational", "statistical", "analytical"]
          },
          "source_section": { "type": "string" },
          "source_snippet": { "type": "string" }
        },
        "required": ["name", "category"]
      }
    },
    "findings": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "text": { "type": "string" },
          "strength": {
            "type": "string",
            "enum": ["strong_evidence", "moderate_evidence", "preliminary", "inconclusive"]
          },
          "source_section": { "type": "string" },
          "source_snippet": { "type": "string" }
        },
        "required": ["text", "strength"]
      }
    },
    "open_problems": {
      "type": "array",
      "items": { "type": "string" }
    },
    "provider": { "type": "string" },
    "model_name": { "type": "string" },
    "annotated_at": { "type": "string" }
  },
  "required": ["paper_type", "methods", "findings", "open_problems"]
}"#;

const MAX_CHARS: usize = 3000;

fn push_section(parts: &mut Vec<String>, header: &str, text: Option<&str>) {
    let body = match text {
        Some(t) if !t.is_empty() => {
            if t.len() > MAX_CHARS {
                format!("{}[truncated]", &t[..MAX_CHARS])
            } else {
                t.to_string()
            }
        }
        _ => "[EMPTY]".to_string(),
    };
    parts.push(format!("[{}]\n{}", header, body));
}

/// Build a section-aware user message from a `TextExtractionResult`.
///
/// Each section is emitted under a header like `[ABSTRACT]`. Sections that are
/// `None` or empty are represented as `[EMPTY]`. Section text is truncated to
/// `MAX_CHARS` characters with a `[truncated]` suffix.
pub fn build_section_aware_user_message(extraction: &TextExtractionResult) -> String {
    let mut parts: Vec<String> = Vec::new();
    push_section(
        &mut parts,
        "ABSTRACT",
        extraction.sections.abstract_text.as_deref(),
    );
    push_section(
        &mut parts,
        "INTRODUCTION",
        extraction.sections.introduction.as_deref(),
    );
    push_section(
        &mut parts,
        "METHODS",
        extraction.sections.methods.as_deref(),
    );
    push_section(
        &mut parts,
        "RESULTS",
        extraction.sections.results.as_deref(),
    );
    push_section(
        &mut parts,
        "CONCLUSION",
        extraction.sections.conclusion.as_deref(),
    );
    parts.join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datamodels::extraction::{SectionMap, TextExtractionResult, ExtractionMethod};

    fn make_extraction(
        abstract_text: Option<&str>,
        introduction: Option<&str>,
        methods: Option<&str>,
        results: Option<&str>,
        conclusion: Option<&str>,
    ) -> TextExtractionResult {
        TextExtractionResult {
            arxiv_id: "2301.12345".to_string(),
            extraction_method: ExtractionMethod::Ar5ivHtml,
            sections: SectionMap {
                abstract_text: abstract_text.map(String::from),
                introduction: introduction.map(String::from),
                methods: methods.map(String::from),
                results: results.map(String::from),
                conclusion: conclusion.map(String::from),
            },
            is_partial: false,
            extracted_at: "2026-03-18T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_annotation_schema_is_valid_json() {
        let result = serde_json::from_str::<serde_json::Value>(LLM_ANNOTATION_SCHEMA);
        assert!(
            result.is_ok(),
            "LLM_ANNOTATION_SCHEMA must be valid JSON: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_annotation_schema_contains_source_section_and_snippet() {
        // Check that schema has source_section and source_snippet in findings items
        assert!(
            LLM_ANNOTATION_SCHEMA.contains("source_section"),
            "Schema must contain source_section"
        );
        assert!(
            LLM_ANNOTATION_SCHEMA.contains("source_snippet"),
            "Schema must contain source_snippet"
        );
    }

    #[test]
    fn test_section_aware_message_all_sections() {
        let extraction = make_extraction(
            Some("Abstract text."),
            Some("Introduction text."),
            Some("Methods text."),
            Some("Results text."),
            Some("Conclusion text."),
        );
        let msg = build_section_aware_user_message(&extraction);
        assert!(msg.contains("[ABSTRACT]"), "Must contain [ABSTRACT] header");
        assert!(msg.contains("[METHODS]"), "Must contain [METHODS] header");
        assert!(msg.contains("[RESULTS]"), "Must contain [RESULTS] header");
        assert!(msg.contains("[CONCLUSION]"), "Must contain [CONCLUSION] header");
        assert!(msg.contains("Abstract text."));
        assert!(msg.contains("Results text."));
    }

    #[test]
    fn test_section_aware_message_partial_sections() {
        let extraction = make_extraction(
            Some("Only abstract available."),
            None,
            None,
            None,
            None,
        );
        let msg = build_section_aware_user_message(&extraction);
        assert!(msg.contains("[ABSTRACT]"), "Must contain [ABSTRACT] header");
        assert!(msg.contains("Only abstract available."));
        assert!(msg.contains("[METHODS]\n[EMPTY]"), "Missing methods should be [EMPTY]");
    }

    #[test]
    fn test_section_aware_message_truncates_long_sections() {
        let long_text = "x".repeat(MAX_CHARS + 500);
        let extraction = make_extraction(Some(&long_text), None, None, None, None);
        let msg = build_section_aware_user_message(&extraction);
        assert!(
            msg.contains("[truncated]"),
            "Long sections must end with [truncated]"
        );
        // The truncated portion should be MAX_CHARS chars of the original content
        assert!(msg.contains(&"x".repeat(MAX_CHARS)));
    }
}
