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
