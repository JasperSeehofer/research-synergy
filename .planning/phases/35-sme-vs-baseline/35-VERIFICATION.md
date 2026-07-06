# Phase 35 — Verification (Plan 35-01, EXP-RS-16)

**Date:** 2026-07-06
**Verdict:** ❌ **SME KILLED. Predictions P2, P3, P4 all FALSIFIED; P1 confirmed.** The
Structure-Mapping generator over blind, role-typed relational schemas does **not** beat — or even
approach — the brute-force LLM baseline: SME roles-ON recall@10 = **0.00** vs baseline **0.60**, and
role-typing *hurts* (roles-ON < roles-OFF < lexical-null). Both pre-registered KILL conditions fire.
The one durable positive: **job zero is done** — the brute-force baseline now has a real number on
the valid testbed (**recall@10 = 0.60, MRR = 0.63**), the bar every future method is judged against.

---

## Executed evidence

Corpus `data/mvp_corpus.json` (C-14): 36 papers = 10 Feynman benchmark endpoints (pairs
01/03/04/05/06 × side_a/b, all present in the testbed) + 26 deterministic distractors
(one per community 0–26, no RNG). Blind role-typed schemas `data/mvp_schemas.json` (C-16): 36
schemas from 36 isolated subagents, each seeing ONLY one `{title, abstract}`; all vocab-valid
(C-15), no dangling relations, domain nouns alpha-renamed (leakage scan clean). Matcher
`prototypes/sme_lite.py` (C-18); toy self-test passes (isomorph→3, unrelated→0, partial 2/3→2).

### Head-to-head (conditional retrieval, side_a query, C-19 / C-20)

| method | recall@1 | recall@5 | recall@10 | MRR | per-pair side_b ranks (of 35) |
|---|---|---|---|---|---|
| **brute-force LLM baseline** (EXP-RS-10) | **0.60** | **0.60** | **0.60** | **0.630** | 12, **1**, **1**, **1**, 15 |
| SME roles-ON (full) | 0.00 | 0.00 | **0.00** | 0.039 | 26, 35, 22, 33, 19 |
| SME roles-OFF (roles stripped) | 0.00 | 0.20 | 0.20 | 0.127 | — |
| lexical-null (abstract TF-IDF cosine) | 0.00 | 0.20 | **0.40** | 0.155 | 31, 2, 17, 7, 25 |

`BENCH_P10` (roles-ON forward recall@10) = **0.000**. (Reverse direction: roles-ON 0.20, roles-OFF
0.20, lexical 0.40 — same ordering.)

### Verdict vs LOCKED predictions (THREAD § Active experiment)

- **P1** (baseline produces a real recall@k — job zero exists): ✅ **CONFIRMED.** Baseline
  recall@10 = 0.60, MRR = 0.63 on the valid testbed. 3/5 pairs rank side_b #1.
- **P2** (SME recall@10 ≥ baseline AND clears TF-IDF floor `BENCH_P10 > 0.15`): ❌ **FALSIFIED.**
  0.00 ≪ 0.60, and 0.00 < 0.15.
- **P3** (roles-ON > roles-OFF — relational structure, not lexical overlap, carries the signal):
  ❌ **FALSIFIED, and inverted.** roles-ON (0.00) < roles-OFF (0.20) < lexical-null (0.40). Adding
  structure *monotonically hurts*.
- **P4** (SME alignment tables match `cross_bridges_ground_truth.json` bridge_names): ❌
  **FALSIFIED.** Alignment tables are **empty** for 3/5 pairs (pair01/03/05 score systematicity 0)
  and trivial 2-entity matches for the other two (pair04: threshold↔threshold UNDERGOES-
  TRANSITION-AT ≈ weak "threshold-phenomenon"; pair06: order-param↔order-param DEPENDS-ON, matches
  neither bridge_name). No systematic bridge recovery.

