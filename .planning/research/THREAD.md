# Thread state — Dynamical LBD (Gen-4)

*Layer-2 thread-state contract (vault: `wiki/analyses/research-routine-packs-spec.md`). Read by
the vault's `/cartographer` bridge in place of the retired `.gpd/` state. Keep current: same-day
update after every experiment run (vault: `wiki/meta/research-operating-manual.md`, "per
numerical experiment / run"). Vault mirror of the hypothesis rows: `wiki/meta/hypothesis-ledger.md`.*

## Hard core

The knowledge graph is a *dynamical system*; bridges *emerge* from generation, not static
scoring (Gen-4 LBD — vault: `wiki/concepts/dynamical-lbd.md`, the three acceptance criteria).

## Live hypotheses (mirror of vault hypothesis-ledger)

| id | statement | discriminating experiment | status |
|---|---|---|---|
| H-RS-substrate | Cellular **sheaves** over the Louvain community graph detect multi-causal bridges better than RAFs or Kuramoto | 4-tier benchmark incl. multi-causal joint-removal ablation on the shared 10-pair Feynman set | **FALSIFIED at the benchmark bar (Phase 34, EXP-RS-15, 2026-07-05).** On the valid testbed, sheaf frustration recovers 0/4 into the top-10 (benchmark pairs rank #69–218), T4 ablation FALSIFIED (0/5) — **tied with Kuramoto at recall@10 = 0.** Sheaves do NOT beat the bar; the "sheaves better" hypothesis fails on a fair test. (Sheaf's self-reported "precision@10=0.400" is a mislabeled full-list metric — not top-10.) Method-level kill criterion met for the two dynamical/spectral candidates. RAF (reaction model) untested. |
| H-RS-method | The dynamical-LBD pipeline (Kuramoto→Fiedler) has real cross-domain-bridge recovery signal when run on a well-posed citation graph containing both literatures | EXP-RS-14: per-pair recall@10 on the fully-valid bridged corpus vs 0.15 baseline, vs nulls | **FALSIFIED (Phase 33, 2026-07-04) — CLEAN, mechanistic.** On a corpus that is connected ∧ bridge-containing (4/4 pairs) ∧ synchronized (r=0.932) ∧ finely-partitioned (32 comms), recall@10 = 0.000; NO benchmark pair in the top-200 Fiedler bridges. Mechanism: single global Fiedler cut (side0=834/side1=564) puts ALL benchmark pairs on the SAME side → structurally invisible. Not a confound — every well-posedness condition met. |
| H-RS-analogy-SME (new chapter) | The cross-field analogy signal is **semantic-conceptual**, recoverable by structure-mapping (SME) over blind LLM-extracted **role-typed relational schemas**, beating the brute-force LLM baseline | EXP-RS-16: conditional-retrieval recall@10 vs the (now-run) brute-force baseline; roles-ON vs OFF; alignment vs ground-truth bridge_names | **FALSIFIED at the benchmark bar (Phase 35, 2026-07-06).** roles-ON recall@10 = 0.00 vs baseline 0.60; role-typing inverts (roles-ON < roles-OFF < lexical); alignment empty 3/5. Over-abstraction collapse on a physics-dense pool. The blind role-schema *representation* is too lossy — NOT that semantic-conceptual analogy is absent (the full-context LLM recovers it at 0.60). Next: less-lossy generators (#2 slot-frames / #4 mechanism-ontology). |
| H-RS-analogy-mechanism (new chapter, EXP-RS-17) | The cross-field analogy signal is recoverable by matching papers on a **shared rare mechanism archetype** from a *frozen, field-agnostic* ontology (MethMeSH), beating the brute-force LLM baseline — and, crucially, holding up on a **leakage-controlled modern held-out set** where the LLM baseline's pretraining advantage is neutralised | EXP-RS-17: C-19 conditional-retrieval recall@10 on Feynman (vs the 0.60 leaky bar + SME 0.00) AND a NEW modern held-out corpus (vs its own brute-force bar); cheap tagging-recall gate; IDF-on/off + λ ablations; shared-archetype artifact vs `bridge_names` | **FALSIFIED (exact-match form) at the cheap gate (Phase 36, 2026-07-07): Feynman tagging-recall = 1/5 < 3/5 → P2 falsified → KILL-by-construction.** Modern passed 4/6. Failure = exact-archetype-ID brittleness (neighboring archetypes), NOT absent representation. **Also FALSIFIED under similarity matching (EXP-RS-18, Phase 37, 2026-07-07): soft exact-or-adjacent gate = Feynman 2/5 < 3/5** (blind frozen adjacency, 124 nodes/427 edges). Adjacency lifts 1→2/5 but can't close the residual domain-knowledge-equivalence gap. **Mechanism-ontology line (#4, exact + soft) KILLED on the Feynman bar → fall back to slot-frames (#2).** Modern held-out 5/6 (its pairs share identical named mechanisms). Does not refute semantic-conceptual analogy broadly (LLM baseline 0.60). |
| H-RS-analogy-generative (EXP-RS-19) | A cross-field analogy = SAME method, DIFFERENT object; it is recoverable by GENERATING (from the query alone) hypothetical abstracts that re-express the query's method in other fields' native vocabulary, then retrieving real candidates against them (HyDE). Generation is an EXPANSION → converts latent equivalences (percolation≡epidemic) into retrievable tokens where pure-lexical splits self-kill | EXP-RS-19: C-19 conditional-retrieval recall@10 on Feynman (vs 0.60 bar + 0.40 lexical null) + modern held-out; cheap forward gate (GATE-A recall≥3/5 & recovers a null-missed pair; GATE-B pair04); auditable transfer card vs `bridge_names` | **PARTIAL-CONFIRM / headline KILLED (Phase 38, 2026-07-07). Mechanism PROVEN: pair04 recovered 17→4 (verified: epidemiology hypothetical matches the real epidemics paper, 0.15 vs null 0.06); K=1 ties LLM 0.60. But pinned headline (K=5 max-pool) = 0.20 < 0.40 null → GATE-A FAILS → KILL. Cause = max-pool distractor inflation (monotonic in K), NOT absent signal. Ceiling = TIE (as predicted). Next: generate→verify cascade #3.** |
| H-RS-analogy-cascade (EXP-RS-20) | A cross-field analogy = SAME method, DIFFERENT object; recoverable by a two-stage **generate→verify** cascade: keep HyDE's proven generation/recall stage (EXP-RS-19, frozen), then add a blind VERIFY stage (LLM/CAS) that audits each proposed transfer for method-coherence ∧ object-difference and prunes the max-pool distractors that killed the HyDE headline (K1 0.60 / K3 0.40 / K5 0.20) | EXP-RS-20: cheap forward gate FIRST (cascade fwd recall@10 > 0.40 null ∧ pair04 in top-10 ∧ ≥1 null-missed pair); then recall@{1,5,10}+MRR on Feynman (vs 0.60 bar) + modern held-out (vs C-35 bar / 0.833 null); auditable transfer card vs `bridge_names` | **PRE-REGISTERED — LOCKED 2026-07-15 (Phase 39). Predictions frozen (P1–P5 + GATE — see Active experiment). Provenance: pi/Mistral draft → Claude review (4 fixes) → 3-family orbiter panel (2 fixes). Expected outcome = PIVOT (bank auditable artifact + cheap retriever) or KILL→embedding-substrate escalation; ADVANCE unlikely (chapter ceiling = TIE at 0.60). RUN 2026-07-16 → KILLED at the cheap gate: Mistral-large full-175 headline cascade fwd recall@10 = 0.20 ≤ 0.40 null → P1 FAILS. recall@10 = 0.20 under ALL 3 C-40 pruning severities (headline/conservative/aggressive) — verify over-prunes true analogues: prune hard → kills 4/5 bridges, prune soft → reverts to HyDE-alone. Only pair04 (percolation→epidemics; PGF/branching-process) survives + rises 4→2 (both backbones; P5 card holds). Sonnet-5 ceiling = 0.40 (keeps 2/5) = ties null → still FAIL. Backbone method_coherence κ=0.45; Mistral systematically over-prunes (W-SYN synthesis weakness). Pre-registered KILL → lexical-intermediate line EXHAUSTED → embedding-substrate / pure-LLM-judge escalation.** |
| H-RS-analogy-embedding (EXP-RS-21) | The cross-field analogy signal — real (LLM 0.60) but invisible to lexical (TF-IDF) retrieval — is recoverable in a **dense semantic-embedding space**: whole-abstract embedding cosine places cross-vocabulary mechanism-analogues (percolation↔epidemic) near each other where surface tokens don't overlap. The single non-lexical substrate the chapter lacked. Sub-hypothesis (illustrative): a GENERAL contrastive embedding beats a citation-trained scientific one (SPECTER2) cross-domain | EXP-RS-21: swap the C-17 TF-IDF vec for dense `emb(title+abstract)`, hold C-14/C-24/C-19/bars fixed. P1 cheap FIRST = ∃ LIVE local model {bge,specter,gte} with strict-P1 (fwd recall@10>0.40 ∧ pair04 top-10 ∧ ≥1 null-missed); then M1=bge recall@{1,5,10}+MRR vs 0.60 bar + 0.40 null + random-chance null; modern = no-regression floor (0.833); objective P5 card (metadata + δ=0.02 margin + random control) | **PRE-REGISTERED — LOCKED 2026-07-16 (Phase 40). v2 predictions frozen (P0–P5 + 6-outcome total-function GATE — see Active experiment). Provenance: Claude design → 5-lens adversarial panel (needs-rework, 9 must + 7 should fixes) → all applied; GATE grid-verified total (`embed_gate_selftest.py`, 4608 cells). Manifest SHA `4dc8da72…`; all 3 local encoders toytest-live; SPECTER proximity adapter active. Honest framing = KILL-vs-TIE (effective n≈1 = pair04; 0.40 null ≈ chance); ADVANCE a stretch. Escalates OUT of lexical intermediates per the RS-20 gate. **RUN 2026-07-16 → KILLED (clean class-negative). NO dense embedding beats the 0.40 lexical null on the discriminating Feynman corpus: general open bge/gte = 0.20 (BELOW null — miss pair04), scientific specter2+proximity & mistral-embed = 0.40 (TIE null — recover the pair04 anchor rank 17→5/7 but LOSE the lexically-easy pair05, net zero). strict-P1 False for all → class↑ False → KILL. All 4 models LIVE (toytest + modern≥0.5) → KILL from working encoders. Mechanism = topical/field dominance (pre-registered Open-Risk-1); objective P5 card refuses to certify pair04 (random cross-field control ~0.81 ≫ 0.30 → margin not distinctive). P3 sub-hypothesis REFUTED (opposite: citation-trained SPECTER recovers pair04, general models don't; n≈1 illustrative). Static geometry ≈ chance-lexical; only full-context LLM reasoning = 0.60. → escalate to a pure LLM-judge cascade that STRUCTURALLY differs from C-20, ELSE terminal chapter verdict.** |
| H-RS-benchmark-leakage (EXP-RS-22) | Is the incumbent 0.60 brute-force LLM baseline — the bar RS-16→21 all failed — real cross-domain REASONING, or MEMORIZATION of famous analogies? (one-directional, honest reach: two panels showed memorization is near-non-identifiable at feasible n, so v3 tests only the falsifiable direction — is there a real reasoning component?) | EXP-RS-22 v3: on pairs the pinned model fails BOTH free-recall AND recognition (memory-absent), does retrieval beat max(lexical, RS-21 embedding) null after removing paper-familiarity + priming? `REASON` = paired Wilcoxon (one-sided > δ=5pt) on ≥110 clean-stratum pairs (K=50), blind-authored constants (SHA `af5ee11c…`); memory-isolating scorer (target_field vs side_b, query-vocab subtracted, semantic judge) | **PRE-REGISTERED — LOCKED 2026-07-17 (Phase 41, v3). Build IN PROGRESS.** Provenance: human critique H-CORRECT-03 → v1 fame-gradient (panel KILL: obscurity≡reasoning-difficulty confound) → v2 direct-probe (panel KILL: confound RELOCATED + recall≠recognition + scorer measured query-paraphrase; memorization non-identifiable) → v3 one-directional (all high-value fixes + blind-authored constants). Outcomes = REASONING-CONFIRMED (chapter CLOSES, KILLs airtight) or INCONCLUSIVE; **memorization/REOPEN not falsifiable here**. Needs a NEW larger leakage-aware benchmark (~several hundred mined → 110 memory-absent) — a durable asset that also re-powers the chronically-n≈5 chapter. Orbiter: pi/Mistral mines/pre-filters/stats; Claude does validity-judgment/probe-gen/semantic-judge/audit; blind subagent authors constants + final adjudication. `.planning/phases/41-benchmark-validity/41-PREREG.md`. |

## Kill criteria

- **Method-level:** sheaf near-section frustration does not beat the brute-force baseline
  (vault: `wiki/concepts/brute-force-lbd-baseline.md`) on held-out bridges.
- **Pivot gate (time-bound, set 2026-07-02, human-approved):** if the Path C TF-IDF
  semantic-edge substrate (EXP-RS-11) yields **<3 evaluable Feynman pairs** or
  **`BENCH_P10 ≤ 0.15`** on the shared 10-pair set by **2026-09-30**, kill the
  dynamical-substrate line and revert to the brute-force baseline.

## Current claims

| claim | status | evidence |
|---|---|---|
| Dynamical-LBD on the pre-2015 cond-mat *citation* graph is empirically infeasible (~41 components / 153 nodes → `K_stable` bisection diverges) | verified (Phase 29 FAIL, 2026-05-05) | `.planning/phases/29-kuramoto-corpus-build/29-VERIFICATION.md` |
| Sheaf near-section frustration ranks bridges on this corpus | HOLD — untestable on VOID corpus (T2 precision@10 = 0.000) | `prototypes/SHEAF_V01_RESULTS.md` |
| TF-IDF cosine edges (τ=0.3) make the same corpus connected enough for spectral/dynamical LBD (`n_cc/N ≤ 0.05`, largest CC ≥ 80%) | **FALSIFIED** (Phase 30 FAIL, 2026-07-04) — actual `n_cc/N`=0.830, largest CC=3.3% at τ=0.3; *more* fragmented than the citation graph (0.268) at every pre-registered τ. Confirmed by 3 independent recomputes. | `.planning/phases/30-tfidf-semantic-edge-graph/30-VERIFICATION.md` |
| ~~The pre-2015 cond-mat corpus (N=153) is too narrow to support *any* substrate for dynamical LBD~~ | **RETRACTED (2026-07-04)** — this conflated the *pre-2015 slice* with the *corpus*. The FULL corpus citation graph is well-posed (227→giant CC 224, 1 component, n_cc/N=0.009). The fragmentation was caused ENTIRELY by the C-1 pre-2015 slice, which is not required by the date-agnostic BENCH_P10 recovery metric. Not "corpus too narrow" — "temporal slice unnecessary and harmful." | full-corpus connectivity check 2026-07-04; see EXP-RS-12 provenance |
| Phases 29/30 non-results were corpus/methodology *connectivity* artifacts (the pre-2015 slice) — CONFIRMED: the full-corpus giant CC is well-posed, K_stable=14.25 converges | **verified** (Phase 31 EXP-RS-12, 2026-07-04) | `.planning/phases/31-dynamical-lbd-giant-cc/31-VERIFICATION.md` |
| The dynamical method recovers Feynman bridges (BENCH_P10 > 0.15) on the well-posed giant CC | **FALSIFIED but test not fair** (Phase 31: BENCH_P10=0.000) — decisive diagnostic: 3/4 evaluable pairs have ZERO inter-community citation edges → the corpus lacks the bridge literature the method is scored on; the 1 pair with a 2-edge bridge (pair04) is diluted out of the global-top-10. Corpus-CONTENT gap now isolated from the (solved) connectivity gap. | 31-VERIFICATION.md § "decisive diagnostic" |
| The brute-force LLM baseline (EXP-RS-10) recovers cross-domain analogies on the valid testbed | **verified** (Phase 35, 2026-07-06) — conditional-retrieval recall@10 = **0.60**, MRR 0.63 (3/5 pairs rank side_b #1). Job zero established; this is the bar for all future generators. Caveat: pretraining-leakage-inflated (not corrected). | `.planning/phases/35-sme-vs-baseline/35-VERIFICATION.md` |
| SME over blind, role-typed relational schemas beats the brute-force baseline / role-typing carries the analogy signal | **FALSIFIED** (Phase 35 EXP-RS-16, 2026-07-06) — roles-ON recall@10 = **0.00** vs baseline 0.60; role-typing *inverts* (roles-ON 0.00 < roles-OFF 0.20 < lexical 0.40); alignment tables empty 3/5. Over-abstraction collapse: closed role vocab maps every network-physics paper onto one skeleton (51% of pairs score 0). Blind schema bottleneck discards content the full-context LLM keeps. Both KILL conditions fired. | 35-VERIFICATION.md |
| Mechanism-ontology tagging (MethMeSH, exact-archetype-ID overlap) represents cross-domain bridges well enough to beat the baseline | **FALSIFIED at the cheap gate** (Phase 36 EXP-RS-17, 2026-07-07) — Feynman tagging-recall = 1/5 (only pair03 shares an archetype) < gate 3/5 → P2 falsified → KILL-by-construction before any full eval. Failure = exact-ID granularity brittleness: analogous papers get *neighboring* non-identical archetypes (pair04 percolation: giant-connected-component/birth-death-branching vs compartmental-flow/simple-contagion). Modern held-out passed 4/6. | `36-methmesh-vs-baseline/36-VERIFICATION.md` |
| Archetype-SIMILARITY matching (blind frozen adjacency graph) rescues mechanism-ontology LBD where exact-ID failed | **FALSIFIED at the soft gate** (Phase 37 EXP-RS-18, 2026-07-07) — soft exact-or-adjacent gate = Feynman 2/5 < 3/5 (adjacency lifts 1→2/5 but not past the gate); modern 5/6. P1 falsified → KILL the mechanism-ontology line (#4, both forms). Residual gap: cross-domain bridges (percolation≈epidemic, reaction-diffusion≈economy) need domain-knowledge equivalences invisible from field-agnostic glosses; blind tagger applies generic archetypes to the non-physics side. | `37-methmesh-soft/37-VERIFICATION.md` |
| Generation-based cross-field retrieval (HyDE) recovers cross-vocabulary analogies the lexical comparator misses | **PARTIALLY CONFIRMED** (Phase 38 EXP-RS-19, 2026-07-07) — VERIFIED: a blind epidemiology hypothetical generated from a percolation paper matches the real epidemics paper (cos 0.15 vs null 0.06 via transmission/epidemics tokens) → pair04 recovered rank 17→4; K=1 ablation ties the LLM baseline (0.60). But the PINNED headline (K=5 max-pool) = 0.20 < 0.40 null → GATE-A fails → KILL; cause = max-pool distractor inflation (monotonic: K1 0.60 / K3 0.40 / K5 0.20), NOT absent signal. Ceiling = TIE (as predicted). | `38-hyde-bridge/38-VERIFICATION.md` |
| A generate→VERIFY cascade (HyDE recall + blind LLM verify pruning) rescues the HyDE aggregation past the lexical null | **FALSIFIED at the cheap gate** (Phase 39 EXP-RS-20, 2026-07-16) — Mistral-large full-175 headline cascade fwd recall@10 = **0.20 ≤ 0.40 null** → P1 FAILS → KILL; **0.20 under all 3 C-40 pruning severities**. Verify OVER-PRUNES true analogues (method-incoherent on 4/5; only pair04 survives, rising 4→2). Sonnet-5 ceiling = 0.40 (keeps 2/5) = ties null → still FAIL. Lexical-intermediate line exhausted → escalate to embedding substrate. First orbiter-migration backbone head-to-head: method_coherence κ=0.45; Mistral over-prunes (W-SYN). | `prototypes/verify_results_feynman_llm.json`, `prototypes/data/verify_compare_feynman.json` |
| A dense semantic-embedding substrate (whole-abstract cosine) recovers cross-domain analogy where TF-IDF cannot | **FALSIFIED — clean class-negative** (Phase 40 EXP-RS-21, 2026-07-16) — NO embedding beats the 0.40 lexical null on Feynman fwd: bge 0.20 / gte 0.20 (general open, BELOW null) / specter2+proximity 0.40 / mistral-embed 0.40 (TIE null). strict-P1 False ∀ 4 LIVE models → class↑ False → KILL (grid-verified total-function gate). Mechanism = topical/field dominance (objective P5 card: pair04 margin not distinctive, random control ~0.81). One real-but-insufficient signal: scientific embeddings pull pair04 rank 17→5/7 (recover the anchor the null misses) but lose lexically-easy pair05, net zero. P3 REFUTED (citation-training HELPS cross-domain, opposite of hypothesis; n≈1). Both static-representation routes (lexical + dense-embedding) now fail 0.60 → only full-context LLM reasoning recovers it. | `.planning/phases/40-embedding-substrate/40-VERIFICATION.md`, `prototypes/data/embed_verdict.json` |

## Active experiment

**EXP-RS-22 → Phase 41 — Benchmark Validity (v3, lean one-directional). LOCKED 2026-07-17. BUILD IN
PROGRESS.** Does the incumbent 0.60 LLM baseline (the bar RS-16→21 all failed) contain a real REASONING
component (→ chapter CLOSES, KILLs airtight) or is it INCONCLUSIVE? **Memorization / chapter-REOPEN is NOT
falsifiable at feasible n** (two adversarial panels proved it near-non-identifiable — you can't cleanly
prove a null), so v3 tests only the falsifiable direction. Design arc: v1 fame-gradient (panel KILL:
obscurity≡reasoning-difficulty confound) → v2 direct-probe (panel KILL: confound RELOCATED + recall≠
recognition + scorer measured query-paraphrase) → **v3 (this)**. Instruments: two-format memory probe
(free-recall + recognition) ⟂ retrieval (independent sessions, hashed inputs); memory-isolating scorer
(score `target_field` vs side_b's held-out field, side_a vocab subtracted, blind semantic judge — fixes
the v2 killer); `REASON` = paired Wilcoxon (one-sided > δ) on `pctile_rank_LLM − max(lexical, RS-21
embedding)-null` over the **≥110 clean (memory-absent) pairs**. Blind-authored frozen constants (C-47, SHA
`af5ee11c…`: K=50, δ=5pt, α=0.05, n_floor=110, repro band [0.50,0.90], κ≥0.60) — the staked orchestrator
does NOT set the decision boundary. Requires a **NEW larger leakage-aware benchmark** (~several hundred
mined → 110 memory-absent) — a durable asset that also re-powers the chronically n≈5 chapter. **Orbiter
build (human directive):** pi/Mistral mines/pre-filters/stats vs the frozen rubric; Claude does the
synthesis-tier mining validity-judgment + probe-generation + semantic-judging + the MANDATORY
over-pruning audit; a blind subagent authored the constants + does final gate adjudication. Conventions
C-46..C-48. **► BUILD STATE (2026-07-17, MID-BUILD — resume point for a fresh session):**
**DONE:** blind constants frozen (`prototypes/data/rs22_constants.json`, SHA `af5ee11c…`, n_floor=110);
mining protocol + 6 instrument/judge prompts frozen (C-46, `prototypes/rs22_{mining_protocol,probe_recall,
probe_recognition,probe_familiarity,probe_openbook,judge_semantic}.md` + `.sha256`); gate harness
(`rs22_gate.py` + `rs22_gate_selftest.py`, total-function verified); MINER built + live-validated
(`rs22_mine.py` — arXiv `abs:"<phrase>"` discovery + Semantic-Scholar refs + arXiv metadata; OpenAlex
DROPPED, was 429-stalling the long run). Commits through `dab8005`+speed-fix.
**RUNNING NOW (detached `setsid nohup`, ~70–80 min):** the full mine → `prototypes/data/rs22_mined_pairs.json`
(checkpoints every 20 pairs), log `prototypes/rs22_mine_run.log`. It is a standalone OS process,
independent of any Claude session.
**RESUME HERE:** (1) confirm the mine finished (~420 pairs in `rs22_mined_pairs.json`); `sha256sum` it +
`rs22_mining_snapshot.json`, record in THREAD, commit `data(exp-rs-22): freeze mined corpus`. (2) **Task 13
— instruments:** run the 5 blind instruments (recall + recognition + familiarity + open-book on the pinned
Opus 4.8 + a cross-family Mistral fan-out via the orbiter; probes ⟂ retrieval — independent fresh sessions,
hashed inputs), memory-isolating scoring (score `target_field` vs side_b via `rs22_judge_semantic.md`),
build the ≥110 clean (fails-both-memory-probes) stratum, deterministic B8+ expansion if short. (3) **Task 14
— verdict:** `rs22_gate.py` → REASONING-CONFIRMED (chapter CLOSES) / INCONCLUSIVE → `41-VERIFICATION.md`.
All frozen; constants blind-authored; gate grid-verified ⇒ the analysis is mechanical + unsteerable. Full
record: `.planning/phases/41-benchmark-validity/41-PREREG.md`.

---

### (just-run, KILLED 2026-07-16) EXP-RS-21 → Phase 40 — Dense Embedding Substrate

**EXP-RS-21 → Phase 40 — Dense Embedding Substrate. LOCKED 2026-07-16 (v2). The first escalation OUT of
lexical intermediates (per the RS-20 gate). RUN 2026-07-16 → KILLED (► RESULT below).** Provenance: Claude design → 5-lens adversarial
panel (gate-band / leakage-forking / metric-stats / ML-IR / kill-integrity → synthesis; verdict
**needs-rework**, 9 must-fixes + 7 should-fixes) → all applied in v2; GATE rebuilt as a grid-verified
TOTAL function (`prototypes/embed_gate_selftest.py`, 4608 cells → exactly one of 6 verdicts). Full
record: `.planning/phases/40-embedding-substrate/40-PREREG.md`.

**Hypothesis (H-RS-analogy-embedding):** the analogy signal — real (LLM 0.60) but invisible to TF-IDF —
is recoverable in a **dense semantic-embedding space** (whole-abstract cosine places percolation↔epidemic
near each other where surface tokens don't overlap). The single non-lexical substrate the chapter lacked.

**Design (LOCKED — C-41..C-45).** ONE change: swap the C-17 TF-IDF vec for `emb(title+abstract)`; hold
C-14 Feynman + C-24 modern + C-19 metric + all bars/nulls fixed (apples-to-apples with every RS-16..20
arm). Frozen model set (manifest SHA `4dc8da72…`): **M1 bge-large-en-v1.5 (HEADLINE, symmetric — no
instruction, MTEB-designated)**, M2 specter2+proximity (scientific/citation P3 contrast, native [SEP]),
M4 gte-large (robustness), **M3 mistral-embed (EU, DESCRIPTIVE-ONLY — excluded from the KILL vote)**.
`class↑` vote = reproducible LOCAL {M1,M2,M4} that are LIVE (per-model toytest ∧ modern≥0.5). All three
local encoders toytest-green; SPECTER proximity adapter asserted active. Harness: `embed_score.py` (reuses
`sme_lite.{rank_candidates,eval_direction}` verbatim) + `embed_gate.py` (total-function verdict).

**Predictions (LOCKED v2):**
- **P0 (job-zero, BLIND):** compute+freeze the C-35 modern LLM baseline (`baseline_results_modern.json`)
  via a FRESH BLIND subagent (C-20 procedure, sees only {id,title,abstract}×35) BEFORE any modern
  embedding scoring. Modern = no-regression FLOOR (0.833); NOT a strict-beat (C-44).
- **P1 (cheap class gate — run FIRST, Feynman fwd):** ∃ LIVE local model with strict-P1 = [fwd recall@10
  > 0.40 ∧ pair04 in its top-10 ∧ ≥1 null-missed (01/04/06) recovered]. FAIL → clean class-KILL.
- **P2 (bar):** M1 Feynman recall@10 ≥ 0.60 WITH pair04 (= strict-P1(M1)) ∧ modern ≥ 0.833.
- **P3 (ILLUSTRATIVE, underpowered — NOT inferred):** general (M1/M4) recover more null-missed pairs than
  SPECTER2 (M2); with effective n≈1 this is a per-pair anecdote, reported with its n.
- **P4:** M1's pair04 recovery corroborated by ≥1 other general model (M4/M3). M1↔M4 disagreement caps
  the verdict at PIVOT.
- **P5 (OBJECTIVE, co-primary):** pair04 card passes iff cross-domain metadata ∧ true-pair cosine >
  same-field distractor median + δ(0.02) ∧ random control < 0.30. Computed in code.

**GATE (6-outcome TOTAL function, grid-verified; MUST-1/2/3):** `¬LIVE(M1)`→INVALID; `headlinePass ∧
modern<0.833`→WEAK-no-bank; `headlinePass ∧ R≥0.80 ∧ P4 ∧ P5obj`→**ADVANCE**; `headlinePass else`
(R=0.60 tie / missing P4-P5)→**PIVOT** (bank card + cheap retriever; next = embedding→LLM re-rank);
`¬headlinePass ∧ class↑ ∧ winner-modern≥0.833`→**WEAK-PIVOT**; `¬headlinePass ∧ ¬class↑`→**KILL**
(static geometry adds nothing beyond the ~chance lexical null → pure LLM-judge cascade that STRUCTURALLY
differs from the C-20 one-shot 35-way ranking; else terminal chapter verdict). **Honest framing =
KILL-vs-TIE**: effective n≈1 (pair04), 0.40 null ≈ chance (binomial P(R≥0.40)=0.44); ADVANCE (needs
dense to recover a pair the LLM cannot) a live-but-low-probability stretch. Random-chance null reported.

**Ablations (after P1):** BGE-asymmetric (`--directional`); reverse + both-dir; text-field title/abstract-only.

**► RESULT (2026-07-16) — EXP-RS-21 KILLED (clean class-negative).** P1 Feynman class gate: **NO dense
embedding beats the 0.40 lexical null forward** — bge 0.20 / gte 0.20 (general open, BELOW null, miss
pair04 rank 20/18) / specter2+proximity 0.40 / mistral-embed 0.40 (scientific & API, TIE null: recover
the pair04 anchor rank 17→5/7 but lose the lexically-easy pair05, net zero). strict-P1 False ∀ → class↑
False → KILL (all 4 LIVE via toytest + modern≥0.5, so from working encoders; `embed_gate.py`, total-
function verdict). **Mechanism = topical/field dominance** (pre-registered Open-Risk-1): the objective P5
card (C-45) refuses to certify pair04 — its cosine margin over the same-field median is positive but the
random cross-field control passes ~0.81 ≫ 0.30 (not distinctive). **P3 sub-hypothesis REFUTED** (opposite
held: citation-trained SPECTER + mistral recover pair04; general bge/gte don't; n≈1 illustrative). Modern
(non-discriminating guard, null 0.833): bge 0.67 / gte,specter,mistral 0.83 — confirms liveness. Blind
C-35 (P0) + ablations DEFERRED (Feynman gate failed first — the C-35 "only if gate passes" lineage, as
RS-19). **Both static-representation routes now fail the 0.60 bar: lexical (RS-16→20) AND dense-embedding
(RS-21). Static geometry ≈ chance-lexical; the analogy signal is recoverable ONLY by full-context LLM
reasoning (C-20 = 0.60).** Full record: `.planning/phases/40-embedding-substrate/40-VERIFICATION.md`,
`prototypes/data/embed_{results_feynman,results_modern,verdict}.json`.
**► NEXT SESSION START HERE:** the remaining pre-registered escalation is a **pure LLM-judge cascade** —
but it MUST structurally differ from the C-20 one-shot 35-way ranking (already the 0.60 incumbent): e.g.
pairwise/tournament judging, or scaling the pool beyond 36 papers. **Terminal-verdict caveat (honest):**
if no structurally-different LLM-judge can be built that beats 0.60, EXP-RS-21's KILL is the **terminal
chapter verdict** — the practical LBD method IS the brute-force LLM baseline (0.60), and the research
contribution is the sharp, publishable separation "no static representation (lexical OR dense-embedding)
recovers cross-domain mechanism analogy; only LLM reasoning does." Human go/kill/pivot decision.
Durable residue: deterministic embedding+gate harness; the counter-intuitive "scientific embeddings beat
general ones cross-domain" finding (worth a follow-up if the cascade runs).

---

### (just-run, KILLED 2026-07-16) EXP-RS-20 → Phase 39 — Generate→Verify Cascade (#3)

**RUN 2026-07-16 → KILLED at the cheap gate (P1 FAIL).** First orbiter-migration backbone head-to-head
(Mistral-large executor 175/175 vs Sonnet-5 Claude overseer). **Mistral headline cascade fwd recall@10 =
0.20 ≤ 0.40 null → KILL; 0.20 under ALL 3 C-40 pruning severities** — verify OVER-PRUNES true analogues
(prune hard → kills 4/5 bridges; prune soft → reverts to HyDE-alone). Only pair04 survives + rises 4→2
(both backbones name PGF/branching-process; P5 card holds — the one durable artifact). Sonnet ceiling =
0.40 (keeps 2/5) = ties null → still FAIL. method_coherence κ=0.45; Mistral over-prunes (W-SYN, invisible
to self-report). This EXHAUSTED the lexical-intermediate line (RS-16→20 all fail the 0.60 bar) → the
pre-registered escalation OUT of lexical intermediates = EXP-RS-21 (above). See [[project_orbiter_migration]]
+ [[project_orbiter_experiment_loop]] (memory) for the executor/overseer loop. Full record:
`.planning/phases/39-generate-verify-cascade/39-PREREG.md`, `prototypes/verify_results_feynman_llm.json`,
`verify_compare_feynman.json`.

---

### Prior (concluded) — EXP-RS-19 → Phase 38 — HyDE-Bridge (#2 slot-frames, artifact-primary). RUN 2026-07-07 — pinned
headline KILLED at the cheap gate, but the generation MECHANISM is PROVEN (pair04 recovered; K=1
ties 0.60). Forward = human's go/kill/pivot.** Human directed starting #2 (2026-07-07, ultracode).
Design chosen by a **21-agent judge panel**

> **RESULT (2026-07-07):** Blind HyDE prompt (hashed) + 5 blind Feynman side_a generations (hashed
> before scoring). Cheap gate (C-34, forward, full 36-candidate retrieval): headline (K=5,max,λ=0)
> forward **recall@10 = 0.20 — BELOW the 0.40 lexical null → GATE-A FAILS → KILL**. BUT **GATE-B
> PASSES**: pair04 (percolation→epidemics) recovered rank **17→4**, and the mechanism is VERIFIED —
> the blind epidemiology hypothetical matches the real "spread of epidemic disease on networks" paper
> at cos 0.1546 (null 0.0586) via *transmission/epidemics/spread* tokens. Cause of the KILL = **max-pool
> distractor inflation** (residual risk #4), monotonic in K: **K=1→0.60, K=3→0.40, K=5→0.20**. The
> pre-registered K=1 ablation TIES the LLM baseline (0.60, same pairs 03/04/05) — the design's honest
> TIE ceiling — but K=1 is a non-promotable descriptive ablation (C-36 pin). pair01/06 unrecovered by
> anyone (LLM too). So: the *pinned aggregation* failed, NOT the generation idea — signal survives,
> violating C-37's "no signal" KILL premise. Both C-37's escalation branch (LLM-judge cascade) and the
> design's PIVOT (verify stage) converge → **generate→VERIFY cascade (#3)** is the best next move.
> Full record: `.planning/phases/38-hyde-bridge/38-VERIFICATION.md`. **Forward = human's call:**
> (rec.) build the generate→verify cascade #3 (HyDE recall stage + LLM/CAS verify) · or escalate to a
> real embedding substrate · or a fresh EXP-RS-20 with distractor-robust aggregation (K=1 / rank-fusion). (5 independent designs × 3 adversarial lenses → synthesis;
`workflows/scripts/design-exp-rs-19-slotframes-*`). The panel proved *from the data* that every
PURE-LEXICAL method/object split SELF-KILLS: true-pair whole-abstract cosines are near-zero for the 4
hard Feynman pairs (a token partition can't manufacture shared-method tokens ABSENT from the text →
caps at the 0.40 lexical null, below the 0.60 bar). Only GENERATION converts a latent cross-field
equivalence (percolation≡epidemic) into retrievable tokens → **HyDE is the spine.**

**Hypothesis:** a cross-field analogy = SAME method, DIFFERENT object; the bridge is recoverable by
generating, from the query alone, hypothetical abstracts that re-express its method in OTHER fields'
native vocabulary and retrieving real candidates against them. This is an **EXPANSION** (richer than
the input) → it structurally CANNOT repeat the RS-16/17/18 masking-loss (no skeleton to collapse onto,
no closed vocab to be brittle about; worst case degrades to the C-17 null, caught by the gate).

**Design (LOCKED — full convention text C-31..C-37 in CONVENTIONS.md):**
- **Blind HyDE generation (C-31/C-32).** One blind subagent per QUERY paper sees ONLY {arxiv_id,
  title, abstract} + a frozen benchmark-agnostic `hyde_prompt.md` (authored by a no-benchmark
  subagent, non-benchmark examples, SHA-256 committed before generation). Emits {method_core,
  query_object, hypotheticals:[{target_field, generic_object, abstract (150–250w native target-field
  vocab, NO query domain nouns, no real-paper names)} × K=5 distinct fields]}. Candidates are NEVER
  extracted — they keep verbatim C-17 whole-abstract TF-IDF (free). Generated files SHA-256-frozen
  before scoring ⇒ deterministic scoring on the frozen generations.
- **Scoring (C-33).** `score(q,c) = hyde_sim(q,c) − λ·object_sim(q,c)`; `hyde_sim = max_{k≤5}
  cos(vec(h_k), tfidf(c))`; `object_sim = cos(tfidf(q), tfidf(c))` = the C-17 null. Constants:
  **λ=0 (headline)**, K=5, POOL=max, IDF=C-17 per-corpus, tokenizer=sme_lite, tie-break lexicographic
  (C-19). Reuse sme_lite verbatim; new `hyde_score.py`.
- **Corpora/eval (C-35/C-36).** Feynman MVP (C-14) + modern held-out (C-24). C-19 conditional
  retrieval, forward primary. Baselines: Feynman 0.60 leaky bar + SME 0.00 + MethMeSH KILLs; **the
  modern brute-force bar (C-35 job-zero: C-20 on modern, frozen BEFORE any modern HyDE scoring)**; the
  C-17 lexical null (**VERIFIED 2026-07-07:** Feynman fwd recall@10 = **0.40** [gets pair03 r2/pair05
  r7, misses pair01 r31/pair04 r17/pair06 r25]; modern = **0.833** [misses only m06 r27]). Ablations
  {λ=0.5, mean-pool, K∈{1,3}, title-only} are DESCRIPTIVE, never promotable (C-36 headline pin).
- **Cheap gate (C-34), run FIRST — forward-only, 5 Feynman side_a generations, candidates free ⇒ a
  FULL 36-candidate retrieval against the 26 REAL distractors with the real eval IDF** (fixes every
  prior endpoint-only gate's blind spot to tie-with-distractors). **GATE-A:** headline HyDE forward
  recall@10 ≥ 3/5 AND the top-10 recovers ≥1 pair the C-17 null MISSES (pair01/04/06). **GATE-B:**
  pair04 (percolation→epidemics) recovered into forward top-10 — the decisive mechanism proof (its
  side_b literally contains SIR tokens a blind epidemiology hypothetical emits). KILL iff GATE-A OR
  GATE-B fails, before any modern/reverse/ablation spend.
- **Auditable transfer card (C-36 — THE primary deliverable).** Per recovered pair: {method_core,
  query_object → winning target_field/generic_object, shared-method & differing-object tokens}, scored
  vs `bridge_names` by the PRE-REGISTERED objective rule (match iff ≥1 hyphen-split bridge_name
  content-token ∈ method_core∪generic_object; a random bridge_name control must NOT match). A
  deliverable the 0.60 brute-force ranker structurally never produces.

**LOCKED PREDICTIONS (before any generation):**
- **P1 (cheap gate / #2 premise — decisive):** headline HyDE forward recall@10 ≥ 3/5 on Feynman AND
  recovers ≥1 null-missed pair, with pair04 specifically in top-10 (GATE-A ∧ GATE-B). FAIL → KILL
  before any modern/reverse/ablation spend.
- **P2 (beats the lexical floor — the load-bearing generation win):** HyDE forward recall@10 > 0.40
  (C-17 null) on Feynman — recovers ≥1 cross-vocabulary pair the null cannot see. If it can't beat
  free lexical retrieval, generation added only noise → HyDE premise refuted.
- **P3 (the bar — HONEST TIE, not beat):** HyDE Feynman recall@10 ≥ 0.60 (tie-or-narrow-beat the
  leaky LLM) AND modern recall@10 ≥ the frozen modern bar AND ≥ 0.833 (NO regression vs the modern
  null). Honest admission: a strict Feynman beat (≥0.80) needs pair01 (diffuse review side_b — coin
  flip) or pair06 (Turing→Zipf, which even the LLM misses); modal outcome = a TIE at 0.60, so the
  recall number is NOT the deliverable.
- **P4 (object-penalty is corpus-interacting, reported not tuned):** λ=0.5 ≥ λ=0 only where same-field
  distractors crowd the true pair (Feynman pair01/04); λ=0 > λ=0.5 on modern (its true pairs are
  SAME-object → the penalty demotes them). One frozen headline can't want the penalty ON and OFF, so
  it's OFF; the differential is a finding, never a per-corpus tune.
- **P5 (auditable artifact — THE PRIMARY DELIVERABLE):** for ≥3/5 recovered Feynman pairs the winning
  hypothetical's {method_core ∪ generic_object} tokens match `bridge_names` under the C-36 objective
  rule (pair04→threshold/connectivity via the epidemiology hypothetical; pair03→compartmental/
  network-diffusion; pair05→coupled-rate-equation), while a random bridge_name control does NOT match.
**GATE:** **ADVANCE** iff STRICT DOUBLE-BEAT — HyDE headline strictly beats BOTH the Feynman 0.60 bar
(≥0.80) AND the frozen modern bar → generation-based cross-field retrieval is a genuine generator;
formalize + scale + build the generate→verify cascade. **PIVOT (pre-registered EXPECTED outcome)** iff
HyDE TIES the incumbent (Feynman ~0.60; modern ≥ frozen bar ∧ ≥ 0.833) AND the C-36 card rule holds
≥3/5 → bank the auditable transfer card + cheap O(#queries) first-stage retriever; next build = a
VERIFY stage (LLM/CAS audits each card) → generate→verify cascade (#3). **KILL** iff GATE-A OR GATE-B
fails → even the strongest generative #2 has no bridging signal surviving the lexical comparator here
→ the lexical-retrieval line is exhausted; escalate OUT of lexical intermediates (real
semantic-embedding substrate or pure LLM-judge cascade), do NOT run another TF-IDF variant. Reuses
C-14, C-17, C-19, C-20, C-24.

### (just-run, KILLED 2026-07-07) EXP-RS-18 → Phase 37 — MethMeSH-Soft (archetype-similarity)

**EXP-RS-18 → Phase 37 — MethMeSH-Soft: archetype-SIMILARITY scoring over the same blind tags.
RUN 2026-07-07 — KILL (soft gate fired; P1 FALSIFIED). This KILLS the mechanism-ontology line (#4,
both exact + soft) → fall back to slot-frames (#2, pre-approved).** Human-directed (2026-07-07)

> **RESULT (2026-07-07):** Blind frozen archetype-adjacency graph (124 nodes / 427 edges, SHA-256
> `620d9c0f…`, built by a no-benchmark subagent, committed unmodified). Soft gate (C-29,
> exact-OR-adjacent) on the existing 22 tags: **Feynman 2/5 (up from exact 1/5) — FAIL (≥3/5)**,
> modern 5/6 (up from 4/6) — PASS. **P1 FALSIFIED → KILL the mechanism-archetype line.** Adjacency
> helped (pair01 now links hub-spreading~contagion; m03 links random-matrix~spectral) but can't close
> the residual gap: the remaining Feynman bridges (percolation≈epidemic, reaction-diffusion≈economy)
> need *domain-knowledge* equivalences invisible from field-agnostic glosses, and the blind tagger
> applies *generic* archetypes to the non-physics (economics) side. Two clean experiments (exact +
> soft) now refute #4 on the Feynman bar. Modern 5/6 persists (its pairs share *identical* named
> mechanisms — less cross-domain in the archetype sense). Does NOT refute semantic-conceptual analogy
> broadly (LLM baseline still 0.60). Full record: `.planning/phases/37-methmesh-soft/37-VERIFICATION.md`.
> **Forward: slot-frames (#2), human-pre-approved — the go to start it is the human's.**
after EXP-RS-17's KILL diagnosed the failure as exact-ID brittleness — the archetype *representation*
is correct (`turing-instability` WAS tagged on the Turing paper) but analogous papers get
*neighboring, non-identical* archetypes ⇒ exact-ID cosine 0. EXP-RS-18 keeps everything that worked
(frozen vocab C-22, blind tags C-23, both corpora) and changes ONLY the matching rule: exact-ID
overlap → soft overlap over a **blind, frozen archetype-adjacency graph**. This is a DISTINCT
hypothesis with its own gate, NOT a re-tune of EXP-RS-17 (whose KILL stands).

**Design (LOCKED — full convention text C-27..C-30 in CONVENTIONS.md):**
- **Frozen archetype adjacency (C-27, leakage control).** A BLIND subagent (NO benchmark/ground-truth
  access) links each of the 125 archetypes to its mechanistically-adjacent neighbors (closely-related
  / overlapping mechanism) using ONLY the vocab glosses — capped degree ~6, symmetric, one-line
  rationale per edge. FROZEN: SHA-256 recorded + committed BEFORE any scoring. Justifiable from the
  glosses alone, never the benchmark (orchestrator has seen `bridge_names` → delegated blind, as C-22).
- **Soft signature + scoring (C-28).** Neighbor-expanded signature `sig*(p)[a] = max over t∈tags(p)
  of w_t·sim(a,t)`, `sim(a,t)=1` if a==t, `=s` (decay s=0.5) if a adjacent to t, else 0; `w_t`=IDF.
  `score(i,j)=cos(sig*_i,sig*_j) − λ·cos(tfidf_i,tfidf_j)`, λ=1.0. Ablations: hard-neighbor s=1.0;
  exact-only s=0 (recovers EXP-RS-17); IDF on/off.
- **Corpora / eval / baseline:** unchanged — Feynman MVP (C-14) + modern held-out (C-24); C-19
  conditional retrieval recall@{1,5,10}+MRR; baselines = the 0.60 Feynman bar + the modern brute-force
  bar (C-20, run on the modern corpus).
- **Cheap soft gate (C-29), run FIRST on the EXISTING 22 endpoint tags:** soft tagging-recall =
  fraction of pairs whose two sides share an exact-OR-adjacent archetype. Feynman ≥3/5 AND modern
  ≥4/6, else KILL. The immediate decisive test of the diagnosis, before tagging any distractor.
- **Over-abstraction guard (C-30):** the adjacency must not re-introduce the EXP-RS-16 collapse —
  the max fraction of corpus papers matching any single archetype's exact-or-adjacent set must stay
  < 0.5, AND the soft arm must beat the lexical-null.

**LOCKED PREDICTIONS (before the adjacency graph is built or scored):**
- **P1 (the diagnosis test):** under soft (exact-or-adjacent) matching the soft gate PASSES — Feynman
  ≥3/5 (up from exact-match 1/5) AND modern ≥4/6. Direct test that brittleness, not absent
  representation, killed EXP-RS-17.
- **P2 (lift over exact-match):** MethMeSH-Soft recall@10 > exact-match MethMeSH on Feynman AND
  > 0.15 floor on both corpora.
- **P3 (the bar):** MethMeSH-Soft recall@10 ≥ the modern brute-force baseline (primary,
  leakage-controlled) AND competitive with the 0.60 Feynman bar.
- **P4 (no over-abstraction / mechanism):** guard holds (flood < 0.5) AND soft > lexical-null AND
  soft(s=0.5) ≥ exact(s=0) — the adjacency, not coarsening-to-mush, carries the lift.
- **P5 (artifact):** recovered pairs' matched exact-or-adjacent archetype pair maps to the
  ground-truth `bridge_names` (pair04: percolation-threshold ~ giant-connected-component → threshold/
  connectivity).
**GATE:** ADVANCE iff P1∧P3∧P4∧P5 → mechanism-archetype LBD viable; formalize + build the CAS
verifier (#3). PIVOT iff P1∧P2∧P4 but only ties the baseline → #4-soft as the cheap O(N) first stage
of a tag→verify cascade. KILL the mechanism-archetype line iff P1 fails (adjacency doesn't restore
tagging recall) OR P4 fails (only over-abstraction lifts it) → fall back to slot-frames (#2). Reuses
C-14, C-17, C-19, C-20, C-22, C-23, C-24.

### (just-run, KILLED 2026-07-07) EXP-RS-17 → Phase 36 — mechanism-ontology (MethMeSH #4), exact-match

**EXP-RS-17 → Phase 36 — mechanism-ontology (MethMeSH, brainstorm #4) generator vs the brute-force
baseline, with a leakage-controlled modern held-out bar. RUN 2026-07-07 — KILL-BY-CONSTRUCTION
(cheap tagging-recall gate FIRED; P2 FALSIFIED). Forward path = human's go/kill/pivot call.**
Human approved BOTH fallback generators (2026-07-07); #4 selected to build first (human, 2026-07-07).

> **RESULT (2026-07-07):** Cheap tagging-recall gate (C-26), run first as designed: **Feynman
> tagging-recall = 1/5 (FAIL, gate ≥3/5)**, modern = 4/6 (PASS). **P2 (≥3/5 Feynman AND ≥4/6 modern)
> FALSIFIED → pre-registered KILL-by-construction**, reached BEFORE any distractor tagging or the
> full eval. Mechanism: exact-archetype-ID overlap is brittle to granularity — genuinely analogous
> Feynman papers get tagged with *neighboring but non-identical* archetypes (pair04: giant-connected-
> component/birth-death-branching vs compartmental-flow/simple-contagion) ⇒ signature cosine 0. The
> OPPOSITE pole from EXP-RS-16's over-abstraction collapse; the representation is fine (turing-
> instability WAS tagged on the Turing paper), the exact-match rule is not. P1/P3/P4 not evaluated
> (gate short-circuited, by design). Constructive residue: 22 reusable blind tags + the leakage-
> controlled modern corpus + a sharp diagnosis motivating similarity-aware scoring (a fresh
> EXP-RS-18, NOT a re-tune). Full record: `.planning/phases/36-methmesh-vs-baseline/36-VERIFICATION.md`.
> **Forward = human's call:** (A) accept KILL → build #2 slot-frames; (B) complete the modern-only
> eval to quantify a PIVOT; (C) pre-register EXP-RS-18 (archetype-similarity scoring, reuses the tags). Layer: semantic-conceptual (mechanism archetypes), NOT graph
topology — same chapter as EXP-RS-16. Direct response to the EXP-RS-16 kill (over-abstraction
collapse of a closed *role* vocab): MethMeSH swaps the lossy role-schema representation for a
*frozen, field-agnostic mechanism ontology* and adds a **cheap tagging-recall gate** that detects the
same collapse *before* the full eval is paid for.

**Design (LOCKED — full convention text C-22..C-26 in CONVENTIONS.md):**
- **Generator.** Freeze a ~50–150 archetype field-agnostic mechanism vocabulary (mean-field/Ising,
  SIR-compartmental, reaction-diffusion, master/Fokker–Planck, percolation-threshold,
  message-passing/cavity, martingale, replicator, …), **seeded from external taxonomies (TRIZ +
  canonical applied-math / math-physics), built by a BLIND subagent with no benchmark/ground-truth
  access, then SHA-256-hashed and committed BEFORE any tagging** (C-22 — the leakage control; the
  orchestrator has already seen `bridge_names`, so vocab construction is delegated blind and each
  archetype carries external provenance). A blind per-paper subagent tags each paper from
  `{title, abstract}` + the frozen vocab with 1–5 archetypes, each carrying an evidence snippet
  (C-23). Candidate score(i,j) = **IDF-weighted archetype-signature similarity MINUS λ·lexical-similarity**
  (λ=1.0; lexical = the C-17 abstract bag-of-words TF-IDF null); rare (high-IDF) shared archetypes
  carry the weight (C-25).
  - **FROZEN 2026-07-07 (leakage-control lock):** `data/mechanism_vocab.json` = **125** field-agnostic
    archetypes, built blind by a subagent with no benchmark access (external seed: TRIZ 40 principles +
    canonical stat-mech / dynamical-systems / network-science / stochastic-process / info-theory /
    optimization / random-matrix taxonomies; per-archetype provenance). **SHA-256
    `aa6584dcbd992bcafc5ceff87961f0271e66c6403ca376e8a1a6ff95dadd1a6a`**, committed BEFORE any tagging.
    No edits post-freeze. (Benchmark bridge mechanisms — threshold/percolation, compartmental,
    reaction-diffusion/Turing, cavity/message-passing, martingale, replicator, phase-sync — appear as
    canonical archetypes present in ANY complete taxonomy, not benchmark-derived; the known
    "circularity" risk is exactly what the modern held-out bar + P4 ablation control for.)
- **Corpora.** (1) Feynman MVP = REUSE `data/mvp_corpus.json` (36 papers, C-14) → apples-to-apples
  vs the 0.60 leaky bar and the SME 0.00. (2) **Modern held-out MVP (NEW, C-24)** = the 6 evaluable
  `modern_lbd_pairs.json` pairs (m01/02/03/04/06/08) × 2 sides = 12 endpoints + ~24 deterministic
  post-2018 arXiv distractors → the **leakage-controlled** testbed (`research_synergy_modern.json` is
  a 3-node stub, unusable → must be built).
- **Eval** = C-19 conditional retrieval (given side_a, rank all others, is side_b in top-k?):
  recall@{1,5,10} + MRR, on BOTH corpora. Baseline = C-20 brute-force LLM ranking (already 0.60 on
  Feynman; must be RUN on modern → the modern bar). **Cheap early gate (run FIRST): tagging recall**
  — do both sides of a benchmark pair share ≥1 frozen archetype? (C-26). **Ablations:** IDF-weight
  ON vs OFF; λ=1 vs λ=0; archetype-frequency distribution (flood diagnostic). **Auditable co-primary
  deliverable:** the shared-archetype + evidence-snippet table for recovered pairs, scored vs
  `cross_bridges_ground_truth.json` `bridge_names`.

**LOCKED PREDICTIONS (before any method runs; no post-hoc adjustment):**
- **P1 (job-zero + leakage direction):** the C-20 brute-force baseline produces a real recall@10 on
  the NEW modern corpus, AND modern recall@10 ≤ Feynman 0.60 (the modern pairs are less famous /
  partly post-pretraining-cutoff → the LLM's leakage advantage shrinks).
- **P2 (representation adequacy — the cheap KILL gate):** ≥3/5 Feynman pairs AND ≥4/6 modern pairs
  have both sides sharing ≥1 frozen archetype. Below → the vocab cannot represent the bridges →
  KILL by construction (the EXP-RS-16 collapse, caught cheaply, before the full eval).
- **P3 (stake vs the bar):** MethMeSH recall@10 ≥ the **modern** brute-force baseline (the
  leakage-controlled comparison is primary) AND recall@10 > 0.15 (C-3 TF-IDF floor) on BOTH corpora.
  (On leaky Feynman, MethMeSH may trail the pretraining-inflated 0.60 — acceptable iff it wins on
  modern.)
- **P4 (mechanism — load-bearing, the roles-ON/OFF analog):** IDF-weighted signature > uniform-weight
  AND λ=1 (signature-minus-lexical) > λ=0 (signature-only). The *rare shared mechanism*, not generic
  tag overlap or lexical similarity, carries the signal.
- **P5 (auditable artifact):** for recovered pairs the shared archetype matches the ground-truth
  `bridge_name` family (pair03→compartmental-transition, pair04→threshold/percolation,
  pair06→reaction-diffusion, …).

**GATE:** **ADVANCE** iff P2 ∧ P3 ∧ P4 ∧ P5 → MethMeSH is a viable generator; formalize the
`abc_bridge.rs` refactor, then build the CAS verifier (#3) as the precision layer. **PIVOT (don't
kill)** iff floor cleared ∧ P4 holds ∧ artifacts high-quality but MethMeSH only *ties* the modern
baseline → keep #4 as the cheap O(N) first stage of a cascade (tag → SME/CAS verify); next build =
CAS verifier (#3). **KILL #4** iff P2 fails (tagging recall below gate) OR recall@10 < floor on BOTH
corpora OR P4 fails (ablations null) → fall back to slot-frames (#2). Reused conventions: C-3 (floor),
C-14 (Feynman corpus), C-17 (lexical-null), C-19 (retrieval metric), C-20 (baseline). Full brainstorm
rationale: `.planning/research/BRAINSTORM-cross-field-transfer.md` §2 (#4).

### (history) EXP-RS-16 chapter open + dynamical-line kill

**KILL DECISION — HUMAN, 2026-07-05: the dynamical-substrate LBD line (Gen-4) is RETIRED.** After
the 6-phase arc (29→34) refuted both graph-dynamical/spectral candidates (Kuramoto single-cut, sheaf
frustration; both recall@10 = 0 on a fair corpus), the human accepted the kill and directed a fresh
brainstorm of the project's CORE GOAL from scratch (cross-field research transfer / synergy
discovery), decoupled from any specific graph-dynamical method. New direction seeded by a fan-out
ideation workflow (2026-07-05; 32 agents, 78 ideas → 13 directions):
`.planning/research/BRAINSTORM-cross-field-transfer.md`. **Reframe: the analogy signal is
semantic-conceptual (equations / mechanisms / problem-structure), not graph-topological.** Recommended
first move: establish the un-run brute-force baseline (EXP-RS-10) on the valid testbed AND head-to-head
test a flagship generator — **Structure-Mapping (SME) over LLM-extracted role-typed relational
schemas** — with a roles-ON-vs-OFF ablation and the alignment table as a co-primary deliverable; spin
up the citance/eponym gold-set harvest in parallel. The hard core "bridges emerge from graph dynamics"
is abandoned; working baseline reverts to brute-force LLM community-pair comparison (EXP-RS-10).

**EXP-RS-16 → Phase 35 DONE (2026-07-06): SME KILLED; brute-force baseline established.** Ran the
full head-to-head on the valid testbed (36-paper MVP corpus, 36 blind role-typed schemas). Result:
**SME roles-ON recall@10 = 0.00 vs brute-force LLM baseline recall@10 = 0.60 (MRR 0.63).**
Role-typing *inverts* the prediction — roles-ON (0.00) < roles-OFF (0.20) < lexical-null (0.40);
adding structure hurts. Alignment tables empty for 3/5 pairs. **P2, P3, P4 all FALSIFIED; P1
confirmed.** Both KILL conditions fire → SME-over-blind-schemas retired. Mechanism: over-abstraction
collapse — the closed role vocab maps every network-physics paper onto one skeleton (51% of pairs
score systematicity 0), so the true analogue ties with distractors; the blind schema bottleneck
discards the discriminating content the full-context LLM (0.60) keeps. **Durable win: job zero is
done — the brute-force baseline now has a real number (recall@10 = 0.60, MRR 0.63), the bar every
future generator is judged against.** Full record: `.planning/phases/35-sme-vs-baseline/35-VERIFICATION.md`.
**NEXT — human approved (2026-07-07) BOTH fallback generators → build EXP-RS-17:** mechanism-ontology
tagging (#4, MethMeSH) and slot-frames (#2, problem↔method typed transfer), either order, each
evaluated against the **0.60 bar** + a **modern-held-out** leakage-controlled bar (the
`modern_lbd_pairs.json` robustness run, now promoted from deferred to run-alongside). Pre-register
EXP-RS-17 predictions here + in the vault before running. **Repo layout change (human, 2026-07-07,
CONVENTIONS C-21 supersedes C-7): ALL implementation now IN-REPO at `research-synergy/prototypes/`;
the professional-vault is management-only.** Harness (`prototypes/{build_mvp_corpus,sme_lite}.py`,
venv from `prototypes/requirements-lock.txt`) is reusable for both generators.

### (history) EXP-RS-16 pre-registration (design LOCKED before run, 2026-07-06)

**EXP-RS-16 — SME generator vs brute-force baseline, head-to-head (conditional-retrieval eval).**
Layer: semantic-conceptual (per the brainstorm reframe), NOT graph topology.
- **Corpus (MVP):** the 5 evaluable Feynman pairs' endpoint papers (10, ~9 have abstracts) +
  ~26–30 distractors sampled 2/community from the testbed, abstracts fetched (arXiv API / OpenAlex).
- **Eval = conditional retrieval** (brainstorm-recommended, cheaper + statistically kinder): for each
  benchmark pair, given side_a, rank all other papers; does side_b appear in top-k? Report recall@k
  + MRR. Also run on held-out `modern_lbd_pairs.json` to blunt stat-phys-overfit + pretraining leakage.
- **SME generator:** Claude extracts a BLIND, abstract-only, role-typed relational schema per paper
  (closed role vocab: control-parameter, order-parameter, coupling, conserved-quantity, threshold, …
  + higher-order relations CAUSES / UNDERGOES-TRANSITION-AT / CONSERVED-UNDER / COUPLES; domain nouns
  alpha-renamed). Python SME-lite matcher scores pairs by SYSTEMATICITY (deep relational overlap,
  surface attrs = 0). The alignment table (spin↔opinion, magnetization↔consensus) is a CO-PRIMARY
  deliverable. Built blind (single-paper extraction; matcher the LLM never sees) → no scoring leakage.
- **Baseline (EXP-RS-10, job zero — currently UN-RUN):** Claude conditional-retrieval ranking of
  candidates per benchmark side_a. Establishes the bar everything is judged against.

**LOCKED PREDICTIONS (before any method runs):**
- P1: baseline produces a real recall@k number on the testbed (job zero exists).
- P2 (stake): SME recall@10 ≥ brute-force baseline AND clears the TF-IDF floor (BENCH_P10 > 0.15).
- P3 (mechanism): roles-ON > roles-OFF (relational structure, not lexical overlap, carries the signal).
- P4: SME alignment tables match `cross_bridges_ground_truth.json` bridge_names on recovered pairs.
**GATE:** ADVANCE iff P2 ∧ P3 ∧ P4. PIVOT (don't kill) iff SME ties baseline on recall@k BUT alignment
tables are high-quality → metric shifts to certified-mapping quality; CAS verifier (brainstorm #3)
becomes next build. KILL SME iff fails TF-IDF floor OR roles-ON ≤ roles-OFF → fall back to slot-frames
(#2) or mechanism-ontology (#4). Full design + wildcards + completeness critique:
`.planning/research/BRAINSTORM-cross-field-transfer.md`.

### (history) None active — the dynamical-substrate LBD line reached its method-level KILL criterion
(2026-07-05). Both graph-dynamical/spectral candidates fail the shared 10-pair Feynman benchmark
at recall@10 = 0 on a fully valid, bridge-containing corpus: Kuramoto–Fiedler (Phase 33, single
global cut) and sheaf frustration (Phase 34, EXP-RS-15, bridges rank #69–218 not top-10; T4
ablation 0/5). **Recommendation: retire the dynamical-substrate line, revert to the brute-force
baseline (EXP-RS-10, BF-community-pairs LLM) as the working LBD method.** The go/kill decision is
the human's. What the 6-phase arc (29→34) produced and leaves behind: a corpus-construction method
(`build_bridge_corpus_openalex.py`), a VALID benchmark testbed
(`research_synergy_bridged_fine{,_sheaf}.json`), and two clean mechanistic method-negatives. RAF
(reaction-model encoding, EXP-RS-08) remains an untested different-data-model track — a possible
last dynamical option before full kill, but low expected value given both graph methods tied at 0.
Full record: `.planning/phases/34-sheaf-vs-kuramoto/34-VERIFICATION.md`.

### (history) EXP-RS-15 → Phase 34 — sheaf vs Kuramoto head-to-head Human-directed (2026-07-05) to run the tournament here.
Sheaf = cellular-sheaf near-section **frustration** ranking of inter-community edges (LOCAL,
per-edge) — the detector Phase 33 predicted should beat Kuramoto's SINGLE global Fiedler cut.
Sheaf v01 (EXP-RS-07) was HELD pending "a larger multi-domain corpus with inter-community edges" —
`research_synergy_bridged_fine_sheaf.json` (1400 nodes, 34 communities, 4/4 pairs bridged,
benchmark communities share 4–42 terms) is exactly that. Runner `sheaf_lbd_v02.py`; same T2
precision@10 + per-pair recall metric as Kuramoto v07 (apples-to-apples). c-TF-IDF per community is
aggregated from per-node vectors (exploratory approximation; formalize via resyn if it shows signal).

**LOCKED PREDICTION (before score):** sheaf per-pair recall@10 ≥ 0.25 (or T2 precision@10 ≥ 0.2) —
local frustration ranks ≥1 benchmark community-pair into the top-10 where the global cut got 0/4.
**Decisive:** sheaf > 0 ⇒ "local beats the global cut" confirmed → H-RS-substrate revived on a fair
test; sheaf = 0 too ⇒ NO graph method surfaces these bridges on this corpus → strong push to the
brute-force baseline. (RAF = separate reaction-model track, not head-to-head here.)

### (history) EXP-RS-14 → Phase 33 closed the Kuramoto–Fiedler line with a CLEAN,
mechanistically-explained method-negative, (recall@10=0 on a fully well-posed corpus; single
global Fiedler cut can't straddle multiple cross-domain pairs). **Thread is now at a go/kill/pivot
gate — the human's call.** The five-phase substrate arc (29→33) produced: (a) a corpus-construction
method, (b) a VALID benchmark testbed (`research_synergy_bridged_fine.json`), (c) the clean Kuramoto
refutation, (d) a sharp prediction that LOCAL/multi-scale detectors (sheaves — the original
H-RS-substrate hypothesis) beat the single global cut. **Recommended next (human decision):** run
the sheaf/RAF/Kuramoto tournament on the valid testbed via `/cartographer --tournament` (out of
scope for this repo session). If sheaves also score 0/4 on this fair test → the dynamical-LBD hard
core is refuted → revert to the brute-force baseline. Full record: `33-VERIFICATION.md`.

### (history) EXP-RS-14 → Phase 33 — the definitive valid run
simultaneously connected + bridge-containing + synchronized + finely-partitioned. Motivation: four
phases, four confounds — connectivity (29/30), corpus content (31), and now **dynamical
non-convergence at scale** (32: on 1400 nodes the Kuramoto system found a low-K scattered fixed
point, r=0.136; the λ₂≥0 K-criterion admits unsynchronized states; 7-community Louvain collapsed
pair03). Kuramoto–Fiedler has a NARROW operating window; we have satisfied each condition alone but
never all together. EXP-RS-14 removes the convergence + granularity confounds (principled,
pre-registered, reported either way — NOT benchmark tuning): finer Louvain (res=3.0
→ 34 communities, all 4 pairs in DISTINCT communities, 4/4 pairs now have inter-community bridge
edges: pair01:9, pair03:23, pair04:27, pair06:4) + sync-aware K (`find_K_sync`: min K with r ≥ 0.90;
the 1400-node graph verified to sync — r=0.71@K=5, 0.96@K=15). Kept the FULL 1400-node corpus (no
reduction → no selection concern). Runner `kuramoto_lbd_v07.py`.

**LOCKED PREDICTION (EXP-RS-14, before run):** on this fully-valid corpus (connected ∧ bridged ∧
synchronized ∧ finely-partitioned) genuine cross-domain per-pair recall@10 ≥ 0.25 (≥1 of 4 pairs
detected). **Decisive:** still 0 here ⇒ the cleanest Kuramoto–Fiedler method-negative — the method
fails to surface present bridges even when everything is well-posed. Positive ⇒ real signal →
independent falsification, then formalize through the resyn pipeline.

### (superseded) EXP-RS-13 — Phase 32, INCONCLUSIVE (confounded) 2026-07-04
pre-registered 2026-07-04, prediction LOCKED before result. Human
approved the Phase 2 corpus rebuild (2026-07-04). Built a benchmark-centric **bridge-containing**
corpus via a targeted OpenAlex fetch (endpoint citation neighborhoods; neutral rule, NOT tuned to
the benchmark): `data/research_synergy_bridged.json` — **1400 nodes, 9624 edges, 9 communities;
3/4 evaluable pairs now have inter-community bridge edges** (pair01:91, pair04:649, pair06:66) vs
1/4 in data-kuramoto. This addresses the EXP-RS-12 corpus-content gap.

**LOCKED PREDICTION (before the score — v06 still computing when this was written):**
per-pair recall@10 ≥ 0.25 (the method surfaces at least the bridges now present); global BENCH_P10
uncertain (top-10 dilution on a larger corpus). **Decisive read:** if even a bridge-CONTAINING
corpus yields ~0 detections → clean statement that Kuramoto–Fiedler fails to surface known,
present bridges (real method-negative). If it detects them → line alive; formalize through the
resyn pipeline (bulk-ingest→analyze→export) for the official number.

Runner: `prototypes/kuramoto_lbd_v06.py` (bridged corpus + per-pair metric; committed before run,
vault `3115c57`). Result lands in `prototypes/data/kuramoto_v06_results.json`.

*EXP-RS-11 (TF-IDF, Phase 30) remains dead. EXP-RS-12 (Phase 31) validated the methodology fix but
found the corpus lacked bridges. EXP-RS-13 tests the same method on a corpus that now contains
them — the fair test EXP-RS-12 could not run.*

## Claim history

`CLAIMS.jsonl` (commission-compatible) is created by the first `/commission --research` run on
this thread; until then the table above is the claim record. (Exact format is an open Layer-2
spec question — session feedback welcome.)

## Last verification

2026-07-16 — Phase 39 (EXP-RS-20): **Generate→Verify Cascade KILLED at the cheap gate.** Blind verify
stage (`verify_prompt.md` SHA `5f79b20b…` frozen pre-run per C-38; C-39 input closure; C-40 pruning).
P1 cheap forward gate (175 inputs) run as the first orbiter/pi-migration **Mistral-executor /
Claude-overseer backbone head-to-head**: Mistral-large full-175 headline cascade fwd recall@10 = **0.20
≤ 0.40 null → P1 FAILS → KILL**; **0.20 under all 3 pruning severities** (verify can't beat the null —
over-prunes 4/5 true analogues; prune-soft reverts to HyDE). Only pair04 (percolation→epidemics
PGF/branching bridge) survives, rising 4→2 (both backbones agree; P5 card holds). Sonnet-5 ceiling 0.40
(keeps 2/5) = ties null → still FAIL. Backbone method_coherence κ=0.45; Mistral over-prunes (W-SYN
synthesis weakness). Pre-registered KILL → lexical-intermediate line EXHAUSTED → escalate to embedding
substrate / pure-LLM-judge cascade (EXP-RS-21). Durable residue: blind verify harness + Sonnet/Mistral
verdict sets + the pair04 transfer card. Full record: `prototypes/verify_results_feynman_llm.json`,
`prototypes/data/verify_compare_feynman.json`. Prior verifications:

2026-07-07 — Phase 38 (EXP-RS-19): **HyDE-Bridge — pinned headline KILLED at the cheap gate, but the
generation MECHANISM is PROVEN.** Blind HyDE prompt (hashed) + 5 blind Feynman side_a generations
(hashed). Headline (K=5,max,λ=0) forward recall@10 = **0.20 < 0.40 null → GATE-A FAILS → KILL**; but
**GATE-B PASSES** (pair04 17→4, verified: epidemiology hypothetical matches the real epidemics paper,
cos 0.15 vs 0.06). Cause = max-pool distractor inflation (K1 0.60 / K3 0.40 / K5 0.20); K=1 ties the
LLM 0.60 (non-promotable, C-36). Signal survives → violates C-37's "no signal" KILL premise → both the
escalation branch and the design's PIVOT converge on the **generate→verify cascade (#3)**. Forward =
human's go/kill/pivot. Full record: `38-hyde-bridge/38-VERIFICATION.md`. Prior verifications:

2026-07-07 — Phase 37 (EXP-RS-18): **MethMeSH-Soft KILLED → mechanism-ontology line (#4) retired.**
Blind frozen archetype-adjacency graph (124 nodes/427 edges, SHA-256 `620d9c0f…`, committed
unmodified). Soft gate (C-29, exact-or-adjacent) on the 22 tags: **Feynman 2/5 (up from 1/5) — FAIL
(≥3/5)**, modern 5/6 — PASS. P1 falsified → KILL. Adjacency helps but can't close the residual gap
(cross-domain bridges need domain-knowledge equivalences invisible from field-agnostic glosses; the
non-physics side gets generic tags). Two clean experiments (exact EXP-RS-17 + soft EXP-RS-18) refute
#4 on the Feynman bar. Forward = slot-frames #2 (pre-approved; go is the human's). Full record:
`37-methmesh-soft/37-VERIFICATION.md`. Prior verifications:

2026-07-07 — Phase 36 (EXP-RS-17): **MethMeSH (mechanism-ontology #4) KILLED-BY-CONSTRUCTION at the
cheap gate.** Blind 125-archetype frozen vocab (SHA-256 `aa6584dc…`, committed before tagging) + 22
blind endpoint taggings. Tagging-recall gate (C-26): **Feynman 1/5 (FAIL, ≥3/5) / modern 4/6 (PASS)**
→ P2 falsified → pre-registered KILL, reached before any distractor tagging or full eval. Mechanism:
exact-archetype-ID overlap is brittle to granularity (analogous papers → neighboring non-identical
archetypes); the opposite pole from EXP-RS-16's over-abstraction. Representation is fine, matching is
not. Forward = human's go/kill/pivot (accept KILL → #2 slot-frames / complete modern eval / fresh
EXP-RS-18 with similarity scoring). Full record: `36-methmesh-vs-baseline/36-VERIFICATION.md`.
Prior verifications:

2026-07-06 — Phase 35 (EXP-RS-16): **SME KILLED; brute-force baseline established.** SME over blind
role-typed schemas recall@10 = **0.00** vs brute-force LLM baseline **0.60** (MRR 0.63) on the
36-paper MVP testbed; role-typing inverts (roles-ON 0.00 < roles-OFF 0.20 < lexical 0.40); alignment
tables empty 3/5. P2/P3/P4 FALSIFIED, P1 confirmed; both KILL conditions fired. Mechanism =
over-abstraction collapse (51% of pairs score systematicity 0; blind schema bottleneck discards the
discriminating content the full-context LLM keeps). Durable win: job-zero baseline number is done
(recall@10 = 0.60), the bar for future generators. Next (human's go/kill): slot-frames (#2) or
mechanism-ontology (#4). Prior verifications:

2026-07-05 — Phase 34 (EXP-RS-15): **sheaf vs Kuramoto head-to-head — both fail, recall@10 = 0.**
Sheaf frustration on the valid testbed recovers 0/4 into the top-10 (benchmark pairs rank #69–218),
T4 ablation 0/5; tied with Kuramoto. Prediction (sheaf ≥ 0.25) FALSIFIED. H-RS-substrate falsified
at the benchmark bar; method-level KILL criterion met for both dynamical/spectral candidates →
recommend brute-force baseline. (Flagged: sheaf's built-in "precision@10=0.400" is a mislabeled
full-list metric, not a real pass.) Prior verifications:

2026-07-04 — Phase 33 (EXP-RS-14): **CLEAN Kuramoto–Fiedler method-negative.** recall@10=0 on a
fully well-posed corpus (connected+bridged+synchronized+fine communities); mechanism verified
(single global Fiedler cut, all pairs same side). Prediction FALSIFIED. Kuramoto line closed;
H-RS-substrate (sheaves) unblocked on the valid testbed with a sharp prediction. Prior verifications:

2026-07-04 — Phase 32 (EXP-RS-13 INCONCLUSIVE/confounded). Corpus fix worked (3/4
pairs bridged; real bridges rank #11/#17 of ~9600 edges) but the 1400-node Kuramoto run did NOT
converge (r=0.136, K_stable collapsed to floor, λ₂<0) → invalid; sole "detection" was a pair03
same-community artifact. Genuine cross-domain recall@10 = 0. Third confound (non-convergence at
scale) identified. Fair test still not run → EXP-RS-14. Prior: 2026-07-04 Phase 31 EXP-RS-12 MIXED.

Earlier Phase 31 (EXP-RS-12 MIXED): Methodology fix VALIDATED (giant CC
well-posed, K_stable=14.25 converges — 29/30 were connectivity artifacts); locked stake P-3
FALSIFIED (BENCH_P10=0.000) but the test was not fair — static diagnostic shows 3/4 evaluable
pairs have zero inter-community edges (bridge literature absent from corpus). Corpus-content gap
isolated from the solved connectivity gap. Kill-criterion check: **not a clean method-kill** (the
method was never given a bridge-containing corpus); decision on Phase 2 corpus rebuild is the
human's. Prior: 2026-07-04 Phase 30 EXP-RS-11 FAIL; 2026-05-05 Phase 29 FAIL.
