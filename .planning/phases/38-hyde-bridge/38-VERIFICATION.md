# Phase 38 — EXP-RS-19 verification: HyDE-Bridge (#2 slot-frames, generation-based retrieval)

**Verdict (2026-07-07): KILL of the pinned headline at the cheap gate — but with PROVEN surviving
signal.** The pre-registered headline arm (K=5, max-pool, λ=0) scores Feynman forward recall@10 =
**0.20**, *below* the 0.40 lexical null → **GATE-A FAILS → KILL** per C-34/C-37. HOWEVER the
generation mechanism is verified to work: **GATE-B PASSES** (pair04 percolation→epidemics recovered
rank 17→4), and the pre-registered **K=1 ablation ties the LLM baseline (0.60)**. The KILL is caused
by the *pinned aggregation* (max-pool over 5 hypotheticals inflates distractors), NOT by absent
signal — a materially different outcome from the RS-16/17/18 flat negatives.

## What was pre-registered (LOCKED — THREAD.md, CONVENTIONS C-31..C-37)

HyDE-Bridge: one blind subagent per QUERY generates K=5 cross-field hypothetical abstracts (same
method, different object, native target-field vocab, from a blind SHA-256-frozen prompt); candidates
keep their free C-17 TF-IDF; `score = max_{k≤5} cos(vec(h_k), tfidf(c)) − λ·object_sim`. Headline
PINNED (C-36) at (λ=0, K=5, max, forward); ablations descriptive, non-promotable. Cheap Feynman gate
(C-34): GATE-A = headline fwd recall@10 ≥ 3/5 AND recovers a null-missed pair (pair01/04/06); GATE-B
= pair04 in top-10; KILL iff GATE-A OR GATE-B fails. Verified C-17 null = Feynman 0.40 / modern 0.833.

## What was executed

1. Blind HyDE prompt authored by a no-benchmark subagent (example domain = compressed-sensing/MRI),
   SHA-256 `f06ed6aa…`, committed before generation (C-31 lock, commit `4d97f36`).
2. `hyde_score.py` + `hyde_toytest.py` (toytest asserts a hypothetical crosses a vocab gap the null
   cannot; passes) committed before data.
3. 5 blind generations for the Feynman side_a query papers — target fields landed on the real
   analogue domains (pair04→epidemiology, pair05→economics, pair06→economic-geography). Frozen:
   SHA-256 `68cbf3f4…`, committed before scoring (commit landed pre-gate).
4. Cheap gate run: headline + 4 descriptive ablations, forward-only, full 36-candidate retrieval.

## Result — the cheap gate (decisive)

Feynman forward recall@10 by arm, and per-pair ranks (null in the first column; * = recovered top-10):

| pair | NULL | **headline K5 max** | λ=0.5 K5 | K5 mean | K3 max | K1 max |
|---|---|---|---|---|---|---|
| pair01 ising↔opinion | 31 | 26 | 20 | 31 | 24 | 26 |
| pair03 sir↔rumour | 2 | 16 | 29 | 11 | 12 | **4*** |
| pair04 perc↔epidemics | 17 | **4*** | **3*** | 13 | **2*** | **1*** |
| pair05 lv↔markets | 7 | 19 | 20 | 21 | **10*** | **6*** |
| pair06 turing↔economy | 25 | 32 | 25 | 32 | 29 | 35 |
| **recall@10** | 0.40 | **0.20** | 0.20 | 0.00 | 0.40 | **0.60** |

- **GATE-A** (headline recall@10 ≥ 3/5 AND recover a null-missed pair): **FALSE** (0.20; it *did*
  recover the null-missed pair04, but 0.20 < 0.60).
- **GATE-B** (pair04 in top-10): **TRUE** (rank 4). ⇒ **KILL** (GATE-A fails).

## Diagnosis (clean, mechanistic)

1. **The mechanism WORKS (verified).** pair04's epidemiology hypothetical — generated blind from a
   percolation/random-graph paper that never mentions disease — matches the real "spread of epidemic
   disease on networks" paper at cosine **0.1546** (vs the other 4 hypotheticals 0.008–0.046, vs the
   null side_a↔side_b 0.0586), driven by *transmission / population / epidemics / cases / spread*
   tokens the hypothetical emitted. Generation genuinely converted the latent percolation≡epidemic
   equivalence into retrievable tokens the lexical comparator could see. This is the #2 premise,
   demonstrated.
