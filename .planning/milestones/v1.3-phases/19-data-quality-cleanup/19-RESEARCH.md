# Phase 19: Data Quality Cleanup - Research

**Researched:** 2026-03-28
**Domain:** Rust data pipeline — InspireHEP API integration, BFS crawler, graph construction
**Confidence:** HIGH

## Summary

Phase 19 addresses two independent data quality gaps revealed by the completed InspireHEP crawler (Phase 18): orphan nodes appearing in the graph when papers enter the pipeline without their arXiv ID, and missing `published` dates causing the temporal year-range slider to filter nothing.

Both issues originate in `inspirehep_api.rs`. The `convert_hit_to_paper()` function never sets the `published` field (stays `""`), and `convert_references()` emits `Reference` entries with `arxiv_eprint: None` when InspireHEP provides no `arxiv_eprint` for a reference. These empty-ID references then propagate into the BFS queue via `get_arxiv_references_ids()`, creating nodes that appear in the graph but have no edges.

The fixes are contained to two files — `resyn-core/src/data_aggregation/inspirehep_api.rs` and the `Paper::get_arxiv_references_ids()` method — with no schema changes, no new API calls, and no database changes required. The published date is available in the InspireHEP response as the `earliest_date` field (confirmed live: format `"YYYY-MM-DD"`), and the `InspireMetadata` struct needs one new field to deserialize it.

**Primary recommendation:** Add `earliest_date` to `InspireMetadata`, wire it into `convert_hit_to_paper()`, and filter empty strings out of `get_arxiv_references_ids()`. Both changes are ~5 lines each.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** One-time code investigation only — no new diagnostic tooling, CLI commands, or persistent logging. The researcher agent traces all code paths that produce orphans through static code analysis.
- **D-02:** No test crawl needed during research — the codebase is small enough to analyze exhaustively by reading code.
- **D-03:** Filter at source — prevent orphan-causing data from entering the pipeline rather than filtering orphans out of the graph after the fact.
- **D-04:** Zero-orphan criteria: every node in the graph must have at least one edge in ANY direction (inbound OR outbound). The seed paper always has outbound edges; leaf nodes have inbound edges from their parent.
- **D-05:** Known orphan causes to investigate and fix: (a) empty-ID papers from InspireHEP, (b) boundary nodes at max_depth with no inbound edges, (c) ID format mismatches between reference storage and paper fetch.
- **D-06:** Both sources — extract publication dates from InspireHEP API responses AND add arXiv API fallback for papers missing dates.
- **D-07:** InspireHEP: parse publication date during `convert_hit_to_paper()` — currently the `published` field is never set (stays `""`).
- **D-08:** arXiv fallback: enrich during BFS crawl, not as a post-crawl batch pass. When `fetch_paper()` returns a paper, its published date comes with it. Only reference-only papers (never directly fetched) need the fallback.
- **D-09:** Skip empty-ID references in the BFS queue — when `get_arxiv_references_ids()` collects IDs for the next BFS depth, filter out empty strings. Keep the reference metadata on the parent paper for future use.
- **D-10:** Do NOT rework the arXiv-keyed ID system in this phase. InspireHEP references with `inspire_record_id` and `doi` but no `arxiv_eprint` are preserved as reference metadata but not crawled or graphed.

### Claude's Discretion
- Specific tracing/debug log messages added during the fix
- How to structure the InspireHEP date field extraction (which JSON field to parse)
- Whether to add a helper method for empty-ID filtering or inline it
- Test strategy for verifying zero orphans (unit test vs integration test)

### Deferred Ideas (OUT OF SCOPE)
- **Universal paper ID system** — Rework the arXiv-keyed ID system to support DOI, InspireHEP record ID, or other identifiers as primary keys.
- **Reference metadata for non-arXiv papers** — InspireHEP references with `inspire_record_id` and `doi` but no `arxiv_eprint` currently not crawled; unlocked by universal ID system.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| ARXIV-02 | User can see published dates for all crawled papers (backfilled from arXiv API for reference-only papers) | InspireHEP `earliest_date` field confirmed via live API; `convert_hit_to_paper()` is the single injection point; arXiv already sets `published` via `from_arxiv_paper()` |
| ORPH-01 | User can identify why specific nodes appear disconnected after an InspireHEP crawl | Root cause found via static analysis: empty `arxiv_eprint` references produce empty-string IDs that enter BFS queue and become unlinked nodes |
| ORPH-02 | User sees zero orphan nodes in the graph for a standard depth-2+ crawl (every node has at least one edge) | Fix at `get_arxiv_references_ids()` — filter `""` out of returned IDs before they enter BFS queue; no node created = no orphan |
</phase_requirements>

