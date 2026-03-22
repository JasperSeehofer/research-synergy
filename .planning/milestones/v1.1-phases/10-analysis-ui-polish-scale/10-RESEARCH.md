# Phase 10: Analysis UI Polish + Scale — Research

**Researched:** 2026-03-18
**Domain:** Leptos UI extension, WebGL2 LOD, SurrealDB schema migration, LLM prompt engineering, performance benchmarking
**Confidence:** HIGH (all claims grounded in codebase inspection and established project patterns)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Provenance display**
- Reuse existing paper side drawer with a new "Source" tab showing section text with highlighted passages
- Clicking a gap finding opens Paper A's drawer with relevant passage highlighted; toggle/tab to switch to Paper B
- For abstract-only papers, show abstract with best-effort fuzzy match highlighting, labeled "Abstract only — full text unavailable"
- Provenance is one-paper-at-a-time in the drawer, not side-by-side split view

**Provenance data model**
- LlmAnnotation findings gain `source_section` (e.g., "results") and `source_snippet` (verbatim quote ~1-2 sentences) fields
- LLM prompted to return source snippets alongside each finding/method/open_problem
- Fuzzy-match the snippet against stored section text for highlighting in the drawer
- No byte offsets — section name + snippet is sufficient

**Section-aware LLM extraction**
- Send all available sections in a single structured prompt with section headers (abstract, methods, results, conclusion)
- LLM sees full paper context and references specific sections in its output
- Abstract-only papers use the same prompt structure, just fewer sections filled — no separate code path
- Re-running section-aware analysis overwrites the old abstract-only annotation (no version history)
- One unified prompt change covers both DEBT-04 (section-aware extraction) and AUI-04 (provenance tracking)

**Semantic zoom LOD**
- At low zoom: only show high-importance nodes (high citation count + close BFS depth from seed)
- Progressive reveal as you zoom in: seed paper and direct refs always visible, then high-citation, then depth-2, then medium-citation, then everything
- Hidden nodes' edges still render as faint traces (very low opacity) to preserve topology awareness
- Visible/hidden transitions are smooth (opacity fade, not pop-in)
- Node count indicator in controls overlay: "Showing 47 of 1,203 nodes"

**Temporal filtering**
- Dual-handle range slider below the graph canvas for min/max year
- Papers outside selected range dimmed to ~10% opacity (not hidden) — preserves graph structure
- Consistent with existing neighbor-dimming pattern from Phase 9 node selection
- Temporal filter is graph-page only — analysis panels (gaps, open problems, methods) always show full corpus
- Real-time update as slider handles are dragged

**Scale testing**
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

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| AUI-04 | Analysis provenance tracking — click a finding, see source text segment | Drawer tab extension pattern; Finding struct gains source_section + source_snippet; fuzzy match against SectionMap text |
| DEBT-04 | Section-aware LLM extraction using detected section boundaries | SYSTEM_PROMPT rewrite; LLM_ANNOTATION_SCHEMA update; SectionMap already holds all section text |
| SCALE-01 | Real test runs at depth 2, 3, 5 with performance profiling | Existing crawl CLI + --db flag; web_sys::performance for frame timing; no new infrastructure needed |
| SCALE-02 | Node clustering / level-of-detail for 1000+ node graphs | GraphState gains per-node lod_visible flag; renderers check flag before drawing; Viewport.scale drives thresholds |
| SCALE-03 | Temporal filtering by publication year | NodeState.year already extracted; GraphState gains temporal_range (u32, u32); renderers apply opacity; HTML range input slider |
</phase_requirements>

---

## Summary

Phase 10 is a pure polish and integration pass — no new data sources, no new LLM providers, no new infrastructure. Everything connects to existing code already in place from Phases 6–9. The work divides cleanly into four independent tracks: (1) data model + LLM prompt evolution, (2) provenance drawer UI, (3) graph-page LOD and temporal controls, and (4) scale testing.

The key insight across all tracks is that the infrastructure is ready: `SectionMap` already stores multi-section text, `NodeState.year` already extracts publication year, `WebGL2Renderer` already applies per-node alpha for neighbor dimming, and the `Drawer` component already fetches `PaperDetail`. The planner should treat each track as incremental wiring rather than new construction.

