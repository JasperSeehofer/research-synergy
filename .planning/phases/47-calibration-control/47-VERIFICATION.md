# EXP-RS-28 — Phase 47 Verification: Calibration Control for the Discovery Pipeline

- **Status**: CONCLUDED — **PASS (decisive).** The RS-26/27 discovery yield is REAL signal from
  the mechanism-reduction retrieval, not LLM permissiveness. The identical card+adjudicator
  pipeline confirms **0/80** random surface-disjoint cross-field pairs vs **6/40** reduction-
  selected ones (card stage) → **0/80 vs 5/40** genuine end-to-end → **10× enrichment**, Fisher
  one-sided p = **0.0035**. Pre-registered PASS condition met.
- **Date**: 2026-07-18
- **Pre-registration**: `47-PREREG.md` (LOCKED before any control LLM call; constants + decision
  SHA-frozen: prereg body `1f99d579…`, adjudicator `d124142e…`, harness `56e31fbe…`, card prompt
  `2c83eef3…`).

## What was tested

The chapter nulls everything except the discovery run. A capable LLM can find *some* shared
abstraction between almost any two math-heavy abstracts; the lexical<0.06 gate guarantees no shared
vocabulary but not that the shared *mechanism* is non-trivial. So: does the discovery pipeline's
~12.5% end-to-end genuine yield reflect real reduction-retrieval signal, or would the same
open-book-card + skeptical-adjudicator stack confirm genuine bridges at a similar rate on random
surface-disjoint cross-field pairs?

## Design (one thing changed: pair selection)

- **Control pairs (N=80):** from the frozen RS-27 corpus (140 papers), all pairs that are
  cross-archive ∧ lexical cos < 0.06 (**same surface-invisible gate as discovery**) ∧ NOT in the
  RS-27 reduction-selected candidate set (228 pairs) → random shuffle (seed 27) of the 8,467
  eligible → first 80. Isolates exactly the reduction signal: treatment = reduction-top-3 pairs;
  control = pairs the reduction did NOT flag. Control lexical median 0.0136 (≥ treatment's carded
  top-40 ~0.003–0.02) → conservatively biased toward confirming.
- **Downstream = byte-identical to treatment:** open-book card (frozen `rs22_probe_openbook.md`,
  80 blind Opus subagents, index-based dispatch) → any shares_method → skeptical blind adjudicator
  (`rs28_adjudicate.md`, anchored exactly as RS-27, sees `proposed_shared_mechanism`).
- **Harness-validity gate (reproduction check):** the RS-27 adjudicator prompt was never frozen to
  a file, so `rs28_adjudicate.md` is a faithful reconstruction. The same reconstructed adjudicator
  re-adjudicated treatment's 6 confirmed; must reproduce ≥ 4/6 (RS-27 original 5/6). This validates
  the reconstruction AND makes both arms use the identical adjudicator.

## Result — PASS

| metric | treatment (reduction-selected) | control (random, non-selected) | enrichment |
|---|---|---|---|
| card-confirm rate (shares_method) | 6/40 = 0.150 | **0/80 = 0.000** | **12×** |
| end-to-end genuine rate | 5/40 = 0.125 | **0/80 = 0.000** | **10×** |

- Fisher exact one-sided p (treatment enriched, end-to-end) = **0.00345** (< 0.05). Both
  pre-registered PASS conditions (p_e2e ≤ 0.05 AND Fisher p < 0.05) met. Result is stronger than
  the pre-registered prediction (predicted ≤ 2/80; actual 0/80).
- **Reproduction check: 5/6 genuine** on treatment's 6 (E0/E2/E3/E4/E5 genuine, E1 rejected as
  generic MaxEnt) — identical to the original RS-27 adjudication (5/6, same E1 rejection) → the
  adjudicator reconstruction is faithful; harness-validity gate PASSES.
- All 80 card outputs parsed cleanly (0 errors, 0 unparseable); 80/80 returned shares_method=false.

## Reading

1. **The discovery yield is well-calibrated.** The reduction retrieval is doing real work: it lifts
   the genuine-bridge rate from 0% (random surface-disjoint cross-field pairs) to 12.5%. The 5 RS-27
   "genuine" bridges are not the tail of an LLM that confirms everything — the same LLM confirms
   **none** of 80 random pairs.
2. **The open-book card stage alone is a strong gate.** All discrimination happened at the card
   stage (0/80 vs 6/40) — the adjudicator never received a control pair. The card writer is NOT a
   rubber stamp on any surface-disjoint pair; it already rejects generic/metaphorical overlaps. This
   is reassuring for the entire RS-26/27 method: the first, cheap filter carries most of the
   precision.
3. **Scaling is now justified.** With a measured 0% false-positive floor on random pairs and a
   validated adjudicator, scaling the finder to thousands of papers will surface more candidates
   without inflating the yield with noise.

## Honest limitations

- Because the card stage rejected all 80 controls, the adjudicator's *own* false-positive rate on
  control pairs was not measured directly (there were none to adjudicate). Mitigated by the 5/6
  reproduction check (the adjudicator is faithful and skeptical) — the end-to-end pipeline FP is
  what matters and it is 0/80.
- Adjudicator anchoring (it sees the proposed mechanism) was kept identical to treatment on purpose
  (this calibrates the ACTUAL pipeline). A de-anchored / cross-model adjudicator arm remains a
  separate, still-worthwhile hardening (chapter weakness #2), but the card-stage 0/80 result makes
  it non-blocking for scaling.
- Single corpus (the 140-paper RS-27 pool), N=80. The result is a clean lower-bound: 0 false
  positives out of 80 conservative (higher-lexical) random pairs.

Artifacts: `prototypes/rs28_control.py`, `prototypes/rs28_adjudicate.md`, `data/rs28_control_pairs.json`,
`data/rs28_card_confirmed.json`, `data/rs28_verdict.json`, `data/rs28_out/openbook/` (80),
`data/rs28_out/adj/treatment_recheck.json`.
