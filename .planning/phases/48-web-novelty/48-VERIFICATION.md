# EXP-RS-29 — Phase 48 Verification: Web / Literature Novelty Check

- **Status**: CONCLUDED — **sobering but decisive.** Of the 7 genuine (non-textbook) cross-field
  bridges from RS-26/27, **0 are novel-looking; 3 are specialist-known; 4 have the exact cross-field
  identity EXPLICITLY published.** The finder is a validated *rediscovery* engine — it correctly
  surfaces real shared machinery with ~0 lexical overlap — but at 140-paper scale it produces
  **known / specialist equivalences, not novel bridges.** The "surprising" model-knowledge label
  systematically over-rated novelty.
- **Date**: 2026-07-18

## What was tested

The honest gap the calibration control did NOT close: RS-28 proved the bridges are REAL (not
confabulation), but "real" ≠ "novel". This check asks, per bridge, whether the **cross-field
mechanism connection itself** is already in the literature. Method: an **adversarial** prior-art
hunter (general-purpose agent, WebSearch/WebFetch, told to FIND published prior art across ≥4–6
query framings, so a "not found" is meaningful) → a **skeptical** novelty classifier that reads the
evidence and assigns `known_crossfield` / `specialist_known` / `novel_looking` (conservative: prefer
specialist_known when in doubt). 7 bridges × (hunt → classify), 14 agents, 0 errors.

## Result — 0 novel / 3 specialist-known / 4 known-cross-field

| # | run / rating | bridge | verdict | decisive prior art |
|---|---|---|---|---|
| **B6** | RS-26 surprising | quasispecies/HGT ↔ quantum annealing (mutation ≡ transverse field) | **known_crossfield** | Baake–Baake–Wagner, *PRL* 78, 559 (1997): "Ising Quantum Chain is Equivalent to a Model of Biological Evolution" — the exact identity is a published theorem |
| **B2** | RS-26 specialist | Casimir (T-operator) ↔ quantum-graph periodic orbits (Tr ln(1−S)) | **known_crossfield** | Fulling–Kaplan–Wilson (quant-ph/0703248, 2007) compute Casimir energy on quantum graphs via the Kottos–Smilansky secular determinant; Wirzba (0711.2395) |
| **B7** | RS-26 specialist | orthogonal polynomials / equilibrium measures ↔ semiclassical focusing NLS (2×2 Riemann–Hilbert) | **known_crossfield** | the RH + weighted-equilibrium g-function core is the explicit, repeatedly-published Deift-school program (Notices-AMS survey; 0708.3867) |
| **E0** | RS-27 specialist | stochastic ultimatum game ↔ predator-prey replicator | **known_crossfield** | Hofbauer–Sigmund replicator ↔ Lotka–Volterra equivalence (1998 Cambridge, canonical textbook) |
| **E5** | RS-27 surprising | galaxy bias/CMB-lensing ↔ Bayesian retail survival (log-Gaussian Cox process) | **specialist_known** | both sides textbook (Coles–Jones 1991 lognormal; Møller 1998 LGCP + Diggle 2010 preferential sampling); spatial-stats even cites Coles–Jones as the LGCP origin (Martinez–Saar monograph bridges the fields); no explicit CMB-bias↔retail-survival pairing found |
| **E3** | RS-27 surprising | rough-surface contact ↔ option pricing (Fokker–Planck) | **specialist_known** | both textbook (Persson magnification-diffusion; Black–Scholes = heat equation); ≈ "both are diffusion PDEs" — near-universal machinery; no explicit pairing found |
| **E2** | RS-27 surprising | spacetime topology ↔ quantum-TDA financial stress (simplicial homology) | **specialist_known** | the "shared mechanism" is the generic definition of TDA (Betti numbers of a Vietoris–Rips complex), standard on both sides; adjudicator deflated it to a shared *tool*, not a shared governing equation |

## Reading (honest)

1. **The bridges are real — 7/7. The novelty proxy was wrong — 7/7.** Every finder-adjudicated
   "genuine" bridge is a real shared mechanism (confirming RS-28 and my own domain read), but NONE is
   novel to the literature. 4 are explicitly published (two of them *famous* — Baake–Baake–Wagner and
   Hofbauer–Sigmund); 3 are standard machinery on both sides. **"Surprising to the model" tracks
   "non-obvious to a generalist," not "unpublished."** The novelty column in RS-26/27 must be read as
   an obviousness proxy, NOT a novelty claim — exactly the over-claim the pre-registration warned against.
2. **The method is a validated cross-field ANALOGY-REDISCOVERY engine, not (yet) a novelty engine.**
   It rediscovers, from abstracts with ~0 word overlap, equivalences that took human insight to find
   (a 1997 PRL, the Deift RH program). That is a real, honest capability — and a legitimate
   contribution — but it is not the "novel discovery" the discovery framing implied.
3. **Why 0 novel at n=140:** with only 140 papers over 12 broad categories, the pairs that share deep
   machinery are precisely those where the machinery is canonical enough to appear in two unrelated
   random papers — i.e. well-known equivalences. And the reduction embedding surfaces the STRONGEST
   mechanism matches, which are the most established ones. Genuinely novel bridges (if the method can
   find them at all) likely live in a much larger/denser corpus and/or in the weaker-match tail, not
   the top-3.

## Implication for the forward path

- **Scaling is now MORE motivated, but reframed:** novelty, if reachable, needs scale — but we should
  EXPECT most bridges at any scale to be rediscoveries and specifically hunt the tail. A **novelty gate**
  (auto-check each surfaced bridge against the literature before presenting, demoting known
  equivalences) becomes a core pipeline stage, not an afterthought.
- **The honest current deliverable** is "a calibrated (0/80 FP), validated unsupervised cross-field
  analogy-REDISCOVERY engine" — do not describe RS-26/27 output as novel discoveries.

## Honest limitations

- Absence of a web hit is NOT proof of novelty (paywalls, books not full-text searched, divergent
  vocabulary). This only *lowers* novelty confidence for the 3 specialist-known ones; the 4
  known_crossfield have positive prior-art evidence (fetched primary sources), so those are firm.
- Agent-mediated web search may miss sources; a domain expert could reclassify. But the direction of
  error here (found MORE prior art than expected) makes the "not novel" conclusion robust — errors
  would only add more prior art, not less.

Artifacts: `prototypes/data/rs29_novelty.json` (full hunter evidence + verdicts per bridge);
workflow `rs29-web-novelty`. No frozen harness (a literature assessment, not a benchmark experiment).
