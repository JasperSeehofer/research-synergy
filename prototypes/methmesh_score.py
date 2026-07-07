#!/usr/bin/env python3
"""EXP-RS-17 (Phase 36) — MethMeSH mechanism-ontology scorer + eval.

Method (LOCKED, CONVENTIONS.md C-25): each paper carries 1-5 archetypes from the FROZEN
field-agnostic vocabulary (C-22, data/mechanism_vocab.json), assigned by a BLIND per-paper tagger
(C-23). signature(p) = IDF-weighted vector over archetypes (IDF over the corpus being scored).
score(i,j) = cos(sig_i, sig_j) - lambda * cos(tfidf_i, tfidf_j), lambda=1.0; tfidf = the C-17
abstract bag-of-words null (imported from sme_lite for apples-to-apples with EXP-RS-16).

Eval (C-19, imported from sme_lite): conditional retrieval; primary = side_a query -> rank all
others -> is side_b in top-k? recall@{1,5,10}+MRR over evaluable pairs; reverse + both-dir avg.
Ablations (C-25): IDF-weight ON vs OFF (uniform archetype weight); lambda=1.0 vs 0.0.
Cheap KILL gate (C-26): tagging_recall = fraction of evaluable pairs whose two sides share >=1
archetype (Feynman >=3/5, modern >=4/6, else KILL by construction).

Usage:
  python methmesh_score.py \
      --corpus data/mvp_corpus.json --tags data/methmesh_tags_feynman.json \
      --pairs data/feynman_10pair_papers.json --out data/methmesh_results_feynman.json
  python methmesh_score.py \
      --corpus data/modern_mvp_corpus.json --tags data/methmesh_tags_modern.json \
      --pairs data/modern_lbd_pairs.json --out data/methmesh_results_modern.json
"""
import argparse
import json
import math
import os
from collections import Counter

from sme_lite import tfidf_vectors, cosine, eval_direction  # C-17 null + C-19 metric

HERE = os.path.dirname(os.path.abspath(__file__))
DATA = os.path.join(HERE, "data")


# ----------------------------- pairs loader (both formats) -----------------------------
def load_pairs(pairs_path, present_ids):
    """Evaluable pairs with both sides present in the corpus. Handles Feynman + modern schemas."""
    j = json.load(open(pairs_path))
    ev = set(j["evaluable_pairs"])
    out = []
    for p in j["pairs"]:
        # Feynman ids look like 'pair01-ising-opinion'; modern ids look like 'm01'
        key = p["id"].split("-")[0] if p["id"].startswith("pair") else p["id"]
        if key not in ev:
            continue
        a = p["side_a"]["arxiv_id"].replace("https://arxiv.org/abs/", "")
        b = p["side_b"]["arxiv_id"].replace("https://arxiv.org/abs/", "")
        import re
        a = re.sub(r"v\d+$", "", a)
        b = re.sub(r"v\d+$", "", b)
        if a in present_ids and b in present_ids:
            out.append({"id": p["id"], "side_a": a, "side_b": b})
    return out


# ----------------------------- archetype signatures -----------------------------
def load_tags(tags_path):
    """-> {arxiv_id: [{archetype_id, evidence_snippet}]}."""
    j = json.load(open(tags_path))
    tags = j["tags"] if "tags" in j else j
    # normalise to {aid: [ {archetype_id, evidence_snippet} ]}
    norm = {}
    for aid, entry in tags.items():
        lst = entry.get("tags", entry) if isinstance(entry, dict) else entry
        norm[aid] = [{"archetype_id": t["archetype_id"],
                      "evidence_snippet": t.get("evidence_snippet", "")} for t in lst]
    return norm


def archetype_sets(tags):
    return {aid: {t["archetype_id"] for t in lst} for aid, lst in tags.items()}


def archetype_idf(sets, all_ids):
    df = Counter()
    for aid in all_ids:
        for a in sets.get(aid, set()):
            df[a] += 1
    N = len(all_ids)
    return {a: math.log((N + 1) / (df[a] + 1)) + 1 for a in df}


def signature_vectors(sets, all_ids, idf, use_idf=True):
    vecs = {}
    for aid in all_ids:
        v = {a: (idf[a] if use_idf else 1.0) for a in sets.get(aid, set())}
        nrm = math.sqrt(sum(x * x for x in v.values())) or 1.0
        vecs[aid] = {a: x / nrm for a, x in v.items()}
    return vecs


