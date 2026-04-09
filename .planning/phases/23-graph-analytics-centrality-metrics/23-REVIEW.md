---
phase: 23-graph-analytics-centrality-metrics
reviewed: 2026-04-09T10:00:00Z
depth: standard
files_reviewed: 22
files_reviewed_list:
  - resyn-app/src/components/graph_controls.rs
  - resyn-app/src/graph/canvas_renderer.rs
  - resyn-app/src/graph/interaction.rs
  - resyn-app/src/graph/label_collision.rs
  - resyn-app/src/graph/layout_state.rs
  - resyn-app/src/graph/lod.rs
  - resyn-app/src/graph/viewport_fit.rs
  - resyn-app/src/graph/webgl_renderer.rs
  - resyn-app/src/pages/dashboard.rs
  - resyn-app/src/pages/graph.rs
  - resyn-app/src/server_fns/analysis.rs
  - resyn-app/src/server_fns/metrics.rs
  - resyn-app/src/server_fns/mod.rs
  - resyn-app/style/main.css
  - resyn-core/src/database/queries.rs
  - resyn-core/src/database/schema.rs
  - resyn-core/src/datamodels/graph_metrics.rs
  - resyn-core/src/datamodels/mod.rs
  - resyn-core/src/graph_analytics/betweenness.rs
  - resyn-core/src/graph_analytics/mod.rs
  - resyn-core/src/graph_analytics/pagerank.rs
  - resyn-core/src/lib.rs
findings:
  critical: 1
  warning: 4
  info: 3
  total: 8
status: issues_found
---

# Phase 23: Code Review Report

**Reviewed:** 2026-04-09T10:00:00Z
**Depth:** standard
**Files Reviewed:** 22
**Status:** issues_found

## Summary

This phase introduces PageRank and betweenness centrality computation, DB persistence (`graph_metrics` table via migration 11), server functions for querying/triggering metrics, and frontend integration — new "Size by" dropdown, `metrics_ready`/`metrics_computing` signals, node radius lerp, and a dashboard "Most Influential Papers" card.

The analytics core (`betweenness.rs`, `pagerank.rs`) is correct, well-tested, and the Brandes implementation is sound. The DB layer, schema migration, and server function design are clean. One critical bug exists in `hex_to_rgb` (panic on short hex strings), and several logic gaps were found around `metrics_computing` state management, title truncation safety, and the betweenness normalization edge case at `n == 2`.

---

## Critical Issues

### CR-01: `hex_to_rgb` panics on hex strings shorter than 6 characters

**File:** `resyn-app/src/graph/webgl_renderer.rs:866`
**Issue:** `hex_to_rgb` slices `hex[0..2]`, `hex[2..4]`, `hex[4..6]` without checking length. All call sites in the codebase pass 6-digit literals so this is safe today, but `hex_to_rgb` is `pub` and the function contract does not validate input. Any caller passing a 3-digit or empty string (for example, a future data-driven color string from the palette or a user-provided theme) will panic with an index out of bounds at runtime.

**Fix:**
```rust
pub fn hex_to_rgb(hex: &str) -> (f32, f32, f32) {
    let hex = hex.trim_start_matches('#');
    if hex.len() < 6 {
        return (0.0, 0.0, 0.0); // fallback: black
    }
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f32 / 255.0;
    (r, g, b)
}
```

---

## Warnings

### WR-01: `metrics_computing` signal is never set to `true` — spinner never appears

**File:** `resyn-app/src/pages/graph.rs:119` and `resyn-app/src/components/graph_controls.rs:191`
**Issue:** `metrics_computing` is initialized to `false` and never set to `true` anywhere in the codebase. The "Size by" label renders a spinner (`<span class="spinner-sm"></span>`) when `metrics_computing.get()` is true, and the recompute button disables itself on that condition. Because the signal is never flipped, the UI provides no visual feedback while `trigger_metrics_compute` is running in the background. The user sees an identical state whether metrics are computing or not.

**Fix:** The `trigger_metrics_compute` server function returns immediately (fire-and-forget). The simplest correct approach is to set `metrics_computing` to `true` before calling `trigger_metrics_compute()` and then poll `get_metrics_status()` after a short delay (or on the next user action) to set it back to `false`. Alternatively, add a `metrics_fingerprint` signal so the RAFloop can detect when the `metrics_map_signal` is refreshed and clear `metrics_computing`. At minimum:

```rust
// In graph_controls.rs on:click handler for the recompute button:
on:click=move |_| {
    use crate::server_fns::metrics::trigger_metrics_compute;
    metrics_computing.set(true); // <-- add this
    leptos::task::spawn_local(async move {
        let _ = trigger_metrics_compute().await;
        // Re-poll status after trigger so metrics_computing is cleared when done
        // (requires passing metrics_ready/metrics_computing into this closure)
    });
}
```

### WR-02: Title truncation in dashboard bypasses char-boundary safety

**File:** `resyn-app/src/pages/dashboard.rs:132`
**Issue:** `&entry.title[..50]` performs a byte-slice, not a character-slice. If the 50th byte falls in the middle of a multi-byte UTF-8 character (e.g. a paper title containing an accented character, Chinese author name, or em-dash), this will panic at runtime.

**Fix:**
```rust
let title_display = if entry.title.chars().count() > 50 {
    format!("{}\u{2026}", entry.title.chars().take(50).collect::<String>())
} else {
    entry.title.clone()
};
```
The same pattern is used in `node_tooltip` in `graph.rs:1133` (`&node.title[..60]`) and should receive the same fix.

