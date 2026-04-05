# Phase 20: LLM Analysis Pipeline Verification - Research

**Researched:** 2026-03-28
**Domain:** Leptos server functions, Tokio background tasks, SSE progress, LLM provider wiring
**Confidence:** HIGH — all findings verified from live source code in this repo

## Summary

Phase 20 wires the existing 4-stage CLI analysis pipeline (`analyze.rs`) into the Leptos web UI. The pipeline itself is complete and correct. What is missing is a `StartAnalysis` server function (parallel to the existing `StartCrawl`), extension of the `ProgressEvent` enum with analysis-stage variants, dashboard UI additions (Run Analysis button, post-crawl prompt, LLM warning banner), and SSE-triggered result panel refresh. The result panels (`gaps.rs`, `open_problems.rs`, `methods.rs`) already fetch and render data — they just need data to exist and a trigger to refetch when analysis completes.

The `StartCrawl` server function in `resyn-app/src/server_fns/papers.rs` is the canonical pattern: call `use_context::<Arc<Db>>()` and `use_context::<broadcast::Sender<ProgressEvent>>()`, spawn a `tokio::spawn` background task, and return immediately while SSE streams progress. This exact pattern must be replicated for `StartAnalysis`.

The `ProgressEvent` struct is currently crawl-centric (depth/paper fields). It needs new `event_type` variants for analysis stages — the `event_type: String` field already carries semantic meaning (e.g. "complete", "idle"); analysis stages can follow the same approach.

**Primary recommendation:** Create `StartAnalysis` server function following the `StartCrawl` pattern exactly. Extend `ProgressEvent` with analysis-stage event types. Add analysis controls to the dashboard. Wire SSE-triggered refetch in result panels.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Two entry points: (1) After a crawl completes, automatically prompt "Run analysis?" with a confirmation action. (2) A "Run Analysis" button on the dashboard page for manual trigger at any time.
- **D-02:** Analysis runs against all crawled papers in the database. Skips already-analyzed papers (existing caching behavior). No paper selection UI needed.
- **D-03:** Reuse the existing `/progress` SSE endpoint for analysis progress. Extend `ProgressEvent` with analysis stage events (text extraction, TF-IDF, LLM annotation, gap analysis). Show progress inline on the dashboard.
- **D-04:** Before analysis has run, result panels show a friendly empty state with a CTA button: "No analysis results yet — Run Analysis". Guides user to the next action.
- **D-05:** After analysis completes, result panels auto-refresh via SSE signal. When the SSE stream signals analysis complete, panels automatically reload their data from server functions.
- **D-06:** LLM provider configured via environment variables at server start (RESYN_LLM_PROVIDER, ANTHROPIC_API_KEY, OLLAMA_URL, etc.). Matches current CLI behavior. No UI settings page in this phase.
- **D-07:** When no LLM provider is configured, run NLP-only analysis (TF-IDF works without LLM). Skip LLM annotation and gap verification steps. Show a visible warning in the UI: "LLM provider not configured — showing NLP-only results. Set RESYN_LLM_PROVIDER for full analysis."
- **D-08:** Partial results are valid — user sees TF-IDF-based method matrix and whatever gap analysis can produce without LLM verification.
- **D-09:** Use Ollama as the primary LLM provider for all verification testing (local, free).
- **D-10:** Full automated E2E test: spin up server, trigger analysis via HTTP, verify panel data via server function responses.
- **D-11:** Feature-gated test strategy: default test uses wiremock to mock Ollama HTTP API responses (deterministic, runs in CI). Optional `#[cfg(feature = "ollama-test")]` test hits a real running Ollama instance for integration verification.

### Claude's Discretion

- How to wire the existing `analyze.rs` pipeline into a Leptos server function (spawn tokio task, shared state, etc.)
- Specific ProgressEvent variants for analysis stages
- Wiremock fixture design for mocked Ollama responses
- Error recovery if analysis fails mid-pipeline (per-paper retry vs abort)

### Deferred Ideas (OUT OF SCOPE)

