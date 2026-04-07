# Phase 21: Search & Filter - Research

**Researched:** 2026-04-07
**Domain:** SurrealDB full-text search, Leptos reactive UI, WebCanvas node highlighting
**Confidence:** HIGH

## Summary

This phase adds full-text paper search backed by SurrealDB BM25 indexing and three UI touch points: a global search bar in the app header, a papers-table inline filter, and graph-node pan/highlight on result selection.

All three features share one server function (`search_papers`). The Leptos app already has `signal_debounced` available through `leptos-use 0.18`, which is the right primitive for 300 ms debounce. The SurrealDB full-text search syntax for v3 (in use) requires one `DEFINE INDEX … FULLTEXT ANALYZER … BM25` per field; three indexes cover title, summary (abstract), and authors. The `@@` match operator combines them in a single query using `search::score()` for relevance ranking.

The graph highlight (D-06: pulse glow ring) must be drawn inside the existing Canvas2D render loop in `canvas_renderer.rs`, because the graph uses raw canvas rather than DOM elements. The "no panning while typing" constraint (D-08) is enforced by separating the search query signal (updates on every keystroke) from the pan trigger signal (updates only when the user commits to a result).

**Primary recommendation:** Add migration 9 with three BM25 fulltext indexes, add `search_papers` server fn returning ranked `SearchResult` items, add `GlobalSearchBar` component to the app-shell header, and wire graph pan through a new `SearchPanRequest` context signal (parallel to `SelectedPaper`).

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Search placement & UX**
- D-01: Global search bar in the app header/nav, persistent across all pages
- D-02: Results appear as a dropdown list below the search bar (top 8-10 matches with title, authors, year)
- D-03: Result actions are context-sensitive: on graph page pans to node + opens detail drawer, on papers page scrolls to row + opens detail drawer, elsewhere opens detail drawer
- D-04: Ctrl+K / Cmd+K keyboard shortcut focuses the search bar from any page

**Graph search integration**
- D-05: Smooth lerp pan+zoom animation to center the target node (reuse existing `viewport_fit` lerp pattern)
- D-06: Matched node gets a pulse glow ring (2–3 pulses over ~2 s) then fades back to normal
- D-07: Multi-match: viewport centers on top-ranked result, other non-matching nodes dimmed
- D-08: Graph does NOT pan while typing — only on explicit result selection (click or Enter). Dropdown updates live with 300 ms debounce, viewport stays still until user commits.

**Search backend strategy**
- D-09: SurrealDB full-text search with DEFINE ANALYZER and search index on paper table
- D-10: Searchable fields: title, abstract (summary), authors
- D-11: Relevance ranking via SurrealDB native `search::score()` (BM25-based)

**Papers table filtering**
- D-12: Separate inline filter bar above the papers table (independent from global search bar)
- D-13: Server-side search — uses the same SurrealDB full-text search as the global bar (debounced)
- D-14: Matching text highlighted (bold or color) in title/author cells when filtering

### Claude's Discretion
- Debounce timing for papers table filter (suggested ~300 ms to match global bar)
- Exact dropdown styling and positioning
- Pulse glow implementation details (CSS animation vs WebGL shader)
- Search result count display and "no results" empty state messaging
- Whether to show search result count badge

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| SRCH-01 | User can search papers by title, abstract, or author from a search bar | GlobalSearchBar component + `search_papers` server fn with three BM25 indexes |
| SRCH-02 | Search results are ranked by relevance using SurrealDB full-text search | `search::score(0)*2 + search::score(1)*1.5 + search::score(2)` combined score, ORDER BY DESC |
| SRCH-03 | User can search from the graph page and viewport pans to matching node with highlight flash | `SearchPanRequest` context signal, reuse `FitAnimState` lerp + pulse counter field in `RenderState` |
| SRCH-04 | Papers table integrates search bar for filtering displayed papers | Inline filter `RwSignal<String>` in `PapersPanel`, debounced `Resource` re-fetch via `signal_debounced` |
</phase_requirements>

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| SurrealDB (kv-surrealkv / kv-mem) | 3 (workspace) | Full-text search indexes + query execution | Already in workspace; BM25 supported since v3 beta |
| leptos-use | 0.18 (resyn-app/Cargo.toml) | `signal_debounced` for 300 ms input debounce | Already in project; avoids hand-rolling setTimeout logic |
| web-sys / wasm-bindgen | 0.2 (workspace) | `document.add_event_listener` for Ctrl+K global shortcut | Already used in graph.rs for canvas events |
| Leptos `RwSignal` / `Resource` | 0.7 (workspace) | Reactive state for search query and results | Established pattern throughout codebase |

