# Phase 32 — Verification (Plan 32-01, EXP-RS-13)

**Date:** 2026-07-04
**Verdict:** ⚠️ **INCONCLUSIVE (confounded)** — the corpus fix worked (bridges now present and
surfaced in the Fiedler ranking), but the Kuramoto **dynamics did not converge** on the 1400-node
graph, so the run is not a valid test of the method. The locked prediction was met only in letter,
via a spurious same-community "detection." **No genuine signal claimed; not a clean negative.**

---

## Executed evidence (`prototypes/kuramoto_lbd_v06.py`, `data/kuramoto_v06_results.json`)

Bridged corpus giant CC: N=1398, 7 communities, 9598 edges.

| quantity | value | read |
|---|---|---|
| `K_stable` | **0.500** (= K_lo floor) | bisection collapsed to the low-K bound |
| `SC3` r_global @ 2·K_stable | **ABORT, r=0.136** | **system never synchronized** |
| `SC5` λ₂(L_sync) | **−0.0066**, gap=nan | sync manifold near-degenerate |
| global `BENCH_P10` | 0.100 (≤ 0.15) | ABORT (below baseline) |
| per-pair recall@10 | 0.25 | **met only via a spurious detection** (see below) |
| per-pair recall@any(top-200) | 0.75 | real bridges ARE in the ranking |
| nulls (ER, CM) `p10` | 0.0, 0.0 | — |

Per-pair bridge ranks in the full Fiedler list (of ~9600 edges):

| pair | inter-comm edges (corpus) | communities | first bridge rank | 
|---|---|---|---|
| pair01 Ising↔opinion | 91 | c0 / c1 | **not in top-200** |
| pair03 SIR↔rumour | (same community) | **c2 / c2** | rank 2 — **SPURIOUS** (same community) |
| pair04 percolation↔epidemics | 649 | c8 / c2 | **rank 17** |
| pair06 Turing↔economy | 66 | c3 / c1 | **rank 11** |

## Why this is not a valid test (three points)

1. **Dynamics did not converge.** `compute_K_stable` bisects for the minimum K where λ₂(L_sync)≥0
   — *local stability of a fixed point*, NOT actual synchronization. On the dense 1400-node graph
   it converged to a **low-K, locally-stable-but-scattered** fixed point (r=0.136). The Fiedler cut
   of an unsynchronized state is not meaningful, so every bridge rank here is unreliable. This is a
   genuine method limitation: the K-selection heuristic (`K_hi = max(4/λ₂(L_uw), 10)`) gives a *low*
   K ceiling for *dense* graphs, and the λ₂≥0 criterion admits scattered fixed points.
2. **The "detection" is spurious.** per-pair recall@10 = 0.25 comes entirely from **pair03, whose
   two endpoints landed in the SAME community (c2)** under the coarse 7-community Louvain. A
   "bridge spanning c2–c2" is a within-community edge, not a cross-domain bridge. **Genuine
   cross-domain recall@10 = 0.** I do NOT count the locked prediction as substantively met.
3. **Communities too coarse.** 7 communities on 1398 nodes (~200 nodes each) is too coarse to
   separate the benchmark domains — it collapsed pair03 and likely blurred others.

## What DID work (the real progress)

- The corpus-content gap from EXP-RS-12 is **fixed**: 3/4 evaluable pairs now have inter-community
  bridge edges (pair01:91, pair04:649, pair06:66), built by a neutral endpoint-neighborhood rule.
- The real bridges are **not random in the ranking**: pair04 (#17) and pair06 (#11) sit in the top
  ~0.2% of ~9600 edges (recall@any-top-200 = 0.75). Weak but non-null evidence the Fiedler ranking
  carries the bridge signal — *if* the dynamics were valid.

## Verdict vs locked prediction (EXP-RS-13)

Prediction: per-pair recall@10 ≥ 0.25. **Literally 0.25, but substantively 0** (the sole
detection is the pair03 same-community artifact). Decisive-read condition ("~0 despite bridges
present ⇒ clean method-negative") is **NOT** cleanly triggered either, because the run is invalid
(non-convergence). Honest status: **the fair test still has not run** — a *third* confound
(dynamical non-convergence at scale + coarse communities) now blocks it, after connectivity
(29/30) and content (31).

## Next (EXP-RS-14, pre-registered separately, this session)

Remove the convergence + granularity confounds to get the first VALID run on a connected,
bridge-containing, synchronized, finely-partitioned corpus — the test we have been trying to reach
since Phase 29. Principled changes (addressing diagnosed confounds, pre-registered before result,
reported either way — NOT benchmark-outcome tuning):
1. Smaller benchmark-centric bridged corpus (neutral reduction) that synchronizes like the
   converged 224-node Phase-31 run, and/or a sync-aware K criterion (min K s.t. r_global ≥ 0.7).
2. Finer Louvain (higher resolution) so the benchmark domains occupy distinct communities.
If, on such a corpus, genuine cross-domain recall@10 is still 0 → the cleanest possible
Kuramoto–Fiedler method-negative. If positive → real signal; formalize through resyn.

## Artifacts
- ✅ `prototypes/kuramoto_lbd_v06.py`, `data/kuramoto_v06_results.json` (vault-committed)
- ✅ This file
