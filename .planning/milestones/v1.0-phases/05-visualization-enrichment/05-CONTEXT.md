# Phase 5: Visualization Enrichment - Context

**Gathered:** 2026-03-14
**Status:** Ready for planning

<domain>
## Phase Boundary

Enrich the existing force-directed citation graph with analysis-derived visual encoding. Nodes are colored by paper type and sized by finding strength. A toggle switches between the raw citation view (current behavior) and the analysis-enriched view. Hover tooltips show keywords and primary method. Does NOT include 3D projection (VIS-03, v2), temporal evolution (VIS-04, v2), or new analysis capabilities.

</domain>

<decisions>
## Implementation Decisions

### Color Palette & Mapping
- Muted academic palette: soft, distinguishable tones — blue for theoretical, green for experimental, amber for review, purple for computational
- Papers with no analysis data rendered in neutral gray — clearly signals "not yet analyzed"
- Edges tinted by source node's paper type color — shows how theoretical vs experimental papers cite each other
- Color legend displayed in the right panel's new Analysis section, visible when enriched view is active

### Node Sizing Strategy
- Strongest finding wins: node size = max finding strength across all findings for that paper
- Strength mapping: strong_evidence = 3x base, moderate_evidence = 2x base, weak_evidence = 1.5x base (moderate 1x to 3x range)
- Papers with no analysis data use default/medium base size — unchanged from raw view
- Size range chosen to provide clear visual hierarchy without overwhelming Fruchterman-Reingold layout

### Toggle Placement & Behavior
- New "Analysis" collapsible section in right panel, placed between Simulation and Widget sections
- Simple checkbox control: "Enriched view" — unchecked = raw, checked = enriched
- Toggle always enabled, even with no analysis data — enriched view with no data shows raw graph unchanged (graceful fallback, no error)
- Instant transition — colors and sizes snap immediately, no animation

### Hover Tooltip Design
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

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `LlmAnnotation` (`src/datamodels/llm_annotation.rs`): `paper_type` (string enum), `methods` (Vec<Method>), `findings` (Vec<Finding> with strength field) — all data needed for color, size, and tooltip
- `PaperAnalysis` (`src/datamodels/analysis.rs`): `tfidf_vector` — sparse term-score maps for tooltip keywords
- `DemoApp` (`src/visualization/force_graph_app.rs`): Main app struct — enrichment integrates here
- `settings.rs`: Pattern for adding new settings structs (SettingsAnalysis for the new section)
- `drawers.rs`: Pattern for drawing UI sections — add analysis section drawer

### Established Patterns
- `Graph<(), (), Directed>` — current graph uses unit types; enrichment requires either custom node/edge drawers or switching to typed nodes with color/size payload
- `DefaultGraphView` — egui_graphs default renderer; may need replacement with custom view for per-node coloring/sizing
- Right panel layout: `CollapsingHeader` sections (Simulation, Debug, Widget) — Analysis section follows same pattern
- `egui_graphs::SettingsStyle` — has `labels_always` but no per-node color/size API; custom drawing likely needed

### Integration Points
- `src/visualization/force_graph_app.rs`: DemoApp needs analysis data (annotations + TF-IDF) passed in alongside the graph
- `src/visualization/force_graph_app.rs`: New `draw_section_analysis()` method for the Analysis panel section
- `src/data_processing/graph_creation.rs`: May need to carry paper metadata through to graph nodes (currently strips to `()`)
- `src/main.rs`: Pass analysis data from DB queries into visualization app constructor

</code_context>

<specifics>
## Specific Ideas

- Edge tinting by source paper type was chosen to visualize how different paper categories cite each other — a subtle but informative layer
- "Not analyzed" tooltip text for unanalyzed papers keeps hover behavior consistent — every node responds in enriched mode
- Legend in the right panel ensures the color encoding is always discoverable without hovering

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 05-visualization-enrichment*
*Context gathered: 2026-03-14*
