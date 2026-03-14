---
phase: 02-nlp-analysis-db-schema
plan: "02"
subsystem: nlp
tags: [tfidf, nlp, stop-words, sha2, section-weighting, corpus-fingerprint, keyword-extraction]

dependency_graph:
  requires:
    - phase: 02-nlp-analysis-db-schema
      provides: [PaperAnalysis, AnalysisMetadata, AnalysisRepository, ExtractionRepository, get_all_extractions]
  provides:
    - tokenize() and build_stop_words() in src/nlp/preprocessing.rs
    - TfIdfEngine with section-weighted TF and corpus-level IDF in src/nlp/tfidf.rs
    - compute_weighted_tf(), compute_smooth_idf(), corpus_fingerprint() standalone functions
    - run_nlp_analysis(db) integrated into run_analysis() pipeline
    - Per-paper keyword logs at info level (top 5 terms with TF-IDF scores)
    - Corpus-level summary log (paper count, avg keywords, top 10 corpus terms)
    - Corpus fingerprint skip: re-runs on unchanged corpus return early
  affects: [phase-03-llm-analysis, phase-04-semantic-search]

tech-stack:
  added: []
  patterns:
    - "Section-weighted TF: abstract 2.0, methods 1.5, results 1.0, intro/conclusion 0.5 — accumulate weighted contributions across sections then normalize"
    - "Smooth IDF formula: ln((1+N)/(1+df))+1 matching sklearn default"
    - "Corpus-level IDF computed once via TfIdfEngine::compute_corpus() — anti-pattern avoided: no per-document IDF recomputation"
    - "Corpus fingerprint: sort IDs, join with comma, SHA-256 hash — deterministic and order-independent"
    - "All extractions loaded from DB before TF-IDF (RESEARCH.md Pitfall 2 compliance)"

key-files:
  created:
    - src/nlp/mod.rs
    - src/nlp/preprocessing.rs
    - src/nlp/tfidf.rs
  modified:
    - src/main.rs

key-decisions:
  - "mod nlp placed after error/mod and before utils alphabetically after cargo fmt reordered it"
  - "Unigrams only for now (per CONTEXT.md: bigrams deferred to v2)"
  - "Stop word filter applied at TF computation stage (not at tokenize stage) to keep tokenize() pure"
  - "avg_keywords computed as min(tfidf_vector.len(), 5) per paper to reflect actual top-N output"

patterns-established:
  - "NLP pipeline: get_all_extractions (full corpus) -> corpus_fingerprint check -> compute_corpus -> persist PaperAnalysis per paper -> upsert AnalysisMetadata"
  - "Early-exit corpus fingerprint guard: if fingerprint + paper_count match existing metadata, skip NLP entirely"

requirements-completed: [TEXT-02, INFR-02]

duration: 5min
completed: "2026-03-14"
---

# Phase 2 Plan 2: TF-IDF Computation Engine and Pipeline Integration Summary

**Section-weighted TF-IDF engine with corpus-level IDF, SHA-256 corpus fingerprint caching, and pipeline integration producing per-paper keyword logs and corpus summary via SurrealDB**

## Performance

- **Duration:** 5 minutes
- **Started:** 2026-03-14T10:56:59Z
- **Completed:** 2026-03-14T11:02:00Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- NLP module (src/nlp/) with preprocessing and TF-IDF engine, 11 unit tests covering all behavior specs
- Section-weighted TF (abstract 2x, methods 1.5x, results/intro/conclusion as specified), smooth IDF computed corpus-wide once
- Corpus fingerprint skip guards (SHA-256 of sorted IDs): re-running --analyze on unchanged corpus logs "Corpus unchanged, skipping" and returns early
- run_nlp_analysis() integrated at end of run_analysis() — reads all extractions from DB, persists PaperAnalysis and AnalysisMetadata, logs per-paper top-5 keywords and corpus summary

## Task Commits

Each task was committed atomically:

1. **Task 1: NLP module -- preprocessing and TF-IDF engine** - `6706f39` (feat)
2. **Task 2: Wire NLP analysis into run_analysis() pipeline** - `1981e23` (feat)

**Plan metadata:** (docs commit — see below)

## Files Created/Modified

- `src/nlp/mod.rs` - Module declaration for preprocessing and tfidf submodules
- `src/nlp/preprocessing.rs` - tokenize() (split on non-alphanumeric, lowercase, len>2) and build_stop_words() (English + academic boilerplate)
- `src/nlp/tfidf.rs` - compute_weighted_tf(), compute_smooth_idf(), corpus_fingerprint(), TfIdfEngine with compute_corpus() and get_top_n()
- `src/main.rs` - Added mod nlp declaration, run_nlp_analysis() function, call at end of run_analysis()

## Decisions Made

- `mod nlp` placed without `#[allow(dead_code)]` initially, but cargo fmt reordered module declarations alphabetically — nlp ended up between error and utils.
- Unigrams only per CONTEXT.md: bigrams deferred to v2 for simplicity.
- avg_keywords in corpus summary computed as min(tfidf_vector_len, 5) per paper to accurately reflect top-N output size.
- Stop word filtering applied inside compute_weighted_tf() (not in tokenize()) so tokenize() remains a pure string utility reusable without NLP context.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] cargo fmt reordered mod declarations and formatting**
- **Found during:** Task 2 (format check)
- **Issue:** cargo fmt reordered `mod nlp` alphabetically and reformatted several code blocks in tfidf.rs and main.rs
- **Fix:** Ran `cargo fmt --all` to apply all formatting
- **Files modified:** src/main.rs, src/nlp/preprocessing.rs, src/nlp/tfidf.rs
- **Verification:** `cargo fmt --all -- --check` passes with no diff
- **Committed in:** 1981e23 (Task 2 commit, after fmt applied)

**2. [Rule 3 - Blocking] clippy collapsible_if on corpus fingerprint guard**
- **Found during:** Task 2 (clippy check)
- **Issue:** Nested `if let Ok(Some(...)) { if ... { return; } }` triggered clippy::collapsible_if
- **Fix:** Collapsed to single `if let Ok(Some(...)) && condition1 && condition2 { return; }` using let-chain syntax (Rust edition 2024)
- **Files modified:** src/main.rs
- **Verification:** `cargo clippy --all-targets --all-features` reports no warnings
- **Committed in:** 1981e23

---

**Total deviations:** 2 auto-fixed (both Rule 3 - blocking for CI)
**Impact on plan:** Both necessary for CI compliance. No scope creep.

## Issues Encountered

None — the TF-IDF implementation compiled and all 11 tests passed on the first run. The plan's behavior specifications were clear enough to write and implement simultaneously.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- TEXT-02 and INFR-02 requirements fulfilled
- paper_analysis table populated with TF-IDF vectors and top-5 keywords per paper
- AnalysisMetadata with corpus fingerprint ready for Phase 3 to read (LLM semantic analysis)
- Phase 3 can read PaperAnalysis.top_terms as seed context for LLM prompts

---
*Phase: 02-nlp-analysis-db-schema*
*Completed: 2026-03-14*