- **UI settings page for LLM provider** — User requested env vars for now, but wants a settings page in the UI for future. Belongs in a future phase.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| LLM-01 | User can trigger LLM analysis from the web UI and see it complete successfully | `StartAnalysis` server fn + `tokio::spawn` pattern from `StartCrawl`; SSE progress via existing `/progress` endpoint |
| LLM-02 | User can view gap findings (contradictions, ABC-bridges) in the web UI after analysis | `gaps.rs` page + `GetGapFindings` server fn already exist; need data from `run_gap_analysis` pipeline stage |
| LLM-03 | User can view open problems panel with results ranked by recurrence | `open_problems.rs` page + `GetOpenProblemsRanked` server fn already exist; need LLM annotations from `run_llm_analysis` |
| LLM-04 | User can view method heatmap showing existing vs absent pairings | `methods.rs` page + `GetMethodMatrix` server fn already exist; need LLM annotations from `run_llm_analysis` |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| leptos | (workspace) | Reactive UI + server functions | Already in use |
| tokio | (workspace) | Async runtime + spawn for background tasks | Already in use; `StartCrawl` pattern |
| leptos_use | (workspace) | `use_event_source` for SSE client side | Already in use in `crawl_progress.rs` |
| wiremock | (dev dep) | Mock Ollama HTTP API in tests | Already in use for InspireHEP tests |
| serde / serde_json | (workspace) | ProgressEvent JSON serialization | Already in use |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| tokio::sync::broadcast | std | Broadcast ProgressEvent to SSE clients | Already the channel type in `serve.rs` |
| reqwest | (workspace) | HTTP client for Ollama provider | Already in OllamaProvider |

**Installation:** No new dependencies needed. All required crates are already in the workspace.

## Architecture Patterns

### Recommended Project Structure — New Files
```
resyn-app/src/server_fns/
├── analysis.rs         # NEW: StartAnalysis server function
resyn-app/src/components/
├── analysis_controls.rs  # NEW: Run Analysis button + post-crawl prompt + LLM warning banner
resyn-core/src/datamodels/
├── progress.rs         # MODIFY: add analysis_stage field or new event_type variants
```

### Pattern 1: Background Task Server Function (from StartCrawl)
**What:** Server function acquires `Arc<Db>` and `broadcast::Sender<ProgressEvent>` from Leptos context, spawns `tokio::spawn`, returns immediately.
**When to use:** For `StartAnalysis` — identical requirement to `StartCrawl`.
**Example:**
```rust
// Source: resyn-app/src/server_fns/papers.rs — StartCrawl implementation
#[server(StartAnalysis, "/api")]
pub async fn start_analysis() -> Result<String, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let db = use_context::<std::sync::Arc<resyn_core::database::client::Db>>()
            .ok_or_else(|| ServerFnError::new("Database not available"))?;
        let tx = use_context::<tokio::sync::broadcast::Sender<resyn_core::datamodels::progress::ProgressEvent>>()
            .ok_or_else(|| ServerFnError::new("Progress channel not available"))?;

        tokio::spawn(async move {
            // read RESYN_LLM_PROVIDER from env
            // construct AnalyzeArgs with provider from env
            // call run_analysis_pipeline(&db, args, rate_limit_secs, skip_fulltext)
            // broadcast ProgressEvent variants at each stage
            // broadcast "analysis_complete" at end
        });

        Ok("Analysis started".to_string())
    }
    #[cfg(not(feature = "ssr"))]
    unreachable!()
}
```

### Pattern 2: ProgressEvent Extension for Analysis Stages
**What:** Add new `event_type` string values for analysis stages. The existing struct already uses `event_type: String` to signal "complete" / "idle".
**When to use:** To track 4 analysis stages: text extraction, NLP/TF-IDF, LLM annotation, gap analysis.
**Proposed event_type values:**
- `"analysis_extracting"` — stage 1 (ar5iv text extraction)
- `"analysis_nlp"` — stage 2 (TF-IDF corpus analysis)
- `"analysis_llm"` — stage 3 (LLM paper annotation)
- `"analysis_gaps"` — stage 4 (contradiction + ABC-bridge detection)
- `"analysis_complete"` — pipeline finished successfully
- `"analysis_error"` — pipeline failed

