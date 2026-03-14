---
phase: 1
slug: text-extraction-foundation
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-14
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test runner (cargo test) |
| **Config file** | none — uses `#[cfg(test)]` inline + `cargo test` |
| **Quick run command** | `cargo test text_extractor` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test text_extractor`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 01-01-01 | 01 | 1 | TEXT-03 | integration | `cargo test text_extractor::tests::test_ar5iv_section_parse` | ❌ W0 | ⬜ pending |
| 01-01-02 | 01 | 1 | TEXT-03 | integration | `cargo test text_extractor::tests::test_ar5iv_404_fallback` | ❌ W0 | ⬜ pending |
| 01-01-03 | 01 | 1 | TEXT-03 | unit | `cargo test text_extractor::tests::test_ar5iv_url_construction` | ❌ W0 | ⬜ pending |
| 01-02-01 | 02 | 1 | TEXT-04 | unit | `cargo test extraction::tests::test_abstract_only_result` | ❌ W0 | ⬜ pending |
| 01-02-02 | 02 | 1 | TEXT-04 | unit | `cargo test extraction::tests::test_abstract_from_paper_summary` | ❌ W0 | ⬜ pending |
| 01-03-01 | 03 | 1 | INFR-03 | unit (DB) | `cargo test database::schema::tests::test_migration_v2_creates_extraction_table` | ❌ W0 | ⬜ pending |
| 01-03-02 | 03 | 1 | INFR-03 | unit (DB) | `cargo test database::schema::tests::test_migration_idempotent` | ❌ W0 | ⬜ pending |
| 01-04-01 | 04 | 1 | INFR-04 | unit | `cargo test -- --test cli_analyze_skipped_without_flag` | ❌ W0 | ⬜ pending |
| 01-04-02 | 04 | 1 | INFR-04 | unit | `cargo test -- --test cli_analyze_requires_db` | ❌ W0 | ⬜ pending |
| 01-04-03 | 04 | 1 | INFR-04 | unit | `cargo test text_extractor::tests::test_skip_fulltext_mode` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/data_aggregation/text_extractor.rs` — main extractor module with `#[cfg(test)]` block covering TEXT-03
- [ ] `src/datamodels/extraction.rs` — data model with unit tests covering TEXT-04
- [ ] `src/database/schema.rs` — modified to add migration system with `#[cfg(test)]` tests covering INFR-03
- [ ] Integration test HTML fixtures: wiremock-served ar5iv HTML snippets for extractor tests

*Existing infrastructure covers framework — no new test framework install needed.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| ar5iv CSS class `ltx_abstract` selector works | TEXT-03 | Class name extrapolated from LaTeXML convention, not confirmed from live HTML | Inspect real ar5iv paper HTML in browser devtools before writing selector |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
