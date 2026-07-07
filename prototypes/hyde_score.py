#!/usr/bin/env python3
"""EXP-RS-19 (Phase 38) — HyDE-Bridge scorer + eval.

Method (LOCKED, CONVENTIONS.md C-31..C-36): each QUERY paper has K=5 blind-generated cross-field
hypothetical abstracts (data/hyde_hypotheticals_{feynman,modern}.json). Candidates keep their
verbatim C-17 whole-abstract TF-IDF (sme_lite.tfidf_vectors). Query vs candidate:
  vec(h_k) = L2norm(tf * idf_corpus over tokens of HYP_TEXT present in the corpus IDF; OOV dropped),
             HYP_TEXT = target_field + ' ' + generic_object + ' ' + abstract
  hyde_sim(q,c)  = max_{k<=K} cosine(vec(h_k), tfidf(c))         (POOL=max)
  object_sim(q,c)= cosine(tfidf(q), tfidf(c))                    (= the C-17 lexical null)
  score(q,c)     = hyde_sim(q,c) - LAMBDA * object_sim(q,c)      (LAMBDA=0.0 headline)
Retrieval metric = C-19 (sme_lite.eval_direction), forward (side_a->side_b) primary.
Tokenizer/IDF are sme_lite's, identical to C-17, for apples-to-apples with the lexical null.

Cheap gate (C-34), Feynman forward-only: GATE-A = headline fwd recall@10 >= 3/5 AND recovers >=1
pair the C-17 null misses (pair01/pair04/pair06); GATE-B = pair04 recovered into top-10.

Usage:
  python hyde_score.py --corpus data/mvp_corpus.json \
      --hyp data/hyde_hypotheticals_feynman.json --pairs data/feynman_10pair_papers.json \
      --out data/hyde_results_feynman.json --gate feynman
"""
import argparse
import json
import math
import os
from collections import Counter

from sme_lite import tfidf_vectors, cosine, eval_direction, _WORD, _STOP
from methmesh_score import load_pairs  # handles Feynman + modern pair schemas

HERE = os.path.dirname(os.path.abspath(__file__))
DATA = os.path.join(HERE, "data")

NULL_MISSED_FEYNMAN = {"pair01-ising-opinion", "pair04-percolation-epidemics",
                       "pair06-turing-spatial-economy"}  # C-17 null misses these (verified)


def tokenize(text):
    return [t for t in _WORD.findall((text or "").lower()) if t not in _STOP and len(t) > 2]


def corpus_idf(corpus):
    docs = {p["arxiv_id"]: Counter(tokenize(p["abstract"])) for p in corpus["papers"]}
    df = Counter()
    for c in docs.values():
        df.update(c.keys())
    n = len(docs)
    return {w: math.log((n + 1) / (df[w] + 1)) + 1 for w in df}


def hyp_vec(hyp, idf):
    text = f"{hyp.get('target_field','')} {hyp.get('generic_object','')} {hyp.get('abstract','')}"
    c = Counter(tokenize(text))
    v = {w: c[w] * idf[w] for w in c if w in idf}  # OOV dropped (zero in every candidate)
    nrm = math.sqrt(sum(x * x for x in v.values())) or 1.0
    return {w: x / nrm for w, x in v.items()}


def load_hyp(path):
    j = json.load(open(path))
    rows = j["queries"] if "queries" in j else j
    out = {}
    for r in (rows.values() if isinstance(rows, dict) else rows):
        out[r["arxiv_id"]] = r
    return out


