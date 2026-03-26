---
phase: 17
slug: viewport-fit-and-label-collision
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-25
---

# Phase 17 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | Cargo.toml (workspace) |
| **Quick run command** | `cargo test --lib` |
| **Full suite command** | `cargo test --all-targets` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib`
- **After every plan wave:** Run `cargo test --all-targets`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 17-01-01 | 01 | 1 | VIEW-01 | unit | `cargo test viewport_fit` | ❌ W0 | ⬜ pending |
| 17-01-02 | 01 | 1 | VIEW-02 | unit | `cargo test user_interaction_latch` | ❌ W0 | ⬜ pending |
| 17-02-01 | 02 | 1 | LABEL-01 | unit | `cargo test label_collision` | ❌ W0 | ⬜ pending |
| 17-02-02 | 02 | 1 | LABEL-02 | unit | `cargo test label_priority` | ❌ W0 | ⬜ pending |
| 17-03-01 | 03 | 2 | VIEW-01 | manual | browser viewport check | N/A | ⬜ pending |
| 17-03-02 | 03 | 2 | LABEL-01 | manual | browser label visual check | N/A | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `resyn-app/src/graph/viewport_fit.rs` — test stubs for bounding box fit math (VIEW-01)
- [ ] `resyn-app/src/graph/label_collision.rs` — test stubs for collision avoidance logic (LABEL-01, LABEL-02)

*Existing cargo test infrastructure covers framework needs. No new test framework install required.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Auto-fit animation smoothness | VIEW-01 | Visual quality — lerp timing cannot be unit tested | Load graph, wait for convergence, verify smooth animated zoom-to-fit |
| User override latch persistence | VIEW-02 | Requires mouse interaction in browser | Pan/zoom after convergence, verify auto-fit does not re-trigger |
| Label pill rendering appearance | LABEL-01 | Visual styling cannot be tested programmatically | Zoom to medium level, verify pill badges render with correct colors/borders |
| Hover label reveal | LABEL-02 | Requires mouse hover interaction | Hover over culled-label node, verify label appears |
| Convergence status badge states | VIEW-01 | UI state display in browser | Observe badge transitions: Simulating → Settled; pause → Paused |
| Fit button manual trigger | VIEW-02 | Requires button click interaction | Click Fit button after manual pan, verify viewport resets |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
