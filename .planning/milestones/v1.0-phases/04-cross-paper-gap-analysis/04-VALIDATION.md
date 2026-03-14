---
phase: 4
slug: cross-paper-gap-analysis
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-14
---

# Phase 4 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness (cargo test) |
| **Config file** | none — inline `#[cfg(test)]` modules |
| **Quick run command** | `cargo test gap` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test gap`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 04-01-01 | 01 | 1 | GAPS-01 | unit | `cargo test gap_analysis::similarity::tests` | ❌ W0 | ⬜ pending |
| 04-01-02 | 01 | 1 | GAPS-01 | unit | `cargo test gap_analysis::contradiction::tests` | ❌ W0 | ⬜ pending |
| 04-01-03 | 01 | 1 | GAPS-01 | unit | `cargo test gap_analysis::contradiction::tests::test_strength_divergence` | ❌ W0 | ⬜ pending |
| 04-01-04 | 01 | 1 | GAPS-01 | unit (DB) | `cargo test test_migrate_schema_applies_migration_6` | ❌ W0 | ⬜ pending |
| 04-01-05 | 01 | 1 | GAPS-01 | unit (DB) | `cargo test test_gap_finding_insert_preserves_history` | ❌ W0 | ⬜ pending |
| 04-02-01 | 02 | 1 | GAPS-02 | unit | `cargo test gap_analysis::abc_bridge::tests::test_finds_bridge` | ❌ W0 | ⬜ pending |
| 04-02-02 | 02 | 1 | GAPS-02 | unit | `cargo test gap_analysis::abc_bridge::tests::test_no_bridge_direct_citation` | ❌ W0 | ⬜ pending |
| 04-03-01 | 01 | 1 | GAPS-01, GAPS-02 | unit | `cargo test gap_analysis::similarity::tests::test_shared_terms` | ❌ W0 | ⬜ pending |
| 04-03-02 | 01 | 1 | GAPS-01, GAPS-02 | unit (DB) | `cargo test test_gap_analysis_corpus_cache_skip` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/gap_analysis/mod.rs` — module declarations and re-exports
- [ ] `src/gap_analysis/similarity.rs` — `cosine_similarity`, `shared_high_weight_terms` with inline tests
- [ ] `src/gap_analysis/contradiction.rs` — detector with inline tests
- [ ] `src/gap_analysis/abc_bridge.rs` — discoverer with inline tests
- [ ] `src/gap_analysis/output.rs` — table formatter (no DB, pure output logic)
- [ ] `src/datamodels/gap_finding.rs` — `GapType` enum, `GapFinding` struct
- [ ] `src/llm/gap_prompt.rs` — prompt constants

*Existing infrastructure covers test framework requirements.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Gap findings printed to stdout grouped by type | GAPS-01, GAPS-02 | Terminal output formatting | Run `--analyze` on test corpus, visually verify table format |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
