---
phase: 04-cross-paper-gap-analysis
plan: "03"
subsystem: gap-analysis-pipeline-wiring
tags: [gap-analysis, cli, output-formatting, pipeline, surrealdb, caching]
dependency_graph:
  requires:
    - GapFinding type (src/datamodels/gap_finding.rs) — from 04-01
    - GapFindingRepository (src/database/queries.rs) — from 04-01
    - LlmProvider::verify_gap — from 04-01
    - find_contradictions (src/gap_analysis/contradiction.rs) — from 04-02
    - find_abc_bridges (src/gap_analysis/abc_bridge.rs) — from 04-02
    - corpus_fingerprint (src/nlp/tfidf.rs) — from 02-01
    - AnalysisRepository (src/database/queries.rs) — from 02-01
    - LlmAnnotationRepository (src/database/queries.rs) — from 03-01
  provides:
    - format_gap_table (src/gap_analysis/output.rs)
    - format_gap_summary (src/gap_analysis/output.rs)
    - run_gap_analysis (src/main.rs)
    - --full-corpus CLI flag
    - --verbose CLI flag
  affects:
    - src/gap_analysis/mod.rs (output module added)
    - src/main.rs (new flags, updated run_analysis, run_gap_analysis added)
tech_stack:
  added: []
  patterns:
    - Grouped table formatting with computed column widths (no external crate)
    - Justification truncation at 60 chars with "..." suffix
    - Corpus fingerprint cache guard using key "gap_analysis" (distinct from "corpus_tfidf")
    - Gap analysis always displays cached results even when corpus unchanged
    - Gap analysis skipped entirely when --llm-provider not specified
key_files:
  created:
    - src/gap_analysis/output.rs
  modified:
    - src/gap_analysis/mod.rs
    - src/main.rs
decisions:
  - format_gap_table uses manual format! with computed column widths — no tabled crate per RESEARCH.md
  - run_gap_analysis falls through to display even when fingerprint matches (show cached findings)
  - Corpus fingerprint key "gap_analysis" is distinct from "corpus_tfidf" to allow independent invalidation
  - --full-corpus informational log when DB paper count matches annotation count
metrics:
  duration: "6 minutes"
  completed: "2026-03-14"
  tasks_completed: 2
  files_created: 1
  files_modified: 2
---

# Phase 4 Plan 3: Gap Analysis Pipeline Wiring Summary

**One-liner:** Output formatter for grouped gap findings table with 60-char truncation, --full-corpus and --verbose CLI flags, and run_gap_analysis wired after LLM step with corpus fingerprint caching and SurrealDB persistence.

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | Output formatter for gap findings table | b259594 | src/gap_analysis/output.rs, src/gap_analysis/mod.rs |
| 2 | CLI flags + run_gap_analysis pipeline wiring | 98ddd62 | src/main.rs |

## What Was Built

### Task 1: Output Formatter (TDD)

**`src/gap_analysis/output.rs`** — Two public functions:

- `format_gap_table(findings: &[GapFinding], verbose: bool) -> String`:
  - Separates findings into Contradictions and ABC Bridges sections
  - Each section has a header with 80-char dashes separator
  - Empty sections display "(none found)" instead of an empty table
  - Table columns: Papers | Shared Terms | Justification with computed widths
  - Justification truncated to 60 chars with "..." suffix when `verbose=false`; full text when `verbose=true`
  - Uses manual `format!` with `{:<width$}` padding — no external crate

- `format_gap_summary(findings: &[GapFinding]) -> String`:
  - Returns: "Gap analysis: N contradictions, M ABC-bridges found across P papers"
  - Counts unique paper IDs across all findings for the P count

11 inline unit tests covering: empty findings, section headers, truncation at 60 chars, verbose mode, unique paper counting.

### Task 2: CLI Flags + run_gap_analysis

**CLI flags added to `Cli` struct:**
- `--full-corpus` (`bool`, default `false`): expand ABC-bridge scope to all DB papers
- `--verbose` (`bool`, default `false`): show full justifications in output table

**`run_analysis` signature extended** to accept `full_corpus: bool` and `verbose: bool`, threaded from both call sites in `main()` (db_only flow and normal flow).

**`run_gap_analysis` function:**
1. Loads all analyses and annotations from DB; skips if either is empty
2. Computes corpus fingerprint from annotation arxiv_ids; checks `get_metadata("gap_analysis")` for cache hit
3. If cache hit: logs "Gap corpus unchanged, skipping gap analysis" and falls through to display
4. If cache miss: loads all papers, builds citation graph via `create_graph_from_papers`, calls `find_contradictions` + `find_abc_bridges`, persists each finding via `GapFindingRepository::insert_gap_finding`, updates metadata with key `"gap_analysis"`
5. Always loads all findings from DB and prints table (stdout) + summary line (info! log)

Gap analysis is scoped inside `if let Some(provider_name) = llm_provider` — skipped entirely when `--llm-provider` not specified.

## Verification

- `cargo check` — clean
- `cargo clippy -- -D warnings` — clean
- `cargo test` — 138 passed (11 new output tests + 127 existing), 0 failed
- `cargo run -- --help` shows `--full-corpus` and `--verbose` flags

## Deviations from Plan

None — plan executed exactly as written.

## Self-Check: PASSED