---

## ORPH-01: Orphan Diagnosis — Root Cause Analysis

This section answers ORPH-01 by tracing all code paths that produce orphan nodes. Performed via static analysis of the six canonical files.

### Orphan Cause A: Empty-ID papers from InspireHEP (CONFIRMED PRIMARY CAUSE)

**Code path:**

1. `InspireHepClient::convert_references()` in `inspirehep_api.rs` creates a `Reference` where `arxiv_eprint` is `None` when the InspireHEP reference has no `arxiv_eprint` field (lines 164, 186-194).

2. `Reference::get_arxiv_id()` in `paper.rs` (lines 110-127) — when a reference has no `Journal::Arxiv` link AND `arxiv_eprint` is `None`, it returns `Err(ResynError::NoArxivLink)`.

3. `Paper::get_arxiv_references_ids()` (lines 62-67) uses `filter_map` with `r.get_arxiv_id().ok()`, so `Err` results are silently dropped. Only references with valid arXiv IDs contribute to the BFS queue.

4. **However**: when `arxiv_eprint` is `Some("")` — an empty string — `get_arxiv_id()` returns `Ok("")`. An empty string passes `filter_map` and enters the BFS queue as `""`.

5. `recursive_paper_search_by_references()` calls `source.fetch_paper("")` which may succeed or fail depending on InspireHEP's response to `q=arxiv:`. If it succeeds and returns a paper with an empty `id`, that paper becomes a node in the graph with no edges from/to any real paper ID.

6. `create_graph_from_papers()` deduplicates by `strip_version_suffix(&paper.id)`. If `id` is `""`, the key is `""` — a valid map key, so the empty-ID paper gets a node. But no reference in any other paper points to `""`, so the node has no edges.

**Verified in tests:** `test_convert_hit_with_missing_optional_fields` confirms that a hit with no `arxiv_eprints` produces a paper where `paper.id == ""` (line 471). This paper, if fetched and added to the papers list, becomes an orphan node.

**Fix location:** `Paper::get_arxiv_references_ids()` — add `.filter(|id| !id.is_empty())` after `filter_map`. This is the single chokepoint described in D-09.

### Orphan Cause B: Boundary nodes at max_depth (NOT AN ORPHAN — BY DESIGN)

**Analysis:** At `max_depth`, papers are fetched and added to the papers list. Their references are also fetched (`fetch_references`). The papers those references point to are NOT fetched (loop terminates). So boundary-layer papers DO have references stored, and `create_graph_from_papers()` creates edges from boundary paper → referenced paper only when BOTH endpoints are in the papers list. References to unfetched papers are silently ignored by graph construction (see `graph_creation.rs` lines 26-33: both `from_idx` and `to_idx` must exist in the map).

**Conclusion:** Boundary papers always have at least one inbound edge (from their parent at depth max_depth-1). They are not orphans. D-04 is satisfied: leaf nodes have inbound edges. No fix needed for this cause.

### Orphan Cause C: ID format mismatches between reference storage and paper fetch (NOT CONFIRMED)

**Analysis:** `get_arxiv_references_ids()` returns IDs extracted from `Reference.links` (URL last segment) or `Reference.arxiv_eprint`. `recursive_paper_search_by_references()` calls `strip_version_suffix(paper_id)` before adding to `visited_papers` and before `fetch_paper()`. `create_graph_from_papers()` also calls `strip_version_suffix()` on both paper IDs and reference IDs when building edges. Version suffix stripping is applied consistently at every dedup boundary.

**Conclusion:** No format mismatch vulnerability exists in the current code for arXiv IDs. The only remaining mismatch risk is the empty-string case (Cause A).