### WR-03: Betweenness normalization is incorrect for `n == 2`

**File:** `resyn-core/src/graph_analytics/betweenness.rs:65`
**Issue:** When `n == 2`, the normalization constant is set to `1.0` with the comment "special case, n<=2". For a 2-node directed graph there are no intermediate nodes, so the raw betweenness of both nodes will be `0.0`, and dividing by `1.0` is harmless. However, `n == 1` is also handled by the same arm (`if n > 2 { ... } else { 1.0 }`): when `n == 1`, the `nodes` loop runs once (for node `s`), the inner loop has no edges, and `centrality[s]` stays `0.0`, so the division by `1.0` is again harmless. The edge case is correct in practice, but the guard should also account for the `n == 0` case which is handled by the early return above, so the overall logic is sound. However, the test comment at line 192–193 asserts "neither can be 'between' two other nodes" which is accurate for `n == 2`, but the normalization formula `(n-1)(n-2)` would give `(1)(0) == 0` for `n == 2` — dividing by zero. The current code avoids this with the `else { 1.0 }` branch, but this exception is undocumented and fragile if the normalization formula is ever revised to apply differently.

**Fix:** Add an explicit comment and guard:
```rust
// For n <= 2: (n-1)(n-2) == 0, which would be division by zero.
// All raw betweenness values are 0 for n <= 2 anyway, so norm = 1.0 is safe.
let norm = if n > 2 { ((n - 1) * (n - 2)) as f32 } else { 1.0 };
```
This is already the code; the fix is to add the comment so the intent is preserved on maintenance.

### WR-04: `get_all_metrics` returns `RankedPaperEntry` with empty `title` and `year` — misleading type reuse

**File:** `resyn-app/src/server_fns/metrics.rs:113-120`
**Issue:** `get_all_metrics` returns `Vec<RankedPaperEntry>` where `title` and `year` are always `String::new()`. The function is used downstream (`get_metrics_pairs` is the correct replacement) but `get_all_metrics` is still exported and could be called by future code expecting a fully populated `RankedPaperEntry`. The `RankedPaperEntry` struct has no `Option<String>` to signal missing fields, so callers cannot distinguish "title was not fetched" from "title is empty".

`get_all_metrics` is currently used only in `get_all_betweenness` (which maps to `(arxiv_id, betweenness)` tuples). Both functions call `metrics_repo.get_all_metrics()` independently — this is a redundant DB round-trip pattern.

**Fix (short-term):** Document the empty fields explicitly:
```rust
/// NOTE: `title` and `year` are always empty in this response.
/// Use `get_top_pagerank_papers` for entries with full paper metadata.
#[server(GetAllMetrics, "/api")]
pub async fn get_all_metrics() -> Result<Vec<RankedPaperEntry>, ServerFnError> {
```
**Fix (long-term):** Introduce a dedicated `MetricsEntry { arxiv_id, pagerank, betweenness }` struct that does not imply paper metadata, and remove `get_all_metrics` / `get_all_betweenness` in favour of `get_metrics_pairs`.

---

## Info

### IN-01: `resyn-app/src/pages/graph.rs` — `metrics_computing` is unused dead state

**File:** `resyn-app/src/pages/graph.rs:119`
**Issue:** `let metrics_computing: RwSignal<bool> = RwSignal::new(false);` is declared and passed to `GraphControls`, but is never set to anything other than `false`. This is directly related to WR-01 above. While not a bug in isolation, it is dead state that adds noise to the signal graph and should either be wired up or removed if the recompute-spinner feature is deferred.

### IN-02: Similarity edges drawn twice in Canvas2D mode

**File:** `resyn-app/src/graph/canvas_renderer.rs:183` and `resyn-app/src/pages/graph.rs:887`
**Issue:** Similarity edges are drawn in `Canvas2DRenderer::draw` (step 6.5, line 183) and also in the label overlay pass in `graph.rs` (lines 887–920) when `show_similarity` is true. The comment at `graph.rs:883` says "This is the rendering path for WebGL2 mode", but the Canvas2D renderer also calls its own draw pass for similarity edges. In Canvas2D mode both passes run, so similarity edges are drawn twice per frame with the same style on overlapping canvases. The visual effect may be invisible (both canvases are composited) but it is wasted work and could cause z-order artifacts on some browsers.

**Fix:** Guard the overlay similarity pass to only run in WebGL2 mode, or remove the Canvas2D-renderer pass and rely solely on the overlay canvas.

### IN-03: `setup_resize_observer` leaks the `ResizeObserver` and closure permanently

**File:** `resyn-app/src/pages/graph.rs:523-524`
**Issue:** Both `cb` and `observer` are leaked via `std::mem::forget` with the comment "Leak both to keep alive for page lifetime." This is intentional but the observer is never disconnected on `on_cleanup`. If the `GraphPage` component is unmounted and remounted (e.g., SPA navigation), each mount leaks a new observer and closure.

**Fix:** Store `observer` and `cb` in a cleanup-safe wrapper, or use `on_cleanup` to call `observer.disconnect()` and drop the closure:
```rust
let observer = Rc::new(web_sys::ResizeObserver::new(...).unwrap());
let observer_clone = observer.clone();
on_cleanup(move || observer_clone.disconnect());
observer.observe(canvas);
std::mem::forget(cb); // closure must still outlive observer
```

---

_Reviewed: 2026-04-09T10:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
