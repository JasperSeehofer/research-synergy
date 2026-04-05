# Phase 19: Data Quality Cleanup - Context

**Gathered:** 2026-03-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Diagnose and eliminate orphan nodes in InspireHEP crawls; backfill missing publication dates for all crawled papers so temporal filtering works. The current system is fully arXiv-ID-keyed — this phase works within that constraint.

</domain>

<decisions>
## Implementation Decisions

### Orphan Diagnosis (ORPH-01)
- **D-01:** One-time code investigation only — no new diagnostic tooling, CLI commands, or persistent logging. The researcher agent traces all code paths that produce orphans through static code analysis.
- **D-02:** No test crawl needed during research — the codebase is small enough to analyze exhaustively by reading code.

### Orphan Elimination (ORPH-02)
- **D-03:** Filter at source — prevent orphan-causing data from entering the pipeline rather than filtering orphans out of the graph after the fact.
- **D-04:** Zero-orphan criteria: every node in the graph must have at least one edge in ANY direction (inbound OR outbound). The seed paper always has outbound edges; leaf nodes have inbound edges from their parent.
- **D-05:** Known orphan causes to investigate and fix: (a) empty-ID papers from InspireHEP, (b) boundary nodes at max_depth with no inbound edges, (c) ID format mismatches between reference storage and paper fetch.

### Published Date Backfill (ARXIV-02)
- **D-06:** Both sources — extract publication dates from InspireHEP API responses AND add arXiv API fallback for papers missing dates.
- **D-07:** InspireHEP: parse publication date during `convert_hit_to_paper()` — currently the `published` field is never set (stays `""`).
- **D-08:** arXiv fallback: enrich during BFS crawl, not as a post-crawl batch pass. When `fetch_paper()` returns a paper, its published date comes with it. Only reference-only papers (never directly fetched) need the fallback.

### Empty-ID Paper Handling
- **D-09:** Skip empty-ID references in the BFS queue — when `get_arxiv_references_ids()` collects IDs for the next BFS depth, filter out empty strings. Keep the reference metadata on the parent paper for future use.
- **D-10:** Do NOT rework the arXiv-keyed ID system in this phase. InspireHEP references with `inspire_record_id` and `doi` but no `arxiv_eprint` are preserved as reference metadata but not crawled or graphed.

### Claude's Discretion
- Specific tracing/debug log messages added during the fix
- How to structure the InspireHEP date field extraction (which JSON field to parse)
- Whether to add a helper method for empty-ID filtering or inline it
- Test strategy for verifying zero orphans (unit test vs integration test)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Crawler and Graph Pipeline
- `resyn-core/src/data_aggregation/arxiv_utils.rs` — `recursive_paper_search_by_references` BFS crawler; collects `get_arxiv_references_ids()` for next depth level
- `resyn-core/src/data_processing/graph_creation.rs` — `create_graph_from_papers` builds graph; edges only created where both endpoints exist in papers list
- `resyn-core/src/datamodels/paper.rs` — `Paper` struct with `published: String` field, `get_arxiv_references_ids()` method, `Reference` struct with `arxiv_eprint`, `inspire_record_id`, `doi` fields

### InspireHEP Source
- `resyn-core/src/data_aggregation/inspirehep_api.rs` — `convert_hit_to_paper()` (missing published date), `convert_references()` (produces empty-ID references), `InspireMetadata` deserialization types

### arXiv Source
- `resyn-core/src/data_aggregation/arxiv_source.rs` — `ArxivSource` PaperSource impl; `fetch_paper` populates published from arxiv-rs
- `resyn-core/src/data_aggregation/arxiv_api.rs` — arXiv API query functions

### Data Model and Validation
- `resyn-core/src/datamodels/paper.rs` — `Paper::from_arxiv_paper()` sets published from `arxiv_paper.published`; `Default` impl sets `published: "".to_string()`
- `resyn-core/src/utils.rs` — `strip_version_suffix()` used at dedup boundaries

### Requirements
- `.planning/REQUIREMENTS.md` — ARXIV-02 (published dates), ORPH-01 (orphan diagnosis), ORPH-02 (zero orphans)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `strip_version_suffix()` — already applied at BFS dedup and graph construction; ensure consistency in any new ID filtering
- `Paper::from_arxiv_paper()` — already correctly populates `published` from arXiv API; pattern to follow for InspireHEP
- `get_arxiv_references_ids()` — the single chokepoint where reference IDs become BFS queue entries; ideal place to filter empty IDs
- Existing graph_creation unit tests — comprehensive coverage of edge cases (version dedup, missing papers, empty input)

### Established Patterns
- BFS crawler logs warnings for individual failures and continues — orphan fixes should follow this resilience pattern
- `convert_hit_to_paper` uses chained `.as_ref().and_then().map()` for optional field extraction — use same pattern for publication date
- InspireHEP deserialization types (`InspireMetadata`, etc.) in inspirehep_api.rs — may need new fields for publication date

### Integration Points
- `convert_hit_to_paper()` — where InspireHEP published date extraction should be added
- `get_arxiv_references_ids()` on Paper — where empty-ID filtering should be added
- `recursive_paper_search_by_references()` — where any enrichment for reference-only papers would hook in
- DB upsert in `queries.rs` — published date will flow through existing upsert path

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches

</specifics>

<deferred>
## Deferred Ideas

- **Universal paper ID system** — Rework the arXiv-keyed ID system to support DOI, InspireHEP record ID, or other identifiers as primary keys. Needed for extending to non-arXiv sources. Significant architectural change touching data model, graph construction, BFS crawler, DB schema, and frontend. Future milestone.
- **Reference metadata for non-arXiv papers** — InspireHEP references with `inspire_record_id` and `doi` but no `arxiv_eprint` are currently stored but not crawled. The universal ID system would unlock these as graph nodes.

</deferred>

---

*Phase: 19-data-quality-cleanup*
*Context gathered: 2026-03-28*
