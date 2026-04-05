# Phase 20: LLM Analysis Pipeline Verification - Context

**Gathered:** 2026-03-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Restore end-to-end LLM analysis in the web UI. Users trigger analysis from the UI, watch it run via SSE progress, and see all result panels (gap findings, open problems, method heatmap) populated with real data. The analysis pipeline already exists as CLI code — this phase wires it to the web UI and verifies everything works.

</domain>

<decisions>
## Implementation Decisions

### Analysis Trigger UX
- **D-01:** Two entry points: (1) After a crawl completes, automatically prompt "Run analysis?" with a confirmation action. (2) A "Run Analysis" button on the dashboard page for manual trigger at any time.
- **D-02:** Analysis runs against all crawled papers in the database. Skips already-analyzed papers (existing caching behavior). No paper selection UI needed.
- **D-03:** Reuse the existing `/progress` SSE endpoint for analysis progress. Extend `ProgressEvent` with analysis stage events (text extraction, TF-IDF, LLM annotation, gap analysis). Show progress inline on the dashboard.

### Result Panel Behavior
- **D-04:** Before analysis has run, result panels show a friendly empty state with a CTA button: "No analysis results yet — Run Analysis". Guides user to the next action.
- **D-05:** After analysis completes, result panels auto-refresh via SSE signal. When the SSE stream signals analysis complete, panels automatically reload their data from server functions.

### LLM Provider Configuration
- **D-06:** LLM provider configured via environment variables at server start (RESYN_LLM_PROVIDER, ANTHROPIC_API_KEY, OLLAMA_URL, etc.). Matches current CLI behavior. No UI settings page in this phase.
- **D-07:** When no LLM provider is configured, run NLP-only analysis (TF-IDF works without LLM). Skip LLM annotation and gap verification steps. Show a visible warning in the UI: "LLM provider not configured — showing NLP-only results. Set RESYN_LLM_PROVIDER for full analysis."
- **D-08:** Partial results are valid — user sees TF-IDF-based method matrix and whatever gap analysis can produce without LLM verification.

### Verification & Testing
- **D-09:** Use Ollama as the primary LLM provider for all verification testing (local, free).
- **D-10:** Full automated E2E test: spin up server, trigger analysis via HTTP, verify panel data via server function responses.
- **D-11:** Feature-gated test strategy: default test uses wiremock to mock Ollama HTTP API responses (deterministic, runs in CI). Optional `#[cfg(feature = "ollama-test")]` test hits a real running Ollama instance for integration verification.

### Claude's Discretion
- How to wire the existing `analyze.rs` pipeline into a Leptos server function (spawn tokio task, shared state, etc.)
- Specific ProgressEvent variants for analysis stages
- Wiremock fixture design for mocked Ollama responses
- Error recovery if analysis fails mid-pipeline (per-paper retry vs abort)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Analysis Pipeline (Core)
- `resyn-server/src/commands/analyze.rs` — Main analysis entry point with `run_extraction()`, `run_nlp_analysis()`, `run_llm_analysis()`, `run_gap_analysis()` stages
- `resyn-core/src/llm/traits.rs` — `LlmProvider` async trait (`annotate_paper()`, `verify_gap()`)
- `resyn-core/src/llm/ollama.rs` — Ollama provider implementation
- `resyn-core/src/llm/noop.rs` — Noop provider for testing
- `resyn-core/src/gap_analysis/contradiction.rs` — Contradiction detection logic
- `resyn-core/src/gap_analysis/abc_bridge.rs` — ABC-bridge discovery logic

### Server Functions (Frontend API)
- `resyn-app/src/server_fns/gaps.rs` — `get_gap_findings()` server function
- `resyn-app/src/server_fns/problems.rs` — `get_open_problems_ranked()` server function
- `resyn-app/src/server_fns/methods.rs` — `get_method_matrix()`, `get_method_drilldown()` server functions
- `resyn-app/src/server_fns/papers.rs` — `get_dashboard_stats()`, `StartCrawl()` server functions

### UI Pages (Result Panels)
- `resyn-app/src/pages/dashboard.rs` — Dashboard with analysis stats
- `resyn-app/src/pages/gaps.rs` — Gap findings panel (contradictions, bridges)
- `resyn-app/src/pages/open_problems.rs` — Ranked open problems list
- `resyn-app/src/pages/methods.rs` — Method co-occurrence heatmap

### SSE Progress
- `resyn-server/src/commands/serve.rs` — SSE `/progress` endpoint and broadcast channel setup

### Analysis Aggregation
- `resyn-core/src/analysis/aggregation.rs` — `aggregate_open_problems()`, `build_method_matrix()` pure functions
- `resyn-core/src/datamodels/gap_finding.rs` — `GapFinding` struct
- `resyn-core/src/datamodels/llm_annotation.rs` — `LlmAnnotation` with methods, findings, open_problems

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `resyn-server/src/commands/analyze.rs` — Complete 4-stage analysis pipeline (extraction, TF-IDF, LLM, gaps). Needs to be callable from a server function, not just CLI.
- `resyn-server/src/commands/serve.rs` — SSE `/progress` endpoint with `broadcast::Sender<ProgressEvent>`. Already used for crawl progress — extend for analysis events.
- `resyn-app/src/server_fns/papers.rs` — `StartCrawl` server function pattern (spawns background task). Reuse this pattern for `StartAnalysis`.
- `resyn-core/src/llm/noop.rs` — Noop provider for deterministic testing.
- All three result panel pages (`gaps.rs`, `open_problems.rs`, `methods.rs`) already fetch and render data via server functions — they just need data to exist.

### Established Patterns
- Server functions use `#[server]` macro with SurrealDB connection from `use_context::<DbPool>()`.
- Background tasks (crawl) spawned via `tokio::spawn` with broadcast channel for progress.
- SSE progress stream at `/progress` sends JSON `ProgressEvent` variants.
- Leptos `create_resource` for async data loading in UI components.

### Integration Points
- Dashboard page needs: "Run Analysis" button, post-crawl prompt, progress display.
- Analysis server function needs: access to same DB pool and broadcast channel as crawl.
- ProgressEvent enum needs: new analysis-stage variants.
- Result panels need: empty state component with CTA, SSE-triggered refetch.

</code_context>

<specifics>
## Specific Ideas

- Post-crawl prompt: after `StartCrawl` completes (SSE signals done), show inline "Analysis available — Run now?" rather than a modal dialog.
- Warning for missing LLM: banner-style warning at top of dashboard, not a blocking modal. User can still use NLP-only results.
- UI settings page for LLM provider config deferred to a future phase.

</specifics>

<deferred>
## Deferred Ideas

- **UI settings page for LLM provider** — User requested env vars for now, but wants a settings page in the UI for future. Belongs in a future phase.

</deferred>

---

*Phase: 20-llm-analysis-pipeline-verification*
*Context gathered: 2026-03-28*
