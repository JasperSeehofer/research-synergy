# Phase 19: Data Quality Cleanup - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-28
**Phase:** 19-data-quality-cleanup
**Areas discussed:** Orphan diagnosis approach, Orphan elimination strategy, Published date backfill, Empty-ID paper handling

---

## Orphan Diagnosis Approach

| Option | Description | Selected |
|--------|-------------|----------|
| One-time code investigation | Analyze code paths + run test crawl to categorize orphan causes. Document findings in plan, fix directly. No new diagnostic tooling. | ✓ |
| Add diagnostic logging | Instrument BFS crawler and graph builder with structured tracing for orphan causes. | |
| Build orphan report command | CLI subcommand or server endpoint that scans DB and reports disconnected nodes with root cause analysis. | |

**User's choice:** One-time code investigation
**Notes:** None

### Follow-up: Investigation method

| Option | Description | Selected |
|--------|-------------|----------|
| Code analysis only | Researcher traces all code paths that produce orphans through static analysis. Codebase small enough to read exhaustively. | ✓ |
| Code analysis + test crawl | Also run actual InspireHEP crawl during research to count orphans empirically. | |

**User's choice:** Code analysis only
**Notes:** None

---

## Orphan Elimination Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Filter at source | Prevent orphan-causing data from entering the pipeline: skip empty-ID references during BFS, ensure version stripping is consistent. | ✓ |
| Filter at graph build | Let all papers enter pipeline but filter out zero-edge nodes when building graph. | |
| Fix ID matching | Keep all papers, improve ID resolution with inspire_id as secondary key. | |

**User's choice:** Filter at source
**Notes:** None

### Follow-up: Zero-orphan criteria

| Option | Description | Selected |
|--------|-------------|----------|
| At least one edge, any direction | Every node has at least one inbound OR outbound edge. Seed has outbound; leaf nodes have inbound from parent. | ✓ |
| Exclude boundary nodes | Only enforce for nodes within max_depth-1. Boundary nodes expected to have fewer connections. | |
| Strict: both directions | Every node must have both inbound AND outbound edges. | |

**User's choice:** At least one edge, any direction
**Notes:** None

---

## Published Date Backfill

| Option | Description | Selected |
|--------|-------------|----------|
| Extract from InspireHEP response | Parse publication info during convert_hit_to_paper. Zero extra requests. | |
| Cross-source enrichment | After crawl, fetch from arXiv API for papers with empty published dates. | |
| Both sources | Parse from InspireHEP AND add arXiv API fallback for reference-only papers. | ✓ |

**User's choice:** Both sources
**Notes:** None

### Follow-up: arXiv fallback timing

| Option | Description | Selected |
|--------|-------------|----------|
| During BFS crawl | When fetch_paper() returns, published date comes with it. Only reference-only papers need fallback. | ✓ |
| Post-crawl batch pass | After BFS completes, scan all papers for empty dates and batch-fetch. | |
| Lazy on DB read | Fetch on demand when loading papers for display. | |

**User's choice:** During BFS crawl
**Notes:** None

---

## Empty-ID Paper Handling

| Option | Description | Selected |
|--------|-------------|----------|
| Skip in reference conversion | In convert_references(), skip references without arxiv_eprint entirely. | |
| Skip in BFS queue | Keep all references on paper, filter empty strings when collecting IDs for BFS queue. | ✓ |
| Use inspire_record_id as key | Use InspireHEP record ID for references without arxiv_eprint. Requires reworking ID system. | |

**User's choice:** Skip in BFS queue (via free-text discussion)
**Notes:** User noted the current system is fully arXiv-ID-keyed and wants to rework this in a future milestone to support non-arXiv sources. Agreed to defer universal ID system and use pragmatic BFS-queue filtering for Phase 19.

---

## Claude's Discretion

- Specific tracing/debug log messages
- InspireHEP date field extraction approach
- Helper method vs inline for empty-ID filtering
- Test strategy (unit vs integration)

## Deferred Ideas

- Universal paper ID system — rework arXiv-keyed IDs to support DOI, InspireHEP record ID, etc. Future milestone.
- Non-arXiv paper graph inclusion — unlock stored references with inspire_record_id/doi as graph nodes.