**Note:** The existing crawl-specific fields (`papers_found`, `papers_pending`, `current_depth`, `max_depth`) can be repurposed for analysis progress (e.g. `papers_found` = papers annotated so far, `papers_pending` = remaining). No struct changes required if we reuse fields; alternatively add an `analysis_stage: Option<String>` field.

### Pattern 3: SSE-Triggered Panel Refetch
**What:** In result panels, listen on the SSE stream for `"analysis_complete"` event type and call `Resource::refetch()`.
**When to use:** For `GapsPanel`, `OpenProblemsPanel`, `MethodsPanel`, and `Dashboard`.
**Example:**
```rust
// Source: resyn-app/src/components/crawl_progress.rs — SSE subscription pattern
let UseEventSourceReturn { message: sse_message, .. } =
    use_event_source::<ProgressEvent, JsonSerdeCodec>("/progress");

Effect::new(move |_| {
    if let Some(msg) = sse_message.get() {
        if msg.data.event_type == "analysis_complete" {
            findings.refetch();
        }
    }
});
```

### Pattern 4: Server Function Registration
**What:** New server functions must be explicitly registered in `resyn-server/src/commands/serve.rs`.
**When to use:** Every new `#[server]` macro fn needs `register_explicit::<analysis::StartAnalysis>()`.

### Pattern 5: LLM Provider from Environment
**What:** The `RESYN_LLM_PROVIDER` env var (decided in D-06) is not yet read anywhere — the CLI uses a `--llm-provider` arg. The `StartAnalysis` background task must read env vars at spawn time.
**When to use:** Inside the `tokio::spawn` closure in `StartAnalysis`.
**Known env vars:**
- `RESYN_LLM_PROVIDER` = `"ollama"` | `"claude"` | `"noop"` | unset (NLP-only)
- `OLLAMA_URL` = Ollama base URL (default: `http://localhost:11434`)
- `ANTHROPIC_API_KEY` = Claude API key

### Anti-Patterns to Avoid
- **Blocking the server function:** Never `await` the full analysis pipeline inside the server function body. Always `tokio::spawn` and return immediately.
- **Using `std::process::exit` in a server context:** The existing `analyze.rs` calls `std::process::exit(1)` on fatal errors — these must be removed or replaced with error propagation via the broadcast channel when called from the web context.
- **Assuming Ollama is running:** Ollama CLI is not installed on this machine. Tests MUST use wiremock to mock the HTTP API. The `#[cfg(feature = "ollama-test")]` optional test is for future manual verification.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| SSE client subscription | Custom EventSource wrapper | `leptos_use::use_event_source` | Already in use in `crawl_progress.rs`; handles reconnect |
| Ollama HTTP mock in tests | Custom mock server | `wiremock` | Already in dev deps; existing Ollama tests show exact pattern in `ollama.rs` |
| LLM annotation parsing | Custom JSON parser | Existing `OllamaProvider`/`ClaudeProvider` with retry logic | Handles retry, schema enforcement already |
| Background task spawn | Thread::spawn | `tokio::spawn` | Required for async operations; crawl already uses this |
| DB connection in server fn | New connection | `use_context::<Arc<Db>>()` | Pattern established in all existing server fns |

**Key insight:** All non-trivial logic is already implemented. This phase is purely wiring + UI additions.

## Common Pitfalls

### Pitfall 1: `std::process::exit` in Web Context
**What goes wrong:** `analyze.rs` calls `std::process::exit(1)` in error paths (e.g. failed DB connection, unknown provider). If called from a web server background task, this kills the entire server process.
**Why it happens:** `run_analysis_pipeline` was designed for CLI use where process exit is acceptable.
**How to avoid:** When calling `run_analysis_pipeline` from `StartAnalysis`, wrap in error handling that broadcasts `"analysis_error"` instead of exiting. OR refactor the pipeline to return `Result` instead of calling `exit`.
**Warning signs:** Cargo test panics mentioning `process::exit`; server process dies silently after triggering analysis.

