# Commission (RESEARCH mode) — Dynamical-LBD (Gen-4) thread — 2026-07-04

**Mode:** right-sized independent falsification (human-approved 2026-07-04). The heavyweight
6-phase `commission-research.workflow.js` was deliberately not run: the claim under review is a
*deterministic negative* (a connectivity computation), and the artifacts live in a separate repo
(`professional-vault/prototypes/`), which the default `$PWD`-tag-freeze flow does not fit. The
right-sized falloff maps onto pre-registered acceptance tests CL-5 (leakage/contamination audit)
and CL-7 (independent connectivity re-computation).

**Reporting period:** Phase 30 / EXP-RS-11 (since Phase 29 FAIL, 2026-05-05).
**Min converging lines:** 2. **Independence:** blind agents A/B were never shown
`build_tfidf_graph.py`, the v04 notebook, or any verdict/verification file; ground truth (the
expected sweep numbers) was withheld.

## Headline verdict

**The period's claimed progress SURVIVES falsification — CONFIRMED FAIL.** The Phase 30 FAIL
verdict (TF-IDF cosine semantic-edge substrate fails the connectivity precheck at every
pre-registered τ; pivot kill gate fires) is correct and reproducible.

## Claim audited

| id | statement | verdict |
|---|---|---|
| substrate-precheck-fail | On the pre-2015 153-node export, the TF-IDF cosine graph fails `n_cc/N ≤ 0.05` at every τ ∈ {0.2,0.3,0.4,0.5}; at τ=0.3, `n_cc/N`=0.830 and largest CC=3.3% → EXP-RS-11 kill gate fires | **CONFIRMED** (3 converging lines) |

## Converging lines

1. **Blind recompute A** (pure-Python union-find, manual sparse cosine, no library CC): τ=0.3 →
   127 cc, largest 5, `n_cc/N`=0.830. All four τ rows match the notebook.
2. **Blind recompute B** (scipy `csr_matrix` + sklearn `cosine_similarity` + scipy
   `connected_components`, independent code path): identical to A and the notebook, to the digit.
3. **Adversarial bug-hunt C** (8-point under-connection audit of `build_tfidf_graph.py` +
   independent BFS cross-check): script correct on all 8 channels; fragmentation is a real data
   property (mean pairwise cosine 0.0305; 1.1% of pairs ≥0.2). No leakage, no contamination.

Convergence = 3 ≥ 2 → CONFIRMED.

## Regressions vs prior claim-history

None. This is the first commission run on this thread; `commission_history.jsonl` is initialized
by this report. Prior claim ("TF-IDF edges make the corpus connected enough", predicted—untested)
is now resolved to FALSIFIED with no contradiction against any earlier verified claim. The
adjacent Phase 29 verified claim (citation graph fragments: 41 cc / 153) is *reinforced*, not
contradicted — both substrates fragment.

## Typed feedback

- **raise-consideration** — The export's top-50 term truncation (`--tfidf-top-n 50`, convention
  C-4) is genuinely lossy and is the one channel that could, in principle, deflate cosine mass. It
  does NOT change the locked τ=0.3-at-top-50 result (predictions are locked; the audit found the
  script faithfully measures what it is given), but it is the honest caveat on the corpus verdict.
- **suggest-direction** — Before declaring the corpus permanently unusable under Path B, a cheap
  pre-check is to re-export with much larger / untruncated `tfidf_vec` and re-run the τ sweep. If
  connectivity is still absent, the "corpus too narrow" conclusion hardens; if not, top-N
  truncation was a confound worth documenting for all EXP-RS-* exports.
- **suggest-change** (process, to the Layer-2 pack) — the research-commission manifest needs a
  first-class `artifact_repo` / `artifact_paths` field distinct from the thread-state repo; the
  cross-repo split (state here, artifacts in the vault) broke the default freeze-`$PWD` flow.

## Decision log

- Chose right-sized falsification over the full workflow: deterministic negative + cross-repo
  artifacts + cost. Human-approved via AskUserQuestion 2026-07-04.
- Verdict CONFIRMED at convergence 3; no INCONCLUSIVE fallback needed.

## Proposals

See `.commission-notes.md` (propose-only; not applied).