**GATE:** ADVANCE requires P2 ∧ P3 ∧ P4 → not met. **KILL SME** fires (fails TF-IDF floor OR
roles-ON ≤ roles-OFF) — **both** trigger. Per the pre-registered branch: fall back to slot-frames
(#2) or mechanism-ontology (#4).

## Mechanism — why SME fails (a real over-abstraction collapse, not just metric brittleness)

- **Systematicity is degenerate on this corpus:** across all 36×35 ordered pairs, roles-ON scores
  **51% exactly 0** and 43% exactly 1 (only 6% ≥ 2). The closed role vocabulary + generic glosses
  map essentially every network/statistical-physics paper onto the *same* skeleton
  (order-parameter · threshold · interaction-network · UNDERGOES-TRANSITION-AT), so a benchmark's
  true cross-domain analogue is structurally **indistinguishable from the many network-physics
  distractors**. For pair04/pair06 the true side_b has systematicity 1 but **ties with 18–20
  distractors** → the lexicographic tie-break scatters it to rank 19–22.
- **Relaxing structure helps monotonically** (roles-ON 0.00 → roles-OFF 0.20 → lexical 0.40): if
  relational structure carried the analogy signal, adding it would help; it hurts. The structural
  constraints are removing discriminating information, not adding it.
- **The signal IS in the abstracts** — the full-context LLM baseline recovers it at 0.60. The blind
  role-schema **bottleneck discards** exactly the content that discriminates the true analogue. This
  is the pre-registered #1 risk (schema information-loss) realized, aggravated by the corpus being
  physics-dense (the closed stat-mech role vocab is a prior suspiciously matched to — and here
  defeated by — an all-stat-phys distractor pool).

## Honest caveats / limits of this result

- **Baseline leakage (not corrected here):** Claude knows these famous analogies from pretraining,
  so 0.60 is an *upper*-ish bar. But SME's 0.00 loses to it by a mile, and even a leakage-discounted
  baseline would win; the KILL does not hinge on the baseline's absolute value (P3 and the TF-IDF
  floor are leakage-independent and already fail). Notably the baseline itself misses the two
  "famous" review-paper endpoints (Ising↔opinion #12, Turing↔Zipf #15) — the near-miss distractor
  problem bites the LLM too.
- **Held-out `modern_lbd_pairs.json` robustness: DEFERRED (non-gating).** It needs its own corpus +
  12 blind extractions; the verdict is already triple-locked (P2/P3/P4) and leakage-independent, so
  modern is confirmatory, not decisive. Flag for a follow-up session, not skipped.
- A **softer type-matching** systematicity (coarse relation classes, approximate role match) might
  lift roles-ON off the floor — but that is a NEW pre-registered experiment, not a post-hoc tweak,
  and roles-OFF/lexical already show the structural arms lose *even when constraints are removed*, so
  the deficit is signal-absence at this granularity, not merely a strict metric.

## Recommendation (kill/continue is the human's, per governance)

**KILL the SME-over-blind-schemas arm** (flagship generator #1). It fails the pre-registered gate on
all three stakes and loses decisively to the full-context LLM. **Keep the constructive residue:**
- The **brute-force LLM baseline is now the established working method and bar** (recall@10 = 0.60,
  MRR = 0.63) — this was the un-run "job zero"; it is done.
- Per the pre-registered fallback, next candidate generator = **slot-frames (#2, directed
  problem↔method transfer)** or **mechanism-ontology tagging (#4, MethMeSH)**. Both were designed to
  avoid the exact failure seen here (over-abstraction on a physics-dense pool). The lesson —
  *a blind, lossy, role-typed bottleneck cannot beat full-context LLM reading on this corpus* —
  should shape their eval: measure them against the 0.60 bar, on a **leakage-controlled** set, and
  keep the auditable-artifact deliverable that a plain similarity prompt cannot produce.

## Artifacts
- ✅ `prototypes/build_mvp_corpus.py`, `data/mvp_corpus.json`
- ✅ `prototypes/exp16_extraction_prompt.md`, `data/mvp_schemas.json` (36 blind schemas)
- ✅ `prototypes/sme_lite.py`, `prototypes/sme_lite_toytest.py`, `data/sme_results.json`
- ✅ `data/baseline_results.json`
- ✅ This file; `.planning/phases/35-sme-vs-baseline/35-01-PLAN.md`; CONVENTIONS.md C-14…C-20

## Pack feedback
- **The blind single-paper extraction + symbolic matcher design worked exactly as intended for
  falsification** — because extraction never saw the matching objective, the 0.00 cannot be an
  overfit/leak; it is a clean statement that this representation loses information. Blind-by-
  construction pipelines are the right shape for a *credible* method-negative.
- **Always run the un-abstracted baseline first-class, not as an afterthought.** Here it is the
  single most valuable output of the phase (the bar), and it also *localises* the failure: signal
  present in abstracts (0.60) but absent after the schema bottleneck (0.00) ⇒ the bottleneck, not the
  benchmark, is the problem. A method-negative without a strong baseline can't make that distinction.
- **Near-miss distractors matter.** Sampling one distractor per community produced a physics-dense
  pool that shares mechanism skeletons with the benchmark — which is what made the over-abstraction
  collapse visible (and also dented the LLM baseline). Keep this distractor discipline for #2/#4.
