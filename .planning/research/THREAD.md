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

**None active — EXP-RS-12 (Phase 31) complete 2026-07-04, MIXED verdict; awaiting human
go/kill/pivot on a Phase 2 corpus rebuild.** EXP-RS-12 confirmed the reanalysis (the giant CC
is well-posed; K_stable=14.25 converges — Phases 29/30 were connectivity artifacts) but the
locked stake P-3 was **falsified** (BENCH_P10=0.000). The decisive static diagnostic showed **why**:
3/4 evaluable benchmark pairs have zero inter-community citation edges — the corpus does not
contain the cross-domain bridge literature the method is scored on, so the method was never fairly
tested. **The connectivity gap is solved; a corpus-CONTENT gap is now the blocker.**

**Proposed Phase 2 (gated on human go — real compute cost):** build a bridge-*containing* corpus
(OpenAlex bulk-ingest of cond-mat + stat-phys + q-fin + nlin, rate-limit-free; or deeper multi-hop
crawl) so the benchmark bridges are present, then re-test — likely also needing the per-pair
BENCH_P10 variant (the global-top-10 metric dilutes weak bridges). Alternative: accept the
unproven-method negative and revert to brute-force. Full record:
`.planning/phases/31-dynamical-lbd-giant-cc/31-VERIFICATION.md`.

*EXP-RS-11 (TF-IDF, Phase 30) remains dead. EXP-RS-12 tested the citation substrate — a distinct,
corrected experiment.*

## Claim history

`CLAIMS.jsonl` (commission-compatible) is created by the first `/commission --research` run on
this thread; until then the table above is the claim record. (Exact format is an open Layer-2
spec question — session feedback welcome.)

## Last verification

2026-07-04 — Phase 31 verification (EXP-RS-12 MIXED). Methodology fix VALIDATED (giant CC
well-posed, K_stable=14.25 converges — 29/30 were connectivity artifacts); locked stake P-3
FALSIFIED (BENCH_P10=0.000) but the test was not fair — static diagnostic shows 3/4 evaluable
pairs have zero inter-community edges (bridge literature absent from corpus). Corpus-content gap
isolated from the solved connectivity gap. Kill-criterion check: **not a clean method-kill** (the
method was never given a bridge-containing corpus); decision on Phase 2 corpus rebuild is the
human's. Prior: 2026-07-04 Phase 30 EXP-RS-11 FAIL; 2026-05-05 Phase 29 FAIL.
