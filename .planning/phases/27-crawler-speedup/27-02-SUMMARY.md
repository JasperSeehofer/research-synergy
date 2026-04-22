---
phase: 27-crawler-speedup
plan: "02"
subsystem: bulk-ingest
tags: [openalex, concept-ids, documentation, physics-corpus, claude-md]

dependency_graph:
  requires:
    - "27-01: DEFAULT_FILTER_PHYSICS constant (already added by Plan 01 Task 2)"
  provides:
    - "CLAUDE.md with correct OpenAlex concept IDs for physics corpus"
    - "CLAUDE.md bulk-ingest examples updated to --api-key pattern"
    - "C2778407487 (Altmetrics) fully removed from CLAUDE.md"
  affects:
    - CLAUDE.md

tech_stack:
  added: []
  patterns:
    - "Document both named constant and inline filter string in CLI examples"
    - "Remove incorrect/misleading concept IDs from developer-facing docs immediately"

key_files:
  created: []
  modified:
    - CLAUDE.md

decisions:
  - "Task 1 was a no-op: DEFAULT_FILTER_PHYSICS was already added by Plan 01 (9856a7e); no duplicate commit needed"
  - "Removed C2778407487 entirely from CLAUDE.md rather than adding a cautionary note (strict zero-occurrence acceptance criterion)"
  - "Comment explaining --mailto removal was rephrased to avoid --mailto appearing anywhere in CLAUDE.md"

metrics:
  duration: "10m"
  completed: "2026-04-22T15:47:14Z"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 1
---

# Phase 27 Plan 02: Physics Filter Constant and CLAUDE.md Concept ID Fix Summary

Fixed a silently wrong OpenAlex concept ID (`C2778407487` = Altmetrics, mislabelled as "Statistical Physics") in CLAUDE.md and updated bulk-ingest CLI examples to use `--api-key`/`OPENALEX_API_KEY` instead of the removed `--mailto` arg. The `DEFAULT_FILTER_PHYSICS` constant (`C26873012|C121864883`) was already present in `bulk_ingest.rs` from Plan 01.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add DEFAULT_FILTER_PHYSICS constant to bulk_ingest.rs | 9856a7e (Plan 01) | resyn-server/src/commands/bulk_ingest.rs |
| 2 | Fix concept IDs and update bulk-ingest docs in CLAUDE.md | 540a765 | CLAUDE.md |

## What Was Built

**Task 1 — bulk_ingest.rs (no-op for Plan 02):**

`DEFAULT_FILTER_PHYSICS` with exact value `"primary_location.source.id:S4306400194,concepts.id:C26873012|C121864883"` was already present from Plan 01's Task 2 commit (9856a7e). No duplicate change needed; verified with `grep` and `cargo check -p resyn-server` (clean).

**Task 2 — CLAUDE.md:**

Three targeted edits:

1. **Bulk-ingest CLI examples block** — replaced the old custom filter example (which had `C2778407487`) with two examples showing:
   - Default ML ingest: `bulk-ingest --db ... --api-key "$OPENALEX_API_KEY"`
   - Physics corpus ingest: `bulk-ingest --db surrealkv://./data-physics --api-key "$OPENALEX_API_KEY" --filter "...concepts.id:C26873012|C121864883"`

2. **Important Notes bullet** — replaced `C2778407487`=Statistical Physics with `C26873012`=Condensed matter physics and `C121864883`=Statistical physics; also added inline mention that `--api-key`/`OPENALEX_API_KEY` is required.

3. **Comment cleanup** — removed all `--mailto` references from CLAUDE.md (including a comment that had said `--mailto removed`).

## Deviations from Plan

### Task 1 Already Complete

**[No deviation — expected overlap]** Plan 01's Task 2 added `DEFAULT_FILTER_PHYSICS` as part of the `bulk_ingest.rs` migration. The `prior_wave_context` in Plan 02 noted this possibility ("If Plan 01 has already run..."). Task 1 of Plan 02 was verified as already satisfied by commit 9856a7e; no new commit was created for it.

### C2778407487 Cautionary Note Removed

**[Plan compliance adjustment]** Initially the CLAUDE.md edit included a parenthetical note `(Note: C2778407487 is Altmetrics, NOT Statistical Physics...)` to help future developers. However, the acceptance criterion requires zero occurrences of `C2778407487` in CLAUDE.md. The note was removed to satisfy the strict criterion; the warning is implicit in its complete absence from the docs.

## Known Stubs

None. No placeholder data paths introduced.

## Threat Flags

None. CLAUDE.md is a developer-facing doc in a private repository; no secrets involved (T-27-06 accepted).

## Self-Check

- [x] `DEFAULT_FILTER_PHYSICS` present in `resyn-server/src/commands/bulk_ingest.rs` with value `"primary_location.source.id:S4306400194,concepts.id:C26873012|C121864883"` — FOUND
- [x] `grep "C2778407487" CLAUDE.md` — zero matches — PASS
- [x] `grep "C26873012" CLAUDE.md` — 2 matches — PASS
- [x] `grep "C121864883" CLAUDE.md` — 2 matches — PASS
- [x] `grep "OPENALEX_API_KEY\|--api-key" CLAUDE.md` — 4 matches — PASS
- [x] `grep -- "--mailto" CLAUDE.md` — zero matches — PASS
- [x] `cargo check -p resyn-server` — clean — PASS
- [x] Commit 540a765 exists — FOUND

## Self-Check: PASSED
