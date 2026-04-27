---
phase: 28-forward-citation-crawl-mode-s2
verified: 2026-04-27T12:00:00Z
status: passed
score: 9/9 must-haves verified
overrides_applied: 0
---

# Phase 28: Forward-Citation Crawl Mode (S2) Verification Report

**Phase Goal:** Add forward-citation (bidirectional) crawl mode using the Semantic Scholar /citations endpoint — so ReSyn can discover papers that CITE a seed, not just papers the seed cites.
**Verified:** 2026-04-27
**Status:** PASSED
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | SemanticScholarSource has `bidirectional` and `max_forward_citations` builder fields | ✓ VERIFIED | `semantic_scholar_api.rs` lines 21-22, 34-35, 55-56; builder methods at lines 84-93 |
| 2 | `fetch_citing_papers_inner` paginates /citations, enforces cap, silently swallows 404 | ✓ VERIFIED | `semantic_scholar_api.rs` lines 257-316; 5 wiremock tests pass (all-features) |
| 3 | `PaperSource` trait has `fetch_citing_papers` default no-op method | ✓ VERIFIED | `traits.rs` lines 18-20: `async fn fetch_citing_papers(&mut self, _paper: &mut Paper) -> Result<(), ResynError> { Ok(()) }` |
| 4 | `Paper` has `citing_papers: Vec<Reference>` transient field with `#[serde(default, skip_serializing)]` | ✓ VERIFIED | `paper.rs` lines 33-35; 4 unit tests pass including serialization-skip test |
| 5 | `Paper::get_citing_arxiv_ids` exists and mirrors `get_arxiv_references_ids` | ✓ VERIFIED | `paper.rs` lines 85-101; mirrors filter_map pattern exactly |
| 6 | `PaperRepository::upsert_inverse_citations_batch` writes `RELATE citing->cites->cited` direction | ✓ VERIFIED | `queries.rs` lines 153-195; 5 DB integration tests pass including direction-inversion test |
| 7 | `CrawlArgs` has `--bidirectional` and `--max-forward-citations` flags | ✓ VERIFIED | `crawl.rs` lines 97-106; clap fields present with correct defaults (false, 500) |
| 8 | Worker loop calls `fetch_citing_papers` + `upsert_inverse_citations_batch` + enqueue when `bidirectional=true` | ✓ VERIFIED | `crawl.rs` lines 403-455; full block confirmed including non-S2 warn path |
| 9 | `SemanticScholarSource` overrides `fetch_citing_papers` to delegate to `fetch_citing_papers_inner` | ✓ VERIFIED | `semantic_scholar_api.rs` lines 411-413 inside `impl PaperSource` block |

**Score:** 9/9 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `resyn-core/src/data_aggregation/semantic_scholar_api.rs` | S2 fields, builders, deserializers, fetch_citing_papers_inner | ✓ VERIFIED | All components present and substantive; wired into PaperSource impl |
| `resyn-core/src/data_aggregation/traits.rs` | PaperSource trait with fetch_citing_papers default no-op | ✓ VERIFIED | 22-line file, method present with Ok(()) default |
| `resyn-core/src/datamodels/paper.rs` | Paper.citing_papers transient field + get_citing_arxiv_ids | ✓ VERIFIED | Field at line 35, method at line 85, serde annotations correct |
| `resyn-core/src/database/queries.rs` | upsert_inverse_citations_batch method | ✓ VERIFIED | Lines 153-195, correct RELATE direction, parameterized binding |
| `resyn-server/src/commands/crawl.rs` | CrawlArgs flags + worker loop integration | ✓ VERIFIED | Flags at lines 97-106; full bidirectional block at lines 401-455 |
| `scripts/crawl-feynman-seeds.sh` | --bidirectional added to per-seed invocation | ✓ VERIFIED | Line 29 contains `--bidirectional` flag |
| `CLAUDE.md` | New flags documented; ChainedPaperSource bug note removed | ✓ VERIFIED | Lines 129-130 (table rows), line 175 (Important Notes bullet); no ChainedPaperSource bug note found |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `CrawlArgs.bidirectional` | `SemanticScholarSource.with_bidirectional` | `make_single_source` builder chain | ✓ WIRED | `crawl.rs` line 130: `.with_bidirectional(bidirectional)` |
| `SemanticScholarSource::fetch_citing_papers` (trait override) | `fetch_citing_papers_inner` | delegate call | ✓ WIRED | `semantic_scholar_api.rs` line 412: `self.fetch_citing_papers_inner(paper).await` |
| `worker loop` | `PaperRepository::upsert_inverse_citations_batch` | after fetch_citing_papers | ✓ WIRED | `crawl.rs` lines 415-427 |
| `worker loop enqueue` | `queue.enqueue_if_absent` | for each `get_citing_arxiv_ids()` | ✓ WIRED | `crawl.rs` lines 429-444 |
| `upsert_inverse_citations_batch` | `RELATE $from->cites->$to` | SurrealQL parameter binding | ✓ WIRED | `queries.rs` line 181; no string interpolation — `.bind()` used |
| `fetch_citing_papers_inner` | `S2CitationsPage` deserializer | pagination loop | ✓ WIRED | `semantic_scholar_api.rs` line 296: `serde_json::from_str::<S2CitationsPage>` |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|--------------|--------|-------------------|--------|
| `crawl.rs` bidirectional block | `paper.citing_papers` | `source.fetch_citing_papers()` → `fetch_citing_papers_inner` → S2 `/citations` HTTP | Yes (real API pagination with cap) | ✓ FLOWING |
| `upsert_inverse_citations_batch` | edges in `cites` table | `Reference.get_arxiv_id()` → `paper_record_id` → `RELATE` | Yes (DB writes confirmed by 5 passing integration tests) | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| 5 S2 integration tests (happy path, 404, pagination, cap, disabled) | `cargo test -p resyn-core --all-features --test semantic_scholar_integration -- fetch_citing_papers` | 5 passed | ✓ PASS |
| 4 paper.rs unit tests (default, serde skip, get_citing_arxiv_ids, deserialize) | `cargo test -p resyn-core --lib citing_papers_tests` | 4 passed | ✓ PASS |
| 5 DB integration tests (direction, skip no-arxiv, empty slice, dangling, version strip) | `cargo test -p resyn-core --all-features inverse_citations_tests` | 5 passed | ✓ PASS |
| Workspace compile check | `cargo check --workspace` | No errors | ✓ PASS |

### Requirements Coverage

No v1.4 requirement IDs map to this phase (infrastructure improvement). Not applicable.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| None found | — | — | — | — |

No TODOs, FIXMEs, placeholder returns, or empty implementations found in phase-modified files. The `citing_papers` field defaults to `Vec::new()` but this is correct design — it is a transient field populated at runtime by `fetch_citing_papers`, not stored.

### Human Verification Required

None. All must-haves are programmatically verifiable and confirmed.

### Gaps Summary

No gaps. All 9 must-haves verified across all 4 plans (wave 1, 2, and 3). The integration test suite (14 new tests: 5 S2 wiremock + 4 paper unit + 5 DB) provides strong correctness coverage including edge-direction inversion proof.

---

_Verified: 2026-04-27T12:00:00Z_
_Verifier: Claude (gsd-verifier)_
