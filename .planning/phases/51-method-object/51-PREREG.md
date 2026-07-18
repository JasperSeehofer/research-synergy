# EXP-RS-32 (E3) — Phase 51 PRE-REGISTRATION: Method/Object Asymmetric Off-Diagonal Retrieval

- **Status**: PRE-REGISTERED — LOCKED 2026-07-18 (before any scoring). Blind constants + KILL/PASS
  frozen; prompt/harness SHA recorded after writing.
- **Date**: 2026-07-18
- **Directions doc**: `.planning/research/RS-DIRECTIONS-20260718.md` (E3). Builds on E1 (RS-31) PASS.

## Motivation

E1 proved the reduction carries a degree-independent future-bridge signal — the rediscovery ceiling is a
SELECTION-objective artifact. E3 is the cheapest objective-flip: a cross-field analogy is the **SAME
method applied to a DIFFERENT object**, so ranking on the **method atom** under a hard **object-distance
gate** should surface the "off-diagonal" transfers that the symmetric whole-mechanism cosine *buries*
(their whole-mechanism similarity is dragged down by object dissimilarity).

## Design

- **Typed atomization:** a NEW prompt `rs32_methobj.md` (frozen; the existing `rs22_probe_mechanism`
  stays untouched) splits each paper into `method_atom` (transferable, field-neutral machinery) +
  `object_atom` (domain-specific system). Blind, one paper at a time → leakage-free, same as the reduction.
- **Embed** method_atom and object_atom separately (bge). Predictors on a surface-disjoint (lexical<0.06)
  cross-field pair: **symmetric** = full-reduction cosine (the E1 winner); **method** = cos(method_A,
  method_B); object-distance = cos(object_A, object_B).
- **Grade on the E1 temporal benchmark** (same 634-paper pool, 82 cross-field positives, degree strata).

## Precondition gate (Feynman split-validation — CHEAP, RUN FIRST)

The proposal's load-bearing risk: physics "objects" are often defined by their methods → the split may be
leaky/degenerate. Reduce the **10 endpoint papers of the 5 evaluable Feynman deep analogies** (Ising↔opinion,
SIR↔rumour, percolation↔epidemics, Lotka-Volterra↔markets, Turing↔spatial-economy) with `rs32_methobj`, then:
- **Separability:** for the 5 deep-analogy pairs, `cos(method_A,method_B) > cos(object_A,object_B)` for
  **≥ 3/5** (the analogy lives in the method, not the object).
- **Non-degeneracy:** mean within-paper `cos(method_atom, object_atom)` **< 0.85** (the two atoms are
  distinct, not paraphrases).
- **GATE FAILS (either condition) → KILL E3** — the method/object split is leaky/degenerate on the exact
  deep analogies it must separate; do not spend the 634-paper pool reduction. Move to E2.

## Main test (on E1, ONLY if the gate passes)

Reduce the 634 pool papers with `rs32_methobj`; on the **Poor–Poor** stratum of the E1 labeled set:
- **PASS:** `AUC(method) ≥ AUC(symmetric) − 0.02` (the flip does not lose) AND, restricted to the
  **object-distant positives** (real future bridges with `cos(object) < τ`), the method predictor ranks
  them strictly higher than the symmetric predictor: `ΔAUC_offdiag = AUC(method) − AUC(symmetric) > 0`
  with paired-bootstrap 95% CI lower bound > 0. (= the objective-flip un-buries transfers.)
- **KILL:** `AUC(method) < AUC(symmetric) − 0.02` on Poor–Poor (the flip loses) OR `ΔAUC_offdiag ≤ 0`
  (no lift for object-distant transfers) → method/object ranking is not a novelty lever here.
- **WEAK:** in between, or fewer than **10** object-distant Poor–Poor positives (underpowered → report).

## Blind constants (frozen)

`τ = 0.5` (object-distance gate); separability gate `≥ 3/5`; non-degeneracy `< 0.85`; `ε = 0.02`;
bootstrap 2000 resamples, seed 32; off-diagonal min positives 10. Authored before any scoring.

## My prediction (pre-registered, honest)

**Genuinely uncertain, ~45% the Feynman gate even passes.** The method/object split is conceptually right
but the leak risk is real (esp. in physics). If the gate passes, I lean toward a modest PASS — E1 already
shows method-ish signal, and object-distance is the defining axis of a transfer — but the "buries transfers"
effect must actually show up as ΔAUC_offdiag > 0, which is not guaranteed. A gate-FAIL is a fast, cheap,
honest KILL that routes straight to E2 (the higher-leverage bet) with no wasted pool reduction.

## Integrity

Blind atomization (no partner/dates); decision constants authored pre-scoring; graded on the frozen E1
benchmark; the Feynman gate prevents an expensive run on a leaky instrument. `rs32_methobj` is an
extraction instrument (not a decision constant); option to blind-author via a no-stake subagent if productionised.

## Frozen artifacts (SHA-256)

Locked 2026-07-18 before any scoring:

| artifact | SHA-256 | role |
|---|---|---|
| `prototypes/rs32_methobj.md` | `91183788ae8be6f0ec9545da64d3857cc9ff773395ec88f24dc8d6699ec5a0c5` | method/object extraction instrument |
| `prototypes/rs32_asymmetric.py` | `c532242f487f7e4ba85a5e8017d398c82dc44894f37388b6b8a42bb4a17b66d7` | harness (τ, gate, AUC, bootstrap) |
| `51-PREREG.md` (pre-freeze body) | `979ba6e50f165ec232b41859f7e3b52b7deefb8a7845ca99c41b14a04dd254a5` | predictions + KILL/PASS |

Graded on the frozen E1 benchmark (`rs31_state.json`, `rs31_degree.json`) + Feynman anchors
(`feynman_10pair_papers.json`, `mvp_corpus.json`).
