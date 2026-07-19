# NEXT SESSION START HERE — Research Synergy / cross-field LBD (2026-07-19)

Read this first, then `.planning/research/THREAD.md` (Active experiment) for full detail.

## Where we are (one screen)

The semantic-conceptual LBD chapter **reopened**. The reduction finder is a **validated cross-field
REDISCOVERY engine** (RS-26/27 works; RS-28 calibration 0/80 FP, 10× enrichment; RS-29/30 = 0 novel →
rediscovery-only). **BUT E1 (RS-31) temporal-holdout → PASS**: the field-neutral reduction cosine predicts
FUTURE cross-field bridges **beyond node degree**, including among **obscure** papers (Poor–Poor AUC
**0.754** vs degree-null 0.50, p=1e-5). → **the rediscovery ceiling was a SELECTION-OBJECTIVE artifact
(argmax = canonical), not a representation limit.** Novelty path reopened; E2/E3/E4 now falsifiable, graded
on the E1 benchmark.

**E3 (RS-32) done:** method/object split gate PASSED 5/5; **method-atom ranking strictly beats the
whole-reduction cosine** (Poor–Poor 0.832 vs 0.754) → carry the **method-atom substrate** forward. (The
specific object-distance "un-buries transfers" test was WEAK-underpowered — kept honest, not a PASS.)

**Tiering VALIDATED (ADOPT-MISTRAL):** Mistral reductions preserve the E1 signal (Poor–Poor 0.732 ≥ 0.60,
beats degree null). → **route all O(N) reductions to Mistral (off the Claude window); keep Opus for
precision only.** This fixes the rate-limit drain (~85–90% less window burn/experiment).

## NEXT = E2 (recommended, human already chose "E3 then E2")

**E2 — candidate-inference / analogical-residue reranking.** Design: `.planning/research/RS-DIRECTIONS-20260718.md` §E2.
Hypothesis: rank on the **UN-aligned projectable residue** (structure-mapping candidate inference), NOT
match-completeness → recover novel bridges that completeness-ranking (argmax) buries. This is the
highest-leverage objective-flip and doesn't depend on object-distance prevalence (unlike E3's stuck test).

**How to run it (concrete):**
1. **Pre-register first** (blind constants + KILL/PASS, SHA-freeze) — Phase 52.
2. **O(N) reduction → Mistral** (off-window; pattern in `rs33_tier_validate.py`; ≤4 workers). Extend
   `rs22_probe_mechanism` into a NEW frozen prompt that also emits a small relational schema
   (roles + relations), still blind (no partner) → leakage-free.
3. **Projection step (precision → Opus):** reuse the `rs22_probe_openbook` structure — from the aligned
   shared card, output base relations with NO target counterpart = candidate inferences. Run as Opus
   Workflow blind-subagent fan-out (hardcode keys in the script — `args` don't propagate).
4. **Rerank** candidates on projectable-residue novelty (not reduction cosine); gate each projected
   relation through the `rs29/30` two-hunter adversarial check ("is this relation asserted in the target
   field?").
5. **Grade** survivors on the E1 temporal benchmark (`rs31_temporal.py` machinery; 634 pool / 165
   positives / T=2010 already built).
6. **KILL:** 0 novel over the pre-registered budget AND no enrichment over random on the new candidate
   source. **PASS:** ≥1 novel-and-plausible survivor that also clears E1.

## Frozen / reusable assets (do NOT rebuild)

- Prompts: `rs22_probe_mechanism` (reduction), `rs22_probe_openbook` (card), `rs28_adjudicate`
  (skeptical adjudicator), `rs32_methobj` (method/object split — validated 5/5).
- Harnesses: `rs25_cascade` (retrieve→union→rerank), `rs28_control` (calibration), rs29/30 two-hunter
  novelty-gate, `rs31_temporal` (E1 benchmark + AUC/bootstrap), `rs32_asymmetric` (method/object),
  `rs33_tier_validate` (Mistral tier — validated ADOPT).
- Benchmarks: `rs22_mined_pairs` (420 timestamped bridges — the E1 substrate), `feynman_10pair`
  (deep-analogy anchors). Local embedders bge/gte/specter2.

## Operational gotchas (carry forward)

- **Model tiering:** O(N) reduction → **Mistral** (off Claude window, EU-first ✓, validated). Precision →
  Opus. Mistral ≤4 workers on the current tier (429 cap); for thousands-scale, validate `mistral-small` +
  more workers, or bump the Mistral tier.
- **Workflow `args` don't propagate** → hardcode/embed values in the script body (P-20260717-rs-8).
- **Large Opus fan-outs hit the session rate limit mid-run** (78/684, 120/634) → the tiering fix above is
  the answer; else run at window-start; harnesses are idempotent (skip-if-output-exists) so re-run-missing
  works.
- **Integrity:** blind-author + SHA-freeze decision constants before running; report honest rates; NEVER
  move the goalpost (E3 stayed WEAK despite a clear secondary effect); never call output "novel" without
  the objective-flip + two-hunter gate + E1 grading.

## Open decision (human's, at resume)

Proceed to **E2** (recommended) · OR harden E1 first (rolling multi-T + a from-scratch OpenAlex temporal
corpus) · OR jump to **E4** generation (gated on E2/E3 signal) · OR write up the reframed story now
(rediscovery ceiling = selection artifact; reduction has degree-independent prospective signal).
