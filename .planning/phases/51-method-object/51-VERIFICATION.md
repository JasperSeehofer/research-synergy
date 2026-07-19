# EXP-RS-32 (E3) — Phase 51 Verification: Method/Object Asymmetric Retrieval

- **Status**: CONCLUDED — **WEAK-underpowered on the pre-registered primary** (the object-distant
  "un-buries transfers" test: only 3/82 positives are object-distant, < N_min=10). **BUT a clear,
  robust secondary finding: the method-atom cosine strictly DOMINATES the symmetric whole-reduction
  cosine as a future-bridge predictor** (overall AUC 0.813 vs 0.714; Poor–Poor 0.832 vs 0.754).
  Object-stripping sharpens the E1 signal. The Feynman split-validation gate passed cleanly (5/5).
- **Date**: 2026-07-18 → completed 2026-07-19
- **Pre-registration**: `51-PREREG.md` (harness SHA `c53224…`, prompt `911837…`, LOCKED pre-scoring).

## What ran

Feynman split-validation gate (10 method/object reductions) → **GATE-PASS** (5/5 deep analogies have
method-sim > object-sim; within-paper method↔object 0.714 < 0.85 non-degenerate; the physics
method/object-leak risk did NOT materialize). Then method/object reduce the 634-paper E1 pool
(633/634; one paper policy-flagged, excluded), embed method_atom + object_atom separately (bge), and
grade on the frozen E1 temporal benchmark: does the method-atom cosine predict future bridging (AUC)
better than the symmetric full-reduction cosine?

## Result

| stratum | positives | AUC(method) | AUC(symmetric, E1) | lift |
|---|---|---|---|---|
| overall | 82 | **0.813** | 0.714 | **+0.099** |
| poor–poor | 25 | **0.832** | 0.754 | **+0.079** |
| off-diag PP (object cos < τ=0.5) — **pre-reg PRIMARY** | **3** | 0.744 | 0.849 | −0.106 (CI [−0.46, 0.08]) |

- **Pre-registered verdict = WEAK-underpowered.** Only 3 of 82 positives clear the object-distance
  gate (cos(object) < 0.5), far below the pre-registered N_min=10. On those 3 the ΔAUC is negative but
  the CI spans 0 (pure noise at n=3). **The specific "symmetric buries object-distant transfers"
  mechanism cannot be confirmed here** — this corpus's mined bridges are mostly NOT extreme-object-
  distant at τ=0.5 (their objects still share abstract structure in bge space). This is a measurement
  limitation, not a refutation; I do NOT promote the main effect to a PASS (that would move the goalpost).
- **Robust secondary (exploratory, not the pre-reg primary): method-atom ranking strictly beats the
  symmetric full-reduction ranking** on both overall (0.813 vs 0.714) and the decisive Poor–Poor
  stratum (0.832 vs 0.754). Mechanistically sensible: the full reduction still carries residual object
  information that dilutes the mechanism match; the method_atom is object-stripped, so it clusters
  cross-field bridges tighter. This SHARPENS the E1 result — the objective/representation flip helps.

## Reading

- E3's headline mechanism (object-distance un-buries transfers) is **not testable in this corpus** (too
  few object-distant positives). Honest WEAK.
- But the experiment delivered a **better substrate for free**: replacing the whole-reduction cosine
  with the method-atom cosine lifts future-bridge AUC by ~0.08–0.10, including on the obscure stratum.
  **Carry the method-atom ranking forward as the E2 substrate.**
- The Feynman gate also banked a reusable, validated result: an LLM CAN cleanly split a paper into a
  transferable method vs a domain-specific object (5/5 on curated deep analogies) — the typed-atom
  decomposition the RS-DIRECTIONS "atomic structure" lens proposed is viable.

## Honest limitations

- The pre-registered primary is underpowered (3 positives) — a τ-relaxation or a corpus with more
  object-distant bridges would power the specific transfers test; that is a future powered follow-up,
  NOT to be run as a post-hoc τ-sweep on this data and re-labelled PASS.
- The method>symmetric main effect is exploratory (reported by the harness but not the pre-registered
  primary metric); it is robust (consistent overall + Poor–Poor, mechanistically expected) but should
  be pre-registered as the primary in any confirmatory rerun.
- Operational: the 634-paper Opus reduction hit the session rate limit at ~514/634 (finished via a
  115-paper retry after partial window recovery) → motivates model-tiering the O(N) reduction step
  (route to Mistral/Haiku; keep Opus for precision) before the next large run.

## Forward

Adopt **method-atom cosine** as the retrieval substrate (dominates the whole-reduction cosine). Run
**E2** (candidate-inference / analogical-residue reranking) on top of it, graded on the E1 benchmark —
E2 is the higher-leverage objective-flip and does not depend on object-distant prevalence. Route E2's
O(N) reduction to a cheaper model tier (validated against E1 first) to stop draining the Claude window.

Artifacts: `prototypes/rs32_asymmetric.py`, `rs32_methobj.md`, `data/rs32_{gate,verdict}.json`,
`data/rs32_out/{feynman,pool}/`.
