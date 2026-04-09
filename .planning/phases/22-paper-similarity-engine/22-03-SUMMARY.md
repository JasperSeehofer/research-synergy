# Plan 22-03: Similarity Edge Overlay + Dual Force Model — Summary

## Status: COMPLETE

## What Was Built

### EdgeType::Similarity and Graph Data Loading
- Added `EdgeType::Similarity` variant to the graph edge enum with serde serialization as `"similarity"`
- `get_graph_data` server fn queries `SimilarityRepository` and emits similarity edges with fixed `0.15` threshold (D-06)
- Deduplication via lexical A<B ordering prevents duplicate edges

### Dashed Amber Rendering
- Canvas2D renderer draws similarity edges as dashed `#f0a030` amber lines with score-based thickness (1.5–4.0px) (D-04, D-05)
- WebGL2 mode renders similarity edges on the label canvas overlay using Canvas2D `setLineDash` (research recommendation)

### Graph Controls
- Independent "Citations" and "Similarity" toggle buttons for edge visibility (D-10)
- Force mode selector with "Citation" and "Similarity" layout options (D-11)
- When both visible, citation forces drive layout (D-12)

### Force Model Swap
- Switching force mode sets `alpha = 0.5` and restarts simulation for visible re-animation (D-13)
- `build_layout_input` selects Regular edges for citation mode, Similarity edges for similarity mode

## Self-Check: PASSED

- `cargo check --all-targets` exits 0
- All 344 tests pass
- `EdgeType::Similarity` exhaustively handled in all match sites

## Deviations

None — all decisions D-04 through D-13 implemented as specified.

## Key Files

### Created
- (no new files — modifications to existing)

### Modified
- `resyn-app/src/server_fns/graph.rs` — similarity edge loading with threshold
- `resyn-app/src/graph/layout_state.rs` — `GraphState` fields: show_similarity, show_citations, force_mode
- `resyn-app/src/graph/canvas_renderer.rs` — dashed amber rendering for similarity edges
- `resyn-app/src/graph/webgl_renderer.rs` — similarity edge rendering via label canvas
- `resyn-app/src/components/graph_controls.rs` — citation/similarity toggles and force mode selector
- `resyn-app/src/pages/graph.rs` — force model swap with alpha reheat, build_layout_input selection

## Human Verification Required

Visual verification of similarity edge overlay rendering (dashed amber lines, thickness scaling, force model swap animation) is recommended before phase completion.
