/// Barnes-Hut quadtree for O(n log n) repulsion computation.

#[derive(Debug, Clone)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }

    /// Returns the four quadrant sub-rects (NW, NE, SW, SE).
    pub fn subdivide(&self) -> [Rect; 4] {
        let hw = self.width / 2.0;
        let hh = self.height / 2.0;
        [
            Rect { x: self.x, y: self.y, width: hw, height: hh },
            Rect { x: self.x + hw, y: self.y, width: hw, height: hh },
            Rect { x: self.x, y: self.y + hh, width: hw, height: hh },
            Rect { x: self.x + hw, y: self.y + hh, width: hw, height: hh },
        ]
    }

    /// Return which quadrant index (0-3) a point falls in.
    pub fn quadrant(&self, x: f64, y: f64) -> usize {
        let hw = self.width / 2.0;
        let hh = self.height / 2.0;
        let right = x >= self.x + hw;
        let bottom = y >= self.y + hh;
        match (right, bottom) {
            (false, false) => 0, // NW
            (true, false) => 1,  // NE
            (false, true) => 2,  // SW
            (true, true) => 3,   // SE
        }
    }
}

/// A node stored in a leaf cell: its index, x, y, mass.
#[derive(Debug, Clone)]
struct LeafNode {
    idx: usize,
    x: f64,
    y: f64,
    mass: f64,
}

#[derive(Debug)]
pub struct QuadTree {
    pub bounds: Rect,
    pub center_of_mass: (f64, f64),
    pub total_mass: f64,
    pub children: Option<Box<[QuadTree; 4]>>,
    /// Stored when this is a leaf (exactly one node).
    leaf: Option<LeafNode>,
}

impl QuadTree {
    fn new(bounds: Rect) -> Self {
        QuadTree {
            bounds,
            center_of_mass: (0.0, 0.0),
            total_mass: 0.0,
            children: None,
            leaf: None,
        }
    }

    /// Build a QuadTree from a set of positions and masses.
    pub fn build(positions: &[(f64, f64)], masses: &[f64]) -> QuadTree {
        if positions.is_empty() {
            return QuadTree::new(Rect { x: 0.0, y: 0.0, width: 1.0, height: 1.0 });
        }

        // Compute bounding rect.
        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;
        for &(px, py) in positions {
            if px < min_x { min_x = px; }
            if py < min_y { min_y = py; }
            if px > max_x { max_x = px; }
            if py > max_y { max_y = py; }
        }

        // Make square and add padding to avoid degenerate zero-size cells.
        let pad = 1.0;
        let size = (max_x - min_x).max(max_y - min_y) + pad * 2.0;
        let bounds = Rect { x: min_x - pad, y: min_y - pad, width: size, height: size };

        let mut tree = QuadTree::new(bounds);
        for (i, (&pos, &mass)) in positions.iter().zip(masses.iter()).enumerate() {
            tree.insert(i, pos.0, pos.1, mass);
        }
        tree
    }

    /// Insert a node into the tree, subdividing as needed.
    pub fn insert(&mut self, idx: usize, x: f64, y: f64, mass: f64) {
        // Update aggregate statistics.
        let new_total = self.total_mass + mass;
        self.center_of_mass = (
            (self.center_of_mass.0 * self.total_mass + x * mass) / new_total,
            (self.center_of_mass.1 * self.total_mass + y * mass) / new_total,
        );
        self.total_mass = new_total;

        if self.children.is_none() && self.leaf.is_none() {
            // Empty leaf — store this node here.
            self.leaf = Some(LeafNode { idx, x, y, mass });
            return;
        }

        if self.children.is_none() {
            // This is a leaf with an existing node — subdivide.
            let existing = self.leaf.take().unwrap();

            let subs = self.bounds.subdivide();
            let children = Box::new([
                QuadTree::new(subs[0].clone()),
                QuadTree::new(subs[1].clone()),
                QuadTree::new(subs[2].clone()),
                QuadTree::new(subs[3].clone()),
            ]);
            self.children = Some(children);

            // Re-insert existing node into children.
            let q = self.bounds.quadrant(existing.x, existing.y);
            if let Some(ref mut children) = self.children {
                children[q].insert(existing.idx, existing.x, existing.y, existing.mass);
            }
        }

        // Insert new node into appropriate child.
        let q = self.bounds.quadrant(x, y);
        if let Some(ref mut children) = self.children {
            children[q].insert(idx, x, y, mass);
        }
    }

    /// Returns true if this is a leaf (external) node — no children and contains at most one node.
    pub fn is_leaf(&self) -> bool {
        self.children.is_none()
    }
}

