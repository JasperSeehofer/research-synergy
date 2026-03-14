---
phase: 03-pluggable-llm-backend
plan: "02"
subsystem: llm
tags: [reqwest, wiremock, anthropic, ollama, rate-limiting, tdd]

requires:
  - phase: 03-01
    provides: LlmProvider trait, LlmAnnotation/Method/Finding types, prompt constants, NoopProvider

provides:
  - ClaudeProvider implementing LlmProvider (src/llm/claude.rs)
  - OllamaProvider implementing LlmProvider (src/llm/ollama.rs)
  - LlmAnnotationRaw pub(crate) shared deserialization type
  - wiremock integration test coverage for both providers

affects:
  - 03-03 (any plan wiring providers into pipeline or CLI)
  - Any code that constructs concrete LlmProvider instances

tech-stack:
  added:
    - reqwest json feature (Cargo.toml feature flag, not new dependency)
  patterns:
    - parse-with-retry: first parse failure retries with RETRY_NUDGE appended to system prompt
    - rate-limit via Instant tracking (mirrors InspireHepClient pattern)
    - with_base_url builder for wiremock test injection without real APIs

key-files:
  created:
    - src/llm/claude.rs
    - src/llm/ollama.rs
  modified:
    - src/llm/mod.rs
    - Cargo.toml

key-decisions:
  - "reqwest json feature added as a feature flag on existing dependency — not a new crate"
  - "LlmAnnotationRaw defined in claude.rs as pub(crate) and reused by ollama.rs — single source of truth for LLM output shape"
  - "with_base_url builder pattern used for both providers enabling wiremock injection without env vars"
  - "OllamaProvider rate limit set to 350ms matching InspireHepClient, not 1s like ClaudeProvider"

patterns-established:
  - "Provider pattern: new() reads env, with_model()/with_rate_limit()/with_base_url() builder chain"
  - "Private rate_limit_check(&mut self) mutates last_called Instant, sleeps remainder if needed"
  - "Private call_api(&self, system, user_content) handles HTTP round-trip and error mapping"
  - "annotate_paper() orchestrates: rate_limit_check → call_api → parse → retry on failure → construct LlmAnnotation"

requirements-completed: [INFR-01]

duration: 12min
completed: 2026-03-14
---

# Phase 03 Plan 02: LLM Provider Implementations Summary

**ClaudeProvider and OllamaProvider built via TDD with reqwest direct HTTP — 8 wiremock tests prove correct headers, request format, retry logic, and parse recovery without real API calls.**

## Performance

- **Duration:** ~12 min
- **Started:** 2026-03-14T12:13:52Z
- **Completed:** 2026-03-14T12:25:16Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- ClaudeProvider sends all 3 required Anthropic headers (x-api-key, anthropic-version, content-type) and parses structured Claude response envelope
- OllamaProvider sends stream:false and LLM_ANNOTATION_SCHEMA as format field, parses Ollama message.content response shape
- Both providers implement parse-with-retry (RETRY_NUDGE on first parse failure), rate limiting via Instant tracking, and the with_base_url builder for test injection
- 8 wiremock integration tests (4 per provider) verify all HTTP contract details without real API calls

## Task Commits

Each task was committed atomically:

1. **Task 1: ClaudeProvider with wiremock integration tests** - `0d10c30` (feat)
2. **Task 2: OllamaProvider with wiremock integration tests** - `f919897` (feat)

**Plan metadata:** (docs commit follows)

## Files Created/Modified

- `src/llm/claude.rs` - ClaudeProvider implementing LlmProvider; LlmAnnotationRaw shared type; 4 wiremock tests
- `src/llm/ollama.rs` - OllamaProvider implementing LlmProvider; reuses LlmAnnotationRaw; 4 wiremock tests
- `src/llm/mod.rs` - Added `pub mod claude;` and `pub mod ollama;`
- `Cargo.toml` - Added `json` feature to reqwest dependency

## Decisions Made

- reqwest's `.json()` method requires the `json` feature flag which was not previously enabled. Added it as a feature on the existing dependency (not a new crate), consistent with the "no new dependencies" constraint.
- LlmAnnotationRaw defined once in claude.rs as `pub(crate)` and imported by ollama.rs — avoids duplication and keeps the LLM output shape as a single source of truth.
- OllamaProvider does not send an `x-api-key` header (Ollama is local-first, no auth by default). The provider can be extended with an auth header via builder if needed.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] reqwest json feature not enabled**
- **Found during:** Task 1 (ClaudeProvider compilation)
- **Issue:** `reqwest::RequestBuilder::json()` and `Response::json()` require the `json` feature flag, which was not in Cargo.toml. The `.json()` method was not found in scope.
- **Fix:** Changed `reqwest = "0.12.15"` to `reqwest = { version = "0.12.15", features = ["json"] }` in Cargo.toml.
- **Files modified:** Cargo.toml
- **Verification:** Both providers compile and all 8 wiremock tests pass.
- **Committed in:** `0d10c30` (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (Rule 3 - blocking dependency feature)
**Impact on plan:** Required for correct reqwest JSON serialization. No scope creep.

## Issues Encountered

None beyond the reqwest feature flag (documented above as deviation).

## User Setup Required

None — provider implementations require environment variables (`ANTHROPIC_API_KEY`, `OLLAMA_URL`) at runtime but no external service configuration is done in this plan.

## Next Phase Readiness

- ClaudeProvider and OllamaProvider are ready for wiring into the application pipeline (CLI selection, annotation runner)
- Both providers compile cleanly, pass clippy with no warnings, and are formatted
- LlmAnnotationRaw is available as pub(crate) in llm::claude for any future provider in the same module tree

---
*Phase: 03-pluggable-llm-backend*
*Completed: 2026-03-14*
