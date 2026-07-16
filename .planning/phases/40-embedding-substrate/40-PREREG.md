# EXP-RS-21 — Dense Embedding Substrate: cross-field analogy retrieval in a semantic-embedding space (escalation out of lexical intermediates)

- **Status**: pre-registration DRAFT **v2** (predictions to be LOCKED before any embedding or baseline run)
- **Provenance**: The pre-registered escalation named by the EXP-RS-20 KILL gate (Phase 39, 2026-07-16).
  The entire *lexical-intermediate* chapter — EXP-RS-16 (SME role-schemas), RS-17 (mechanism-ontology
  exact), RS-18 (soft), RS-19 (HyDE), RS-20 (generate→verify cascade) — failed the brute-force LLM bar
  (Feynman conditional-retrieval recall@10 = 0.60). Clean, repeatedly-confirmed finding: **the analogy
  signal is real** (full-context LLM = 0.60) **but not recoverable through any lexical (TF-IDF)
  intermediate** — bag-of-words cosine is blind to cross-vocabulary equivalence (percolation ≡ epidemic
  share zero surface tokens; true-pair whole-abstract cosine ≈ 0 for the 4 hard Feynman pairs). Both the
  RS-20 and RS-19 gates name the same escalations OUT of lexical intermediates: **(a) a real
  semantic-embedding substrate, or (b) a pure LLM-judge cascade.** Human decision (2026-07-16): run
  **(a) the dense embedding substrate first**. Model scope (human, vault values open-source-by-default /
  EU-preferred / reproducible): **open-local headline + `mistral-embed` (EU) descriptive tier.**
- **v2 provenance**: v1 was reviewed by a 5-lens adversarial panel (gate-band / leakage-forking /
  metric-stats / ML-IR / kill-integrity → synthesis; 2026-07-16), verdict **needs-rework** with 9
  must-fixes + 7 should-fixes. v2 applies all of them; the changes are summarized in "Panel fixes
  applied" at the end. The rebuilt GATE is a **grid-verified total function** (`embed_gate_selftest.py`,
  4608 cells → exactly one of 6 verdicts).

## Hypothesis (H-RS-analogy-embedding)

The cross-field analogy signal, real but invisible to lexical retrieval, **is recoverable in a dense
semantic-embedding space**: cosine retrieval over whole-abstract embeddings places cross-vocabulary
mechanism-analogues (percolation↔epidemic) near each other where surface tokens do not overlap — the
qualitatively non-lexical substrate the whole chapter lacked.

**Sub-hypothesis (illustrative, P3 — NOT a powered inferential claim, see below):** a *general*
contrastive semantic embedding may surface cross-domain analogy better than a citation-trained
scientific embedding (SPECTER2), because citation training optimizes co-citation proximity and genuine
cross-domain analogues do not cite each other.

**Discriminating experiment**: replace the C-17 TF-IDF vector with a dense embedding (the ONE change),
holding the C-14 Feynman corpus, C-24 modern corpus, C-19 metric, and all frozen bars/nulls fixed.
Score forward recall@{1,5,10}+MRR vs the Feynman 0.60 LLM bar + 0.40 lexical null + a **random-chance
null**, with the modern set as a **no-regression / liveness guard** (not a discriminator — its lexical
null is already 0.833).

## Honest power statement (SHOULD-3, load-bearing — read before the predictions)

The Feynman set has **5 evaluable pairs** ⇒ recall@10 lives on the coarse grid {0, .2, .4, .6, .8, 1}.
Under the chapter's established ceiling, pairs 01 (diffuse review side_b) and 06 (Turing→Zipf) are
unrecovered by *anyone including the LLM*, and pairs 03/05 are lexically easy (the 0.40 null already
gets them). **So the entire Feynman headline effectively reduces to whether pair04
(percolation→epidemics) crosses rank 10 — effective n ≈ 1.** Consequences, all pre-registered:
- On this grid, "beat the 0.40 null" ⇒ recall@10 ≥ 0.60, which *coincides* with "tie the 0.60 LLM bar."
  So on Feynman, P1 (beat null) and P2 (tie incumbent) are the **same event**; the discriminating unit
  is pair04's rank, promoted to a co-decisive tie-breaker at band boundaries.