### Pitfall 2: ProgressEvent Fields Are Crawl-Centric
**What goes wrong:** Reusing fields like `current_depth`/`max_depth` for analysis progress requires clients to interpret them differently based on `event_type`, which is fragile.
**Why it happens:** `ProgressEvent` was designed for crawl state.
**How to avoid:** Add an `analysis_stage: Option<String>` field to `ProgressEvent`, or use `papers_found`/`papers_pending` as "annotated so far"/"remaining" without reusing depth fields. Document the semantic clearly.
**Warning signs:** Dashboard shows confusing "depth 0/0" during analysis.

### Pitfall 3: Server Function Not Registered
**What goes wrong:** `StartAnalysis` server function returns 404 because it was not explicitly registered in `serve.rs`.
**Why it happens:** `serve.rs` comment explicitly states "inventory auto-registration doesn't work across crate boundaries in this setup" — all server fns are registered manually.
**How to avoid:** Add `register_explicit::<analysis::StartAnalysis>()` to `serve.rs` alongside existing registrations.
**Warning signs:** HTTP 404 on `/api/StartAnalysis` POST; no Leptos server fn error, just 404.

### Pitfall 4: Analysis State Not Tracked — Multiple Concurrent Triggers
**What goes wrong:** User clicks "Run Analysis" twice quickly → two concurrent analysis pipelines race to write DB records.
**Why it happens:** `StartAnalysis` spawns a new task without checking if one is already running.
**How to avoid:** Use a shared `Arc<AtomicBool>` flag (like a crawl guard) injected into app state, OR rely on the pipeline's existing caching (papers already analyzed are skipped). The caching approach is simpler and already validated.
**Warning signs:** Duplicate LLM annotation writes; inconsistent DB state.

### Pitfall 5: Panel Refetch Requires SSE Connection on Panel Load
**What goes wrong:** If the user navigates to `/gaps` after analysis completes without re-navigating through the dashboard, the SSE "analysis_complete" event was already sent and missed.
**Why it happens:** SSE is fire-and-forget; there is no replay.
**How to avoid:** D-05 says panels refetch on SSE signal — this handles the in-flight case. For the "already done, navigating cold" case, panels already show data from `Resource::new(|| (), ...)` which fires on mount. No additional action needed; the panel will load current data on mount regardless.

### Pitfall 6: NLP-Only Mode Must Not Call Gap Analysis
**What goes wrong:** Gap analysis (`find_contradictions`, `find_abc_bridges`) requires both `PaperAnalysis` (TF-IDF) AND `LlmAnnotation` records. If no LLM provider is configured, annotations are empty and gap analysis returns empty results but logs warnings.
**Why it happens:** The pipeline's guard `if analyses.is_empty() || annotations.is_empty() { return; }` handles this correctly.
**How to avoid:** When `RESYN_LLM_PROVIDER` is unset, only call `run_extraction` and `run_nlp_analysis`. Skip `run_llm_analysis` and `run_gap_analysis`. Show D-07 warning banner.

## Code Examples

Verified patterns from live source code:

### SSE Event Subscription (Client Side)
```rust
// Source: resyn-app/src/components/crawl_progress.rs lines 20-35
use codee::string::JsonSerdeCodec;
use leptos_use::{UseEventSourceReturn, use_event_source};
use resyn_core::datamodels::progress::ProgressEvent;

let UseEventSourceReturn { message: sse_message, .. } =
    use_event_source::<ProgressEvent, JsonSerdeCodec>("/progress");

let last_event: RwSignal<Option<ProgressEvent>> = RwSignal::new(None);
Effect::new(move |_| {
    if let Some(msg) = sse_message.get() {
        last_event.set(Some(msg.data));
    }
});
```

