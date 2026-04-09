---
phase: 23
slug: graph-analytics-centrality-metrics
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-09
---

# Phase 23 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | `Cargo.toml` — workspace test config |
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
| 23-01-01 | 01 | 1 | GANA-01 | — | N/A | unit | `cargo test --lib page_rank` | ❌ W0 | ⬜ pending |
| 23-01-02 | 01 | 1 | GANA-02 | — | N/A | unit | `cargo test --lib betweenness` | ❌ W0 | ⬜ pending |
| 23-01-03 | 01 | 1 | GANA-01 | — | N/A | unit | `cargo test --lib corpus_fingerprint` | ❌ W0 | ⬜ pending |
| 23-02-01 | 02 | 2 | GANA-04 | — | N/A | unit | `cargo test --lib get_cited` | ✅ | ⬜ pending |
| 23-02-02 | 02 | 2 | GANA-04 | — | N/A | unit | `cargo test --lib get_citing` | ✅ | ⬜ pending |
| 23-03-01 | 03 | 3 | GANA-03 | — | N/A | integration | `cargo test --test graph_metrics` | ❌ W0 | ⬜ pending |
| 23-03-02 | 03 | 3 | GANA-05 | — | N/A | manual | Browser: select "Size by" dropdown | — | ⬜ pending |
| 23-03-03 | 03 | 3 | GANA-06 | — | N/A | manual | Browser: check dashboard card | — | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `resyn-core/src/data_processing/graph_metrics.rs` — test stubs for PageRank and betweenness centrality
- [ ] `resyn-core/src/database/queries.rs` — existing test infrastructure covers N+1 refactor

*Existing test infrastructure covers DB tests. New metric computation tests needed.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| "Size by" dropdown resizes nodes | GANA-05 | Visual rendering in browser | Open graph, select PageRank in "Size by", verify nodes resize |
| "Most Influential Papers" dashboard card | GANA-06 | Dashboard layout and content | Navigate to dashboard, verify 6th card shows top-5 ranked papers |
| Lerp animation on metric switch | D-02 | Animation timing | Switch between Size by options, verify smooth ~300ms transition |
| Disabled dropdown options when computing | D-10 | UI state during computation | Clear cache, trigger crawl, verify PageRank/Betweenness grayed out |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
