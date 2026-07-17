#!/usr/bin/env python3
"""EXP-RS-22 (Phase 41) — analysis + total-function gate (C-48).

One-directional reasoning test. Decisive quantity on the CLEAN (memory-absent) stratum:
    d_i    = pctile_rank_bestnull(side_b_i) − pctile_rank_LLM(side_b_i)     (percentile rank, lower=better)
             bestnull = the better (lower rank) of the lexical-TF-IDF and RS-21 embedding nulls per pair
    REASON = median_i d_i            (>0  ⇒ LLM ranks the target better than the best memory-free method)
Primary test = one-sided paired Wilcoxon signed-rank that median(d − δ) > 0 (alternative='greater'),
plus a bootstrap CI on REASON; REASONING-CONFIRMED iff p<α AND CI-lower > δ, AND all posctrl gates hold.
MEMORIZATION is NOT an available outcome (near-non-identifiable at feasible n — two panels). Constants are
BLIND-authored + SHA-frozen in rs22_constants.json (K=50, δ=5, α=0.05, n_floor=110, ...). No mixed model
(panel MF8): the paired Wilcoxon on the clean stratum is the whole primary test.

Gate is a TOTAL function over (posctrl, REASON-vs-δ, CI, power); grid-verified by rs22_gate_selftest.py.

Usage:
  python rs22_gate.py --records data/rs22_clean_records.json --posctrl data/rs22_posctrl.json \
      --out data/rs22_verdict.json
"""
import argparse
import json
import os

import numpy as np
from scipy import stats

HERE = os.path.dirname(os.path.abspath(__file__))
DATA = os.path.join(HERE, "data")
CONST = json.load(open(os.path.join(DATA, "rs22_constants.json")))["constants"]

DELTA = CONST["delta_reason_margin_pts"]
ALPHA = CONST["alpha_one_sided"]
N_FLOOR = CONST["n_floor_clean_stratum_pairs"]
POSCTRL_FLOOR = CONST["posctrl_fire_floor"]
REPRO_LO, REPRO_HI = CONST["repro_band_recall_at_10"]
FALSEMISS_MAX = CONST["falsemiss_max"]
KAPPA_FLOOR = CONST["kappa_floor"]
MIN_FIELDS = 3  # clean stratum must not concentrate in <3 distinct field-pairs (panel; not a frozen const)


def reason_stats(d):
    """d = per-pair (rank_bestnull − rank_LLM); positive ⇒ LLM better. One-sided Wilcoxon vs δ + boot CI."""
    d = np.asarray(d, dtype=float)
    n = len(d)
    reason = float(np.median(d))
    # one-sided paired Wilcoxon: is median(d) > DELTA ?  test (d − DELTA) with alternative='greater'
    shifted = d - DELTA
    nz = shifted[shifted != 0]
    if len(nz) < 6:
        p = 1.0  # too few non-zero → cannot reject
    else:
        try:
            p = float(stats.wilcoxon(shifted, alternative="greater", zero_method="wilcox").pvalue)
        except ValueError:
            p = 1.0
    # bootstrap CI (one-sided lower bound) on the median via percentile bootstrap, deterministic seed
    rng = np.random.default_rng(0)
    boots = np.array([np.median(rng.choice(d, size=n, replace=True)) for _ in range(5000)])
    ci_lower = float(np.percentile(boots, 100 * ALPHA))          # one-sided (1−α) lower bound
    ci_upper = float(np.percentile(boots, 100 * (1 - ALPHA)))
    return {"REASON_median": reason, "mean": float(np.mean(d)), "n": n,
            "wilcoxon_p_onesided_gt_delta": p, "boot_ci_lower": ci_lower, "boot_ci_upper": ci_upper,
            "delta": DELTA, "alpha": ALPHA}


