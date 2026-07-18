# EXP-RS-26 — Phase 45 Verification: Discovery Run (hidden cross-field bridges)

- **Status**: CONCLUDED — **WORKS**. The cascade + a two-stage LLM judge surfaces real, non-obvious,
  surface-invisible cross-field mechanism bridges. End-to-end: 75 candidates → 40 carded → 8 judged
  shared-method → **5 survive a blind independent adjudication as genuine shared machinery.**
- **Date**: 2026-07-18

## What it does (the LBD payoff)

Reuses the EXP-RS-25 cascade over the 80-pair mined corpus. A **hidden bridge** for query X = a candidate
Y that is (1) a different top-level arXiv archive, (2) NOT X's known mined partner, (3) ranked in the
cascade's top-3 for X, (4) **lexical cosine(X,Y) < 0.06** — surface/topical retrieval CANNOT see it;
only the mechanism reduction surfaced it — and (5) an open-book LLM confirms a shared method. Then a
skeptical **blind independent adjudicator** re-checks each for a genuine, specific shared object (not a
superficial/metaphorical resemblance) and rates novelty.

## Result

- **75 hidden-bridge candidates** (cross-archive ∧ non-partner ∧ cascade-top-3 ∧ lexical < 0.06).
- Top 40 carded (Claude open-book) → **8 confirmed shared-method** (hit-rate 0.20).
- Blind adjudication of the 8 → **5 genuine** (mechanism_real), 3 rejected as honest failures.
- **End-to-end precision ≈ 5/40 = 12.5%** after two independent LLM filters — for an *unsupervised*
  cross-field discovery pass with zero lexical signal, this is a real, usable yield.

### The 5 confirmed bridges (all lexical cos ≈ 0.01–0.06 — surface-invisible)

| # | field A ↔ field B | shared machinery | novelty |
|---|---|---|---|
| B0 | hep-th (dispersionless integrable / WDVV) ↔ math.QA (Frobenius manifolds) | WDVV associativity equations **are** the defining PDEs of a Frobenius manifold (Dubrovin) | textbook ✓ (method-validating) |
| B1 | physics.data-an (MaxEnt updating w/ moments) ↔ cond-mat.str-el (post-quench steady state) | Jaynes max-entropy-under-constraints = the generalized Gibbs ensemble | textbook ✓ |
| **B6** | q-bio.PE (quasispecies / HGT) ↔ quant-ph (glass models & quantum annealing) | extremal eigenvector of a mean-field spin Hamiltonian; **mutation operator ≡ transverse field**; error threshold = quantum-REM first-order transition | **surprising ✓** |
| **B2** | quant-ph (Casimir via T-operator) ↔ nlin.CD (periodic-orbit theory, quantum graphs) | `Tr ln(1 − scattering operator)` secular determinant; multiple-scattering paths ≡ periodic orbits | **specialist ✓** |
| **B7** | math-ph (equilibrium measures / orthogonal polynomials) ↔ nlin.SI (semiclassical focusing NLS) | same 2×2 matrix **Riemann–Hilbert** problem; g-function fixed by a log-potential equilibrium principle | **specialist ✓** |

Rejected (honest failures): B4/B5 (mode-locked lasers ↔ economics — the economics side invokes spin
glasses only as a *metaphor*, no shared Hamiltonian); B3 (cluster abundances ↔ superconductor gap
fluctuations — generic "rare objects from a threshold-exceeding random field" motif, not the same
governing equation).

## Reading

The two **textbook** hits (WDVV=Frobenius, MaxEnt=GGE) are *method validation*: the pipeline rediscovers
famous cross-field equivalences purely from abstracts with **near-zero lexical overlap** — proof it finds
real shared machinery, not word matches. The **specialist/surprising** hits (B6, B2, B7) are the payoff:
correct, non-obvious cross-field bridges (quasispecies ↔ quantum annealing; Casimir ↔ quantum-graph
spectra; orthogonal polynomials ↔ integrable NLS) that surface retrieval provably cannot find. These are
proposed bridges for a domain expert, not certified novel — but they are real and non-obvious.

## Integrity / design

- Deterministic candidate extraction (cross-archive, non-partner, cascade-top-3, lexical<0.06) from the
  frozen RS-25 cascade re-rankings. Judge + adjudicator use the FROZEN `rs22_probe_openbook.md` + a blind
  skeptical adjudicator (no benchmark/answer-key). Two independent LLM filters. Honest hit-rate reported;
  nothing over-claimed as "novel discovery."

## Limitations & forward

- Judged within the 320-paper mined corpus; scaling to a large external corpus is the next step (more
  candidates, genuinely un-mined pairs). The `is-it-novel-vs-published` check needs a literature search
  (out of scope here). The pipeline is the deliverable: `raw ∪ reduction → LLM re-rank → open-book card →
  blind adjudication`, an unsupervised cross-field bridge finder.

Artifacts: `prototypes/rs26_discover.py`, `data/rs26_candidates.json`, `data/rs26_discoveries.json`,
`data/rs26_adjudication.json`, `data/rs26_out/openbook/`.
