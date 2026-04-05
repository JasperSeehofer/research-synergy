# Phase 18: arXiv Crawl Repair - Context

**Gathered:** 2026-03-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Fix the arXiv HTML reference parser to extract arXiv IDs from plain text in bibliography entries (not just `<a>` tags), so arXiv crawls produce densely connected citation graphs with edge coverage comparable to InspireHEP for the same seed paper.

</domain>

<decisions>
## Implementation Decisions

### ID Extraction Scope
- **D-01:** Extract arXiv IDs in both new format (`YYMM.NNNNN`) and old format (`category/NNNNNNN`) from reference plain text, in addition to existing hyperlink extraction
- **D-02:** Also extract DOI patterns (`10.NNNN/...`) from reference plain text and store as `Reference.doi`. DOIs won't create graph edges but enrich metadata for future use
- **D-03:** When both a hyperlink and a plain-text arXiv ID are found in the same reference, merge both as Links on the Reference and dedup by ID (same arXiv ID not added twice)

### Edge Density Validation
- **D-04:** Verify "comparable edge density" via an automated integration test using wiremock — crawl the same seed paper via both arXiv and InspireHEP sources and assert comparable edge counts
- **D-05:** Use a real arXiv HTML page snapshot as the wiremock fixture (not synthetic HTML) to test against actual HTML structure

### Backward Compatibility
- **D-06:** Only new crawls benefit from the fix — no backfill or re-crawl mechanism. Document that users should delete their local DB (`data/` directory) and re-crawl from scratch after upgrading

### Claude's Discretion
- Regex pattern design for arXiv ID and DOI extraction from plain text
- Where in the parsing pipeline to inject text-based extraction (within `aggregate_references_for_arxiv_paper` or as a separate pass)
- Threshold for "comparable" edge density in the integration test assertion

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Parser Code
- `resyn-core/src/data_aggregation/arxiv_utils.rs` — Contains `aggregate_references_for_arxiv_paper` (the function to fix) and `recursive_paper_search_by_references` (BFS crawler)
- `resyn-core/src/datamodels/paper.rs` — `Reference` struct with `get_arxiv_id()` method that currently only checks `links` for `Journal::Arxiv`; also `arxiv_eprint` field (currently unused by arXiv source)

### Data Models
- `resyn-core/src/datamodels/paper.rs` — `Link`, `Journal` enum, `Reference.doi` field, `Reference.arxiv_eprint` field
- `resyn-core/src/data_aggregation/html_parser.rs` — `ArxivHTMLDownloader` for HTML fetching with rate limiting

### Validation
- `resyn-core/src/validation.rs` — Existing arXiv ID validation patterns (new and old format) — reuse for text extraction regex
- `tests/` — Existing wiremock-based integration tests for arXiv HTML parsing patterns

### Requirements
- `.planning/REQUIREMENTS.md` — ARXIV-01 (plain text ID extraction), ARXIV-03 (comparable edge density)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `validation.rs` has arXiv ID regex patterns for both new (`YYMM.NNNNN`) and old (`category/NNNNNNN`) formats — reuse for text extraction
- `Reference` struct already has `doi: Option<String>` and `arxiv_eprint: Option<String>` fields — text-extracted values can populate these directly
- `Link::from_url()` creates a `Link` with correct `Journal` classification — can be used for text-extracted arXiv IDs by constructing a URL
- Existing wiremock integration tests in `tests/` provide patterns for HTML fixture-based testing

### Established Patterns
- Rate-limited HTML downloads via `ArxivHTMLDownloader` with `Duration`-based configuration
- `strip_version_suffix()` applied at dedup boundaries — text-extracted IDs should also be stripped
- `get_arxiv_id()` on `Reference` returns `Result<String, ResynError>` — may need updating to also check `arxiv_eprint` field

### Integration Points
- `aggregate_references_for_arxiv_paper` is the single function where text extraction should be added
- `get_arxiv_references_ids()` on `Paper` calls `get_arxiv_id()` per reference — this is where text-extracted IDs become graph edges
- DB upsert in `queries.rs` stores references as part of the paper record

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 18-arxiv-crawl-repair*
*Context gathered: 2026-03-28*