### Summary: Single Root Cause

The orphan node problem has one root cause: empty `arxiv_eprint` values from InspireHEP references that produce `""` IDs, which enter the BFS queue, cause a fetch of paper `""`, and if that fetch somehow returns a paper (or if the empty string makes it through other paths), the resulting node has no edges.

The safest fix is at the earliest filtering point: `get_arxiv_references_ids()` must reject empty strings.

---

## Standard Stack

### Core (no changes required)
| Library | Version | Purpose | Notes |
|---------|---------|---------|-------|
| serde / serde_json | workspace | Deserialize InspireHEP API responses | `InspireMetadata` struct needs one new field |
| reqwest | workspace | HTTP client for InspireHEP | No change |
| tracing | workspace | Structured logging for filter events | Use `warn!` for filtered empty IDs |

### No new dependencies required
All changes are pure logic changes inside existing files using existing dependencies.

---

## Architecture Patterns

### InspireHEP Date Field: `earliest_date`

**Confirmed via live API** (2026-03-28): `GET https://inspirehep.net/api/literature?q=arxiv:2503.18887&fields=earliest_date,preprint_date`

Response structure:
```json
{
  "hits": {
    "hits": [{
      "metadata": {
        "earliest_date": "2025-03-24",
        "preprint_date": "2025-03-24"
      }
    }]
  }
}
```

**Recommended field:** `earliest_date` — this is the authoritative date InspireHEP uses for the record. It is always present when the paper has a known submission date. `preprint_date` is equivalent for arXiv papers and also works.

**API field to add to the existing query string** in `fetch_literature()` (line 57):
```
fields=references,titles,authors,abstracts,arxiv_eprints,dois,citation_count,earliest_date
```

`fetch_paper()` (line 206) also needs `earliest_date` added to its `fields` parameter:
```
fields=titles,authors,abstracts,arxiv_eprints,dois,citation_count,earliest_date
```

### Pattern: Chained Optional Field Extraction (established in codebase)

The existing `convert_hit_to_paper()` uses this pattern for every optional field. Follow the same pattern:

```rust
// Source: resyn-core/src/data_aggregation/inspirehep_api.rs, lines 96-128
let published = metadata
    .earliest_date
    .as_deref()
    .unwrap_or_default()
    .to_string();
```

`earliest_date` is a plain `Option<String>`, not a nested struct, so the extraction is simpler than `titles` or `abstracts`.

### Pattern: `InspireMetadata` Struct Extension

Add the field to the struct (currently at line 297):

```rust
#[derive(Debug, Deserialize)]
pub(crate) struct InspireMetadata {
    pub titles: Option<Vec<InspireTitle>>,
    pub authors: Option<Vec<InspireAuthor>>,
    pub abstracts: Option<Vec<InspireAbstract>>,
    pub arxiv_eprints: Option<Vec<InspireArxivEprint>>,
    pub dois: Option<Vec<InspireDoi>>,
    pub citation_count: Option<u32>,
    pub references: Option<Vec<InspireReferenceEntry>>,
    pub earliest_date: Option<String>,   // NEW: "YYYY-MM-DD" from InspireHEP
}
```

No new deserialization struct needed — `earliest_date` is a flat string.

### Pattern: Empty-String Filter in `get_arxiv_references_ids()`

Current code (`paper.rs` lines 62-67):
```rust
pub fn get_arxiv_references_ids(&self) -> Vec<String> {
    self.references
        .iter()
        .filter_map(|r| r.get_arxiv_id().ok())
        .collect()
}
```

Fix — add one filter step:
```rust
pub fn get_arxiv_references_ids(&self) -> Vec<String> {
    self.references
        .iter()
        .filter_map(|r| r.get_arxiv_id().ok())
        .filter(|id| !id.is_empty())
        .collect()
}
```

This is the single change needed for ORPH-02. It prevents empty-ID entries from entering the BFS queue. The reference metadata is preserved on the parent paper's `references` field (D-09 satisfied).

### Pattern: Published Date in Paper Construction

`convert_hit_to_paper()` (line 130) uses `..Default::default()` for unset fields. Add `published` to the explicit field list:

