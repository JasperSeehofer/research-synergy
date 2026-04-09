---
phase: 22
slug: paper-similarity-engine
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-09
---

# Phase 22 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | Cargo.toml (workspace) |
| **Quick run command** | `cargo test --lib` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 22-01-01 | 01 | 1 | SIM-01 | — | N/A | unit | `cargo test similarity` | ❌ W0 | ⬜ pending |
| 22-01-02 | 01 | 1 | SIM-01 | — | N/A | integration | `cargo test --test db` | ✅ | ⬜ pending |
| 22-02-01 | 02 | 2 | SIM-02 | — | N/A | unit | `cargo check --all-targets` | ✅ | ⬜ pending |
| 22-02-02 | 02 | 2 | SIM-03 | — | N/A | unit | `cargo check --all-targets` | ✅ | ⬜ pending |
| 22-03-01 | 03 | 2 | SIM-04 | — | N/A | integration | `cargo test` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Similarity computation unit tests — stubs for SIM-01
- [ ] DB integration test for similarity storage/retrieval

*Existing infrastructure covers test framework requirements.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Similar Papers tab displays ranked list with shared keywords | SIM-02 | Visual UI verification | Open drawer, check Similar tab shows ranked papers with scores and keywords |
| Similarity edge overlay renders dashed amber lines | SIM-03 | Visual rendering | Toggle overlay in graph controls, verify dashed amber edges appear |
| Force model swap animation on mode toggle | SIM-03 | Visual animation | Toggle to similarity-only, verify graph re-simulates with new layout |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
