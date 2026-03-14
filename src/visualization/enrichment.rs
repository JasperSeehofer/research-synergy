use egui::{Color32, Shape, Stroke};
use egui_graphs::{DefaultEdgeShape, DisplayEdge, DisplayNode, DrawContext, EdgeProps, Node};
use petgraph::EdgeType;
use petgraph::stable_graph::IndexType;

use crate::datamodels::llm_annotation::Finding;

/// Gray color for nodes that have not been LLM-analyzed.
pub const GRAY_UNANALYZED: Color32 = Color32::from_rgb(140, 140, 140);

/// Default node color for the raw (non-enriched) view — neutral gray matching egui_graphs default.
pub const DEFAULT_NODE_COLOR: Color32 = Color32::from_rgb(128, 128, 128);

/// Base radius for graph nodes in the enriched view.
pub const BASE_RADIUS: f32 = 5.0;

/// Maps a paper type string to a muted academic color.
///
/// Matching is case-insensitive. Unknown types return `GRAY_UNANALYZED`.
pub fn paper_type_to_color(paper_type: &str) -> Color32 {
    match paper_type.to_lowercase().as_str() {
        "theoretical" => Color32::from_rgb(100, 140, 200),
        "experimental" => Color32::from_rgb(90, 170, 110),
        "review" => Color32::from_rgb(200, 160, 70),
        "computational" => Color32::from_rgb(150, 100, 190),
        _ => GRAY_UNANALYZED,
    }
}

/// Computes node radius based on the maximum finding strength across all findings.
///
/// Multipliers:
/// - `strong_evidence`   → 3.0x base
/// - `moderate_evidence` → 2.0x base
/// - `weak_evidence`     → 1.5x base
/// - unknown / empty     → 1.0x base
///
/// Returns `base * multiplier` where the multiplier is the maximum across all findings.
pub fn finding_strength_radius(findings: &[Finding], base: f32) -> f32 {
    let multiplier = findings
        .iter()
        .map(|f| match f.strength.as_str() {
            "strong_evidence" => 3.0_f32,
            "moderate_evidence" => 2.0_f32,
            "weak_evidence" => 1.5_f32,
            _ => 1.0_f32,
        })
        .fold(1.0_f32, f32::max);
    base * multiplier
}

/// A custom edge display shape that delegates to [`DefaultEdgeShape`] but allows an optional
/// color override. When `color_override` is `Some(c)`, the edge is drawn in color `c` instead of
/// the default theme color. This is used to tint edges by their source node's paper type color in
/// enriched view.
#[derive(Clone, Debug)]
pub struct TintedEdgeShape {
    inner: DefaultEdgeShape,
    /// When set, overrides the default edge stroke color.
    pub color_override: Option<Color32>,
}

impl<E: Clone> From<EdgeProps<E>> for TintedEdgeShape {
    fn from(props: EdgeProps<E>) -> Self {
        Self {
            inner: DefaultEdgeShape::from(props),
            color_override: None,
        }
    }
}

impl<N, E, Ty, Ix, D> DisplayEdge<N, E, Ty, Ix, D> for TintedEdgeShape
where
    N: Clone,
    E: Clone,
    Ty: EdgeType,
    Ix: IndexType,
    D: DisplayNode<N, E, Ty, Ix>,
{
    fn shapes(
        &mut self,
        start: &Node<N, E, Ty, Ix, D>,
        end: &Node<N, E, Ty, Ix, D>,
        ctx: &DrawContext,
    ) -> Vec<Shape> {
        if let Some(color) = self.color_override {
            // Build shapes using the default implementation, then patch the stroke color.
            let mut shapes = self.inner.shapes(start, end, ctx);
            patch_stroke_color(&mut shapes, color);
            shapes
        } else {
            self.inner.shapes(start, end, ctx)
        }
    }

    fn update(&mut self, state: &EdgeProps<E>) {
        <DefaultEdgeShape as DisplayEdge<N, E, Ty, Ix, D>>::update(&mut self.inner, state);
    }

    fn is_inside(
        &self,
        start: &Node<N, E, Ty, Ix, D>,
        end: &Node<N, E, Ty, Ix, D>,
        pos: egui::Pos2,
    ) -> bool {
        self.inner.is_inside(start, end, pos)
    }
}