### Background Task with DB + Progress Broadcast
```rust
// Source: resyn-app/src/server_fns/papers.rs lines 142-314 (StartCrawl)
// Key pattern: acquire context, spawn, return immediately
let db = use_context::<std::sync::Arc<resyn_core::database::client::Db>>()
    .ok_or_else(|| ServerFnError::new("Database not available"))?;
let tx = use_context::<tokio::sync::broadcast::Sender<ProgressEvent>>()
    .ok_or_else(|| ServerFnError::new("Progress channel not available"))?;

tokio::spawn(async move {
    // background work...
    tx.send(ProgressEvent { event_type: "complete".to_string(), ... }).ok();
});
Ok("Started".to_string())
```

### LLM Provider Construction (Ollama)
```rust
// Source: resyn-server/src/commands/analyze.rs lines 82-90
let client = resyn_core::utils::create_http_client();
let p = OllamaProvider::new(client); // reads OLLAMA_URL from env
Box::new(if let Some(ref m) = args.llm_model {
    p.with_model(m.clone())
} else {
    p
})
```

### Wiremock Mock Pattern for Ollama (Existing Test)
```rust
// Source: resyn-core/src/llm/ollama.rs lines 185-241
fn make_ollama_response(content: &str) -> serde_json::Value {
    serde_json::json!({
        "message": {"role": "assistant", "content": content},
        "done": true
    })
}

Mock::given(method("POST"))
    .and(path("/api/chat"))
    .respond_with(
        ResponseTemplate::new(200)
            .set_body_json(make_ollama_response(VALID_ANNOTATION_JSON)),
    )
    .mount(&server)
    .await;
```

### ProgressEvent Current Structure
```rust
// Source: resyn-core/src/datamodels/progress.rs
pub struct ProgressEvent {
    pub event_type: String,        // "crawling", "complete", "idle"
    pub papers_found: u64,
    pub papers_pending: u64,
    pub papers_failed: u64,
    pub current_depth: usize,
    pub max_depth: usize,
    pub elapsed_secs: f64,
    pub current_paper_id: Option<String>,
    pub current_paper_title: Option<String>,
}
```

### Analysis Pipeline Entry Point
```rust
// Source: resyn-server/src/commands/analyze.rs lines 61-106
// run_analysis_pipeline takes &Db, AnalyzeArgs, rate_limit_secs, skip_fulltext
// - Calls run_extraction (ar5iv text fetch with rate limit)
// - Calls run_nlp_analysis (TF-IDF corpus)
// - If provider set: run_llm_analysis + run_gap_analysis
// WARNING: run_extraction calls std::process::exit(1) on fatal errors
pub async fn run_analysis_pipeline(
    db: &Db,
    args: AnalyzeArgs,
    rate_limit_secs: u64,
    skip_fulltext: bool,
) -> anyhow::Result<()>
```

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust/Cargo | Build | Yes | (workspace) | — |
| Ollama CLI | Real LLM testing | No | — | wiremock mock (D-11) |
| Ollama HTTP service | `#[cfg(feature = "ollama-test")]` | No | — | wiremock mock in default tests |
| SurrealDB | DB layer | Yes (in-memory kv-mem) | (crate feature) | — |

**Missing dependencies with no fallback:**
- None that block execution. Ollama is only needed for `#[cfg(feature = "ollama-test")]` optional tests.

