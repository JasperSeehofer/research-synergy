# Phase 30 — Verification (Plan 30-01, EXP-RS-11)

**Date:** 2026-07-04
**Verdict:** ❌ **FAIL** — connectivity precheck fails at all τ; the pivot kill gate FIRES.
**Falsification:** ✅ FAIL verdict independently CONFIRMED (3 converging lines; see § Independent falsification).
**Decision routing:** the kill decision itself is the human's, via the vault — not patched locally.

---

## Executed computational evidence

All artifacts committed to the vault (`professional-vault/prototypes/`), scripts committed
**before** their real-data run per research-operating-manual discipline:

- `build_tfidf_graph.py` — vault commit `4e9b7dc` (committed before real-data run)
- `kuramoto_lbd_v04.ipynb` — vault commit `df6956d` (committed before execution)
- executed evidence (`kuramoto_lbd_v04.executed.ipynb` + `data/tfidf_graph_pre2015.json`) — vault commit `0fbb4ac`
- input export `data/research_synergy_pre2015.json` — sha256 `8e92a433…`, corpus fingerprint
  `bbaa202d79b6b775ae120b4bbb012faf209741db9e0a7d1e8c37a07b9adc9bec`, N=153 nodes, 34 communities.

### TF-IDF cosine τ-sweep (executed notebook, cell 2 / cell 17)

Undirected edge iff `cosine(v_i, v_j) >= τ` on per-node `tfidf_vec` (convention C-8):

| τ | n_edges | n_cc | largest_cc | n_cc/N | precheck (`n_cc/N ≤ 0.05`) |
|---|---|---|---|---|---|
| 0.20 | 129 | 68 | 37 | 0.444 | **FAIL** |
| **0.30** | **27** | **127** | **5** | **0.830** | **FAIL** |
| 0.40 | 8 | 146 | 3 | 0.954 | **FAIL** |
| 0.50 | 1 | 152 | 2 | 0.993 | **FAIL** |

**Precheck fails at every pre-registered τ.** Dynamics cells (`compute_K_stable`, benchmark)
correctly SKIP; `BENCH_P10` is not produced (this is the pre-registered behavior, not a crash).

### Locked predictions (EXP-RS-11) — evaluated, not adjusted

| # | Locked prediction | Actual | Result |
|---|---|---|---|
| P-1 | τ=0.3: `n_cc/N ≤ 0.05` | 0.830 (127 cc / 153) | ❌ NOT MET |
| P-2 | τ=0.3: largest CC ≥ 80% | 3.3% (5 / 153) | ❌ NOT MET |
| P-3 | `compute_K_stable` completes ≤ 300 s | NOT RUN (precheck failed) | ❌ NOT MET |
| P-4 | gate `n_eval ≥ 3` AND `BENCH_P10 > 0.15` | n_eval=4; `BENCH_P10` not produced | ❌ NOT MET |

### The premise is falsified, not merely unmet

EXP-RS-11's hypothesis was that TF-IDF cosine edges would be **more** connected than the S2
citation graph (Phase 29 baseline: 41 cc / 153 = `n_cc/N` = 0.268, largest CC 38%). Observed:
the TF-IDF graph is **more fragmented than the citation graph at every pre-registered τ**
(τ=0.2 → 0.444; τ=0.3 → 0.830). The substrate swap did not just miss the target — it moved
connectivity in the *wrong direction*. Driver: mean pairwise cosine = 0.0305; only 1.1% of the
11,628 node pairs reach ≥ 0.2 (0.23% ≥ 0.3). The similarity mass is not there.

---

## Claims → acceptance-tests contract (from 30-01-PLAN.md)

| # | Claim | Acceptance test | Result |
|---|---|---|---|
| CL-1 | τ=0.3 gives `n_cc/N ≤ 0.05` | sweep row τ=0.3 `n_cc ≤ 7` | ❌ n_cc=127 — **claim false** |
| CL-2 | τ=0.3 largest CC ≥ 80% (≥123) | sweep row τ=0.3 | ❌ largest_cc=5 — **claim false** |
| CL-3 | `compute_K_stable` ≤ 300 s, finite | v04 K_stable cell | ⏭ not reached (precheck gate) |
| CL-4 | real `BENCH_P10`; gate evaluated | executed notebook | ❌ gate NOT met; either outcome valid — this is the FAIL outcome |
| CL-5 | no NEW leakage / no benchmark-pair contamination | adversarial audit | ✅ PASS (see falsification C) |
| CL-6 | pre-2015 export reproducible | fingerprint == `bbaa202d…`, 153/180 | ✅ PASS (with C-10 tie-break caveat) |
| CL-7 | connectivity stats correct | independent re-computation matches | ✅ PASS — 3 independent recomputes match exactly |

---

## Independent falsification (right-sized `/commission --research`, human-approved 2026-07-04)

Falsification-first: three independent lines were tasked to **break** the FAIL — i.e. to *connect*
the substrate or find a bug that artificially under-connects it. All three converge on the FAIL.

| Line | Method (independence) | n_cc/N @ τ=0.3 | largest_cc | outcome |
|---|---|---|---|---|
| Blind A | pure-Python union-find, manual sparse cosine; **never shown** the script or verdict | 0.830 (127 cc) | 5 | reproduces FAIL |
| Blind B | scipy/sklearn csr matrix + `connected_components`; **never shown** the script or verdict | 0.830 (127 cc) | 5 | reproduces FAIL |
| Adversarial C | 8-point under-connection bug-hunt on `build_tfidf_graph.py` + leakage/contamination audit + independent BFS cross-check | 0.830 (127 cc) | 5 | script correct; no bug |

