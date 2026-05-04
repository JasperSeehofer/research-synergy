# Phase 29 — Resume Handoff

**Created:** 2026-05-04 (end of working session)
**Phase:** 29-kuramoto-corpus-build
**Plan:** 29-01 (paused after 2 aborted crawl attempts)
**Reason for handoff:** queue blowup from `--max-forward-citations 500` → ~40h total runtime estimate; user closing session

## Current state

| Item | Status |
|---|---|
| `.env` auto-load via dotenvy | ✅ Shipped (commit `2ceb9aa`) |
| Phase 29 planning artifacts | ✅ Shipped (commit `a628e4a`) |
| `--parallel 1` fix in script | ✅ Shipped (commit `c0202ce`) |
| `MAX_FWD` default lowered to 50 | ✅ In this commit |
| Crawl run | ❌ Killed twice; **not started** with cap=50 |
| Analyze | ⏳ Pending crawl |
| Export pre-2015 graph | ⏳ Pending analyze |
| Notebook benchmark gate | ⏳ Pending export |

## Why crawl was paused

- **Attempt 1** (parallel=4, cap=500): 429s within 3 minutes. Root cause: external rate-limiter sized for 2 calls/paper but bidirectional adds a 3rd. Killed and committed `--parallel 1` fix.
- **Attempt 2** (parallel=1, cap=500): No 429s, but seed 1's bidirectional depth-2 expansion produced **23,264 pending queue entries** for a single seed. Projected total runtime: ~40h across all 10 seeds. Not feasible in a working session.
- **Decision:** lower cap to 50 (10× less forward expansion per paper) → estimated total runtime ~3–4h.

## ⚠️ Existing `data-kuramoto/` is poisoned

The current `data-kuramoto/` directory on disk (~61MB) contains the BFS queue from the killed cap=500 attempt, including ~23k pending entries that were enqueued under the old high cap. **Re-running on top of it would still process all 23k entries** because the cap only affects forward-citation pagination at fetch time — already-enqueued entries get processed regardless.

**First action in next session:**

```bash
rm -rf data-kuramoto/
```

This is safe: gitignored, no committed artifacts, only contains aborted-crawl data.

## Resume commands (next session)

```bash
# 0. Verify env
echo "S2_API_KEY set: ${S2_API_KEY:+yes}"   # should print "yes" (auto-loaded from .env at binary startup)

# 1. Nuke the poisoned partial DB
rm -rf data-kuramoto/

# 2. Run the crawl (cap=50 default, ~3-4h estimate, --parallel 1)
./scripts/crawl-feynman-pairs.sh surrealkv://./data-kuramoto > /tmp/phase-29-crawl.log 2>&1 &
CRAWL_PID=$!
echo "Crawl PID: $CRAWL_PID"

# Watch progress (one event per seed transition / completion / 429 / error):
tail -F /tmp/phase-29-crawl.log | grep -E --line-buffered "=== Seeding|Crawl complete|429|panicked|^Error:"

# 3. After all 10 seeds complete (look for "All 10 Feynman seeds complete"), run analysis
./target/release/resyn analyze --db surrealkv://./data-kuramoto 2>&1 | tee /tmp/phase-29-analyze.log

# 4. Export pre-2015 graph
./target/release/resyn export-louvain-graph \
  --db surrealkv://./data-kuramoto \
  --output professional-vault/prototypes/data/research_synergy_pre2015.json \
  --published-before 2014-12-31 \
  --tfidf-top-n 50

# 5. Pair-presence checkpoint (the python one-liner from 29-01-PLAN.md Step 4)
#    Halt and write VERIFICATION.md FAIL if n_eval < 3

# 6. Run the notebook (the benchmark gate)
jupyter nbconvert --to notebook --execute \
  professional-vault/prototypes/kuramoto_lbd_v03.ipynb \
  --output kuramoto_lbd_v03.executed.ipynb \
  2>&1 | tee /tmp/phase-29-notebook.log

# 7. Write 29-VERIFICATION.md with BENCH_V output, BENCH_P10, pass/fail
```

## If cap=50 still produces too-large a queue

Fallback hierarchy (try in order):

1. **Drop cap further:** `MAX_FWD=20 ./scripts/crawl-feynman-pairs.sh ...`
2. **Drop depth to 1:** Edit `scripts/crawl-feynman-pairs.sh` line `--max-depth 2` → `--max-depth 1`. Loses depth-2 cross-domain bridges but keeps direct seed neighbourhoods.
3. **Subset seeds:** Only run pair01 + pair03 (the two highest-quality pairs from `feynman_10pair_papers.json`). Need n_eval ≥ 3 from 5 pairs to pass; with 2 pairs, gate is unreachable. So this only makes sense as a smoke test before scaling up.

## If notebook gate fails

The dynamical-LBD direction is empirically falsified for this corpus shape. Document in `29-VERIFICATION.md` and revisit the three paths from `~/.claude/projects/-home-jasper-Repositories-research-synergy/memory/project_kuramoto_v03_status.md`:

- **Path B:** Redesign the 10-pair benchmark for 2015+ era (HTML-crawlable references)
- **Path C:** Drop citation graph entirely; build edges from TF-IDF cosine similarity > threshold

Either path is a new phase, not a continuation of 29.

## Post-execution: write VERIFICATION.md

After the notebook completes (pass or fail), populate `.planning/phases/29-kuramoto-corpus-build/29-VERIFICATION.md` with:

- Crawl stats (final paper count, edge count, runtime, 429 retries)
- Pair-presence breakdown (5 pairs × A/B sides × community ID)
- Notebook gate result (`BENCH_V` output verbatim, `BENCH_P10` value)
- Pass/Fail verdict
- If FAIL: which path forward (B / C)

Then commit with message style `docs(phase-29): complete plan — Kuramoto-LBD v03 corpus build [PASS|FAIL]`.

## Files to read at start of next session

1. `.planning/STATE.md` (current focus)
2. `.planning/phases/29-kuramoto-corpus-build/29-CONTEXT.md` (decisions D-01 through D-08)
3. `.planning/phases/29-kuramoto-corpus-build/29-01-PLAN.md` (full step-by-step)
4. **This file** (handoff specifics)
5. `~/.claude/projects/-home-jasper-Repositories-research-synergy/memory/project_kuramoto_v03_status.md` (root cause history)
