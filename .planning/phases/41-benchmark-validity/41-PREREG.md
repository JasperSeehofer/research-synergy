# EXP-RS-22 — Benchmark Validity (v3, lean one-directional): does the 0.60 LLM baseline contain a real REASONING component?

- **Status**: pre-registration DRAFT **v3** (predictions to be LOCKED before any mining/probe/scoring)
- **Kind**: validity / meta experiment on the incumbent recall@10 = 0.60 brute-force LLM baseline — the
  bar every prior LBD method (EXP-RS-16→21) failed. Adjudicates whether that bar reflects real
  cross-domain reasoning.
- **Scope (deliberately one-directional — the honest reach)**: two adversarial panels (2026-07-17)
  established that the memorization↔reasoning decomposition is **near-non-identifiable at achievable
  sample size** — confirming *memorization* needs a TOST equivalence whose margin is vacuous at n≈15–50
  (v2 panel MF8), and you cannot cleanly prove a null. So v3 tests **only the falsifiable direction:**
  *is there a real reasoning component?* Outcomes are **REASONING-CONFIRMED (chapter CLOSES, airtight)**
  or **INCONCLUSIVE**. **MEMORIZATION-CONFIRMED / chapter-REOPEN is explicitly NOT an available outcome**
  (not falsifiable here) — a finding in itself.
- **Provenance / version arc**: human critique (H-CORRECT-03) → v1 fame-gradient (panel KILL: obscurity ≡
  reasoning-difficulty confound vs a lexical-only control) → v2 direct-probe (panel KILL: confound
  RELOCATED onto probe✗ / recall≠recognition / a scorer measuring query-vocabulary paraphrase; +
  memorization non-identifiable) → **v3: one-directional, with the panels' cheap high-value fixes.**
- **Robust result, NOT under test**: no static method (lexical OR dense embedding) beats its leakage-free
  null (embeddings tie lexical at 0.40; SME 0.00). Stands regardless of this experiment.

## The identified question

For pairs where the pinned model demonstrably lacks the analogy in memory (fails BOTH free-recall AND
recognition), does its retrieval still beat BOTH memory-free floors (lexical AND dense-embedding), after
removing paper-recognition and in-context priming? A positive answer is **reasoning over the given texts**
— it cannot be memory (removed by the two-format probe✗), lexical similarity (removed vs the lexical
null), distributional semantic similarity (removed vs the embedding null, RS-21), paper familiarity
(controlled), or priming (independence). A non-positive answer is **INCONCLUSIVE** — explicitly NOT a
memorization claim.

`REASON = median over the clean stratum of ( pctile_rank_LLM(side_b) − pctile_rank_maxnull(side_b) )`,
where `maxnull = the better (higher) of the lexical-TF-IDF and RS-21 dense-embedding retrieval` per pair.
One-sided test that `REASON > +δ` (δ blind-authored, below). Lower percentile rank = better; sign
oriented so `REASON > 0` = LLM ranks side_b better than the best memory-free method.

## Instruments (all on the pinned model; probe ⟂ retrieval — panel MF5)

Every call runs in an **independent, fresh session, zero shared history**; probe outputs never enter any
retrieval context; per-pair order counterbalanced; a SHA of each call's full input is logged to prove no
leakage.

1. **Retrieval** (C-19/C-20): side_a + fixed pool of K candidates → percentile rank of side_b.
2. **Free-recall probe** (C-46): side_a {title,abstract} ONLY, no candidates → NAME {target_field,
   shared_mechanism, confidence}.
3. **Recognition probe** (C-46, panel MF2): forced-choice — given side_a, pick the analogous FIELD from a
   fixed list (no reasoning scaffold). Recall is strictly harder than recognition, so a pair must fail
   BOTH to count as memory-absent.
4. **Paper-familiarity probe** (control, SHOULD-7): given side_b's title only, does the model
   recognize/summarize the specific paper? (recognition-format).