[VERIFIED: resyn-app/Cargo.toml, resyn-core/Cargo.toml — all packages already in workspace]

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `leptos_router::use_location` | 0.7 | Detect current page for context-sensitive result action | Needed by GlobalSearchBar to know if on `/graph` page |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| SurrealDB BM25 FTS | Client-side filter (String::contains) | Client-side is simpler for tiny datasets but does not rank by relevance and breaks D-11 |
| `signal_debounced` | Manual `set_timeout` with `Closure` | `signal_debounced` is already a dependency; manual Closure approach is more error-prone and verbose |

**Installation:** No new packages — all dependencies already in Cargo.toml.

---

## Architecture Patterns

### Recommended Project Structure

```
resyn-core/src/database/
├── schema.rs          # Add migration 9: DEFINE ANALYZER + 3 FULLTEXT indexes
├── queries.rs         # Add SearchRepository with search_papers()
└── (no new files)

resyn-app/src/
├── app.rs             # Add SearchPanRequest context + provide_context
├── server_fns/
│   └── papers.rs      # Add SearchResult struct + search_papers server fn
├── components/
│   └── search_bar.rs  # NEW: GlobalSearchBar component
├── pages/
│   ├── papers.rs      # Add inline filter bar + highlight rendering
│   └── graph.rs       # Add SearchPanRequest effect → pan trigger
└── graph/
    ├── layout_state.rs # Add search_highlighted: Option<String> to GraphState
    └── canvas_renderer.rs  # Add pulse glow ring rendering for highlighted node
```

### Pattern 1: SurrealDB Full-Text Search — Migration

**What:** Add a new `apply_migration_9` to `schema.rs` that defines an analyzer and three fulltext indexes on `paper`.
**When to use:** Any time a new indexed capability is added to SurrealDB.

```rust
// Source: surrealdb.com/docs/surrealdb/models/full-text-search [CITED]
async fn apply_migration_9(db: &Surreal<Any>) -> Result<(), ResynError> {
    db.query(
        "
        DEFINE ANALYZER IF NOT EXISTS paper_analyzer
            TOKENIZERS blank, class
            FILTERS lowercase, ascii;

        DEFINE INDEX IF NOT EXISTS idx_paper_fts_title
            ON paper FIELDS title
            FULLTEXT ANALYZER paper_analyzer BM25;

        DEFINE INDEX IF NOT EXISTS idx_paper_fts_summary
            ON paper FIELDS summary
            FULLTEXT ANALYZER paper_analyzer BM25;

        DEFINE INDEX IF NOT EXISTS idx_paper_fts_authors
            ON paper FIELDS authors
            FULLTEXT ANALYZER paper_analyzer BM25;
        ",
    )
    .await
    .map_err(|e| ResynError::Database(format!("migration 9 DDL failed: {e}")))?;
    Ok(())
}
```

**Key insight on authors field:** SurrealDB FTS indexes support `array<string>` fields — elements are tokenized individually. [CITED: surrealdb.com/docs/surrealdb/models/full-text-search]

**HIGHLIGHTS clause:** The CONTEXT.md does not require server-side highlighting (only client-side bold for the table filter), so `HIGHLIGHTS` can be omitted from the index definitions. This keeps migration simpler. [ASSUMED: omitting HIGHLIGHTS is compatible with search::score() — verify if search_papers returns snippets in future]

### Pattern 2: SurrealDB Multi-Field Search Query

**What:** A single SELECT that searches across three indexed fields and returns a combined BM25 score.
**When to use:** Every call to `search_papers` server fn.

```sql
-- Source: surrealdb.com/docs/surrealql/functions/database/search [CITED]
SELECT
    arxiv_id,
    title,
    authors,
    published,
    (search::score(0) * 2.0 + search::score(1) * 1.5 + search::score(2) * 1.0) AS score
FROM paper
WHERE title @0@ $query
   OR summary @1@ $query
   OR authors @2@ $query
ORDER BY score DESC
LIMIT 10
```

