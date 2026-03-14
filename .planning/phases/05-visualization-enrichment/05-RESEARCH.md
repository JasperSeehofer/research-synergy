# Phase 5: Visualization Enrichment - Research

**Researched:** 2026-03-14
**Domain:** egui/egui_graphs per-node visual encoding, tooltip integration, analysis data wiring
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Color Palette & Mapping**
- Muted academic palette: soft, distinguishable tones — blue for theoretical, green for experimental, amber for review, purple for computational
- Papers with no analysis data rendered in neutral gray — clearly signals "not yet analyzed"
- Edges tinted by source node's paper type color — shows how theoretical vs experimental papers cite each other
- Color legend displayed in the right panel's new Analysis section, visible when enriched view is active

**Node Sizing Strategy**
- Strongest finding wins: node size = max finding strength across all findings for that paper
- Strength mapping: strong_evidence = 3x base, moderate_evidence = 2x base, weak_evidence = 1.5x base (moderate 1x to 3x range)
- Papers with no analysis data use default/medium base size — unchanged from raw view
- Size range chosen to provide clear visual hierarchy without overwhelming Fruchterman-Reingold layout

**Toggle Placement & Behavior**
- New "Analysis" collapsible section in right panel, placed between Simulation and Widget sections
- Simple checkbox control: "Enriched view" — unchecked = raw, checked = enriched
- Toggle always enabled, even with no analysis data — enriched view with no data shows raw graph unchanged (graceful fallback, no error)
- Instant transition — colors and sizes snap immediately, no animation

**Hover Tooltip Design**
- Tooltips appear in enriched view only — raw view stays as-is
- Tooltip content: paper title, paper type badge, top 5 TF-IDF keywords, primary method (name + category)
- Papers with no analysis data show: paper title + "Not analyzed" — consistent hover behavior across all nodes
- Uses egui's tooltip system (show_tooltip_at_pointer)

### Claude's Discretion
- Exact muted color hex values (within the muted academic constraint)
- Edge tint opacity/blending approach
- Tooltip layout and typography details
- Legend visual design within the Analysis section
- How to pass analysis data into the graph (custom node type vs lookup map vs payload approach)
- Force simulation parameter adjustments if needed for variable node sizes

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| VIS-01 | Citation graph nodes colored/sized by extracted analysis dimensions (paper type, primary method, finding strength) | Node::set_color() + DefaultNodeShape radius via custom DisplayNode; lookup maps from DB |
| VIS-02 | User can toggle between raw citation view and analysis-enriched view | Boolean flag in SettingsAnalysis; conditional set_color/radius calls in update() loop |
</phase_requirements>

---

## Summary

Phase 5 enriches the existing force-directed citation graph with visual encoding of analysis results. The work is almost entirely within the egui/egui_graphs layer, with a lightweight wiring step to thread analysis data from SurrealDB into the visualization app constructor.

The egui_graphs 0.25.0 API (currently pinned in `Cargo.toml`) exposes `Node::set_color(Color32)` for direct per-node color control without requiring a custom `DisplayNode`. Node radius control is accessible via the `display_mut()` accessor on the `Node` struct, which returns a mutable reference to the `DefaultNodeShape` whose `radius: f32` field can be set. Edge tinting by source-node color is the only feature that requires a custom `DisplayEdge` implementation, because `DefaultEdgeShape` renders edges with fixed color and does not receive the source node's color.

A critical version constraint: egui_graphs 0.25.0 does NOT have hover events (those were added in v0.27.0). Hover detection for tooltips must use egui's pointer position API (`ctx.pointer_hover_pos()`) combined with checking whether the pointer is within a node's radius by iterating graph nodes — this is the established pattern for pre-0.27 hover tooltip implementations.

**Primary recommendation:** Use a `HashMap<String, VisualData>` lookup approach to pass analysis data into `DemoApp`. Apply color and size to nodes each frame in the update loop when enriched view is active. Implement custom `DisplayEdge` only for edge tinting; keep `DefaultNodeShape` for node rendering with `display_mut()` radius access.

