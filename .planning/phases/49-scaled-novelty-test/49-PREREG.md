# EXP-RS-30 — Phase 49 PRE-REGISTRATION: Scaled Novelty Test (does the finder surface ANY novel bridge?)

- **Status**: PRE-REGISTERED — LOCKED 2026-07-18 (before any RS-30 LLM call). Blind constants +
  KILL/PASS frozen below; harness SHA recorded at the end after the freeze.
- **Date**: 2026-07-18

## Motivation

RS-28 proved the finder's bridges are REAL (0/80 FP). RS-29 proved that at n=140, all 7 genuine
bridges are KNOWN/specialist — the strongest reduction matches are the canonical equivalences
(machinery famous enough to co-occur in two random papers). RS-29's mechanism implies **novelty, if
reachable, lives in the WEAK-MATCH TAIL and at larger scale**, not the top-3. This experiment tests
that directly and decisively.

## Hypothesis

**H-RS-30:** with (a) a larger un-mined corpus and (b) explicit tail-hunting (reduction-embed ranks
beyond the top-3), the finder surfaces at least one bridge that is REAL (survives card+adjudicator)
AND plausibly-NOVEL (survives an adversarial two-hunter literature novelty-gate). If instead every
genuine bridge — top-3 or tail — is KNOWN/specialist even at scale, the finder is terminally a
cross-field REDISCOVERY engine, not a novelty engine.

## Design

- **Corpus (scale):** fetch **N = 720** fresh arXiv papers (18 diverse categories × 40, most-recent
  by submission date, NOT via bridge papers → genuinely un-mined). ~5× RS-27's 140. Categories chosen
  for cross-field breadth (physics / math / CS / q-bio / q-fin / stat / econ / nonlinear / gr-qc).
- **Reduce** every paper (frozen `rs22_probe_mechanism.md`, SHA `72de2252…`, O(N), blind Opus
  fan-out) — same reduction as RS-23/24/27.
- **Candidate extraction (the key change — TWO strata):** reduction-embed (bge) cosine; for each
  query keep cross-archive ∧ lexical cos < 0.06 candidates at:
  - **TOP stratum:** reduction ranks 1–3 (canonical; POSITIVE CONTROL that the pipeline still works
    at scale — expected to yield mostly known/specialist, like RS-26/27).
  - **TAIL stratum:** reduction ranks 4–12 (the NOVELTY HUNT — less-canonical mechanism matches).
  Dedup pairs across strata (a pair is labeled by its best/lowest rank).
- **Card + adjudicate** (frozen `rs22_probe_openbook.md` `2c83eef3…` → reconstructed skeptical
  adjudicator `rs28_adjudicate.md` `d124142e…`), byte-identical to RS-26/27/28. Card budget: up to
  **M = 100** cards, **stratified** — ≥40 from the tail stratum (so the novelty hunt is actually
  exercised, not crowded out by top-rank canonical pairs).
- **Novelty-gate (RS-29 pattern, HARDENED):** every adjudicated-genuine bridge → **two independent
  adversarial prior-art hunters** (WebSearch, told to FIND prior art, ≥4–6 query framings) → skeptical
  classifier. A bridge is **robust-novel** iff the classifier returns `novel_looking` AND BOTH hunters
  independently report `cross_field_identity_status = not_found`. (Single-hunter novel_looking =
  candidate-novel, reported but not counted for the PASS.)

## Metrics

- Per stratum: #candidates, #carded, #card-confirmed, #adjudicated-genuine.
- Novelty breakdown of genuine bridges: {known_crossfield, specialist_known, candidate_novel,
  robust_novel}.
- Positive-control check: TOP stratum yields ≥1 adjudicated-genuine bridge (pipeline works at scale).

## Pre-registered decision (BLIND — frozen before running)

- **INVALID (pipeline broke at scale):** TOP stratum yields 0 adjudicated-genuine bridges. (Would
  mean the scale-up itself failed; re-examine, not a real KILL.)
- **PASS — novelty engine viable → scale to true thousands + expert validation:** ≥ 1 **robust-novel**
  genuine bridge (novel_looking ∧ both hunters not_found). The finder can surface a plausibly-unpublished
  cross-field bridge → worth scaling further and sending to a domain expert.
- **KILL — terminal: rediscovery engine, novelty not reachable this way → bank + write up:** 0
  robust-novel across BOTH strata (every genuine bridge is known/specialist, or novel_looking only
  under a single hunter). Combined with RS-29's 0/7 at n=140, a 0 here at 5× scale WITH tail-hunting
  is a strong cumulative terminal signal.
- **WEAK/borderline:** ≥1 candidate_novel (single-hunter not_found) but 0 robust_novel → report as a
  lead for a targeted deeper check; lean KILL unless the candidate is compelling.

## My prediction (pre-registered, honest)

**Lean KILL, with real uncertainty.** RS-29's mechanism (strongest matches = canonical) predicts even
the tail will surface mostly known/specialist machinery. BUT tail-hunting is the one untested lever, so
≥1 robust-novel is a live possibility (p≈0.3 subjective) — that is exactly why the experiment is worth
running rather than assuming. **Named risk:** the novelty-gate's "not_found" is a weak positive (absence
≠ novelty); the two-hunter requirement mitigates single-search misses, but a robust-novel survivor is a
CANDIDATE for expert review, never a certified discovery — reported as such.

## Scope / honesty guards

- "robust-novel" = plausibly-unpublished per adversarial search, NOT certified novel. No over-claim.
- Tail candidates are weaker matches → more spurious; the card+adjudicator+RS-28-calibration carry the
  reality filter, the novelty-gate carries the novelty filter. Both are frozen/validated.
- Report the FULL known/specialist/candidate/robust breakdown regardless of verdict — the distribution
  is the result, not just the binary.

## Frozen artifacts (SHA-256)

Locked 2026-07-18 before any RS-30 LLM call:

| artifact | SHA-256 | role |
|---|---|---|
| `prototypes/rs30_scale.py` | `e8c45b3fe775149b084ec2449c147b304f33129f5d93cf00a41fe4b9802cfd33` | harness (N/cats/strata/budget/verdict) |
| `49-PREREG.md` (pre-freeze body) | `50ae42626a4d881bb153c68431d811fc21e7991438a243ff660854fb197f202a` | predictions + KILL/PASS |
| `prototypes/rs22_probe_mechanism.md` | `72de22528b26480b120794f8050871930b54fff81e7c53b6f0e4f297e8509440` | reduction (frozen, unchanged) |
| `prototypes/rs22_probe_openbook.md` | `2c83eef3117d69db982a89e732ca12f635eb86af7c550cdb5d03b2679ad1e6b0` | card (frozen) |
| `prototypes/rs28_adjudicate.md` | `d124142e6ab56c46457fb4d9598a2ec95ddd2127e77311bf20f42ef96b72b389` | adjudicator (frozen from RS-28) |

**Deviation (transparent, non-biasing):** fetch yielded **N = 684** papers, not the target 720 — 5 of
the 18 categories had fewer recent unique papers with ≥200-char abstracts (math-ph 27, gr-qc 24, math.PR
36, cs.IT 37, others 40). Still ~4.9× RS-27's 140; does not affect any decision constant. Corpus:
`data/rs30_corpus.json`.
