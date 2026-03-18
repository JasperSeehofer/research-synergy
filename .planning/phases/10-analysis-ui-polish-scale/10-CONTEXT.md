# Phase 10: Analysis UI Polish + Scale - Context

**Gathered:** 2026-03-18
**Status:** Ready for planning

<domain>
## Phase Boundary

Complete the analysis surface with provenance tracking (click a finding, see source text), improve LLM extraction quality by feeding detected sections instead of just abstracts, and verify/polish the app at 1000+ node scale with semantic zoom LOD and temporal filtering. No new analysis types or data sources.

</domain>

<decisions>
## Implementation Decisions

### Provenance display
- Reuse existing paper side drawer with a new "Source" tab showing section text with highlighted passages
- Clicking a gap finding opens Paper A's drawer with relevant passage highlighted; toggle/tab to switch to Paper B
- For abstract-only papers, show abstract with best-effort fuzzy match highlighting, labeled "Abstract only — full text unavailable"
- Provenance is one-paper-at-a-time in the drawer, not side-by-side split view

### Provenance data model
- LlmAnnotation findings gain `source_section` (e.g., "results") and `source_snippet` (verbatim quote ~1-2 sentences) fields
- LLM prompted to return source snippets alongside each finding/method/open_problem
- Fuzzy-match the snippet against stored section text for highlighting in the drawer
- No byte offsets — section name + snippet is sufficient

### Section-aware LLM extraction
- Send all available sections in a single structured prompt with section headers (abstract, methods, results, conclusion)
- LLM sees full paper context and references specific sections in its output
- Abstract-only papers use the same prompt structure, just fewer sections filled — no separate code path
- Re-running section-aware analysis overwrites the old abstract-only annotation (no version history)
- One unified prompt change covers both DEBT-04 (section-aware extraction) and AUI-04 (provenance tracking)

### Semantic zoom LOD
- At low zoom: only show high-importance nodes (high citation count + close BFS depth from seed)
- Progressive reveal as you zoom in: seed paper and direct refs always visible, then high-citation, then depth-2, then medium-citation, then everything
- Hidden nodes' edges still render as faint traces (very low opacity) to preserve topology awareness
- Visible/hidden transitions are smooth (opacity fade, not pop-in)
- Node count indicator in controls overlay: "Showing 47 of 1,203 nodes"

### Temporal filtering
- Dual-handle range slider below the graph canvas for min/max year
- Papers outside selected range dimmed to ~10% opacity (not hidden) — preserves graph structure
- Consistent with existing neighbor-dimming pattern from Phase 9 node selection
- Temporal filter is graph-page only — analysis panels (gaps, open problems, methods) always show full corpus
- Real-time update as slider handles are dragged

### Scale testing
- Real test runs at depth 2, 3, 5 with performance profiling
- Verify LOD and temporal filter work correctly at 1000+ nodes
- Profile WebGL2 renderer frame rate with full-scale graphs

### Claude's Discretion
- Exact LOD visibility thresholds (citation count cutoffs, zoom level breakpoints)
- Fuzzy matching algorithm for snippet-to-section-text highlighting
- Slider component implementation details (CSS, range input handling)
- LLM prompt wording and JSON schema for section-aware extraction
- Force layout parameter adjustments for large graphs
- DB migration details for new LlmAnnotation fields

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Provenance & annotation data model
- `resyn-core/src/datamodels/llm_annotation.rs` — LlmAnnotation, Finding, Method structs — need source_section and source_snippet fields added
- `resyn-core/src/datamodels/gap_finding.rs` — GapFinding with paper_ids, justification, confidence — provenance traces through these to papers
- `resyn-core/src/datamodels/extraction.rs` — TextExtractionResult, SectionMap with section text storage

### LLM extraction pipeline
- `resyn-core/src/llm/prompt.rs` — Current system prompt (abstract-only) — needs section-aware rewrite
- `resyn-core/src/llm/gap_prompt.rs` — Contradiction and ABC-bridge prompts
- `resyn-core/src/data_aggregation/text_extractor.rs` — Ar5ivExtractor with section detection (normalize_section_title, section_category)

