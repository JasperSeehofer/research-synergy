---
phase: 13
slug: graph-interaction
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-23
---

# Phase 13 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust) + agent-browser (visual/interactive) |
| **Config file** | `Cargo.toml` (workspace) |
| **Quick run command** | `cargo test -p resyn-app` |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p resyn-app`
- **After every plan wave:** Run `cargo test --workspace`
- **Before `/gsd:verify-work`:** Full suite must be green + agent-browser confirms drag, pan, zoom
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 13-01-01 | 01 | 1 | INTERACT-01 | visual/manual | `agent-browser` | ❌ W0 | ⬜ pending |
| 13-01-02 | 01 | 1 | INTERACT-02 | visual/manual | `agent-browser` | ❌ W0 | ⬜ pending |
| 13-01-03 | 01 | 1 | INTERACT-03 | visual/manual | `agent-browser` | ❌ W0 | ⬜ pending |

Interaction logic unit tests (7 existing) serve as automated regression guard:
- `cargo test -p resyn-app` covers `test_screen_to_world_*`, `test_zoom_*`, `test_find_node_at_*`, `test_find_edge_at_*`

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements. The 7 interaction unit tests in `interaction.rs` validate logic. Browser verification is manual-only (CSS/event delivery cannot be unit-tested).

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Node drag repositions node | INTERACT-01 | CSS event delivery, visual positioning | Open graph page, click-drag a node, verify it moves and stays |
| Canvas pan via drag | INTERACT-02 | CSS event delivery, visual viewport shift | Drag empty canvas space, verify viewport pans |
| Scroll wheel zoom | INTERACT-03 | CSS event delivery, visual zoom change | Scroll wheel over graph, verify zoom in/out |
| No state corruption after interaction | All | Visual consistency check | After drag/pan/zoom, verify nodes don't jump or reset |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
