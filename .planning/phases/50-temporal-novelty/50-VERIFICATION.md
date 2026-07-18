# EXP-RS-31 (E1) — Phase 50 Verification: Temporal-Holdout Novelty Benchmark

- **Status**: CONCLUDED — **PASS (pre-registered, decisive).** The field-neutral reduction cosine
  predicts FUTURE cross-field bridging **beyond node degree**, including on the graph-distant
  **Poor–Poor (both-obscure) stratum** where fame cannot explain the bridge. This is the first
  evidence the substrate carries a degree-independent, prospective novelty signal — **the rediscovery
  ceiling (RS-29/30) is a selection-objective artifact, not a representation limit.**
- **Date**: 2026-07-18
- **Pre-registration**: `50-PREREG.md` (LOCKED before scoring; harness SHA `cc506607…`).

## What ran

Substrate: the 420-pair mined benchmark = a timestamped set of REALIZED cross-field bridges (each pair
= side_a, side_b, and a dated **bridge_paper** that explicitly asserts the analogy). At **T=2010**: 634
pre-T pool papers, 165 future-bridge positives (both sides ≤ 2010 < bridge_paper). Reduce every pool
paper (frozen `rs22_probe_mechanism`; 332 reduced this phase + 302 reused → 634/634), embed (bge). Fetch
degree = Semantic Scholar `citationCount` (634/634, unauthenticated). Score AUC of the **reduction-cosine**
predictor vs the **preferential-attachment degree null** (`log1p(deg_A)+log1p(deg_B)`) separating
positives from all cross-archive ∧ lexical<0.06 non-positive pairs, per degree stratum, with a paired
bootstrap ΔAUC (seed 31).

## Result — PASS

| stratum | n pairs | positives | AUC(reduction) | AUC(pa-null) | ΔAUC | Mann–Whitney p |
|---|---|---|---|---|---|---|
| overall | 166,826 | 82 | **0.714** | 0.504 | +0.210 | ≈0 |
| **poor–poor** | 41,334 | 25 | **0.754** | 0.495 | **+0.259** | 1e-5 |
| rich–rich | 41,633 | 26 | 0.751 | 0.572 | +0.180 | ≈0 |

- Poor–poor ΔAUC paired-bootstrap 95% CI = **[0.084, 0.418]** (excludes 0). Positives = 25 ≥ N_min=20 → powered.
- Pre-registered PASS = Poor–Poor AUC(reduction) ≥ 0.60 ✓ (0.754), p<0.05 ✓ (1e-5), ΔAUC>0 with CI
  lower>0 ✓. **PASS.** (Only the 82 of 165 positives that are genuinely cross-archive ∧ surface-disjoint
  ∧ fully-reduced ∧ degreed enter scoring — the exact regime the finder operates in.)

## Circularity audit (clean)

The mined positives were selected **purely by a third paper explicitly asserting the analogy** (frozen
phrase patterns "is analogous to" / "in direct analogy with" …) + the bridge's citation structure
(`rs22_mining_protocol.md` §1.1–2.3). **No embedding, cosine, reduction, or similarity was ever used to
select pairs.** The reduction cosine is therefore a fully independent predictor of a human/paper-asserted
bridge → the PASS is not circular. The reduction is also computed blind (one paper, no partner, no dates,
no citations) → no temporal leakage into the predictor.

## Reading — this revises, not contradicts, the terminal narrative

- **Degree predicts future bridging at CHANCE (AUC 0.50). Reduction predicts it at 0.75** — and crucially
  it holds on the **obscure (Poor–Poor)** stratum, so the signal is not "famous papers get bridged."
- **Reconciliation with RS-29/30 (0 novel):** those measured the *retrieve-then-confirm* pipeline, which
  ranks by argmax reduction-cosine = the strongest match = the canonical known equivalence → surfaces only
  rediscoveries. E1 shows the predictive signal EXTENDS into the obscure stratum. Both hold. The synthesis:
  **the representation carries a degree-independent future-bridge signal; the SELECTION objective (argmax)
  was the limiter, not the representation.** The rediscovery ceiling is an objective artifact.
- **This is the instrument the thread lacked**, and it did double duty: it converts "novel" into a
  checkable prospective number AND returns a positive one. E2/E3/E4 are now falsifiable — a different
  ranking objective can be graded on this benchmark (does it harvest the obscure-but-predicted bridges?).

## Honest limitations (do NOT over-read)

- **PASS ≠ "we can surface novel bridges today."** It says the reduction *ranks* eventually-bridged pairs
  above degree-matched non-bridged ones, including obscure ones. With 25 positives against 41k negatives,
  AUC 0.75 is a real ranking signal but **low head-precision** — the top of the list is still mostly
  negatives. Turning this into a usable novel-bridge finder needs the objective-flip experiments (E2/E3)
  and a novelty gate; E1 proves the signal is THERE and worth harvesting.
- Degree is current citationCount, not as-of-T (coarse; can only inflate the null → conservative).
- Negatives may contain un-recorded real bridges (false negatives) → conservative, biases against PASS.
- Single T (2010), single substrate (the mined corpus, which over-samples asserted analogies). A rolling
  multi-T replication + a from-scratch OpenAlex temporal corpus would harden it (natural follow-up).

## Forward

The gap has moved: from "can the substrate reach novelty?" (**answered: yes, degree-independently**) to
"can a SELECTION objective harvest the obscure-predicted bridges into usable candidates?" → run **E2**
(candidate-inference / residue reranking) and **E3** (method/object asymmetric retrieval), both graded on
THIS benchmark; escalate to **E4** (generation) if they show signal. The write-up + product-pivot hedge is
now the fallback, not the base case.

Artifacts: `prototypes/rs31_temporal.py`, `data/rs31_{state,degree,verdict}.json`, `data/rs31_out/mechanism/` (reductions).
