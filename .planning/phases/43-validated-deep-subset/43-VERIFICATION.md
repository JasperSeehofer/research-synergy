# EXP-RS-24 — Phase 43 Verification: Orbiter-Validated Deep-Analogy Subset

- **Status**: CONCLUDED — **CONFIRM** (upgraded from WEAK by the same-session expansion to N=160, below).
  The reduction is a validated deep-analogy specialist: validated-deep reduction R@10 = 0.75 vs raw 0.00
  at **n=12 ≥ 8**. Plus a strong descriptive cascade-value result and a clean orbiter audit.

> **EXPANSION ADDENDUM (2026-07-18, N=160 — upgrades WEAK→CONFIRM).** Doubled the self-contained sample
> to 160 pairs (320 papers, all reduced; Mistral open-book validity on all 160 + Claude on the 61
> surface-hard). **VALIDATED-DEEP (n=12): reduction R@10 = 0.75 vs raw 0.00 vs lexical 0.17;
> VALIDATED-EASY (n=47): raw 1.00 vs reduction 0.64.** P1 ∧ P2 hold with n=12 ≥ 8 → **CONFIRM** per the
> frozen gate. Orbiter audit at the 71-pair overlap: **κ = 0.77** (agree 0.90); Mistral now slightly
> LENIENT (over-prune 2 / under-prune 5) — at n=80 it was κ=0.89 with over-prune 1. So Mistral stays a
> usable coarse-validity executor (κ 0.77–0.89) with a mild, granularity-dependent bias direction. Data:
> `data/rs24_results.json` (n=160). The original n=80 WEAK run is preserved below as the record.

---

## Original run (n=80, WEAK) — preserved record
- **Date**: 2026-07-18
- **Orbiter**: first faithful executor/overseer loop on a coarse-validity task — Mistral executor (80
  pairs) + Claude overseer (27 deep pairs + audit). `pi-migration-ledger` row filed.

## Verdict (frozen 43-PREREG gate)

**WEAK** — P1 ∧ P2 both hold, but n(validated-deep) = 5 < 8.

| split | n | reduction R@10 | raw-bge R@10 | lexical R@10 |
|---|---|---|---|---|
| **VALIDATED-DEEP** (surface-hard ∧ shared-method) | 5 | **0.80** | **0.00** | 0.20 |
| **VALIDATED-EASY** (surface-easy ∧ shared-method) | 29 | 0.79 | **1.00** | 0.90 |

- **P1 (reduction > raw on validated-deep): TRUE, decisively** — 0.80 vs 0.00 (raw fails *by
  construction* on the surface-hard tail; the reduction recovers 4/5). Corroborates Feynman (0.60 vs
  0.20) on cross-archive, method-validated pairs.
- **P2 (raw ≥ reduction on validated-easy): TRUE** — 1.00 vs 0.79. The anti-correlation is airtight on
  validated pairs: reduction = deep specialist, raw = surface specialist.
- **Why WEAK**: only **5 of 27** surface-hard pairs are genuine analogies (Claude open-book
  shares_method=true). Most "hard" mined pairs are not deep analogies — they are unrelated/weak-link
  pairs surface retrieval fails on for other reasons. Confirms the mine is predominantly topical (the
  RS-23 caveat), so it yields few validated-deep pairs.

## Orbiter audit (the load-bearing overseer step)

Claude vs Mistral open-book `shares_method` on the 38-pair overlap: **Cohen's κ = 0.89** (raw agreement
0.95); Mistral over-prune (Claude keep / Mistral drop) = **1**, under-prune = 1. **Mistral is a reliable
executor for COARSE binary validity** — a sharp contrast with EXP-RS-20, where the *fine-grained*
method_coherence pruning had κ = 0.45 with *systematic* over-pruning (W-SYN). **Finding: Mistral's
trustworthiness as an executor is a function of task granularity** — reliable for coarse KEEP/DROP,
unreliable for fine severity judgments. The mandatory audit did its job (quantified reliability before
any verdict rested on the Mistral pass). Mistral validated 34/80; Claude authoritative on the 27 deep.

## Cascade value (DESCRIPTIVE, post-hoc — informs the build, not the frozen verdict)

On the FULL mixed 80-pair corpus:

| retriever | recall@10 |
|---|---|
| raw-embedding alone | 0.662 |
| reduction alone | 0.525 |
| **UNION (raw top-10 ∪ reduction top-10)** | **0.775** |
| oracle best-of-two | 0.775 |

The **union** of the two retrievers lifts recall **+11.2 pts (0.66 → 0.78), recovering 9 deep-tail pairs
raw misses, losing none**, and captures the entire oracle gain at k=10. This is the opposite of the
RS-23 RRF *fusion* (0.64, dilutive): the correct architecture is **retrieve-with-both → UNION the
candidate sets → LLM re-rank**, not rank-averaging. The union is the cheap-recall stage; the (known
ceiling-level) LLM re-rank supplies precision on the ~20-candidate union → a **scalable discovery
pipeline**: O(N) reductions + O(N) embeddings (free retrieval) + a small per-query LLM re-rank.

## Design / integrity

- Reuses the RS-23 80-pair sample (reductions + raw bge + lexical in hand) + the deterministic
  surface-hard/easy split. Validity probe = the FROZEN `rs22_probe_openbook.md`. Predictions frozen in
  `43-PREREG.md` before running. Union/cascade numbers are descriptive (post-hoc), explicitly not
  promoted to the pre-registered headline.

## Limitations & forward

- Validated-deep n = 5 (WEAK). Combined with Feynman's 5, the specialist claim rests on ~10 deep
  analogies — a large gap, consistently one-directional, but small n. To firm it up: **expand** — reduce
  + orbiter-validate more of the 420 mine to grow the validated-deep stratum to n ≥ 15 (yield ≈ 6% of
  mined pairs → ~250 more pairs; Mistral executor + Claude audit, now trusted at κ=0.89 for the coarse
  pass).
- **Recommended build (empirically grounded):** the **two-retriever-union → LLM-rerank cascade**
  (raw ∪ reduction → LLM sorts the union). The union stage's value is already demonstrated (0.66→0.78);
  the LLM re-rank is the known-strong precision layer. This is the scalable LBD retriever the project
  has lacked. Expansion and the cascade build can proceed in parallel.

Artifacts: `prototypes/rs24_validate.py`, `data/rs24_results.json`, `data/rs24_out_mistral/openbook/`,
`data/rs24_out/openbook/`.
