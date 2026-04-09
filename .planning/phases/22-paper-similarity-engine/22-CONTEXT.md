# Phase 22: Paper Similarity Engine - Context

**Gathered:** 2026-04-09
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can see which papers are most similar to any given paper, with similarity edges optionally shown on the graph. The system computes top-10 cosine similarity from existing TF-IDF vectors, stores results in SurrealDB, and provides a "Similar Papers" drawer tab plus a toggleable similarity edge overlay with its own force model. This phase does NOT add centrality metrics, community detection, recommendations, or export — those are separate phases (23-26).

</domain>

<decisions>
## Implementation Decisions

### Similar Papers tab presentation
- **D-01:** Ranked list showing similarity score (percentage), title, authors, year, plus 2-3 shared keywords explaining WHY papers are similar (using existing `shared_high_weight_terms()`)
- **D-02:** Clicking a similar paper navigates to it (opens its detail drawer)
- **D-03:** New `DrawerTab::Similar` variant added to existing `DrawerTab` enum (currently has Overview, Source)

### Similarity edge overlay visuals
- **D-04:** Dashed lines in warm color (orange/amber) to clearly distinguish from solid gray citation edges
- **D-05:** Edge thickness scales with similarity score (higher similarity = thicker)
- **D-06:** Fixed minimum similarity threshold for displaying edges (Claude picks sensible default, e.g. top-5 neighbors or score > 0.3) — no user-adjustable slider for v1.4

### Computation trigger & freshness
- **D-07:** Silent background recompute — similarity updates automatically after TF-IDF analysis completes, no toast or progress bar
- **D-08:** When TF-IDF vectors don't exist yet, the "Similar Papers" tab shows a spinner with "Waiting for TF-IDF analysis..." message
- **D-09:** Once vectors exist, similarity data is just available — no freshness indicators needed

### Graph interaction with similarity mode
- **D-10:** Dual-layer toggle — citation edges and similarity edges can be independently shown/hidden via graph controls
- **D-11:** Two distinct force models: citation topology (default) vs content similarity. Toggling to similarity-only mode swaps the force simulation so similar papers cluster together
- **D-12:** When both edge types are visible, citation forces drive layout (similarity edges are visual overlay only)
- **D-13:** Switching force models triggers a re-simulation with visible animation

### Claude's Discretion
- Exact amber/orange color value for similarity edges
- Dash pattern (length, gap) for similarity edges
- Fixed similarity threshold value for edge display
- Spinner styling and "waiting for TF-IDF" message copy
- Similar papers list item layout details
- Force simulation parameters for similarity-based layout

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` — SIM-01 through SIM-04 define similarity acceptance criteria
- `.planning/ROADMAP.md` §Phase 22 — Success criteria and phase boundary

### Existing similarity code
- `resyn-core/src/gap_analysis/similarity.rs` — `cosine_similarity()` and `shared_high_weight_terms()` already implemented with tests
- `resyn-core/src/nlp/tfidf.rs` — TF-IDF computation with section weighting, corpus fingerprint
- `resyn-core/src/datamodels/analysis.rs` — `PaperAnalysis` struct with `tfidf_vector: HashMap<String, f32>`, `corpus_fingerprint`

### Drawer and graph integration points
- `resyn-app/src/app.rs:16` — `DrawerTab` enum (Overview, Source) — add Similar variant here
- `resyn-app/src/layout/drawer.rs` — Drawer component with tab system and `initial_tab` prop
- `resyn-app/src/components/graph_controls.rs` — Graph controls overlay (add similarity toggle here)
- `resyn-app/src/graph/canvas_renderer.rs` — Canvas 2D renderer (add similarity edge rendering)
- `resyn-app/src/graph/webgl_renderer.rs` — WebGL2 renderer (add similarity edge rendering)
- `resyn-app/src/server_fns/analysis.rs` — Analysis server fns (trigger point for similarity recompute)

### Prior phase patterns
- `.planning/phases/21-search-filter/21-CONTEXT.md` — Established patterns: server fn conventions, context signals, drawer integration

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `cosine_similarity()` in `gap_analysis/similarity.rs`: Core algorithm already implemented and tested
- `shared_high_weight_terms()` in same file: Returns overlapping keywords between two vectors — powers the "why similar" display
- `PaperAnalysis.tfidf_vector`: Sparse HashMap<String, f32> already stored per paper in SurrealDB
- `DrawerTab` enum: Extensible with new variants
- `SelectedPaper` context signal: Drives drawer navigation, reusable for similar-paper clicks

### Established Patterns
- Server fns with `#[server(..., "/api")]` for all DB queries
- Context signals for cross-component state (`SelectedPaper`, `DrawerOpenRequest`)
- Graph controls overlay for toggles (existing pattern for adding similarity toggle)
- Corpus fingerprint for cache invalidation (reuse for similarity cache)

### Integration Points
- `DrawerTab` enum: Add `Similar` variant
- Drawer component: Add third tab rendering similar papers list
- Graph controls: Add similarity edge toggle + citation/similarity edge visibility toggles
- Graph renderers (Canvas 2D + WebGL2): Render dashed amber edges for similarity
- Force simulation: Need second force model for similarity-based layout, switchable at runtime
- Analysis server fns: Hook similarity recompute after TF-IDF pipeline completes

</code_context>

<specifics>
## Specific Ideas

- Force model swap is a key UX feature: citation view shows "who cites whom" topology, similarity view shows "what's about the same thing" clusters — two fundamentally different lenses on the same corpus
- Shared keywords in the similar papers list give users actionable insight ("these papers both discuss quantum entanglement and decoherence") rather than just an opaque score
- Fixed threshold keeps v1.4 simple — a user-adjustable slider can be added in a future iteration if needed

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 22-paper-similarity-engine*
*Context gathered: 2026-04-09*
