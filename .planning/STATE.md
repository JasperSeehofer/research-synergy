---
gsd_state_version: 1.0
milestone: v1.4
milestone_name: Discovery & Intelligence
status: in_progress
stopped_at: "EXP-RS-17 (Phase 36) RUN 2026-07-07 — MethMeSH (mechanism-ontology #4) KILLED-BY-CONSTRUCTION at the cheap tagging-recall gate: Feynman 1/5 (FAIL, gate 3/5), modern 4/6 (PASS) → P2 falsified → pre-registered KILL, reached before any distractor tagging/full eval. Mechanism = exact-archetype-ID granularity brittleness (analogous papers get neighboring non-identical archetypes). Full record: .planning/phases/36-methmesh-vs-baseline/36-VERIFICATION.md. **EXP-RS-19 (Phase 38) PRE-REGISTERED 2026-07-07 (ultracode) — HyDE-Bridge (#2 slot-frames, artifact-primary); predictions LOCKED (THREAD + vault + CONVENTIONS C-31..C-37); NOT yet run.** Design chosen by a 21-agent judge panel (5 designs × 3 adversarial lenses → synthesis) that proved every pure-lexical method/object split self-kills (true-pair cosines near-zero on the hard Feynman pairs); only GENERATION converts latent equivalences (percolation≡epidemic) into retrievable tokens. HyDE = blind per-query generation of K=5 cross-field hypothetical abstracts, retrieve real candidates by max-pooled TF-IDF cosine (an EXPANSION → immune to the RS-16/17/18 masking-loss). Honest ceiling = TIE at 0.60 → the auditable transfer card is the primary deliverable; PIVOT-oriented gate. Verified C-17 nulls: Feynman 0.40, modern 0.833. **Next execution (build): (1) blind subagent authors + hashes `hyde_prompt.md`; (2) cheap Feynman gate FIRST (5 side_a generations → full 36-candidate retrieval; GATE-A recall≥3/5 & recover a null-missed pair; GATE-B pair04); KILL iff fails; (3) if it passes: generate the rest + job-zero modern baseline (C-35) + full eval + ablations + transfer cards + adversarial verify → 38-VERIFICATION.md.** (EXP-RS-18 prior: MethMeSH-Soft KILLED, mechanism-ontology line retired — `37-methmesh-soft/37-VERIFICATION.md`.) Prior EXP-RS predecessors: 16 (SME killed, baseline 0.60), 17 (MethMeSH exact killed)."
last_updated: "2026-07-07T00:00:00.000Z"
last_activity: 2026-07-07
progress:
  total_phases: 10
  completed_phases: 8
  total_plans: 20
  completed_plans: 20
  percent: 80
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-04-06)