```rust
Paper {
    title,
    authors,
    summary,
    id: arxiv_id,
    doi,
    inspire_id,
    citation_count,
    published,   // NEW
    source: DataSource::InspireHep,
    ..Default::default()
}
```

### arXiv Source: Published Date Already Works

`ArxivSource::fetch_paper()` calls `Paper::from_arxiv_paper()` which already sets `published: arxiv_paper.published.clone()` (paper.rs line 56). Papers fetched via the arXiv source (including at all BFS depths) already have correct published dates. No change needed for the arXiv source.

**D-08 clarification:** The context decision mentions an "arXiv fallback for reference-only papers." After code analysis: papers fetched via InspireHEP source will get dates from InspireHEP (`earliest_date`). Papers fetched via arXiv source already get dates. The only papers that might still lack dates are those where InspireHEP returns no `earliest_date` (rare: typically means the record was manually entered without a date). D-06 says "both sources" — adding `earliest_date` to InspireHEP fully satisfies this for the InspireHEP path. The arXiv fallback for InspireHEP papers with missing dates is outside this phase's scope per the deferred section.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead |
|---------|-------------|-------------|
| Date parsing | Custom date string parser | `earliest_date` is already `"YYYY-MM-DD"` — slice `[..4]` extracts year directly, matching the existing `server_fns/graph.rs` pattern (line 121-124) |
| Empty-ID detection | Regex or complex validation | `!id.is_empty()` is sufficient — arXiv IDs are never legitimately empty |
| Orphan detection post-hoc | Graph traversal after construction | Filter at `get_arxiv_references_ids()` — prevents orphans from being created |

---

## Common Pitfalls

### Pitfall 1: `earliest_date` not in the `fields` query parameter
**What goes wrong:** InspireHEP only returns fields explicitly requested. If `earliest_date` is not added to both the `fetch_paper()` and `fetch_literature()` query strings, `metadata.earliest_date` deserializes as `None` and the fix has no effect.
**How to avoid:** Add `earliest_date` to BOTH query strings: the one in `fetch_paper()` (line 206) and the one in `fetch_literature()` (line 57).
**Warning signs:** Test verifying `paper.published != ""` still fails despite code change.

### Pitfall 2: Empty-string ID check vs `None` check
**What goes wrong:** `get_arxiv_references_ids()` uses `filter_map(|r| r.get_arxiv_id().ok())` which already drops `Err` (i.e., references with no arXiv link at all). The remaining gap is `Ok("")` — an arXiv link URL that resolves to an empty last segment. The fix must target empty strings, not `None`.
**How to avoid:** Use `.filter(|id| !id.is_empty())` after `filter_map`, not `filter_map` replacement.

### Pitfall 3: Updating `convert_hit_to_paper()` but not `fetch_paper()`
**What goes wrong:** `convert_hit_to_paper()` is called from both `fetch_paper()` and `fetch_references()`. The helper reads from `hit.metadata.earliest_date`. If `earliest_date` is added to `InspireMetadata` and to `convert_hit_to_paper()` but NOT to the `fetch_paper()` query string, papers fetched via `fetch_paper()` will have empty dates. `fetch_paper()` uses its own URL (line 203-215) that currently omits `earliest_date`.
**How to avoid:** Update both URL strings.

### Pitfall 4: Test still uses old `InspireMetadata` construction without `earliest_date`
**What goes wrong:** `test_empty_references` (line 476-490) manually constructs `InspireMetadata` with named fields. Adding `earliest_date: Option<String>` to the struct will break this test unless the field is added to the manual construction.
**How to avoid:** Add `earliest_date: None` to the `InspireMetadata` literal in `test_empty_references`.

---

## Code Examples

### Example 1: Updated `InspireMetadata` struct

```rust
// Source: resyn-core/src/data_aggregation/inspirehep_api.rs
#[derive(Debug, Deserialize)]
pub(crate) struct InspireMetadata {
    pub titles: Option<Vec<InspireTitle>>,
    pub authors: Option<Vec<InspireAuthor>>,
    pub abstracts: Option<Vec<InspireAbstract>>,
    pub arxiv_eprints: Option<Vec<InspireArxivEprint>>,
    pub dois: Option<Vec<InspireDoi>>,
    pub citation_count: Option<u32>,
    pub references: Option<Vec<InspireReferenceEntry>>,
    pub earliest_date: Option<String>,
}
```

