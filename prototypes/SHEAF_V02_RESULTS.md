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
| T2 precision@10 | precision@10 | 0.400 | ≥ 0.4 | PASS |
| T4 ablation | pass rate | 0.00 | ≥ 0.5 | FALSIFIED |
| SC4 spectral gap | λ₂₁/λ₂₀ | N/A | ≥ 1.1 | FLAG |
| SC5 Jaccard | J(sheaf, cosine) | 0.023 | ≤ 0.8 | PASS |

## Decision

**FALSIFIED** — T4 < 20% — multi-causal thesis falsified; RAF-LBD remains the only multi-causal candidate.

## T4 ablation detail

| Bridge | i* | λ* | ratio | Status |
|--------|----|----|-------|--------|
| comm0↔comm2 | -1 | 0.0000 | 0.00 | FAIL |
| comm0↔comm3 | -1 | 0.0000 | 0.00 | FAIL |
| comm0↔comm4 | -1 | 0.0000 | 0.00 | FAIL |
| comm0↔comm5 | -1 | 0.0000 | 0.00 | FAIL |
| comm0↔comm6 | -1 | 0.0000 | 0.00 | FAIL |