Blind A and B independently reproduced the notebook's numbers to the digit across all four τ.
Adversarial C verified all eight under-connection channels OK (cosine normalization, threshold
direction `>=`, sparse-vector alignment, zero/empty handling, top-50 truncation interaction,
undirected component counting), and confirmed **no temporal leakage** and **no benchmark-pair
contamination** (the script reads only the export; never opens the Feynman-pair files present in
the same `data/` dir). Convergence ≥ 2 lines → **CONFIRMED**.

**Standing typed feedback (raise-consideration / suggest-direction, NOT a verdict-changer):**
the only lever that could challenge fragmentation is upstream — the export's top-50 term
truncation (`--tfidf-top-n 50`, convention C-4) is genuinely lossy (a shared term below one
node's cutoff loses that overlap). Re-exporting with much larger / untruncated `tfidf_vec`
*might* raise cosine mass. This is a property of the substrate/export contract, not a defect in
`build_tfidf_graph.py`, and it does **not** change this pre-registered τ=0.3-at-top-50 result
(predictions are locked). It is recorded for the human/vault as a possible future direction.

---

## Verdict

❌ **FAIL** — the TF-IDF cosine semantic-edge substrate does not connect the pre-2015 corpus at
any pre-registered τ; it is in fact more fragmented than the citation graph it replaced. The
`n_cc/N ≤ 0.05` precheck is not met, so `compute_K_stable` and `BENCH_P10` are (correctly) not
reached. Per the time-bound pivot kill gate (set 2026-07-02, human-approved): **the dynamical-
substrate line's kill gate has FIRED** — `<3 evaluable pairs or BENCH_P10 ≤ 0.15 by 2026-09-30`
is satisfied by "BENCH_P10 not producible at all." Per EXP-RS-11's pre-registered follow-ups,
the corpus itself is too narrow for TF-IDF semantic edges; **Path B (seed selection / newer
corpus) is the remaining option.** The go/kill/pivot decision returns to the human via the vault.

This is an honest negative. The verdict was reached under blind, pre-registered conditions and
survived independent falsification — it is a real result, not a null.

---

## Artifacts

- ✅ `build_tfidf_graph.py` (vault `4e9b7dc`, committed before real-data run)
- ✅ `kuramoto_lbd_v04.ipynb` (vault `df6956d`) + `.executed.ipynb` (vault `0fbb4ac`)
- ✅ `data/tfidf_graph_pre2015.json` (substrate artifact, vault `0fbb4ac`)
- ✅ `data/research_synergy_pre2015.json` (input export; sha256 `8e92a433…`, fingerprint `bbaa202d…`)
- ✅ Independent-falsification record: `results/commission_research_2026-07-04/REPORT.md` (this repo)
- ✅ Convention locks C-8…C-11 (`.planning/research/CONVENTIONS.md`, commit `0509c17`)
- ✅ This file

---

## Pack feedback — first field test of the software-research routine pack

Per the kickoff's feedback obligation (vault `research-routine-packs-spec.md` defers the pack's
design to this pivot). Concrete notes:

- **THREAD.md / CONVENTIONS.md carried real weight.** The append-only convention lock (C-6 τ set,
  C-4 top-50) is what made "the τ sweep is sensitivity analysis, not tuning" enforceable — there
  was no room to quietly widen τ below 0.2 to rescue the result. The locked-predictions table in
  the PLAN made the FAIL mechanical to adjudicate. This is the pack's strongest feature.
- **The claims→acceptance-tests contract worked.** CL-1…CL-7 mapped one-to-one to executable
  checks; writing the verdict was transcription, not argument. Recommend this become a required
  PLAN section for every EXP-RS-* phase.
- **Cross-repo split is the main friction.** Thread state lives in `research-synergy/.planning/`
  but every artifact (scripts, notebooks, exports) lives in `professional-vault/prototypes/`.
  `/commission --research`'s default (freeze a base tag in `$PWD`, give investigators that repo)
  does not fit — the code under audit is in the *other* repo. I right-sized around this by
  spawning blind agents pointed at the vault export directly. **Pack gap:** the research-commission
  manifest needs a first-class `artifact_repo` / `artifact_paths` field distinct from the
  thread-state repo. Flagging for the Layer-2 spec.
- **An algorithm-change gate would NOT have caught anything here** — the failure is upstream of
  any algorithm, in substrate connectivity. The precheck (`n_cc/N ≤ 0.05` before `compute_K_stable`)
  is the gate that mattered, and it fired correctly. Recommend the pack promote "connectivity /
  well-posedness precheck before the expensive dynamical step" to a named, reusable guardrail
  (it has now caught the same class of failure twice: Phase 29 K_stable divergence, Phase 30
  precheck).
- **`--dry` on the commission would have helped** surface the cross-repo/budget issue before
  spawning; the right-sized manual falloff was the right call for a deterministic negative but
  the decision took a round-trip. A `research_artifact_repo` field + a documented "negative-result
  → lightweight recompute-convergence" recipe would streamline the next kill-gate verification.
