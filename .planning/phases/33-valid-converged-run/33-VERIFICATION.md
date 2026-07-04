# Phase 33 — Verification (Plan 33-01, EXP-RS-14)

**Date:** 2026-07-04
**Verdict:** ❌ **CLEAN METHOD-NEGATIVE (mechanistically explained).** On a fully well-posed corpus
— connected ∧ bridge-containing ∧ synchronized ∧ finely-partitioned — the Kuramoto–Fiedler
dynamical-LBD method recovers **0 of 4** benchmark bridges. The locked prediction (per-pair
recall@10 ≥ 0.25) is **FALSIFIED**. Root cause identified and verified: the method extracts bridges
from a **single global Fiedler bipartition**, and all benchmark pairs sit on the **same side** of
that cut → they are structurally invisible to it.

---

## Executed evidence (`prototypes/kuramoto_lbd_v07.py`, `data/kuramoto_v07_results.json`)

Corpus `research_synergy_bridged_fine.json` giant CC: N=1398, **32 communities**, 9598 edges.

**Every well-posedness condition is met (the confounds of Phases 29–32 are all removed):**

| condition | status |
|---|---|
| Connected (single component) | ✅ giant CC |
| Contains the bridges | ✅ 4/4 evaluable pairs have inter-community edges (pair01:9, pair03:23, pair04:27, pair06:4) |
| Synchronized (valid dynamics) | ✅ K_sync=12, r_global=0.932; SC3 **PASS**; SC5 λ₂=0.229>0, gap=57 **PASS** |
| Finely partitioned (pairs separated) | ✅ 32 communities; all 4 pairs in distinct communities |
| Null-controlled | ✅ ER + config-model nulls (topology-sensitive) |

**Result:** `BENCH_P10 = 0.000`, `per-pair recall@10 = 0.000`, **`recall@any(top-200) = 0.000`** —
not one benchmark community-pair appears among the top-200 Fiedler bridges (of ~9600 edges). All
four `perpair_ranks` are `None`.

## Root cause (verified, not asserted)

`extract_fiedler_bridges_sparse` defines a bridge as an edge crossing the **sign of the single
Fiedler eigenvector** (`v2 = eigvecs[:,1]; bridge_mask = signs[i] != signs[j]`). The Fiedler cut is
one global bipartition. Diagnostic on the graph Laplacian Fiedler vector:

```
Global Fiedler bipartition: side0 = 834 nodes, side1 = 564 nodes
  pair01 (Ising↔opinion):      A side0, B side0  -> SAME side (invisible)
  pair03 (SIR↔rumour):         A side0, B side0  -> SAME side (invisible)
  pair04 (percolation↔epi):    A side0, B side0  -> SAME side (invisible)
  pair06 (Turing↔economy):     A side0, B side0  -> SAME side (invisible)
```

**All benchmark pairs lie on the same side of the global cut.** Their inter-community bridge edges,
which genuinely exist, never cross the Fiedler sign boundary, so they can never be ranked as Fiedler
bridges — recall is structurally 0 independent of the dynamics or the bridge count. A single global
bipartition cannot surface multiple independent, locally-cross-domain bridges scattered within one
half of the graph.

(Note: this valid run scored *worse* than the confounded v06, where pair04/06 ranked #11/#17. That
is expected — v06's non-converged low-K state produced a noisier, more diffuse "Fiedler" vector
whose cut happened to clip a couple of pairs; the properly synchronized cut is cleaner and cleanly
excludes them. Cleaner dynamics, sharper negative.)

## Verdict vs locked prediction

Predicted per-pair recall@10 ≥ 0.25 → **actual 0.000. FALSIFIED.** This is the pre-registered
decisive outcome: "still 0 on a valid run ⇒ the cleanest Kuramoto–Fiedler method-negative." It is
**not** a confound — connectivity, corpus content, convergence, and community granularity are all
demonstrably handled. The method genuinely does not recover the benchmark bridges.

## What the five-phase arc actually produced (this is constructive, not a dead end)

1. **A corpus-construction methodology** — targeted OpenAlex citation-neighborhood fetch that
   builds a benchmark-centric, bridge-containing graph (`build_bridge_corpus_openalex.py`).
2. **A valid benchmark testbed** — `research_synergy_bridged_fine.json` (1398 nodes, connected,
   4/4 pairs bridged, synchronizing, 32 communities). This is exactly the well-posed corpus the
   original sheaf/RAF tournament (H-RS-substrate) always needed and never had.
3. **A clean, mechanistic refutation of the Kuramoto–Fiedler baseline** for multi-bridge recovery.
4. **A sharp, testable prediction**: the failure is the *single global cut*. Methods that detect
   bridges **locally / multi-scale** — cellular **sheaves** (local sections; the original
   H-RS-substrate hypothesis) or per-community-pair frustration, or a multi-eigenvector cut — are
   predicted to succeed where the single Fiedler cut fails. The negative motivates the sheaf
   direction rather than killing the thread.

## Recommendation (kill/continue is the human's per governance)

- **Kill** the Kuramoto–Fiedler single-cut branch as an LBD method — cleanly refuted.
- **Continue** H-RS-substrate on the now-valid testbed: run the sheaf/RAF/Kuramoto tournament
  (`/cartographer --tournament`, from the vault — out of scope for this repo session) on
  `research_synergy_bridged_fine.json`, with the pre-registered prediction that local/multi-scale
  detectors beat the single global cut. If sheaves ALSO recover 0/4 here, the whole dynamical-LBD
  hard core is refuted on a fair test → revert to the brute-force baseline.

## Artifacts
- ✅ `prototypes/kuramoto_lbd_v07.py`, `data/kuramoto_v07_results.json` (vault-committed, run-before-commit)
- ✅ `data/research_synergy_bridged_fine.json` (the valid testbed)
- ✅ Fiedler-cut diagnostic (this file, § Root cause)
- ✅ This file

## Pack feedback (EXP-RS-14 / the five-phase arc)

- **Systematically removing one confound at a time was the winning strategy.** Each phase isolated
  a distinct blocker (connectivity → content → convergence → granularity) and the honest "not a
  clean test yet" verdict at each step prevented a false negative *and* a false positive. Only after
  all four were removed did the negative become interpretable. A method that "fails" should be
  driven to a *well-posed* failure before the failure is believed.
- **Gate on ACTUAL synchronization (r_global ≥ θ), not fixed-point stability (λ₂≥0), before scoring
  a dynamical run.** The λ₂≥0 criterion admitted a scattered state (Phase 32); `find_K_sync` fixed
  it. Reusable guardrail for the pack.
- **Run a static structural precheck before a dynamical benchmark**: "do the benchmark pairs'
  communities straddle the method's cut / share an edge?" This one-line check explains the negative
  and would have flagged the single-cut mismatch immediately.
- **Pre-registration under a moving target held.** Four locked predictions across four phases, each
  reported against its lock; the corpus was never tuned to the benchmark outcome (neutral
  endpoint-neighborhood rule; principled confound fixes only). The discipline is what makes this
  negative credible.
