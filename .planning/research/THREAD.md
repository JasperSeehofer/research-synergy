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
| H-RS-analogy-mechanism (new chapter, EXP-RS-17) | The cross-field analogy signal is recoverable by matching papers on a **shared rare mechanism archetype** from a *frozen, field-agnostic* ontology (MethMeSH), beating the brute-force LLM baseline — and, crucially, holding up on a **leakage-controlled modern held-out set** where the LLM baseline's pretraining advantage is neutralised | EXP-RS-17: C-19 conditional-retrieval recall@10 on Feynman (vs the 0.60 leaky bar + SME 0.00) AND a NEW modern held-out corpus (vs its own brute-force bar); cheap tagging-recall gate; IDF-on/off + λ ablations; shared-archetype artifact vs `bridge_names` | **FALSIFIED (exact-match form) at the cheap gate (Phase 36, 2026-07-07): Feynman tagging-recall = 1/5 < 3/5 → P2 falsified → KILL-by-construction.** Modern passed 4/6. Failure = exact-archetype-ID brittleness (neighboring archetypes), NOT absent representation. Similarity-aware scoring = a fresh hypothesis (EXP-RS-18), not a re-tune. |

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

## Active experiment

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
