---
phase: 24
slug: community-detection
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-10
---

# Phase 24 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust standard) |
| **Config file** | none — existing test harness |
| **Quick run command** | `cargo test` |
| **Full suite command** | `cargo test --all-targets` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test`
- **After every plan wave:** Run `cargo test --all-targets`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 24-01-01 | 01 | 1 | COMM-01 | — | N/A | unit | `cargo test community` | ❌ W0 | ⬜ pending |
| 24-01-02 | 01 | 1 | COMM-01 | — | N/A | unit | `cargo test louvain` | ❌ W0 | ⬜ pending |
| 24-01-03 | 01 | 2 | COMM-01 | — | N/A | integration | `cargo test db_community` | ❌ W0 | ⬜ pending |
| 24-02-01 | 02 | 2 | COMM-02 | — | N/A | unit | `cargo test color_mode` | ❌ W0 | ⬜ pending |
| 24-03-01 | 03 | 3 | COMM-03 | — | N/A | manual | see Manual-Only | — | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/community/mod.rs` — stub module with unit test stubs for COMM-01
- [ ] `src/community/louvain.rs` — stub with `#[cfg(test)]` stubs
- [ ] `src/database/community_queries.rs` — stub with DB test stubs

*If none: "Existing infrastructure covers all phase requirements."*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Community summary panel shows correct top papers, keywords, methods | COMM-03 | UI/visual interaction | Open app, run crawl, open community panel, verify content accuracy |
| Community color legend click opens drawer | COMM-03 | UI/egui interaction | Click legend chip, confirm drawer opens with community summary |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
