use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum GapType {
    Contradiction,
    AbcBridge,
}

impl GapType {
    pub fn as_str(&self) -> &'static str {
        match self {
            GapType::Contradiction => "contradiction",
            GapType::AbcBridge => "abc_bridge",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapFinding {
    pub gap_type: GapType,
    pub paper_ids: Vec<String>,
    pub shared_terms: Vec<String>,
    pub justification: String,
    pub confidence: f32,
    pub found_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gap_finding_contradiction_round_trip() {
        let finding = GapFinding {
            gap_type: GapType::Contradiction,
            paper_ids: vec!["2301.11111".to_string(), "2301.22222".to_string()],
            shared_terms: vec!["quantum".to_string(), "entanglement".to_string()],
            justification: "Paper A claims X while Paper B claims not-X.".to_string(),
            confidence: 0.85,
            found_at: "2026-03-14T10:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&finding).unwrap();
        let deserialized: GapFinding = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.gap_type, GapType::Contradiction);
        assert_eq!(deserialized.paper_ids, vec!["2301.11111", "2301.22222"]);
        assert_eq!(deserialized.shared_terms, vec!["quantum", "entanglement"]);
        assert_eq!(
            deserialized.justification,
            "Paper A claims X while Paper B claims not-X."
        );
        assert!((deserialized.confidence - 0.85).abs() < 1e-5);
        assert_eq!(deserialized.found_at, "2026-03-14T10:00:00Z");
    }

    #[test]
    fn test_gap_finding_abc_bridge_round_trip() {
        let finding = GapFinding {
            gap_type: GapType::AbcBridge,
            paper_ids: vec!["2301.11111".to_string(), "2301.33333".to_string()],
            shared_terms: vec!["topological".to_string()],
            justification:
                "Paper A's methods connect to Paper C's results via topological concepts."
                    .to_string(),
            confidence: 0.72,
            found_at: "2026-03-14T11:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&finding).unwrap();
        let deserialized: GapFinding = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.gap_type, GapType::AbcBridge);
        assert_eq!(deserialized.paper_ids.len(), 2);
        assert_eq!(deserialized.shared_terms.len(), 1);
    }

    #[test]
    fn test_gap_type_as_str() {
        assert_eq!(GapType::Contradiction.as_str(), "contradiction");
        assert_eq!(GapType::AbcBridge.as_str(), "abc_bridge");
    }
}
