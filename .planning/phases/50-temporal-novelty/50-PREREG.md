# EXP-RS-31 (E1) — Phase 50 PRE-REGISTRATION: Temporal-Holdout, Degree-Controlled Novelty Benchmark

- **Status**: PRE-REGISTERED — LOCKED 2026-07-18 (before any scoring). The instrument the thread
  lacked: a checkable, prospective ground truth for "novel bridge," replacing the adversarial web-guess
  (which failed 0/12 in RS-29/30). Harness SHA recorded after it is written.
- **Date**: 2026-07-18
- **Directions doc**: `.planning/research/RS-DIRECTIONS-20260718.md` (E1, rank 1).

## Motivation

RS-29/30 proved the reduction finder is a REDISCOVERY engine: retrieve-then-confirm ranks by argmax of
mechanism-similarity, and that argmax IS the canonical known equivalence → anti-novel by construction.
But "is this bridge novel?" had no ground truth — we adversarially web-guessed. This experiment installs
the field-standard LBD evaluation (temporal holdout, cf. Swanson replication / AGATHA / Science4Cast):
**freeze at year T, predict cross-field bridges from PRE-T information only, grade against bridges that
were actually asserted AFTER T** — with a degree-controlled null so canonical (high-degree) rediscoveries
cannot win by construction.

## Ready-made substrate (why this is powered, not rare)

The **420-pair mined benchmark** (`data/rs22_mined_pairs.json`, SHA `e7929b33…`) is a timestamped set of
REALIZED cross-field bridges: each pair = (side_a, side_b, **bridge_paper** with an asserted-analogy
snippet). Years come free from arXiv IDs (parser: `YY≥91→19YY`, else `20YY`). At **T=2010** (chosen for
power — a design parameter, not outcome-affecting): **634 pre-T pool papers, 165 future-bridge positives**
(both sides ≤ 2010 < bridge_paper). The bridge_paper being POST-T is exactly "the connection was made
after the prediction cutoff" → a genuine future bridge. This solves the rarity the RS-DIRECTIONS proposal
flagged for raw co-citation.

## Hypothesis

**H-RS-31:** the field-neutral **reduction cosine** between two pre-T papers predicts whether they get
BRIDGED after T, **beyond node degree**, specifically on the graph-distant **Poor–Poor** (both-obscure)
stratum where a canonical/famous pairing is not the explanation. If reduction only predicts bridging via
degree (i.e. only on Rich–Rich), the substrate is confirmed rediscovery-only.

## Design

