# EXP-RS-24 — Phase 43 Pre-registration: Orbiter-Validated Deep-Analogy Subset

- **Status**: pre-registered; predictions frozen BEFORE running (2026-07-18).
- **Kind**: confirmation experiment for EXP-RS-23's specialist claim, + first faithful orbiter loop on a
  new task type (Mistral executor + mandatory Claude audit), per the human's standing orbiter directive.

## Motivation

EXP-RS-23 found the mechanism-reduction substrate is a **specialist** for deep cross-vocabulary analogies
(Feynman 0.60 vs raw-embedding 0.20) but net-negative on the broad mine (0.53 vs raw 0.66). Two
confounds blocked a clean large-n conclusion: the 420-mine pairs are (a) mostly **topical** (surface
already finds them) and (b) of **unvalidated method-sharing** (the RS-22 §6.1 validity judgment was
deferred). This experiment removes both: restrict to **surface-hard ∧ method-validated** pairs and re-test
reduction vs raw there.

## Orbiter roles (human standing directive)

- **Mistral (executor, mechanical bulk):** run the FROZEN open-book method-sharing probe
  (`rs22_probe_openbook.md`, SHA on file) on all 80 mined-sample pairs → `{mapping, shares_method,
  brief_justification}`. Direct Mistral API, ≤4 workers (429 cap). This is the deferred RS-22 §6.1
  validity judgment executed mechanically.
- **Claude (overseer, mandatory audit — non-delegable):** compare Mistral's `shares_method` against the
  EXISTING Claude open-book verdicts on the 15 RS-22-slice pairs (overlap), compute agreement + Cohen's
  κ + the **over-pruning delta** (pairs Claude=share / Mistral=¬share), the known Mistral W-SYN failure.
  If κ < 0.6 OR over-pruning is material, the Mistral KEEP set is treated as a LOWER bound and Claude
  re-adjudicates the deep pairs directly. Log a `pi-migration-ledger` row.
- The **validated set** = pairs with `shares_method = true` (Mistral, audit-corrected on the deep tail).

## Design

- Corpus: the 80-pair mined sample from EXP-RS-23 (reductions + raw-abstract bge embeddings + lexical
  already computed; 160 papers). Deterministic surface split (from RS-23): **surface-hard** = side_b
  raw-embedding rank > 10 (n=27); **surface-easy** = ≤ 10 (n=53).
- Score `reduction_bge` vs `raw_bge` vs `lexical_null` (identical machinery to RS-23) on:
  **VALIDATED-DEEP** (surface-hard ∧ validated) and **VALIDATED-EASY** (surface-easy ∧ validated).

## Predictions (FROZEN)

- **P1 (the confirmation):** on VALIDATED-DEEP pairs, `reduction_bge` recall@10 > `raw_bge` recall@10 —
  the reduction wins the deep tail once shallow/non-analogy pairs are removed (corroborates Feynman at
  larger, validated n).
- **P2 (the anti-correlation holds):** on VALIDATED-EASY pairs, `raw_bge` recall@10 ≥ `reduction_bge` —
  surface wins where surface works.
- **P3 (orbiter audit):** report Claude↔Mistral `shares_method` κ on the 15-pair overlap; if Mistral
  over-prunes (drops deep validated pairs), it is flagged and corrected (the audit is load-bearing, not
  cosmetic).

## GATE

```
CONFIRM  ⇔ P1 (∧ P2 as expected)  → reduction is a validated deep-analogy specialist at n>5 →
                                     build the router/cascade (raw for the bulk + reduction for the
                                     hard tail → LLM re-rank the union) as the scalable LBD retriever.
WEAK     ⇔ P1 holds but VALIDATED-DEEP n < 8  → directionally supportive, underpowered → expand the
                                     mined sample (more reductions) before building.
REFUTE   ⇔ ¬P1 (raw ≥ reduction even on validated-deep)  → the Feynman win was a small-n fluke /
                                     corpus artifact; the reduction is not a reliable deep-tail retriever
                                     → the practical method stays raw-text LLM (O(N²)), scaling unsolved.
```

Honest expectation: given Feynman + the RS-23 hard-tail rescue (9/27), P1 is likely to hold
directionally; the risk is small validated-deep n (→ WEAK). The orbiter audit may reveal Mistral
over-pruning (expected from RS-20 W-SYN), which the design corrects.