/// Compute Barnes-Hut repulsive force on a particle at (x, y) with given mass.
///
/// `theta` — opening angle criterion. Smaller = more accurate, larger = faster.
/// Returns (fx, fy) force vector.
pub fn barnes_hut_repulsion(tree: &QuadTree, x: f64, y: f64, mass: f64, theta: f64) -> (f64, f64) {
    if tree.total_mass == 0.0 {
        return (0.0, 0.0);
    }

    // Skip self-interaction for leaf nodes containing the queried point.
    if tree.is_leaf() {
        if let Some(ref leaf) = tree.leaf {
            let dx = leaf.x - x;
            let dy = leaf.y - y;
            if dx.abs() < 1e-10 && dy.abs() < 1e-10 {
                // This leaf IS the queried particle — skip.
                return (0.0, 0.0);
            }
        }
    }

    let dx = tree.center_of_mass.0 - x;
    let dy = tree.center_of_mass.1 - y;
    let dist_sq = (dx * dx + dy * dy).max(1.0);
    let dist = dist_sq.sqrt();

    // Barnes-Hut criterion: if leaf or s/d < theta, treat as single body.
    let s = tree.bounds.width.max(tree.bounds.height);
    if tree.is_leaf() || (s / dist) < theta {
        // Repulsion: F = k * m1 * m2 / r^2, directed away from tree's CoM.
        const REPULSION_K: f64 = -30.0;
        let force = REPULSION_K * mass * tree.total_mass / dist_sq;
        // Direction: from CoM toward our particle (negate dx/dy since dx = CoM - particle).
        let fx = -force * dx / dist;
        let fy = -force * dy / dist;
        return (fx, fy);
    }

    // Recurse into children.
    let mut fx = 0.0;
    let mut fy = 0.0;
    if let Some(ref children) = tree.children {
        for child in children.iter() {
            if child.total_mass > 0.0 {
                let (cfx, cfy) = barnes_hut_repulsion(child, x, y, mass, theta);
                fx += cfx;
                fy += cfy;
            }
        }
    }
    (fx, fy)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quadtree_total_mass_two_nodes() {
        let positions = vec![(0.0_f64, 0.0_f64), (100.0, 100.0)];
        let masses = vec![1.0_f64, 1.0_f64];
        let tree = QuadTree::build(&positions, &masses);
        assert!((tree.total_mass - 2.0).abs() < 1e-10, "total_mass should be 2.0, got {}", tree.total_mass);
    }

    #[test]
    fn test_quadtree_center_of_mass_two_nodes() {
        let positions = vec![(0.0_f64, 0.0_f64), (100.0, 100.0)];
        let masses = vec![1.0_f64, 1.0_f64];
        let tree = QuadTree::build(&positions, &masses);
        let (cx, cy) = tree.center_of_mass;
        assert!((cx - 50.0).abs() < 1e-6, "center_x should be 50.0, got {}", cx);
        assert!((cy - 50.0).abs() < 1e-6, "center_y should be 50.0, got {}", cy);
    }

    #[test]
    fn test_barnes_hut_repulsion_nonzero_for_separated_nodes() {
        let positions = vec![(0.0_f64, 0.0_f64), (100.0, 0.0)];
        let masses = vec![1.0_f64, 1.0_f64];
        let tree = QuadTree::build(&positions, &masses);
        let (fx, fy) = barnes_hut_repulsion(&tree, 0.0, 0.0, 1.0, 0.9);
        let mag = (fx * fx + fy * fy).sqrt();
        assert!(mag > 0.0, "repulsion force should be non-zero, got magnitude {}", mag);
    }

    #[test]
    fn test_barnes_hut_repulsion_decreases_with_distance() {
        // Two-body tree: source at origin, query at two different distances.
        let positions = vec![(0.0_f64, 0.0_f64), (1.0, 0.0)];
        let masses = vec![1.0_f64, 1.0_f64];
        let tree = QuadTree::build(&positions, &masses);

        let (fx1, _) = barnes_hut_repulsion(&tree, 10.0, 0.0, 1.0, 0.9);
        let (fx2, _) = barnes_hut_repulsion(&tree, 50.0, 0.0, 1.0, 0.9);

        let mag1 = fx1.abs();
        let mag2 = fx2.abs();
        assert!(mag1 > mag2, "force at distance 10 ({}) should be greater than at 50 ({})", mag1, mag2);
    }

    #[test]
    fn test_quadtree_build_empty() {
        let tree = QuadTree::build(&[], &[]);
        assert_eq!(tree.total_mass, 0.0);
    }

    #[test]
    fn test_quadtree_single_node() {
        let positions = vec![(5.0_f64, 7.0_f64)];
        let masses = vec![2.0_f64];
        let tree = QuadTree::build(&positions, &masses);
        assert!((tree.total_mass - 2.0).abs() < 1e-10);
        assert!((tree.center_of_mass.0 - 5.0).abs() < 1e-10);
        assert!((tree.center_of_mass.1 - 7.0).abs() < 1e-10);
    }
}
