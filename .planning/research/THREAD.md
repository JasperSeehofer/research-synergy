# Thread state — Dynamical LBD (Gen-4)

*Layer-2 thread-state contract (vault: `wiki/analyses/research-routine-packs-spec.md`). Read by
the vault's `/cartographer` bridge in place of the retired `.gpd/` state. Keep current: same-day
update after every experiment run (vault: `wiki/meta/research-operating-manual.md`, "per
numerical experiment / run"). Vault mirror of the hypothesis rows: `wiki/meta/hypothesis-ledger.md`.*

## Hard core

The knowledge graph is a *dynamical system*; bridges *emerge* from generation, not static
scoring (Gen-4 LBD — vault: `wiki/concepts/dynamical-lbd.md`, the three acceptance criteria).

## Live hypotheses (mirror of vault hypothesis-ledger)

| id | statement | discriminating experiment | status |
|---|---|---|---|
| H-RS-substrate | Cellular **sheaves** over the Louvain community graph detect multi-causal bridges better than RAFs or Kuramoto | 4-tier benchmark incl. multi-causal joint-removal ablation on the shared 10-pair Feynman set | **UNBLOCKED (2026-07-04 reanalysis)** — a well-posed substrate exists after all: the FULL `data-kuramoto` citation corpus giant CC (224 nodes, 1 component, 4 evaluable pairs). Phases 29/30 failed on a methodology artifact (unnecessary pre-2015 slice, C-1), not on the substrate. Gated behind EXP-RS-12 producing a real BENCH_P10 on the corrected corpus. |
| H-RS-method | The dynamical-LBD pipeline (Kuramoto→Fiedler) has real cross-domain-bridge recovery signal when run on a well-posed citation graph containing both literatures | EXP-RS-12: BENCH_P10 on the 224-node giant CC vs 0.15 baseline / 0.30 target, vs ER + config-model nulls | **NOT YET TESTABLE** — EXP-RS-12 (Phase 31) got a well-posed graph (K_stable=14.25 converged) but BENCH_P10=0.000; diagnostic: 3/4 benchmark pairs have ZERO inter-community edges (bridge literature absent from corpus), so the method was never fairly tested. Blocked on a bridge-*containing* corpus (proposed Phase 2). |

## Kill criteria

- **Method-level:** sheaf near-section frustration does not beat the brute-force baseline
  (vault: `wiki/concepts/brute-force-lbd-baseline.md`) on held-out bridges.
- **Pivot gate (time-bound, set 2026-07-02, human-approved):** if the Path C TF-IDF
  semantic-edge substrate (EXP-RS-11) yields **<3 evaluable Feynman pairs** or
  **`BENCH_P10 ≤ 0.15`** on the shared 10-pair set by **2026-09-30**, kill the
  dynamical-substrate line and revert to the brute-force baseline.

## Current claims

| claim | status | evidence |
|---|---|---|
| Dynamical-LBD on the pre-2015 cond-mat *citation* graph is empirically infeasible (~41 components / 153 nodes → `K_stable` bisection diverges) | verified (Phase 29 FAIL, 2026-05-05) | `.planning/phases/29-kuramoto-corpus-build/29-VERIFICATION.md` |
| Sheaf near-section frustration ranks bridges on this corpus | HOLD — untestable on VOID corpus (T2 precision@10 = 0.000) | `prototypes/SHEAF_V01_RESULTS.md` |
| TF-IDF cosine edges (τ=0.3) make the same corpus connected enough for spectral/dynamical LBD (`n_cc/N ≤ 0.05`, largest CC ≥ 80%) | **FALSIFIED** (Phase 30 FAIL, 2026-07-04) — actual `n_cc/N`=0.830, largest CC=3.3% at τ=0.3; *more* fragmented than the citation graph (0.268) at every pre-registered τ. Confirmed by 3 independent recomputes. | `.planning/phases/30-tfidf-semantic-edge-graph/30-VERIFICATION.md` |
| ~~The pre-2015 cond-mat corpus (N=153) is too narrow to support *any* substrate for dynamical LBD~~ | **RETRACTED (2026-07-04)** — this conflated the *pre-2015 slice* with the *corpus*. The FULL corpus citation graph is well-posed (227→giant CC 224, 1 component, n_cc/N=0.009). The fragmentation was caused ENTIRELY by the C-1 pre-2015 slice, which is not required by the date-agnostic BENCH_P10 recovery metric. Not "corpus too narrow" — "temporal slice unnecessary and harmful." | full-corpus connectivity check 2026-07-04; see EXP-RS-12 provenance |
| Phases 29/30 non-results were corpus/methodology *connectivity* artifacts (the pre-2015 slice) — CONFIRMED: the full-corpus giant CC is well-posed, K_stable=14.25 converges | **verified** (Phase 31 EXP-RS-12, 2026-07-04) | `.planning/phases/31-dynamical-lbd-giant-cc/31-VERIFICATION.md` |
| The dynamical method recovers Feynman bridges (BENCH_P10 > 0.15) on the well-posed giant CC | **FALSIFIED but test not fair** (Phase 31: BENCH_P10=0.000) — decisive diagnostic: 3/4 evaluable pairs have ZERO inter-community citation edges → the corpus lacks the bridge literature the method is scored on; the 1 pair with a 2-edge bridge (pair04) is diluted out of the global-top-10. Corpus-CONTENT gap now isolated from the (solved) connectivity gap. | 31-VERIFICATION.md § "decisive diagnostic" |

## Active experiment

**EXP-RS-14** (→ Phase 33) — the definitive test: first VALID run on a corpus that is
simultaneously connected + bridge-containing + synchronized + finely-partitioned. Motivation: four
phases, four confounds — connectivity (29/30), corpus content (31), and now **dynamical
non-convergence at scale** (32: on 1400 nodes the Kuramoto system found a low-K scattered fixed
point, r=0.136; the λ₂≥0 K-criterion admits unsynchronized states; 7-community Louvain collapsed
pair03). Kuramoto–Fiedler has a NARROW operating window; we have satisfied each condition alone but
never all together. EXP-RS-14 removes the convergence + granularity confounds (principled,
pre-registered, reported either way — NOT benchmark tuning): finer Louvain (res=3.0
→ 34 communities, all 4 pairs in DISTINCT communities, 4/4 pairs now have inter-community bridge
edges: pair01:9, pair03:23, pair04:27, pair06:4) + sync-aware K (`find_K_sync`: min K with r ≥ 0.90;
the 1400-node graph verified to sync — r=0.71@K=5, 0.96@K=15). Kept the FULL 1400-node corpus (no
reduction → no selection concern). Runner `kuramoto_lbd_v07.py`.

**LOCKED PREDICTION (EXP-RS-14, before run):** on this fully-valid corpus (connected ∧ bridged ∧
synchronized ∧ finely-partitioned) genuine cross-domain per-pair recall@10 ≥ 0.25 (≥1 of 4 pairs
detected). **Decisive:** still 0 here ⇒ the cleanest Kuramoto–Fiedler method-negative — the method
fails to surface present bridges even when everything is well-posed. Positive ⇒ real signal →
independent falsification, then formalize through the resyn pipeline.

### (superseded) EXP-RS-13 — Phase 32, INCONCLUSIVE (confounded) 2026-07-04
pre-registered 2026-07-04, prediction LOCKED before result. Human
approved the Phase 2 corpus rebuild (2026-07-04). Built a benchmark-centric **bridge-containing**
corpus via a targeted OpenAlex fetch (endpoint citation neighborhoods; neutral rule, NOT tuned to
the benchmark): `data/research_synergy_bridged.json` — **1400 nodes, 9624 edges, 9 communities;
3/4 evaluable pairs now have inter-community bridge edges** (pair01:91, pair04:649, pair06:66) vs
1/4 in data-kuramoto. This addresses the EXP-RS-12 corpus-content gap.

**LOCKED PREDICTION (before the score — v06 still computing when this was written):**
per-pair recall@10 ≥ 0.25 (the method surfaces at least the bridges now present); global BENCH_P10
uncertain (top-10 dilution on a larger corpus). **Decisive read:** if even a bridge-CONTAINING
corpus yields ~0 detections → clean statement that Kuramoto–Fiedler fails to surface known,
present bridges (real method-negative). If it detects them → line alive; formalize through the
resyn pipeline (bulk-ingest→analyze→export) for the official number.

Runner: `prototypes/kuramoto_lbd_v06.py` (bridged corpus + per-pair metric; committed before run,
vault `3115c57`). Result lands in `prototypes/data/kuramoto_v06_results.json`.

*EXP-RS-11 (TF-IDF, Phase 30) remains dead. EXP-RS-12 (Phase 31) validated the methodology fix but
found the corpus lacked bridges. EXP-RS-13 tests the same method on a corpus that now contains
them — the fair test EXP-RS-12 could not run.*

## Claim history

`CLAIMS.jsonl` (commission-compatible) is created by the first `/commission --research` run on
this thread; until then the table above is the claim record. (Exact format is an open Layer-2
spec question — session feedback welcome.)

## Last verification

2026-07-04 — Phase 32 verification (EXP-RS-13 INCONCLUSIVE/confounded). Corpus fix worked (3/4
pairs bridged; real bridges rank #11/#17 of ~9600 edges) but the 1400-node Kuramoto run did NOT
converge (r=0.136, K_stable collapsed to floor, λ₂<0) → invalid; sole "detection" was a pair03
same-community artifact. Genuine cross-domain recall@10 = 0. Third confound (non-convergence at
scale) identified. Fair test still not run → EXP-RS-14. Prior: 2026-07-04 Phase 31 EXP-RS-12 MIXED.

Earlier Phase 31 (EXP-RS-12 MIXED): Methodology fix VALIDATED (giant CC
well-posed, K_stable=14.25 converges — 29/30 were connectivity artifacts); locked stake P-3
FALSIFIED (BENCH_P10=0.000) but the test was not fair — static diagnostic shows 3/4 evaluable
pairs have zero inter-community edges (bridge literature absent from corpus). Corpus-content gap
isolated from the solved connectivity gap. Kill-criterion check: **not a clean method-kill** (the
method was never given a bridge-containing corpus); decision on Phase 2 corpus rebuild is the
human's. Prior: 2026-07-04 Phase 30 EXP-RS-11 FAIL; 2026-05-05 Phase 29 FAIL.