Score weights: title (2×) > abstract (1.5×) > authors (1×). The `@N@` predicate reference number must match the argument passed to `search::score(N)`. [CITED: surrealdb.com/docs/surrealql/functions/database/search]

### Pattern 3: Leptos Debounced Resource

**What:** A server fn-backed Resource that re-fetches when a debounced signal changes.
**When to use:** Both GlobalSearchBar dropdown and PapersPanel inline filter.

```rust
// Source: leptos-use.rs/reactivity/signal_debounced.html [CITED]
use leptos_use::signal_debounced;

let query = RwSignal::new(String::new());
// signal_debounced produces a derived signal that only updates after 300ms quiet period
let debounced_query = signal_debounced(query, 300.0);

// Resource re-fetches only when debounced_query changes
let results = Resource::new(
    move || debounced_query.get(),
    |q| async move {
        if q.is_empty() {
            Ok(vec![])
        } else {
            search_papers(q).await
        }
    },
);
```

**Important caveat:** `signal_debounced` uses `setTimeout` internally and is a no-op on SSR. This is acceptable because search is a client-driven interaction. [CITED: leptos-use.rs/reactivity/signal_debounced.html]

### Pattern 4: SearchPanRequest Context Signal

**What:** An app-level context signal (parallel to `SelectedPaper`) that tells the graph page to pan to a specific node and highlight it.
**When to use:** When user selects a search result while on the graph page, or navigates to the graph page after selecting a result from another page.

```rust
// Source: resyn-app/src/app.rs — SelectedPaper pattern [VERIFIED]

// In app.rs:
#[derive(Clone, Debug, Default)]
pub struct SearchPanRequest {
    pub paper_id: String,
}

#[derive(Clone, Copy)]
pub struct SearchPanTrigger(pub RwSignal<Option<SearchPanRequest>>);

// provide_context(SearchPanTrigger(RwSignal::new(None)));

// In graph.rs Effect — fires when SearchPanTrigger changes:
// 1. Find node index by paper_id in graph_state.nodes
// 2. Set render_state.fit_anim to center on that node (reuse compute_fit_target with single node)
// 3. Set render_state.graph.search_highlighted = Some(paper_id)
// 4. Set render_state.graph.pulse_start_frame = Some(current_frame_count)
```

### Pattern 5: Ctrl+K Global Keyboard Shortcut

**What:** A global `keydown` listener on `document` that focuses the search input.
**When to use:** Once, in the `GlobalSearchBar` component's `Effect::new`.

```rust
// Source: resyn-app/src/pages/graph.rs add_event_listener pattern [VERIFIED]
use wasm_bindgen::prelude::*;
use web_sys::KeyboardEvent;

// In GlobalSearchBar component:
let input_ref = NodeRef::<leptos::html::Input>::new();

Effect::new(move |_| {
    let doc = web_sys::window().unwrap().document().unwrap();
    let input_ref = input_ref.clone();
    let cb = Closure::<dyn FnMut(KeyboardEvent)>::new(move |e: KeyboardEvent| {
        if (e.ctrl_key() || e.meta_key()) && e.key() == "k" {
            e.prevent_default();
            if let Some(el) = input_ref.get() {
                let _ = el.focus();
            }
        }
    });
    doc.add_event_listener_with_callback("keydown", cb.as_ref().unchecked_ref())
        .unwrap();
    cb.forget(); // intentional: lives for app lifetime
});
```

### Pattern 6: Pulse Glow Ring in Canvas2D

**What:** A time-based ring drawn on the search-matched node in `canvas_renderer.rs`.
**When to use:** When `GraphState.search_highlighted == Some(node_id)` and pulse animation is active.

Decision D-06 says "pulse glow ring (2–3 pulses over ~2 s) then fades back to normal." This belongs in the canvas render loop, not CSS (the graph is a canvas element).

Implementation approach: add a `pulse_frame: Option<u32>` field to `RenderState` (set when pan is triggered). Each `draw()` call, if `pulse_frame` is set:
- Compute `t = (current_frame - pulse_frame) / 120` (120 frames ≈ 2 s at 60fps)
- Draw 1–2 additional arcs at `radius + 4 + 8*sin(t * 3π)` with alpha = `(1.0 - t).max(0.0)`
- When `t >= 1.0`, clear `pulse_frame`