def build_scorer(corpus, hyp, idf, lam=0.0, k=5, pool="max"):
    lex = tfidf_vectors(corpus)
    hvecs = {}
    for aid, r in hyp.items():
        hs = r.get("hypotheticals", [])[:k]
        hvecs[aid] = [hyp_vec(h, idf) for h in hs]

    def score(q, c):
        sims = [cosine(v, lex[c]) for v in hvecs.get(q, [])]
        if not sims:
            hyde = 0.0
        else:
            hyde = max(sims) if pool == "max" else sum(sims) / len(sims)
        obj = cosine(lex[q], lex[c])
        return hyde - lam * obj
    return score


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--corpus", default=os.path.join(DATA, "mvp_corpus.json"))
    ap.add_argument("--hyp", default=os.path.join(DATA, "hyde_hypotheticals_feynman.json"))
    ap.add_argument("--pairs", default=os.path.join(DATA, "feynman_10pair_papers.json"))
    ap.add_argument("--out", default=os.path.join(DATA, "hyde_results_feynman.json"))
    ap.add_argument("--gate", choices=["feynman", "none"], default="none")
    ap.add_argument("--forward-only", action="store_true")
    args = ap.parse_args()

    corpus = json.load(open(args.corpus))
    all_ids = [p["arxiv_id"] for p in corpus["papers"]]
    present = set(all_ids)
    hyp = load_hyp(args.hyp)
    idf = corpus_idf(corpus)
    pairs = load_pairs(args.pairs, present)

    # arms: headline (C-36 pin) + descriptive ablations (NOT promotable)
    ARMS = [
        ("headline_lam0_k5_max", dict(lam=0.0, k=5, pool="max")),
        ("abl_lam0.5_k5_max", dict(lam=0.5, k=5, pool="max")),
        ("abl_lam0_k5_mean", dict(lam=0.0, k=5, pool="mean")),
        ("abl_lam0_k3_max", dict(lam=0.0, k=3, pool="max")),
        ("abl_lam0_k1_max", dict(lam=0.0, k=1, pool="max")),
    ]
    results = {"experiment": "EXP-RS-19", "corpus": os.path.basename(args.corpus),
               "hyp": os.path.basename(args.hyp), "n_corpus": len(all_ids),
               "n_eval_pairs": len(pairs), "pair_ids": [p["id"] for p in pairs],
               "n_queries_with_hyp": sum(1 for p in pairs
                                         for s in (p["side_a"], p["side_b"]) if s in hyp),
               "arms": {}}
    for name, kw in ARMS:
        sc = build_scorer(corpus, hyp, idf, **kw)
        fwd = eval_direction(pairs, all_ids, sc, "side_a", "side_b")
        arm = {"forward": fwd}
        if not args.forward_only:
            rev = eval_direction(pairs, all_ids, sc, "side_b", "side_a")
            arm["reverse"] = rev
            arm["both_dir_avg"] = {kk: (fwd[kk] + rev[kk]) / 2
                                   for kk in ("recall@1", "recall@5", "recall@10", "mrr")}
        results["arms"][name] = arm

    head = results["arms"]["headline_lam0_k5_max"]["forward"]
    results["BENCH_P10"] = head["recall@10"]

    # cheap gate (C-34) — Feynman forward only
    if args.gate == "feynman":
        ranks = head["ranks"]
        recovered = {pid for pid, r in ranks.items() if r is not None and r <= 10}
        null_missed_recovered = sorted(recovered & NULL_MISSED_FEYNMAN)
        gate_a = (head["recall@10"] >= 0.6) and (len(null_missed_recovered) >= 1)
        gate_b = ranks.get("pair04-percolation-epidemics") is not None and \
            ranks["pair04-percolation-epidemics"] <= 10
        results["gate"] = {
            "GATE_A_recall10_ge_3of5_and_null_missed_recovered": gate_a,
            "GATE_B_pair04_recovered": gate_b,
            "recall@10": head["recall@10"],
            "null_missed_pairs_recovered": null_missed_recovered,
            "pair04_rank": ranks.get("pair04-percolation-epidemics"),
            "PASS": gate_a and gate_b,
            "forward_ranks": ranks,
        }

    json.dump(results, open(args.out, "w"), indent=1, ensure_ascii=False)

    # console summary
    print(f"corpus={results['corpus']} N={len(all_ids)} pairs={len(pairs)} "
          f"queries_with_hyp={results['n_queries_with_hyp']}")
    hdr = f"{'arm':22} {'dir':8} {'R@1':>5} {'R@5':>5} {'R@10':>5} {'MRR':>6}"
    print(hdr); print("-" * len(hdr))
    for name, _ in ARMS:
        a = results["arms"][name]
        for d in (("forward",) if args.forward_only else ("forward", "reverse")):
            m = a[d]
            print(f"{name:22} {d:8} {m['recall@1']:5.2f} {m['recall@5']:5.2f} "
                  f"{m['recall@10']:5.2f} {m['mrr']:6.3f}")
    print(f"\nheadline forward recall@10 (BENCH_P10) = {results['BENCH_P10']:.3f}")
    print("headline forward ranks:", head["ranks"])
    if args.gate == "feynman":
        g = results["gate"]
        print(f"\nGATE-A (recall@10>=0.6 AND recover a null-missed pair): {g['GATE_A_recall10_ge_3of5_and_null_missed_recovered']} "
              f"(null-missed recovered: {g['null_missed_pairs_recovered']})")
        print(f"GATE-B (pair04 recovered, rank={g['pair04_rank']}): {g['GATE_B_pair04_recovered']}")
        print(f"==> CHEAP GATE {'PASS' if g['PASS'] else 'FAIL (KILL)'}")
    print(f"wrote {args.out}")


if __name__ == "__main__":
    main()
