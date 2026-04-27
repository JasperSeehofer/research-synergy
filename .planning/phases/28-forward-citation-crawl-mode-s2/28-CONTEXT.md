# Phase 28: Forward-citation crawl mode (S2) - Context

**Gathered:** 2026-04-27
**Status:** Ready for planning
**Source:** PRD Express Path (/home/jasper/.claude/plans/federated-yawning-haven.md)

<domain>
## Phase Boundary

This phase adds bidirectional citation discovery to the SemanticScholarSource. Pre-2015 cond-mat seeds have `externalIds: null` in S2 backward-citations, making the graph terminate after one hop. The S2 `/paper/{id}/citations` (forward-citations) endpoint does return resolved arXiv IDs for the same seeds and is the only viable expansion path. We implement a `--bidirectional` crawl mode that fetches both directions and merges discoveries into the existing BFS queue while preserving graph edge directionality.

**Explicitly out of scope:** Generalising forward-citations to InspireHEP or arXiv sources; backfilling already-crawled corpora.

</domain>

<decisions>
## Implementation Decisions

### API Endpoint
- D-01: Forward-citation discovery uses only the S2 `/paper/arXiv:{id}/citations?fields=externalIds,title,authors,year&limit=500&offset=N` endpoint. No other source is extended.

### CLI Interface
- D-02: The feature is opt-in via a new `--bidirectional` CLI flag on the `crawl` subcommand (default off). Non-S2 sources receive a `tracing::warn!` if the flag is set but the source doesn't support it; they do not error.
- D-03: A new `--max-forward-citations N` CLI flag (default 500) caps citing papers discovered per seed paper. Pagination stops when the cap is reached. This prevents enqueue blowup on highly-cited seeds.

### Graph Correctness
- D-04: Forward-citation edges MUST be persisted via a new `PaperRepository::upsert_inverse_citations_batch(&self, cited_arxiv_id: &str, citing_papers: &[Reference])` method. This method writes edges as `(from = citing_arxiv_id, to = cited_arxiv_id)` — the correct direction. Naively merging forward citations into `paper.references` and using the existing `upsert_citations` is forbidden because it would invert the graph direction.

### Trait Design
- D-05: The `PaperSource` trait gains a new `async fn fetch_citing_papers(&mut self, paper: &mut Paper) -> Result<(), ResynError>` with a no-op default implementation (`Ok(())`). Only `SemanticScholarSource` overrides it. All other implementors (ArxivSource, InspireHepClient, ChainedPaperSource) inherit the default with no code changes needed.

### Data Model
- D-06: `Paper` gains a transient `citing_papers: Vec<Reference>` field annotated `#[serde(default, skip_serializing)]`. It exists only for the duration of the crawl-loop iteration; it does not get written to the DB schema or appear in JSON exports. `Paper::get_citing_arxiv_ids(&self) -> Vec<String>` is added alongside the existing `get_arxiv_references_ids`.

### SemanticScholarSource Internals
- D-07-a: `SemanticScholarSource` gains `bidirectional: bool` (default false) and `max_forward_citations: usize` (default 500) fields, plus `with_bidirectional(self, bool)` and `with_max_forward_citations(self, usize)` builder methods.
- D-07-b: New deserializers `S2CitationsPage { data: Vec<S2CitationItem>, next: Option<u32> }` and `S2CitationItem { citingPaper: S2Paper }` mirror the existing `S2RefsPage` shape.
- D-07-c: `fetch_citing_papers` reuses `get_with_backoff`, paginates until `next: None` or cap reached. 404 returns `Ok(vec![])` silently (mirrors `fetch_references` behaviour).

### Docs Refresh
- D-08: The stale CLAUDE.md paragraph about the `ChainedPaperSource` empty-refs bug is removed — the bug was fixed in `chained_source.rs:52-85`. CLAUDE.md is updated with `--bidirectional` and `--max-forward-citations` in the crawl argument table and a note under Important Notes.

### Claude's Discretion
- Whether to extract `S2Paper → Reference` conversion into a private helper (recommended for code reuse between `convert_s2_refs` and the new citations converter)
- Exact SurrealQL statement inside `upsert_inverse_citations_batch` (mirror existing `upsert_citations_batch` syntax)
- Location of new unit tests (in-file vs integration test file, following existing conventions)
- `crawl-feynman-seeds.sh` exact flag placement and order

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### S2 Source (Plan 01)
- `resyn-core/src/data_aggregation/semantic_scholar_api.rs:84-132` — `get_with_backoff` (reuse)
- `resyn-core/src/data_aggregation/semantic_scholar_api.rs:176-305` — `convert_s2_refs`, `fetch_references` pagination pattern
- `resyn-core/tests/semantic_scholar_integration.rs:55-200` — wiremock test pattern

### Trait + Datamodel (Plan 02)
- `resyn-core/src/data_aggregation/traits.rs:1-20` — `PaperSource` trait
- `resyn-core/src/datamodels/paper.rs:17-79` — `Paper` struct and `get_arxiv_references_ids`
- `resyn-core/src/datamodels/paper.rs:106-139` — `Reference` struct and `get_arxiv_id`

### Database (Plan 03)
- `resyn-core/src/database/queries.rs:120-260` — `upsert_citations`, `upsert_citations_batch`, `get_citing_papers`
- `resyn-core/src/database/queries.rs:1-60` — `PaperRecord` shape

### Crawler (Plan 04)
- `resyn-server/src/commands/crawl.rs:47-140` — `CrawlArgs`, `make_single_source`, `make_source`
- `resyn-server/src/commands/crawl.rs:290-464` — worker loop, enqueue at ~line 388-400
- `scripts/crawl-feynman-seeds.sh` — seed crawl script
- `CLAUDE.md:170-180` — stale bug note to update

</canonical_refs>

<specifics>
## Specific Ideas

**S2 citations endpoint (verified working):**
```
GET /graph/v1/paper/arXiv:{id}/citations?fields=externalIds,title,authors,year&limit=500&offset=N
Response: {"data": [{"citingPaper": {...}}, ...], "next": N}
```

**Smoke test command (post-implementation):**
```bash
cargo run --release --bin resyn -- crawl \
  --paper-id 1411.4903 \
  --source semantic_scholar \
  --bidirectional \
  --max-forward-citations 50 \
  --max-depth 1 \
  --db surrealkv:///tmp/resyn-bidir-test
```

**Reused existing code (no new implementations):**
- `SemanticScholarSource::get_with_backoff` — backoff/retry/rate-limit
- `SemanticScholarSource::convert_s2_refs` / factored `S2Paper → Reference` helper
- `Reference::get_arxiv_id` — handles `arxiv_eprint` vs `Link` already
- `PaperRepository::get_citing_papers` — used for test assertions in Plan 03
- `PaperRepository::upsert_citations_batch` — template for inverse-direction method
- `queue_repo.enqueue_if_absent` — dedup handled, no change needed

</specifics>

<deferred>
## Deferred Ideas

- Generalising `fetch_citing_papers` to InspireHEP or arXiv sources (future phase)
- Backfilling forward citations on already-crawled corpora (batch tool, separate phase)
- Year-filter on forward citations (`--forward-citations-since YYYY`) — the `--max-forward-citations` cap was chosen as the simpler bound

</deferred>

---

*Phase: 28-forward-citation-crawl-mode-s2*
*Context gathered: 2026-04-27 via PRD Express Path*