**Core value:** Surface research gaps and unexplored connections that no single paper reveals — by structurally analyzing and comparing papers across a citation graph
**Current focus:** New chapter (semantic-conceptual analogy) opened after the dynamical-LBD kill. **Phase 35 (EXP-RS-16, 2026-07-06) complete:** the flagship SME generator over blind role-typed relational schemas is KILLED — roles-ON recall@10 = 0.00 vs the brute-force LLM baseline 0.60; role-typing inverts (roles-ON < roles-OFF < lexical); over-abstraction collapse on a physics-dense corpus. **Constructive win: job zero is done — the brute-force LLM baseline now has a real number on the valid testbed (recall@10 = 0.60, MRR 0.63), the bar every future generator is judged against.** Next (human's go/kill): the pre-registered fallback generators — slot-frames (#2, problem↔method typed transfer) or mechanism-ontology tagging (#4, MethMeSH) — evaluated against the 0.60 bar on a leakage-controlled set. Deferred (non-gating): modern_lbd_pairs.json robustness.

## Current Position

Phase: 35 (complete — SME KILLED, baseline established)
Plan: 35-01 (complete, EXP-RS-16)
Status: SME over blind role-typed schemas recall@10 = 0.00 vs brute-force LLM baseline 0.60 (MRR 0.63). P2/P3/P4 falsified, P1 confirmed; both KILL conditions fired. Mechanism = over-abstraction collapse (51% of pairs score systematicity 0; blind schema bottleneck discards the discriminating content the full-context LLM keeps). Durable win: brute-force baseline number established (the bar). Awaiting human go/kill on next generator (#2 slot-frames / #4 mechanism-ontology).
Last activity: 2026-07-06

Progress: [████████████████░░░░] 80% (v1.4 phases 25 Discovery Recommendations, 26 Export & Interop still unstarted)

## Accumulated Context

### Decisions

(Full decision log in PROJECT.md Key Decisions table)

Recent decisions affecting v1.4:

- SurrealDB FLEXIBLE TYPE for complex fields — works but limits server-side querying; revisit for analytics queries in Phase 23
- TF-IDF vectors already stored per paper — Phase 22 similarity engine builds on this without new extraction
- [Phase 24]: Community summaries computed on-read (lazy) — no sidecar cache table
- [Phase 29]: FAIL verdict 2026-05-05 — pre-2015 cond-mat citation graph too sparse for dynamical LBD (41 cc / 153 nodes); benchmark gate never reached. Honest negative; deviations (S2 429 tarpit → cap 20 / depth 1) recorded in 29-VERIFICATION.md
- [2026-07-02, human]: Path C pivot approved (`.cartographer-notes.md`) — rebuild substrate as TF-IDF cosine semantic-edge graph (EXP-RS-11, pre-registered). Time-bound kill gate: <3 evaluable Feynman pairs or BENCH_P10 ≤ 0.15 by 2026-09-30 → kill dynamical-substrate line, revert to brute-force baseline
- [Phase 30]: EXP-RS-11 FAIL verdict 2026-07-04 — TF-IDF cosine semantic edges make the pre-2015 corpus *more* fragmented (n_cc/N=0.830 @ τ=0.3) than the citation graph (0.268) at every pre-registered τ; precheck fails, `BENCH_P10` not producible. **Pivot kill gate FIRED** (well before the 2026-09-30 deadline). Verdict survived a right-sized `/commission --research` (3 converging lines; no under-connection bug, no leakage/contamination). Both substrate candidates now exhausted → the corpus itself is the limiter; Path B (seed selection) is the remaining option. Kill vs Path-B decision = human's, via the vault. See 30-VERIFICATION.md
- [2026-07-05, human]: Dynamical-substrate LBD line KILLED after the 6-phase arc (29→34) refuted both graph-dynamical/spectral candidates (Kuramoto single-cut + sheaf frustration, both recall@10=0). New chapter: semantic-conceptual analogy, decoupled from graph dynamics. Working baseline reverts to brute-force LLM community-pair comparison (EXP-RS-10).
- [Phase 35]: EXP-RS-16 KILL verdict 2026-07-06 — SME over blind role-typed schemas recall@10=0.00 vs brute-force LLM baseline 0.60 (MRR 0.63); role-typing inverts (roles-ON < roles-OFF < lexical); alignment tables empty 3/5. P2/P3/P4 falsified. Over-abstraction collapse (blind schema bottleneck too lossy on a physics-dense pool). **Constructive: brute-force baseline number established (recall@10=0.60) — the bar.** Next generator (human's go/kill): #2 slot-frames or #4 mechanism-ontology. See 35-VERIFICATION.md

### Roadmap Evolution

- Phase 28 added: Forward-citation crawl mode (S2)
- Phase 29 added: Kuramoto-LBD v03 Corpus Build (exploratory benchmark, gates EXP-RS-07) — completed with FAIL verdict
- Phase 30 added: TF-IDF Semantic-Edge Graph + Downstream LBD Method (EXP-RS-11, Path C pivot)
- Phases 31–34 added: dynamical/spectral tournament (EXP-RS-12/13/14/15) — ended in clean method-negative; dynamical-substrate line KILLED (human, 2026-07-05)
- Phase 35 added: SME generator vs brute-force baseline (EXP-RS-16, new semantic-conceptual chapter) — SME KILLED, brute-force baseline established (recall@10=0.60)

### Pending Todos

None.

### Blockers/Concerns

- Phase 25 depends on Phases 22, 23, 24 (needs similarity neighbors, centrality scores, community assignments)
- Phase 30: no new crawling permitted (S2 429 tarpit); predictions locked — no post-hoc adjustment; τ sweep is sensitivity analysis, not tuning

## Session Continuity

Last session: 2026-07-07 (post-EXP-RS-16 review — prototypes migrated in-repo; next steps set)
Research thread state: `.planning/research/THREAD.md` (Layer-2 contract; same-day updates required)

### RESUME POINTER — EXP-RS-17 PRE-REGISTERED (locked), build next. (updated 2026-07-07)

**STATUS 2026-07-07:** EXP-RS-17 is now **pre-registered and predictions LOCKED** — generator = **#4
mechanism-ontology (MethMeSH)** (human's choice). Design + LOCKED predictions P1–P5 + ADVANCE/PIVOT/KILL
gate are in `.planning/research/THREAD.md` (Active experiment) + vault
`agentic-experiments-research.md` + CONVENTIONS C-22..C-26. **NOT yet run** — held at the
pre-registration commit boundary for the human to witness the locked predictions before the run.
Remaining execution (tasks tracked): (1) freeze the ~50–150 archetype field-agnostic vocab via a
BLIND subagent (external TRIZ/math-physics seed, no benchmark access), SHA-256-hash + commit; (2)
build `data/modern_mvp_corpus.json` (12 modern endpoints + 24 deterministic post-2018 distractors,
C-24); (3) `methmesh_score.py` + blind tagging subagents (C-23/C-25); (4) run the tagging-recall gate
FIRST (cheap KILL check, C-26), then the full eval on Feynman + modern; (5) verdict vs locked
predictions in `.planning/phases/36-*/36-VERIFICATION.md`, same-day thread-state update. Below is the
original post-EXP-RS-16 handoff that set this up:



**Repo layout changed (human decision 2026-07-07, CONVENTIONS C-21, supersedes C-7):** ALL
research/LBD implementation now lives **in-repo at `./prototypes/`** (Python + Rust prototypes,
scripts, `data/`, `figures/`). The professional-vault is management-only (thread state in
`.planning/`, brainstorms, conventions) — no implementation. Rebuild the venv once per machine:
`cd prototypes && uv venv --python 3.13 .venv && uv pip install --python .venv -r requirements-lock.txt`
(gitignored). Verify: `.venv/bin/python sme_lite_toytest.py` (asserts pass) and
`.venv/bin/python sme_lite.py` (reproduces roles-ON recall@10=0.00 / lexical 0.40).

**Where EXP-RS-16 landed (done, committed):** SME over blind role-typed schemas KILLED — roles-ON
recall@10 = **0.00** vs brute-force LLM baseline **0.60** (MRR 0.63). recall@10 = fraction of the 5
evaluable Feynman pairs whose known partner paper is retrieved in the top-10 (0.60 = 3/5, all at
rank 1). Full record: `.planning/phases/35-sme-vs-baseline/35-VERIFICATION.md`. Harness (reusable,
`--corpus/--schemas/--pairs`): `prototypes/{build_mvp_corpus.py, sme_lite.py, sme_lite_toytest.py}`
+ `prototypes/data/{mvp_corpus,mvp_schemas,sme_results,baseline_results}.json`.

**THE BAR:** brute-force LLM baseline **recall@10 = 0.60, MRR 0.63** — leakage-inflated (Claude knows
the famous analogies), so treat as an upper-ish bound.

**NEXT — human approved BOTH generators (2026-07-07); build EXP-RS-17.** Do these; they are the
plan, not a go/kill prompt:
1. **Pick a generator to build first — #4 mechanism-ontology (MethMeSH) or #2 slot-frames** (human
   likes both; either order). #4 = tag each paper with 1–5 labels from a *frozen, curated,
   field-agnostic* mechanism vocab (~50–150 archetypes, seed from TRIZ/nLab, **freeze BEFORE looking
   at the benchmark = leakage control**); a bridge = two papers in *different* communities sharing a
   *rare (high-IDF)* archetype, ranked by signature-overlap MINUS lexical-similarity; O(N) tag +
   inverted index. Watch the granularity knob (too coarse → "everything is a phase transition").
   #2 = per-paper problem-slot vs method-slot frames; bridge = method from field B satisfies an open
   problem in field A. See `.planning/research/BRAINSTORM-cross-field-transfer.md` §2 (#4, #2).
2. **Run the modern held-out baseline ALONGSIDE** (human-approved): rebuild an MVP corpus for the 6
   evaluable `modern_lbd_pairs.json` pairs + distractors, run the C-20 brute-force baseline on it →
   a leakage-controlled bar. Reuse `build_mvp_corpus.py` + `sme_lite.py`.
3. **Discipline (unchanged):** pre-register EXP-RS-17 predictions in THREAD.md + the vault
   `agentic-experiments-research.md` BEFORE running; commit scripts before touching data; evaluate
   against the 0.60 bar (and the modern bar); keep the auditable shared-label/frame artifact as a
   co-primary deliverable; no re-tuning to force a pass.

### (history) Dynamical-LBD KILL criterion met (Phases 31→34 complete)

**Phase 34 (EXP-RS-15) DONE:** sheaf-vs-Kuramoto head-to-head on the valid testbed → **both fail,
recall@10 = 0.** Sheaf frustration recovers 0/4 into the top-10 (pairs rank #69–218), T4 ablation
0/5; tied with Kuramoto (Phase 33). H-RS-substrate falsified at the benchmark bar. **Method-level
kill criterion met for both graph-dynamical/spectral candidates.** See
`.planning/phases/34-sheaf-vs-kuramoto/34-VERIFICATION.md`.
**NEXT (human's go/kill — NOT auto-executed):** retire the dynamical-substrate line, revert to the
brute-force baseline (EXP-RS-10, BF-community-pairs LLM). Optional last dynamical straw: RAF on its
reaction-model encoding (EXP-RS-08, different data model, untested) — low expected value. Do NOT
re-tune corpora/metrics to force a pass (spec-gaming).

### (history) OVERNIGHT OUTCOME (Phases 31→33) — Kuramoto clean negative

**Phase 33 (EXP-RS-14) DONE — CLEAN Kuramoto–Fiedler method-negative.** On a fully well-posed corpus
(`research_synergy_bridged_fine.json`: connected + 4/4 pairs bridged + synchronized r=0.932 + 32
communities) the method recovers 0/4 benchmark bridges; NO pair in the top-200 Fiedler bridges.
Mechanism verified: single global Fiedler cut puts all pairs on the same side → structurally
invisible. Not a confound. See `.planning/phases/33-valid-converged-run/33-VERIFICATION.md`.

**The five-phase substrate arc (29→33) delivered:** (a) a corpus-construction method
(`build_bridge_corpus_openalex.py`); (b) a VALID benchmark testbed (`research_synergy_bridged_fine.json`);
(c) the clean Kuramoto refutation; (d) a sharp prediction — LOCAL/multi-scale detectors (sheaves)
should beat the single global cut.

**NEXT (human's go/kill/pivot decision — NOT auto-executed):** run the sheaf/RAF/Kuramoto tournament
on the valid testbed via `/cartographer --tournament` (vault, out of scope for this repo session).
If sheaves also score 0/4 on this fair test → dynamical-LBD hard core refuted → revert to brute-force
baseline. Do NOT re-tune the corpus or start the tournament autonomously — both are governance-gated.

**Decision tree when v06 result is available:**
1. Read `kuramoto_v06_results.json` (BENCH_P10, perpair_recall_at10, perpair_ranks, K_stable, nulls).
2. Write `.planning/phases/32-*/32-VERIFICATION.md` (executed evidence + verdict vs the LOCKED
   prediction — no post-hoc adjustment). Update THREAD.md (same-day), STATE, ROADMAP, vault EXP-RS-13.
3. **If per-pair recall@10 ≥ 0.25 OR global BENCH_P10 > 0.15** → SIGNAL. Next: (a) independent
   falsification (right-sized, blind re-score) that the detections are real not artifacts; (b) if
   confirmed, formalize the corpus through the resyn pipeline (bulk-ingest needs an OpenAlex key —
   currently only unauthenticated polite pool works; note this) → analyze → export → re-run for the
   official number; (c) flag tournament-readiness in THREAD.
4. **If ~0 (no detections despite 3/4 bridges present)** → clean method-negative: Kuramoto–Fiedler
   does not surface known present bridges. Write it up honestly; the kill-vs-continue call is the
   human's (record in THREAD, do not self-kill). Consider whether the global-top-10 metric or the
   Louvain community granularity (only 9 communities on 1400 nodes) is the confound.
5. Commit each step (conventional commits, both repos). `/scribe-debrief` at a clean stopping point.

**Do NOT** re-tune the corpus to make the benchmark pass (spec-gaming). The corpus was built by a
neutral endpoint-neighborhood rule; keep it fixed. Metric/community-granularity are separate,
declarable knobs — changing them is a NEW pre-registered experiment, not a tweak.