The only genuinely novel engineering is (a) the dual-handle range slider — which needs an HTML `<input type="range">` pair or a custom CSS-driven implementation since there is no UI component library in this stack — and (b) the fuzzy snippet matching, which should use a simple Levenshtein-distance or substring search rather than a heavy library.

**Primary recommendation:** Plan in four sequential plans — data model first (unblocks everything else), then LLM prompt + server, then renderer LOD + temporal filter, then scale profiling.

---

## Standard Stack

### Core (already in project — no new dependencies needed)
| Library | Version | Purpose | Notes |
|---------|---------|---------|-------|
| leptos | 0.8 | Reactive WASM UI | `Callback.run()`, `RwSignal`, `Resource`, `Effect` |
| web-sys | * | DOM APIs (range input, performance timing) | `web_sys::HtmlInputElement` for sliders |
| surrealdb | 3.x | Schema migration for new LlmAnnotation fields | `DEFINE FIELD IF NOT EXISTS` DDL pattern |
| serde_json | * | `findings` stored as JSON string in DB (established pattern) | Serialize `Vec<Finding>` to string |

### No New Dependencies
All work uses the existing project dependency set. The fuzzy matching logic (substring search or simple Levenshtein) is implemented inline in Rust — no external crate needed for ~1-2 sentence snippets.

---

## Architecture Patterns

### Established Pattern: JSON-String Fields in SurrealDB SCHEMAFULL
The `llm_annotation` table stores `methods` and `findings` as `TYPE string` containing JSON (see migration 5 and `LlmAnnotationRecord`). Adding `source_section` and `source_snippet` to `Finding` means updating:
1. `Finding` struct in `llm_annotation.rs` — add `source_section: Option<String>` and `source_snippet: Option<String>`
2. `LLM_ANNOTATION_SCHEMA` in `prompt.rs` — add fields to JSON schema
3. `SYSTEM_PROMPT` in `prompt.rs` — instruct LLM to populate fields
4. `LlmAnnotationRecord` / `to_annotation()` in `queries.rs` — no change needed (findings already stored as JSON string, new optional fields deserialize to `None` on old records)
5. DB schema migration 8 — no DDL change needed (findings field is already `TYPE string`; JSON content evolves transparently)

The `source_section` and `source_snippet` fields are `Option<String>` so old annotations without them continue to deserialize cleanly without any migration.

### Established Pattern: Per-Node Alpha in Renderer
The WebGL2 renderer already computes per-node alpha for neighbor dimming (0.5 for dimmed, 1.0 for active). The same `instance_data` vec drives the instanced draw call. LOD and temporal filter extend this by contributing an additional alpha multiplier per node. The logical flow per node becomes:

```
base_alpha = neighbor_alpha(node, selected)   // existing
lod_alpha  = if node.lod_visible { 1.0 } else { 0.05 }  // new
time_alpha = if in_temporal_range(node) { 1.0 } else { 0.10 }  // new
final_alpha = base_alpha * lod_alpha * time_alpha
```

Both renderers (WebGL2 and Canvas2D) must be updated consistently. The `GraphState` struct holds the filter state; renderers remain stateless — they re-evaluate per frame.

### Established Pattern: GraphState Holds All Render-Visible State
`GraphState` already holds `show_contradictions`, `show_bridges`, `simulation_running`, `selected_node`. New fields follow the same pattern:

```rust
// Add to GraphState:
pub temporal_min_year: u32,  // inclusive
pub temporal_max_year: u32,  // inclusive
pub lod_zoom_threshold: f64, // current viewport scale snapshot (written each RAF frame)
pub seed_paper_id: Option<String>,  // for depth-0 always-visible rule
```

For LOD, the RFC loop already reads `viewport.scale` — it should snapshot this into `GraphState` each frame so renderers don't need a `Viewport` borrow alongside the existing `GraphState` borrow.

