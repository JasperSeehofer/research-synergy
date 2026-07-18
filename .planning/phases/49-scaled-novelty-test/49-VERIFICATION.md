# EXP-RS-30 — Phase 49 Verification: Scaled Novelty Test

- **Status**: CONCLUDED — **KILL (pre-registered, terminal).** At ~5× scale (684 papers) WITH explicit
  weak-match-tail hunting, the finder surfaces **0 novel bridges**: of 5 adjudicated-genuine bridges,
  3 known-cross-field / 2 specialist-known / **0 robust-novel / 0 candidate-novel**. Decisively, the
  **tail stratum (reduction ranks 4–12, where RS-29 predicted novelty would live) produced 0 genuine
  bridges** (1/40 card-confirmed, 0 survived adjudication) — the weak matches are spurious, not
  hidden-novel. Combined with RS-29 (0/7 novel at n=140), this is a strong cumulative terminal signal:
  **the finder is a validated cross-field REDISCOVERY engine, not a novelty engine.**
- **Date**: 2026-07-18
- **Pre-registration**: `49-PREREG.md` (LOCKED before any RS-30 LLM call; harness SHA `e8c45b3f…`).

## What ran

684 fresh arXiv papers (18 categories, un-mined) → 684 blind Opus mechanism-reductions (78 hit a
session rate limit; re-run cleanly → 684/684 valid) → reduction-embed candidate extraction in **two
strata**: TOP (ranks 1–3, canonical positive-control) and TAIL (ranks 4–12, the novelty hunt) →
stratified 100 cards (60 top + 40 tail) → frozen skeptical adjudicator → **two-hunter** adversarial
novelty-gate on every genuine bridge → pre-registered scoring.

## Result — KILL

| stage | TOP (rank 1–3) | TAIL (rank 4–12) |
|---|---|---|
| candidates (cross-archive ∧ lexical<0.06) | 1009 | 3406 |
| carded | 60 | 40 |
| card-confirmed (shares_method) | 13 (21.7%) | **1 (2.5%)** |
| adjudicated-genuine | 5 (8.3%) | **0** |
| novelty: known-crossfield / specialist / robust-novel | 3 / 2 / **0** | 0 / 0 / **0** |

- **Positive control PASSES:** the TOP stratum yields 5 genuine bridges (8.3% end-to-end, matching
  RS-26/27's ~12.5%) → the pipeline works at scale; this is not a broken-pipeline INVALID.
- **`robust_novel = 0`, `candidate_novel = 0` → pre-registered KILL.**
- The 5 genuine bridges (all TOP, all standard machinery): **S10** cosmology Fisher-forecast ↔ signal-
  processing Cramér–Rao (known); **S11** Horvitz–Thompson ≡ importance sampling (known, hunter found it
  explicitly published); **S13** information-bottleneck / rate-distortion, ANN generalization ↔ bounded-
  rational demand (known); **S1** pitchfork symmetry-breaking bifurcation, economic geography ↔ black-hole
  scalarization (specialist — a *Physics Reports* review explicitly lists both economics and GR under
  equivariant SSB bifurcation); **S7** Bogoliubov / adiabatic-breakdown occupation, gap solitons ↔
  leptogenesis (specialist).

## Reading — the decisive finding is the empty tail

RS-29 localized the failure to "the top-3 are canonical; novelty, if anywhere, is in the weak-match
tail." RS-30 tested exactly that and **the tail is empty of genuine bridges**: 40 tail candidates → 1
card-confirmed → 0 adjudicated-genuine. So the weak reduction matches are weak because they **do not
share machinery** (spurious), not because they share *hidden* machinery. This closes the last escape
hatch — novelty is not reachable by hunting weaker matches. The finder reliably surfaces REAL shared
mechanisms (positive control 8.3%, calibration-validated in RS-28), but at every scale those are
KNOWN/specialist equivalences.

## Terminal chapter conclusion

Across the discovery arc: **RS-26/27** the finder works (surfaces real surface-invisible bridges);
**RS-28** the bridges are real, not confabulation (0/80 FP calibration, 10× enrichment); **RS-29** at
n=140 all 7 genuine bridges are known/specialist (0 novel); **RS-30** at 5× scale WITH tail-hunting,
still 0 novel and the tail is empty. **The durable, honest deliverable is a calibrated, validated,
unsupervised cross-field analogy-REDISCOVERY engine** — it recovers, from abstracts with ~0 lexical
overlap, equivalences that took human insight to find (Baake–Baake–Wagner; Hofbauer–Sigmund; the Deift
RH program; Horvitz–Thompson≡IS). It is NOT a demonstrated novelty-discovery engine, and there is now
converging evidence (two scales, two designs) that it will not become one by scale or tail-hunting
alone. Do not describe its output as novel discoveries.

## Honest limitations

- N=684 is ~5×, not "thousands"; but the tail being *empty of genuine bridges* (not merely
  empty-of-novel) makes a 10×/100× scale-up unlikely to change the class — more tail candidates would
  be more spurious pairs, not more genuine-novel ones. A true-thousands run remains the only way to be
  fully certain, and is the natural (low-expected-value) follow-up if desired.
- Novelty "not_found" is a weak positive (absence ≠ novelty); but here it only *reinforces* KILL — the
  gate found MORE prior art than expected (3/5 known), and 0 bridges even reached novel_looking.
- Agent-mediated search could miss sources; direction of error (finding more prior art) is conservative
  for the KILL.

Artifacts: `prototypes/rs30_scale.py`, `data/rs30_{corpus,candidates,carded,adjudicate_input,
novelty_input,novelty_verdicts,verdict}.json`, `data/rs30_out/` (reductions + cards + adjudication).
