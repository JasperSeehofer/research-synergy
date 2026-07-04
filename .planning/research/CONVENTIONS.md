# Conventions ledger — Dynamical LBD thread

*Append-only convention lock (Layer-2 pack anatomy, vault:
`wiki/analyses/research-routine-packs-spec.md`). Every experiment/artifact declares which
conventions it uses; changing a CRITICAL row requires re-verifying all downstream consumers.
Never edit rows — append a superseding row and mark the old one.*

| # | convention | value | set by | criticality |
|---|---|---|---|---|
| C-1 | Pre-2015 cutoff | `--published-before 2014-12-31` (inclusive, lexicographic on ISO dates) | Phase 29 | CRITICAL |
| C-2 | Corpus | `data-kuramoto/` — 421 papers, 35 Louvain communities (bidirectional S2 crawl from 10 Feynman-pair seeds) | Phase 29 | CRITICAL |
| C-3 | Benchmark gate | shared 10-pair Feynman-reduction set; evaluable pair = maps to non-Other communities; gate `n_eval ≥ 3` AND `BENCH_P10 > 0.15` (EXP-RS-06 baseline; ≥ 0.30 = success) | Phase 29 roadmap gate | CRITICAL |
| C-4 | Node vectors | c-TF-IDF, top-50 terms per node (`--tfidf-top-n 50`), from `export-louvain-graph` | Phase 29 | high |
| C-5 | Export schema | `{louvain_params, corpus_fingerprint, nodes, communities, edges}`; "Other" community excluded; uniform edge weight 1.0 | Phase 28/29 | high |
| C-6 | EXP-RS-11 τ sweep | cosine threshold τ ∈ {0.2, 0.3, 0.4, 0.5}; headline prediction at τ=0.3; connectivity precheck `n_cc/N ≤ 0.05` required before `compute_K_stable` | EXP-RS-11 pre-registration | CRITICAL |
| C-7 | Notebook workspace | LBD notebooks + exports live in `../professional-vault/prototypes/` (`kuramoto_lbd_v0*.ipynb`, `data/`); Rust-side prototypes in `prototypes/` here | Phase 29 practice | high |
| C-8 | EXP-RS-11 edge rule | undirected edge (i<j) iff `cosine(v_i, v_j) >= τ` (inclusive), edge weight = cosine value, isolated nodes retained as singleton components (`build_tfidf_graph.py`, vault commit `4e9b7dc`) | Phase 30 | CRITICAL |
| C-9 | v04 substrate artifact | v04 notebook consumes `data/tfidf_graph_pre2015.json` (single source of truth for the semantic substrate) and asserts `artifact.input_sha256 == sha256(export)`; pre-2015 export sha256 `8e92a433…f391` | Phase 30 | high |
| C-10 | Export tie-break caveat | `export-louvain-graph` node `tfidf_vec` top-N truncation is NON-deterministic among equal-score terms (HashMap iteration order; 4/153 nodes affected at top-50 boundary). Canonical pre-2015 export = the committed Phase 29 artifact (fingerprint `bbaa202d…`); regeneration reproduces fingerprint/edges/communities exactly, node vectors only up to ties. Do not regenerate mid-experiment. | Phase 30 (found during CL-6 repro check) | high |
| C-11 | Python env | `prototypes/.venv` pinned via `prototypes/requirements-lock.txt` (python 3.13.12, numpy 2.5.0, scipy 1.18.0, networkx 3.6.1); `compute_K_stable` wall-clock budget = 300 s (v04) | Phase 30 | medium |
| C-12 | **Supersedes C-1** — benchmark framing | The BENCH_P10 gate (C-3) is a **date-agnostic recovery** metric: it ranks global Fiedler bridge edges and checks top-10 against benchmark community-pairs; the scoring code uses no dates. Per `dynamical-lbd.md` Criterion 3(a) the corpus must "contain both literatures" — a temporal pre-2015 holdout (C-1) is NOT required and is the separate EXP-RS-02 predict-the-future test the notebooks do not run. C-1 is retired for the recovery benchmark; predict-the-future runs must re-invoke it explicitly. Verified 2026-07-04 by reading v03/v04 scoring cells + concept pages. | Phase 31 (EXP-RS-12) | CRITICAL |
| C-13 | EXP-RS-12 substrate | Citation-edge adjacency (export `edges`, uniform weight) on the **largest connected component** of the full `data-kuramoto` corpus (`research_synergy_kuramoto_full.json`, N=227 → giant CC 224). Run dynamics on the giant CC only (`n_cc=1` → `λ₂(L_uw) > 0`; even 2 components zero λ₂ and diverge `K_hi`). Precheck: giant-CC frac ≥ 0.95 AND `n_eval ≥ 3` in the giant CC. | Phase 31 (EXP-RS-12) | CRITICAL |
