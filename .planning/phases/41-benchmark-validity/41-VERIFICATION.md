# EXP-RS-22 — Phase 41 Verification: Benchmark Validity (v3, one-directional reasoning test)

- **Status**: TERMINAL — **INCONCLUSIVE** (frozen-gate verdict `INVALID/INCONCLUSIVE`).
- **Date**: 2026-07-17
- **Decision basis**: end-to-end pipeline built + validated; verdict drawn on the **validation-slice**
  evidence (human decision 2026-07-17, after the slice revealed the full run is a foregone null;
  the full 420-instrument run was NOT executed — see §6).
- **Adjudicated by**: the frozen total-function gate `prototypes/rs22_gate.py` (C-48; grid-verified
  by `rs22_gate_selftest.py`), NOT the staked orchestrator. Constants blind-authored (SHA
  `af5ee11c…`). Verdict artifact: `prototypes/data/rs22_verdict_slice.json`.

## 1. Verdict

**INCONCLUSIVE** — a real reasoning component in the incumbent 0.60 brute-force LLM baseline
**cannot be confirmed** at feasible sample size. Per the pre-registered scope (C-48, two adversarial
panels), MEMORIZATION / chapter-REOPEN is **not an available outcome** (near-non-identifiable), so
INCONCLUSIVE is the honest terminal result of the falsifiable direction. The frozen gate returns
`INVALID/INCONCLUSIVE` because **≥1 validity precondition fails** — REASON may not even be
interpreted. This is one of the two pre-registered non-REASONING outcomes; it is **explicitly NOT a
memorization claim**.

Frozen gate output (`rs22_verdict_slice.json`):

| posctrl precondition | result | meaning |
|---|---|---|
| `positive_control_fire >= 0.90` | **PASS** (5/5 sample) | instruments fire on famous analogies — the probes work |
| `negative_control_holds` | **PASS** (0/3 spurious) | recall well-calibrated: returns `null`/conf≈0.07 on method-free papers |
| `pinned_retrieval_reproduces_incumbent` | **FAIL** | Opus 4.8 anchor recall@10 = **1.00** ∉ blind band [0.50, 0.90] |
| `clean_n >= 110` | **FAIL** | **0** memory-absent pairs in the slice → ~0/420 extrapolated |
| `field_diverse (>=3)` | FAIL | follows from `clean_n = 0` |
| `audit_passed` / `false_miss <= 0.10` / `rubric_kappa >= 0.60` | FAIL | not run on the slice path (§6) |

The two **instrument** controls PASS; the two **substantive** preconditions FAIL. The instruments are
sound — the null is real, not a harness artifact.

## 2. What was built (all delivered, frozen, committed)

- **Frozen leakage-aware benchmark corpus** — `rs22_mined_pairs.json`, **420 cross-field analogy
  pairs** (7 deterministic blocks × 60), mined memory-blind per the frozen protocol (SHA
  `97ee43a7…`); snapshot `658bf02e…`, pairs `e7929b33…`. A durable, reusable asset (banked
  regardless of this verdict — it also re-powers the chronically-n≈5 chapter).
- **Blind operational spec** (`rs22_operational_spec.md`, SHA `aecc04a0…`) + **mechanism probe**
  (`rs22_probe_mechanism.md`, `72de2252…`) — authored by a no-stake blind subagent (C-22/C-31/C-38
  lineage): deterministic field-labels, K_menu=8 recognition menu, K=50 retrieval pool,
  rank-repair, self-field guard, clean predicate. RNG-free, reproducible.
- **Blind control sets** — `rs22_posctrl_set.json` (40 famous analogies, `204d4da7…`),
  `rs22_negctrl_set.json` (15 method-free papers, `07b174ed…`).
- **Retrieval instrument** (`rs22_retrieval_prompt.md`, `f0fa45cc…`) — faithful C-20 conditional
  ranking, reproduction-gated.
- **Deterministic harness** `rs22_probe.py` — emit-inputs → Workflow/Mistral dispatch → score;
  lexical (C-17) + RS-21 dense-embedding nulls; total-function core self-tested.
- **Instrument dispatch** — a Workflow fan-out of fresh-session, hashed-input blind subagents on the
  pinned Opus 4.8 (probe ⟂ retrieval isolation, C-46 MF5). Slice: **128/128 calls, 0 errors.**

## 3. The three structural findings (why the full run is a foregone null)

Verified from per-pair slice data (`rs22_score_slice15_claude.json`); all three are **structural**
(mechanistic), not sampling noise, so n=420 would not change them.

**F1 — The incumbent 0.60 bar is model-relative → reproduction FAILS (INVALID).**
Pinned Opus 4.8 ranks side_b at **rank 1 on all 5 Feynman anchors** (recall@10 = 1.00), including
pair01 (incumbent rank 12) and pair06 (incumbent rank 15) that the 0.60 baseline missed. The
incumbent 0.60 (C-20, EXP-RS-10/16, "one Claude call ranks 35 candidates") was produced by a
**weaker/earlier Claude**; Opus 4.8 far exceeds it. Consequence: the "0.60 bar that every static
method RS-16→21 failed" is **not a fixed property of the task** — it is the ranking of one (now
superseded) model. On a current model the same conditional-retrieval task is near-trivial.

