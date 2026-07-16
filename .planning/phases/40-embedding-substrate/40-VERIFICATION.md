# EXP-RS-21 — Dense Embedding Substrate — VERIFICATION

- **Phase**: 40 · **Run date**: 2026-07-16 · **Verdict**: **KILL** (clean class-negative)
- **Pre-registration**: `40-PREREG.md` (v2, LOCKED 2026-07-16; conventions C-41..C-45; manifest SHA `4dc8da72…`)
- **Artifacts**: `prototypes/data/embed_results_feynman.json`, `embed_results_modern.json`,
  `embed_verdict.json`, `embed_model_manifest.json`, `embed_toytest_pass.json`

## Result — no dense embedding beats the lexical null on the discriminating corpus

**Feynman (5 pairs, discriminating; forward recall@10; per-pair rank of 35):**

| model | 01 | 03 | 04 | 05 | 06 | recall@10 | vs null 0.40 | strict-P1 |
|---|---|---|---|---|---|---|---|---|
| **bge** (headline, general) | 17 | 3 | **20** | 28 | 29 | **0.20** | **below** | ✗ |
| gte (general) | 15 | 2 | **18** | 25 | 20 | **0.20** | **below** | ✗ |
| specter2+proximity (scientific) | 17 | 4 | **5** | 21 | 22 | **0.40** | tie | ✗ |
| mistral-embed (EU API) | 16 | 3 | **7** | 15 | 34 | **0.40** | tie | ✗ |
| — lexical null (C-17) | 31 | 2 | 17 | 7 | 25 | 0.40 | — | — |
| — brute-force LLM (C-20) | 12 | 1 | 1 | 1 | 15 | **0.60** | — | — |

**No model has strict-P1** (needs fwd recall@10 **> 0.40** ∧ pair04 top-10 ∧ ≥1 null-missed). `class↑ =
False` → **KILL**. All four models are **LIVE** (per-model toytest green + modern recall ≥ 0.5), so the
KILL is from working encoders, not a broken pipeline. `embed_gate.py` verdict = KILL (grid-verified total
function; `embed_gate_selftest.py` 4608 cells green).

**Modern (6 pairs, no-regression / liveness guard — lexically trivial, non-discriminating as pre-registered):**
bge 0.67, gte 0.83, specter 0.83, mistral 0.83 forward — all ≥ 0.5 (LIVE); confirms the encoders work and
that modern cannot discriminate a cross-vocabulary win (null already 0.833).

## Mechanism — topical/field dominance (the pre-registered load-bearing risk, Open-Risk-1)

Dense embeddings encode **field/topic** more than **shared mechanism**: same-field distractors sit closer
to the query than the cross-domain mechanism-analogue. Two clean pieces of evidence:

1. **The general open models (bge/gte) are WORSE than lexical** (0.20 < 0.40) — they miss pair04 entirely
   (rank 20/18), recovering only the lexically-easy pair03. Adding dense semantics *hurt* here.
2. **The objective P5 card (C-45) refuses to certify pair04**: on every model pair04's cosine exceeds the
   query's same-field distractor median by the δ=0.02 margin (`objective_pass=True`), BUT the deterministic
   random cross-field control passes at **~0.73–0.81 ≫ 0.30** → `P5obj = False`. The margin is not
   distinctive: ~80% of *random* cross-field pairs clear it too. The objective rule + control did exactly
   their job — they blocked an over-claim that pair04 was a "genuine geometric cross-domain recovery."

## Sub-finding — P3 sub-hypothesis REFUTED (illustrative, n≈1)

The pre-registered sub-hypothesis was "a *general* contrastive embedding beats a *citation-trained
scientific* one (SPECTER2) cross-domain." **The opposite held:** SPECTER2+proximity (rank 5) and
mistral-embed (rank 7) **recover the pair04 anchor**; the general open models bge/gte **do not** (rank
20/18). Citation training *helped* on the cross-domain anchor, not hurt. Per the locked P3 downgrade
(effective n≈1), this is an illustrative per-pair observation, not an inferential claim.

## The one real (but insufficient) signal

The scientific/API embeddings DO pull the hard cross-vocabulary pair04 closer than lexical does
(rank **17 → 5/7**, into the top-10 the null misses) — a genuine, modest cross-vocabulary signal. But
they simultaneously **lose the lexically-easy pair05** (null rank 7 → embedding rank 21/15), netting
**exactly the null's 0.40**. So dense embeddings *redistribute* which pairs they get (trading an easy
lexical pair for a hard cross-vocab one) without a net gain over free lexical retrieval, and stay well
below the 0.60 LLM bar. Honest read: **TIE the lexical null, do not reach the incumbent.**

## Statistical honesty (as pre-registered)

Effective n ≈ 1 (pair04); recall@10 chance level ≈ 10/35 = 0.286/pair → Binomial(5, 0.286) gives
P(recall ≥ 0.40) = 0.44. The 0.40 lexical null is itself near chance; the KILL is phrased "static
embedding geometry adds nothing beyond the ~chance lexical null," NOT "captures nothing."

## Protocol note (C-44 / P0 ordering)

The blind C-35 modern LLM baseline was **NOT computed**: the Feynman P1 gate KILLED first, and C-35 is
"only needed if the Feynman gate passes" (the C-35 lineage; matches EXP-RS-19's identical deferral).
Modern embeddings were run ONLY to confirm encoder liveness (modern ≥ 0.5), not for any C-35 comparison.
Since modern is a no-regression FLOOR (not a strict-beat) and the KILL is decided entirely on Feynman,
no C-35 bar was needed and no leakage-control ordering was compromised (there was no bar to tune to).
Ablations (BGE-asymmetric, reverse, text-field) also deferred per the "only if P1 passes" rule; the
already-computed reverse direction is 0.40 for all models (still ties the null).

## Escalation (per the pre-registered KILL gate)

Both static-representation routes now fail the 0.60 bar: the lexical-intermediate line (RS-16→20) AND the
dense-embedding substrate (RS-21). The remaining pre-registered escalation is a **pure LLM-judge cascade**
— but it MUST structurally differ from the C-20 one-shot 35-way ranking (already the 0.60 incumbent):
e.g. pairwise/tournament judging, or scaling beyond the 36-paper pool. **Honest terminal-verdict caveat:**
the C-20 baseline IS a full-context LLM at 0.60; if no structurally-different LLM-judge can be made to beat
it, this KILL is the **terminal chapter verdict** — the cross-field analogy signal is real but recoverable
ONLY by full-context LLM reasoning, and the practical LBD method IS the brute-force LLM baseline (0.60).

## Durable residue

- A reusable deterministic **dense-embedding retrieval + gate harness** (`embed_score.py`,
  `embed_gate.py`, grid-verified total-function gate + self-test, per-model liveness toytest).
- The finding that **scientific/citation embeddings surface cross-domain mechanism analogy better than
  general ones** (recover pair04) — counter-intuitive, worth a follow-up if the LLM-judge cascade runs.
- The frozen model manifest + the clean "static geometry ≈ chance-lexical, LLM reasoning = 0.60"
  separation — a sharp, publishable negative that isolates the capability to LLM reasoning.
