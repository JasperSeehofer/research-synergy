# Phase 27: Crawler Speedup - Context

**Gathered:** 2026-04-22
**Status:** Ready for planning

<domain>
## Phase Boundary

Replace OpenAlex `--mailto` polite-pool authentication with free API key auth; add a `DEFAULT_FILTER_PHYSICS` constant for cond-mat/stat-phys corpus ingest; fix the wrong concept ID in CLAUDE.md; and update CLAUDE.md bulk-ingest documentation to reflect all changes.

This is a pure infrastructure improvement — no UI changes, no new traits, no new crates required.

</domain>

<decisions>
## Implementation Decisions

### API Key Migration
- **D-01:** Replace `--mailto` arg and `DEFAULT_MAILTO` constant with `--api-key` arg that reads from `OPENALEX_API_KEY` env var (clap `env = "OPENALEX_API_KEY"`). Hard remove `--mailto` — no deprecation shim needed (single-user CLI).
- **D-02:** If `OPENALEX_API_KEY` is not set at runtime, **hard fail immediately** with a clear error message: `"OPENALEX_API_KEY not set — register a free key at openalex.org/settings/api"`. Do not fall back to unauthenticated.
- **D-03:** In `openalex_bulk.rs`, replace `mailto: String` field with `api_key: String`; remove `?mailto=...` from URL; add `Authorization: Bearer {api_key}` header. Keep `User-Agent` header for courtesy.

### Physics Corpus Filter
- **D-04:** Add `DEFAULT_FILTER_PHYSICS` constant to `bulk_ingest.rs`:
  ```
  "primary_location.source.id:S4306400194,concepts.id:C26873012|C121864883"
  ```
  Where `C26873012` = Condensed matter physics (2.9 M works), `C121864883` = Statistical physics (1.9 M works), `S4306400194` = arXiv.
- **D-05:** The existing `DEFAULT_FILTER` (ML/stat.ML/NeuralNet) remains unchanged. `DEFAULT_FILTER_PHYSICS` is an additional constant for the EXP-RS-07 use case.

### CLAUDE.md Update Scope
- **D-06:** Full update — fix wrong concept ID (`C2778407487` → documented as Altmetrics, not Statistical Physics; correct IDs are `C26873012` and `C121864883`), update the `bulk-ingest` example command to show `--api-key "$OPENALEX_API_KEY"` and the physics filter, and note the removal of `--mailto`.

### T3 — arXiv id_list Batching
- **D-07:** Deferred. arXiv metadata batching via `id_list` parameter addresses the BFS crawler metadata path only (0× benefit for reference scraping). Out of scope for Phase 27; note for a future crawler optimization phase.

### Claude's Discretion
- Exact error message wording for missing API key (keep actionable — include the openalex.org URL)
- Whether to add a unit test for the `Authorization: Bearer` header injection (research test map suggests modifying existing `openalex_bulk` tests)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Existing Code (files to touch)
- `resyn-core/src/data_aggregation/openalex_bulk.rs` — Replace `mailto` with `api_key`; update header
- `resyn-server/src/commands/bulk_ingest.rs` — Replace `--mailto` with `--api-key`/env var; add `DEFAULT_FILTER_PHYSICS`
- `CLAUDE.md` — Fix concept ID; update bulk-ingest docs

### Research
- `.planning/phases/27-crawler-speedup/27-RESEARCH.md` — Verified concept IDs, API key auth pattern, pitfalls, code examples. HIGH confidence throughout. **Read before planning.**

### No external specs — requirements fully captured in decisions above.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `OpenAlexBulkLoader::new(client, &args.mailto)` in `bulk_ingest.rs:52` — change second arg to `&args.api_key`
- `BulkIngestArgs` struct in `bulk_ingest.rs:16-36` — add `--api-key` field, remove `--mailto`
- Two-phase JSONL spill pattern (`bulk_ingest.rs`) — unchanged; already correct
- `upsert_citations_batch` — unchanged

### Established Patterns
- clap `env = "..."` pattern already used elsewhere in the codebase for env-var-backed args
- `ResynError` enum for error propagation — add `OpenAlexApi` variant message for missing key (or use `tracing::error!` + `std::process::exit(1)` like the DB connect error at `bulk_ingest.rs:43`)

### Integration Points
- `OpenAlexBulkLoader::new()` call at `bulk_ingest.rs:52` — only callsite to update
- The `fetch_page()` URL builder in `openalex_bulk.rs` — remove `mailto` param, add `Authorization` header

</code_context>

<specifics>
## Specific Ideas

- Research Example 1 (`27-RESEARCH.md`) has the exact updated `fetch_page` implementation with correct header injection — use it directly.
- Research Example 2 has the updated `BulkIngestArgs` struct with `--api-key` — use it directly.
- Physics filter invocation (Example 4) should be the new canonical example in CLAUDE.md.

</specifics>

<deferred>
## Deferred Ideas

- **arXiv id_list batching (T3)** — `fetch_papers_batch(ids: &[&str])` in `arxiv_api.rs` using `id_list` param. Metadata-only speedup (200× for BFS metadata, 0× for reference scraping). Defer to a future crawler optimization phase.

</deferred>

---

*Phase: 27-crawler-speedup*
*Context gathered: 2026-04-22*