---

## Standard Stack

### Core (already in project)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| egui_graphs | 0.25.0 | Graph widget with interaction | Already integrated; provides Node::set_color and display_mut() |
| egui | 0.31.1 | UI framework | Already integrated; provides CollapsingHeader, Color32, tooltip APIs |
| eframe | 0.31.1 | App runner | Already integrated |
| petgraph | 0.7.0 | Graph data structure | Already integrated |

### No New Dependencies Required
Phase 5 requires no new crate dependencies. All needed functionality exists in the project's current dependency set:
- Color encoding: `egui::Color32`
- Tooltip: `egui::show_tooltip_at_pointer` or `Response::on_hover_ui_at_pointer`
- Panel layout: `egui::CollapsingHeader` (existing pattern)
- Data lookup: `std::collections::HashMap`

---

## Architecture Patterns

### Recommended Project Structure Changes
```
src/visualization/
├── force_graph_app.rs   # Add: SettingsAnalysis field, analysis lookup maps, draw_section_analysis(), tooltip logic
├── settings.rs          # Add: SettingsAnalysis struct
├── drawers.rs           # Add: draw_section_analysis() function
├── mod.rs               # Unchanged
└── (no new files needed)

src/main.rs              # Add: DB queries for annotations + analyses before launch_visualization()
                         # Modify: launch_visualization() to accept VisualData maps
```

### Pattern 1: Analysis Data as Lookup Maps (Claude's Discretion — Recommended)

**What:** Pass `HashMap<String, LlmAnnotation>` and `HashMap<String, PaperAnalysis>` into `DemoApp::new()`. These are keyed by arxiv_id and used during the per-frame render loop to look up each node's visual data by its payload.

**When to use:** When graph nodes already carry `Paper` payload (via `create_graph_from_papers` which uses `StableGraph<Paper, f32>`). The `graph_without_weights` call in `main.rs` currently strips this to `()` — Phase 5 must NOT strip the paper payload, or must pass paper IDs separately.

**Critical constraint from codebase:** `main.rs` line 597 currently does:
```rust
let graph_without_weights = paper_graph.map(|_, _| (), |_, _| ());
```
This discards all paper metadata. For Phase 5, either:
- (Option A) Pass `paper_graph` directly using `Graph<Paper, f32>` — requires changing `DemoApp` type params and `DefaultGraphView` to a `GraphView<'a, Paper, f32, ...>`
- (Option B) Pass lookup maps alongside the stripped graph, using node index → arxiv_id mapping built during graph construction

