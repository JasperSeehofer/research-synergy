# EXP-RS-07 Results — Sheaves-LBD v0.1

**Experiment:** EXP-RS-07 — Cellular sheaf bridge detection on the Louvain community graph.
**Status:** completed
**Date:** 2026-04-20

## Test results

| Test | Metric | Value | Target | Status |
|------|--------|-------|--------|--------|
| T1 toy | top-3 ∋ ground-truth edge | yes | yes | PASS |
| SC1 H⁰(F) | dim(ker L_F) on toy | 22 | ≤ 5 | FLAG (expected on toy) |
| SC2 symmetry | ‖L−Lᵀ‖_F/‖L‖_F | 0.00e+00 | < 1e-6 | PASS |
| SC3 PSD | min non-zero eigenvalue | 2.00e+00 | ≥ −1e-8 | PASS |
| T2 precision@10 | precision@10 | 0.000 | ≥ 0.4 | FAIL |
| T4 ablation | pass rate | N/A | ≥ 0.5 | VOID |
| SC4 spectral gap | λ₂₁/λ₂₀ | N/A | ≥ 1.1 | FLAG |
| SC5 Jaccard | J(sheaf, cosine) | 0.000 | ≤ 0.8 | PASS |

## Decision

**HOLD** — Stage B corpus insufficient (< 2 communities or 0 inter-community edges). Re-crawl with a larger, multi-domain corpus and re-run.

