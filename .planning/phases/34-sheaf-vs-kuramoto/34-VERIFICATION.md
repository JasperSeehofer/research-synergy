# Phase 34 — Verification (Plan 34-01, EXP-RS-15)

**Date:** 2026-07-05
**Verdict:** ❌ **Prediction FALSIFIED — sheaf does NOT beat the benchmark bar; head-to-head both
methods score recall@10 = 0.** On the shared apples-to-apples metric, the cellular-sheaf frustration
detector recovers **0 of 4** benchmark bridges into the top-10, same as Kuramoto–Fiedler. The
dynamical/spectral graph-LBD hard core does not deliver on the shared 10-pair Feynman benchmark for
either method tested.

---

## Executed evidence (`prototypes/sheaf_lbd_v02.py`, `data/sheaf_v02_results.json`)

Testbed: `research_synergy_bridged_fine_sheaf.json` (1400 nodes, 34 communities with aggregated
c-TF-IDF, 4/4 evaluable pairs bridged, benchmark communities share 4–42 terms). Toy T1 self-test
PASS; SC2/SC3 PASS (sheaf Laplacian symmetric, PSD).

**Shared metric (identical to Kuramoto v07 — the honest head-to-head):**

| method | per-pair recall@10 | benchmark-pair ranks in the method's full bridge list |
|---|---|---|
| Kuramoto–Fiedler (v07) | **0.000** | all 4 pairs **not in top-200** (single global cut, all pairs same side) |
| Sheaf frustration (v02) | **0.000** | pair04 #69, pair01 #167, pair06 #188, pair03 #218, pair05 absent |

Neither surfaces a benchmark community-pair into the top-10. Sheaf T4 ablation **FALSIFIED (0/5)** —
its top-5 bridges do not pass the causal-ablation test.

## ⚠ The sheaf's self-reported "T2 precision@10 = 0.400 PASS" is a metric BUG — not a win

`compute_t2_precision` (sheaf v01, unchanged) is passed the **full** ranked bridge list but names
the argument `top10_bridges`, builds the set of **all** bridge community-pairs, intersects with the
5 ground-truth pairs (4 present → `hits=4`), and divides by a hard-coded 10 → "precision@10=0.400".
This is **"fraction of gt pairs present anywhere in the full list ÷ 10"**, not a top-10 precision.
The genuine top-10 metric (my `sheaf_perpair`, mirroring Kuramoto) is **0.000**. Do not read 0.400
as a sheaf success — flagged here so the vault registry/tournament does not inherit the error.

## Verdict vs locked prediction (EXP-RS-15)

Predicted: sheaf per-pair recall@10 ≥ 0.25 (or T2 precision@10 ≥ 0.2) — local frustration ranks ≥1
benchmark pair into the top-10 where the global cut got 0/4. **Actual: recall@10 = 0.000. FALSIFIED.**
The pre-registered decisive branch fires: *"sheaf = 0 too ⇒ NO graph method surfaces these bridges on
this corpus → strong push to the brute-force baseline."*

## Honest nuance (what little the sheaf did do)

The sheaf does rank 4/5 benchmark community-pairs *somewhere* in its full frustration list (ranks
69–218 of ~300+ community-pairs) — i.e. above random, and it found them at all (Kuramoto's edge-space
ranking did not surface them in its top-200). But the ranking objects differ (sheaf ranks
community-pairs; Kuramoto ranks paper-edges), so this is NOT a clean "sheaf > Kuramoto" — and on the
one shared, comparable metric (recall@10) they are **tied at 0**. Neither clears the bar.

## Recommendation (kill/continue is the human's per governance)

The two graph-dynamical/spectral detectors that were the substrate hypothesis's candidates —
Kuramoto single-cut (Phase 33) and sheaf frustration (Phase 34) — **both fail the shared benchmark
at recall@10 = 0 on a fully valid, bridge-containing corpus**. This satisfies the thread's
pre-registered method-level kill criterion. **Recommend: retire the dynamical-substrate LBD line and
revert to the brute-force baseline** (EXP-RS-10, BF-community-pairs LLM comparison) as the working
LBD method. Residual open threads (do not block the kill): (a) RAF on its reaction-model encoding
(different data model, untested here); (b) whether a *fixed* top-k metric or per-community-pair
normalization would change the picture — but that is a benchmark-design question, and the raw fact
(benchmark pairs rank 69–218, not top-10) would not move enough to pass.

## Artifacts
- ✅ `prototypes/sheaf_lbd_v02.py`, `data/sheaf_v02_results.json`, `SHEAF_V02_RESULTS.md`
- ✅ `data/research_synergy_bridged_fine_sheaf.json` (sheaf-ready testbed)
- ✅ This file

## Pack feedback
- **Same-metric head-to-head is essential.** The sheaf's built-in "precision@10" would have reported
  a false PASS (0.400); only re-scoring it with the *identical* metric used for Kuramoto exposed the
  tie at 0. When comparing methods, re-implement the metric once and apply it to all — never trust
  each method's self-reported score.
- A method's *self-test that passes on a toy* (sheaf T1 PASS) says nothing about corpus performance —
  keep toy-correctness and benchmark-recovery as separate gates.
