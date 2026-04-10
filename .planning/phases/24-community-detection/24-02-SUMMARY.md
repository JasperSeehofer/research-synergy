---
phase: 24-community-detection
plan: "02"
subsystem: resyn-app/graph + resyn-app/components + resyn-app/pages
tags: [community-detection, color-mode, lerp, canvas-renderer, webgl-renderer, leptos]
dependency_graph:
  requires: [COMM-01]
  provides: [COMM-02]
  affects:
    - resyn-app/src/graph/layout_state.rs
    - resyn-app/src/graph/canvas_renderer.rs
    - resyn-app/src/graph/webgl_renderer.rs
    - resyn-app/src/components/graph_controls.rs
    - resyn-app/src/pages/graph.rs
    - resyn-app/src/graph/lod.rs
    - resyn-app/src/graph/viewport_fit.rs
    - resyn-app/src/graph/interaction.rs
    - resyn-app/src/graph/label_collision.rs
tech_stack:
  added: []
  patterns:
    - "ColorMode enum (Community default, BfsDepth, Topic) mirrors SizeMode precedent from Phase 23"
    - "compute_target_color free function shared by both Canvas2D and WebGL2 renderers"
    - "advance_color_lerp exponential decay — 1.0 - (0.05).powf(dt_ms/300.0)"
    - "color_lerp_pending flag prevents RAF loop extra work after convergence"
    - "Resource::new + Effect::new pattern for community data fetching (matches existing codebase style)"
    - "PendingCommunityDrawerOpen newtype context — Plan 03 consumes to open drawer panel"
    - "Interaction overrides (dimmed/hover/selected) layer on top of data-driven base fill"
key_files:
  created: []
  modified:
    - resyn-app/src/graph/layout_state.rs
    - resyn-app/src/graph/canvas_renderer.rs
    - resyn-app/src/graph/webgl_renderer.rs
    - resyn-app/src/components/graph_controls.rs
    - resyn-app/src/pages/graph.rs
    - resyn-app/src/graph/lod.rs
    - resyn-app/src/graph/viewport_fit.rs
    - resyn-app/src/graph/interaction.rs
    - resyn-app/src/graph/label_collision.rs
decisions:
  - "compute_target_color placed in layout_state.rs (not a separate color.rs) — both renderers already import layout_state, no new module needed"
  - "Color lerp uses assumed dt_ms=16.67 (60fps) rather than actual timestamps — matches existing radius lerp approach"
  - "Topic mode returns ORPHAN_COLOR from compute_target_color — the renderers already handle topic-ring coloring independently via the palette; the ColorMode::Topic path is a pass-through placeholder"
  - "Community data synced into GraphState inside RAF loop (not via Effect) — avoids RefCell borrow conflicts between Leptos reactive graph and mutable RenderState"
  - "Seed gold fill (#d29922) removed from both renderers per plan D-12; seed ring decoration (gold annulus) preserved in both Canvas2D and WebGL2"
  - "create_local_resource not available in Leptos 0.8 — used Resource::new + Effect::new instead (same as existing metrics pattern)"
metrics:
  completed_date: "2026-04-10"
  tasks_completed: 3
  files_modified: 9
---

# Phase 24 Plan 02: Community Color Mode UI

Deliver the data-driven node fill pipeline (ColorMode), 300ms color lerp, "Color by" dropdown, community legend, and end-to-end signal wiring from Plan 01's server fns to the graph renderers.

**One-liner:** ColorMode enum + 300ms exponential lerp pipeline wired end-to-end from community server fns through GraphState to Canvas2D and WebGL2 renderers, with "Color by" dropdown and community swatch legend in graph controls.

## What Was Built

### Task 1: ColorMode enum + NodeState color lerp + palette constants

**resyn-app/src/graph/layout_state.rs** (modified):

