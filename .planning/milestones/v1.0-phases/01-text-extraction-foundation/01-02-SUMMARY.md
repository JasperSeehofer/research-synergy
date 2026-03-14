---
phase: 01-text-extraction-foundation
plan: 02
subsystem: data_aggregation
tags: [rust, scraper, css-selectors, ar5iv, text-extraction, cli]

# Dependency graph
requires:
  - TextExtractionResult, ExtractionMethod, SectionMap from src/datamodels/extraction.rs (01-01)
  - ExtractionRepository from src/database/queries.rs (01-01)
provides:
  - Ar5ivExtractor with rate-limited fetch and CSS-selector section parsing in src/data_aggregation/text_extractor.rs
  - --analyze and --skip-fulltext CLI flags in src/main.rs with analysis pipeline
affects:
  - 01-03 (NLP pipeline reads SectionMap produced by Ar5ivExtractor)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Ar5ivExtractor mirrors ArxivHTMLDownloader pattern (last_called/call_per_duration/client fields, rate_limit_check method)
    - CSS selector hierarchy: .ltx_abstract for abstract, section.ltx_section/.ltx_para for named sections
    - Delimiter-guarded Roman numeral stripping in title normalization (only strips I/II/... when followed by '.', ':', or ' ')
    - Category keyword matching with contains() for flexible title formats
    - async fn run_analysis() helper extracted for DRY reuse across db_only and normal flows

key-files:
  created:
    - src/data_aggregation/text_extractor.rs
  modified:
    - src/data_aggregation/mod.rs
    - src/main.rs

key-decisions:
  - "Delimiter-guarded Roman numeral stripping: only strip I/II/... when followed by delimiter, not as word-starts (fixed 'Introduction' -> 'ntroduction' bug)"
  - "Abstract extracted from .ltx_para children of .ltx_abstract (not all text content) to avoid capturing the 'Abstract' heading element"
  - "run_analysis() extracted as async helper for reuse in both db_only and normal flows"

requirements-completed: [TEXT-03, TEXT-04, INFR-04]

# Metrics
duration: ~95min (including two disk-full clean cycles)
completed: 2026-03-14
---

# Phase 1 Plan 02: Ar5ivExtractor Implementation and CLI Integration Summary

**Ar5ivExtractor with CSS-selector section parsing + abstract fallback wired into main.rs via --analyze/--skip-fulltext flags**

## Performance

- **Duration:** ~95 min (2 disk-full events requiring cargo clean added ~40 min)
- **Started:** 2026-03-14T02:05:18Z
- **Completed:** 2026-03-14T04:00:00Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Created Ar5ivExtractor mirroring ArxivHTMLDownloader pattern with configurable rate limiting
- Implemented parse_sections() using CSS selectors (.ltx_abstract, section.ltx_section, .ltx_para) for structured section extraction
- Title normalization handles numbered sections ("1 Introduction"), Roman numerals ("I. Introduction"), lettered sections ("A. Motivation")
- Section categorization maps introduction/methods/results/conclusion via keyword contains() matching
- Bibliography, references, acknowledgements, appendix excluded from extraction
- HTTP 404/500 and network errors trigger immediate abstract-only fallback with is_partial=true
- Wired --analyze (post-crawl extraction pipeline) and --skip-fulltext (force abstract-only) CLI flags
- run_analysis() helper reused from both db_only and normal pipeline flows

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement Ar5ivExtractor with section parsing and fallback** - `9a90fa8` (feat)
2. **Task 2: Wire --analyze and --skip-fulltext CLI flags into main.rs pipeline** - `26706c0` (feat)

## Files Created/Modified

- `src/data_aggregation/text_extractor.rs` - Ar5ivExtractor, parse_sections(), normalize_section_title(), section_category(), 9 tests
- `src/data_aggregation/mod.rs` - Added pub mod text_extractor
- `src/main.rs` - Added --analyze, --skip-fulltext flags, run_analysis() async helper, analysis step in both flows

## Decisions Made

- Delimiter-guarded Roman numeral stripping: only strip I/II/III/... when followed by '.', ':', or ' ' — prevents "Introduction" being stripped to "ntroduction" (the "I" matched as Roman numeral "I" without the guard)
- Abstract text extracted from .ltx_para children within .ltx_abstract rather than all text content, which avoids capturing the "Abstract" heading element that ar5iv LaTeXML emits as a separate element
- run_analysis() extracted as a standalone async helper callable from both db_only and normal flows, keeping main() readable

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed Roman numeral stripping consuming word-initial letters**
- **Found during:** Task 1 (TDD RED phase — test_parse_sections_all_sections and test_parse_sections_title_normalization failed)
- **Issue:** normalize_section_title("1 Introduction") returned "ntroduction" — after stripping leading "1", the Roman numeral "I" matched the start of "Introduction" and was stripped without checking for a following delimiter
- **Fix:** Added delimiter guard to Roman numeral stripping: only strip if the match is followed by '.', ':', ' ', or end-of-string
- **Files modified:** src/data_aggregation/text_extractor.rs
- **Commit:** 9a90fa8

**2. [Rule 1 - Bug] Fixed abstract text extraction capturing "Abstract" heading**
- **Found during:** Task 1 (test_extract_200_returns_ar5iv_html failed — expected "Abstract of the paper." got "of the paper.")
- **Issue:** Abstract extraction collected all text content from .ltx_abstract and stripped "Abstract" prefix, but this removed word "Abstract" from test text "Abstract of the paper."
- **Fix:** Extract .ltx_para children within .ltx_abstract rather than all text content; fall back to full-text-minus-prefix only when no .ltx_para found
- **Files modified:** src/data_aggregation/text_extractor.rs
- **Commit:** 9a90fa8

**3. [Rule 1 - Bug] Fixed clippy warning for nested if-let**
- **Found during:** Task 1 post-implementation clippy check
- **Issue:** `if let Ok(sel) = ... { if let Some(el) = ... { ... } }` can be collapsed (Rust edition 2024 allows `&&` chaining)
- **Fix:** Changed to `if let Ok(sel) = ... && let Some(el) = ... { ... }`
- **Files modified:** src/data_aggregation/text_extractor.rs
- **Commit:** 9a90fa8

---

**Total deviations:** 3 auto-fixed (2 bugs in title normalization logic, 1 clippy lint)

## Issues Encountered

- Disk full twice during build (target/ accumulated 10+ GB). Resolved with `cargo clean` each time. Added ~40 minutes to overall execution.

## Test Coverage

- 9 new tests in text_extractor.rs (unit + integration via wiremock)
- All 56 tests pass (47 pre-existing + 9 new)

## Next Phase Readiness

- Ar5ivExtractor ready for use by NLP pipeline (Plan 03)
- SectionMap fields populated for introduction/methods/results/conclusion detection
- All data flows through ExtractionRepository for caching (idempotent re-runs)

---
*Phase: 01-text-extraction-foundation*
*Completed: 2026-03-14*