- **Random-chance null**: with a 35-candidate pool, chance recall@10 ≈ 10/35 = 0.286/pair; over 5 pairs
  Binomial(5, 0.286) gives P(recall ≥ 0.40) = 0.44 and P(recall ≥ 0.60) = 0.15. So even the 0.40 lexical
  null sits *near chance*. Every comparison is reported as a delta above this random baseline, and a KILL
  is phrased as "ties the lexical null (≈ chance) — static geometry adds nothing," NOT "captures nothing."
- **Framing**: this experiment is honestly a **KILL-vs-TIE(PIVOT) falsification**. ADVANCE (Feynman
  ≥ 0.80 = 4/5) requires dense embeddings to recover a pair the LLM cannot — possible but a stretch; it
  is retained as a live-but-low-probability band, not the expected outcome.

## Frozen model set (pinned BEFORE any scoring — C-43)

| id | model | role | class↑ vote? | encoding | text |
|---|---|---|---|---|---|
| **M1** | `BAAI/bge-large-en-v1.5` | **HEADLINE** — general open contrastive | **yes (local)** | **SYMMETRIC, no instruction** (canonical for a symmetric abstract↔abstract analogy task; MUST-8) | `title + '. ' + abstract` |
| M2 | `allenai/specter2_base` + **proximity adapter** | scientific/citation (illustrative P3 contrast) | yes (local) | native SPECTER: `title [SEP] abstract`, CLS pooling (MUST-7) | title, abstract as text-pair |
| M4 | `thenlper/gte-large` | 2nd general open (robustness P4) | yes (local) | symmetric | `title + '. ' + abstract` |
| M3 | `mistral-embed` (API) | EU strong tier | **NO — descriptive only** (server-side non-reproducible; MUST-4) | symmetric | `title + '. ' + abstract` |

- **Headline designation (SHOULD-6, benchmark-independent)**: M1 = bge-large-en-v1.5 is designated
  headline over M4 = gte-large by a *pre-committed external criterion* — higher MTEB retrieval-average at
  the leaderboard snapshot (bge-large-en-v1.5 ≈ 54.3 > gte-large ≈ 52.2), recorded before any benchmark
  scoring. NOT chosen by any Feynman/modern number.
- **class↑ vote** ranges over the **reproducible local** models {M1, M2, M4} only. M3 (API) can corroborate
  but can never block a class-KILL or be banked (MUST-4).
- **BGE-asymmetric** (query-instruction `"Represent this sentence for searching relevant passages: "`,
  applied to the side_a query only) is a **descriptive ablation** (`--directional`), never the headline.
  The instruction string + per-model HF revision + `max_seq_length` + embedding dim are recorded in
  `data/embed_model_manifest.json`, committed before scoring.
- **No-prior-eval attestation (SHOULD-7)**: no model in the set has been scored on the Feynman/modern
  pairs before this lock; the headline/robustness split was fixed with no per-pair numbers in hand. Only
  the non-benchmark `embed_toytest` (synthetic percolation/epidemic + cooking/music) has been run.

## Predictions (to LOCK before any embedding or baseline run)

**P0 (job-zero — closes the deferred C-35 hole, BLIND per MUST-5)**: compute + SHA-256-freeze the
**C-35 modern brute-force LLM baseline** (`baseline_results_modern.json`) via the C-20 procedure on
`modern_mvp_corpus.json`, executed by a **fresh blind subagent with NO access to `bridge_names` /
`cross_bridges_ground_truth` / any pair file** (sees only {id,title,abstract}×35), **BEFORE any modern
embedding scoring**. Modern is a no-regression FLOOR only; C-35 is descriptive context, never a
strict-beat requirement (MUST-3).

