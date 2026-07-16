#!/usr/bin/env python3
"""EXP-RS-21 gate self-test — grid-verify embed_gate.compute is a TOTAL function (MUST-2).

Enumerates every consistent combination of the decision inputs, builds a synthetic per-corpus result
pair, and asserts compute() returns exactly one of the 6 valid verdicts with no exception. Also pins a
few hand-checked mappings so a logic regression is caught. Runs offline; no models, no benchmark.
"""
import itertools
import sys

from embed_gate import compute, MODERN_FLOOR

VALID = {"INVALID-headline-broken", "WEAK-no-bank", "ADVANCE", "PIVOT", "WEAK-PIVOT", "KILL"}
FLOOR = MODERN_FLOOR


def mk_model(recall10, pair04_top10=False, null_missed=0, modern=1.0, ctrl=0.0, card_pass=False):
    """One model's Feynman + modern record; strictP1 derived consistently (needs R>0.40)."""
    strict = (recall10 > 0.40) and pair04_top10 and (null_missed >= 1)
    feyn = {
        "forward": {"recall@10": recall10, "ranks": {}},
        "strict_P1": {"recall@10": recall10, "beats_null_0.40": recall10 > 0.40,
                      "pair04_rank": 5 if pair04_top10 else 20, "pair04_top10": pair04_top10,
                      "null_missed_recovered": ["pair04-percolation-epidemics"] * (1 if null_missed else 0),
                      "strict_P1_pass": strict},
        "cards": {"pair04-percolation-epidemics": {"objective_pass": card_pass}},
        "random_control": {"control_pass_rate": ctrl},
    }
    mod = {"forward": {"recall@10": modern}}
    return feyn, mod


def build(bge, gte, spec, mist):
    feyn = {"n_eval_pairs": 5, "models": {}}
    modern = {"models": {}}
    for name, spec_tuple in (("bge", bge), ("gte", gte), ("specter", spec), ("mistral", mist)):
        f, m = spec_tuple
        feyn["models"][name] = f
        modern["models"][name] = m
    return feyn, modern


def run():
    seen = {}
    n = 0
    # exhaustive grid over the drivers
    R_grid = [0.0, 0.2, 0.4, 0.6, 0.8, 1.0]
    for R in R_grid:
        for bge_p04 in (False, True):
            for bge_nm in (0, 1):
                for bge_modern in (0.5, FLOOR, 1.0):
                    for bge_toy in (False, True):
                        for bge_ctrl in (0.1, 0.5):
                            for bge_card in (False, True):
                                # a 'winner' local model (specter) that may or may not pass strictP1+live
                                for win in (False, True):
                                    for win_modern in (0.5, 1.0):
                                        for p4 in (False, True):
                                            bge = mk_model(R, bge_p04, bge_nm, bge_modern, bge_ctrl, bge_card)
                                            spec = mk_model(0.6 if win else 0.2, win, 1 if win else 0,
                                                            win_modern, 0.0, True)
                                            gte = mk_model(0.6 if p4 else 0.2, p4, 1 if p4 else 0,
                                                           1.0, 0.0, True)
                                            mist = mk_model(0.2, False, 0, 1.0, 0.0, False)
                                            toy = {"bge": bge_toy, "specter": True, "gte": True}
                                            feyn, modern = build(bge, gte, spec, mist)
                                            out = compute(feyn, modern, toy)
                                            v = out["verdict"]
                                            assert v in VALID, f"invalid verdict {v}"
                                            seen[v] = seen.get(v, 0) + 1
                                            n += 1
    print(f"grid cells checked: {n}")
    print("verdicts reached:", {k: seen[k] for k in sorted(seen)})
    for v in VALID:
        if v not in seen:
            print(f"  NOTE: verdict {v!r} unreached by the grid (may be fine if construction can't hit it)")

    # hand-pinned mappings
    checks = []
    # all fail, bge live but no strictP1, no class winner -> KILL
    bge = mk_model(0.4, False, 0, 1.0, 0.1, False)
    spec = mk_model(0.2, False, 0, 1.0, 0.0, False)
    gte = mk_model(0.2, False, 0, 1.0, 0.0, False)
    mist = mk_model(0.2, False, 0, 1.0, 0.0, False)
    v = compute(*build(bge, gte, spec, mist), {"bge": True, "specter": True, "gte": True})["verdict"]
    checks.append(("all-fail live -> KILL", v == "KILL", v))
    # bge strictP1, R=0.6, modern ok, but P4/P5 fail -> PIVOT
    bge = mk_model(0.6, True, 1, 1.0, 0.1, True)
    v = compute(*build(bge, mk_model(0.2), mk_model(0.2), mk_model(0.2)),
                {"bge": True, "specter": True, "gte": True})["verdict"]
    checks.append(("bge tie 0.6 w/anchor, no P4 -> PIVOT", v == "PIVOT", v))
    # bge strictP1 R=0.8, P4, P5 -> ADVANCE
    bge = mk_model(0.8, True, 1, 1.0, 0.1, True)
    gte = mk_model(0.6, True, 1, 1.0, 0.0, True)
    v = compute(*build(bge, gte, mk_model(0.2), mk_model(0.2)),
                {"bge": True, "specter": True, "gte": True})["verdict"]
    checks.append(("bge 0.8 +P4 +P5 -> ADVANCE", v == "ADVANCE", v))
    # bge not live (toy fail) -> INVALID
    bge = mk_model(0.8, True, 1, 1.0, 0.1, True)
    v = compute(*build(bge, mk_model(0.2), mk_model(0.2), mk_model(0.2)),
                {"bge": False, "specter": True, "gte": True})["verdict"]
    checks.append(("bge toy-fail -> INVALID", v == "INVALID-headline-broken", v))
    # bge fails, specter (live) passes strictP1, modern ok -> WEAK-PIVOT
    bge = mk_model(0.4, False, 0, 1.0, 0.1, False)
    spec = mk_model(0.6, True, 1, 1.0, 0.0, True)
    v = compute(*build(bge, mk_model(0.2), spec, mk_model(0.2)),
                {"bge": True, "specter": True, "gte": True})["verdict"]
    checks.append(("bge fail, specter passes -> WEAK-PIVOT", v == "WEAK-PIVOT", v))
    # bge strictP1 but modern regressed -> WEAK-no-bank
    bge = mk_model(0.6, True, 1, 0.5, 0.1, True)
    v = compute(*build(bge, mk_model(0.2), mk_model(0.2), mk_model(0.2)),
                {"bge": True, "specter": True, "gte": True})["verdict"]
    checks.append(("bge anchor but modern<floor -> WEAK-no-bank", v == "WEAK-no-bank", v))

    ok = True
    print("\nhand-pinned checks:")
    for name, passed, got in checks:
        print(f"  [{'PASS' if passed else 'FAIL'}] {name}  (got {got})")
        ok = ok and passed
    print("\n==> GATE SELFTEST", "PASS" if ok else "FAIL")
    return ok


if __name__ == "__main__":
    sys.exit(0 if run() else 1)
