# Phase 36 — EXP-RS-17 verification: MethMeSH (mechanism-ontology #4) vs the brute-force baseline

**Verdict (2026-07-07): KILL-by-construction (exact-match MethMeSH).** The pre-registered cheap
tagging-recall gate (C-26) FIRED: the frozen field-agnostic mechanism vocabulary cannot represent
the Feynman cross-domain bridges under exact-archetype-ID matching. P2 FALSIFIED
(Feynman tagging-recall = 1/5 < 3/5); the pre-registered KILL condition is met **before** any
distractor tagging or the full retrieval eval was paid for — exactly what the cheap gate is for.

## What was pre-registered (LOCKED before the run — THREAD.md, CONVENTIONS C-22..C-26)

Generator: freeze a ~50–150 archetype field-agnostic mechanism vocabulary (blind, external-seeded,
SHA-256-hashed before tagging = leakage control); blind per-paper tagging (1–5 archetypes + evidence
snippet); score(i,j) = IDF-weighted archetype-signature cosine − λ·lexical cosine (λ=1.0). Two
corpora: Feynman MVP (reuse `mvp_corpus.json`, C-14) + a NEW modern held-out MVP (C-24). Eval =
C-19 conditional retrieval vs the 0.60 Feynman bar + a new modern brute-force bar. **Cheap gate
run FIRST (C-26):** tagging-recall (both sides of a pair share ≥1 archetype); Feynman ≥3/5 AND
modern ≥4/6, else KILL by construction.

