# EXP-RS-23 — Phase 42 Verification: Mechanism-Reduction Substrate

- **Status**: CONCLUDED — **CONDITIONAL / SCOPED POSITIVE**. Universal-retriever claim REFUTED; a
  validated **specialist role** for deep cross-vocabulary analogies confirmed on the curated testbed +
  a descriptive hard-tail analysis.
- **Date**: 2026-07-18
- **Question (human)**: does an LLM "Feynman reduction" of each paper (structure) beat raw full-text
  comparison, or is no assisting structure needed?

## Verdict

**It depends on the depth of the analogy — and the answer is genuinely informative:**

- On **curated DEEP cross-vocabulary analogies** (Feynman, the LBD prize — bridges with almost no shared
  surface vocabulary), the field-neutral mechanism reduction is the **first compressed substrate in the
  chapter to beat the memory-free null**: forward recall@10 = **0.60** vs lexical **0.40** vs raw-abstract
  embedding **0.20** (RS-21). P1 ∧ P2 → per-corpus **ADVANCE**.
- On the **broad 420-pair mined benchmark** (cross-archive but largely topical + method-sharing
  unvalidated), the reduction is **net-negative**: 0.53 vs raw-embedding **0.66** vs lexical **0.59**.
  P1 ∧ P2 both FALSE → confirmation **NEGATIVE**. The universal-substrate claim is refuted.

The reduction is therefore **not a universal retriever** — it is a **specialist for the deep,
cross-vocabulary tail** that surface methods cannot see.

## Evidence

| testbed | raw-embedding R@10 | reduction R@10 | lexical R@10 |
|---|---|---|---|
| Feynman (curated deep, n=5) | 0.20 | **0.60** | 0.40 |
| Mined all (cross-archive, n=80) | **0.66** | 0.53 | 0.59 |
| Mined EASY: raw finds side_b ≤10 (n=53) | 1.00 | 0.62 | — |
| Mined HARD: raw fails, side_b >10 (n=27) | 0.00 | **0.33** | — |

- **Reduction rescues the deep tail** (descriptive, post-hoc split): on the 27 mined pairs where
  raw-embedding fails, the reduction pulls **9 into the top-10** (raw #24→red #2, #53→#5, #19→#1,
  #16→#1, #11→#1, …) — the same field-dominance rescue as Feynman pair04 (raw #20 → reduction #2),
  reproduced at n=27 on cross-archive pairs.
- **Reduction hurts the topical majority**: on the 53 easy pairs it drops recall 1.00→0.62 — over-
  abstraction discards the surface signal those pairs rely on.
- **Reduction-win ⟂ raw-win** (anti-correlated). **Fusion (RRF of raw+reduction) does not help** —
  Feynman 0.40 (< reduction 0.60), mined 0.64 (≈ raw 0.66): averaging dilutes the winner on both sides.
- `reduction_bge_full` (core_mechanism + brief_reason) = 0.40 < 0.60: the **pure field-neutral
  core_mechanism** is the right text; re-adding reasoning prose re-introduces field vocabulary.

## Mechanism

RS-21 killed dense embeddings by **field/topical dominance** (field ≫ mechanism in whole-abstract
vectors). Distilling each paper to its field-neutral core mechanism (e.g. "probability-generating-function
branching process → percolation threshold → giant component") strips the field vocabulary, so the
transferable machinery becomes the dominant signal. This **helps exactly when surface vocabulary is the
obstacle** (deep cross-vocabulary analogies) and **hurts when surface vocabulary is the signal**
(topical pairs). It is the same over-abstraction trade-off that killed EXP-RS-16 (role-schemas) and
EXP-RS-17 (mechanism-tags) — but here it is quantified as a clean, actionable **selectivity** rather
than a flat loss: the reduction is precisely a retriever for the non-obvious bridges that are the point
of LBD.

## Design / integrity

- Apples-to-apples with RS-21: identical corpus/pairs/pool/metric/encoder (bge symmetric, C-19 forward);
  the ONLY change is the embedded text (raw abstract → the `core_mechanism` reduction).
- The reduction = the FROZEN blind mechanism probe `rs22_probe_mechanism.md` (SHA `72de2252…`), run per
  paper on `{title, abstract}` ONLY (fresh session, no partner, no benchmark → no pair/answer-key
  leakage). O(N): one LLM call per paper (36 Feynman + 160 mined; 196 total, 1 spurious content-filter
  retry). bge deterministic (seed 0, CPU). Predictions frozen in `42-PREREG.md` before any reduction.
- Descriptive analyses (hard-tail split, fusion) are post-hoc/exploratory — they inform the next
  experiment and are NOT promoted to the pre-registered headline.

## Limitations

- Feynman n=5 (tiny); the mined hard-tail rescue is n=27 (9 rescues) — modest but consistent with
  Feynman. The strong claim ("reduction wins on deep cross-vocabulary analogies") needs a LARGER set of
  **validated** deep analogies — which the 420-pair mine LACKS (its pairs are mostly topical and its
  method-sharing was never validated → it is the wrong testbed for deep-analogy retrieval).

## Forward (highest-value next step)

Not a universal substrate → do NOT replace raw retrieval with it. Instead a **router / two-retriever
cascade** motivated by the anti-correlation: cheap surface retrieval (raw embedding) for the bulk +
the reduction-embedding as a **targeted second retriever for the low-confidence / hard tail**, then LLM
re-rank the union. Precondition for a fair large-n test: build the **validated deep-analogy subset** of
the 420 mine (pairs where surface retrieval fails AND an LLM confirms a genuine shared mechanism), then
re-run reduction vs raw there. That directly tests the specialist claim at n≫5.

Artifacts: `prototypes/rs23_reduce.py`, `data/rs23_results.json`, `data/rs23_results_mined.json`,
`data/rs23_out{,_mined}/mechanism/`.
