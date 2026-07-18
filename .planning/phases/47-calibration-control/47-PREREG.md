# EXP-RS-28 — Phase 47 PRE-REGISTRATION: Calibration Control for the Discovery Pipeline

- **Status**: PRE-REGISTERED — LOCKED 2026-07-18 (before any control LLM call). Predictions +
  decision thresholds frozen below; the harness (`prototypes/rs28_control.py`, blind constants
  N_CTRL=80/SEED=27/LEX_MAX=0.06) and the reconstructed adjudicator (`rs28_adjudicate.md`) are
  SHA-recorded at the end of this file.
- **Date**: 2026-07-18

## Motivation (the gap this closes)

RS-26/27 report a discovery yield of **~12.5% end-to-end** (RS-27: 40 carded → 6 card-confirmed
→ 5 adjudicated-genuine) and call 3 bridges "surprising". The whole chapter has been rigorous
about NULLS everywhere (lexical 0.40, random-chance, embedding) — **except the discovery run,
which has no control.** A capable LLM can find *some* shared abstraction between almost any two
math-heavy abstracts ("both Fokker–Planck", "both maximize entropy"). The lexical<0.06 gate
guarantees no shared vocabulary but **not** that the shared mechanism is non-trivial — that is
entirely the card-writer + adjudicator's job, and their false-positive rate has never been
measured. **Until we know the pipeline's FP rate on random surface-disjoint cross-field pairs,
the 0.15 card-rate / 12.5% end-to-end yield is uncalibrated and scaling it just produces more
uncalibrated candidates.**

## Hypothesis

**H-RS-28:** the discovery pipeline's genuine-bridge yield reflects **real signal from the
mechanism-reduction retrieval**, not LLM permissiveness on any surface-disjoint cross-field
pair. If real, the identical downstream pipeline run on RANDOM (non-reduction-selected)
surface-invisible cross-field pairs should confirm genuine bridges at a **substantially lower**
rate.

## Design (ONE thing changes: pair selection)

- **Corpus:** the frozen RS-27 external corpus (`data/rs27_corpus.json`, 140 papers, 12 cats).
- **Control pairs (N_CTRL = 80):** all unordered pairs that are (1) cross-archive (different
  top-level arXiv archive), (2) lexical cosine < 0.06 (**same surface-invisible gate as
  discovery**), (3) **NOT** in the RS-27 reduction-selected candidate set
  (`data/rs27_candidates.json`, 228 pairs) → random shuffle (SEED=27) → first 80. This isolates
  exactly the reduction signal: treatment pairs are reduction-top-3 (mechanism-nearest);
  control pairs are pairs the reduction did **not** flag.
- **Downstream = BYTE-IDENTICAL to treatment.** Open-book card = frozen `rs22_probe_openbook.md`
  (SHA `2c83eef3…`), blind Opus subagent per pair. Card-confirmed (shares_method) →
  skeptical blind adjudicator (`rs28_adjudicate.md`, SHA below), anchored exactly as RS-27 (sees
  `proposed_shared_mechanism`). Only the pair-selection differs; nothing downstream does.
- **Conservative bias (noted):** the treatment's carded top-40 were the extreme-low-lexical
  rank-1/2/3 pairs (~0.003–0.02); random lexical<0.06 control pairs average higher lexical → more
  surface hooks → if anything biased TOWARD confirming. So a low control rate is strong evidence.

## Harness-validity gate (reproduction check)

The RS-27 adjudicator prompt was never frozen to a file (ran ad-hoc). `rs28_adjudicate.md` is a
faithful reconstruction (same I/O contract + skeptic-by-default + reject-generic-meta-principle
behavior that rejected E1/B3/B4/B5). To validate it, **the same reconstructed adjudicator
re-adjudicates treatment's 6 confirmed** (`rs27_adjudicate_input.json`). It must recover
**≥ 4/6 genuine** (RS-27 original = 5/6). If < 4/6 → verdict = **HARNESS-INVALID** (reconstruction
drifted; fix before trusting the control). This also makes BOTH arms use the identical adjudicator
(removes prompt-drift confound).

## Metrics