**F2 — The clean (memory-absent) stratum is starved → INCONCLUSIVE.**
Free-recall fails on **15/15** slice pairs (it never names side_b's field — the hard probe works),
but the recognition probe is **never low-confidence**: confidence range 0.55–0.82 across all 15,
even on its 4/15 wrong answers. The blind-frozen clean predicate (fails BOTH formats **AND**
recognition confidence ≤ 0.5) is therefore essentially unsatisfiable → **0/15 clean** → ~0/420. The
model is confidently right or confidently wrong on forced choice, ~never uncertain. (The 0.5
threshold is blind-frozen; relaxing it post-hoc to manufacture a stratum is exactly the un-blinding
the design forbids.)

**F3 — No reasoning signal even where memory is absent.**
The retrieval instrument ranks side_b **#1 on 15/15** pairs (pctile_llm = 0 everywhere). But so does
the lexical null on the memory-absent-candidate pairs: on the 4/15 that fail both formats, the
lexical TF-IDF null also ranks side_b #1 → **d = null − LLM = 0** (no reasoning signal). The 8/15
pairs where the LLM strictly beats the null (d>0) are precisely the ones the model **recognizes**
(not memory-absent). "Reasoning-win" and "memory-absent" are **largely disjoint** → REASON on the
clean stratum ≈ 0. Mined cross-field pairs — though cross-archive — share enough surface vocabulary
(they were linked by a bridge asserting equivalence) that TF-IDF already finds side_b whenever the
model can't recall it.

## 4. Interpretation

The incumbent recall@10 = 0.60 **cannot be shown to contain a reasoning component surviving the
removal of memory + lexical similarity**, at feasible n — because (F2) the memory-absent stratum is
empty for this model, and (F3) where memory is absent, retrieval merely ties the lexical null. Under
the pre-registered scope this is **INCONCLUSIVE**, not a reasoning finding and (by design) not a
memorization finding. Independently, (F1) the reproduction gate fails: Opus 4.8 does not reproduce
the 0.60 incumbent, so the instrument is not even measuring the same model that set the bar.

**Robust chapter-level residue (NOT under test, unaffected):** no static representation — lexical
(RS-16→20) or dense embedding (RS-21) — beats its leakage-free null; the practical LBD method is the
brute-force full-context LLM. F1 sharpens this: that bar rises with model capability (Opus 4.8 ≈
1.00 on the Feynman anchors).

## 5. Deviations & transparency

- **field_label bug-fix (documented deviation from `rs22_operational_spec.md` §1).** The blind spec
  lowercases the category before the SUBCAT_LABELS lookup, but the table keys carry uppercase arXiv
  subtags (`math.QA`, `q-fin.ST`, `cs.LG`), so every letter-code subcategory missed the table and
  fell to a cryptic "archive (subtag)" label. The table's existence shows the intended labels; the
  harness matches it **case-insensitively** (collision-free), applied consistently everywhere
  `field_label` is used. It only makes labels clearer and cannot bias the REASON direction. (In-code
  comment at `rs22_probe.py:field_label`.)
- **Slice-based verdict.** The verdict rests on the 15-pair validation slice + 5 anchors + 5 posctrl
  + 3 negctrl (128 pinned calls), NOT the full 420 run. The full run was declined (human, 2026-07-17)
  because the three findings are structural — n=420 cannot flip INVALID→valid (F1 is anchor-level;
  F2 is a model-calibration property; F3 is a corpus property) — so ~2,300 further calls would only
  formalize the null.
- **Not run on the slice path** (would be needed only for a REASONING-CONFIRMED claim, which is
  precluded): the cross-family Mistral confirmatory arm, the mandatory Claude over-pruning audit,
  the analogy-strength κ rubric, the probe✗ false-miss adjudication, the full RS-21 embedding null.
  The lexical-only null used for the slice is **conservative-favorable to REASON** (a weaker null is
  easier to beat) and still shows d≈0 on memory-absent pairs, so the embedding null could only
  strengthen the INCONCLUSIVE.

## 6. Durable assets

1. **The 420-pair leakage-aware cross-field-analogy benchmark** (frozen, deterministic,
   provenance-complete) — the largest asset the chapter has produced.
2. **A reusable, blind-authored, total-function instrument harness** (probes + retrieval + judge +
   nulls + gate) for LLM analogy-memory/reasoning studies.
3. **F1 as a standalone result**: the brute-force-LLM analogy-retrieval bar is model-relative and
   rises sharply with capability (0.60 → ~1.00 from the EXP-RS-10 model to Opus 4.8) — directly
   relevant to how every RS-16→21 comparison-to-baseline should be read.

## 7. Reproduction

```
cd prototypes
.venv/bin/python rs22_probe.py emit-inputs --units slice:15   # + anchors/posctrl/negctrl
# dispatch phase-1 + judge via the Workflow fan-out (pinned Opus 4.8, fresh sessions)
.venv/bin/python rs22_probe.py score --units slice:15 --arm claude
.venv/bin/python rs22_probe.py score --units anchors --arm claude
.venv/bin/python rs22_gate.py --records data/rs22_clean_records_slice15_claude.json \
    --posctrl data/rs22_posctrl_slice.json --out data/rs22_verdict_slice.json
```
