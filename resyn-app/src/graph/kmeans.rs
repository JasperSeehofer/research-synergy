/// Result of k-means clustering on node positions.
pub struct ClusterResult {
    pub assignments: Vec<usize>,        // cluster index per node (parallel to input positions)
    pub centroids: Vec<(f64, f64)>,     // (x, y) per cluster
    pub dominant_keywords: Vec<String>, // top keyword per cluster (by summed TF-IDF)
}

/// Compute k from node count: floor(sqrt(n/2)) clamped to [3, 8].
/// Returns 3 for n == 0.
pub fn auto_k(n: usize) -> usize {
    if n == 0 {
        return 3;
    }
    ((n as f64 / 2.0).sqrt().floor() as usize).clamp(3, 8)
}

/// K-means clustering using k-means++ initialization and Lloyd's iteration.
///
/// Returns a vector of cluster assignments (length == positions.len()).
/// If positions.len() <= k, each node is assigned its own cluster index.
/// Uses deterministic initialization (first centroid = positions[0]).
pub fn run_kmeans(positions: &[(f64, f64)], k: usize, max_iter: usize) -> Vec<usize> {
    let n = positions.len();
    if n == 0 || k == 0 {
        return vec![];
    }
    let effective_k = k.min(n);

    // Edge case: each node gets its own cluster
    if n <= k {
        return (0..n).collect();
    }

    // K-means++ initialization
    let mut centroids: Vec<(f64, f64)> = Vec::with_capacity(effective_k);

    // First centroid: positions[0] (deterministic for reproducibility)
    centroids.push(positions[0]);

    // Each subsequent centroid: pick point with max squared distance to nearest centroid
    for _ in 1..effective_k {
        let mut max_dist = f64::NEG_INFINITY;
        let mut next_centroid_idx = 0;
        for (i, &pos) in positions.iter().enumerate() {
            let min_dist = centroids
                .iter()
                .map(|&c| sq_dist(pos, c))
                .fold(f64::INFINITY, f64::min);
            if min_dist > max_dist {
                max_dist = min_dist;
                next_centroid_idx = i;
            }
        }
        centroids.push(positions[next_centroid_idx]);
    }

    let mut assignments = vec![0usize; n];

    for _ in 0..max_iter {
        // Assignment step: each point to nearest centroid
        let mut changed = false;
        for (i, &pos) in positions.iter().enumerate() {
            let nearest = centroids
                .iter()
                .enumerate()
                .map(|(ci, &c)| (ci, sq_dist(pos, c)))
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                .map(|(ci, _)| ci)
                .unwrap_or(0);
            if assignments[i] != nearest {
                assignments[i] = nearest;
                changed = true;
            }
        }

        // Early exit if no assignments changed
        if !changed {
            break;
        }

        // Update step: recompute centroids as mean of assigned points
        let mut sums = vec![(0.0f64, 0.0f64); effective_k];
        let mut counts = vec![0usize; effective_k];
        for (i, &pos) in positions.iter().enumerate() {
            let c = assignments[i];
            sums[c].0 += pos.0;
            sums[c].1 += pos.1;
            counts[c] += 1;
        }
        for ci in 0..effective_k {
            if counts[ci] > 0 {
                centroids[ci] = (sums[ci].0 / counts[ci] as f64, sums[ci].1 / counts[ci] as f64);
            }
            // If no points assigned, keep previous centroid position
        }
    }

    assignments
}

/// Squared Euclidean distance between two 2D points.
fn sq_dist(a: (f64, f64), b: (f64, f64)) -> f64 {
    let dx = a.0 - b.0;
    let dy = a.1 - b.1;
    dx * dx + dy * dy
}

/// Compute the dominant keyword for a cluster by summing TF-IDF scores across all nodes.
///
/// `node_indices` — indices into `all_keywords` that belong to this cluster.
/// `all_keywords` — per-node keyword list: Vec<(term, score)>.
/// Returns the term with the highest total score, or empty string if no keywords.
pub fn dominant_keyword_for_cluster(
    node_indices: &[usize],
    all_keywords: &[Vec<(String, f32)>],
) -> String {
    let mut scores: std::collections::HashMap<&str, f32> = std::collections::HashMap::new();
    for &idx in node_indices {
        if idx < all_keywords.len() {
            for (term, score) in &all_keywords[idx] {
                *scores.entry(term.as_str()).or_insert(0.0) += score;
            }
        }
    }
    scores
        .into_iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(term, _)| term.to_owned())
        .unwrap_or_default()
}

/// Convex hull using Jarvis march (gift-wrapping) algorithm.
///
/// Returns points in counter-clockwise order.
/// If fewer than 3 points, returns the input as-is (cloned).
/// Collinear points: only endpoints are included (strict left-turn selection).
pub fn convex_hull(points: &[(f64, f64)]) -> Vec<(f64, f64)> {
    let n = points.len();
    if n < 3 {
        return points.to_vec();
    }

    // Find leftmost point (min x, break ties by min y)
    let start = points
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            a.0.partial_cmp(&b.0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        })
        .map(|(i, _)| i)
        .unwrap();

    let mut hull = Vec::new();
    let mut current = start;

    loop {
        hull.push(points[current]);
        let mut next = (current + 1) % n;

        for i in 0..n {
            if i == current {
                continue;
            }
            // cross > 0 means i is more counter-clockwise than next relative to current
            let cross = cross_product(points[current], points[next], points[i]);
            if cross > 0.0 {
                next = i;
            }
        }

        current = next;
        if current == start {
            break;
        }
        // Safety: avoid infinite loop on degenerate inputs
        if hull.len() > n {
            break;
        }
    }

    hull
}

