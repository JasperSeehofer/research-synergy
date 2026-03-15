use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Finding {
    pub text: String,
    pub strength: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Method {
    pub name: String,
    pub category: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LlmAnnotation {
    pub arxiv_id: String,
    pub paper_type: String,
    pub methods: Vec<Method>,
    pub findings: Vec<Finding>,
    pub open_problems: Vec<String>,
    pub provider: String,
    pub model_name: String,
    pub annotated_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_annotation_serde_roundtrip() {
        let ann = LlmAnnotation {
            arxiv_id: "2301.12345".to_string(),
            paper_type: "theoretical".to_string(),
            methods: vec![Method {
                name: "variational method".to_string(),
                category: "analytical".to_string(),
            }],
            findings: vec![Finding {
                text: "Energy gap is non-zero".to_string(),
                strength: "strong_evidence".to_string(),
            }],
            open_problems: vec!["Extension to 3D case".to_string()],
            provider: "noop".to_string(),
            model_name: "noop".to_string(),
            annotated_at: "2026-03-14T10:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&ann).unwrap();
        let decoded: LlmAnnotation = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.arxiv_id, ann.arxiv_id);
        assert_eq!(decoded.paper_type, ann.paper_type);
        assert_eq!(decoded.methods.len(), 1);
        assert_eq!(decoded.methods[0].name, "variational method");
        assert_eq!(decoded.methods[0].category, "analytical");
        assert_eq!(decoded.findings.len(), 1);
        assert_eq!(decoded.findings[0].text, "Energy gap is non-zero");
        assert_eq!(decoded.findings[0].strength, "strong_evidence");
        assert_eq!(decoded.open_problems.len(), 1);
        assert_eq!(decoded.open_problems[0], "Extension to 3D case");
        assert_eq!(decoded.provider, "noop");
    }

    #[test]
    fn test_finding_serde_roundtrip() {
        let f = Finding {
            text: "Some finding".to_string(),
            strength: "moderate_evidence".to_string(),
        };
        let json = serde_json::to_string(&f).unwrap();
        let decoded: Finding = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.text, f.text);
        assert_eq!(decoded.strength, f.strength);
    }

    #[test]
    fn test_method_serde_roundtrip() {
        let m = Method {
            name: "Monte Carlo".to_string(),
            category: "computational".to_string(),
        };
        let json = serde_json::to_string(&m).unwrap();
        let decoded: Method = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.name, m.name);
        assert_eq!(decoded.category, m.category);
    }

    #[test]
    fn test_llm_annotation_empty_vecs_serde() {
        let ann = LlmAnnotation {
            arxiv_id: "2301.12345".to_string(),
            paper_type: "unknown".to_string(),
            methods: vec![],
            findings: vec![],
            open_problems: vec![],
            provider: "noop".to_string(),
            model_name: "noop".to_string(),
            annotated_at: "2026-03-14T10:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&ann).unwrap();
        let decoded: LlmAnnotation = serde_json::from_str(&json).unwrap();
        assert!(decoded.methods.is_empty());
        assert!(decoded.findings.is_empty());
        assert!(decoded.open_problems.is_empty());
    }
}
