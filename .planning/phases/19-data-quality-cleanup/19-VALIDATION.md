---
phase: 19
slug: data-quality-cleanup
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-28
---

# Phase 19 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test + tokio::test for async, wiremock for HTTP mocking |
| **Config file** | Cargo.toml (dev-dependencies: wiremock) |
| **Quick run command** | `cargo test inspirehep && cargo test graph_creation && cargo test paper` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test inspirehep && cargo test graph_creation && cargo test paper`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 19-01-01 | 01 | 1 | ARXIV-02 | unit | `cargo test test_convert_hit_to_paper` | ✅ (extend) | ⬜ pending |
| 19-01-02 | 01 | 1 | ARXIV-02 | unit | `cargo test test_convert_hit_with_missing_optional_fields` | ✅ (extend) | ⬜ pending |
| 19-01-03 | 01 | 1 | ARXIV-02 | integration | `cargo test test_inspirehep_fetch_paper_published` | ❌ W0 | ⬜ pending |
| 19-02-01 | 02 | 1 | ORPH-02 | unit | `cargo test test_get_arxiv_references_ids_filters_empty` | ❌ W0 | ⬜ pending |
| 19-02-02 | 02 | 1 | ORPH-02 | unit | `cargo test test_get_arxiv_references_ids` | ✅ (existing) | ⬜ pending |
| 19-XX-XX | — | — | ORPH-01 | manual-only | — | N/A | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `resyn-core/src/datamodels/paper.rs` — add `test_get_arxiv_references_ids_filters_empty` unit test verifying that a reference with `arxiv_eprint: Some("".to_string())` does NOT appear in output
- [ ] `resyn-core/src/data_aggregation/inspirehep_api.rs` — extend `test_convert_hit_to_paper` to assert `paper.published` is set from `earliest_date`; extend `test_convert_hit_with_missing_optional_fields` to assert `paper.published == ""`
- [ ] `resyn-core/src/data_aggregation/inspirehep_api.rs` — add `test_inspirehep_fetch_paper_published` wiremock integration test verifying `fetch_paper()` returns a paper with `published` populated

*Existing infrastructure covers framework installation — wiremock already in dev-dependencies.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Orphan root cause diagnosis documented | ORPH-01 | Static code analysis, not runtime behavior | Review 19-RESEARCH.md "Orphan Root Cause Analysis" section for completeness |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