- **Positives (label 1):** the 165 pairs with `max(year(side_a), year(side_b)) ≤ T < year(bridge_paper)`.
  Bridging is REAL and its assertion is POST-T (the bridge_paper's analogy snippet is recorded).
- **Pool:** all 634 side papers with year ≤ T. **Negatives (label 0):** all cross-archive (different
  top-level arXiv) ∧ surface-disjoint (tf-idf lexical cosine < 0.06) pairs among pool papers that are NOT
  positives. (Incompleteness note: some negatives may have bridged un-recorded → conservative, biases
  AGAINST a PASS.)
- **Predictor under test:** `reduction_cos(A,B)` — cosine of the two papers' independently-reduced
  field-neutral core mechanisms (frozen `rs22_probe_mechanism`, bge). Reductions are computed BLIND, one
  paper at a time, no partner/benchmark shown → no leakage from the pairing; and positives were selected
  via the bridge_paper, NOT via reduction cosine → no circularity.
- **Null (must beat):** `pa = log1p(deg_A) + log1p(deg_B)` — the preferential-attachment / degree
  predictor (co-citation ∝ degree; the rediscovery mechanism). `deg` = Semantic Scholar `citationCount`
  per paper (`ARXIV:<id>` batch). Limitation (documented, non-biasing): current count, not as-of-T — a
  coarse degree proxy; it can only inflate the null, making the reduction's job harder → conservative.
- **Strata:** median-split the pool by degree → Poor(<median)/Rich(≥median). **Poor–Poor** = both Poor
  (the decisive stratum); **Rich–Rich** = both Rich (consistency check).
- **Metric:** AUC (= Mann–Whitney U, prevalence-independent) of each predictor separating positives from
  negatives, computed OVERALL and per stratum. Primary comparison: `ΔAUC = AUC(reduction_cos) −
  AUC(pa)` on Poor–Poor, with a paired bootstrap 95% CI (2000 resamples, seed 31).

## Pre-registered decision (BLIND — authored before any scoring)

- **PASS — reduction carries a degree-independent novelty signal → unlock E2/E3/E4:** on **Poor–Poor**,
  `AUC(reduction_cos) ≥ 0.60` AND Mann–Whitney p < 0.05 AND `ΔAUC > 0` with bootstrap CI lower bound > 0
  (reduction beats the degree null among obscure papers). Expected companion: on Rich–Rich, reduction ≈ pa
  (both predict; ΔAUC ≈ 0) — bridging there is degree-driven.
- **KILL — rediscovery-only confirmed → ship the write-up + method-transfer pivot:** on **Poor–Poor**,
  `AUC(reduction_cos) ≤ 0.55` OR `ΔAUC ≤ 0` (reduction does not beat degree among obscure papers). The
  substrate predicts future bridging no better than fame → cannot reach novelty even measured honestly.
- **WEAK / underpowered:** in between, OR fewer than **N_min = 20** Poor–Poor positives (report the
  positives-per-stratum up front; treat the temporal hit-rate as a LOWER bound, never an upper bound).
- **INVALID (instrument broken):** OVERALL `AUC(reduction_cos) ≤ 0.5` (reduction fails to predict even
  aggregate bridging) → the substrate/label is broken, not a real KILL; re-examine.

## My prediction (pre-registered, honest)

**Genuinely uncertain — lean WEAK-to-KILL, ~35% PASS.** The whole prior arc says the strongest reduction
matches are canonical (high-degree), so I expect a clear reduction signal on Rich–Rich and OVERALL, but the
decisive Poor–Poor stratum is exactly where the substrate has never been shown to work. A PASS would be the
first evidence that reduction predicts bridges among OBSCURE papers beyond fame — the precondition that
makes generation (E4) and objective-flip (E2/E3) falsifiable. A KILL is decisive and routes to the honest
write-up + product pivot. Either outcome is a clean, publishable, pre-registered result.

## Scope / integrity

- This measures whether reduction predicts FUTURE bridging beyond degree — a prospective novelty signal at
  cutoff T (the bridge did not exist at prediction time). It is NOT a claim that any specific pair is
  unpublished today.
- Constants (T, degree source, median split, AUC thresholds, bootstrap seed, N_min) authored here before
  scoring; harness SHA-frozen below. Option (if productionised): blind-author the operational spec via a
  no-stake subagent (per P-20260717-rs-7).

## Frozen artifacts (SHA-256)

Locked 2026-07-18 before any scoring:

| artifact | SHA-256 | role |
|---|---|---|
| `prototypes/rs31_temporal.py` | `cc506607567336bd1b5354b1be462cdf0f9653738e8c13a8ae96c14441be3514` | harness (T, strata, AUC thresholds, bootstrap) |
| `50-PREREG.md` (pre-freeze body) | `28bbaf366d6765d040a9751314f6265fef2f068d32cc7fd063d164501da615d0` | predictions + KILL/PASS |
| `prototypes/rs22_probe_mechanism.md` | `72de22528b26480b120794f8050871930b54fff81e7c53b6f0e4f297e8509440` | reduction (frozen, unchanged) |
| `prototypes/data/rs22_mined_pairs.json` | (existing `e7929b33…`) | timestamped bridge substrate |

Build (deterministic, pre-scoring): T=2010 → pool 634, positives 165, degree 634/634 resolved
(Semantic Scholar `citationCount`, unauthenticated batch). 332 pool papers reduced this phase; 302
reused from prior runs (reductions are corpus-agnostic under the frozen prompt).