This is the same approach used for the `is_selected` outer ring in `canvas_renderer.rs` (draw extra `arc` after node fill). [VERIFIED: resyn-app/src/graph/canvas_renderer.rs lines 266–281]

### Pattern 7: Node Dimming for Multi-Match (D-07)

**What:** When a search result is selected, non-matching nodes are dimmed (like the current selection dimming).
**When to use:** After user commits to a result with multiple matches in the graph.

`GraphState` already has `topic_dimmed` per-node and `selected_node`-based dimming in `canvas_renderer.rs`. Add a `search_dimmed` concept: when `search_highlighted` is set, nodes not matching the query have their alpha reduced (like `is_dimmed` in canvas_renderer). Given D-07 says "other non-matching nodes dimmed," the simplest approach is to add a `search_highlight_ids: Vec<String>` to `GraphState` — populated when a search result is committed — and extend `is_dimmed` to also check search context.

### Anti-Patterns to Avoid

- **Triggering pan on every keystroke:** D-08 explicitly forbids this. The debounced query updates the dropdown, but pan only fires on `on:click` or `on:keydown Enter` on a result.
- **Searching with empty query string:** The SurrealDB FTS `@@` operator will return all records if given an empty string in some versions. Always short-circuit: return `Ok(vec![])` when `query.trim().is_empty()`. [ASSUMED: behavior on empty string — verify at implementation]
- **Single combined FTS index on multiple fields:** SurrealDB requires one index per field for multi-field search. One `DEFINE INDEX` with two fields is NOT supported for fulltext. [CITED: surrealdb.com/docs/surrealdb/models/full-text-search]
- **Using CSS animations for graph node pulse:** The graph node is rendered on an HTML `<canvas>` element — CSS animations have no effect on canvas content. Pulse must be drawn in the render loop.
- **Forgetting `IF NOT EXISTS` on `DEFINE INDEX`:** Existing migration pattern always uses `IF NOT EXISTS` (migration 1–8). Follow this pattern. [VERIFIED: resyn-core/src/database/schema.rs]
- **`cb.forget()` omission for global event listener:** The Ctrl+K listener must `forget()` the closure to keep it alive for the app lifetime. The graph.rs pattern uses this approach. [VERIFIED: resyn-app/src/pages/graph.rs]

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Debounce 300ms before server call | Manual `setTimeout` Closure | `leptos_use::signal_debounced` | Already a dependency; safer lifetime management in Leptos reactive graph |
| BM25 relevance ranking | Custom TF-IDF client-side ranking | SurrealDB `search::score()` | BM25 is nontrivial to implement correctly; SurrealDB already has it |
| Client-side fuzzy search | `strsim` / custom edit-distance | SurrealDB FTS tokenizer + BM25 | Phase specifically decided D-09 server-side; client-side alternatives ignore this |
| Text highlight on canvas | CSS `<mark>` | Canvas2D `arc` + `globalAlpha` | Canvas doesn't expose DOM — must use canvas drawing primitives |

---

## Common Pitfalls

### Pitfall 1: Empty Query Returns All Papers
**What goes wrong:** SurrealDB `@0@ ""` may match all records (behavior is version-dependent for empty string tokenization).
**Why it happens:** Empty token after tokenization is edge-case behavior.
**How to avoid:** Guard in `search_papers`: `if query.trim().is_empty() { return Ok(vec![]); }` before executing the SurrealDB query.
**Warning signs:** Dropdown shows all papers when search bar is focused but empty.

### Pitfall 2: `@@` Operator Requires Fulltext Index — Runtime Error Without It
**What goes wrong:** If migration 9 hasn't run (e.g., connecting to an existing DB that predates the migration), the `@0@` operator throws a `surrealdb::Error` at query time.
**Why it happens:** The match operator is only valid when a fulltext index covers the field.
**How to avoid:** `migrate_schema` in `schema.rs` runs all pending migrations at startup; because the existing pattern uses a version-gated `if version < N` block, migration 9 will run once on first startup with the new code.
**Warning signs:** `ServerFnError` containing "There was a problem with the database" or "no search index found" in server logs.

