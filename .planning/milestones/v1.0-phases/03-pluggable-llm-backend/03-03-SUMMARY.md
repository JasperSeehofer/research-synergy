---
phase: 03-pluggable-llm-backend
plan: "03"
subsystem: llm
tags: [clap, cli, pipeline, caching, tracing]

requires:
  - phase: 03-01
    provides: LlmProvider trait, LlmAnnotation types, NoopProvider, LlmAnnotationRepository
  - phase: 03-02
    provides: ClaudeProvider, OllamaProvider implementing LlmProvider

provides:
  - --llm-provider CLI flag (claude, ollama, noop) wired into main pipeline
  - --llm-model CLI flag for model override
  - run_llm_analysis() loop with per-paper caching, skip-on-failure, end-of-run summary
  - Full end-to-end LLM annotation pipeline via cargo run -- --analyze --llm-provider noop

affects:
  - Any future phase adding new LLM providers or annotation consumers
  - Phase 4 (vector search) which will read from llm_annotation table

tech-stack:
  added: []
  patterns:
    - "LLM provider selected at startup from CLI flag, boxed as dyn LlmProvider for pipeline use"
    - "Per-item caching pattern: annotation_exists() check before each annotate_paper() call"
    - "Soft failure: single paper LLM error logs warn! and increments failed counter, pipeline continues"
    - "End-of-run summary: annotated/skipped/failed/total/provider matching NLP analysis style"

key-files:
  created: []
  modified:
    - src/main.rs

key-decisions:
  - "NoopProvider is a unit struct without new() — constructed directly as NoopProvider not NoopProvider::new()"
  - "LLM step runs only when --llm-provider is explicitly specified (Option<String>) — default disabled"
  - "run_analysis() extended with llm_provider/llm_model params rather than reading cli directly — keeps function testable and composable"

patterns-established:
  - "Provider dispatch pattern: match on provider_name string, construct Box<dyn LlmProvider>, pass as &mut dyn LlmProvider to runner"
  - "run_llm_analysis() is a standalone async fn taking db + provider — mirrors run_nlp_analysis() structure"

requirements-completed: [TEXT-01, INFR-01]

duration: 12min
completed: 2026-03-14
---

# Phase 03 Plan 03: Pipeline Wiring Summary

**CLI flags --llm-provider and --llm-model wire ClaudeProvider/OllamaProvider/NoopProvider into the main analysis pipeline with per-paper caching, soft failure handling, and end-of-run stats.**

## Performance

- **Duration:** ~12 min
- **Started:** 2026-03-14T12:30:00Z
- **Completed:** 2026-03-14T12:42:00Z
- **Tasks:** 1
- **Files modified:** 1

## Accomplishments

- `--llm-provider` (claude/ollama/noop) and `--llm-model` CLI flags added to `Cli` struct
- `run_llm_analysis()` iterates all papers in DB, skips already-annotated via `annotation_exists()`, logs failures and continues
- Provider construction dispatches to ClaudeProvider/OllamaProvider/NoopProvider based on CLI flag, with `process::exit(1)` for unknown providers
- End-of-run summary mirrors NLP analysis style: `annotated/skipped/failed/total/provider` structured logging
- All 100 tests pass, clippy clean, `--help` shows new flags

## Task Commits

Each task was committed atomically:

1. **Task 1: CLI flags and run_llm_analysis() pipeline wiring** - `73f688e` (feat)

**Plan metadata:** (docs commit follows)

## Files Created/Modified

- `src/main.rs` — Added CLI flags, LLM imports, updated run_analysis() signature, implemented run_llm_analysis()

## Decisions Made

- `NoopProvider` is a unit struct (no `new()` method) — constructed with bare `NoopProvider` literal. The plan's code example used `NoopProvider::new()` which does not exist. Auto-corrected inline.
- LLM analysis step placed after `run_nlp_analysis()` call within `run_analysis()`, consistent with the sequential extract → NLP → LLM pipeline ordering.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] NoopProvider::new() does not exist**
- **Found during:** Task 1 (compilation)
- **Issue:** Plan template code called `NoopProvider::new()` but `NoopProvider` is a unit struct with no constructor method
- **Fix:** Changed `NoopProvider::new()` to `NoopProvider` (unit struct literal)
- **Files modified:** src/main.rs
- **Verification:** `cargo check` passes
- **Committed in:** `73f688e` (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 - Bug in plan code example)
**Impact on plan:** Minor — unit struct construction syntax only. No scope creep.

## Issues Encountered

None beyond the NoopProvider constructor (documented above as deviation).

## User Setup Required

None — providers use environment variables (`ANTHROPIC_API_KEY`, `OLLAMA_URL`) at runtime but no external service configuration is done in this plan.

## Next Phase Readiness

- Phase 3 (03-pluggable-llm-backend) is now complete: traits, providers, and pipeline wiring all done
- `cargo run -- --db surrealkv://./data --analyze --llm-provider noop` produces LLM annotations in the database
- Phase 4 (vector search / semantic similarity) can read from `llm_annotation` table for downstream analysis

---
*Phase: 03-pluggable-llm-backend*
*Completed: 2026-03-14*
