---
phase: 12
slug: graph-force-rendering
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-23
---

# Phase 12 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust) + agent-browser (visual) |
| **Config file** | `Cargo.toml` (workspace) |
| **Quick run command** | `cargo test -p resyn-worker` |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p resyn-worker`
- **After every plan wave:** Run `cargo test --workspace`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 12-01-01 | 01 | 1 | GRAPH-01 | unit | `cargo test -p resyn-app` | ✅ | ⬜ pending |
| 12-01-02 | 01 | 1 | GRAPH-02 | visual | `agent-browser screenshot` | ❌ W0 | ⬜ pending |
| 12-01-03 | 01 | 1 | GRAPH-03 | visual | `agent-browser screenshot` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Existing `cargo test` infrastructure covers force simulation unit tests
- [ ] Visual verification via agent-browser for DPR/rendering checks (manual-only)

*Existing infrastructure covers force simulation requirements. Visual rendering requires manual/screenshot verification.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Nodes visibly animate and spread | GRAPH-01 | Visual animation requires browser rendering | Load graph page, observe nodes spreading apart in first few seconds |
| Edges drawn between connected nodes | GRAPH-03 | Visual rendering check | Load graph page, verify lines connecting related papers |
| Crisp rendering at all zoom levels | GRAPH-02 | DPR/pixel quality is visual | Load graph page, zoom in/out, check for blur or aliasing artifacts |
| Simulation settles to stable layout | GRAPH-01 | Animation convergence is visual | Wait ~10s, verify nodes stop moving and don't collapse/explode |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