- `COMMUNITY_PALETTE: [[u8; 3]; 10]` — 10-slot categorical palette (UI-SPEC exact hex)
- `OTHER_COMMUNITY_COLOR: [u8; 3]` — neutral dark gray for Other bucket
- `BFS_DEPTH_COLORS: [[u8; 3]; 5]` — warm→cool depth scale
- `ORPHAN_COLOR: [u8; 3]` — dark gray for unassigned nodes
- `community_color_for_index(idx: u32) -> [u8; 3]` — cycling with u32::MAX → Other sentinel
- `bfs_depth_color(depth: Option<u32>) -> [u8; 3]` — None → ORPHAN_COLOR, clamps at depth 4
- `u8_rgb_to_f32(rgb: [u8; 3]) -> [f32; 3]` — channel conversion helper
- `advance_color_lerp(current, target, dt_ms)` — exponential lerp per UI-SPEC §5 formula
- `compute_target_color(node, color_mode, community_color_indices, _communities) -> [f32; 3]` — shared by both renderers
- `ColorMode` enum: `#[default] Community`, `BfsDepth`, `Topic` with `as_str/from_str`
- `NodeState` extended: `current_color: [f32; 3]`, `target_color: [f32; 3]`, `community_id: Option<u32>`
- `GraphState` extended: `color_mode: ColorMode`, `communities: HashMap<String, u32>`, `community_color_indices: HashMap<u32, u32>`, `color_lerp_pending: bool`
- `GraphState::update_node_target_colors()` — recomputes all node target colors from current mode
- Fixed NodeState initializers in: `lod.rs`, `viewport_fit.rs`, `interaction.rs`, `label_collision.rs`

**Tests added (8):** `test_color_mode_default_is_community`, `test_community_color_index_mapping`, `test_bfs_depth_color_seed`, `test_bfs_depth_color_clamp`, `test_bfs_depth_color_none`, `test_color_lerp_advances_toward_target`, `test_color_lerp_converges`, `test_community_summaries_to_color_indices_map`, plus 3 `compute_target_color` tests (community_known, bfs_depth_seed, community_unknown).

### Task 2: Canvas2D + WebGL2 renderer refactor

**canvas_renderer.rs:**
- Added `f32_rgb_to_css(c: [f32; 3]) -> String` helper
- Replaced hardcoded fill branches with `f32_rgb_to_css(node.current_color)` as base
- Interaction overrides (dimmed → `#2a3a4f`, hover/selected → `#58a6ff`) layered on top
- Seed gold fill (`#d29922`) removed from fill logic; gold ring decoration preserved

**webgl_renderer.rs:**
- Per-instance color now reads `node.current_color` (f32 tuple) instead of hardcoded hex
- Same interaction override structure as Canvas2D
- Seed gold fill removed from fill logic; seed ring annulus preserved

**pages/graph.rs (RAF loop additions):**
- `color_mode: RwSignal<ColorMode>` signal (Community default)
- `prev_color_mode` tracker detects signal changes → calls `update_node_target_colors()` + sets `color_lerp_pending = true`
- Per-frame `advance_color_lerp()` applied to all nodes when `color_lerp_pending`
- `color_lerp_pending` cleared when max channel delta < 0.01 (Pitfall 6 guard)
- Community data sync block in RAF: when `community_assignments` arrives, applies per-node `community_id`, rebuilds `community_color_indices`, triggers color lerp

### Task 3: Color by dropdown + community legend + signal wiring

**pages/graph.rs:**
- `community_status: RwSignal<Option<CommunityStatus>>` — drives communities_ready derived signal
- `community_summaries: RwSignal<Vec<CommunitySummary>>` — for legend rendering
- `communities_computing: RwSignal<bool>` — for future Plan 03 recompute button
- `community_assignments_signal: RwSignal<HashMap<String, u32>>` — per-paper community map
- `pending_community_drawer_open: RwSignal<Option<u32>>` — Plan 03 drawer context
- `PendingCommunityDrawerOpen` newtype context provided via `provide_context`
- `communities_ready = Signal::derive(...)` — true when status.ready
- Three `Resource::new` fetches (status, summaries, assignments) + `Effect::new` hooks to populate signals
- `GraphControls` call extended with 4 new props

**graph_controls.rs:**
- Imports `ColorMode`, `community_color_for_index` from layout_state
- New params: `color_mode`, `community_summaries`, `communities_ready`, `communities_computing`
- `PendingCommunityDrawerOpen` context consumed via `expect_context`
- "Color by" `<select>` with three `<option>` values; Community option disabled with "(computing…)" text when not ready
- Community legend section: conditionally rendered when `ColorMode::Community` and summaries non-empty; each entry has swatch + label + aria-label; click sets `PendingCommunityDrawerOpen`
- Topic Colors legend preserved and fully independent (D-10)

## Renderer Refactor Diff Summary

### Branches removed
- `canvas_renderer.rs`: `else if node.is_seed { "#d29922" } else { "#4a9eff" }` branches removed
- `webgl_renderer.rs`: same two branches removed

### Helpers added
- `f32_rgb_to_css` in canvas_renderer.rs
- `compute_target_color` in layout_state.rs (shared path)

### ColorMode signal data flow

