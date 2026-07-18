# EXP-RS-25 — Phase 44 Verification: raw ∪ reduction → LLM-rerank Cascade

- **Status**: CONCLUDED — **WORKS**. The two-retriever-union → LLM-rerank cascade lifts mixed-corpus
  recall@10 from 0.662 (raw-alone) to **0.775** (+11.3 pts, essentially the union ceiling), with strong
  precision (MRR 0.74). The scalable LBD retriever the chapter set out to build.
- **Date**: 2026-07-18
- **Orbiter**: Claude re-rank (overseer/primary) + Mistral re-rank (executor cross-family) — a clean
  W-SYN result (below).

## Result (mixed 80-pair cross-archive sample)

| retriever | recall@10 | recall@5 | MRR |
|---|---|---|---|
| raw-embedding alone | 0.662 | — | — |
| reduction alone | 0.525 | — | — |
| union membership ceiling (side_b ∈ raw-top15 ∪ reduction-top15) | 0.800 | — | — |
| **CASCADE — Claude re-rank of the union** | **0.775** | **0.750** | **0.740** |
| cascade — Mistral re-rank of the union | 0.637 | 0.487 | 0.360 |

- **The cascade works.** Union pools (mean 25 candidates, K=15 each) contain side_b for 80% of pairs
  (ceiling); the Claude LLM re-rank places side_b in the top-10 for **62/80 = 0.775** — recovering all
  but 2 of the in-union targets, and lifting MRR to 0.74 (raw-embedding cannot even rank the deep tail).
  This is +11 pts over raw-alone, matching the earlier descriptive union-recall estimate (0.775) — the
  LLM re-rank realizes the union's full recovery while adding precision.
- **Orbiter / W-SYN finding: the re-rank MUST be Claude.** Mistral re-ranking the *same* union pools
  scores **0.637 — BELOW raw-alone (0.662)** — MRR 0.36. Cross-field analogy re-ranking is a synthesis
  task; the Mistral executor degrades it (textbook W-SYN, consistent with RS-20's over-pruning and
  RS-24's granularity finding). Route the precision/re-rank stage to Claude; Mistral is not usable here.

## Architecture (the deliverable)

```
  query paper (side_a)
      │
      ├── raw-abstract bge  → top-15   ┐
      │                                 ├── UNION (dedup, ~25) ── Claude LLM re-rank ── top-k
      └── reduction bge     → top-15   ┘        (frozen rs22_retrieval_prompt.md)
```

- **Cheap recall stage** = two embedding retrievers, both O(N) to index (raw abstracts + O(N) LLM
  mechanism reductions), free cosine at query time. The reduction retriever is the RS-23/24 validated
  deep-analogy specialist; the raw retriever covers the topical bulk; their UNION recovers both (0.66
  raw + the deep tail → 0.80 ceiling). Fusion (RRF) was dilutive (RS-23, 0.64) — UNION is the right
  combiner because it feeds *both* candidate sets to the precision stage rather than averaging ranks.
- **Precision stage** = the LLM re-ranks the small (~25-candidate) union — the known ceiling-level
  brute-force ranker, now applied to O(1) candidates per query instead of the whole corpus. O(N)
  cheap retrieval + O(#queries) small LLM re-ranks = a scalable discovery pipeline (vs the O(N²)
  all-pairs LLM baseline).

## Design / integrity

- Mixed 80-pair cross-archive sample (RS-23/24). Union built deterministically (raw-top15 then
  reduction-top15, dedup). Re-rank = the FROZEN `rs22_retrieval_prompt.md` (SHA on file); Claude via a
  Workflow fan-out (index-based dispatch), Mistral via direct API (≤4 workers, 429-gap-filled). Rank
  recovered with deterministic repair. Reproducible modulo the pinned models.

## Limitations & forward

- Cascade ceiling = union membership (0.800 at K=15); raising K_each trades recall ceiling for a larger
  (costlier) re-rank pool — a tunable operating point, not evaluated exhaustively.
- Evaluated as benchmark *retrieval* (recall of a known partner), not open-ended *discovery*. The
  natural next step is a genuine discovery run: apply the cascade to a query with no known partner, take
  the top union candidates, and have the LLM name + audit the shared mechanism (the auditable transfer
  card from EXP-RS-19/20) — turning the retriever into a discovery+explanation pipeline.
- Scale/parameters: K_each, the encoder choice, and the re-rank pool size are open tuning knobs; the
  larger validated-deep set (EXP-RS-24 N=160) supports a fairer large-n re-evaluation.

Artifacts: `prototypes/rs25_cascade.py`, `data/rs25_results.json`, `data/rs25_out/retrieval/`,
`data/rs25_out_mistral/retrieval/`.