### Pitfall 3: `signal_debounced` Is a No-Op on SSR
**What goes wrong:** If `search_papers` were called during SSR, the debounce would not apply and the raw signal would be used.
**Why it happens:** `setTimeout` is not available server-side.
**How to avoid:** Search is triggered by user typing (client-only interaction). The Resource will be `None` / empty on SSR initial render, which is correct behavior — the search bar should render empty on first load.
**Warning signs:** Unexpected server-side calls to `search_papers` with empty or initial state.

### Pitfall 4: `compute_fit_target` Fits All Visible Nodes — Need Single-Node Variant
**What goes wrong:** Using `compute_fit_target` directly to pan to a single node may zoom too far in (scale clamps to 4.0 but at single-node the scale will be maxed and the view will be very zoomed in, losing graph context).
**Why it happens:** `compute_fit_target` computes the bounding box of all visible nodes, not a single target.
**How to avoid:** For search-pan, do not use `compute_fit_target`. Instead, directly set `fit_anim.target_offset_x/y` to center the target node at a reasonable scale (e.g., preserve current scale or use `clamp(current_scale, 0.5, 2.0)`). The lerp mechanism (`FitAnimState`) is still the right primitive.
**Warning signs:** Graph zooms to maximum scale (4.0) when a search result is selected, making the node fill the screen.

### Pitfall 5: Authors Is `array<string>` — Query Uses Join or String Conversion
**What goes wrong:** `authors @2@ $query` might not work as expected if SurrealDB's FTS tokenizer handles array fields differently than string fields in the `@@` operator.
**Why it happens:** Array fields are supported for FTS indexing (each element is indexed), but the `@@` predicate behavior on arrays needs verification.
**How to avoid:** Test the authors search case explicitly in the DB integration test. Fallback: if `authors @2@` is problematic, concatenate authors on the server before the FTS query using `string::join(authors, ' ')` in SurrealQL — but prefer the native array approach first.
**Warning signs:** Author name searches return zero results while title/abstract searches work.

### Pitfall 6: Cross-Page Navigation After Search Result Selection
**What goes wrong:** If user selects a result from the Dashboard (non-graph, non-papers page), navigation to `/graph` must also trigger the pan. But the `SearchPanTrigger` signal may not be read by `GraphPage` until after mount.
**Why it happens:** Leptos router mounts the new route component after navigation; the `Effect` in `GraphPage` that reads `SearchPanTrigger` fires after `GraphPage` mounts, which happens after the signal was set.
**How to avoid:** The signal persists in context until explicitly cleared, so `GraphPage`'s `Effect::new` will naturally read it on mount. Do NOT clear the signal on the navigate call — clear it after the pan animation completes in `GraphPage`.
**Warning signs:** Pan does not occur when navigating from Dashboard to Graph after search selection.

---

## Code Examples

### SearchResult data model (server fn return type)
```rust
// Source: resyn-app/src/server_fns/papers.rs — PaperDetail pattern [VERIFIED]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub arxiv_id: String,
    pub title: String,
    pub authors: Vec<String>,
    pub year: String,        // first 4 chars of published
    pub score: f32,
}

#[server(SearchPapers, "/api")]
pub async fn search_papers(query: String) -> Result<Vec<SearchResult>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        if query.trim().is_empty() {
            return Ok(vec![]);
        }
        use resyn_core::database::queries::SearchRepository;
        let db = use_context::<std::sync::Arc<resyn_core::database::client::Db>>()
            .ok_or_else(|| ServerFnError::new("Database not available"))?;
        SearchRepository::new(&db)
            .search_papers(&query, 10)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))
    }
    #[cfg(not(feature = "ssr"))]
    unreachable!()
}
```

### SearchRepository in queries.rs
```rust
// Pattern follows existing PaperRepository style [VERIFIED: resyn-core/src/database/queries.rs]
pub struct SearchRepository<'a> {
    db: &'a Db,
}

impl<'a> SearchRepository<'a> {
    pub fn new(db: &'a Db) -> Self { Self { db } }

    pub async fn search_papers(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResultRow>, ResynError> {
        // SurrealDB query using @0@, @1@, @2@ predicates
        // Returns rows with arxiv_id, title, authors, published, score
        // ORDER BY score DESC LIMIT $limit
        // ...
    }
}
```