```
get_community_status()        ─┐
get_all_community_summaries() ─┼─ Resource::new + Effect → community_status/summaries signals
get_community_assignments()   ─┘

community_assignments signal ─→ RAF loop sync block
                               ─→ node.community_id populated for all nodes
                               ─→ GraphState.community_color_indices rebuilt
                               ─→ update_node_target_colors() called
                               ─→ color_lerp_pending = true

color_mode RwSignal ──────────→ RAF loop detects change
                               ─→ update_node_target_colors() (mode switch)
                               ─→ color_lerp_pending = true

color_lerp_pending = true ────→ advance_color_lerp() per node per frame
                               ─→ node.current_color moves toward target_color
                               ─→ renderer reads current_color for fill
                               ─→ color_lerp_pending = false when all channels < 0.01 delta
```

## Plan 01 Amendments

`get_community_assignments` was already included in Plan 01's delivery (see 24-01-SUMMARY.md). No amendments needed.

## Deviations from Plan

### Auto-fix: create_local_resource not available in Leptos 0.8

**Found during:** Task 3 implementation
**Issue:** The plan used `create_local_resource(|| (), move |_| async move { ... })` which does not exist in Leptos 0.8.
**Fix:** Used `Resource::new(|| (), |_| get_fn())` + `Effect::new(move |_| { ... })` pattern — identical to how `metrics_status_resource` and `metrics_pairs_resource` are wired in the existing codebase.
**Files modified:** `resyn-app/src/pages/graph.rs`
**Commits:** db28af5

### Auto-fix: NodeState struct initializers in 4 test helper files

**Found during:** Task 1 — adding `current_color`, `target_color`, `community_id` fields to `NodeState`
**Issue:** Rust requires all struct fields to be explicitly initialized; 4 files had `NodeState { ... }` initializers that needed updating.
**Fix:** Added `current_color: [0.0; 3], target_color: [0.0; 3], community_id: None` to each initializer in `lod.rs`, `viewport_fit.rs`, `interaction.rs`, `label_collision.rs`.
**Files modified:** All 4 files listed above
**Commits:** bac7b2e

### Design decision: Community data sync in RAF loop vs. Effect

**Found during:** Task 3 — considering where to apply community assignments to GraphState
**Issue:** GraphState lives inside `Rc<RefCell<RenderState>>` which is already borrowed during the RAF loop. An `Effect` outside the loop cannot safely borrow it at the same time.
**Decision:** Community data sync placed inside the RAF loop body (same pattern as metrics sync block). This avoids RefCell conflicts and keeps all GraphState mutations in one location.

## Known Stubs

- **ColorMode::Topic** in `compute_target_color`: returns `ORPHAN_COLOR` as placeholder. Topic coloring is already implemented via the topic-ring overlay canvas (Phase 999.2) — the Topic ColorMode path in node fill is a pass-through. The topic ring palette (PaletteEntry) is independent of node fill color. This is intentional: the two systems (topic rings vs. node fill) operate independently per D-10.

## Threat Flags

No new network endpoints, auth paths, or file access patterns introduced. All community data flows through existing Plan 01 server fns (read-only DB queries). `ColorMode::from_str` is total — unknown values fall through to Community default (T-24-09 mitigated).

## Commits

| Hash | Message |
|------|---------|
| bac7b2e | feat(24-02): add ColorMode enum + palette constants + color lerp |
| 88fce44 | feat(24-02): refactor node fill pipeline for ColorMode + 300ms lerp |
| db28af5 | feat(24-02): Color by dropdown + community legend + signal wiring |

## Self-Check: PASSED

- resyn-app/src/graph/layout_state.rs: FOUND (ColorMode, COMMUNITY_PALETTE, compute_target_color, advance_color_lerp)
- resyn-app/src/graph/canvas_renderer.rs: FOUND (f32_rgb_to_css, ColorMode fill path)
- resyn-app/src/graph/webgl_renderer.rs: FOUND (node.current_color used for instance data)
- resyn-app/src/components/graph_controls.rs: FOUND (Color by, community legend, COMMUNITY COLORS)
- resyn-app/src/pages/graph.rs: FOUND (color_mode, community_status, community_assignments, PendingCommunityDrawerOpen)
- cargo check -p resyn-app (WASM): PASSED
- cargo check -p resyn-app --features ssr: PASSED
- cargo test -p resyn-app: 107/107 PASSED
- ColorMode default is Community: CONFIRMED
- Interaction overrides preserved: CONFIRMED (dimmed/hover/selected branches in both renderers)
- Seed ring decoration preserved: CONFIRMED (gold annulus in both renderers unchanged)
- Topic Rings toggle independent: CONFIRMED (topic-legend-section untouched)
- get_community_assignments already in Plan 01: CONFIRMED (no amendments needed)