**P1 (substrate-class discriminating gate — run FIRST, Feynman forward, LIVE local models)**: **at least
one LIVE** model in {M1,M2,M4} passes **strict-P1** = [forward recall@10 > 0.40 **AND** pair04 in its
own top-10 **AND** recovers ≥1 null-missed pair (01/04/06)]. `class↑` in the GATE is this exact
3-conjunct predicate (MUST-1) — a 0.60 reached via {03,05}+a topical third *without pair04* does NOT
count. **FAIL (no live model) → clean class-KILL** → escalate to the pure LLM-judge cascade. (NOT "no
signal" — the LLM proves signal at 0.60; the KILL means no static embedding geometry captures it.)

**P2 (headline reaches/ties the incumbent)**: M1 (headline, symmetric bge) Feynman recall@10 ≥ 0.60
**with pair04 recovered** (= strict-P1(M1)) AND modern recall@10 ≥ 0.833 (no regression). On the grid P2
coincides with P1 for M1 (see power statement); the deliverable is the anchor recovery + the objective
card, not the bare number.

**P3 (sub-hypothesis — ILLUSTRATIVE per-pair observation, NOT a powered claim; SHOULD-5)**: report
whether the general models (M1/M4) recover more cross-domain / null-missed Feynman pairs than SPECTER2
(M2). With effective n ≈ 1 this reduces to "does M1/M4 get pair04 and M2 not" — a 1-vs-0 anecdote. Stated
as an observation with its n; the causal claim "citation training hurts cross-domain" is explicitly
**underpowered and not inferred**. If the SPECTER-v1 fallback fires (objective trigger, nice-to-have #1),
P3 is downgraded to "inconclusive on citation-training attribution."

**P4 (robustness across family)**: M1's anchor recovery is corroborated by ≥1 other general model
(M4 gte and/or M3 mistral: pair04 in top-10 AND beats the null). **P3/P4 joint adjudication (SHOULD-5)**:
if M1 and M4 disagree on pair04, P4 fails AND P3 is reported "unstable"; a disagreement caps the verdict
at PIVOT (no ADVANCE).

**P5 (auditable artifact — OBJECTIVE, CO-PRIMARY; MUST-6)**: for the recovered anchor (pair04) under M1,
an **objective** card passes iff (i) query and recovered side_b differ in frozen metadata field
(`community_id`/`domain`/`primary_category`) — genuinely cross-domain — AND (ii) the true-pair cosine
exceeds the query's **same-field** distractor-cosine median by a margin **δ = 0.02** frozen before
scoring; AND (iii) a deterministic random-pair control (non-benchmark cross-field pairs) FAILS the rule
at rate < 0.30. Computed in code (`embed_score.objective_cards` + `random_control`), not a manual read.

## GATE — total-function verdict (grid-verified; MUST-1/2/3; SHOULD-1/2)

Decision variables: `LIVE(m)` = toytest_pass[m] ∧ modern recall@10 ≥ **0.5** (liveness floor «
no-regression floor — "encoder works at all"); `strictP1(m)` = P1 predicate on Feynman; `class↑` =
∃ LIVE local m ∈ {M1,M2,M4} with strictP1; `headlinePass` = LIVE(M1) ∧ strictP1(M1); `R` = M1 Feynman
recall@10; `M1mod` = M1 modern recall@10; `MODERN_FLOOR` = **0.833**; `P4`, `P5obj` as above.

```
¬LIVE(M1)                               → INVALID-headline-broken (investigate; NOT a class-negative)
LIVE(M1) ∧ headlinePass:
    M1mod < 0.833                       → WEAK-no-bank (anchor recovered but modern regressed)
    R ≥ 0.80 ∧ P4 ∧ P5obj              → ADVANCE (strict double-beat + corroboration + objective card)
    else (R=0.60 tie, or missing P4/P5) → PIVOT (bank auditable card + cheap retriever; provisional,
                                                  effective n≈1; next build = embedding→LLM re-rank)
LIVE(M1) ∧ ¬headlinePass:
    class↑ ∧ winner modern ≥ 0.833      → WEAK-PIVOT (bank the winning LIVE local substrate; → emb→LLM
                                                       re-rank; headline underperforms — provisional)
    class↑ ∧ winner modern < 0.833      → WEAK-no-bank
    ¬class↑                             → KILL (no live reproducible dense substrate recovers the pair04
                                                anchor beyond the ~chance lexical null → static geometry
                                                adds nothing; escalate to a pure LLM-judge cascade that
                                                STRUCTURALLY differs from the C-20 one-shot 35-way ranking
                                                — pairwise/tournament, or scaling beyond 36 papers; if no
                                                such structural difference exists, KILL is the TERMINAL
                                                chapter verdict, not a live hand-off)
```

Six terminal outcomes, verified total & non-overlapping over the full discrete grid by
`embed_gate_selftest.py`. The no-regression floor is checked on the *actual model being banked* (M1 for
PIVOT/ADVANCE, the class↑ winner for WEAK-PIVOT), not a discarded model (SHOULD-1).

## Setup

**Reused (frozen residue)**: Feynman MVP (`data/mvp_corpus.json`, 36, C-14, 5 pairs 01/03/04/05/06);
modern held-out (`data/modern_mvp_corpus.json`, 36, C-24, 6 pairs); pairs files; C-19 metric +
`sme_lite.{rank_candidates, eval_direction}` verbatim; bars/nulls — Feynman LLM baseline (C-20) = 0.60
(ranks: 01→12, 03→1, 04→1, 05→1, 06→15); Feynman lexical null = 0.40 (03→2, 05→7; misses 01/04/06);
modern lexical null = 0.833; pair04 anchor.

**NEW**: `embed_score.py` (per-corpus per-model: ranks, recall, MRR, revision, seq-len/truncation,
objective P5 cards, random control, per-model strict-P1; liveness assertions — len==corpus,
non-degenerate, mistral batch alignment); `embed_gate.py` (the total-function verdict + random-chance
null); `embed_gate_selftest.py` (grid totality proof); `embed_toytest.py` (per-model liveness →
`data/embed_toytest_pass.json`); `data/embed_model_manifest.json` (pinned ids/revisions/seq-lens/dims/
instruction string); `data/baseline_results_modern.json` (blind C-35, P0).

**Ablations (DESCRIPTIVE, run after P1; never promotable)**: BGE-asymmetric (`--directional`); direction
reverse + both-avg; text-field title-only / abstract-only; models M2/M3/M4 vs M1.

## Metric

Primary = M1 Feynman forward recall@{1,5,10}+MRR (headline = recall@10), vs the 0.60 bar + 0.40 null +
the random-chance null, with pair04 rank as co-decisive at boundaries. Co-primary = the objective P5
card (+ the illustrative P3 general-vs-SPECTER read). Guard = modern no-regression floor 0.833. Modern is
NOT a discriminator.

## Conventions (proposed C-41..C-45; continue from C-40; do NOT renumber locked ones)

- **C-41 (embedding substrate)**: dense `emb(p) = L2norm(model.encode(TEXT_m(p)))`,
  `score(q,c) = cos(emb(q),emb(c))`. `TEXT_m` = `title + '. ' + abstract` for bge/gte/mistral; native
  `title [SEP] abstract` (CLS pooling) for SPECTER (MUST-7). Corpus (C-14), modern (C-24), C-19 metric,
  bars/nulls unchanged. `embed_score.py` reuses `sme_lite.{rank_candidates, eval_direction}` +
  `methmesh_score.load_pairs` verbatim; embeddings cached; deterministic (torch seed 0, eval, CPU).
- **C-42 (headline pin + SYMMETRIC encoding)**: HEADLINE = M1 bge-large-en-v1.5, **symmetric** (no
  instruction — canonical for symmetric analogy; MUST-8), forward C-19, headline metric = M1 recall@10.
  ADVANCE/PIVOT/KILL hang on this arm. BGE-asymmetric + all model/direction/text-field variants are
  DESCRIPTIVE, non-promotable. M1 designated over M4 by external MTEB-retrieval, benchmark-independent.
- **C-43 (frozen model set + reproducibility + class↑ scope)**: {M1 bge, M2 specter2_base+proximity
  (objective fallback specter-v1), M3 mistral-embed, M4 gte-large}; ids/revisions/seq-lens/dims/
  instruction-string in `embed_model_manifest.json`, committed before scoring. `class↑` / class-KILL vote
  = **reproducible local {M1,M2,M4} only**; M3 descriptive, never blocks a KILL or is banked (MUST-4). A
  model counts only if LIVE (toytest ∧ modern ≥ 0.5; MUST-9). SPECTER must run with the proximity adapter
  active (asserted).
- **C-44 (blind modern job-zero baseline — closes C-35; MUST-5)**: C-20 procedure on
  `modern_mvp_corpus.json` by a FRESH BLIND subagent (no bridge_names/ground-truth/pair files; sees only
  {id,title,abstract}×35), SHA-256-frozen → `baseline_results_modern.json`, before any modern embedding
  scoring. Modern = no-regression FLOOR (0.833); C-35 descriptive, NOT a strict-beat (MUST-3).
- **C-45 (total-function gate + objective artifact)**: the 6-outcome verdict tree above (MUST-1/2/3),
  grid-verified total by `embed_gate_selftest.py`; no-regression floor on the banked model (SHOULD-1);
  random-chance null reported (SHOULD-2); liveness gate (MUST-9). Objective P5 card = cross-domain
  metadata ∧ cosine-margin δ=0.02 over same-field median ∧ random control < 0.30 (MUST-6). CO-PRIMARY.
  Reuses C-14, C-17, C-19, C-20, C-24, C-35.

## Open Risks (updated)

1. **Topical dominance (load-bearing)**: dense embeddings may encode field/topic, not shared mechanism →
   they also fail → clean class-KILL (only LLM reasoning bridges these domains). P1 tests this first,
   cheaply. The objective P5 card + random control guard against a topical-fluke "pass."
2. **Effective n ≈ 1 / near-chance null**: the whole Feynman headline ≈ pair04's rank; 0.40 sits near
   chance. Mitigated by the explicit power statement, random-chance null, pair04 tie-breaker, and leaning
   on the objective card + modern floor rather than the bare recall number. Framed as KILL-vs-TIE.
3. **Headline-model unluck**: M1 pinned; class-KILL needs ALL live local models to fail (a single local
   pass → PIVOT-to-winner, not KILL). M1 chosen by external MTEB, not benchmark numbers.
4. **Modern non-discriminating**: null already 0.833 → guard/liveness only, never a discriminator or a
   strict-beat (MUST-3). Stated up front.
5. **API reproducibility**: mistral-embed can drift server-side → descriptive-only, excluded from the KILL
   vote; dim + model-id + date logged.
6. **Directional artifact**: forward-only could hide a one-way bridge → reverse + both-avg ablation.
7. **Truncation confound (SHOULD-4)**: max_seq_length pinned per model (bge/gte/specter 512, mistral
   8192), token counts logged, endpoints (esp. pair04) checked for truncation.
8. **Leakage**: embeddings are fixed deterministic functions of title+abstract with no tunable
   researcher-df once the set is pinned (this file); C-44 baseline is blind; P5 is objective; no-prior-eval
   attestation recorded.

## Panel fixes applied (v1 → v2)

MUST: (1) `class↑` = strict-P1 verbatim incl. pair04. (2) GATE rebuilt as a grid-verified total function,
6 outcomes, modal tie mapped. (3) modern demoted to a no-regression FLOOR (no strict-beat). (4) mistral
excluded from the class↑/KILL vote. (5) C-44 modern baseline made blind. (6) P5 objectivized (metadata +
δ-margin + random control). (7) SPECTER native [SEP] input + adapter asserted active. (8) headline BGE
made symmetric (no instruction). (9) per-model liveness gate + degeneracy/len assertions.
SHOULD: (1) no-regression floor checked on the banked model. (2) random-chance null added + KILL rephrased.
(3) effective-n≈1 power statement + pair04 tie-breaker + KILL-vs-TIE framing. (4) max_seq_length pinned.
(5) P3 downgraded to illustrative + P3/P4 joint adjudication. (6) M1 designated by external MTEB. (7) manifest
instruction-string + no-prior-eval attestation. NICE: objective SPECTER-v1 fallback trigger; mistral dim
logging; KILL escalation operationalized (must structurally differ from C-20, else terminal).
```
