---
phase: 9
slug: graph-renderer-canvas-to-webgl
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-17
---

# Phase 9 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust unit tests) |
| **Config file** | Cargo.toml workspace — no additional config needed |
| **Quick run command** | `cargo test graph` |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test graph`
- **After every plan wave:** Run `cargo test --workspace`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 09-01-01 | 01 | 0 | GRAPH-01 | unit | `cargo test graph::layout_state` | ❌ W0 | ⬜ pending |
| 09-01-02 | 01 | 0 | GRAPH-02 | unit | `cargo test graph::interaction::viewport` | ❌ W0 | ⬜ pending |
| 09-01-03 | 01 | 0 | GRAPH-02 | unit | `cargo test graph::interaction::hit_test` | ❌ W0 | ⬜ pending |
| 09-01-04 | 01 | 0 | GRAPH-04 | unit | `cargo test resyn_worker::barnes_hut` | ❌ W0 | ⬜ pending |
| 09-01-05 | 01 | 0 | GRAPH-04 | unit | `cargo test resyn_worker::convergence` | ❌ W0 | ⬜ pending |
| 09-02-01 | 02 | 1 | GRAPH-01 | unit | `cargo test graph::canvas_renderer` | ❌ W0 | ⬜ pending |
| 09-02-02 | 02 | 1 | GRAPH-01 | manual | browser: trunk serve + inspect canvas | N/A | ⬜ pending |
| 09-03-01 | 03 | 2 | GRAPH-02 | unit | `cargo test graph::interaction` | ❌ W0 | ⬜ pending |
| 09-03-02 | 03 | 2 | GRAPH-02 | manual | browser: pan/zoom/click verification | N/A | ⬜ pending |
| 09-04-01 | 04 | 3 | GRAPH-03 | manual | browser: WebGL2 rendering at 1000 nodes | N/A | ⬜ pending |
| 09-04-02 | 04 | 3 | GRAPH-03 | unit | `cargo test graph::webgl_renderer` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `resyn-worker/` crate — create with gloo-worker dependency, basic worker registration, Barnes-Hut stubs
- [ ] `resyn-app/src/graph/mod.rs` — module scaffold with layout_state, interaction, renderer submodules
- [ ] `resyn-app/src/graph/layout_state.rs` — NodeState, EdgeData, GraphState structs with tests
- [ ] `resyn-app/src/graph/interaction.rs` — Viewport, hit_test with unit tests
- [ ] `resyn-app/src/server_fns/graph.rs` — GraphData, GraphNode, GraphEdge types
- [ ] Worker spike: compile and load minimal gloo-worker via Trunk to validate build pipeline

*Existing infrastructure: cargo test, no new framework install needed*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Canvas2D graph renders with nodes/edges visible | GRAPH-01 | Canvas drawing requires browser DOM | `trunk serve`, navigate to /graph, verify nodes render |
| WebGL2 renderer draws circles at 1000+ nodes | GRAPH-03 | WebGL2 requires GPU context | Load 1000-node dataset, verify WebGL2 context used in console |
| Pan/zoom interactions feel responsive | GRAPH-02 | Subjective interaction quality | Scroll to zoom, drag to pan, verify smooth response |
| Node click opens paper drawer | GRAPH-02 | DOM event + Leptos signal integration | Click a node, verify drawer opens with correct paper |
| Force layout converges visually | GRAPH-04 | Visual convergence vs numeric convergence | Load graph, observe nodes settle within ~10s |
| Contradiction edges red, ABC-bridge edges orange/dashed | GRAPH-01 | Visual color/style verification | Load graph with gap findings, inspect edge rendering |
| 30+ fps at 1000 nodes with WebGL2 | GRAPH-03 | Performance requires real GPU workload | DevTools Performance tab during 1000-node render |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
