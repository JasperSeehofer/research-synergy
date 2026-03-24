# Bug Fix Status (2026-03-23)

## Context

Browser exploration with agent-browser revealed critical issues in the v1.1 web UI.
Bug fixes were attempted in this session. Some fixes are confirmed working, others need further investigation.

## Confirmed Working (verified via agent-browser + API)

### 1. Graph citation edges restored
- **Problem:** Graph showed 351+ nodes with zero edges (spiral pattern, no connections)
- **Root cause:** `PaperRecord::to_paper()` hardcodes `references: Vec::new()` (queries.rs:78). Graph builder relied on `Paper.references` which was always empty from DB.
- **Fix:** Added `PaperRepository::get_all_citations()` method that queries `cites` relations directly from SurrealDB. Rewrote `get_graph_data()` to use DB edges instead of `Paper.references`.
- **Verification:** API returns 720 edges for 375 nodes. Graph shows connected clustering.
- **Files:** `resyn-core/src/database/queries.rs`, `resyn-app/src/server_fns/graph.rs`

### 2. SSE crawl progress endpoint
- **Problem:** Sidebar always showed "No active crawl" even during crawl. No `/progress` SSE endpoint on web server.
- **Root cause:** SSE only existed in CLI `crawl` command. Web server's `serve.rs` had no `/progress` route. `start_crawl()` background task never broadcast events.
- **Fix:** Added `broadcast::channel<ProgressEvent>` to serve.rs, registered `/progress` SSE GET route, provided sender via `provide_context`. Modified `start_crawl()` to broadcast `paper_fetched`, `paper_failed`, and `complete` events.
- **Verification:** Screenshot showed "Crawl in progress" with live stats (17 found, depth 1/1, current paper title).
- **Files:** `resyn-server/src/commands/serve.rs`, `resyn-app/src/server_fns/papers.rs`

### 3. "Analyzed" badge renamed to "Crawled"
- **Problem:** Papers showed "Analyzed" badge based on `citation_count.is_some()`, but no LLM analysis was actually run.
- **Fix:** Changed `status_str()` to return "Crawled" instead of "Analyzed". Updated CSS class.
- **Verification:** Papers table shows "Crawled" badge.
- **Files:** `resyn-app/src/pages/papers.rs`, `resyn-app/style/main.css`

---

## NOT Working / Needs Further Investigation

### 4. Temporal slider thumbs behind track
- **Problem:** One thumb always renders behind the other input's track in the dual-range slider.
- **Attempted fix:** Made tracks transparent, added `::before` pseudo-element as shared visible track, min input z-index 2.
- **Status:** UNCLEAR — screenshots show thumbs but user reports still broken. May need testing on actual browser (not just headless).
- **Files:** `resyn-app/style/main.css` (lines ~1453-1472), `resyn-app/src/components/graph_controls.rs`

### 5. Fuzzy/blurry graph nodes
- **Problem:** Graph nodes appear with fuzzy edges, possibly DPR mismatch in WebGL renderer.
- **Attempted fix:** Divided `self.width/height` by `devicePixelRatio()` in WebGL renderer's resolution uniforms.
- **Status:** NOT VERIFIED by user. The fix changes coordinate mapping — may have introduced other visual issues. Needs user verification.
- **Files:** `resyn-app/src/graph/webgl_renderer.rs` (line ~161)

### 6. Force-directed graph animation not working
- **Problem:** Graph nodes don't animate — they appear in static positions with no visible force simulation.
- **Root cause (confirmed):** The original `poll_bridge_sync()` used a noop waker, so the gloo-worker `ReactorBridge` stream never received wake notifications. `poll_next()` always returned `Pending`, meaning worker outputs were never read.
- **Attempted fixes (3 iterations):**
  1. `spawn_local` with `futures::poll!` — same noop waker problem
  2. `futures::stream::poll_fn` wrapper — borrow issues, still didn't work
  3. **Current:** Inline `run_ticks()` on main thread (bypasses worker entirely)
- **Current state:** RAF loop IS running (confirmed via document.title debug: `F31 sim=true n=414 a=0.8604`). `run_ticks()` is being called inline. Simulation IS computing new positions. But user reports **no visible animation and cannot interact with nodes**.
- **Possible remaining issues:**
  - Force parameters may need tuning (currently: repulsion=-200, attraction=0.3, damping=0.4, ideal_distance=80, jitter=±40% spread)
  - Simulation may converge too fast (settled before user sees it)
  - Node interaction (drag/pan/zoom) may be broken — mousedown/mouseup/mousemove event listeners need verification
  - Canvas event listeners may not be attaching correctly after the code changes
- **Files:**
  - `resyn-app/src/pages/graph.rs` — RAF loop, inline `run_ticks()`, event listeners
  - `resyn-worker/src/forces.rs` — force constants
  - `resyn-app/src/graph/layout_state.rs` — initial jitter
  - `resyn-app/src/graph/worker_bridge.rs` — bridge (currently unused for output)

### 7. Node interaction (drag/pan/zoom) not working
- **Problem:** User reports cannot move nodes or interact with graph.
- **Possible causes:**
  - Event listeners may not be attaching to the canvas element
  - The `InteractionState` enum transitions (Idle → DraggingNode/Panning) may have a bug
  - The canvas element may be covered by another element (z-index issue)
  - The viewport coordinate transformation may be broken after the DPR fix
- **Status:** Code looks correct on inspection but NOT verified working.
- **Files:** `resyn-app/src/pages/graph.rs` (attach_event_listeners ~line 477), `resyn-app/src/graph/interaction.rs`

---

## Code Changes Made This Session

### New code
- `resyn-core/src/database/queries.rs` — `get_all_citations()` method
- `resyn-server/src/commands/serve.rs` — SSE `/progress` endpoint + broadcast channel
- `resyn-app/src/pages/graph.rs` — inline force layout, spawn_local output buffer (unused)

### Modified code
- `resyn-app/src/server_fns/graph.rs` — rewritten `get_graph_data()` to use DB edges
- `resyn-app/src/server_fns/papers.rs` — `start_crawl()` broadcasts progress events
- `resyn-app/src/pages/papers.rs` — "Crawled" badge
- `resyn-app/style/main.css` — slider z-index, track transparency
- `resyn-app/src/graph/webgl_renderer.rs` — DPR fix for resolution uniforms
- `resyn-app/src/graph/layout_state.rs` — jitter on initial positions
- `resyn-worker/src/forces.rs` — tuned force constants, updated convergence test
- `resyn-app/Cargo.toml` — added `gloo-timers` dep

### Dependencies added
- `gloo-timers = { version = "0.3", features = ["futures"] }` in resyn-app

---

## Recommended Next Steps

1. **Debug node interaction** — Add console logging to mousedown/mouseup handlers to verify events fire. Check if canvas has `pointer-events: none` or is covered by an overlay div.
2. **Debug force animation** — Add a visible counter or position delta log to confirm nodes actually move between frames. The simulation runs but positions may not change enough to be visible.
3. **Test DPR fix** — The WebGL resolution uniform change may have broken the coordinate system. Compare node positions before/after the DPR change.
4. **Consider reverting WebGL DPR fix** — If it broke interactions, the screen_to_world coordinate conversion in viewport may no longer match the WebGL coordinate space.
5. **Test slider** — Try dragging both thumbs independently in the actual browser.