### Established Pattern: Leptos RwSignal for UI State
The temporal slider values flow as `RwSignal<u32>` signals from `GraphPage`, passed to `GraphControls`, synced into `GraphState` in the RAF loop (same as `show_contradictions` / `show_bridges`). This avoids borrow-checker issues between the `RenderState` and Leptos reactive graph.

### Established Pattern: Drawer Tab Navigation
The existing `DrawerBody` component renders a flat body with sections (Abstract, Methods, Findings, Open Problems). Adding a "Source" tab means introducing tab state:

```rust
// In DrawerBody or DrawerContent:
let active_tab = RwSignal::new(DrawerTab::Overview);
```

The tab strip renders two buttons (Overview / Source). The Source tab body displays the `TextExtractionResult` sections with highlighted snippets.

### Pattern: Dual-Handle Range Slider (No Component Library)
This project has no UI component library — all UI is hand-rolled CSS+HTML. The standard approach for a dual-handle range slider in pure HTML/CSS/JS is two overlapping `<input type="range">` elements positioned absolutely over a shared track. Key implementation notes:
- Both inputs share the same min/max but have independent values
- z-index swapping is needed when handles cross, so the lower handle can be clicked past the upper
- The track fill (between handles) is drawn via a CSS gradient on the container background
- `on:input` events drive reactive signals in real time

Alternative: A single custom-drawn slider using Canvas2D pointer events — but this adds complexity without benefit.

**Recommended approach for this Rust/Leptos/WASM stack:** Use `web_sys::HtmlInputElement` for each handle, read `.value_as_number()` in the `on:input` closure, write to `RwSignal<u32>` signals. The Leptos `view!` macro supports standard HTML `<input type="range">` directly.

### Pattern: Fuzzy Snippet Matching
Decision is left to Claude's discretion. For ~1-2 sentence snippets matched against section text (typically a few hundred to a few thousand words):

**Recommended approach:** Normalized substring search with whitespace normalization:
1. Normalize both snippet and section text (collapse whitespace, lowercase)
2. Find the snippet as a substring of the normalized text
3. Map the byte offset back to the original text for character-level highlight bounds
4. If no exact substring match, fall back to "longest common subsequence" window search on word tokens (find the span of section text that has the most words in common with the snippet)

This stays fully in safe Rust with no external crate. The `source_snippet` is guaranteed to be LLM-extracted from the section text, so exact or near-exact substring matches will be common. True fuzzy matching (Levenshtein edit distance) is only needed as a fallback for LLM paraphrasing.

```rust
// Source: project-internal, no crate needed
fn find_highlight_range(section_text: &str, snippet: &str) -> Option<(usize, usize)> {
    let norm_text = normalize_whitespace(section_text);
    let norm_snip = normalize_whitespace(snippet);
    if let Some(start) = norm_text.find(&norm_snip) {
        return Some((start, start + norm_snip.len()));
    }
    // Fallback: word-overlap sliding window
    word_overlap_search(&norm_text, &norm_snip)
}
```

### Pattern: LOD Visibility Thresholds
Decision is left to Claude's discretion. Based on the existing `radius_from_citations()` formula and the progressive reveal spec:

| Zoom level (viewport.scale) | Visible condition |
|-----------------------------|-------------------|
| < 0.3 | seed paper and depth-1 only |
| 0.3 – 0.6 | + citation_count >= 50 (high-citation) |
| 0.6 – 1.0 | + citation_count >= 10 (medium-citation) and depth <= 2 |
| > 1.0 | all nodes visible |

These thresholds are starting points — final values depend on observed graph density. The planner should document them as constants in a new `lod.rs` module for easy tuning.

Hidden nodes (lod_visible = false) render edges as `alpha = 0.05` (barely visible traces). The node itself is still drawn but at `alpha = 0.0` or `alpha = 0.02` — effectively invisible but preserving the instance draw call for topology hints.

