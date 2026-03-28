---
gsd_state_version: 1.0
milestone: v1.3
milestone_name: Data Pipeline Fixes
status: executing
stopped_at: Completed 20-01-PLAN.md
last_updated: "2026-03-28T21:59:44.333Z"
last_activity: 2026-03-28
progress:
  total_phases: 5
  completed_phases: 2
  total_plans: 7
  completed_plans: 4
  percent: 0
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-27)

**Core value:** Surface research gaps and unexplored connections that no single paper reveals — by structurally analyzing and comparing papers across a citation graph
**Current focus:** Phase 20 — llm-analysis-pipeline-verification

## Current Position

Phase: 20 (llm-analysis-pipeline-verification) — EXECUTING
Plan: 2 of 4
Status: Ready to execute
Last activity: 2026-03-28

Progress: [░░░░░░░░░░░░░░░░░░░░] 0% (v1.3: 0/3 phases)

## Performance Metrics

**Velocity (v1.2):**

| Phase | Duration | Tasks | Files |
|-------|----------|-------|-------|
| Phase 15-force-simulation-rebalancing P01 | — | 1 tasks | 3 files |
| Phase 15-force-simulation-rebalancing P02 | 6min | 2 tasks | 2 files |
| Phase 16-edge-and-node-renderer-fixes P01 | 8min | 2 tasks | 4 files |
| Phase 16-edge-and-node-renderer-fixes P02 | 12min | 2 tasks | 18 files |
| Phase 17-viewport-fit-and-label-collision P01 | 8min | 2 tasks | 5 files |
| Phase 17-viewport-fit-and-label-collision P02 | 4min | 2 tasks | 5 files |
| Phase 18-arxiv-crawl-repair P01 | 5min | 2 tasks | 4 files |
| Phase 18-arxiv-crawl-repair P02 | 13min | 1 tasks | 2 files |
| Phase 19-data-quality-cleanup P01 | 5min | 2 tasks | 2 files |
| Phase 20-llm-analysis-pipeline-verification P01 | 6min | 3 tasks | 9 files |

## Accumulated Context

### Decisions

(Full decision log in PROJECT.md Key Decisions table)

Recent decisions relevant to v1.3:

- [Memory]: arXiv crawls silently fail to store citation edges — use InspireHEP for reliable edge data (see project_arxiv_edge_bug.md)
- [Memory]: Most papers have empty published fields — temporal filtering needs data backfill (see project_data_enrichment_needed.md)
- [Phase 18-arxiv-crawl-repair]: OnceLock<Regex> statics for compiled patterns: initialized once at first call, zero overhead thereafter
- [Phase 18-arxiv-crawl-repair]: get_arxiv_id() Link-based lookup takes priority over arxiv_eprint fallback: preserves existing behavior for papers with hyperlinks
- [Phase 18-arxiv-crawl-repair]: Augment real HTML fixtures with synthetic entries to isolate plain-text-only extraction paths for testing
- [Phase 18-arxiv-crawl-repair]: Old-format arXiv IDs (hep-ph/...) have get_arxiv_id() return last URL segment; verify full ID via arxiv_eprint field in tests
- [Phase 19-data-quality-cleanup]: Filter at source in get_arxiv_references_ids() rather than at BFS queue ingestion
- [Phase 19-data-quality-cleanup]: earliest_date added to both fetch_paper() and fetch_literature() URL field params for consistency
- [Phase 20-llm-analysis-pipeline-verification]: StartAnalysis inlines resyn-core pipeline logic directly — avoids circular dependency (resyn-app cannot depend on resyn-server)
- [Phase 20-llm-analysis-pipeline-verification]: run_extraction/run_llm_analysis return anyhow::Result<()>; CLI run() retains process::exit for user-facing errors

### Pending Todos

None.

### Blockers/Concerns

- Phase 18: arXiv HTML parser bug is the primary known issue — `<span class="ltx_bibblock">` parsing drops references without `<a>` tags
- Phase 19: Orphan node root cause unknown — investigation (ORPH-01) must precede fix (ORPH-02)
- Phase 20: LLM analysis pipeline was built in v1.0/v1.1 but not verified against the current Leptos web UI

## Session Continuity

Last session: 2026-03-28T21:59:44.332Z
Stopped at: Completed 20-01-PLAN.md
Resume file: None
