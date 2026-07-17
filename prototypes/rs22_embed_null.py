#!/usr/bin/env python3
"""EXP-RS-22 — RS-21 dense-embedding null over the mined-corpus K=50 pools.

The gate's memory-free floor is maxnull = the BETTER (lower percentile rank) of the lexical-TF-IDF
null (rs22_probe.lexical_pctile) and the RS-21 dense-embedding null. This computes the latter on the
SAME deterministic K=50 pools rs22_probe builds, using the RS-21 headline encoder (bge-large-en-v1.5,
symmetric, TEXT = title + '. ' + abstract), reused verbatim from embed_score.encode_st. Optionally
also gte/specter for a sensitivity check.

For each pair: rank side_b within its 50-candidate pool by cos(emb(side_a), emb(candidate)); write
{pair_id: {rank_1based, pctile}}. rs22_probe.score folds pctile into rank_bestnull_pctile via min().

Usage:
  python rs22_embed_null.py --model bge --out data/rs22_embed_null.json
"""
import argparse
import json
import os

import numpy as np

import rs22_probe as R
from embed_score import encode_st, encode_specter, MODEL_SPECS, l2norm_rows

HERE = os.path.dirname(os.path.abspath(__file__))
DATA = os.path.join(HERE, "data")


def encode_all(papers_list, model_key):
    hf_id, kind = MODEL_SPECS[model_key]
    if kind == "st":
        vecs, rev, seq, _ = encode_st(hf_id, papers_list, model_key)
    elif kind == "specter":
        vecs, rev, seq, _ = encode_specter(hf_id, papers_list, model_key)
    else:
        raise SystemExit(f"model {model_key} kind {kind} not supported for the local null")
    return l2norm_rows(vecs), rev


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--model", default="bge", choices=["bge", "gte", "specter"])
    ap.add_argument("--out", default=os.path.join(DATA, "rs22_embed_null.json"))
    args = ap.parse_args()

    pairs = R.load_corpus()
    papers = R.build_papers(pairs)                     # dict id -> {arxiv_id,title,abstract,category}
    ids = list(papers.keys())
    papers_list = [papers[i] for i in ids]
    print(f"encoding {len(ids)} mined papers with {args.model} (CPU, symmetric)...")
    V, rev = encode_all(papers_list, args.model)
    idx = {i: k for k, i in enumerate(ids)}

    out = {}
    for p in pairs:
        presented, K = R.retrieval_pool(p, papers)
        a_id = R.normalize_id(p["side_a"]["arxiv_id"])
        b_id = R.normalize_id(p["side_b"]["arxiv_id"])
        qv = V[idx[a_id]]
        sims = {cid: float(np.dot(qv, V[idx[cid]])) for cid in presented}
        ranked = sorted(presented, key=lambda c: (-sims[c], c))
        rank_1 = ranked.index(b_id) + 1
        out[p["pair_id"]] = {"rank_1based": rank_1, "pctile": 100.0 * (rank_1 - 1) / (K - 1)}

    manifest = {"experiment": "EXP-RS-22", "model": args.model, "hf_id": MODEL_SPECS[args.model][0],
                "revision": rev, "n_papers": len(ids), "n_pairs": len(pairs),
                "text": "title + '. ' + abstract (symmetric)", "K_pool": R.K_POOL,
                "note": "RS-21 dense-embedding null over the frozen K=50 pools; folded into maxnull via min(pctile)"}
    json.dump({"manifest": manifest, "ranks": out}, open(args.out, "w"), indent=1, ensure_ascii=False)
    pct = [v["pctile"] for v in out.values()]
    print(f"wrote {args.out}: {len(out)} pairs, median pctile={np.median(pct):.1f} "
          f"(rev {rev[:12]})")


if __name__ == "__main__":
    main()