For LOD to implement the "depth-1 always visible" rule, `GraphData` needs a `seed_paper_id` field, or `GraphState` needs to mark the seed node's index at construction time.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Persistent drawer tab state | Custom context/store | `RwSignal<DrawerTab>` local to `DrawerBody` | Tabs are local UI state, not global — no context needed |
| Complex slider component | Custom canvas slider | Two overlapping `<input type="range">` elements | Browser handles accessibility, keyboard nav, pointer events |
| LLM output validation | Custom JSON parser | Existing `serde_json::from_str` + `LLM_ANNOTATION_SCHEMA` retry loop | Already implemented in `llm/` pipeline |
| Snippet search with external crate | `fuzzy-matcher` or `strsim` | Inline normalized substring + word-overlap | Snippets are short; no crate warranted |
| Performance profiling infra | Custom timer | `web_sys::window().performance()` in WASM; `std::time::Instant` on server | Already available |

---

## Common Pitfalls

### Pitfall 1: LlmAnnotationRecord Does Not Need Schema Migration
**What goes wrong:** Adding `source_section`/`source_snippet` to `Finding` struct (which is stored as a JSON string in the `findings` field) does not require any DDL change. The `findings` column is `TYPE string` — it holds arbitrary JSON. Attempting to `DEFINE FIELD IF NOT EXISTS` on sub-fields of a JSON string field will fail or have no effect.

**How to avoid:** Only add a migration if a new top-level column on `llm_annotation` is needed. If `source_section`/`source_snippet` live inside `Finding` (which they do, as per the decision), no migration is required. The `#[serde(default)]` or `Option<>` on the new fields ensures backwards compatibility when reading old records.

**Warning signs:** If you see `migration 8` that touches `llm_annotation`, it should only be for adding a new top-level string column, not sub-fields.

### Pitfall 2: Borrow Checker with GraphState + Viewport in RAF Loop
**What goes wrong:** The RAF loop already has a pattern where `RenderState` holds `graph: GraphState` and `viewport: Viewport` — and they are borrowed separately. Adding `viewport.scale` reads to GraphState-dependent LOD logic may cause borrow conflicts if you try to read `viewport` and mutate `graph` in the same scope.

**How to avoid:** Snapshot `viewport.scale` into a local `f64` variable before borrowing `graph` mutably. Or snapshot it into `GraphState.current_scale: f64` at the top of the RAF frame where the full `RenderState` borrow is already held. The existing code pattern in `graph.rs` already separates concerns — follow it.

### Pitfall 3: `get_paper_detail` Does Not Return TextExtractionResult
**What goes wrong:** The provenance drawer needs section text to display. Currently `get_paper_detail()` in `server_fns/papers.rs` returns `PaperDetail { paper, annotation }` — it does NOT include `TextExtractionResult`. Failing to update this server function means the drawer always shows "Abstract only — full text unavailable" even for ar5iv-extracted papers.

**How to avoid:** Update `PaperDetail` to include `extraction: Option<TextExtractionResult>` and update `get_paper_detail()` to query `TextExtractionRepository`. The `TextExtractionRepository` and `get_extraction()` method already exist in `queries.rs`.

### Pitfall 4: LLM Prompt Token Budget at Scale
**What goes wrong:** Section-aware prompts send 4–5 sections of text, which for a dense paper can be 5,000–15,000 tokens. Some Ollama/LLM provider configurations have context limits.

**How to avoid:** Truncate each section to a max character count (e.g., 3,000 chars per section) before inserting into the prompt. Document the truncation in the prompt itself ("truncated to 3000 chars"). The LLM still has full structural context; truncation is a practical guard not an architectural compromise.

### Pitfall 5: Dual-Handle Slider Z-Index Crossing
**What goes wrong:** When the min-year handle is dragged past the max-year handle (or vice versa), one handle becomes unreachable because the other sits on top.

**How to avoid:** Use `pointer-events: none` on the lower-z handle when the values are equal or crossed, or swap `z-index` reactively based on which handle was last moved. Many implementations track which handle is "active" and boost its z-index.

### Pitfall 6: Graph Data Does Not Include Seed Paper ID for LOD
**What goes wrong:** The LOD spec requires the seed paper and depth-1 papers to always be visible. `GraphData` currently includes nodes with `id`, `title`, `year`, `citation_count` — but no BFS depth or seed designation.

