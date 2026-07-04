# Phase 31 — Verification (Plan 31-01, EXP-RS-12)

**Date:** 2026-07-04
**Verdict:** ⚠️ **MIXED** — the methodology fix succeeded (well-posed graph, dynamics converge),
but the locked stake **P-3 (BENCH_P10 > 0.15) is FALSIFIED** (BENCH_P10 = 0.000). A decisive
diagnostic shows the failure is **corpus-content incompleteness**, not (cleanly) a method failure:
3 of 4 evaluable benchmark pairs have **zero inter-community citation edges** — the corpus does not
contain the bridge literature the benchmark asks the method to find.

---

## Executed computational evidence

- Script: `prototypes/kuramoto_lbd_v05.py` (vault commit `fd177a9`, committed BEFORE run).
- Corpus: `research_synergy_kuramoto_full.json` → **giant CC 224 nodes** (of 227; 2nd component = 3 nodes dropped), 34 communities, 344 undirected citation edges.
- Results artifact: `prototypes/data/kuramoto_v05_results.json`. Deterministic seeds (42/99/77).

### Locked predictions (EXP-RS-12) — evaluated, not adjusted

| # | Locked prediction | Actual | Result |
|---|---|---|---|
| P-1 | Giant CC single component; `compute_K_stable` finite ≤ 300 s | n_cc=1; **K_stable = 14.25** (converged; SC5 λ₂=0.56>0, gap=1.71) | ✅ MET |
| P-2 | `n_eval = 4 ≥ 3` in the giant CC | n_eval = 4 (pairs 01/04/05/06) | ✅ MET |
| P-3 | **BENCH_P10 > 0.15** (stake) | **BENCH_P10 = 0.000** | ❌ **FALSIFIED** |
| P-4 | Real signal > ER + config-model nulls | real J vs nulls: SC1a/SC1b J=0.00 (topology-distinct); but benchmark: real=null=0.000 | ⚠️ bridges are topology-distinct from nulls, yet **no benchmark signal above null** (both 0) |

Supporting checks all PASS: SC3 global-lock PASS, SC5 spectral PASS, SC1a/SC1b null-separation PASS.
The dynamics are healthy — this is **not** the Phase 29/30 hang. The graph is genuinely well-posed.

### The methodology fix is validated

The 2026-07-04 reanalysis was **correct**: Phases 29/30 failed because the pre-2015 slice (C-1)
shattered the citation graph, not because of anything intrinsic. On the full-corpus giant CC the
Kuramoto ODE converges (K_stable finite), the Fiedler cut is spectrally clean, and null controls
pass. We now have the well-posed test that 29/30 never reached. **C-12 (drop the pre-2015 slice)
is confirmed sound.**

### But BENCH_P10 = 0 — decisive diagnostic (why)

Static citation-graph structure between the 4 evaluable benchmark community-pairs:

| pair | domains | communities | **direct inter-community edges** | graph dist | 
|---|---|---|---|---|
| pair01 | Ising ↔ opinion | c65 / c11 | **0** | 3 |
| pair04 | percolation ↔ epidemics | c53 / c69 | **2** | 1 |
| pair05 | Lotka-Volterra ↔ markets | c26 / c27 | **0** | 6 |
| pair06 | Turing ↔ spatial economy | c25 / c135 | **0** | 3 |

- **3 of 4 pairs have NO citation edge between their communities.** The cross-domain bridge papers
  (what the benchmark asks the method to "discover") are **absent from the crawled corpus**. No
  graph method — dynamical, spectral, or semantic — can rank a bridge that is not in the graph.
  (The TF-IDF cosine baseline is correspondingly ~0: endpoint cosines 0.009 / 0.029 / 0.0 / 0.0.)
- **pair04 HAS a 2-edge bridge**, yet scored not-detected: the global top-10 Fiedler bridges are
  dominated by stronger within-corpus community-pairs (top counts: [11,18]=8, [4,28]=5, several ×4).
  A 2-edge bridge cannot crack a *global* top-10 — a **metric-dilution** effect of the fixed-k=10
  denominator when the benchmark pairs are weak bridges among many.

