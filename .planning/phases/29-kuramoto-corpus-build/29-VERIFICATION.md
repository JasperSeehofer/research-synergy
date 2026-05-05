# Phase 29 — Verification (Plan 29-01)

**Date:** 2026-05-05
**Verdict:** ❌ **FAIL** (notebook gate not reached — corpus shape inadequate)
**Recommended next path:** Path C (TF-IDF cosine edges) per `project_kuramoto_v03_status.md`

---

## Plan deviations from 29-01-PLAN.md

The original plan (D-02, D-03, D-09 → cap=50, depth=2) projected ~3-4 h total runtime. In practice, seed 1 (`cond-mat/0203227`) ran >60 min with no completion signal and a flat-line 429 backoff pattern, despite a valid `S2_API_KEY` in the process environment. Continuing would have exceeded the working-session budget by an order of magnitude.

Two scoped fallbacks from `29-RESUME.md` were applied **together**:

| Knob | Plan | Actual | Reason |
|---|---|---|---|
| `--max-forward-citations` | 50 | **20** | Throughput bottleneck on heavily-cited cond-mat seeds |
| `--max-depth` | 2 | **1** | Lose 2-hop bridges; preserve direct seed neighbourhoods |

All 10 Feynman pair members are explicit seeds, so pair-presence is structurally guaranteed regardless of depth.

