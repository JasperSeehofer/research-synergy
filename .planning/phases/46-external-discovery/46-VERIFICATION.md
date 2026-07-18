# EXP-RS-27 — Phase 46 Verification: External-Corpus Discovery

- **Status**: CONCLUDED — **WORKS on a genuinely un-mined corpus.** The finder generalizes to fresh
  arXiv papers (fetched by category, NOT via bridge papers) and surfaces real, non-obvious, surface-
  invisible cross-field mechanism bridges. End-to-end: 228 candidates → 40 carded → 6 shared-method →
  **5 survive blind adjudication as genuine shared machinery**, 3 of them rated *surprising*.
- **Date**: 2026-07-18

## Setup

Fresh corpus: **140 papers**, 12 diverse categories × ~12 (cond-mat.stat-mech, hep-th, quant-ph, math.AP,
q-bio.PE, q-fin.ST, cs.LG, nlin.CD, astro-ph.CO, physics.soc-ph, math-ph, stat.ME), most-recent by
submission date — **not** selected via bridge papers, so any bridge is genuinely un-mined. Reduce every
paper (frozen `rs22_probe_mechanism.md`, O(N)); a hidden bridge = reduction-embedding top-3 ∧ cross-
archive ∧ lexical cos < 0.06. Open-book card (Claude) → blind skeptical adjudication (independent).

## Result — hit-rate 0.15 (6/40 carded); 5/6 genuine after adjudication

All 5 confirmed have lexical cos ≈ 0.003–0.021 (near-zero shared vocabulary — surface retrieval cannot
find them):

| # | field A ↔ field B | shared machinery | novelty |
|---|---|---|---|
| **E5** | astro-ph.CO (galaxy bias / CMB-lensing HODs) ↔ stat.ME (Bayesian retail-store survival, Tokyo) | **log-Gaussian Cox process** — point pattern intensity modulated by a latent Gaussian field; bias ↔ loading coupling | **surprising ✓** |
| **E3** | cond-mat.soft (rough-surface contact) ↔ q-fin.PR (jump-diffusion option pricing) | same **Fokker–Planck / heat-diffusion PDE** (Persson magnification ↔ option time) | **surprising ✓** |
| **E2** | hep-th (observers, local measurements, topology) ↔ quant-ph (quantum TDA for financial stress) | **simplicial homology / Betti numbers** of a complex built from discrete observations | **surprising ✓** |
| E0 | nlin.AO (stochastic ultimatum game) ↔ q-bio.PE (predator-dependent replicator) | replicator / eco-evolutionary dynamics coupled to a resource variable | specialist ✓ |
| E4 | cond-mat.mes-hall (magnetotransport, Corbino) ↔ hep-ph (bubble-wall friction, cosmological PT) | the **linearized Boltzmann transport equation** (Liouville streaming + collision integral) | textbook ✓ |

Rejected (honest): E1 (thermodynamic voting ↔ entropic option pricing) — over-abstracts to the generic
Jaynes MaxEnt *meta*-principle, not a shared governing equation (one side physical Rayleigh–Jeans
thermalization, the other epistemic inference).

## Reading

On a **fresh external corpus**, 5/6 hold as genuine same-equation cross-field bridges — the pipeline finds
real structural analogies, not keyword collisions (there are no shared keywords: lexical ≈ 0). The most
compelling are the *surprising* ones few would connect: **cosmological clustering ↔ retail-store survival
(both log-Gaussian Cox processes)**, **rough-surface contact ↔ option pricing (both Fokker–Planck)**,
**spacetime topology ↔ quantum finance-TDA (both simplicial homology)**. Hit-rate 0.15 ≈ the RS-26 mined
0.20 → the method is corpus-robust.

## Novelty check

The adjudicator's textbook/specialist/surprising rating is a model-knowledge novelty proxy (2 textbook =
method-validators; 3 surprising = non-obvious candidates). A full literature/web verification of the
*surprising* ones (are they published anywhere?) is the honest next step and is deferred — these are
proposed candidate bridges for a domain expert, not certified-novel discoveries.

## Limitations & forward

- 140-paper sample, self-contained pool; scaling to thousands (with the cascade re-rank, not just
  reduction-embed) would surface more. The web-novelty verification + a domain-expert pass are the path
  to a publishable "novel cross-field bridge." The pipeline (fetch → reduce → hidden-bridge extract →
  open-book card → blind adjudication) is the reusable deliverable — an unsupervised, corpus-agnostic
  cross-field bridge finder.

Artifacts: `prototypes/rs27_external.py`, `data/rs27_{corpus,candidates,discoveries,adjudication}.json`.