### Graph rendering & scale
- `resyn-app/src/graph/renderer.rs` — Renderer trait, make_renderer(), WEBGL_THRESHOLD — LOD integrates here
- `resyn-app/src/graph/webgl_renderer.rs` — WebGL2 shaders, instanced rendering — needs LOD-aware draw calls
- `resyn-app/src/graph/canvas_renderer.rs` — Canvas2D renderer — needs LOD-aware draw calls
- `resyn-app/src/graph/layout_state.rs` — NodeState (has year field), GraphState — add visibility/LOD state
- `resyn-worker/src/forces.rs` — Barnes-Hut force layout constants

### UI components to extend
- `resyn-app/src/layout/drawer.rs` — Paper side drawer — add Source tab with section text + highlights
- `resyn-app/src/components/gap_card.rs` — Gap finding card — add click-to-provenance interaction
- `resyn-app/src/components/graph_controls.rs` — Graph overlay controls — add node count indicator
- `resyn-app/src/pages/graph.rs` — Graph page — add temporal slider and LOD integration
- `resyn-app/src/pages/gaps.rs` — Gap findings panel — wire click-to-drawer provenance

### Server functions
- `resyn-app/src/server_fns/papers.rs` — get_paper_detail() — needs to include TextExtractionResult for provenance
- `resyn-app/src/server_fns/graph.rs` — get_graph_data() — graph data endpoint
- `resyn-app/src/server_fns/gaps.rs` — get_gap_findings() — gap findings endpoint

### Database schema
- `resyn-core/src/database/schema.rs` — text_extraction table, llm_annotation table — may need migration for new fields

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `TextExtractionResult` with `SectionMap`: already stores abstract, introduction, methods, results, conclusion text per paper — directly feeds provenance display
- `Drawer` component: paper side drawer with Overview content — extend with Source tab
- `GapCard` component: expandable cards with paper ID buttons — add provenance click handler
- `graph_controls` component: overlay with toggle buttons — add node count indicator and temporal slider
- `NodeState.year` field: already extracted from `paper.published` — directly feeds temporal filter
- `radius_from_citations()`: citation-based node sizing — same importance metric feeds LOD visibility

### Established Patterns
- Dark minimal theme with separate CSS files (Phase 8)
- Sidebar + content layout with collapsible rail (Phase 8)
- Neighbor-dimming on node selection via opacity (Phase 9) — same pattern for temporal filter dimming
- Toggle filter buttons (contradictions/bridges) in gap panel and graph controls (Phase 8/9)
- Server functions in `resyn-app/src/server_fns/` calling resyn-core repositories (Phase 8)
- `ssr` feature gate: DB/LLM code behind ssr, WASM-safe types always available

### Integration Points
- Drawer component: add tab navigation (Overview / Source / Analysis) to existing drawer
- GapCard: wire paper ID button click to open drawer with Source tab + finding highlight context
- GraphState: add visibility flags per node for LOD, add temporal filter year range
- WebGL2/Canvas2D renderers: check node visibility before drawing (LOD + temporal)
- LLM prompt: replace abstract-only input with section-structured input
- LlmAnnotation DB table: migration for source_section, source_snippet fields on Finding

</code_context>

<specifics>
## Specific Ideas

- Provenance should feel like "click a finding, instantly see where it came from in the paper text" — minimal friction
- Faint edge traces for hidden nodes preserve the sense of graph density even when zoomed out
- The node count indicator ("Showing 47 of 1,203") gives researchers confidence they're seeing the right level of detail
- Temporal dimming (not hiding) preserves graph layout stability — no jarring layout shifts when sliding the year range
- Section-aware extraction and provenance tracking are two sides of the same coin — the LLM outputs source references, the UI displays them

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 10-analysis-ui-polish-scale*
*Context gathered: 2026-03-18*
