---
phase: 16
slug: edge-and-node-renderer-fixes
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-25
---

# Phase 16 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust) + browser visual inspection |
| **Config file** | `Cargo.toml` (workspace) |
| **Quick run command** | `cargo test -p resyn-app` |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p resyn-app`
- **After every plan wave:** Run `cargo test --workspace`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 16-01-01 | 01 | 1 | NODE-01 | unit+compile | `cargo test -p resyn-app` | ✅ | ⬜ pending |
| 16-01-02 | 01 | 1 | NODE-02 | unit+compile | `cargo test -p resyn-app` | ✅ | ⬜ pending |
| 16-01-03 | 01 | 1 | NODE-03 | unit+compile | `cargo test -p resyn-app` | ✅ | ⬜ pending |
| 16-02-01 | 02 | 1 | EDGE-01 | unit+compile | `cargo test -p resyn-app` | ✅ | ⬜ pending |
| 16-02-02 | 02 | 1 | EDGE-02 | compile | `cargo test -p resyn-app` | ✅ | ⬜ pending |
| 16-02-03 | 02 | 1 | EDGE-03 | compile | `cargo test -p resyn-app` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

*Existing infrastructure covers all phase requirements.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Edge visibility on dark background | EDGE-01 | Visual perception test — contrast requires human eye | Load graph, verify edges visible at default zoom on #0d1117 background |
| Node border sharpness at zoom | NODE-02 | Sub-pixel rendering quality requires visual inspection | Zoom in on nodes, verify borders remain crisp without blur |
| Seed node distinction | NODE-03 | Visual distinction is a perceptual judgment | Load graph, verify seed node immediately identifiable (amber + ring) |
| Canvas 2D / WebGL2 consistency | EDGE-03 | Requires testing with different node counts to trigger both renderers | Test with <300 nodes (Canvas 2D) and 300+ nodes (WebGL2), compare appearance |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
