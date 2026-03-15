pub const SYSTEM_PROMPT: &str = r#"You are a scientific paper analyst. Extract structured information from the paper abstract.

Extract the following:
- paper_type: classify the paper as one of: experimental, theoretical, review, computational
- methods: array of methods used, each with:
  - name: method name (string)
  - category: one of: experimental, theoretical, computational, statistical, analytical
- findings: array of key findings, each with:
  - text: description of the finding (string)
  - strength: evidence strength, one of: strong_evidence, moderate_evidence, preliminary, inconclusive
- open_problems: array of strings describing open questions or future work mentioned

Respond ONLY with a JSON object. No text before or after."#;

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
          }
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
          }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_annotation_schema_is_valid_json() {
        let result = serde_json::from_str::<serde_json::Value>(LLM_ANNOTATION_SCHEMA);
        assert!(
            result.is_ok(),
            "LLM_ANNOTATION_SCHEMA must be valid JSON: {:?}",
            result.err()
        );
    }
}
