# Phase 18: arXiv Crawl Repair - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-28
**Phase:** 18-arxiv-crawl-repair
**Areas discussed:** ID extraction scope, Edge density validation, Backward compatibility

---

## ID Extraction Scope

### Q1: What identifier patterns should the parser extract from reference plain text?

| Option | Description | Selected |
|--------|-------------|----------|
| arXiv IDs only | Extract arXiv:YYMM.NNNNN and old-format (hep-ph/NNNNNNN) patterns from text. Matches the existing crawl boundary. | |
| arXiv IDs + DOIs | Also extract DOI patterns (10.NNNN/...) from text and store as Reference.doi. Won't create edges but enriches metadata. | ✓ |
| arXiv new-format only | Only YYMM.NNNNN patterns. Simpler regex, but misses old-format papers. | |

**User's choice:** arXiv IDs + DOIs
**Notes:** DOIs stored for future use but won't create graph edges (non-arXiv references not followed).

### Q2: When both a hyperlink and a plain-text arXiv ID are found in the same reference, which takes priority?

| Option | Description | Selected |
|--------|-------------|----------|
| Merge both | Add both as Links on the Reference. Dedup by ID so the same arXiv ID isn't added twice. | ✓ |
| Hyperlink wins | If a hyperlink already provided an arXiv link, skip the text-extracted one. | |
| You decide | Claude picks the approach that best fits the existing code patterns. | |

**User's choice:** Merge both
**Notes:** Maximizes edge coverage while preventing duplicate links.

---

## Edge Density Validation

### Q3: How should 'comparable edge density' be verified after the fix?

| Option | Description | Selected |
|--------|-------------|----------|
| Automated integration test | A test crawls the same seed paper via both sources (using wiremock fixtures) and asserts comparable edge counts. | ✓ |
| CLI diagnostic command | Add a --compare-sources flag for manual side-by-side comparison. | |
| Both | Automated test + CLI diagnostic. | |

**User's choice:** Automated integration test
**Notes:** Uses wiremock fixtures in CI for regression protection.

### Q4: Should we use a real arXiv HTML page snapshot as the test fixture?

| Option | Description | Selected |
|--------|-------------|----------|
| Real snapshot | Capture a real arXiv HTML bibliography page as the wiremock fixture. | ✓ |
| Synthetic HTML | Hand-craft HTML with known arXiv ID patterns. | |
| You decide | Claude picks based on existing test patterns. | |

**User's choice:** Real snapshot
**Notes:** Tests against actual HTML structure rather than synthetic examples.

---

## Backward Compatibility

### Q5: Should existing DB data from prior arXiv crawls be patched, or only new crawls benefit?

| Option | Description | Selected |
|--------|-------------|----------|
| New crawls only | Fix the parser for future crawls. Users can re-crawl to get improved results. | |
| Re-crawl command | Add a --recrawl flag to re-fetch HTML for existing DB papers. | |
| Wipe and re-crawl | Document that users should delete local DB and re-crawl from scratch. | ✓ |

**User's choice:** Wipe and re-crawl
**Notes:** Simplest approach — no migration needed. Acceptable since this is a single-user tool with no shared state.

---

## Claude's Discretion

- Regex pattern design for arXiv ID and DOI extraction
- Where in the parsing pipeline to inject text-based extraction
- Threshold for "comparable" edge density in integration test assertion

## Deferred Ideas

None — discussion stayed within phase scope.