**How to avoid:** Either (a) add a `bfs_depth: Option<u32>` and `is_seed: bool` field to `GraphNode` DTO, or (b) pass the seed paper ID as a separate field on `GraphData`. Option (a) is cleaner for the progressive reveal logic. The server-side `get_graph_data()` query has access to the crawl queue's `depth_level` field.

---

## Code Examples

### Adding source_section / source_snippet to Finding

```rust
// resyn-core/src/datamodels/llm_annotation.rs
// Source: project-internal — extend existing struct

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Finding {
    pub text: String,
    pub strength: String,
    // New optional provenance fields — None on old records (backward-compatible)
    #[serde(default)]
    pub source_section: Option<String>,  // e.g. "results", "methods"
    #[serde(default)]
    pub source_snippet: Option<String>,  // verbatim ~1-2 sentence quote
}
```

No DB migration needed — `findings` is stored as `TYPE string` (JSON). `#[serde(default)]` handles `None` on deserialization of old records.

### Section-Aware SYSTEM_PROMPT Structure

```rust
// resyn-core/src/llm/prompt.rs
// Replace the existing abstract-only prompt

pub const SYSTEM_PROMPT: &str = r#"You are a scientific paper analyst.
Extract structured information from the paper text below.

The paper text is organized by section. Sections marked [EMPTY] were not available.

For each finding and method you extract, include:
- source_section: the section name where this was found (e.g., "results", "methods", "abstract")
- source_snippet: a verbatim 1-2 sentence quote from that section supporting this finding/method

Extract:
- paper_type: experimental | theoretical | review | computational
- methods: array of {name, category, source_section, source_snippet}
- findings: array of {text, strength, source_section, source_snippet}
- open_problems: array of strings

Respond ONLY with a JSON object."#;
```

### Calling the Section-Aware Prompt

```rust
// resyn-core/src/llm/ — call site (assembles user message)
// Source: project-internal pattern

fn build_section_aware_user_message(extraction: &TextExtractionResult) -> String {
    let mut parts = vec![];
    let sec = &extraction.sections;
    push_section(&mut parts, "ABSTRACT", sec.abstract_text.as_deref());
    push_section(&mut parts, "INTRODUCTION", sec.introduction.as_deref());
    push_section(&mut parts, "METHODS", sec.methods.as_deref());
    push_section(&mut parts, "RESULTS", sec.results.as_deref());
    push_section(&mut parts, "CONCLUSION", sec.conclusion.as_deref());
    parts.join("\n\n")
}

fn push_section(parts: &mut Vec<String>, header: &str, text: Option<&str>) {
    const MAX_CHARS: usize = 3000;
    let body = match text {
        None => "[EMPTY]".to_string(),
        Some(t) if t.len() > MAX_CHARS => format!("{}…[truncated]", &t[..MAX_CHARS]),
        Some(t) => t.to_string(),
    };
    parts.push(format!("[{}]\n{}", header, body));
}
```

### Adding LOD visibility to GraphState

```rust
// resyn-app/src/graph/layout_state.rs — extend existing structs

pub struct NodeState {
    // ... existing fields unchanged ...
    pub bfs_depth: Option<u32>,  // NEW: populated from GraphData
    pub lod_visible: bool,        // NEW: computed each frame by LOD logic
    pub temporal_visible: bool,   // NEW: computed from temporal_range
}

pub struct GraphState {
    // ... existing fields unchanged ...
    pub temporal_min_year: u32,   // NEW: from slider signal
    pub temporal_max_year: u32,   // NEW: from slider signal
    pub seed_paper_id: Option<String>,  // NEW: from GraphData
    pub current_scale: f64,       // NEW: snapshot from Viewport each frame
}
```

### Computing LOD visibility each RAF frame

```rust
// Runs once per frame in start_render_loop before renderer.draw()
fn update_lod_visibility(state: &mut RenderState) {
    let scale = state.viewport.scale;
    state.graph.current_scale = scale;

    for node in &mut state.graph.nodes {
        let depth = node.bfs_depth.unwrap_or(u32::MAX);
        let cites = node.citation_count;
        node.lod_visible = match scale {
            s if s < 0.3  => depth <= 1,
            s if s < 0.6  => depth <= 1 || cites >= 50,
            s if s < 1.0  => depth <= 2 || cites >= 10,
            _              => true,
        };
    }
}

fn update_temporal_visibility(state: &mut RenderState) {
    let (min_y, max_y) = (state.graph.temporal_min_year, state.graph.temporal_max_year);
    for node in &mut state.graph.nodes {
        let year: u32 = node.year.parse().unwrap_or(0);
        node.temporal_visible = year == 0 || (year >= min_y && year <= max_y);
    }
}
```

