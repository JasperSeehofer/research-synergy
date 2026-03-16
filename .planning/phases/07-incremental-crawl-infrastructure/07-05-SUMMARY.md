---
phase: 07-incremental-crawl-infrastructure
plan: "05"
subsystem: cli
tags: [clap, rust, crawl, queue, subcommand]

requires:
  - phase: 07-incremental-crawl-infrastructure
    provides: CrawlSubcommand enum and queue management dispatch block in crawl.rs

provides:
  - CrawlSubcommand variants with per-variant --db field accepted after subcommand name
  - UAT Test 2 (--db flag after subcommand) unblocked

affects:
  - UAT tests 2-5 for queue management subcommands

tech-stack:
  added: []
  patterns:
    - "Clap struct variants: each subcommand variant carries its own args to avoid positional/flag collision with parent args"

key-files:
  created: []
  modified:
    - resyn-server/src/commands/crawl.rs

key-decisions:
  - "Each CrawlSubcommand variant owns its --db arg rather than sharing the parent CrawlArgs.db — clap subcommand context stops parent parsing"

patterns-established:
  - "Clap subcommand struct variants: give each variant its own fields so flags parsed in subcommand context work correctly"

requirements-completed: [CRAWL-03]

duration: 1min
completed: "2026-03-16"
---

# Phase 07 Plan 05: Fix CrawlSubcommand --db Flag Summary

**clap CrawlSubcommand unit variants converted to named struct variants, each with its own --db field, fixing `crawl status/clear/retry --db <val>` being rejected as unexpected argument**

## Performance

- **Duration:** 1 min
- **Started:** 2026-03-16T18:17:49Z
- **Completed:** 2026-03-16T18:18:55Z
- **Tasks:** 1
- **Files modified:** 1

## Accomplishments

- Converted `CrawlSubcommand::Status`, `::Clear`, `::Retry` from unit variants to named struct variants each carrying `db: String` with `default_value = "surrealkv://./data"`
- Updated dispatch block to extract `db_str` from the variant, then connect, then dispatch — eliminating the broken `args.db` reference for subcommand path
- All 172 tests pass, clippy clean, fmt clean

## Task Commits

Each task was committed atomically:

1. **Task 1: Convert CrawlSubcommand unit variants to struct variants with --db** - `a7098c1` (fix)

**Plan metadata:** (docs commit follows)

## Files Created/Modified

- `resyn-server/src/commands/crawl.rs` - CrawlSubcommand variants now carry their own --db field; dispatch block updated to destructure db_str from variant

## Decisions Made

None - followed plan exactly as specified.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- UAT Test 2 ("Queue status subcommand accepts --db flag") is now unblocked
- UAT Tests 3, 4, 5 (clear, retry, and pdf_url gap) can be retested
- Phase 08 can proceed once UAT is re-run and confirmed

---
*Phase: 07-incremental-crawl-infrastructure*
*Completed: 2026-03-16*
