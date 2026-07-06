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
**Current focus:** Dynamical-LBD thread has reached its method-level KILL criterion (Phase 34, 2026-07-05). Both graph-dynamical/spectral candidates fail the shared 10-pair Feynman benchmark at recall@10 = 0 on a fully valid bridge-containing corpus: Kuramoto–Fiedler (Phase 33, single global cut) and sheaf frustration (Phase 34). Recommendation (human's go/kill): retire the dynamical-substrate line, revert to the brute-force baseline (EXP-RS-10). The 6-phase arc leaves a valid testbed + corpus-construction method + two clean method-negatives.

## Current Position

Phase: 31 (complete — MIXED verdict)
Plan: 31-01 (complete)
Status: EXP-RS-12 got a well-posed graph (K_stable=14.25 converged — reanalysis validated, 29/30 were connectivity artifacts) but BENCH_P10=0.000 (locked stake P-3 falsified). Diagnostic: 3/4 benchmark pairs have zero inter-community edges → corpus lacks the bridge literature; method not fairly tested. Connectivity gap SOLVED; corpus-CONTENT gap now the blocker. Awaiting human go/kill on proposed Phase 2 (bridge-containing corpus via OpenAlex bulk-ingest, real compute cost).
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

Last session: 2026-07-04 (autonomous overnight)
Research thread state: `.planning/research/THREAD.md` (Layer-2 contract; same-day updates required)

### RESUME POINTER — EXP-RS-16 (Phase 35) pre-registered; run it in a FRESH session

**New chapter opened.** Dynamical-LBD KILLED (human, 2026-07-05). Brainstorm ran (32 agents →
`.planning/research/BRAINSTORM-cross-field-transfer.md`). Human chose the recommended first move →
**EXP-RS-16: SME generator vs brute-force baseline, conditional-retrieval eval** (pre-registered in
THREAD.md § Active experiment — design + LOCKED predictions P1–P4 + gate).

**To run EXP-RS-16 (fresh session, has full context budget):**
1. **Assemble corpus** `professional-vault/prototypes/data/mvp_corpus.json` (fetch FAILED this
   session — arXiv-API batch retry needed). Benchmark endpoints resolve via OpenAlex title.search
   (9/10 had abstracts); distractors via arXiv API `id_list` (old-format IDs like `cond-mat/…` work
   there, but the OpenAlex `landing_page_url.search` filter 400s on them). Target: 10 benchmark +
   ~26 distractor papers with title+abstract. Testbed for sampling/communities:
   `data/research_synergy_bridged_fine_sheaf.json`; benchmark: `data/feynman_10pair_papers.json`
   (evaluable pairs 01/03/04/05/06); ground-truth bridge names: `data/cross_bridges_ground_truth.json`.
2. **Schema extraction** (Claude, ~35 papers × roles-ON + a roles-OFF variant) — blind, abstract-only,
   role-typed relational schema (spec in THREAD). Best as a small Workflow (parallel, structured output).
3. **SME-lite matcher** (Python, `prototypes/sme_lite.py`) — systematicity score under best
   role-consistent VF2/greedy mapping; emit alignment tables.
4. **Baseline** (Claude conditional-retrieval per benchmark side_a) — job zero, the bar.
5. **Score** recall@k + MRR, SME vs baseline, roles-ON vs OFF, alignment vs ground-truth → verdict vs
   the LOCKED gate. Same discipline as prior phases (commit scripts before running on data).

Do NOT re-tune to force a pass. Predictions are locked in THREAD before any method runs.

### (history) Dynamical-LBD KILL criterion met (Phases 31→34 complete)

**Phase 34 (EXP-RS-15) DONE:** sheaf-vs-Kuramoto head-to-head on the valid testbed → **both fail,
recall@10 = 0.** Sheaf frustration recovers 0/4 into the top-10 (pairs rank #69–218), T4 ablation
0/5; tied with Kuramoto (Phase 33). H-RS-substrate falsified at the benchmark bar. **Method-level
kill criterion met for both graph-dynamical/spectral candidates.** See
`.planning/phases/34-sheaf-vs-kuramoto/34-VERIFICATION.md`.
**NEXT (human's go/kill — NOT auto-executed):** retire the dynamical-substrate line, revert to the
brute-force baseline (EXP-RS-10, BF-community-pairs LLM). Optional last dynamical straw: RAF on its
reaction-model encoding (EXP-RS-08, different data model, untested) — low expected value. Do NOT
re-tune corpora/metrics to force a pass (spec-gaming).

### (history) OVERNIGHT OUTCOME (Phases 31→33) — Kuramoto clean negative

**Phase 33 (EXP-RS-14) DONE — CLEAN Kuramoto–Fiedler method-negative.** On a fully well-posed corpus
(`research_synergy_bridged_fine.json`: connected + 4/4 pairs bridged + synchronized r=0.932 + 32
communities) the method recovers 0/4 benchmark bridges; NO pair in the top-200 Fiedler bridges.
Mechanism verified: single global Fiedler cut puts all pairs on the same side → structurally
invisible. Not a confound. See `.planning/phases/33-valid-converged-run/33-VERIFICATION.md`.

**The five-phase substrate arc (29→33) delivered:** (a) a corpus-construction method
(`build_bridge_corpus_openalex.py`); (b) a VALID benchmark testbed (`research_synergy_bridged_fine.json`);
(c) the clean Kuramoto refutation; (d) a sharp prediction — LOCAL/multi-scale detectors (sheaves)
should beat the single global cut.

**NEXT (human's go/kill/pivot decision — NOT auto-executed):** run the sheaf/RAF/Kuramoto tournament
on the valid testbed via `/cartographer --tournament` (vault, out of scope for this repo session).
If sheaves also score 0/4 on this fair test → dynamical-LBD hard core refuted → revert to brute-force
baseline. Do NOT re-tune the corpus or start the tournament autonomously — both are governance-gated.

**Decision tree when v06 result is available:**
1. Read `kuramoto_v06_results.json` (BENCH_P10, perpair_recall_at10, perpair_ranks, K_stable, nulls).
2. Write `.planning/phases/32-*/32-VERIFICATION.md` (executed evidence + verdict vs the LOCKED
   prediction — no post-hoc adjustment). Update THREAD.md (same-day), STATE, ROADMAP, vault EXP-RS-13.
3. **If per-pair recall@10 ≥ 0.25 OR global BENCH_P10 > 0.15** → SIGNAL. Next: (a) independent
   falsification (right-sized, blind re-score) that the detections are real not artifacts; (b) if
   confirmed, formalize the corpus through the resyn pipeline (bulk-ingest needs an OpenAlex key —
   currently only unauthenticated polite pool works; note this) → analyze → export → re-run for the
   official number; (c) flag tournament-readiness in THREAD.
4. **If ~0 (no detections despite 3/4 bridges present)** → clean method-negative: Kuramoto–Fiedler
   does not surface known present bridges. Write it up honestly; the kill-vs-continue call is the
   human's (record in THREAD, do not self-kill). Consider whether the global-top-10 metric or the
   Louvain community granularity (only 9 communities on 1400 nodes) is the confound.
5. Commit each step (conventional commits, both repos). `/scribe-debrief` at a clean stopping point.

**Do NOT** re-tune the corpus to make the benchmark pass (spec-gaming). The corpus was built by a
neutral endpoint-neighborhood rule; keep it fixed. Metric/community-granularity are separate,
declarable knobs — changing them is a NEW pre-registered experiment, not a tweak.