### WebGL2 renderer: per-node final alpha

```rust
// In webgl_renderer.rs draw() — replace the existing alpha computation
let alpha = {
    let base = if dimmed && !is_selected && !is_hovered { 0.5_f32 } else { 1.0_f32 };
    let lod  = if node.lod_visible { 1.0_f32 } else { 0.03_f32 };
    let time = if node.temporal_visible { 1.0_f32 } else { 0.10_f32 };
    base * lod * time
};
```

### Temporal slider in GraphControls

```rust
// resyn-app/src/components/graph_controls.rs — add to component props

#[component]
pub fn GraphControls(
    // ... existing props ...
    temporal_min: RwSignal<u32>,
    temporal_max: RwSignal<u32>,
    year_bounds: (u32, u32),  // (earliest_year, latest_year) from graph data
) -> impl IntoView {
    view! {
        // ... existing controls ...
        <div class="temporal-slider-row">
            <label class="text-label">"Year range"</label>
            <div class="dual-range-wrapper">
                <input
                    type="range"
                    min=year_bounds.0
                    max=year_bounds.1
                    prop:value=move || temporal_min.get()
                    on:input=move |e| {
                        use leptos::wasm_bindgen::JsCast;
                        let val = e.target().unwrap()
                            .dyn_into::<web_sys::HtmlInputElement>().unwrap()
                            .value_as_number() as u32;
                        temporal_min.set(val);
                    }
                />
                <input
                    type="range"
                    min=year_bounds.0
                    max=year_bounds.1
                    prop:value=move || temporal_max.get()
                    on:input=move |e| {
                        use leptos::wasm_bindgen::JsCast;
                        let val = e.target().unwrap()
                            .dyn_into::<web_sys::HtmlInputElement>().unwrap()
                            .value_as_number() as u32;
                        temporal_max.set(val);
                    }
                />
            </div>
            <span class="text-label">
                {move || format!("{} – {}", temporal_min.get(), temporal_max.get())}
            </span>
        </div>
    }
}
```

### Node count indicator in GraphControls

```rust
// Add a prop: visible_count: Signal<(usize, usize)>  // (visible, total)
// In view!:
<span class="node-count-indicator">
    {move || {
        let (v, t) = visible_count.get();
        format!("Showing {} of {}", v, t)
    }}
</span>
```

The visible count is computed in the RAF loop: `state.graph.nodes.iter().filter(|n| n.lod_visible).count()` and written to an `RwSignal<(usize, usize)>` that the GraphControls component reads.

### PaperDetail extended for provenance

```rust
// resyn-app/src/server_fns/papers.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperDetail {
    pub paper: Paper,
    pub annotation: Option<LlmAnnotation>,
    pub extraction: Option<TextExtractionResult>,  // NEW
}
```

The `get_paper_detail()` server fn adds one query:
```rust
let extraction = TextExtractionRepository::new(&db)
    .get_extraction(&id)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;
```

### Drawer Source tab

```rust
// resyn-app/src/layout/drawer.rs — inside DrawerBody
// Source: extend existing component

#[derive(Clone, PartialEq)]
enum DrawerTab { Overview, Source }

// In DrawerBody, add tab state and Source tab body:
let active_tab = RwSignal::new(DrawerTab::Overview);
let highlight_snippet: Option<String> = None; // passed as prop when opened via GapCard

// Source tab renders extraction.sections fields with highlighted snippet spans:
fn render_section_with_highlight(text: &str, snippet: &Option<String>) -> impl IntoView {
    // Find highlight range, split text at boundaries, render with <mark> for highlighted span
}
```

---

## State of the Art

