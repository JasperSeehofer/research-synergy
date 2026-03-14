---
phase: 02-nlp-analysis-db-schema
verified: 2026-03-14T12:00:00Z
status: passed
score: 9/9 must-haves verified
re_verification: false
---

# Phase 2: NLP Analysis + DB Schema Verification Report

**Phase Goal:** Every paper in the corpus has keyword rankings and TF-IDF vectors stored in SurrealDB, computed offline without any API calls, and future schema changes apply via migrations rather than manual DDL
**Verified:** 2026-03-14
**Status:** PASSED
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| #  | Truth | Status | Evidence |
|----|-------|--------|----------|
| 1  | Running `--analyze` populates paper_analysis table with per-paper keyword rankings and TF-IDF vectors | VERIFIED | `run_nlp_analysis()` in `src/main.rs` calls `TfIdfEngine::compute_corpus()` then `analysis_repo.upsert_analysis()` for each paper (lines 261-291) |
| 2  | Re-running `--analyze` on unchanged corpus skips NLP recomputation | VERIFIED | Corpus fingerprint guard at lines 249-258: matches fingerprint + paper_count before returning early with "Corpus unchanged, skipping" log |
| 3  | Migrations 3+4 create paper_analysis and analysis_metadata tables without data loss | VERIFIED | `apply_migration_3` and `apply_migration_4` use `DEFINE TABLE IF NOT EXISTS` / `DEFINE FIELD IF NOT EXISTS` guards; `migrate_schema()` uses `if version < 3` / `if version < 4` version guards |
| 4  | Keyword rankings visible in CLI output at info level | VERIFIED | `src/main.rs` lines 273-278: `info!(paper = ..., "Paper {}: {}", arxiv_id, keywords_display.join(", "))` for top-5 terms with scores |
| 5  | TF-IDF computation is offline — no API calls | VERIFIED | `src/nlp/tfidf.rs` and `src/nlp/preprocessing.rs` use only `sha2`, `stop_words`, and standard library. No `reqwest`, no `HttpClient`, no network code anywhere in the NLP module |
| 6  | PaperAnalysis and AnalysisMetadata structs exist and round-trip through serde | VERIFIED | `src/datamodels/analysis.rs` — 4 serde roundtrip tests pass (test_paper_analysis_roundtrip_serde, test_analysis_metadata_roundtrip_serde, test_paper_analysis_empty_tfidf_serializes, test_paper_analysis_top_terms_top_scores_same_length) |
| 7  | AnalysisRepository can upsert, get, check existence, and get_all analysis records | VERIFIED | `src/database/queries.rs` lines 478-541: all five methods implemented and backed by 7 DB integration tests against in-memory SurrealDB |
| 8  | Corpus fingerprint can be stored and retrieved from analysis_metadata | VERIFIED | `AnalysisRepository::upsert_metadata` / `get_metadata` implemented; `test_upsert_and_get_metadata` test confirms roundtrip of key, paper_count, corpus_fingerprint, last_analyzed |
| 9  | Abstract-only papers produce valid TF-IDF scores (not empty) | VERIFIED | `src/nlp/tfidf.rs` test_abstract_only_paper_produces_scores: single-extraction corpus with abstract-only produces non-empty, positive-score TF-IDF vector |