**Missing dependencies with fallback:**
- Ollama: not installed/running. Default tests MUST use wiremock. Real Ollama test is feature-gated per D-11.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in (`cargo test`) |
| Config file | none — workspace Cargo.toml |
| Quick run command | `cargo test` |
| Full suite command | `cargo test --all-targets` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| LLM-01 | `StartAnalysis` server fn spawns task and returns immediately | unit | `cargo test start_analysis` | No — Wave 0 |
| LLM-01 | Analysis completes without error with noop provider | integration | `cargo test analysis_pipeline_noop` | No — Wave 0 |
| LLM-01 | Analysis broadcasts progress SSE events (extraction, nlp, llm, gaps, complete) | unit | `cargo test analysis_sse_events` | No — Wave 0 |
| LLM-01 | Analysis broadcasts "analysis_complete" event when done | unit (sub-test above) | included above | No — Wave 0 |
| LLM-02 | `GetGapFindings` returns contradiction findings after analysis run | integration | `cargo test get_gap_findings` | No — Wave 0 |
| LLM-02 | Wiremock Ollama returns YES for contradiction → finding persisted to DB | integration | `cargo test analysis_wiremock_ollama` | No — Wave 0 |
| LLM-03 | `GetOpenProblemsRanked` returns ranked problems after LLM analysis | integration | `cargo test get_open_problems_ranked_after_analysis` | No — Wave 0 |
| LLM-04 | `GetMethodMatrix` returns non-empty matrix after LLM analysis | integration | `cargo test get_method_matrix_after_analysis` | No — Wave 0 |
| LLM-01 | NLP-only mode (no LLM env var) completes extraction + TF-IDF only | unit | `cargo test analysis_noop_nlp_only` | No — Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test`
- **Per wave merge:** `cargo test --all-targets`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `resyn-app/src/server_fns/analysis.rs` — `StartAnalysis` server function (LLM-01)
- [ ] `resyn-server/tests/analysis_pipeline_test.rs` — integration test for full pipeline with wiremock (LLM-01 through LLM-04)
- [ ] Analysis stage `event_type` strings documented in `resyn-core/src/datamodels/progress.rs` — needed for client-side SSE parsing

## Open Questions

1. **ProgressEvent field reuse vs extension**
   - What we know: `event_type: String` already carries semantic meaning; crawl-specific fields (`current_depth`, `max_depth`) are meaningless for analysis stages
   - What's unclear: Whether to add `analysis_stage: Option<String>` or repurpose existing fields
   - Recommendation: Add `analysis_stage: Option<String>` to `ProgressEvent` — zero breaking change (Option is None for crawl events), clear semantics for analysis events

2. **`std::process::exit` removal scope**
   - What we know: `run_extraction` and `run_llm_analysis` call `std::process::exit(1)` on fatal errors
   - What's unclear: Whether to refactor the entire pipeline to return `Result` or only guard the web call path
   - Recommendation: In the `StartAnalysis` background task, use `unwrap_or_else` to catch failures and broadcast `"analysis_error"` rather than calling the pipeline functions that may exit. Defer full pipeline refactor to a future phase.

3. **Analysis guard (prevent concurrent runs)**
   - What we know: `StartCrawl` has no guard either — multiple crawls can run concurrently
   - What's unclear: Whether to add an `Arc<AtomicBool>` or rely on pipeline caching
   - Recommendation: Rely on pipeline's existing `extraction_exists()` / `annotation_exists()` caching. Concurrent runs will redundantly check caches and skip already-processed papers. No guard needed for v1.3.

## Sources

### Primary (HIGH confidence)
- `resyn-server/src/commands/analyze.rs` — complete pipeline source code, all stage functions
- `resyn-app/src/server_fns/papers.rs` — `StartCrawl` pattern (canonical model for `StartAnalysis`)
- `resyn-server/src/commands/serve.rs` — SSE endpoint, server function registration pattern
- `resyn-core/src/datamodels/progress.rs` — `ProgressEvent` struct definition
- `resyn-app/src/components/crawl_progress.rs` — SSE client subscription pattern
- `resyn-core/src/llm/ollama.rs` — Ollama provider, wiremock test fixtures
- `resyn-core/src/llm/noop.rs` — NoopProvider for deterministic testing
- `resyn-core/src/gap_analysis/contradiction.rs` — contradiction pipeline (ssr-gated)
- `resyn-core/src/gap_analysis/abc_bridge.rs` — ABC-bridge pipeline (ssr-gated)
- `resyn-app/src/pages/gaps.rs`, `open_problems.rs`, `methods.rs` — existing result panels

### Secondary (MEDIUM confidence)
- None needed — all relevant architecture is directly observable in the codebase

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all libraries are already present in the workspace
- Architecture: HIGH — `StartCrawl` pattern is directly observable and fully analogous
- Pitfalls: HIGH — identified from direct code reading (`std::process::exit`, missing registration)
- Test strategy: HIGH — existing wiremock tests in `ollama.rs` confirm the pattern

**Research date:** 2026-03-28
**Valid until:** Stable — no external APIs or fast-moving libraries involved. Valid until major Leptos version bump.
