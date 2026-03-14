---
phase: 2
slug: nlp-analysis-db-schema
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-14
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` + `#[tokio::test]` |
| **Config file** | `Cargo.toml` (no separate test config) |
| **Quick run command** | `cargo test nlp` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test nlp`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 02-01-01 | 01 | 1 | TEXT-02 | unit | `cargo test nlp::tfidf::test_section_weighting` | ❌ W0 | ⬜ pending |
| 02-01-02 | 01 | 1 | TEXT-02 | unit | `cargo test nlp::preprocessing::test_stop_words_excluded` | ❌ W0 | ⬜ pending |
| 02-01-03 | 01 | 1 | TEXT-02 | unit | `cargo test nlp::tfidf::test_smooth_idf` | ❌ W0 | ⬜ pending |
| 02-01-04 | 01 | 1 | TEXT-02 | unit | `cargo test nlp::tfidf::test_top_n_ranking` | ❌ W0 | ⬜ pending |
| 02-02-01 | 02 | 1 | TEXT-02 | integration | `cargo test database::queries::test_analysis_upsert_and_get` | ❌ W0 | ⬜ pending |
| 02-02-02 | 02 | 1 | INFR-02 | integration | `cargo test database::queries::test_corpus_fingerprint_skip` | ❌ W0 | ⬜ pending |
| 02-02-03 | 02 | 1 | INFR-02 | integration | `cargo test database::queries::test_migrate_schema_v3_v4` | ❌ W0 | ⬜ pending |
| 02-02-04 | 02 | 1 | INFR-02 | unit | `cargo test database::queries::test_analysis_exists` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/nlp/mod.rs` — module declaration
- [ ] `src/nlp/tfidf.rs` — `TfIdfEngine` with unit tests
- [ ] `src/nlp/preprocessing.rs` — `tokenize()`, `build_stop_words()` with unit tests
- [ ] `src/datamodels/analysis.rs` — `PaperAnalysis`, `AnalysisMetadata` structs
- [ ] Migration 3 + 4 in `src/database/schema.rs` — new DB tables
- [ ] `AnalysisRepository` in `src/database/queries.rs` — with DB integration tests

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Keyword rankings visible in CLI output at info level | TEXT-02 | Requires running full binary with `--analyze` flag | Run `cargo run -- --db mem:// --analyze --paper-id 2503.18887 --max-depth 1` and verify top-5 keywords logged per paper |
| Corpus-level summary printed after analysis | TEXT-02 | Full pipeline output verification | Same run as above — verify summary line with paper count and top corpus terms |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
