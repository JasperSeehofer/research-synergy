# EXP-RS-07 Results — Sheaves-LBD v0.1

**Experiment:** EXP-RS-07 — Cellular sheaf bridge detection on the Louvain community graph.
**Status:** stage-A only (Feynman corpus pending)
**Date:** 2026-04-20

## Test results

| Test | Metric | Value | Target | Status |
|------|--------|-------|--------|--------|
| T1 toy | top-3 ∋ ground-truth edge | yes | yes | PASS |
| SC1 H⁰(F) | dim(ker L_F) on toy | 22 | ≤ 5 | FLAG (expected on toy) |
| SC2 symmetry | ‖L−Lᵀ‖_F/‖L‖_F | 0.00e+00 | < 1e-6 | PASS |
| SC3 PSD | min non-zero eigenvalue | 2.00e+00 | ≥ −1e-8 | PASS |
| T2 precision@10 | precision@10 | PENDING | ≥ 0.4 | HOLD |
| T4 ablation | pass rate | PENDING | ≥ 0.5 | HOLD |
| SC4 spectral gap | λ₂₁/λ₂₀ | PENDING | ≥ 1.1 | HOLD |
| SC5 Jaccard | J(sheaf, cosine) | PENDING | ≤ 0.8 | HOLD |

## Decision

**HOLD** — Stage A complete (T1 PASS, SC2/SC3 PASS). Feynman corpus crawl pending. Re-run after `cargo run --bin resyn -- analyze` and `export-louvain-graph`.

