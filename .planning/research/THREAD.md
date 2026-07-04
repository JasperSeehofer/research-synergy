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
| H-RS-substrate | Cellular **sheaves** over the Louvain community graph detect multi-causal bridges better than RAFs or Kuramoto | 4-tier benchmark incl. multi-causal joint-removal ablation on the shared 10-pair Feynman set | **BLOCKED** — no viable substrate on this corpus. Citation graph fragments (Phase 29 FAIL); TF-IDF semantic edges fragment *worse* (Phase 30 FAIL, EXP-RS-11). Pivot kill gate FIRED 2026-07-04. Kill/Path-B decision → human via vault. |

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
| The pre-2015 cond-mat corpus (N=153) is too narrow to support *any* semantic/citation substrate for dynamical LBD; Path B (seed selection / newer corpus) is the remaining option | verified (Phase 29 + Phase 30 FAIL, both substrates fragment) | 30-VERIFICATION.md § "premise is falsified" |

## Active experiment

**None active.** EXP-RS-11 (→ Phase 30) concluded FAIL 2026-07-04: the TF-IDF cosine
semantic-edge substrate fragments worse than the citation graph at every pre-registered τ;
precheck fails, `BENCH_P10` not producible, **pivot kill gate FIRED**. Verdict survived
independent falsification (3 converging lines). Per pre-registered follow-ups, the corpus is
too narrow — Path B (seed selection / newer corpus) is the remaining option. The kill vs Path-B
decision is the human's, routed via the vault (`.cartographer-notes.md` → hypothesis-ledger).
Full record: `.planning/phases/30-tfidf-semantic-edge-graph/30-VERIFICATION.md`.

**Awaiting human go/kill/pivot decision before any new experiment is opened.**

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
