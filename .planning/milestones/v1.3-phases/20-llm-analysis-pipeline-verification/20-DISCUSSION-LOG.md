# Phase 20: LLM Analysis Pipeline Verification - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-28
**Phase:** 20-llm-analysis-pipeline-verification
**Areas discussed:** Analysis trigger UX, Result panel behavior, LLM provider config, Verification scope

---

## Analysis Trigger UX

### Trigger placement

| Option | Description | Selected |
|--------|-------------|----------|
| Dashboard action button | Prominent "Run Analysis" button on dashboard page | Combined |
| Post-crawl automatic prompt | After crawl completes, prompt "Run analysis on these papers?" | Combined |
| Nav bar or global action | Always-visible trigger in navigation/header | |

**User's choice:** Both options 1 and 2 combined — post-crawl prompt AND dashboard button
**Notes:** User wants both entry points for different workflows

### Analysis scope

| Option | Description | Selected |
|--------|-------------|----------|
| All crawled papers in DB | Analyze everything, skip already-analyzed (caching) | ✓ |
| User selects papers first | Pick which papers to analyze from a list | |
| Current graph view only | Analyze only papers visible in current graph | |

**User's choice:** All crawled papers in DB
**Notes:** None

### Progress feedback

| Option | Description | Selected |
|--------|-------------|----------|
| Reuse SSE progress stream | Extend /progress endpoint with analysis stage events | ✓ |
| Simple spinner with stage label | Polling-based, no streaming | |
| Detailed progress panel | Dedicated panel with per-paper status | |

**User's choice:** Reuse SSE progress stream
**Notes:** None

---

## Result Panel Behavior

### Empty state

| Option | Description | Selected |
|--------|-------------|----------|
| CTA to run analysis | Friendly empty state with "Run Analysis" button | ✓ |
| Hidden until results exist | Don't show panels until analysis has run | |
| Greyed out with placeholder | Show panel structure but greyed out | |

**User's choice:** CTA to run analysis
**Notes:** None

### Auto-refresh

| Option | Description | Selected |
|--------|-------------|----------|
| Auto-refresh via SSE | Panels reload when SSE signals analysis complete | ✓ |
| Manual refresh button | User clicks refresh on each panel | |
| Page reload prompt | Toast/banner saying "New results available" | |

**User's choice:** Auto-refresh via SSE
**Notes:** None

---

## LLM Provider Config

### Configuration method

| Option | Description | Selected |
|--------|-------------|----------|
| Environment variables only | Provider set via env vars at server start | ✓ |
| Settings page in UI | Config page with provider selection and API key input | |
| Server config file | Read from resyn.toml | |

**User's choice:** Environment variables for now
**Notes:** User wants a UI settings page in the future — deferred to a later phase

### Missing LLM behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Run NLP-only, skip LLM steps | TF-IDF works without LLM, show partial results with warning | ✓ |
| Block with error message | Show error, don't run anything | |
| Fall back to Noop provider | Use Noop silently | |

**User's choice:** Run NLP-only with a visible warning
**Notes:** User emphasized the warning should be visible, not silent

---

## Verification Scope

### Test provider

| Option | Description | Selected |
|--------|-------------|----------|
| Noop for CI, real for manual | Automated tests use Noop, manual uses real provider | |
| Ollama local only | All tests use Ollama with small model | ✓ |
| Claude API for everything | Use Claude API for all tests | |

**User's choice:** Ollama local only
**Notes:** None

### Done criteria

| Option | Description | Selected |
|--------|-------------|----------|
| Manual E2E + unit tests | Manual smoke test plus unit/integration tests | |
| Full automated E2E test | Automated test: spin up server, trigger analysis, check panels | ✓ |
| Manual smoke test only | Just verify manually | |

**User's choice:** Full automated E2E test
**Notes:** None

### E2E test LLM handling

| Option | Description | Selected |
|--------|-------------|----------|
| Mock LLM responses | Wiremock mocks Ollama API, deterministic, CI-friendly | Combined |
| Require Ollama running | Test calls real Ollama | Combined |
| Feature-gated: both | Default uses mocks; optional feature flag for real Ollama | ✓ |

**User's choice:** Feature-gated: both
**Notes:** Default mocked for CI, optional real Ollama behind feature flag

---

## Claude's Discretion

- How to wire analyze.rs pipeline into a server function (tokio::spawn pattern)
- ProgressEvent variant design for analysis stages
- Wiremock fixture design for mocked Ollama responses
- Error recovery strategy for mid-pipeline failures

## Deferred Ideas

- UI settings page for LLM provider configuration (user explicitly requested for future)
