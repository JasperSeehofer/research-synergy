---
phase: 14
slug: temporal-controls
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-24
---

# Phase 14 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust) + agent-browser (visual verification) |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo test --lib` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 14-01-01 | 01 | 1 | TEMPORAL-01 | visual/manual | agent-browser inspection | N/A | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements. This is a CSS/handler bugfix — no new test framework needed.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Both slider thumbs visible | TEMPORAL-01 | Visual CSS rendering cannot be unit tested | Load graph page in browser, verify two circular thumbs visible on slider row |
| Each thumb independently draggable | TEMPORAL-01 | Interaction behavior requires browser | Drag min thumb right, verify it moves. Drag max thumb left, verify it moves independently |
| Year range label updates | TEMPORAL-01 | Reactive signal + DOM rendering | Move either thumb, verify "YYYY – YYYY" label updates in real-time |
| Graph filters on slider change | TEMPORAL-01 | End-to-end pipeline: signal → RAF → visibility | Move slider, verify nodes outside range disappear from graph |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
