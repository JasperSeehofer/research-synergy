---
gsd_state_version: 1.0
milestone: v1.4
milestone_name: Discovery & Intelligence
status: in_progress
stopped_at: "Phase 29 — crawl paused after 2 cap=500 aborts; cap lowered to 50; resume per 29-RESUME.md"
last_updated: "2026-05-04T18:30:00.000Z"
last_activity: 2026-05-04
progress:
  total_phases: 9
  completed_phases: 6
  total_plans: 19
  completed_plans: 18
  percent: 95
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-04-06)

**Core value:** Surface research gaps and unexplored connections that no single paper reveals — by structurally analyzing and comparing papers across a citation graph
**Current focus:** Phase 29 — Kuramoto-LBD v03 Corpus Build (exploratory dynamical-LBD benchmark)

## Current Position

Phase: 29
Plan: 29-01 (paused mid-execution)
Status: Resume per `.planning/phases/29-kuramoto-corpus-build/29-RESUME.md`
Last activity: 2026-05-04

Progress: [██████████████░░░░░░] 75%

## Accumulated Context

### Decisions

(Full decision log in PROJECT.md Key Decisions table)

Recent decisions affecting v1.4:

- SurrealDB FLEXIBLE TYPE for complex fields — works but limits server-side querying; revisit for analytics queries in Phase 23
- TF-IDF vectors already stored per paper — Phase 22 similarity engine builds on this without new extraction
- [Phase 24]: PendingCommunityDrawerOpen provided at App level (not GraphPage) — Leptos context flows downward only
- [Phase 24]: DrawerOpenRequest.paper_id relaxed to Option<String>; community_id Option<u32> added for legend-click mode D-16/D-17
- [Phase 24]: Community summaries computed on-read (lazy) — no sidecar cache table
- [Phase 24]: Stage 6 community auto-compute placed after Stage 5 metrics so PageRank is available for hybrid ranking

### Roadmap Evolution

- Phase 28 added: Forward-citation crawl mode (S2)
- Phase 29 added: Kuramoto-LBD v03 Corpus Build (exploratory benchmark, gates EXP-RS-07)

### Pending Todos

None.

### Blockers/Concerns

- Phase 24 depends on Phase 23 (needs graph_metrics table + PageRank)
- Phase 25 depends on Phases 22, 23, 24 (needs similarity neighbors, centrality scores, community assignments)
- Phases 21, 22, 23, 26 are independent and can be executed in any order

## Session Continuity

Last session: 2026-05-04T18:30:00.000Z
Stopped at: User-initiated end of session (Phase 29 crawl paused after queue-blowup discovery)
Resume file: `.planning/phases/29-kuramoto-corpus-build/29-RESUME.md`