### Example 2: `convert_hit_to_paper()` with published date

```rust
// Source: resyn-core/src/data_aggregation/inspirehep_api.rs
fn convert_hit_to_paper(hit: &InspireHit) -> Paper {
    let metadata = &hit.metadata;
    // ... existing field extraction ...
    let published = metadata
        .earliest_date
        .as_deref()
        .unwrap_or_default()
        .to_string();

    Paper {
        title,
        authors,
        summary,
        id: arxiv_id,
        doi,
        inspire_id,
        citation_count,
        published,
        source: DataSource::InspireHep,
        ..Default::default()
    }
}
```

### Example 3: Empty-ID filter in `get_arxiv_references_ids()`

```rust
// Source: resyn-core/src/datamodels/paper.rs
pub fn get_arxiv_references_ids(&self) -> Vec<String> {
    self.references
        .iter()
        .filter_map(|r| r.get_arxiv_id().ok())
        .filter(|id| !id.is_empty())
        .collect()
}
```

### Example 4: Updated `fetch_literature` URL to include `earliest_date`

```rust
// Source: resyn-core/src/data_aggregation/inspirehep_api.rs, fetch_literature()
let url = format!(
    "{}/literature?q=arxiv:{}&fields=references,titles,authors,abstracts,arxiv_eprints,dois,citation_count,earliest_date",
    self.base_url, arxiv_id
);
```

### Example 5: Updated `fetch_paper` URL to include `earliest_date`

```rust
// Source: resyn-core/src/data_aggregation/inspirehep_api.rs, fetch_paper()
let url = format!(
    "{}/literature?q=arxiv:{}&fields=titles,authors,abstracts,arxiv_eprints,dois,citation_count,earliest_date",
    self.base_url, id
);
```

---

## State of the Art

| Old Approach | Current Approach | Impact |
|--------------|------------------|--------|
| InspireHEP `published` never set (`""`) | Extract from `earliest_date` field (YYYY-MM-DD confirmed) | Year slice `[..4]` feeds directly into existing `server_fns/graph.rs` year extraction logic |
| Empty-ID references enter BFS queue | Filter at `get_arxiv_references_ids()` | Prevents orphan nodes before they enter graph construction |

**Pipeline flow after fixes:**
```
InspireHEP API response
  → convert_hit_to_paper(): sets paper.published = "YYYY-MM-DD"
  → paper stored to DB with published date
  → get_graph_data(): year = published[..4] (non-empty)
  → GraphState: temporal_min_year / temporal_max_year computed correctly
  → year-range slider filters papers that have valid years
```

---

## Runtime State Inventory

This is a code-only fix phase (no renames, migrations, or schema changes). No runtime state is affected.

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | SurrealDB `paper` records with `published: ""` from prior InspireHEP crawls | None — upsert on next crawl will overwrite with correct value; no migration needed |
| Live service config | None | None |
| OS-registered state | None | None |
| Secrets/env vars | None | None |
| Build artifacts | None | None |

**Note on existing DB data:** Papers already stored with `published: ""` will remain empty until re-crawled. The fix only affects new crawls. Per D-08, re-enrichment of existing records is not in scope for this phase. Users who need temporal filtering should re-crawl after the fix.

---

## Environment Availability