### Inline filter in PapersPanel (signal pattern)
```rust
// Pattern follows sort_col / sort_dir RwSignal pattern [VERIFIED: resyn-app/src/pages/papers.rs]
let filter_query = RwSignal::new(String::new());
let debounced_filter = signal_debounced(filter_query, 300.0);

let filter_resource = Resource::new(
    move || debounced_filter.get(),
    |q| async move {
        if q.is_empty() { get_papers().await }
        else { search_papers(q).await.map(|r| r.into_iter().map(|sr| /* convert */).collect()) }
    }
);
```

### Client-side text highlight in table cells
```rust
// Decision D-14: highlight matching text bold/color in title/author cells
// Since server returns title string and query string, highlight is done client-side:
fn highlight_text(text: &str, query: &str) -> impl IntoView {
    // Simple case-insensitive split and highlight
    // Returns text with <mark> or <strong> wrapping matches
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `SEARCH ANALYZER` syntax | `FULLTEXT ANALYZER` | SurrealDB 3.0.0-beta | New syntax required — old docs may show deprecated form |
| Per-row `contains()` client filter | BM25 fulltext index | Phase 21 introduces | Proper relevance ranking replaces insertion-order results |

**Deprecated/outdated:**
- `SEARCH ANALYZER` keyword: replaced by `FULLTEXT ANALYZER` in SurrealDB v3 beta. [CITED: surrealdb.com/docs/surrealdb/models/full-text-search]

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `signal_debounced` from `leptos-use 0.18` has the same API as documented for leptos 0.7 compatibility | Standard Stack / Pattern 3 | Debounce implementation needs rework; fallback is manual `set_timeout` |
| A2 | `HIGHLIGHTS` clause can be omitted from the index definition without breaking `search::score()` | Pattern 1 | If HIGHLIGHTS is required for score to work, migration 9 must add it; safe to add, no downside |
| A3 | Empty string passed to `@0@` operator returns all matching records rather than zero | Pitfall 1 | If SurrealDB returns zero results for empty string, the guard is still correct but the pitfall description changes |
| A4 | `array<string>` authors field is tokenized element-by-element and supports `@@` match operator correctly | Pitfall 5 | Authors search returns no results; fallback: use `string::join` in SurrealQL |
| A5 | `FitAnimState` can be reused for single-node pan by computing offset directly (not via `compute_fit_target`) | Pattern 4 / Pitfall 4 | Single-node pan zoom behavior differs; adjust scale logic if needed |

---

## Open Questions

1. **`HIGHLIGHTS` required for BM25 indexes?**
   - What we know: `search::score()` and `HIGHLIGHTS` are listed as separate capabilities in SurrealDB docs
   - What's unclear: Whether `BM25` alone (without `HIGHLIGHTS`) still populates the score function
   - Recommendation: Add `HIGHLIGHTS` to all three indexes to be safe; it only increases index size slightly and enables future snippet extraction

2. **SurrealDB FTS on `array<string>` with `@@` operator**
   - What we know: FTS indexes support arrays (docs confirm elements are indexed individually)
   - What's unclear: Whether `authors @2@ $query` correctly matches within array elements at query time
   - Recommendation: Write a targeted DB integration test (`test_search_by_author`) using `connect_memory()` before implementing the frontend

3. **Performance: FTS on `kv-surrealkv` vs `kv-mem`**
   - What we know: Both backends support the same SurrealDB feature set including FTS
   - What's unclear: FTS index rebuild time on an existing database with hundreds of papers when migration 9 runs
   - Recommendation: Acceptable risk — migration runs at startup, FTS index build is fast for hundreds of records

---

## Environment Availability

Step 2.6: SKIPPED — this phase adds code and schema changes only. No new external tools, services, or runtimes required beyond the existing SurrealDB and Leptos stack already present in the workspace.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust `#[tokio::test]` (server-side), `wasm-bindgen-test` (not currently used in project) |
| Config file | `Cargo.toml` test configuration (workspace) |
| Quick run command | `cargo test search` |
| Full suite command | `cargo test` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SRCH-01 | `search_papers("quantum")` returns results ranked by score | unit (DB) | `cargo test test_search_papers_returns_ranked_results` | ❌ Wave 0 |
| SRCH-01 | Empty query returns empty vec | unit (DB) | `cargo test test_search_papers_empty_query` | ❌ Wave 0 |
| SRCH-02 | Title match scores higher than author match | unit (DB) | `cargo test test_search_papers_title_scores_higher` | ❌ Wave 0 |
| SRCH-02 | Results ordered by score DESC | unit (DB) | `cargo test test_search_papers_result_order` | ❌ Wave 0 |
| SRCH-03 | `compute_single_node_pan_target` returns correct offset for known node position | unit | `cargo test test_compute_single_node_pan_target` | ❌ Wave 0 |
| SRCH-04 | Authors search returns results containing author name | unit (DB) | `cargo test test_search_papers_by_author` | ❌ Wave 0 |

