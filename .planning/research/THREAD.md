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
| H-RS-method | The dynamical-LBD pipeline (Kuramoto→Fiedler) has real cross-domain-bridge recovery signal when run on a well-posed citation graph containing both literatures | EXP-RS-12: BENCH_P10 on the 224-node giant CC vs 0.15 baseline / 0.30 target, vs ER + config-model nulls | **live** — pre-registered 2026-07-04, predictions locked (Phase 31) |

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
| Phases 29/30 non-results were corpus/methodology artifacts; the dynamical method recovers Feynman bridges (BENCH_P10 > 0.15) on the well-posed full-corpus giant CC | **predicted — untested** (EXP-RS-12, pre-registered 2026-07-04, LOCKED) | vault `agentic-experiments-research.md` § EXP-RS-12 |

## Active experiment

**EXP-RS-12** (→ Phase 31) — pre-registered 2026-07-04, predictions LOCKED. The 2026-07-04
reanalysis (human-directed: "reanalyze rather than inherit old decisions") found that Phases
29/30 failed on a **methodology artifact**, not the corpus/substrate: the BENCH_P10 gate is a
date-agnostic *recovery* metric (`dynamical-lbd.md` Criterion 3(a): corpus must "contain both
literatures"), so the C-1 pre-2015 slice was unnecessary and it alone shattered the citation
graph. The FULL `data-kuramoto` citation corpus is already well-posed: giant CC = 224 nodes (1
component, λ₂>0), 34 communities, n_eval=4. EXP-RS-12 runs the validated v03 citation pipeline on
this giant CC → the first real BENCH_P10. Stake: `BENCH_P10 > 0.15` revives the method; `≤ 0.15`
is a clean, well-posed negative → kill on solid ground. No new crawling. Full pre-registration:
vault `agentic-experiments-research.md` § EXP-RS-12. Conventions: C-12 (supersedes C-1), C-13.

*The Phase 30 pivot kill gate fired on EXP-RS-11 (TF-IDF), which remains dead. EXP-RS-12 is a
distinct, corrected experiment on the citation substrate — not a retry of the TF-IDF path.*

## Claim history

`CLAIMS.jsonl` (commission-compatible) is created by the first `/commission --research` run on
this thread; until then the table above is the claim record. (Exact format is an open Layer-2
spec question — session feedback welcome.)

## Last verification

2026-07-04 — Phase 30 verification (EXP-RS-11 FAIL; TF-IDF substrate fragments worse than
citation graph; pivot kill gate FIRED). Independently falsified-and-CONFIRMED via a right-sized
`/commission --research` (3 converging lines; no under-connection bug, no leakage, no
contamination). Kill-criterion check: **pivot gate condition met** (`BENCH_P10` not producible
≤ threshold; well before the 2026-09-30 deadline). Prior: 2026-05-05 Phase 29 FAIL.
