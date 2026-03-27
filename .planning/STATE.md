---
gsd_state_version: 1.0
milestone: v1.3
milestone_name: Data Pipeline Fixes
status: planning
stopped_at: Phase 18 context gathered
last_updated: "2026-03-27T23:25:14.237Z"
last_activity: 2026-03-28 — v1.3 roadmap created, phases 18-20 defined
progress:
  total_phases: 5
  completed_phases: 0
  total_plans: 0
  completed_plans: 0
  percent: 0
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-27)

**Core value:** Surface research gaps and unexplored connections that no single paper reveals — by structurally analyzing and comparing papers across a citation graph
**Current focus:** Phase 18 — arXiv Crawl Repair

## Current Position

Phase: 18 of 20 (arXiv Crawl Repair)
Plan: — (not yet planned)
Status: Ready to plan
Last activity: 2026-03-28 — v1.3 roadmap created, phases 18-20 defined

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

## Accumulated Context

### Decisions

(Full decision log in PROJECT.md Key Decisions table)

Recent decisions relevant to v1.3:

- [Memory]: arXiv crawls silently fail to store citation edges — use InspireHEP for reliable edge data (see project_arxiv_edge_bug.md)
- [Memory]: Most papers have empty published fields — temporal filtering needs data backfill (see project_data_enrichment_needed.md)

### Pending Todos

None.

### Blockers/Concerns

- Phase 18: arXiv HTML parser bug is the primary known issue — `<span class="ltx_bibblock">` parsing drops references without `<a>` tags
- Phase 19: Orphan node root cause unknown — investigation (ORPH-01) must precede fix (ORPH-02)
- Phase 20: LLM analysis pipeline was built in v1.0/v1.1 but not verified against the current Leptos web UI

## Session Continuity

Last session: 2026-03-27T23:25:14.235Z
Stopped at: Phase 18 context gathered
Resume file: .planning/phases/18-arxiv-crawl-repair/18-CONTEXT.md