| Old Pattern | Current Pattern | Impact |
|-------------|-----------------|--------|
| Abstract-only LLM extraction | Section-structured extraction with source references | Better annotation quality; provenance links become possible |
| All nodes always rendered | LOD-aware instanced rendering with per-node alpha | 1000+ node graphs remain readable |
| No temporal filtering | Year-range dimming via dual slider | Researchers can isolate time periods without losing graph topology |

**No deprecated approaches in use.** The project is on current Leptos 0.8, SurrealDB v3, and web-sys WebGL2 — all current as of 2026-03.

---

## Open Questions

1. **BFS depth data in GraphData**
   - What we know: `GraphNode` DTO has no `bfs_depth` field; `CrawlQueue` records have `depth_level`; the `cites` relation does not store depth
   - What's unclear: Is BFS depth queryable from the graph DB, or does it need to be computed client-side via BFS from the seed?
   - Recommendation: Add `bfs_depth: Option<u32>` to `GraphNode` and populate it server-side with a SurrealDB graph traversal query from the seed paper, OR add a shortest-path computation client-side in `from_graph_data()`. The server-side query is preferable for correctness; SurrealDB supports path queries via `SELECT ->cites->paper FROM paper:⟨seed_id⟩`.

2. **GraphData seed paper identification**
   - What we know: `get_graph_data()` returns all papers — there is no current field indicating the seed paper
   - What's unclear: How does the client know which node is the seed for LOD priority?
   - Recommendation: Add `seed_paper_id: Option<String>` to `GraphData` DTO; populate from a new query or from a config/metadata table.

3. **Year bounds for temporal slider initialization**
   - What we know: `NodeState.year` is computed from `paper.published` during `from_graph_data()` as a 4-char string slice
   - What's unclear: Whether min/max year should be computed server-side (and added to `GraphData`) or client-side (computed in `from_graph_data()`)
   - Recommendation: Compute client-side in `GraphState::from_graph_data()` — it already iterates all nodes and year is already extracted. Return `(min_year, max_year)` alongside `GraphState`, or expose a method on `GraphState`.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (Rust built-in) |
| Config file | Cargo.toml workspace test targets |
| Quick run command | `cargo test --lib -q` |
| Full suite command | `cargo test -q` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| DEBT-04 | section-aware prompt assembles correct section blocks | unit | `cargo test test_build_section_aware_message -q` | ❌ Wave 0 |
| DEBT-04 | Finding serde round-trip with source_section + source_snippet | unit | `cargo test test_finding_serde_with_provenance -q` | ❌ Wave 0 |
| AUI-04 | highlight range found for exact snippet match | unit | `cargo test test_find_highlight_range_exact -q` | ❌ Wave 0 |
| AUI-04 | highlight range returns None for unmatched snippet | unit | `cargo test test_find_highlight_range_no_match -q` | ❌ Wave 0 |
| SCALE-02 | LOD visibility: seed node always visible at scale 0.1 | unit | `cargo test test_lod_seed_always_visible -q` | ❌ Wave 0 |
| SCALE-02 | LOD visibility: all nodes visible at scale 1.5 | unit | `cargo test test_lod_all_visible_at_high_zoom -q` | ❌ Wave 0 |
| SCALE-03 | temporal visibility: node in range is visible | unit | `cargo test test_temporal_node_in_range -q` | ❌ Wave 0 |
| SCALE-03 | temporal visibility: node out of range is dimmed | unit | `cargo test test_temporal_node_out_of_range -q` | ❌ Wave 0 |
| SCALE-01 | performance metrics recorded (manual) | manual-only | — | N/A |

Reasoning for manual-only on SCALE-01: Real crawl performance profiling requires network access, a real SurrealDB, and wall-clock timing across hundreds of HTTP requests — not suitable for automated unit tests.