# ----------------------------- main -----------------------------
def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--corpus", default=os.path.join(DATA, "mvp_corpus.json"))
    ap.add_argument("--tags", default=os.path.join(DATA, "methmesh_tags_feynman.json"))
    ap.add_argument("--pairs", default=os.path.join(DATA, "feynman_10pair_papers.json"))
    ap.add_argument("--vocab", default=os.path.join(DATA, "mechanism_vocab.json"))
    ap.add_argument("--out", default=os.path.join(DATA, "methmesh_results_feynman.json"))
    ap.add_argument("--lam", type=float, default=1.0)
    ap.add_argument("--gate", type=int, default=3, help="tagging-recall gate threshold (pairs)")
    args = ap.parse_args()

    corpus = json.load(open(args.corpus))
    vocab_ids = {a["id"] for a in json.load(open(args.vocab))["archetypes"]}
    all_ids = [p["arxiv_id"] for p in corpus["papers"]]
    present = set(all_ids)
    tags = load_tags(args.tags)
    sets = archetype_sets(tags)

    # validate tags reference the frozen vocab only
    unknown = {a for s in sets.values() for a in s if a not in vocab_ids}
    if unknown:
        print(f"WARNING: {len(unknown)} tag archetype ids not in frozen vocab: "
              f"{sorted(unknown)[:8]}")

    pairs = load_pairs(args.pairs, present)
    idf = archetype_idf(sets, all_ids)
    lex = tfidf_vectors(corpus)

    # ---- tagging-recall gate (C-26) ----
    gate_rows = {}
    for p in pairs:
        shared = sets.get(p["side_a"], set()) & sets.get(p["side_b"], set())
        gate_rows[p["id"]] = sorted(shared)
    tag_recall_hits = sum(1 for v in gate_rows.values() if v)
    tagging_recall = tag_recall_hits / len(pairs) if pairs else 0.0

    # ---- score functions / arms ----
    def make_score(use_idf, lam):
        sig = signature_vectors(sets, all_ids, idf, use_idf=use_idf)
        def f(q, c):
            return cosine(sig[q], sig[c]) - lam * cosine(lex[q], lex[c])
        return f

    def lex_only(q, c):
        return cosine(lex[q], lex[c])

    arms = {
        "methmesh_idf_lam": make_score(True, args.lam),      # primary (IDF-on, minus-lexical)
        "methmesh_noidf": make_score(False, args.lam),       # ablation: uniform archetype weight
        "methmesh_nolex": make_score(True, 0.0),             # ablation: signature only (lambda=0)
        "lexical_null": lex_only,                            # reference floor (C-17)
    }

    results = {
        "experiment": "EXP-RS-17", "corpus": os.path.basename(args.corpus),
        "tags": os.path.basename(args.tags), "lambda": args.lam,
        "n_corpus": len(all_ids), "n_eval_pairs": len(pairs),
        "pair_ids": [p["id"] for p in pairs],
        "n_archetypes_used": len({a for s in sets.values() for a in s}),
        "tagging_recall": tagging_recall, "tagging_recall_hits": tag_recall_hits,
        "tagging_recall_gate": args.gate,
        "tagging_recall_pass": tag_recall_hits >= args.gate,
        "shared_archetypes_per_pair": gate_rows,
        "arms": {},
    }
    for name, fn in arms.items():
        fwd = eval_direction(pairs, all_ids, fn, "side_a", "side_b")
        rev = eval_direction(pairs, all_ids, fn, "side_b", "side_a")
        avg = {k: (fwd[k] + rev[k]) / 2 for k in ("recall@1", "recall@5", "recall@10", "mrr")}
        results["arms"][name] = {"forward": fwd, "reverse": rev, "both_dir_avg": avg}

    results["BENCH_P10"] = results["arms"]["methmesh_idf_lam"]["forward"]["recall@10"]

    # ---- auditable artifact: shared archetype + evidence for each pair ----
    ev_by_id = {aid: {t["archetype_id"]: t["evidence_snippet"] for t in lst}
                for aid, lst in tags.items()}
    artifact = {}
    for p in pairs:
        shared = gate_rows[p["id"]]
        artifact[p["id"]] = [{
            "archetype_id": a,
            "evidence_side_a": ev_by_id.get(p["side_a"], {}).get(a, ""),
            "evidence_side_b": ev_by_id.get(p["side_b"], {}).get(a, ""),
        } for a in shared]
    results["shared_archetype_artifact"] = artifact

    json.dump(results, open(args.out, "w"), indent=1, ensure_ascii=False)

    # ---- console summary ----
    print(f"corpus={results['corpus']} N={len(all_ids)} eval_pairs={len(pairs)} "
          f"archetypes_used={results['n_archetypes_used']}")
    print(f"TAGGING-RECALL GATE (C-26): {tag_recall_hits}/{len(pairs)} pairs share an archetype "
          f"(gate>= {args.gate}) -> {'PASS' if results['tagging_recall_pass'] else 'FAIL (KILL)'}")
    for pid, sh in gate_rows.items():
        print(f"   {pid}: {sh if sh else '(none shared)'}")
    hdr = f"\n{'arm':18} {'dir':8} {'R@1':>5} {'R@5':>5} {'R@10':>5} {'MRR':>6}"
    print(hdr); print("-" * len(hdr))
    for name in arms:
        for d in ("forward", "reverse"):
            a = results["arms"][name][d]
            print(f"{name:18} {d:8} {a['recall@1']:5.2f} {a['recall@5']:5.2f} "
                  f"{a['recall@10']:5.2f} {a['mrr']:6.3f}")
    print(f"\nBENCH_P10 (methmesh_idf_lam fwd recall@10) = {results['BENCH_P10']:.3f}")
    print("per-pair fwd ranks (primary):", results["arms"]["methmesh_idf_lam"]["forward"]["ranks"])
    print(f"wrote {args.out}")


if __name__ == "__main__":
    main()
