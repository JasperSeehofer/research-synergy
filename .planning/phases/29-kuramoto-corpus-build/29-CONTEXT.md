# Phase 29: Kuramoto-LBD v03 Corpus Build — Context

**Gathered:** 2026-05-04
**Status:** Ready for planning
**Source:** Conversational planning + `.planning/.continue-here.md` (deferred 14d on S2_API_KEY)

<domain>
## Phase Boundary

This is a **data-collection / benchmark-evaluation phase**, not a code-change phase. Phase 28 shipped the `--bidirectional` crawl mode; Phase 29 *uses* it to build the corpus that the deferred Kuramoto-LBD v03 prototype notebook needs, then runs the notebook as the success gate.

The notebook lives at `professional-vault/prototypes/kuramoto_lbd_v03.ipynb` and tests whether dynamical-LBD (Kuramoto-style oscillator coupling on the Louvain community graph) can recover known cross-domain bridges from the 10-pair Feynman benchmark.

**Explicitly out of scope:**
- Modifying any source code in `resyn-*` crates.
- Implementing EXP-RS-07 Sheaves-LBD (next phase, gated on this one passing).
- Re-curating the Feynman pair list (already locked in `feynman_10pair_papers.json` schema v0.2).
- Backfilling earlier corpora (`./data` LIGO, `./data-s2-smoke`).

</domain>

<decisions>
## Decisions

### D-01: Use bidirectional S2 crawl
The original `.continue-here.md` plan (backward-only S2, depth-2) was written before Phase 28 shipped. Pre-2015 cond-mat backward-references frequently return `externalIds: null` from S2, terminating BFS after one hop. Phase 28's `--bidirectional --max-forward-citations 500` was built specifically for this scenario — modern citers of pre-2015 seeds densify the graph in exactly the direction Louvain modularity needs.

### D-02: Cap forward citations at 500 per paper
The Phase-28 default. High-impact 2001 cond-mat papers (e.g. `cond-mat/0010317`, ~3000 citers) would otherwise cause enqueue blowup. 500 is enough to give the Louvain community structure topological signal without exploding crawl runtime.

### D-03: Depth 2
Same as the deferred plan. Depth 2 gives seed → refs/citers → their refs/citers, which is enough connective tissue for cross-domain bridges to appear in the same modularity community.

### D-04: Fresh DB at `surrealkv://./data-kuramoto`
`./data` is the LIGO corpus from earlier work (383 papers, wrong domain). `./data-s2-smoke` is a 1-paper artifact from `.env` verification. Neither is reused; `./data-kuramoto` is fresh and dedicated.

### D-05: 10 Feynman seeds, all evaluable pairs
From `feynman_10pair_papers.json` schema v0.2:
- pair01: `cond-mat/0203227` (Dorogovtsev Ising) ↔ `0710.3256` (Castellano social)
- pair03: `cond-mat/0010317` (Pastor-Satorras SIR) ↔ `cond-mat/0312131` (Moreno rumour)
- pair04: `cond-mat/0007235` (Newman percolation) ↔ `cond-mat/0205009` (Newman SIR)
- pair05: `nlin/0202034` (Drossel food webs) ↔ `cond-mat/0002374` (Bouchaud wealth)
- pair06: `1005.1986` (Nakao Turing) ↔ `cond-mat/9801289` (Marsili Zipf)

Pair02 dropped (Hinton CD has no arXiv preprint). 5 evaluable pairs is enough for the `n_eval >= 3` notebook gate with margin.

### D-06: Phase passes iff notebook gate passes
Hard success criteria, identical to the notebook's internal `BENCH_V` validator:
1. `n_eval >= 3` (at least 3 of 5 pairs map to valid corpus IDs in different communities)
2. `BENCH_P10 > 0.15` (precision-at-10 above 0.15)
3. No `ABORT` raised in `BENCH_V`

If the gate fails, Phase 29 documents the failure mode in `29-VERIFICATION.md` and the dynamical-LBD direction must reconsider Path B (redesign for 2015+) or Path C (semantic-similarity edges) from `project_kuramoto_v03_status.md`.

### D-07: Release-mode binary
The crawl will run for ~30–90 min. Build `--release` for the SurrealDB serialization hot path, even though crawl throughput is HTTP-bound — release saves measurable CPU on writes.

### D-08: `--parallel 1` (sequential workers)
Discovered empirically during the first crawl attempt (2026-05-04 17:43). The default `concurrency=4` combined with `--bidirectional` triggers S2 429s even with a valid keyed API key. Root cause: `make_semantic_scholar_limiter` (rate_limiter.rs:42) sizes the Arc-shared external governor at 400ms/token = 2.5 papers/sec, assuming 2 API calls per paper (`fetch_paper + fetch_references`). Bidirectional adds a third call (`fetch_citing_papers`), pushing aggregate rate past S2's 5 rps keyed limit. `--parallel 1` (matching the precedent set by Phase 28's `crawl-feynman-seeds.sh`) serialises workers so a single source instance enforces 200ms-between-call internal pacing, keeping aggregate at ~5 rps.

### D-09: Lower `--max-forward-citations` from 500 to 50
Discovered during the second crawl attempt (2026-05-04 17:50). Bidirectional depth-2 expansion of seed 1 (cond-mat/0203227 — Pastor-Satorras Ising-on-networks, ~3000 citers on S2) produced **23,264 pending queue entries from a single seed**. At ~600ms/paper (3 API calls × 200ms keyed), this projects to ~40h total runtime across 10 seeds. Lowering cap to 50 reduces forward expansion 10× → estimated total ~3-4h while preserving enough topological signal for Louvain modularity to surface cross-domain bridges. The cap only bounds pagination at fetch time, so re-running on top of the 23k-entry queue would still process all entries — D-10 (fresh DB) follows.

### D-10: Fresh `data-kuramoto/` for cap=50 run
The existing `data-kuramoto/` (~61MB) contains queue entries from the killed cap=500 attempt that would be processed regardless of the new cap. `rm -rf data-kuramoto/` is the first step of the resume handoff (29-RESUME.md).

</decisions>

<canonical_refs>
## Canonical References

- `professional-vault/prototypes/kuramoto_lbd_v03.ipynb` — the notebook gate
- `professional-vault/prototypes/data/feynman_10pair_papers.json` — seed list (schema v0.2)
- `.planning/.continue-here.md` — original deferred plan (Tasks 5–8); superseded by this phase
- `~/.claude/projects/-home-jasper-Repositories-research-synergy/memory/project_kuramoto_v03_status.md` — root cause + alternative paths
- `resyn-core/src/data_aggregation/semantic_scholar_api.rs` — S2 source (Phase 28 includes `fetch_citing_papers`)
- `resyn-core/src/data_aggregation/rate_limiter.rs:43` — keyed/unkeyed governor switch
- `scripts/crawl-feynman-seeds.sh` — Phase 28 artifact, only 2 seeds; **do not modify**

</canonical_refs>