All tests use `connect_memory()` — no external DB required. [VERIFIED: existing test pattern in queries.rs line 855]

### Sampling Rate
- **Per task commit:** `cargo test search`
- **Per wave merge:** `cargo test`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `resyn-core/src/database/queries.rs` — add `SearchRepository` with tests in `mod tests`
- [ ] `resyn-app/src/graph/viewport_fit.rs` — add `compute_single_node_pan_target` function + test
- [ ] No new test files needed — tests live in existing `mod tests` blocks

---

## Security Domain

### Applicable ASVS Categories
| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | N/A (no auth system) |
| V3 Session Management | no | N/A |
| V4 Access Control | no | Single-user local tool |
| V5 Input Validation | yes | SurrealDB parameterized query with `$query` bind parameter |
| V6 Cryptography | no | N/A |

### Known Threat Patterns
| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| SurrealQL injection via search query | Tampering | Use `db.query(sql).bind(("query", &query))` — never string-interpolate query into SurrealQL |
| Excessive server load from rapid typing | DoS | 300ms `signal_debounced` on client; empty-string guard on server |

**Critical:** The SurrealDB query MUST use bind parameters, not string interpolation:
```rust
// CORRECT — parameterized [VERIFIED: existing pattern in queries.rs]
db.query("SELECT ... FROM paper WHERE title @0@ $query ...")
  .bind(("query", query))
  .await

// WRONG — injection risk
db.query(format!("SELECT ... WHERE title @0@ '{query}'")).await
```

---

## Sources

### Primary (HIGH confidence)
- SurrealDB docs: full-text search model — `DEFINE ANALYZER`, `DEFINE INDEX FULLTEXT`, `@@` operator, `search::score()` [CITED]
- SurrealDB docs: search functions reference — multi-predicate scoring, parameter numbering [CITED]
- `resyn-core/src/database/schema.rs` — migration pattern, field names confirmed [VERIFIED]
- `resyn-app/src/server_fns/papers.rs` — server fn pattern, context extraction [VERIFIED]
- `resyn-app/src/pages/papers.rs` — `RwSignal` sort state pattern [VERIFIED]
- `resyn-app/src/graph/canvas_renderer.rs` — node draw loop, ring drawing pattern [VERIFIED]
- `resyn-app/src/graph/viewport_fit.rs` — `FitAnimState` / `lerp` reuse opportunity [VERIFIED]
- `resyn-app/src/app.rs` — `SelectedPaper` context signal pattern [VERIFIED]
- `resyn-app/Cargo.toml` — `leptos-use 0.18` confirmed [VERIFIED]

### Secondary (MEDIUM confidence)
- leptos-use.rs/reactivity/signal_debounced — `signal_debounced` API [CITED via WebSearch]
- SurrealDB: array fields supported for FTS indexing [CITED via WebSearch summary of official docs]

### Tertiary (LOW confidence)
- SurrealDB FTS behavior on empty string queries — not directly verified [ASSUMED: A3]
- `HIGHLIGHTS` optional for `search::score()` — not directly verified [ASSUMED: A2]

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all libraries verified in Cargo.toml; SurrealDB FTS syntax cited from official docs
- Architecture: HIGH — patterns derived from verified existing codebase patterns
- SurrealDB FTS query syntax: MEDIUM — cited from docs, not executed in this session; two edge cases remain ASSUMED
- Pitfalls: HIGH — derived from code inspection + known SurrealDB behavior

**Research date:** 2026-04-07
**Valid until:** 2026-07-07 (SurrealDB v3 API is stable; leptos-use 0.18 is pinned in Cargo.toml)
