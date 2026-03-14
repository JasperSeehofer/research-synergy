use std::collections::HashMap;

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
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), *v))
            .collect()
    }

    #[test]
    fn test_cosine_similarity_identical() {
        let v = vec_from(&[("quantum", 0.8), ("entanglement", 0.6)]);
        let sim = cosine_similarity(&v, &v);
        assert!((sim - 1.0).abs() < 1e-5, "identical vectors should return 1.0, got {sim}");
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec_from(&[("quantum", 0.8)]);
        let b = vec_from(&[("topology", 0.5)]);
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 0.0).abs() < 1e-5, "orthogonal vectors should return 0.0, got {sim}");
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
}