LOCKED predictions: **P1** modern baseline runs ∧ modern ≤ 0.60. **P2** tagging-recall gate (above).
**P3** MethMeSH recall@10 ≥ the modern baseline ∧ > 0.15 floor on both. **P4** IDF > uniform ∧ λ=1 >
λ=0. **P5** recovered pairs' shared archetype matches ground-truth `bridge_names`. **GATE:** ADVANCE
iff P2∧P3∧P4∧P5 · PIVOT iff ties modern with good artifacts · **KILL #4 iff P2 fails** OR recall<floor
on both OR P4 null → fall back to slot-frames (#2).

## What was executed

1. **Frozen vocab (C-22).** A blind subagent (no benchmark/ground-truth access) built
   `data/mechanism_vocab.json` = **125** field-agnostic archetypes from external taxonomies (TRIZ +
   canonical stat-mech / dynamical-systems / network-science / stochastic-process / info-theory /
   optimization / random-matrix families; per-archetype provenance). Frozen: SHA-256
   `aa6584dcbd992bcafc5ceff87961f0271e66c6403ca376e8a1a6ff95dadd1a6a`, committed BEFORE tagging
   (commit `0f2596d`). Validated: 125 unique, correct schema, field-agnostic, good anti-flood
   granularity (phase transitions split into continuous/first-order/spinodal/percolation variants).
2. **Corpora.** Feynman MVP reused (36 papers, C-14). Modern held-out built (`build_modern_corpus.py`,
   C-24, commit `3e5f4d7`): 36 papers = 12 endpoints (all 6 evaluable `modern_lbd_pairs`) + 24
   deterministic post-2018 distractors over 9 primary categories; all abstracts ≥200 chars.
3. **Blind tagging (C-23).** 22 benchmark endpoints (10 Feynman + 12 modern) tagged by 22 blind
   per-paper subagents — each saw ONLY `{title, abstract}` + the frozen vocab (commit `1666e8f`).
   Distractors NOT tagged (the cheap gate short-circuited, as designed).
4. **Tagging-recall gate (C-26)** computed on the 22 endpoints.

## Result — the gate (decisive)

| corpus | tagging-recall | gate | pairs that share an archetype | verdict |
|---|---|---|---|---|
| **Feynman** | **1/5** | ≥3/5 | pair03 only (`arch-068 hub-dominated-spreading`) | **FAIL** |
| **Modern** | **4/6** | ≥4/6 | m01 (`arch-076`), m02 (`arch-082`), m06 (`arch-108`), m08 (`arch-065`) | PASS |

**P2 = (Feynman ≥3/5 AND modern ≥4/6) → FALSIFIED** (Feynman side fails). → **KILL-by-construction.**

Per-pair archetype sets (Feynman), showing the failure mode:
- **pair04 percolation↔epidemics:** a=`{giant-connected-component, birth-death-branching}` ·
  b=`{compartmental-flow-model, simple-contagion-spread}` → *neighboring* mechanisms in the SAME
  percolation/spreading family, different exact ids ⇒ overlap 0.
- **pair01 Ising↔opinion:** a=`{mean-field-order-parameter, continuous-order-parameter-transition,
  hub-dominated-spreading, scale-free-degree-distribution}` · b=`{simple-contagion-spread,
  imitation-consensus-dynamics}` ⇒ overlap 0.
- **pair06 Turing↔spatial-economy:** a=`{turing-instability, activator-inhibitor-loop, …}` (correct!)
  · b=`{mass-action-bilinear-coupling, scale-invariance-power-law}` — the far (economics) side did
  not recover the reaction-diffusion framing ⇒ overlap 0.

## Verdict vs each locked prediction

- **P2 — FALSIFIED** (Feynman 1/5). KILL condition met.
- **P1, P3, P4 — NOT EVALUATED** (the cheap gate short-circuited the full retrieval eval and the
  modern baseline, by design — the whole point of running the gate first). Derivable bound: with
  4/5 Feynman pairs sharing NO archetype, their signature cosine is 0, so they can be retrieved only
  by the −λ·lexical term ⇒ Feynman recall@10 ≤ ~0.20, far below the 0.60 bar and near/under the 0.15
  floor. So P3 would almost certainly also fail on Feynman.
- **P5 — PARTIAL/UNEVALUATED:** only 1 Feynman + 4 modern pairs have a shared archetype to audit;
  the shared archetypes that DO exist are on-target (pair03→`hub-dominated-spreading`/contagion;
  m08→`simple-contagion-spread`), consistent with `bridge_names`, but coverage is too thin to score.

## Mechanism (clean, like the EXP-RS-16 negative)

Exact-archetype-ID overlap is **brittle to granularity**: a specific 125-archetype vocab tags two
genuinely-analogous papers with *neighboring but non-identical* archetypes, so the signature cosine
(exact-id) is 0 and the pair is invisible. This is the **opposite pole** from EXP-RS-16's
over-abstraction collapse (too coarse → everything matches). MethMeSH's *representation* is not
empty — the archetypes are individually correct (e.g. `turing-instability` was tagged on the Turing
paper) — but the *matching rule* discards the adjacency. The brainstorm predicted exactly this
"fixed-vocabulary granularity bind" for #4.

## The modern wrinkle

Modern held-out PASSED its gate (4/6) under the SAME exact-match rule. The Feynman set is older, more
famous, and both sides are stat-phys papers whose surface mechanisms diverge under fine archetypes;
the modern pairs (dense-Hopfield↔spin-glass, Kuramoto↔brain, entropy-production↔diffusion-models,
SIR↔info-diffusion) more often share an *identical* archetype. This says the failure is
benchmark-and-granularity-specific, not a flat "no signal." It is NOT enough on its own to overturn
the pre-registered KILL (P2 requires BOTH corpora), but it is a documented positive.

## Constructive residue

- 22 blind endpoint tags (Feynman + modern) — **reusable** for any similarity-aware rescore.
- A sharp, mechanistic diagnosis: the archetype *representation* has signal; exact-ID *matching* is
  the failure. Motivates a similarity-aware signature (archetype family / embedding) as a clean
  next hypothesis.
- The modern held-out corpus (`modern_mvp_corpus.json`) + its build method — the leakage-controlled
  testbed, now ready for the next generator.
- Reusable harness: `methmesh_score.py`, `collect_tags.py`, `build_modern_corpus.py`, tagging prompt.

## Forward path — HUMAN's go/kill/pivot call (options presented, not auto-executed)

The pre-registered verdict is **KILL exact-match MethMeSH #4**. Per the locked gate, the fallback is
**#2 slot-frames** (already human-approved). Two alternatives the diagnosis motivates:
(A) accept KILL → build #2 slot-frames; (B) complete the modern-only eval (tag 24 modern distractors
+ run the modern baseline) to quantify the modern PIVOT before deciding; (C) pre-register a fresh
**EXP-RS-18** that keeps the (working) blind archetype tagging but replaces exact-ID overlap with
**archetype-family / similarity scoring** — directly targets the diagnosed brittleness and reuses the
22 tags. Relaxing the scoring *within* EXP-RS-17 is explicitly NOT done (that would be post-hoc
goalpost-moving); it must be a new pre-registration.