2. **Max-pool distractor inflation killed the headline** (the design's residual risk #4). recall@10
   is monotonic in K: **K=1 → 0.60, K=3 → 0.40, K=5 (headline) → 0.20.** Each extra hypothetical
   gives distractors another chance to match; max-pool takes the best, so the true pair drowns.
   pair03 (null r2) degrades to r16 and pair05 (null r7) to r19 at K=5. The pinned K=5 aggregation
   is the failure — not the generation.
3. **The honest ceiling is a TIE (as predicted).** K=1 recovers exactly {pair03, pair04, pair05} =
   the SAME three pairs the LLM baseline gets (0.60) — including pair04 which the lexical null misses.
   Nobody (null, LLM, or any HyDE arm) recovers pair01 (diffuse review side_b) or pair06 (Turing→Zipf,
   which the LLM also misses). So even the best pre-registered ablation only TIES the incumbent — the
   design's honest P3 prediction — and the recall number is not the product.
4. **Object-penalty negligible on Feynman** (λ=0.5 ≈ λ=0 = 0.20): true-pair object cosines are near
   zero, so the penalty barely moves the headline — consistent with P4's motivation.

## Verdict vs each locked prediction

- **P1 (gate) — FALSIFIED** (headline recall@10 = 0.20 < 3/5; GATE-A fails). KILL.
- **P2 (beat the 0.40 null) — FALSIFIED for the headline** (0.20 < 0.40 — max-pool is worse than
  doing nothing); the K=1 ablation (0.60) beats it, but K=1 is non-promotable (C-36).
- **P3 (tie 0.60) — not met by the headline;** met only by the non-promotable K=1 ablation.
- **P4 — supported** (object penalty ≈ inert on Feynman, as predicted).
- **P5 (artifact ≥3/5) — not met by the headline** (only pair04 robustly recovered); BUT pair04's
  transfer card is real and auditable (percolation→epidemiology, invasion-threshold / offspring
  distribution → matches `bridge_names` threshold-phenomenon / graph-connectivity).

## The tension the KILL surfaces

C-37's KILL branch says "the lexical-retrieval line is exhausted → escalate OUT of lexical
intermediates → do NOT run another TF-IDF variant." That directive assumed a gate failure ⇒ *no
signal survives the comparator*. **That premise is violated here:** signal DOES survive (pair04
recovered and verified; K=1 ties the LLM). So the KILL is real for the *pinned headline*, but the
generation idea is not dead — it is a proven RECALL mechanism throttled by a distractor-inflating
aggregation. Both C-37's escalation branch ("LLM-judge cascade") and the design's own PIVOT ("next
build = a VERIFY stage") converge on the same next move.

## Constructive residue

- **Proven:** blind cross-field generation surfaces a null-missed cross-vocabulary analogy (pair04)
  and ties the LLM baseline at K=1 — the first method in the chapter to recover a pair the lexical
  comparator misses, and the first auditable directed transfer card (percolation→epidemiology).
- **Diagnosed:** max-pool over K hypotheticals inflates distractors (monotonic in K); a wide
  generative net needs a precision stage.
- Reusable: `hyde_prompt.md` (frozen), `hyde_score.py`, the 5 frozen Feynman generations, the harness.

## Forward path — HUMAN's go/kill/pivot (options; not auto-executed)

The pinned headline is KILLED. The convergent, best-supported next move is the **generate→VERIFY
cascade (brainstorm #3)**: HyDE is the (proven) recall stage that casts the cross-field net; an
LLM/CAS VERIFY stage audits each proposed transfer (shared method + differing object) to prune the
max-pool distractor inflation. Alternatives: (A) honor C-37 literally and escalate to a real
semantic-embedding substrate; (B) a fresh pre-registered EXP-RS-20 with a distractor-robust
aggregation (K=1 or reciprocal-rank fusion instead of max-pool cosine) — motivated by the K=1 tie,
but C-37 cautioned against another TF-IDF variant. The go/kill/pivot is the human's.
