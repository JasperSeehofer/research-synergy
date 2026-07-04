---
gsd_state_version: 1.0
milestone: v1.4
milestone_name: Discovery & Intelligence
status: in_progress
stopped_at: "Phase 30 complete — EXP-RS-11 FAIL (TF-IDF substrate fragments worse than citation graph); pivot kill gate FIRED; awaiting human go/kill/pivot"
last_updated: "2026-07-04T00:00:00.000Z"
last_activity: 2026-07-04
progress:
  total_phases: 10
  completed_phases: 8
  total_plans: 20
  completed_plans: 20
  percent: 80
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-04-06)

**Core value:** Surface research gaps and unexplored connections that no single paper reveals — by structurally analyzing and comparing papers across a citation graph
**Current focus:** Phase 31 — EXP-RS-12: run the validated dynamical-LBD pipeline on the well-posed FULL-corpus giant CC (224 nodes) for the first real BENCH_P10. 2026-07-04 reanalysis found Phases 29/30 failed on a methodology artifact (unnecessary pre-2015 slice), not the corpus. Pre-registered, predictions locked.

## Current Position

Phase: 31 (in progress — EXP-RS-12)
Plan: 31-01
Status: Reanalysis reversed the "corpus too narrow" conclusion — full-corpus giant CC (224 nodes) is well-posed and benchmark-evaluable (n_eval=4). Pre-registered EXP-RS-12 (predictions locked); running the citation-adjacency pipeline for the first real BENCH_P10. Phase 30 EXP-RS-11 (TF-IDF) remains dead; this is a distinct corrected experiment.
Last activity: 2026-07-04

Progress: [████████████████░░░░] 80% (v1.4 phases 25 Discovery Recommendations, 26 Export & Interop still unstarted)

## Accumulated Context

### Decisions

(Full decision log in PROJECT.md Key Decisions table)

Recent decisions affecting v1.4:

- SurrealDB FLEXIBLE TYPE for complex fields — works but limits server-side querying; revisit for analytics queries in Phase 23
- TF-IDF vectors already stored per paper — Phase 22 similarity engine builds on this without new extraction
- [Phase 24]: Community summaries computed on-read (lazy) — no sidecar cache table
- [Phase 29]: FAIL verdict 2026-05-05 — pre-2015 cond-mat citation graph too sparse for dynamical LBD (41 cc / 153 nodes); benchmark gate never reached. Honest negative; deviations (S2 429 tarpit → cap 20 / depth 1) recorded in 29-VERIFICATION.md
- [2026-07-02, human]: Path C pivot approved (`.cartographer-notes.md`) — rebuild substrate as TF-IDF cosine semantic-edge graph (EXP-RS-11, pre-registered). Time-bound kill gate: <3 evaluable Feynman pairs or BENCH_P10 ≤ 0.15 by 2026-09-30 → kill dynamical-substrate line, revert to brute-force baseline
- [Phase 30]: EXP-RS-11 FAIL verdict 2026-07-04 — TF-IDF cosine semantic edges make the pre-2015 corpus *more* fragmented (n_cc/N=0.830 @ τ=0.3) than the citation graph (0.268) at every pre-registered τ; precheck fails, `BENCH_P10` not producible. **Pivot kill gate FIRED** (well before the 2026-09-30 deadline). Verdict survived a right-sized `/commission --research` (3 converging lines; no under-connection bug, no leakage/contamination). Both substrate candidates now exhausted → the corpus itself is the limiter; Path B (seed selection) is the remaining option. Kill vs Path-B decision = human's, via the vault. See 30-VERIFICATION.md

### Roadmap Evolution

- Phase 28 added: Forward-citation crawl mode (S2)
- Phase 29 added: Kuramoto-LBD v03 Corpus Build (exploratory benchmark, gates EXP-RS-07) — completed with FAIL verdict
- Phase 30 added: TF-IDF Semantic-Edge Graph + Downstream LBD Method (EXP-RS-11, Path C pivot)

### Pending Todos

None.

### Blockers/Concerns

- Phase 25 depends on Phases 22, 23, 24 (needs similarity neighbors, centrality scores, community assignments)
- Phase 30: no new crawling permitted (S2 429 tarpit); predictions locked — no post-hoc adjustment; τ sweep is sensitivity analysis, not tuning

## Session Continuity

Last session: 2026-07-04
Stopped at: Phase 30 closed (EXP-RS-11 FAIL, kill gate fired). Verdict paperwork complete — 30-VERIFICATION.md, THREAD.md, ROADMAP.md, commission falsification record all landed. Next action is the **human's go/kill/pivot decision** (kill dynamical-substrate line vs Path B seed selection), routed via the vault cartographer — not a coding task in this repo.
Research thread state: `.planning/research/THREAD.md` (Layer-2 contract; same-day updates required)