**Option B is recommended** (stays within Claude's Discretion, minimal change to existing patterns): build a `HashMap<NodeIndex, String>` mapping during `DemoApp::new()` by iterating `petgraph_graph.node_weights()`, then look up annotations/analyses by the arxiv_id associated with each `NodeIndex`. The visualization app keeps the existing `Graph<(), (), Directed>` type — no type parameter changes to `DemoApp` or `DefaultGraphView`.

```rust
// In DemoApp::new(), build the index-to-id mapping:
let mut node_id_map: HashMap<NodeIndex<u32>, String> = HashMap::new();
for node_idx in petgraph_graph.node_indices() {
    let paper = &petgraph_graph[node_idx];
    node_id_map.insert(node_idx, strip_version_suffix(&paper.id));
}
```

Then during the enriched view update:
```rust
// For each node in self.g.g.node_weights_mut():
if let Some(arxiv_id) = self.node_id_map.get(&node.id()) {
    if let Some(annotation) = self.annotations.get(arxiv_id) {
        let color = paper_type_color(&annotation.paper_type);
        node.set_color(color);
        // radius via display_mut()
    }
}
```

### Pattern 2: Per-Node Color via Node::set_color()

**What:** Call `node.set_color(Color32::from_rgb(r, g, b))` each frame when enriched view is active. Call with neutral gray or reset when toggling back to raw view.

**Verified in egui_graphs 0.25.0 API:** `Node::set_color(&mut self, color: Color32)` is a direct method — no custom `DisplayNode` needed.

**Reset to default:** When toggling back to raw view, iterate all nodes and call `node.set_color(Color32::from_gray(128))` or store the original `DefaultNodeShape` color. The simplest approach: set a specific "raw view" neutral color on toggle-off and apply the analysis color on toggle-on, each frame.

### Pattern 3: Per-Node Radius via display_mut()

**What:** `DefaultNodeShape` has a `radius: f32` field. It is accessible via `node.display_mut().radius = value`.

**Base size:** The `DefaultNodeShape` default radius is typically 5.0 or 8.0 (verify at runtime). Multipliers from CONTEXT.md:
- strong_evidence → 3x base
- moderate_evidence → 2x base
- weak_evidence → 1.5x base
- no analysis data → 1x base (raw behavior)

**Implementation note:** Apply radius changes in the same frame loop as color changes. Reset to base radius on toggle-off.

### Pattern 4: Edge Tinting via Custom DisplayEdge

**What:** `DefaultEdgeShape` does not expose source-node color. To tint edges by their source paper's type, implement a `struct TintedEdgeShape` that holds an `Option<Color32>` tint and renders with that color when set.

**When to use:** Only when enriched view is active and source node has annotation data.

**Minimal custom DisplayEdge:**
```rust
#[derive(Clone)]
pub struct TintedEdgeShape {
    // mirrors DefaultEdgeShape fields
    pub width: f32,
    pub tip_size: f32,
    pub tip_angle: f32,
    pub curve_size: f32,
    pub loop_size: f32,
    // enrichment fields
    pub color_override: Option<Color32>,
    // ... other NodeProps-derived fields
}
```

**Alternative lower-effort approach:** Skip custom `DisplayEdge` initially and use a single tint for all edges when enriched view is active (e.g., use `SettingsStyle` to apply a global edge color). This avoids implementing `DisplayEdge` entirely. Given that per-edge-type tinting requires source-node lookup, the per-edge approach requires iterating edges and finding their source node's annotation during the update loop.

**CONTEXT says:** "Edges tinted by source node's paper type color" — this IS a locked decision; custom `DisplayEdge` or equivalent per-edge color setting is required.

**Verify in egui_graphs 0.25.0:** Check whether `Edge::set_color()` exists analogous to `Node::set_color()`. If it does, no custom `DisplayEdge` is needed — same pattern as node coloring.

### Pattern 5: Tooltip via Pointer Position

**What:** egui_graphs 0.25.0 has no `NodeHover` event (added in 0.27.0). Hover detection requires:
1. Get cursor position: `ctx.pointer_hover_pos()`
2. Transform to graph coordinate space (accounting for pan/zoom)
3. Iterate nodes, check if cursor is within `node.display().radius` of `node.location()`
4. If inside a node, call `egui::show_tooltip_at_pointer(ctx, ui.layer_id(), id, |ui| { ... })`

**Coordinate transform:** The graph viewport applies pan and zoom. The `GraphView` widget occupies `egui::CentralPanel`. The pointer position from `ctx.pointer_hover_pos()` is in screen space. The node's `location()` is in graph space. The zoom/pan values tracked in `DemoApp` (`self.zoom`, `self.pan`) define the transform:
```
screen_pos = graph_pos * zoom + pan_offset
graph_pos = (screen_pos - pan_offset) / zoom
```

**Note:** `self.zoom` and `self.pan` in the existing `DemoApp` are populated from `Event::Pan` and `Event::Zoom` events from egui_graphs. Verify their units match what GraphView applies to positions before relying on this transform. If pan/zoom tracking doesn't produce accurate inverse transforms, an alternative is to check distance between `ctx.pointer_hover_pos()` and each node's **screen-space position** by tracking where nodes were rendered. The simpler fallback: show tooltip based on `Event::NodeClick` rather than hover — but this contradicts the locked tooltip-on-hover decision.

**The tooltip call:**
```rust
// Source: egui 0.31 docs - show_tooltip_at_pointer is a free function
egui::show_tooltip_at_pointer(ctx, ui.layer_id(), egui::Id::new("node_tooltip"), |ui| {
    ui.label(&paper_title);
    ui.label(paper_type_badge);
    for keyword in top_5_keywords { ui.label(keyword); }
    ui.label(format!("Method: {} ({})", method.name, method.category));
});
```

### Anti-Patterns to Avoid

- **Don't rebuild the egui_graphs Graph from scratch on every frame.** Color and radius are set on the existing `Graph` nodes in-place. Rebuilding would reset layout positions.
- **Don't use `graph_without_weights` stripping if keeping paper metadata.** The strip in `main.rs` discards all paper data — use the NodeIndex→arxiv_id map approach instead.
- **Don't implement a full custom `DisplayNode` for coloring alone.** `Node::set_color()` is sufficient. Custom `DisplayNode` is a larger API surface and adds compile complexity.
- **Don't assume `self.zoom` gives accurate coordinate transforms without verification.** The zoom value from events may be a delta or absolute — test against node positions before implementing tooltip hit detection.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Color representation | Custom color type | `egui::Color32::from_rgb(r, g, b)` | Already in scope; widely used in codebase |
| Node color setting | Custom rendering pipeline | `node.set_color(color)` | Direct Node API in egui_graphs 0.25 |
| Collapsible panel section | Custom accordion widget | `CollapsingHeader::new(...).show(...)` | Existing pattern in force_graph_app.rs |
| Tooltip display | Custom overlay widget | `egui::show_tooltip_at_pointer` | egui standard API |
| Analysis data lookup | Graph node metadata fields | `HashMap<String, LlmAnnotation>` | Simple, no type param changes |

---

## Common Pitfalls

### Pitfall 1: egui_graphs Hover Events Not Available in 0.25.0
**What goes wrong:** Developer assumes egui_graphs has a `NodeHover` event (as in v0.27+), writes event handler that never fires.
**Why it happens:** Release notes for v0.27.0 document hover support; the project is pinned at 0.25.0.
**How to avoid:** Use pointer position + distance check (Pattern 5). Do NOT upgrade egui_graphs — it would require egui version bump too.
**Warning signs:** No `NodeHover` variant in the `Event` enum at compile time.

### Pitfall 2: Coordinate Space Mismatch for Tooltip Hit Testing
**What goes wrong:** Tooltip either appears for the wrong node or never triggers because screen-space and graph-space coordinates don't match.
**Why it happens:** The pan/zoom event values may be in different units than node positions.
**How to avoid:** Write a test: add a print or log when hovering the center of the screen — verify the computed graph coordinate matches a node's location. Consider using `Node::location()` transformed by the CentralPanel rect origin rather than `self.pan`.
**Warning signs:** Tooltip shows for wrong node, or only shows when cursor is far from the visual node position.

### Pitfall 3: Node Radius Reset on Each Frame
**What goes wrong:** Node radius reverts to default because `DefaultNodeShape` is recreated from `NodeProps` each frame.
**Why it happens:** The `update()` method on `DisplayNode` is called every frame; if radius is not persisted in the shape state, it resets.
**How to avoid:** Set radius via `node.display_mut().radius` and verify it persists across frames. If it doesn't persist, consider storing enriched visual properties in a parallel `HashMap<NodeIndex, EnrichedProps>` and applying them fresh each frame unconditionally.
**Warning signs:** Node sizes flash or revert to default during simulation frames.

### Pitfall 4: Edge Color API May Not Exist in 0.25.0
**What goes wrong:** Assume `Edge::set_color()` exists like `Node::set_color()`, write code that doesn't compile.
**Why it happens:** Node and Edge APIs are not symmetric in all egui_graphs versions.
**How to avoid:** Verify `Edge::set_color()` exists at compile time before relying on it. If not present, implement minimal custom `DisplayEdge`.
**Warning signs:** Compile error on `edge.set_color(...)`.

### Pitfall 5: Analysis Data Not Available Without --db Flag
**What goes wrong:** Visualization launches without analysis data because no database was specified; app panics or shows error.
**Why it happens:** Analysis data lives in SurrealDB; if `--db` is not passed, there's no DB connection, so no annotations to load.
**How to avoid:** Treat missing analysis data as the normal case. When `db` is `None`, pass empty `HashMap`s into `DemoApp`. The graceful fallback (enriched view shows raw graph) handles this per the locked decision.
**Warning signs:** Panic on `db.unwrap()` in `launch_visualization`.

### Pitfall 6: Type Parameter Explosion if Switching to Typed Graph
**What goes wrong:** Switching `DemoApp` to use `Graph<Paper, f32>` instead of `Graph<(), ()>` requires changing `DefaultGraphView` type alias to a full `GraphView<..., Paper, f32, ...>` which cascades through all method signatures.
**Why it happens:** `DefaultGraphView` is hardcoded to unit types — it becomes unusable with non-unit data.
**How to avoid:** Use the lookup map approach (Option B in Pattern 1) to avoid touching the Graph type parameters.
**Warning signs:** Compiler complains about `DefaultGraphView` not accepting `Paper` nodes.

---

## Code Examples

### Set Node Color (Verified via egui_graphs 0.25.0 docs.rs API)
```rust
// Source: docs.rs/egui_graphs/0.25.0 - Node::set_color exists
self.g.g.node_weights_mut().for_each(|node| {
    let arxiv_id = self.node_id_map.get(&node.id()).cloned().unwrap_or_default();
    if self.settings_analysis.enriched_view {
        let color = self.annotations.get(&arxiv_id)
            .map(|ann| paper_type_to_color(&ann.paper_type))
            .unwrap_or(GRAY_UNANALYZED);
        node.set_color(color);
        // Radius via display_mut — verify field name at compile time
        node.display_mut().radius = self.annotations.get(&arxiv_id)
            .map(|ann| finding_strength_radius(&ann.findings, BASE_RADIUS))
            .unwrap_or(BASE_RADIUS);
    } else {
        node.set_color(DEFAULT_NODE_COLOR);
        node.display_mut().radius = BASE_RADIUS;
    }
});
```

### Color Mapping Function
```rust
// Source: decided palette from CONTEXT.md; hex values are Claude's Discretion
fn paper_type_to_color(paper_type: &str) -> Color32 {
    match paper_type.to_lowercase().as_str() {
        "theoretical"   => Color32::from_rgb(100, 140, 200), // muted blue
        "experimental"  => Color32::from_rgb( 90, 170, 110), // muted green
        "review"        => Color32::from_rgb(200, 160,  70), // muted amber
        "computational" => Color32::from_rgb(150, 100, 190), // muted purple
        _               => Color32::from_gray(140),           // neutral gray
    }
}
```

### Finding Strength to Radius
```rust
// Source: CONTEXT.md strength mapping — 1.5x / 2x / 3x base
fn finding_strength_radius(findings: &[Finding], base: f32) -> f32 {
    let max_multiplier = findings.iter().map(|f| match f.strength.as_str() {
        "strong_evidence"   => 3.0_f32,
        "moderate_evidence" => 2.0_f32,
        "weak_evidence"     => 1.5_f32,
        _                   => 1.0_f32,
    }).fold(1.0_f32, f32::max);
    base * max_multiplier
}
```

### Tooltip in Enriched View
```rust
// Source: egui 0.31 docs - show_tooltip_at_pointer is a free function in egui
// Called in update() after rendering the graph, only in enriched view
if self.settings_analysis.enriched_view {
    if let Some(hovered_id) = self.find_hovered_node(ctx) {
        egui::show_tooltip_at_pointer(
            ctx,
            ui.layer_id(),
            egui::Id::new("node_tooltip"),
            |ui| {
                if let Some(ann) = self.annotations.get(&hovered_id) {
                    ui.strong(&ann.arxiv_id); // or paper title from separate map
                    ui.label(format!("Type: {}", ann.paper_type));
                    // TF-IDF keywords from analyses map
                    if let Some(analysis) = self.analyses.get(&hovered_id) {
                        ui.label(format!("Keywords: {}", analysis.top_terms[..5.min(analysis.top_terms.len())].join(", ")));
                    }
                    if let Some(method) = ann.methods.first() {
                        ui.label(format!("Method: {} ({})", method.name, method.category));
                    }
                } else {
                    ui.label(&hovered_id);
                    ui.label("Not analyzed");
                }
            }
        );
    }
}
```

### SettingsAnalysis (new settings struct)
```rust
// Source: settings.rs pattern — follows existing SettingsInteraction pattern
#[derive(Default)]
pub struct SettingsAnalysis {
    pub enriched_view: bool,
}
```

### CollapsingHeader for Analysis Section (follows existing pattern)
```rust
// Source: force_graph_app.rs - existing pattern for Simulation/Debug/Widget sections
CollapsingHeader::new("Analysis")
    .default_open(true)
    .show(ui, |ui| self.draw_section_analysis(ui));
```

### DemoApp constructor signature change
```rust
// main.rs: build lookup maps before launch
fn launch_visualization(
    papers: &[Paper],
    annotations: HashMap<String, LlmAnnotation>,
    analyses: HashMap<String, PaperAnalysis>,
) {
    let paper_graph = create_graph_from_papers(papers);
    // Build NodeIndex -> arxiv_id map BEFORE stripping weights
    let node_id_map: HashMap<NodeIndex<u32>, String> = paper_graph
        .node_indices()
        .map(|idx| (idx, strip_version_suffix(&paper_graph[idx].id)))
        .collect();
    let graph_without_weights = paper_graph.map(|_, _| (), |_, _| ());
    // Pass all to DemoApp::new()
    run_native(..., Box::new(|cc| Ok(Box::new(
        DemoApp::new(cc, graph_without_weights, node_id_map, annotations, analyses)
    ))));
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Manual hover tracking | egui_graphs hover events | v0.27.0 | Project at 0.25.0 — must use manual tracking |
| `Graph<(), ()>` with `DefaultGraphView` | `Graph<N, E>` with `GraphView` type params | v0.20+ | Using typed graph requires abandoning `DefaultGraphView` alias |
| Uniform node appearance | `Node::set_color()` + `display_mut().radius` | v0.20+ | Direct per-node visual control, no custom renderer needed for color |

**Deprecated/outdated:**
- `DefaultGraphView` (type alias): Still valid but locks node/edge data to `()` — use `GraphView<...>` if carrying typed data.

---

## Open Questions

1. **Does `Edge::set_color()` exist in egui_graphs 0.25.0?**
   - What we know: `Node::set_color()` is confirmed. Edge and Node APIs were designed in parallel.
   - What's unclear: Whether `Edge` struct has `set_color()` or requires custom `DisplayEdge` for tinting.
   - Recommendation: Check at compile time in Wave 0. If `Edge::set_color()` exists, use it directly (same pattern as node coloring). If not, implement minimal `TintedEdgeShape` as a `DisplayEdge` implementor.

2. **Does `node.display_mut().radius` persist across frames?**
   - What we know: `DefaultNodeShape` has a `radius: f32` field, accessible via `display_mut()`.
   - What's unclear: Whether `update()` resets the shape from `NodeProps` each frame, discarding manual changes.
   - Recommendation: Test in Wave 0 by setting a fixed radius on one node and verifying it persists. If it resets, maintain a `HashMap<NodeIndex, f32>` of desired radii and re-apply each frame — already acceptable since we're iterating all nodes anyway.

3. **Accurate coordinate transform for tooltip hit testing**
   - What we know: `self.pan` and `self.zoom` track pan/zoom from egui_graphs events. Node locations are in graph space.
   - What's unclear: Whether the pan/zoom values match what GraphView applies to map graph→screen coordinates.
   - Recommendation: In Wave 0, log `node.location()` for a known node and `ctx.pointer_hover_pos()` at the same position, then derive the transform empirically before committing to the formula.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in (`cargo test`) |
| Config file | none (Cargo.toml test configuration) |
| Quick run command | `cargo test visualization` |
| Full suite command | `cargo test` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| VIS-01 | paper_type_to_color maps all paper types correctly | unit | `cargo test visualization::tests::test_paper_type_to_color` | ❌ Wave 0 |
| VIS-01 | finding_strength_radius returns correct multipliers | unit | `cargo test visualization::tests::test_finding_strength_radius` | ❌ Wave 0 |
| VIS-01 | no-analysis-data nodes get neutral gray and base radius | unit | `cargo test visualization::tests::test_unenriched_node_defaults` | ❌ Wave 0 |
| VIS-02 | enriched view toggle defaults to false | unit | `cargo test visualization::tests::test_settings_analysis_default` | ❌ Wave 0 |
| VIS-02 | enriched view with empty annotation map does not panic | unit | `cargo test visualization::tests::test_enriched_view_empty_maps` | ❌ Wave 0 |

**Note:** GUI rendering (actual color appearance, tooltip display) cannot be unit tested without egui test harness. The above tests cover the pure logic functions: color mapping, radius calculation, settings defaults, and graceful fallback. Visual correctness is verified manually at Phase verification time.

### Sampling Rate
- **Per task commit:** `cargo test visualization`
- **Per wave merge:** `cargo test`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src/visualization/tests.rs` (or inline `#[cfg(test)]` module in `force_graph_app.rs`) — covers VIS-01 logic tests
- [ ] No framework install needed — `cargo test` already works

---

## Sources

### Primary (HIGH confidence)
- docs.rs/egui_graphs/0.25.0 — `Node::set_color`, `DefaultNodeShape` fields (`radius`, `color`), `DisplayNode` trait, `DefaultGraphView` type alias, `GraphView` type parameters
- docs.rs/egui_graphs/0.25.0/struct.Node — confirmed `set_color(&mut self, color: Color32)`, `display_mut()`, `id()`, `location()`
- Cargo.toml in project — confirmed egui_graphs = "0.25.0", egui = "0.31.1"
- `src/visualization/force_graph_app.rs` — confirmed existing Graph<(), (), Directed, DefaultIx> type, `graph_without_weights` strip, event handling pattern
- `src/datamodels/llm_annotation.rs` — confirmed `LlmAnnotation` fields: `paper_type: String`, `methods: Vec<Method>`, `findings: Vec<Finding>`
- `src/datamodels/analysis.rs` — confirmed `PaperAnalysis` fields: `top_terms: Vec<String>`, `top_scores: Vec<f32>`

### Secondary (MEDIUM confidence)
- github.com/blitzarx1/egui_graphs/releases — confirmed hover events added in v0.27.0 (not in 0.25.0)
- docs.rs/egui_graphs/0.25.0 — `DefaultGraphView` type alias definition (`GraphView<'a, (), (), Directed, DefaultIx, DefaultNodeShape, DefaultEdgeShape, State, Random>`)
- egui 0.31 docs search — `show_tooltip_at_pointer`, `on_hover_ui_at_pointer` are the egui tooltip APIs

### Tertiary (LOW confidence)
- Search results re: `Edge::set_color()` — NOT verified in 0.25.0 docs; must confirm at compile time

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all dependencies already in project, versions confirmed from Cargo.toml
- Architecture: HIGH — egui_graphs 0.25.0 Node API verified from docs.rs; lookup map pattern derived from existing codebase
- Pitfalls: HIGH — hover event gap confirmed from release notes; coordinate transform warning is a known pattern issue
- Edge tinting API: LOW — `Edge::set_color()` existence in 0.25.0 not confirmed; treat as open question

**Research date:** 2026-03-14
**Valid until:** 2026-04-14 (egui_graphs 0.25.0 is a pinned version; stable until upgraded)