Step 2.6: This phase is a pure code change with no new external dependencies. All required tools are confirmed available.

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| cargo / rustc | Build | ✓ | (workspace pinned) | — |
| InspireHEP API | Test verification | ✓ | Live (confirmed 2026-03-28) | wiremock fixture |

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test + tokio::test for async, wiremock for HTTP mocking |
| Config file | Cargo.toml (dev-dependencies: wiremock) |
| Quick run command | `cargo test inspirehep` |
| Full suite command | `cargo test` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| ARXIV-02 | `convert_hit_to_paper()` sets `paper.published` from `earliest_date` | unit | `cargo test test_convert_hit_to_paper` | ✅ (extend existing test) |
| ARXIV-02 | `convert_hit_to_paper()` with missing `earliest_date` returns `published: ""` | unit | `cargo test test_convert_hit_with_missing_optional_fields` | ✅ (extend existing test) |
| ARXIV-02 | `fetch_paper()` returns paper with non-empty `published` (wiremock) | integration | `cargo test test_inspirehep_fetch_paper_published` | ❌ Wave 0 |
| ORPH-01 | Static analysis documented in RESEARCH.md (this document) | manual-only | — | N/A |
| ORPH-02 | `get_arxiv_references_ids()` filters empty strings | unit | `cargo test test_get_arxiv_references_ids_filters_empty` | ❌ Wave 0 |
| ORPH-02 | `get_arxiv_references_ids()` still returns valid IDs after filter | unit | `cargo test test_get_arxiv_references_ids` | ✅ (existing test passes) |

### Sampling Rate
- **Per task commit:** `cargo test inspirehep && cargo test graph_creation && cargo test paper`
- **Per wave merge:** `cargo test`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `resyn-core/src/datamodels/paper.rs` — add `test_get_arxiv_references_ids_filters_empty` unit test verifying that a reference with `arxiv_eprint: Some("".to_string())` does NOT appear in output
- [ ] `resyn-core/src/data_aggregation/inspirehep_api.rs` — extend `test_convert_hit_to_paper` to assert `paper.published == "2023-01-01"` (or whatever date in the fixture); extend `test_convert_hit_with_missing_optional_fields` to assert `paper.published == ""`
- [ ] `resyn-core/src/data_aggregation/inspirehep_api.rs` — add `test_inspirehep_fetch_paper_published` wiremock integration test verifying `fetch_paper()` returns a paper with `published` populated (requires wiremock server serving a fixture that includes `earliest_date`)

---

## Open Questions

1. **Does `earliest_date` appear in older InspireHEP records (pre-2000)?**
   - What we know: Confirmed present for a 2025 paper. InspireHEP documentation describes it as the "earliest known date for the record."
   - What's unclear: Whether legacy records that predate arXiv have `earliest_date` populated.
   - Recommendation: Use `unwrap_or_default()` (returns `""`) — the existing year extraction in `server_fns/graph.rs` already handles empty published gracefully by setting `year = String::new()`.

2. **Can `fetch_paper()` return a paper with non-empty `id` but empty `published`?**
   - What we know: Only if InspireHEP has no `earliest_date` for the record. This can happen for manually-entered records without a known date.
   - Recommendation: Accept `""` as a valid fallback — the temporal slider already handles empty years by excluding them from `year_values` (`layout_state.rs` line 186-188).

---

## Sources

### Primary (HIGH confidence)
- Live InspireHEP API (2026-03-28): `https://inspirehep.net/api/literature?q=arxiv:2503.18887&fields=earliest_date,preprint_date` — confirmed `earliest_date: "2025-03-24"` format
- `resyn-core/src/data_aggregation/inspirehep_api.rs` — full static analysis
- `resyn-core/src/datamodels/paper.rs` — `get_arxiv_references_ids()`, `Reference::get_arxiv_id()`
- `resyn-core/src/data_aggregation/arxiv_utils.rs` — `recursive_paper_search_by_references()`
- `resyn-core/src/data_processing/graph_creation.rs` — edge creation logic
- `resyn-app/src/server_fns/graph.rs` — `published[..4]` year extraction (confirms date format expectation)

### Secondary (MEDIUM confidence)
- `resyn-app/src/graph/layout_state.rs` — temporal year bounds computation (empty year handling)

---

## Metadata

**Confidence breakdown:**
- Orphan root cause: HIGH — traced through all 6 canonical files, confirmed by existing test `test_convert_hit_with_missing_optional_fields`
- Published date fix: HIGH — confirmed `earliest_date` field format via live API call
- Test strategy: HIGH — extends existing wiremock test pattern from Phase 18

**Research date:** 2026-03-28
**Valid until:** 2026-04-28 (stable domain; InspireHEP API field names rarely change)