**Score:** 9/9 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/datamodels/analysis.rs` | PaperAnalysis, AnalysisMetadata structs | VERIFIED | 117 lines; both structs with serde derives, parallel arrays, HashMap<String, f32> sparse vector; 4 tests |
| `src/database/schema.rs` | Migrations 3 and 4 | VERIFIED | `apply_migration_3` (lines 80-96) and `apply_migration_4` (lines 98-112); `migrate_schema()` calls both with version guards (lines 138-146) |
| `src/database/queries.rs` | AnalysisRepository with upsert/get/exists/get_all | VERIFIED | `AnalysisRepository<'a>` defined at lines 474-541 with all five required methods plus upsert_metadata/get_metadata |
| `src/nlp/preprocessing.rs` | tokenize(), build_stop_words() | VERIFIED | 77 lines; both functions implemented, 2 tests passing |
| `src/nlp/tfidf.rs` | TfIdfEngine with corpus-aware scoring | VERIFIED | 419 lines; TfIdfEngine::compute_corpus(), get_top_n(), plus standalone compute_weighted_tf(), compute_smooth_idf(), corpus_fingerprint(); 9 unit tests |
| `src/main.rs` | NLP analysis step after text extraction in run_nlp_analysis() | VERIFIED | `run_nlp_analysis()` at lines 226-342; called from `run_analysis()` at line 223 after extraction loop completes |
| `src/nlp/mod.rs` | Module declaration for NLP submodules | VERIFIED | Declares `pub mod preprocessing; pub mod tfidf;` |
| `src/datamodels/mod.rs` | analysis module exported | VERIFIED | `pub mod analysis;` added alongside extraction and paper |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `src/database/queries.rs` | `src/datamodels/analysis.rs` | `From<&PaperAnalysis> for AnalysisRecord` | WIRED | `impl From<&PaperAnalysis> for AnalysisRecord` at line 404; `AnalysisRecord::to_analysis()` at line 420 |
| `src/database/schema.rs` | `migrate_schema` | `if version < 3` guard | WIRED | Line 138: `if version < 3 { apply_migration_3(db).await?; ... }` — exact pattern present |
| `src/nlp/tfidf.rs` | `src/datamodels/extraction.rs` | SectionMap used for section-weighted TF | WIRED | `use crate::datamodels::extraction::{SectionMap, TextExtractionResult};` at line 5; `compute_weighted_tf(sections: &SectionMap, ...)` at line 19 |
| `src/nlp/tfidf.rs` | `src/datamodels/analysis.rs` | produces PaperAnalysis structs | WIRED | `PaperAnalysis` struct constructed at `src/main.rs` lines 280-287 from TF-IDF results (tfidf.rs output used as input to PaperAnalysis) |
| `src/main.rs` | `src/nlp/tfidf.rs` | `run_nlp_analysis` calls TfIdfEngine | WIRED | `nlp::tfidf::TfIdfEngine::compute_corpus(&extractions)` at line 261; `nlp::tfidf::TfIdfEngine::get_top_n(...)` at line 265; `nlp::tfidf::corpus_fingerprint(...)` at line 246 |
| `src/main.rs` | `src/database/queries.rs` | AnalysisRepository for persistence | WIRED | `database::queries::AnalysisRepository::new(db)` at line 228; `analysis_repo.upsert_analysis()` at line 289; `analysis_repo.get_metadata()` at line 249; `analysis_repo.upsert_metadata()` at line 301 |
| `src/main.rs` | `src/database/queries.rs` | get_all_extractions before compute_corpus | WIRED | `extraction_repo.get_all_extractions().await` at line 231 completes before `TfIdfEngine::compute_corpus(&extractions)` at line 261 — strict sequential ordering confirmed |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| TEXT-02 | 02-02-PLAN.md | Corpus-relative keywords per paper using TF-IDF (offline, no API cost) | SATISFIED | TF-IDF engine in `src/nlp/tfidf.rs` operates entirely offline; `run_nlp_analysis()` persists per-paper TF-IDF vectors and top-5 keywords to SurrealDB paper_analysis table |
| INFR-02 | 02-01-PLAN.md, 02-02-PLAN.md | Analysis results cached in SurrealDB; re-runs skip already-analyzed papers | SATISFIED | Two-level caching: (1) corpus fingerprint check in `run_nlp_analysis()` skips full NLP when fingerprint + paper_count match; (2) `AnalysisRepository::upsert_analysis()` stores results per paper in SurrealDB |

Both requirements declared in phase plans are present in REQUIREMENTS.md and marked as complete. No orphaned requirements found — the phase claims only TEXT-02 and INFR-02, and REQUIREMENTS.md maps both to Phase 2.

### Anti-Patterns Found

No anti-patterns detected in phase-modified files:

- No TODO/FIXME/HACK/PLACEHOLDER comments in `src/nlp/`, `src/database/schema.rs`, `src/database/queries.rs`, `src/datamodels/analysis.rs`, or `src/main.rs` (phase additions)
- No empty implementations (`return null`, `return {}`, handler-only stubs)
- No stub API routes or placeholder renders
- No `unwrap()` calls on user-facing paths (only in test helpers and internal data conversion with documented fallback behavior)

### Human Verification Required

#### 1. End-to-end --analyze run with real paper data

**Test:** Run `cargo run -- --db surrealkv://./test_data --paper-id 2503.18887 --max-depth 1 --analyze`
**Expected:** Per-paper keyword logs appear at info level in the form "Paper 2503.XXXXX: term1 (0.XX), term2 (0.XX), ..."; corpus summary appears with paper count and top corpus terms; paper_analysis table is populated
**Why human:** Requires real arXiv HTTP requests to populate text_extraction table first; network-dependent

#### 2. Corpus fingerprint skip on second run

**Test:** Run the above command twice against the same surrealkv database path
**Expected:** Second run logs "Corpus unchanged (N papers), skipping NLP analysis" and terminates the NLP phase early without re-computing
**Why human:** Requires actual DB state persistence across two process invocations

### Gaps Summary

No gaps. All must-haves across both plans are verified at all three levels (exists, substantive, wired). The two items flagged for human verification are integration smoke tests that cannot be checked without network access and a running process; all automated checks pass.

---

## Supporting Evidence

**Test count:** 78 unit/integration tests pass (up from 56 before phase 2); includes 4 serde roundtrip tests (datamodels/analysis.rs), 7 DB integration tests (analysis repository and migrations), 11 NLP unit tests (preprocessing and tfidf)

**Commits verified:**
- `3a674b0` — feat(02-01): add PaperAnalysis/AnalysisMetadata models and sha2/stop-words deps
- `db2553d` — feat(02-01): add DB migrations 3+4 and AnalysisRepository
- `6706f39` — feat(02-02): NLP module with preprocessing and TF-IDF engine
- `1981e23` — feat(02-02): wire NLP analysis into run_analysis() pipeline

---

_Verified: 2026-03-14_
_Verifier: Claude (gsd-verifier)_
