---
phase: 18-arxiv-crawl-repair
plan: 01
subsystem: data-aggregation
tags: [arxiv, html-parser, regex, citation-extraction, reference-dedup]
dependency_graph:
  requires: []
  provides: [ARXIV-01]
  affects: [resyn-core/data_aggregation, resyn-core/datamodels]
tech_stack:
  added: [regex = "1"]
  patterns: [OnceLock<Regex> static patterns, text-based ID extraction, fallback field lookup]
key_files:
  created: []
  modified:
    - Cargo.toml
    - resyn-core/Cargo.toml
    - resyn-core/src/data_aggregation/arxiv_utils.rs
    - resyn-core/src/datamodels/paper.rs
decisions:
  - "Use OnceLock<Regex> statics for compiled patterns — initialized once at first call, zero overhead thereafter"
  - "Dedup via seen_arxiv_ids HashSet prevents duplicate Links when hyperlink and plain text reference the same arXiv ID"
  - "get_arxiv_id() Link-based lookup takes priority over arxiv_eprint fallback — preserves existing behavior for papers with hyperlinks"
  - "split('/').next_back() instead of .last() throughout — avoids clippy::double_ended_iterator_last warning"
metrics:
  duration: "5 minutes"
  completed_date: "2026-03-28"
  tasks: 2
  files: 4
---

# Phase 18 Plan 01: arXiv HTML Reference Parser Text Extraction Summary

Regex-based arXiv ID and DOI extraction from bibliography plain text, plus get_arxiv_id() fallback to arxiv_eprint field.

## Objective

Fix the arXiv HTML reference parser to extract arXiv IDs and DOIs from plain text in bibliography entries (not just `<a>` tags), and update `get_arxiv_id()` to fall back to the `arxiv_eprint` field when no `Journal::Arxiv` Link is present.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add regex dep and text-based arXiv ID/DOI extraction | a9f125c | Cargo.toml, resyn-core/Cargo.toml, arxiv_utils.rs, resyn-app (fmt) |
| 2 | Add get_arxiv_id() fallback to arxiv_eprint and unit tests | 903f6ca | resyn-core/src/datamodels/paper.rs |

## What Was Built

### Task 1: Text-based ID Extraction (arxiv_utils.rs)

Added `regex = "1"` to workspace dependencies (server-only, ssr-gated in resyn-core). Three `OnceLock<Regex>` statics compile patterns once:

- `ARXIV_NEW_RE`: matches `YYMM.NNNNN(vN)` format (e.g. `2301.12345`)
- `ARXIV_OLD_RE`: matches `category/NNNNNNN(vN)` format (e.g. `hep-ph/0601234`)
- `DOI_RE`: matches `10.NNNN/...` DOI patterns

Inside `aggregate_references_for_arxiv_paper`, after the child-node iteration loop, text extraction runs on `reference_string`:

1. Build `seen_arxiv_ids` HashSet from existing `<a>`-tag links (dedup guard)
2. Extract new-format arXiv IDs — push `https://arxiv.org/abs/{id}` to links if not already seen
3. Extract old-format arXiv IDs — same push logic
4. Extract first DOI from text

The `Reference` is constructed with `doi: text_extracted_doi` and `arxiv_eprint: text_extracted_eprint` populated.

Three unit tests added: `test_arxiv_new_re_matches`, `test_arxiv_old_re_matches`, `test_doi_re_matches`.

### Task 2: get_arxiv_id() Fallback (paper.rs)

`Reference::get_arxiv_id()` now checks `arxiv_eprint` when no `Journal::Arxiv` Link exists:

```rust
// Primary: Journal::Arxiv Link lookup (unchanged behavior)
// Fallback: self.arxiv_eprint.clone().ok_or(ResynError::NoArxivLink)
```

Four unit tests added covering all paths: fallback to eprint, link priority over eprint, eprint-only (no links), and both-none error case.

## Verification Results

- `cargo fmt --all -- --check`: clean
- `cargo clippy -p resyn-core --features ssr -- -D warnings`: clean
- `cargo test -p resyn-core --features ssr`: 193 unit + 6 integration = 199 passed, 0 failed
- `cargo test --workspace`: full suite green, no regressions

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed clippy::double_ended_iterator_last in text-extraction dedup**
- **Found during:** Task 1 (clippy check)
- **Issue:** `url.split('/').last().map(|s| strip_version_suffix(s))` triggers `redundant_closure` and `double_ended_iterator_last`
- **Fix:** Changed to `url.split('/').next_back().map(strip_version_suffix)`
- **Files modified:** resyn-core/src/data_aggregation/arxiv_utils.rs
- **Commit:** a9f125c

**2. [Rule 1 - Bug] Fixed clippy::double_ended_iterator_last in get_arxiv_id()**
- **Found during:** Task 2 (clippy check)
- **Issue:** `link.url.split('/').last()` on `Split<'_, char>` which is `DoubleEndedIterator`
- **Fix:** Changed to `.next_back()`
- **Files modified:** resyn-core/src/datamodels/paper.rs
- **Commit:** 903f6ca

**3. [Rule 2 - Formatting] Applied cargo fmt to pre-existing resyn-app formatting issues**
- **Found during:** Task 1 (cargo fmt --all run)
- **Issue:** resyn-app/src/graph/canvas_renderer.rs and resyn-app/src/pages/graph.rs had trailing blank lines and long lines that would fail CI fmt check
- **Fix:** cargo fmt --all applied and included in Task 1 commit
- **Files modified:** resyn-app/src/graph/canvas_renderer.rs, resyn-app/src/pages/graph.rs
- **Commit:** a9f125c

## Known Stubs

None. Both text extraction fields (`doi`, `arxiv_eprint`) are wired to actual regex extraction from reference text. The `get_arxiv_id()` fallback is live code.

## Self-Check: PASSED

Files exist:
- resyn-core/src/data_aggregation/arxiv_utils.rs: FOUND
- resyn-core/src/datamodels/paper.rs: FOUND

Commits exist:
- a9f125c: FOUND (feat(18-01): add regex dep and text-based arXiv ID/DOI extraction)
- 903f6ca: FOUND (feat(18-01): add get_arxiv_id() fallback to arxiv_eprint field)
