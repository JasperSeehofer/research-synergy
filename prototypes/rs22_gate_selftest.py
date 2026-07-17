#!/usr/bin/env python3
"""EXP-RS-22 gate self-test — grid-verify rs22_gate.compute is a TOTAL function (C-45 pattern).

Enumerates (posctrl pass/fail × REASON regime) and asserts each maps to exactly one of the 3 valid
verdicts with no exception; plus hand-pinned mappings. Offline; no benchmark, no mining.
"""
import sys

import numpy as np

from rs22_gate import compute, DELTA, N_FLOOR

VALID = {"INVALID/INCONCLUSIVE", "REASONING-CONFIRMED", "INCONCLUSIVE"}


def records(n, d_center, d_sd, seed):
    """n clean-stratum records with per-pair d = rank_bestnull − rank_llm ~ N(d_center, d_sd)."""
    rng = np.random.default_rng(seed)
    out = []
    for i in range(n):
        d = rng.normal(d_center, d_sd)
        llm = float(np.clip(rng.uniform(0, 60), 0, 100))
        bestnull = float(np.clip(llm + d, 0, 100))
        out.append({"rank_llm_pctile": llm, "rank_bestnull_pctile": bestnull})
    return out


def posctrl(ok=True, **override):
    pc = {"posctrl_fire_rate": 0.95, "negative_control_holds": True, "anchor_recall_at_10": 0.6,
          "audit_passed": True, "probe_false_miss_rate": 0.05, "n_clean": 120,
          "n_distinct_field_pairs": 10, "strength_kappa": 0.70}
    if not ok:
        pc["n_clean"] = 50  # below floor → posctrl fails
    pc.update(override)
    return pc


def run():
    seen = {}
    n = 0
    for pc_ok in (True, False):
        for d_center in (0.0, 3.0, 8.0, 25.0):   # null, sub-δ, just-over-δ, strong
            for d_sd in (5.0, 20.0):
                for nrec in (120, 150):
                    recs = records(nrec, d_center, d_sd, seed=hash((d_center, d_sd, nrec)) % 2**32)
                    out = compute(recs, posctrl(pc_ok, n_clean=nrec if pc_ok else 50))
                    v = out["verdict"]
                    assert v in VALID, f"invalid verdict {v}"
                    seen[v] = seen.get(v, 0) + 1
                    n += 1
    print(f"grid cells: {n}  verdicts: {dict(sorted(seen.items()))}")

    checks = []
    # posctrl pass + strong reasoning → REASONING-CONFIRMED
    v = compute(records(120, 25.0, 8.0, 1), posctrl(True))["verdict"]
    checks.append(("posctrl ok + strong d → REASONING", v == "REASONING-CONFIRMED", v))
    # posctrl pass + null d → INCONCLUSIVE (not memorization)
    v = compute(records(120, 0.0, 8.0, 2), posctrl(True))["verdict"]
    checks.append(("posctrl ok + null d → INCONCLUSIVE", v == "INCONCLUSIVE", v))
    # posctrl pass + d just under δ → INCONCLUSIVE
    v = compute(records(120, 2.0, 6.0, 3), posctrl(True))["verdict"]
    checks.append(("posctrl ok + sub-δ d → INCONCLUSIVE", v == "INCONCLUSIVE", v))
    # posctrl FAIL (n<floor) + strong d → INVALID (cannot interpret)
    v = compute(records(120, 25.0, 8.0, 4), posctrl(False))["verdict"]
    checks.append(("posctrl fail (n<floor) → INVALID", v == "INVALID/INCONCLUSIVE", v))
    # posctrl fail via reproduction band (anchor recall = null 0.40) → INVALID
    v = compute(records(120, 25.0, 8.0, 5), posctrl(True, anchor_recall_at_10=0.40))["verdict"]
    checks.append(("anchor recall=0.40 (=null) → INVALID", v == "INVALID/INCONCLUSIVE", v))
    # posctrl fail via audit not passed → INVALID
    v = compute(records(120, 25.0, 8.0, 6), posctrl(True, audit_passed=False))["verdict"]
    checks.append(("audit not passed → INVALID", v == "INVALID/INCONCLUSIVE", v))

    ok = True
    print("\nhand-pinned checks:")
    for name, passed, got in checks:
        print(f"  [{'PASS' if passed else 'FAIL'}] {name}  (got {got})")
        ok = ok and passed
    print(f"\n(constants: δ={DELTA}, n_floor={N_FLOOR})")
    print("==> RS22 GATE SELFTEST", "PASS" if ok else "FAIL")
    return ok


if __name__ == "__main__":
    sys.exit(0 if run() else 1)