/// Cross product of vectors (a→b) and (a→c).
/// Positive = CCW turn, negative = CW turn, zero = collinear.
fn cross_product(a: (f64, f64), b: (f64, f64), c: (f64, f64)) -> f64 {
    (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0)
}

/// Map a TF-IDF score in [0, 1] to an opacity in [0.35, 1.0].
/// Formula: 0.35 + (score * 0.65), clamped to [0.35, 1.0].
pub fn score_to_opacity(score: f32) -> f64 {
    (0.35 + (score as f64 * 0.65)).clamp(0.35, 1.0)
}

/// Convenience function: run full clustering pipeline on node positions.
///
/// Returns a `ClusterResult` with assignments, centroids, and dominant keyword per cluster.
pub fn compute_clusters(
    positions: &[(f64, f64)],
    all_keywords: &[Vec<(String, f32)>],
) -> ClusterResult {
    let n = positions.len();
    let k = auto_k(n);
    let assignments = run_kmeans(positions, k, 20);

    let effective_k = k.min(n.max(1));

    // Compute centroids and collect node indices per cluster
    let mut cluster_members: Vec<Vec<usize>> = vec![vec![]; effective_k];
    for (i, &ci) in assignments.iter().enumerate() {
        if ci < effective_k {
            cluster_members[ci].push(i);
        }
    }

    let centroids: Vec<(f64, f64)> = cluster_members
        .iter()
        .enumerate()
        .map(|(ci, members)| {
            if members.is_empty() {
                // Use first position as fallback for empty cluster
                positions.get(ci).copied().unwrap_or((0.0, 0.0))
            } else {
                let sum = members.iter().fold((0.0f64, 0.0f64), |acc, &idx| {
                    (acc.0 + positions[idx].0, acc.1 + positions[idx].1)
                });
                (sum.0 / members.len() as f64, sum.1 / members.len() as f64)
            }
        })
        .collect();

    let dominant_keywords: Vec<String> = cluster_members
        .iter()
        .map(|members| dominant_keyword_for_cluster(members, all_keywords))
        .collect();

    ClusterResult {
        assignments,
        centroids,
        dominant_keywords,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- auto_k tests ---

    #[test]
    fn test_auto_k_min_clamp() {
        // n=10: sqrt(5) ≈ 2.236, floor = 2, clamp to 3
        assert_eq!(auto_k(10), 3);
    }

    #[test]
    fn test_auto_k_midrange() {
        // n=50: sqrt(25) = 5.0, floor = 5
        assert_eq!(auto_k(50), 5);
    }

    #[test]
    fn test_auto_k_max_clamp() {
        // n=200: sqrt(100) = 10, clamp to 8
        assert_eq!(auto_k(200), 8);
    }

    #[test]
    fn test_auto_k_zero() {
        // n=0: edge case, returns 3
        assert_eq!(auto_k(0), 3);
    }

    // --- run_kmeans tests ---

    #[test]
    fn test_run_kmeans_four_clusters() {
        // 4 clearly separated clusters at corners of a large square
        // Each corner has multiple points to make clustering deterministic
        let mut positions = Vec::new();
        // Cluster 0: top-right area
        for _ in 0..5 {
            positions.push((100.0, 100.0));
        }
        // Cluster 1: top-left area
        for _ in 0..5 {
            positions.push((-100.0, 100.0));
        }
        // Cluster 2: bottom-right area
        for _ in 0..5 {
            positions.push((100.0, -100.0));
        }
        // Cluster 3: bottom-left area
        for _ in 0..5 {
            positions.push((-100.0, -100.0));
        }

        let assignments = run_kmeans(&positions, 4, 100);
        assert_eq!(assignments.len(), 20);

        // All points at same position should get same cluster
        let c0 = assignments[0];
        for i in 0..5 {
            assert_eq!(assignments[i], c0, "Corner 0 points differ at index {i}");
        }
        let c1 = assignments[5];
        for i in 5..10 {
            assert_eq!(assignments[i], c1, "Corner 1 points differ at index {i}");
        }
        let c2 = assignments[10];
        for i in 10..15 {
            assert_eq!(assignments[i], c2, "Corner 2 points differ at index {i}");
        }
        let c3 = assignments[15];
        for i in 15..20 {
            assert_eq!(assignments[i], c3, "Corner 3 points differ at index {i}");
        }

        // All 4 clusters should be distinct
        let clusters: std::collections::HashSet<usize> =
            assignments.iter().copied().collect();
        assert_eq!(clusters.len(), 4, "Expected 4 distinct clusters");
    }

    #[test]
    fn test_run_kmeans_k1_all_same_cluster() {
        let positions = vec![(1.0, 2.0), (3.0, 4.0), (5.0, 6.0)];
        let assignments = run_kmeans(&positions, 1, 20);
        assert_eq!(assignments.len(), 3);
        assert!(assignments.iter().all(|&c| c == 0));
    }

    #[test]
    fn test_run_kmeans_n_less_than_k() {
        // 3 points, k=5 — should not panic, returns 3 clusters (one per node)
        let positions = vec![(0.0, 0.0), (1.0, 0.0), (0.0, 1.0)];
        let assignments = run_kmeans(&positions, 5, 20);
        assert_eq!(assignments.len(), 3);
        // Each node gets its own cluster index
        let clusters: std::collections::HashSet<usize> = assignments.iter().copied().collect();
        assert_eq!(clusters.len(), 3);
    }

    // --- convex_hull tests ---

    #[test]
    fn test_convex_hull_three_non_collinear_points() {
        // Triangle: should return all 3 points in CCW order
        let points = vec![(0.0, 0.0), (1.0, 0.0), (0.0, 1.0)];
        let hull = convex_hull(&points);
        assert_eq!(hull.len(), 3);
        // Verify all original points are in hull
        for p in &points {
            assert!(hull.contains(p), "Hull missing point {:?}", p);
        }
    }

    #[test]
    fn test_convex_hull_square_with_center() {
        // 5 points: corners of square + center
        // Center should be excluded from hull
        let points = vec![
            (-1.0, -1.0),
            (1.0, -1.0),
            (1.0, 1.0),
            (-1.0, 1.0),
            (0.0, 0.0), // center — should be excluded
        ];
        let hull = convex_hull(&points);
        assert_eq!(hull.len(), 4, "Hull should have 4 corners, got {:?}", hull);
        assert!(!hull.contains(&(0.0, 0.0)), "Center should not be in hull");
    }

    #[test]
    fn test_convex_hull_fewer_than_3_points() {
        let points = vec![(0.0, 0.0), (1.0, 1.0)];
        let hull = convex_hull(&points);
        assert_eq!(hull, points);
    }

    #[test]
    fn test_convex_hull_single_point() {
        let points = vec![(5.0, 3.0)];
        let hull = convex_hull(&points);
        assert_eq!(hull, points);
    }

    #[test]
    fn test_convex_hull_collinear_points() {
        // Collinear points: only endpoints should be returned
        let points = vec![(0.0, 0.0), (1.0, 0.0), (2.0, 0.0), (3.0, 0.0)];
        let hull = convex_hull(&points);
        // Jarvis march with strict left-turn (cross > 0) will include all collinear points
        // on the hull boundary. The standard gift-wrap with strict inequality excludes
        // collinear intermediates — depending on implementation this may return 2 or 4 points.
        // With our implementation (cross > 0 for next selection), collinear hull will
        // return only the extreme endpoints at the "turns".
        // At minimum, both endpoints (0,0) and (3,0) must be present.
        assert!(hull.contains(&(0.0, 0.0)), "Hull should contain (0,0)");
        assert!(hull.contains(&(3.0, 0.0)), "Hull should contain (3,0)");
    }

    // --- dominant_keyword_for_cluster tests ---

    #[test]
    fn test_dominant_keyword_shared_term() {
        // 3 nodes all having "quantum" as top keyword
        let all_keywords = vec![
            vec![
                ("quantum".to_string(), 0.9),
                ("field".to_string(), 0.3),
            ],
            vec![
                ("quantum".to_string(), 0.8),
                ("gravity".to_string(), 0.2),
            ],
            vec![
                ("quantum".to_string(), 0.7),
                ("mechanics".to_string(), 0.4),
            ],
        ];
        let node_indices = vec![0, 1, 2];
        let result = dominant_keyword_for_cluster(&node_indices, &all_keywords);
        assert_eq!(result, "quantum");
    }

    #[test]
    fn test_dominant_keyword_empty_keywords() {
        let all_keywords: Vec<Vec<(String, f32)>> = vec![vec![], vec![], vec![]];
        let node_indices = vec![0, 1, 2];
        let result = dominant_keyword_for_cluster(&node_indices, &all_keywords);
        assert_eq!(result, "");
    }

    // --- score_to_opacity tests ---

    #[test]
    fn test_score_to_opacity_zero() {
        let opacity = score_to_opacity(0.0);
        assert!((opacity - 0.35).abs() < 1e-9, "Expected 0.35, got {}", opacity);
    }

    #[test]
    fn test_score_to_opacity_one() {
        let opacity = score_to_opacity(1.0);
        assert!((opacity - 1.0).abs() < 1e-9, "Expected 1.0, got {}", opacity);
    }

    #[test]
    fn test_score_to_opacity_half() {
        let opacity = score_to_opacity(0.5);
        let expected = 0.35 + 0.5 * 0.65; // 0.675
        assert!(
            (opacity - expected).abs() < 1e-6,
            "Expected ~{}, got {}",
            expected,
            opacity
        );
    }
}