### Sampling Rate
- **Per task commit:** `cargo test --lib -q`
- **Per wave merge:** `cargo test -q`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `resyn-core/src/llm/tests.rs` (or inline in `prompt.rs`) — section-aware message builder tests (DEBT-04)
- [ ] `resyn-core/src/datamodels/llm_annotation.rs` — extend existing test module with provenance field round-trip (AUI-04)
- [ ] `resyn-core/src/analysis/highlight.rs` — new module with `find_highlight_range()` tests (AUI-04)
- [ ] `resyn-app/src/graph/lod.rs` — new module with `update_lod_visibility()` + `update_temporal_visibility()` tests (SCALE-02, SCALE-03). Note: these are pure logic with no web-sys calls, so they run natively with `cargo test --lib`.

---

## Sources

### Primary (HIGH confidence)
- `/home/jasper/Repositories/research-synergy/resyn-core/src/datamodels/llm_annotation.rs` — `Finding`, `Method`, `LlmAnnotation` structs; established `#[serde(default)]` pattern
- `/home/jasper/Repositories/research-synergy/resyn-core/src/database/schema.rs` — migration 5 shows `findings TYPE string` — no DDL change needed for new Finding fields
- `/home/jasper/Repositories/research-synergy/resyn-core/src/database/queries.rs` — `LlmAnnotationRecord` JSON string pattern; `TextExtractionRepository.get_extraction()` confirmed present
- `/home/jasper/Repositories/research-synergy/resyn-core/src/datamodels/extraction.rs` — `SectionMap` structure confirmed; all section fields are `Option<String>`
- `/home/jasper/Repositories/research-synergy/resyn-app/src/graph/webgl_renderer.rs` — instanced draw + per-node alpha pattern; neighbor dimming implementation
- `/home/jasper/Repositories/research-synergy/resyn-app/src/graph/layout_state.rs` — `NodeState` fields including `year: String`; `GraphState` signal-sync pattern
- `/home/jasper/Repositories/research-synergy/resyn-app/src/pages/graph.rs` — RAF loop; signal-to-state sync pattern; borrow-checker split (Rc<RefCell> pair)
- `/home/jasper/Repositories/research-synergy/resyn-app/src/layout/drawer.rs` — `DrawerBody` structure; `PaperDetail` usage; `DrawerContent` Resource pattern
- `/home/jasper/Repositories/research-synergy/resyn-app/src/server_fns/papers.rs` — `PaperDetail` struct; `get_paper_detail()` confirmed to not include `TextExtractionResult` currently
- `/home/jasper/Repositories/research-synergy/resyn-core/src/llm/prompt.rs` — `SYSTEM_PROMPT` abstract-only; `LLM_ANNOTATION_SCHEMA` structure
- `/home/jasper/Repositories/research-synergy/resyn-app/src/components/graph_controls.rs` — existing control structure; `RwSignal` prop pattern
- `/home/jasper/Repositories/research-synergy/resyn-app/src/components/gap_card.rs` — `SelectedPaper` context; paper ID click → drawer open pattern
- `/home/jasper/Repositories/research-synergy/.planning/STATE.md` — accumulated decisions (Leptos 0.8 `.run()`, `StoredValue`, SurrealDB SCHEMAFULL pitfalls)
- `cargo test --lib -q` output — 214 tests passing, confirmed baseline

### Secondary (MEDIUM confidence)
- Leptos 0.8 `prop:value` attribute for controlled `<input>` elements — established by Phase 8/9 code patterns using `prop:` prefix for DOM properties vs `attr:` for HTML attributes
- `web_sys::HtmlInputElement::value_as_number()` — standard web-sys API for range input values; consistent with existing `web_sys::HtmlCanvasElement` pattern in the codebase

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all dependencies already present in Cargo.toml; no new crates required
- Architecture patterns: HIGH — all patterns extend confirmed existing code; no speculation
- Pitfalls: HIGH — Pitfall 1 (JSON string DB field) and Pitfall 3 (missing TextExtractionResult in PaperDetail) verified directly from source files; others derive from established SurrealDB/borrow-checker patterns in STATE.md
- LOD thresholds: MEDIUM — recommended values are a reasonable starting point; actual tuning needed at runtime
- Fuzzy match algorithm: MEDIUM — approach is sound but exact implementation details depend on LLM snippet quality in practice

**Research date:** 2026-03-18
**Valid until:** 2026-04-18 (stable stack; no fast-moving dependencies)