5. **Open-book analogy probe** (panel MF1): given side_a AND side_b's shared method shown, produce the
   mapping — separates "can't recall" from "can't reason" (used to anchor the strength interpretation).

## Memory-isolating probe scorer (panel MF3 — the load-bearing fix)

The v2 killer: bridge_names tokens are side_a's OWN native vocabulary, so token-matching scored
query-paraphrase, not cross-field memory. v3:
- Score the **memory-bearing `target_field`** (and mechanism) against **side_b's HELD-OUT field**, NOT
  side_a's tokens; **subtract side_a's native abstract vocabulary** so a probe✓ cannot be earned by
  paraphrasing the query.
- Judge by a **blind semantic-equivalence judge** (does the named target field/mechanism denote side_b's
  field/mechanism), NOT a brittle token rule (token rule kept only as a cheap pre-filter).
- Same **any-valid-cross-field granularity** as retrieval (removes the SHOULD-3 mismatch).

## Clean stratum (memory-absent, unambiguous)

A pair enters the clean (probe✗) stratum iff: fails free-recall AND fails recognition (memory-absent) AND
the non-match is **low-confidence** (high-confidence wrong = "confident-hallucination / wrong-reasoning"
stratum, analyzed separately — panel S2). The **clean-stratum composition is reported and gated** (field
diversity; if it concentrates in 1–2 field-pairs → INCONCLUSIVE). Estimate the **probe✗ false-miss rate**
from a blind-adjudicated sample; a pre-registered max tolerable rate is a GATE variable (panel MF2).
`analogy_strength` (fame-blind rubric, ≥2 blind raters, κ-floor) is reported; the reasoning claim is
**anchored on the high-strength ∧ clean cell** (a real, tight analogy the model genuinely can't recall).

## Metric, estimand, statistics (panels MF6/MF8/S4)

- **Metric = percentile rank** of side_b in a **fixed pool size K across ALL pairs** (Feynman + modern +
  mined) — pool-size-invariant; the reopened-methods re-run (if ever) uses the same unit/pool.
- **ONE estimand** = the paired difference `pctile_rank_LLM − pctile_rank_maxnull` per clean-stratum pair;
  **primary test = paired Wilcoxon signed-rank (one-sided REASON > +δ) + a bootstrap CI**. NO mixed model,
  NO arm×covariate interaction as primary (unfittable at this n). Covariates (strength, familiarity)
  reported as stratified/conditional descriptive checks.
- **Power**: a pre-LOCK bootstrap/assurance simulation sets the clean-stratum n-floor for the MDE; default
  **INCONCLUSIVE** if the floor is unmet. Corpus sized to clear it (below).

## Blind-authored decision constants (panel MF7 — remove the staked orchestrator)

`δ` (reasoning margin, percentile-rank units), the positive-control probe-fire floor, the pinned-retrieval
reproduction band, the max probe✗ false-miss rate, the clean-stratum n-floor, and the strength-rubric
κ-floor are **authored by a no-stake blind subagent** (the C-22/C-31/C-38 lineage — no benchmark/
bridge_names/thread-outcome access), justified against the historical 0.60−0.40 = 20-point recall
headroom, and **SHA-frozen BEFORE any scoring**. The staked orchestrator does NOT set the decision
boundary. (`δ` is the verdict; it must be blind + frozen.)

**FROZEN (blind-authored 2026-07-17 → `prototypes/data/rs22_constants.json`, SHA-256
`af5ee11c7828fbec0bf9eb6a9520e82cb57dbebfcacf4f3290b108c3a8643c33`):** K=50 pool · δ=5 percentile-rank
pts · α=0.05 one-sided · **n_floor = 110 clean-stratum pairs** (paired-Wilcoxon 80% power at δ, SD_d≈20;
standardized effect 0.25) · positive control m=40, fire ≥0.90 (Wilson-LB clears) · reproduction band
recall@10 ∈ [0.50, 0.90] (excludes the 0.40 null) · probe false-miss ≤0.10 · rubric κ ≥0.60.
**Decision rule = REASONING-CONFIRMED iff one-sided Wilcoxon p<0.05 AND bootstrap CI-lower > 5 pts on
≥110 clean pairs, all gates passed; else INCONCLUSIVE.**

**⚠ Scale implication (honest):** n_floor = **110 memory-absent pairs** is the real power cost. Since the
model recalls/recognizes most *famous* analogies (they fail the clean-stratum test), reaching 110
clean pairs implies mining **several hundred** cross-field analogy pairs (deterministic frozen blocks) —
a substantial multi-session orbiter build (mining + 5 instruments × 2 families + scoring), NOT a
few-hour run. This is where pi/Mistral does the heavy mechanical lifting under the mandatory Claude
audit. If this scale is not warranted, the alternative is the terminal-verdict write-up (chapter option
C) — a scope call for the human at LOCK.

## Corpus (moderate blind, frozen, deterministic — panels MF9/S3)

- Mine cross-field analogy pairs from "bridge papers" asserting an explicit cross-field equivalence.
  **Split the mining**: (i) *mechanical* frozen-query candidate retrieval → **Mistral-OK**; (ii) the
  *"do the two papers really share a method"* validity judgment = a W-SYN synthesis task → **Claude / a
  cross-family adjudicated panel, under the mandatory audit** (NOT Mistral alone — it over-pruned in
  RS-20).
- side_a/side_b = the two INDEPENDENT domain papers (NOT the bridge paper); side_a's abstract must not
  contain bridge-assertion language (checked). Filters: cross-field ∧ ≥200-char abstracts. Distractors per
  the C-14 rule but to a **fixed pool K**; a per-pair distractor-difficulty statistic is measured and
  **balanced across probe✓/✗** (panel S7).
- **Frozen deterministic draw**: snapshot the source index + date + query strings + raw returned IDs,
  deterministic tie-break, **commit + SHA the raw mined set BEFORE the probe split**. Expansion (if the
  clean-stratum n-floor is unmet) is a **pre-specified additional deterministic block**, never an
  iterate-until-n loop that peeks at the split. Target ~60–100 mined + Feynman 5 + modern 6 anchors.

## GATE — total function (panel MF3/MF8/MF10/MF11; grid-self-tested, C-45 pattern)

`posctrl` = [free-recall+recognition probes fire on the ultra-famous positive-control set above the blind
floor] ∧ [negative control holds: single-domain/method-free side_a does NOT spuriously emit a
high-confidence analogy] ∧ [**pinned-model retrieval reproduces the incumbent** on the 5 Feynman pairs:
recall@10 ∈ the blind reproduction band, min > the 0.40 lexical null — panel MF10] ∧ [**audit passed**:
Claude audited a defined sample of Mistral scoring/mining KEEP-DROP, fix-count logged, disagreement below
the blind threshold — panel MF11] ∧ [probe✗ false-miss rate ≤ blind max] ∧ [clean-stratum n ≥ blind
floor ∧ field-diverse].

```
¬posctrl                                   → INVALID / INCONCLUSIVE (instrument or reproduction or audit
                                             failed) — fix + re-run; NOT a verdict.
posctrl:
   REASON one-sided Wilcoxon p<α ∧ CI lower > +δ   → REASONING-CONFIRMED → the 0.60 has a real reasoning
                                             component surviving removal of lexical+embedding+recall+
                                             recognition memory + familiarity + priming → the RS-16→21
                                             KILLs are AIRTIGHT → terminal chapter verdict (CLOSE).
   otherwise                               → INCONCLUSIVE → cannot confirm reasoning at this power;
                                             explicitly NOT a memorization claim (not falsifiable here).
```
Two terminal outcomes + INVALID. MEMORIZATION-CONFIRMED is intentionally absent (see Scope). Final gate
adjudication + the scoring audit are performed by a party WITHOUT a stake in the outcome (blind
subagent / cross-family panel), not the staked orchestrator (panel MF11).

## Orbiter / pi execution roles (human directive — active orbiter use; W-SYN-aware)

- **pi/Mistral (executor, T0–T2, mechanical only)**: frozen-query candidate mining retrieval; the
  objective token-rule PRE-FILTER of probe outputs; assembling rank tables + deterministic stats. Direct
  Mistral API ≤4 workers for >~50 calls (429 cap).
- **Claude (T3, non-delegable)**: the "really share a method" mining validity judgment; probe GENERATION
  on the pinned primary model; the blind semantic-equivalence probe judging; the **operationalized
  mandatory audit** (defined sample %, logged fix-count as the comparability metric, a pre-registered
  disagreement threshold that triggers a fan-out re-run before any verdict).
- **No-stake blind subagent**: authors the decision constants (δ etc.) + performs the final gate
  adjudication (removes the orchestrator's stake from the verdict).
- **Cross-family Mistral probe arm (confirmatory-only, pre-registered rule — panel S6)**: report REASON on
  each family's own clean stratum + the intersection; **disagreement → report both, do NOT upgrade to
  REASONING**; Mistral's probe✗ is confounded by its own W-SYN generation weakness, so it is corroborative
  context, never decisive.
- Log a `pi-migration-ledger` row per pi run.

## Proposed conventions (redefine the not-yet-locked C-46..C-48 for v3)

- **C-46 (two-format memory probe + memory-isolating scorer)**: free-recall + recognition probes, blind
  frozen prompts (SHA), probe ⟂ retrieval (fresh sessions, hashed inputs, counterbalanced); score the
  memory-bearing `target_field`/mechanism vs side_b's HELD-OUT field with side_a's native vocabulary
  subtracted, by a blind semantic-equivalence judge (token rule = pre-filter only); clean stratum = fails
  BOTH formats ∧ low-confidence; run on the pinned model + a confirmatory cross-family arm.
- **C-47 (controls + blind constants)**: positive (famous must fire) + negative (method-free must not
  spuriously fire, redefined to match the generation probe — panel S1) instrument controls; pinned-model
  incumbent-retrieval reproduction check; paper-familiarity recognition control; fame-blind
  analogy-strength rubric (≥2 blind raters + κ-floor); probe✗ false-miss-rate control. ALL decision
  constants (δ, floors, MDE-n, κ, band) authored by a no-stake blind subagent, SHA-frozen before scoring.
- **C-48 (one-directional metric + total gate + orbiter roles)**: percentile rank at fixed pool K; ONE
  estimand = paired Wilcoxon (one-sided REASON > +δ) + bootstrap CI on `LLM − max(lexical,embedding)` over
  the clean stratum; the total gate {REASONING-CONFIRMED / INCONCLUSIVE / INVALID}, MEMORIZATION not an
  outcome; deterministic frozen mining (snapshot+SHA, pre-specified expansion blocks); Mistral executes
  mechanical work vs the frozen rubric with a MANDATORY operationalized Claude audit (fix-count +
  disagreement-triggered re-run) and no-stake final adjudication; reuses C-14/C-17/C-19/C-20/C-24/C-36 +
  the RS-21 embedding null.

## Open risks (residual, for the record)

1. **One-directional by design**: can CLOSE (confirm reasoning) or be INCONCLUSIVE; cannot confirm
   memorization (not falsifiable at scale). Stated up front; an INCONCLUSIVE is an honest non-result,
   never read as "reasoning."
2. **Clean-stratum n**: if the model recalls+recognizes almost everything, the memory-absent stratum is
   starved → INCONCLUSIVE (pre-registered), not a forced verdict; the deterministic expansion block
   mitigates.
3. **Strength-rubric reliability**: blind, ≥2 raters, κ-floor gate; below floor → INCONCLUSIVE.
4. **Residual reasoning-difficulty on probe✗**: the one-directional framing is robust to it (finding
   REASON>0 is not threatened by "probe✗ pairs are harder"; only a null-direction memorization claim was,
   and that is not attempted). The open-book probe + high-strength anchor interpret the effect.
5. **Cross-family arm is confirmatory-only** (Mistral W-SYN); never upgrades the verdict.