A second deviation: the analyze step initially failed with `parse citation edges failed: Failed to deserialize field 'from_id': Expected string, got none`. Root cause: the bidirectional crawl writes `cites` edges where the citing paper has not been persisted (forward-citation inverse edges from S2's `/citations` endpoint). The `get_all_citation_edges` query was returning rows with NULL `in.arxiv_id`. Patched in `resyn-core/src/database/queries.rs:314` by adding `WHERE in.arxiv_id != NONE AND out.arxiv_id != NONE` to the query. The fix is committed independently of this verification.

---

## Crawl stats

| Metric | Value |
|---|---|
| Wall-clock runtime | ~12 min (10 seeds, sequential) |
| Total papers in DB | 421 |
| Errors / panics / 5xx | 0 |
| 429 backoffs (all retried successfully) | ~340 across all seeds |
| DB size on disk | 23 MB |

Per-seed completion (papers found / elapsed):

| # | Seed | Papers | Elapsed |
|---|---|---|---|
| 1 | `cond-mat/0203227` (Dorogovtsev) | 17 | 28.5s |
| 2 | `0710.3256` (Castellano review) | 283 | 508s |
| 3 | `cond-mat/0010317` (Pastor-Satorras SIR) | 1 | 0.004s (already in DB) |
| 4 | `cond-mat/0312131` (Moreno rumour) | 1 | 0.004s (already in DB) |
| 5 | `cond-mat/0007235` (Newman percolation) | 25 | 41.2s |
| 6 | `cond-mat/0205009` (Newman SIR) | 29 | 34.0s |
| 7 | `nlin/0202034` (Drossel food webs) | 13 | 19.5s |
| 8 | `cond-mat/0002374` (Bouchaud wealth) | 18 | 20.4s |
| 9 | `1005.1986` (Nakao Turing) | 24 | 36.2s |
| 10 | `cond-mat/9801289` (Marsili Zipf) | 10 | 18.1s |

The Castellano social-dynamics review (seed 2) dominated runtime as expected for a heavily-cited 2007 review paper.

---

## Analyze + community detection

| Metric | Value |
|---|---|
| Papers analyzed (NLP, TF-IDF top 5) | 404 |
| Avg keywords per paper | 5.0 |
| Communities detected (Louvain) | 35 |
| Louvain iterations | 3 (404 → 190 → 186) |
| Corpus fingerprint | `d530a1a6be30453ce06a217be32fa23439ce86a61bb9e9f0f81321b3f1be7b17` |

Top corpus terms: `model` (293), `dynamics` (197), `network` (162), `networks` (151), `models` (143), `social` (140), `time` (138).

---

## Pre-2015 export

| Metric | Value |
|---|---|
| Path | `professional-vault/prototypes/data/research_synergy_pre2015.json` |
| Filter | `published <= 2014-12-31` |
| Nodes | **153** |
| Edges | **180** (directed) → 174 undirected after dedup |
| Fingerprint | `bbaa202d79b6b775ae120b4bbb012faf209741db9e0a7d1e8c37a07b9adc9bec` |

---

## Pair-presence checkpoint (Step 4 gate)

```
Nodes: 153  Edges: 180
  cond-mat/0203227 (c=65)  <->  0710.3256 (c=11)         [OK]
  cond-mat/0010317 (c=4)   <->  cond-mat/0312131 (c=None) [FAIL — Moreno absent from pre-2015 export]
  cond-mat/0007235 (c=53)  <->  cond-mat/0205009 (c=69)  [OK]
  nlin/0202034 (c=26)      <->  cond-mat/0002374 (c=27)  [OK]
  1005.1986 (c=25)         <->  cond-mat/9801289 (c=135) [OK]
n_eval = 4 / 5  (gate: >= 3) — PASSED
```

Pair 03-B (`cond-mat/0312131`, Moreno rumour) is missing because seeds 3 and 4 short-circuited (already in DB from seed-1 BFS) without expanding their depth-1 neighbourhoods, and Moreno landed in the Louvain "Other" bucket (community_id = u32::MAX-1) which is excluded from the export by design.

---

## Notebook gate (Step 5)

**Outcome: FAIL — `compute_K_stable` cell timed out at 1800 s wall-clock.**

The cell prints `Computing K_stable via bisection ...` and never returns. With cell timeout extended to 30 minutes (the comment says "may take several minutes for large N" — and N=153 is small), the cell still does not complete. `BENCH_V` and `BENCH_P10` were never produced.

### Root cause

Direct execution of cells 1–4 (skipping `compute_K_stable`) reveals the structural problem:

```
N = 153
A type = csr_matrix, shape = (153, 153)
nnz = 348, density = 0.0149
degree: mean=2.35, max=45.0, min=0.0, isolated=38
connected components = 41
largest component = 58
component size dist: [1×many, 3, 54, 58, ...]
```

- **41 connected components** in 153 nodes — graph is highly fragmented
- **38 isolated nodes** (zero-degree)
- Largest component is only 58 nodes (38% of corpus)

`compute_K_stable` calls `eigsh(L_unweighted)` to estimate `lambda_2(L_uw)` and sets `K_hi = 4 / lambda_2`. With 41 connected components, the unweighted Laplacian has **41 zero eigenvalues**, so `lambda_2 ≈ machine_epsilon`, and `K_hi` blows up to 10^6+. The Kuramoto ODE integration over `t_end=200` with `max_step=0.5` is then asked to integrate at coupling strengths so far above any meaningful regime that `solve_ivp` either takes pathological step counts or fails to find a stable theta_star. The bisection bracket loop never terminates within the 30-min budget.

This is not a tunable runtime — it is a **structural property of the corpus**: the dynamical-LBD analysis assumes a connected (or near-connected) graph where Kuramoto synchronisation is meaningful, and a 41-component graph violates that assumption.

### Why scope reduction made the structure worse

`--max-depth 1 --max-forward-citations 20` produces direct neighbourhoods of each seed but very few cross-pair bridges. The cap=500 depth=2 plan would have produced a richer (likely more connected) graph but at infeasible runtime cost. There is **no operating point** in the (depth, cap) plane that simultaneously fits within session budget AND produces a connected enough graph for Kuramoto-LBD on this seed set.

---

## Verdict

❌ **FAIL** — gate criteria not met:
1. ~~`n_eval >= 3`~~ → satisfied (4/5)
2. `BENCH_P10 > 0.15` → **not produced** (notebook never reaches the BENCH cell)
3. `BENCH_V` → **never raised, never passed** (notebook aborts at K_stable bisection)

The dynamical-LBD direction is **empirically infeasible** for this corpus shape. The hypothesis assumes "Kuramoto synchronisation on a Louvain community graph reveals cross-domain bridges"; the necessary precondition (a graph dense enough for synchronisation to be a meaningful concept) cannot be built from pre-2015 cond-mat seeds via S2 within session budget.

---

## Path forward

Per `~/.claude/projects/-home-jasper-Repositories-research-synergy/memory/project_kuramoto_v03_status.md`, three alternative paths exist:

- **Path A** (continue current direction) — ❌ ruled out by this verification.
- **Path B** (redesign benchmark for 2015+ era, where HTML crawler builds dense edges) — viable but requires reissuing the 10-pair Feynman benchmark with newer seeds. Not a continuation of Phase 29; it is a separate phase that revisits both seed selection and crawler choice.
- **Path C** (drop citation graph entirely; build edges from TF-IDF cosine similarity above a threshold) — viable as a same-phase pivot but requires a new notebook (kuramoto_lbd_v04 or sheaves_v01), not the current v03. **Recommended next step.**

**Recommendation:** advance to a Phase 30 with explicit goal "TF-IDF semantic-edge graph + downstream LBD method (Sheaves or Kuramoto)". The 421-paper `data-kuramoto` corpus and 35-community Louvain partition are reusable as input.

---

## Artifacts

- ✅ `scripts/crawl-feynman-pairs.sh` (committed, with `--max-depth 1 --max-forward-citations $MAX_FWD` after fallback)
- ✅ `data-kuramoto/` (gitignored, local)
- ✅ `professional-vault/prototypes/data/research_synergy_pre2015.json` (153 nodes, 180 edges)
- ❌ `professional-vault/prototypes/kuramoto_lbd_v03.executed.ipynb` — not produced (cell timeout)
- ✅ `resyn-core/src/database/queries.rs:314` — fix for null-arxiv_id citation edges (incidental)
- ✅ This file
