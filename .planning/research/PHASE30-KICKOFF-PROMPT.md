# Phase 30 kickoff prompt

*Prepared 2026-07-02 in the vault session that resolved the Path C pivot decision. Jasper's
instruction to the Phase 30 session — start a session in this repo (Fable 5 + ultracode) and
paste the block below, or just say: "Read and execute .planning/research/PHASE30-KICKOFF-PROMPT.md.
ultracode." Delete this file when Phase 30 completes.*

---

Continue the Dynamical-LBD (Gen-4) research thread: execute the approved Path C pivot as
Phase 30, running the pre-registered experiment EXP-RS-11 to a verdict. Use ultracode
workflows where they add rigor, not ceremony.

CONTEXT. Phase 29 (Kuramoto-LBD v03 corpus build) completed as an honest FAIL on
2026-05-05: the pre-2015 cond-mat citation graph is too sparse (~41 components / 153
nodes → K_stable bisection diverges), so the benchmark that would discriminate
H-RS-substrate never ran. On 2026-07-02 I approved the vault cartographer's pivot:
rebuild the substrate as a TF-IDF cosine semantic-edge graph. A time-bound kill gate is
set: if the semantic-edge substrate yields <3 evaluable Feynman pairs or BENCH_P10 ≤ 0.15
by 2026-09-30, the dynamical-substrate line dies and we revert to the brute-force baseline.

READ FIRST (in order):
1. .planning/research/THREAD.md + CONVENTIONS.md — thread state and the append-only
   convention lock (honor it; append, never edit).
2. .cartographer-notes.md — the decision record.
3. .planning/phases/29-kuramoto-corpus-build/29-VERIFICATION.md — what failed, why, and
   the Path C rationale (note the S2 429 tarpit in the deviations — do NOT re-crawl).
4. ../professional-vault/wiki/meta/agentic-experiments-research.md § EXP-RS-11 — the
   pre-registered protocol: hypothesis, predictions, setup, metrics. Predictions are
   locked; they are not adjusted post-hoc.

BOOKKEEPING BEFORE RESEARCH: STATE.md and ROADMAP.md are stale — Phase 29 is still marked
in-progress. Mark it complete (FAIL verdict, gate not reached) and register Phase 30
("TF-IDF semantic-edge graph + downstream LBD method") with a claims→acceptance-tests
contract in its PLAN. Keep GSD scaffolding light; the real work runs on workflows.

THE WORK (per EXP-RS-11's setup):
1. Regenerate the pre-2015 export from data-kuramoto/ (export-louvain-graph …
   --published-before 2014-12-31 --tfidf-top-n 50 → ../professional-vault/prototypes/
   data/research_synergy_pre2015.json — it does not currently exist). No new crawling.
2. New script ../professional-vault/prototypes/build_tfidf_graph.py: c-TF-IDF vectors
   from the export's nodes field → pairwise cosine → threshold τ ∈ {0.2, 0.3, 0.4, 0.5};
   report (n_nodes, n_edges, n_cc, largest_cc_size, mean_degree) per τ.
3. Connectivity precheck: proceed to dynamics only where n_cc/N ≤ 0.05.
4. v04 notebook adapted from ../professional-vault/prototypes/kuramoto_lbd_v03.ipynb with
   the TF-IDF adjacency replacing citation adjacency; run to a real BENCH_P10.
5. Verdict against the locked predictions: at τ=0.3, n_cc/N ≤ 0.05 (vs 0.27 citation) and
   largest CC ≥ 80% (vs 38%); K_stable completes within a 5-minute budget; gate
   n_eval ≥ 3 AND BENCH_P10 > 0.15 (≥ 0.30 = success, per EXP-RS-06).

DISCIPLINE (vault research-operating-manual, "per numerical experiment / run"):
- Commit analysis scripts before running them on real data; log seeds; reproducible runs.
- The τ sweep is pre-registered sensitivity analysis, not tuning. If no τ passes the
  precheck, that IS the result — it fires the kill gate. Do not patch the corpus ad hoc.
- Same-day THREAD.md update after each run: observation, implication per hypothesis,
  anomalies, kill-criterion check. Append new conventions to CONVENTIONS.md.
- Use workflows for rigor: adversarial verification of build_tfidf_graph.py (leakage —
  no post-2015 information in the TF-IDF vectors; no benchmark-pair contamination in the
  substrate construction), independent re-computation of the connectivity stats, parallel
  τ evaluations.
- Before accepting the final verdict — either direction — run /commission --research for
  independent falsification. Use /consult research "<question>" for vault lookups.

OUTCOME HANDLING:
- Substrate connects + benchmark runs → record BENCH_P10 vs thresholds; H-RS-substrate's
  discriminating experiment is unblocked. Flag tournament-readiness in THREAD.md but do
  not run the sheaves/RAFs/Kuramoto tournament — that is /cartographer --tournament's
  job, from the vault.
- Precheck or gate fails → the pivot kill gate has fired. Write the honest FAIL
  verification; the kill decision goes back to me via the vault, not patched locally.
- Either way: 30-VERIFICATION.md with executed computational evidence (a report with
  zero executed evidence is INCOMPLETE), THREAD.md updated, EXP-RS-11 results noted for
  the vault registry.

FEEDBACK OBLIGATION — this session is the software-research pack's first field test.
The vault's Layer-2 spec (../professional-vault/wiki/analyses/research-routine-packs-spec.md)
defers the pack's design to exactly this pivot. As you work, note concretely: which
routines you actually needed, where THREAD.md/CONVENTIONS.md helped or obstructed, and
whether an algorithm-change gate would have caught anything. Leave a "Pack feedback"
section in 30-VERIFICATION.md.

END OF SESSION: conventional commits, then /scribe-debrief.