/// Patches every [`Shape::LineSegment`] and [`Shape::Path`] stroke color in a shape list.
/// Used by [`TintedEdgeShape`] to override the edge color without duplicating the rendering logic.
fn patch_stroke_color(shapes: &mut [Shape], color: Color32) {
    for shape in shapes.iter_mut() {
        match shape {
            Shape::LineSegment { stroke, .. } => {
                *stroke = Stroke::new(stroke.width, color);
            }
            Shape::Path(path) => {
                path.stroke = egui::epaint::PathStroke::new(path.stroke.width, color);
            }
            Shape::CubicBezier(b) => {
                b.stroke = egui::epaint::PathStroke::new(b.stroke.width, color);
            }
            Shape::QuadraticBezier(q) => {
                q.stroke = egui::epaint::PathStroke::new(q.stroke.width, color);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- paper_type_to_color tests ---

    #[test]
    fn test_theoretical_returns_muted_blue() {
        let color = paper_type_to_color("theoretical");
        // Muted blue: blue channel should dominate
        assert!(
            color.b() > color.r() && color.b() > color.g(),
            "Expected blue-dominant color, got r={} g={} b={}",
            color.r(),
            color.g(),
            color.b()
        );
    }

    #[test]
    fn test_experimental_returns_muted_green() {
        let color = paper_type_to_color("experimental");
        // Muted green: green channel should dominate
        assert!(
            color.g() > color.r() && color.g() > color.b(),
            "Expected green-dominant color, got r={} g={} b={}",
            color.r(),
            color.g(),
            color.b()
        );
    }

    #[test]
    fn test_review_returns_muted_amber() {
        let color = paper_type_to_color("review");
        // Muted amber: red and green channels should both be high, blue low
        assert!(
            color.r() > color.b() && color.g() > color.b(),
            "Expected amber (high r,g low b), got r={} g={} b={}",
            color.r(),
            color.g(),
            color.b()
        );
    }

    #[test]
    fn test_computational_returns_muted_purple() {
        let color = paper_type_to_color("computational");
        // Muted purple: red and blue channels should both be notably present, green lower
        assert!(
            color.r() > color.g() && color.b() > color.g(),
            "Expected purple (r,b > g), got r={} g={} b={}",
            color.r(),
            color.g(),
            color.b()
        );
    }

    #[test]
    fn test_unknown_type_returns_gray_unanalyzed() {
        let color = paper_type_to_color("unknown_type");
        assert_eq!(color, GRAY_UNANALYZED);
    }

    #[test]
    fn test_empty_string_returns_gray_unanalyzed() {
        let color = paper_type_to_color("");
        assert_eq!(color, GRAY_UNANALYZED);
    }

    #[test]
    fn test_case_insensitive_theoretical() {
        let lower = paper_type_to_color("theoretical");
        let upper = paper_type_to_color("Theoretical");
        let mixed = paper_type_to_color("THEORETICAL");
        assert_eq!(lower, upper);
        assert_eq!(lower, mixed);
    }

    #[test]
    fn test_case_insensitive_experimental() {
        let lower = paper_type_to_color("experimental");
        let title = paper_type_to_color("Experimental");
        assert_eq!(lower, title);
    }

    // --- finding_strength_radius tests ---

    #[test]
    fn test_empty_findings_returns_1x_base() {
        let radius = finding_strength_radius(&[], BASE_RADIUS);
        assert!(
            (radius - BASE_RADIUS).abs() < 1e-6,
            "Expected {BASE_RADIUS}, got {radius}"
        );
    }

    #[test]
    fn test_strong_evidence_returns_3x_base() {
        let findings = vec![Finding {
            text: "A finding".to_string(),
            strength: "strong_evidence".to_string(),
        }];
        let radius = finding_strength_radius(&findings, BASE_RADIUS);
        assert!(
            (radius - 3.0 * BASE_RADIUS).abs() < 1e-6,
            "Expected {}, got {radius}",
            3.0 * BASE_RADIUS
        );
    }

    #[test]
    fn test_moderate_evidence_returns_2x_base() {
        let findings = vec![Finding {
            text: "A finding".to_string(),
            strength: "moderate_evidence".to_string(),
        }];
        let radius = finding_strength_radius(&findings, BASE_RADIUS);
        assert!(
            (radius - 2.0 * BASE_RADIUS).abs() < 1e-6,
            "Expected {}, got {radius}",
            2.0 * BASE_RADIUS
        );
    }

    #[test]
    fn test_weak_evidence_returns_1_5x_base() {
        let findings = vec![Finding {
            text: "A finding".to_string(),
            strength: "weak_evidence".to_string(),
        }];
        let radius = finding_strength_radius(&findings, BASE_RADIUS);
        assert!(
            (radius - 1.5 * BASE_RADIUS).abs() < 1e-6,
            "Expected {}, got {radius}",
            1.5 * BASE_RADIUS
        );
    }

    #[test]
    fn test_unknown_strength_returns_1x_base() {
        let findings = vec![Finding {
            text: "A finding".to_string(),
            strength: "speculative".to_string(),
        }];
        let radius = finding_strength_radius(&findings, BASE_RADIUS);
        assert!(
            (radius - BASE_RADIUS).abs() < 1e-6,
            "Expected {BASE_RADIUS}, got {radius}"
        );
    }

    #[test]
    fn test_picks_max_strength_across_findings() {
        // Multiple findings — should pick strong_evidence (3x)
        let findings = vec![
            Finding {
                text: "weak".to_string(),
                strength: "weak_evidence".to_string(),
            },
            Finding {
                text: "strong".to_string(),
                strength: "strong_evidence".to_string(),
            },
            Finding {
                text: "moderate".to_string(),
                strength: "moderate_evidence".to_string(),
            },
        ];
        let radius = finding_strength_radius(&findings, BASE_RADIUS);
        assert!(
            (radius - 3.0 * BASE_RADIUS).abs() < 1e-6,
            "Expected max (strong_evidence = 3x base = {}), got {radius}",
            3.0 * BASE_RADIUS
        );
    }

    #[test]
    fn test_custom_base_radius() {
        let findings = vec![Finding {
            text: "Finding".to_string(),
            strength: "moderate_evidence".to_string(),
        }];
        let custom_base = 10.0_f32;
        let radius = finding_strength_radius(&findings, custom_base);
        assert!((radius - 20.0).abs() < 1e-6, "Expected 20.0, got {radius}");
    }
}