def posctrl_gate(pc):
    """Every precondition that must hold before REASON may be interpreted (C-47/C-48)."""
    checks = {
        "positive_control_fire>=floor": pc.get("posctrl_fire_rate", 0) >= POSCTRL_FLOOR,
        "negative_control_holds": bool(pc.get("negative_control_holds", False)),
        "pinned_retrieval_reproduces_incumbent":
            REPRO_LO <= pc.get("anchor_recall_at_10", -1) <= REPRO_HI,
        "audit_passed": bool(pc.get("audit_passed", False)),
        "false_miss<=max": pc.get("probe_false_miss_rate", 1.0) <= FALSEMISS_MAX,
        "clean_n>=floor": pc.get("n_clean", 0) >= N_FLOOR,
        "field_diverse": pc.get("n_distinct_field_pairs", 0) >= MIN_FIELDS,
        "rubric_kappa>=floor": pc.get("strength_kappa", 0) >= KAPPA_FLOOR,
    }
    return checks, all(checks.values())


def compute(records, posctrl):
    checks, posctrl_ok = posctrl_gate(posctrl)
    d = [r["rank_bestnull_pctile"] - r["rank_llm_pctile"] for r in records]
    rs = reason_stats(d) if len(d) >= 6 else {"REASON_median": None, "n": len(d),
                                              "wilcoxon_p_onesided_gt_delta": 1.0, "boot_ci_lower": None}
    reasoning = (posctrl_ok
                 and rs.get("wilcoxon_p_onesided_gt_delta", 1.0) < ALPHA
                 and rs.get("boot_ci_lower") is not None and rs["boot_ci_lower"] > DELTA)

    if not posctrl_ok:
        verdict = "INVALID/INCONCLUSIVE"
        rationale = ("A validity precondition failed (%s) — REASON may not be interpreted; fix + re-run."
                     % ", ".join(k for k, v in checks.items() if not v))
    elif reasoning:
        verdict = "REASONING-CONFIRMED"
        rationale = ("On the memory-absent clean stratum the pinned model beats max(lexical, embedding) "
                     "null by REASON=%.1f pts (p=%.3f < %.2f, boot CI-lower=%.1f > δ=%d): a real reasoning "
                     "component survives removing memory+lexical+embedding+familiarity+priming → the "
                     "RS-16→21 KILLs are AIRTIGHT → chapter CLOSES."
                     % (rs["REASON_median"], rs["wilcoxon_p_onesided_gt_delta"], ALPHA,
                        rs["boot_ci_lower"], DELTA))
    else:
        verdict = "INCONCLUSIVE"
        rationale = ("Cannot confirm a reasoning component at this power (REASON=%s, p=%.3f, CI-lower=%s "
                     "vs δ=%d). Explicitly NOT a memorization claim (not falsifiable here)."
                     % (rs.get("REASON_median"), rs.get("wilcoxon_p_onesided_gt_delta", 1.0),
                        rs.get("boot_ci_lower"), DELTA))
    return {"verdict": verdict, "rationale": rationale, "posctrl_ok": posctrl_ok,
            "posctrl_checks": checks, "reason": rs, "constants": CONST}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--records", default=os.path.join(DATA, "rs22_clean_records.json"))
    ap.add_argument("--posctrl", default=os.path.join(DATA, "rs22_posctrl.json"))
    ap.add_argument("--out", default=os.path.join(DATA, "rs22_verdict.json"))
    args = ap.parse_args()
    records = json.load(open(args.records))
    records = records.get("records", records) if isinstance(records, dict) else records
    posctrl = json.load(open(args.posctrl))
    out = compute(records, posctrl)
    json.dump(out, open(args.out, "w"), indent=1, ensure_ascii=False)
    print("=== EXP-RS-22 VERDICT ===")
    print(f"  posctrl_ok={out['posctrl_ok']}  " + "  ".join(f"{k}={v}" for k, v in out["posctrl_checks"].items()))
    r = out["reason"]
    print(f"  REASON(median)={r.get('REASON_median')}  n={r.get('n')}  "
          f"p={r.get('wilcoxon_p_onesided_gt_delta'):.3f}  CI-lower={r.get('boot_ci_lower')}  δ={DELTA}")
    print(f"\n  ==> {out['verdict']}\n      {out['rationale']}\n\nwrote {args.out}")


if __name__ == "__main__":
    main()