- Control card-confirm rate `p_card = k1/80`; control end-to-end genuine rate `p_e2e = k2/80`.
- Treatment reference (FIXED): card 6/40 = 0.150; end-to-end 5/40 = 0.125.
- Enrichment `E_e2e = 0.125 / max(p_e2e, 1/80)`; enrichment_card analogously.
- Fisher exact one-sided p (treatment enriched) on the 2×2 [[5,35],[k2,80−k2]].

## Pre-registered decision (BLIND — frozen before running)

Evaluated only if the harness-validity gate passes (repro ≥ 4/6):

- **PASS — signal is REAL → scaling justified:** `p_e2e ≤ 0.05` **AND** Fisher one-sided p < 0.05.
  (⇒ control ≤ ~2/80 genuine; enrichment ≥ ~2.5×.) The reduction retrieval meaningfully enriches
  genuine bridges over the pipeline's baseline permissiveness → proceed to web-novelty check + scale.
- **FAIL — yield ≈ NOISE → do NOT scale; redesign:** `p_e2e ≥ 0.10` **OR** Fisher one-sided p > 0.10.
  (control genuine within ~0.8× of treatment.) The card+adjudicator stack confirms shared mechanisms
  on random pairs at ~the discovery rate → the 12.5% yield is LLM confabulation, not retrieval signal
  → the "discoveries" are uncalibrated → rethink (stricter adjudicator, cross-model check, or drop the
  discovery claim).
- **WEAK — inconclusive:** otherwise (0.05 < p_e2e < 0.10 with p<0.05). Report enrichment + note
  larger n or an unanchored/cross-model adjudicator arm is needed before scaling.

## My prediction (pre-registered, honest)

**PASS.** I expect control p_e2e ≤ 2/80 (enrichment ≥ ~5×): random surface-disjoint cross-field
pairs rarely share a *specific* governing equation, and both the open-book prompt and the skeptical
adjudicator explicitly reject generic meta-principles/metaphor (the adjudicator already rejected E1
= generic MaxEnt). **Named risk:** if the LLM is permissive enough to confirm "both diffusion /
both optimization / both entropy" on ~5–10% of random pairs, this lands WEAK/FAIL — which would
itself be the important finding (the discovery yield is inflated by confabulation, and every RS-26/27
"genuine" count needs re-reading net of that baseline).

## Scope / what this does NOT test

- Not a novelty check (that's the deferred web/literature step). "Genuine shared mechanism" here =
  the pipeline's own adjudicated `mechanism_real`, calibrated against random pairs — NOT "unpublished".
- Adjudicator anchoring (it sees the proposed mechanism) is kept IDENTICAL to treatment on purpose:
  this calibrates the ACTUAL pipeline that produced the discoveries. De-anchoring is a separate
  improvement, out of scope here.

## Frozen artifacts (SHA-256)

Locked 2026-07-18 before any control LLM call:

| artifact | SHA-256 | role |
|---|---|---|
| `prototypes/rs22_probe_openbook.md` | `2c83eef3117d69db982a89e732ca12f635eb86af7c550cdb5d03b2679ad1e6b0` | card stage (frozen, unchanged from RS-22/26/27) |
| `prototypes/rs28_adjudicate.md` | `d124142e6ab56c46457fb4d9598a2ec95ddd2127e77311bf20f42ef96b72b389` | reconstructed skeptical adjudicator (both arms) |
| `prototypes/rs28_control.py` | `56e31fbe470c904c765cf87da53c30f965a04822ac52703027280ded611a1c8d` | harness (N_CTRL=80, SEED=27, LEX_MAX=0.06, thresholds) |
| `47-PREREG.md` (this file, pre-freeze body) | `1f99d57987fb87f6b68e14861c68e8617e2085ee6cc11b830c91b1ddccfe7c57` | predictions + decision (self-SHA of the body above this table) |

Control pair set: `data/rs28_control_pairs.json` — 80 pairs from 8467 eligible non-candidate
cross-archive surface-invisible pairs (SEED=27); control lexical_cos min 0.0 / median 0.0136 /
max 0.0539 (treatment's carded top-40 were the extreme-low tail ~0.003–0.02 → control conservatively
biased toward confirming).
