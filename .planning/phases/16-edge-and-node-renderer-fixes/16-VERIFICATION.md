---
phase: 16-edge-and-node-renderer-fixes
verified: 2026-03-25T00:00:00Z
status: passed
score: 10/10 must-haves verified
re_verification: false
---

# Phase 16: Edge and Node Renderer Fixes — Verification Report

**Phase Goal:** Citation edges are visible at a glance on the dark background and node circles are crisp at all zoom levels, with the seed paper clearly identified
**Verified:** 2026-03-25
**Status:** passed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| #  | Truth | Status | Evidence |
|----|-------|--------|----------|
| 1  | Canvas 2D regular citation edges use color #8b949e with depth-based alpha (0.15-0.5) | VERIFIED | `canvas_renderer.rs:96` `set_stroke_style_str("#8b949e")`, `depth_alpha()` fn at line 323 returning 0.50/0.35/0.25/0.15 by BFS depth, called at line 98 |
| 2  | Canvas 2D node borders are bright (#7cb8ff for regular, #e8b84b for seed) and 1px screen-space at all zoom | VERIFIED | `canvas_renderer.rs:251-253`: `border_color` selects `"#e8b84b"` for seed and `"#7cb8ff"` otherwise; `set_line_width(1.0 / viewport.scale)` |
| 3  | Seed paper node renders with amber #d29922 fill and outer planetary ring in Canvas 2D | VERIFIED | `canvas_renderer.rs:236-265`: `node.is_seed` branch yields `"#d29922"` fill; ring drawn at `node.radius + 2.0 + 1.5` with `3.0 / viewport.scale` line width |
| 4  | NodeState has is_seed field set from GraphState.seed_paper_id | VERIFIED | `layout_state.rs:19`: `pub is_seed: bool`; `layout_state.rs:132-136`: computed from `data.seed_paper_id` before struct literal to avoid move issues |
| 5  | All existing tests compile and pass after is_seed addition | VERIFIED | `cargo test -p resyn-app` exits 0 with 50 tests passing (including 2 new is_seed tests) |
| 6  | WebGL2 regular edges use color #8b949e with depth-based alpha matching Canvas 2D | VERIFIED | `webgl_renderer.rs:610`: `hex_to_rgb("#8b949e")` in `edge_color()`; `depth_alpha_f32()` at line 629 uses identical thresholds 0.50/0.35/0.25/0.15 |
| 7  | WebGL2 edges rendered via quad triangle geometry (6 verts/edge) not GL.LINES | VERIFIED | `webgl_renderer.rs:326` `TRIANGLES` for edge draw; `LINES` constant absent from file; `build_quad_edge()` fn at line 652; `half_width = 0.75 / scale` at line 294 |
| 8  | WebGL2 node fragment shader uses fwidth() for resolution-independent AA | VERIFIED | `webgl_renderer.rs:51`: `float fw = fwidth(d);` in NODE_FRAG; `smoothstep(0.9, 1.0, d)` absent |
| 9  | WebGL2 node borders are bright and adapt to screen-space via fwidth | VERIFIED | NODE_FRAG at lines 41-68 uses `border_blend = smoothstep(border_inner, border_inner + fw, d)` and `v_color * 1.6` brightening; `a_is_seed` attribute plumbed through for seed distinction |
| 10 | Seed node renders amber in WebGL2 with is_seed passed through instance data | VERIFIED | `webgl_renderer.rs:358-361`: `node.is_seed` selects `hex_to_rgb("#d29922")`; `is_seed` packed as float at line 385; stride updated to `8 * 4` at line 408; `a_is_seed` attribute pointer at line 458-468; seed ring triangle annulus drawn via edge shader at line 499 |

**Score:** 10/10 truths verified

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `resyn-app/src/graph/layout_state.rs` | NodeState.is_seed field + from_graph_data sets it | VERIFIED | `pub is_seed: bool` at line 19; set via `data.seed_paper_id` comparison at lines 132-136; two new unit tests at lines 358 and 373 |
| `resyn-app/src/graph/canvas_renderer.rs` | Updated edge color, depth alpha, node border, seed ring | VERIFIED | Contains `#8b949e`, `depth_alpha` fn, `#7cb8ff`/`#e8b84b` borders, seed ring; no `#404040` or `#30363d` |
| `resyn-app/src/graph/webgl_renderer.rs` | Updated shaders, quad edge geometry, seed node support, depth-based alpha | VERIFIED | Contains `fwidth`, `build_quad_edge`, `depth_alpha_f32`, `#8b949e`, `node.is_seed`, stride `8 * 4`; no `#404040`, `#30363d`, or `smoothstep(0.9, 1.0, d)` |
| `resyn-app/src/graph/lod.rs` | make_node test helper includes is_seed: false | VERIFIED | `is_seed: false` at lines 66, 233, 250, 267 |
| `resyn-app/src/graph/interaction.rs` | make_node test helper includes is_seed: false | VERIFIED | `is_seed: false` at line 102 |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `layout_state.rs` | `canvas_renderer.rs` | `node.is_seed` read during node drawing | VERIFIED | `canvas_renderer.rs:236,251,257` all branch on `node.is_seed` |
| `layout_state.rs` | `canvas_renderer.rs` | `bfs_depth` used in `depth_alpha()` | VERIFIED | `depth_alpha()` reads `n.bfs_depth` via `and_then`; called on every regular edge at line 98 |
| `layout_state.rs` | `webgl_renderer.rs` | `node.is_seed` and `bfs_depth` read during buffer construction | VERIFIED | `webgl_renderer.rs:358` and `385` read `node.is_seed`; `depth_alpha_f32()` reads `n.bfs_depth` at line 629 |
| `webgl_renderer.rs` | self (NODE_FRAG) | `fwidth(d)` instead of `smoothstep(0.9, 1.0, d)` | VERIFIED | `fwidth(d)` present at line 51; old pattern absent |
| `webgl_renderer.rs` | self (edge draw) | `TRIANGLES` instead of `LINES` | VERIFIED | `TRIANGLES` at line 326; `LINES` absent from file |

---

### Data-Flow Trace (Level 4)

These are rendering components — data originates from `GraphData` deserialized server-side and flows into `GraphState` via `from_graph_data()`. The rendering path is:

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `canvas_renderer.rs` | `node.is_seed` | `layout_state::from_graph_data()` reading `data.seed_paper_id` | Yes — derived from server-supplied `GraphData.seed_paper_id` | FLOWING |
| `canvas_renderer.rs` | `node.bfs_depth` (via `depth_alpha`) | `GraphData.nodes[i].bfs_depth` passed through | Yes — BFS depth set during server-side graph construction | FLOWING |
| `webgl_renderer.rs` | `node.is_seed` | Same `NodeState.is_seed` via instance buffer at stride offset 7 | Yes — packed as f32 `1.0`/`0.0`, consumed by `a_is_seed` in NODE_VERT | FLOWING |
| `webgl_renderer.rs` | edge `depth_alpha_f32` | `EdgeData.from_idx`/`to_idx` → `nodes[idx].bfs_depth` | Yes — same `bfs_depth` source as canvas | FLOWING |

---

### Behavioral Spot-Checks

This is a WASM/WebGL2 frontend crate — no runnable entry point exists without a browser. Spot-checks are limited to compile-time and test-time verification.

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| All 50 resyn-app tests pass | `cargo test -p resyn-app` | 50 passed; 0 failed | PASS |
| `is_seed` set correctly for seed paper | `test_is_seed_set_for_seed_paper` | assert passes | PASS |
| `is_seed` false when no seed_paper_id | `test_is_seed_false_when_no_seed_id` | assert passes | PASS |
| cargo fmt clean | `cargo fmt --all -- --check` | exits 0, no output | PASS |
| clippy clean with -D warnings | `cargo clippy -p resyn-app -- -D warnings` | exits 0, "Finished" only | PASS |

---

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| EDGE-01 | 16-01, 16-02 | Regular citation edges visible at-a-glance on dark (#0d1117) background | SATISFIED | Canvas 2D: `#8b949e` + depth alpha 0.50-0.15; WebGL2: same color and alpha via `edge_color()` + `depth_alpha_f32()` |
| EDGE-02 | 16-02 | WebGL2 edges rendered via quad-based triangle geometry instead of 1px-capped LINES primitive | SATISFIED | `build_quad_edge()` produces 6 verts/edge; draw mode is `TRIANGLES`; `LINES` absent |
| EDGE-03 | 16-01, 16-02 | Edge color and alpha consistent between Canvas 2D and WebGL2 renderers | SATISFIED | Both renderers use `#8b949e`; both `depth_alpha()` functions use identical thresholds (0.50/0.35/0.25/0.15) with `max(from_depth, to_depth)` logic |
| NODE-01 | 16-02 | Node circles sharp at all sizes using resolution-independent anti-aliasing (fwidth in WebGL2) | SATISFIED | NODE_FRAG uses `fw = fwidth(d)` and `smoothstep(1.0 - fw, 1.0 + fw, d)` mask; old `smoothstep(0.9, 1.0, d)` absent |
| NODE-02 | 16-01, 16-02 | Node borders crisp at all zoom levels (line width scaled by inverse viewport scale) | SATISFIED | Canvas 2D: `1.0 / viewport.scale`; WebGL2: fwidth-based border ring in fragment shader adapts to screen resolution |
| NODE-03 | 16-01, 16-02 | Seed paper node visually distinct with gold/amber color and outer ring | SATISFIED | Both renderers: `#d29922` fill for `is_seed` nodes; outer ring in Canvas 2D (`3.0 / viewport.scale`); triangle annulus ring in WebGL2 via edge shader pass |

All 6 requirements satisfied. REQUIREMENTS.md marks all 6 as `[x] Complete` for Phase 16.

**Orphaned requirements check:** No additional requirement IDs mapped to Phase 16 in REQUIREMENTS.md beyond the 6 declared in the plans.

---

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| — | — | — | — | — |

No anti-patterns found. Specifically confirmed absent:
- No `#404040` or `#30363d` in `canvas_renderer.rs` or `webgl_renderer.rs`
- No `smoothstep(0.9, 1.0, d)` in WebGL2 NODE_FRAG
- No `GL.LINES` draw mode for edges
- No TODO/FIXME/placeholder comments in modified files
- No stub return patterns (`return null`, `return []`, empty handlers)

---

### Human Verification Required

The following behaviors require visual confirmation in a browser (cannot be verified programmatically):

#### 1. Edge Visibility on Dark Background

**Test:** Load a graph with 10+ papers via the UI and observe the citation edges on the dark (#0d1117) canvas background.
**Expected:** Grey edges (#8b949e) are clearly visible without appearing washed out; depth-1 edges appear at ~50% opacity, depth-4+ edges at ~15% (noticeably dimmer but still visible).
**Why human:** Color perception on screen depends on monitor calibration and ambient lighting; cannot assert "visible at a glance" programmatically.

#### 2. Node Crispness at All Zoom Levels

**Test:** Load a graph, then zoom in and out using the scroll wheel or pinch. Observe node circle edges at multiple zoom levels (e.g., 0.3x, 1x, 3x).
**Expected:** Node circles remain sharply anti-aliased at all zoom levels in both Canvas 2D and WebGL2 modes. Borders remain approximately 1px wide regardless of zoom.
**Why human:** Pixel-level AA quality requires visual inspection; fwidth behavior depends on the actual GPU and driver.

#### 3. Seed Node Visual Distinction

**Test:** Load a graph and identify the seed paper node (gold/amber).
**Expected:** The seed paper is immediately recognizable with an amber (#d29922) fill, a brighter amber border (#e8b84b), and a visible outer planetary ring separated by a gap from the node circle. All other nodes appear in blue (#4a9eff) with no ring.
**Why human:** "Clearly identified" is a perceptual judgment; visual salience of the distinction requires human confirmation.

#### 4. Canvas 2D vs WebGL2 Visual Parity

**Test:** Trigger the mode switch between Canvas 2D (<300 nodes) and WebGL2 (300+ nodes) renderers. Compare edge and node appearance.
**Expected:** Edge colors, depth-based alpha dimming, border brightness, and seed node appearance look visually consistent between the two renderers.
**Why human:** Cross-renderer visual parity requires side-by-side human comparison.

---

### Gaps Summary

No gaps found. All automated checks passed:
- 50/50 tests pass including 2 new `is_seed` unit tests
- `cargo fmt --all -- --check` exits 0
- `cargo clippy -p resyn-app -- -D warnings` exits 0
- All key color constants, functions, and wiring verified present in the correct files
- All 6 requirements satisfied with evidence in the codebase
- Both commits referenced in SUMMARY (ad581b7, 96dfd17 for Plan 01; 86ed9f3, 9ce524b for Plan 02) exist in git history

Phase goal is achieved: citation edges are visible on the dark background (correct color + depth alpha in both renderers), node circles are crisp at all zoom levels (fwidth in WebGL2, viewport-compensated line widths in Canvas 2D), and the seed paper is clearly identified (amber fill + border + outer ring in both renderers).

---

_Verified: 2026-03-25_
_Verifier: Claude (gsd-verifier)_
