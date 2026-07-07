#!/usr/bin/env python3
"""EXP-RS-17 — collect per-paper blind tag outputs into a single tags file + compute the C-26 gate.

Usage:
  python collect_tags.py --sub feynman --pairs data/feynman_10pair_papers.json \
      --out data/methmesh_tags_feynman.json --gate 3
  python collect_tags.py --sub modern --pairs data/modern_lbd_pairs.json \
      --out data/methmesh_tags_modern.json --gate 4
"""
import argparse
import glob
import json
import os
import re

HERE = os.path.dirname(os.path.abspath(__file__))
DATA = os.path.join(HERE, "data")


def norm(a):
    return re.sub(r"v\d+$", "", a.replace("https://arxiv.org/abs/", ""))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--sub", required=True)
    ap.add_argument("--pairs", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--gate", type=int, default=3)
    args = ap.parse_args()

    tags = {}
    for f in sorted(glob.glob(os.path.join(DATA, "tag_outputs", args.sub, "*.json"))):
        d = json.load(open(f))
        tags[d["arxiv_id"]] = {"tags": d["tags"]}

    j = json.load(open(args.pairs))
    ev = set(j["evaluable_pairs"])
    feyn = "feynman" in args.sub
    gate_rows = {}
    for p in j["pairs"]:
        key = p["id"].split("-")[0] if feyn else p["id"]
        if key not in ev:
            continue
        a, b = norm(p["side_a"]["arxiv_id"]), norm(p["side_b"]["arxiv_id"])
        sa = {t["archetype_id"] for t in tags.get(a, {}).get("tags", [])}
        sb = {t["archetype_id"] for t in tags.get(b, {}).get("tags", [])}
        gate_rows[p["id"]] = sorted(sa & sb)
    hits = sum(1 for v in gate_rows.values() if v)

    out = {
        "experiment": "EXP-RS-17", "convention": "C-23",
        "sub": args.sub, "n_tagged": len(tags),
        "tagging_recall_hits": hits, "n_pairs": len(gate_rows), "gate": args.gate,
        "tagging_recall_pass": hits >= args.gate,
        "shared_archetypes_per_pair": gate_rows,
        "tags": tags,
    }
    json.dump(out, open(args.out, "w"), indent=1, ensure_ascii=False)
    print(f"{args.sub}: tagged={len(tags)} tagging_recall={hits}/{len(gate_rows)} "
          f"(gate>={args.gate}) -> {'PASS' if out['tagging_recall_pass'] else 'FAIL'}")
    print(f"  wrote {args.out}")


if __name__ == "__main__":
    main()