**Two distinct gaps, now isolated** (the whole point of getting to a well-posed graph):
1. **Corpus-content gap (dominant):** the interdisciplinary bridge literature is missing (shallow
   depth-1/cap-20 S2 crawl captured each seed's neighborhood, not the papers *between* domains).
2. **Metric-dilution (secondary):** global-top-10 BENCH_P10 buries weak-but-real bridges; a
   per-pair "is there a top-k bridge for THIS community-pair" would be fairer.

---

## Verdict

⚠️ **MIXED — the stake is falsified, but the test was not fully fair.** P-3 (BENCH_P10 > 0.15)
is falsified on this corpus. However, the pre-registration assumed the well-posed corpus was a
fair test of the *method*; the diagnostic reveals the corpus is **content-incomplete** — it lacks
3/4 of the bridges the method is scored on. Therefore:

- This is **NOT** the clean method-kill P-3 was designed to license. We cannot attribute BENCH_P10=0
  to the method when 3/4 target bridges are simply absent from the graph.
- It **IS** a decisive refutation of the "just fix connectivity and the method works" hypothesis,
  and it isolates the real blocker precisely: **corpus content, not corpus connectivity.**
- It also flags a benchmark-design issue (global-top-10 dilution) independent of the corpus.

**Honest status:** the dynamical method has shown **no recovery signal on any corpus we have built
to date** — but it has never been given a corpus that actually contains the benchmark bridges. A
fair method test requires that corpus (Phase 2). The connectivity problem is solved; the content
problem is now the gate.

---

## Path forward (decision is the human's)

- **Phase 2 — bridge-containing corpus (proposed, gated on human go):** acquire a corpus that
  plausibly *contains* the cross-domain bridge literature — e.g. OpenAlex bulk-ingest of
  cond-mat + stat-phys + q-fin + nlin (rate-limit-free; the S2 429 tarpit does not apply), or a
  deeper multi-hop crawl around the Feynman seeds. Then re-test. Caveat: a larger corpus worsens
  metric-dilution, so Phase 2 likely also needs the per-pair metric variant.
- **Alternative — accept the negative:** if the appetite for corpus-building is exhausted, the
  honest position is "dynamical-LBD unproven; no corpus yet demonstrates signal" → revert to the
  brute-force baseline. This is weaker than a clean method-kill (the method was never fairly tested).

---

## Artifacts

- ✅ `prototypes/kuramoto_lbd_v05.py` (vault `fd177a9`, committed before run)
- ✅ `prototypes/data/kuramoto_v05_results.json` (executed results)
- ✅ Diagnostic (inter-community bridge structure) — this file, § "decisive diagnostic"
- ✅ This file

---

## Pack feedback (EXP-RS-12)

- **Getting to a well-posed graph was the highest-value move** — it converted two non-results
  (29/30 hangs) into a *diagnosable* result. The lesson: when a dynamical method won't run, fix
  well-posedness FIRST; a method that produces a real 0 is far more informative than one that hangs.
- **Pre-registration held under a surprising result.** P-3 falsified, and the discipline (locked
  thresholds, commit-before-run) meant the honest read — "falsified but the test wasn't fair" —
  could not be laundered into either a pass or a clean kill. The claims→acceptance contract worked.
- **New pattern worth codifying:** before scoring a graph-LBD benchmark, run a *static* precheck —
  "do the benchmark pairs' communities share any edge / what is their graph distance?" — to
  distinguish method failure from corpus-content absence. Had this run in Phase 29, it would have
  flagged the content gap immediately. Companion to the connectivity precheck (C-13).
- **Benchmark-design flag for the vault:** BENCH_P10's fixed global-k=10 denominator conflates
  "weak bridge" with "no bridge" and dilutes as the corpus grows. A per-pair top-k detection
  (recall-style) is more faithful to the recovery claim. This is the benchmark owner's call.
