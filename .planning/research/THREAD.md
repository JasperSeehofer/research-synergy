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
| H-RS-substrate | Cellular **sheaves** over the Louvain community graph detect multi-causal bridges better than RAFs or Kuramoto | 4-tier benchmark incl. multi-causal joint-removal ablation on the shared 10-pair Feynman set | live (**pivoting** — substrate rebuild via EXP-RS-11) |

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
| TF-IDF cosine edges (τ=0.3) make the same corpus connected enough for spectral/dynamical LBD (`n_cc/N ≤ 0.05`, largest CC ≥ 80%) | predicted — untested (EXP-RS-11, pre-registered) | vault: `wiki/meta/agentic-experiments-research.md` § EXP-RS-11 |

## Active experiment

**EXP-RS-11** (→ Phase 30) — TF-IDF cosine semantic-edge substrate. Full pre-registered
hypothesis / predictions / setup / metrics: vault `wiki/meta/agentic-experiments-research.md`
§ EXP-RS-11. Decision provenance: `.cartographer-notes.md` (pivot approved 2026-07-02, human;
recorded in vault hypothesis-ledger, research-portfolio, decision-timeline).

## Claim history

`CLAIMS.jsonl` (commission-compatible) is created by the first `/commission --research` run on
this thread; until then the table above is the claim record. (Exact format is an open Layer-2
spec question — session feedback welcome.)

## Last verification

2026-05-05 — Phase 29 verification (FAIL verdict, honest negative; gate not reached).
