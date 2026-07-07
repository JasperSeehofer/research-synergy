#!/usr/bin/env python3
"""EXP-RS-16 (Phase 35) — SME-lite structure-mapping matcher.

Systematicity (LOCKED, CONVENTIONS.md C-18): for query schema G1 and candidate G2, build the
association (modular-product) graph over entity-pairs that are role-compatible (roles-ON) or
all-compatible (roles-OFF); connect two pairs when the directed, typed relation structure between
them is consistent in both graphs. Maximal cliques (Bron-Kerbosch, networkx) = common
role/relation-consistent mappings. **Score = number of directed typed relations preserved AND
mutually connected under the best mapping** (isolated entity matches score 0). Ties in retrieval
-> candidate arxiv_id lexicographic (C-19).

Arms (C-17): roles-ON, roles-OFF (roles stripped, relations kept), lexical-null (abstract
bag-of-words TF-IDF cosine).

Eval (C-19): conditional retrieval; primary = side_a query -> rank all others -> is side_b in
top-k? recall@{1,5,10} + MRR over the evaluable pairs; reverse + both-direction average reported.
BENCH_P10 = roles-ON primary-direction recall@10.

Usage: python sme_lite.py   (defaults to the Feynman MVP corpus)
       python sme_lite.py --corpus <corpus.json> --schemas <schemas.json> --pairs <pairs.json> --out <out.json>
"""
import argparse
import json
import math
import os
import re
from collections import Counter, defaultdict

import networkx as nx

HERE = os.path.dirname(os.path.abspath(__file__))
DATA = os.path.join(HERE, "data")


# ----------------------------- schema graph helpers -----------------------------
def relset(schema):
    """Directed typed relations as a set of (src,dst,type)."""
    return {(r["src"], r["dst"], r["type"]) for r in schema["relations"]
            if r["src"] != r["dst"]}


def directed_typed_edges(schema):
    """map (u,v) -> frozenset of types on u->v."""
    d = defaultdict(set)
    for r in schema["relations"]:
        if r["src"] != r["dst"]:
            d[(r["src"], r["dst"])].add(r["type"])
    return {k: frozenset(v) for k, v in d.items()}


def systematicity(g1, g2, use_roles=True):
    """Max connected preserved-relation count over role/relation-consistent mappings."""
    e1 = [e["id"] for e in g1["entities"]]
    e2 = [e["id"] for e in g2["entities"]]
    role1 = {e["id"]: e["role"] for e in g1["entities"]}
    role2 = {e["id"]: e["role"] for e in g2["entities"]}
    de1 = directed_typed_edges(g1)
    de2 = directed_typed_edges(g2)

    # association-graph nodes = compatible entity pairs
    nodes = []
    for u in e1:
        for v in e2:
            if (not use_roles) or role1[u] == role2[v]:
                nodes.append((u, v))
    if not nodes:
        return 0, []

    def consistent(p, q):
        (u1, v1), (u2, v2) = p, q
        if u1 == u2 or v1 == v2:
            return False  # injective
        # directed typed edge-sets must match in both directions (modular product)
        if de1.get((u1, u2), frozenset()) != de2.get((v1, v2), frozenset()):
            return False
        if de1.get((u2, u1), frozenset()) != de2.get((v2, v1), frozenset()):
            return False
        return True

    A = nx.Graph()
    A.add_nodes_from(nodes)
    for i in range(len(nodes)):
        for j in range(i + 1, len(nodes)):
            if consistent(nodes[i], nodes[j]):
                A.add_edge(nodes[i], nodes[j])

    best_score, best_map = 0, []
    for clique in nx.find_cliques(A):
        mp = dict(clique)  # u -> v
        uset = set(mp)
        # preserved relations internal to the clique (guaranteed present+typed by construction)
        pres = [(a, b, t) for (a, b, t) in relset(g1)
                if a in uset and b in uset and (mp[a], mp[b], t) in relset(g2)]
        if not pres:
            continue
        # largest connected component of the preserved-relation subgraph
        H = nx.Graph()
        H.add_nodes_from(uset)
        for a, b, _ in pres:
            H.add_edge(a, b)
        comps = list(nx.connected_components(H))
        for comp in comps:
            sc = sum(1 for (a, b, _) in pres if a in comp and b in comp)
            if sc > best_score:
                best_score = sc
                best_map = [(a, mp[a]) for a in comp if a in mp]
    return best_score, best_map


def alignment_table(g1, g2, use_roles=True):
    """Return the best mapping as a human-auditable alignment table."""
    score, mp = systematicity(g1, g2, use_roles=use_roles)
    gloss1 = {e["id"]: (e["role"], e.get("gloss", "")) for e in g1["entities"]}
    gloss2 = {e["id"]: (e["role"], e.get("gloss", "")) for e in g2["entities"]}
    rows = []
    for a, b in mp:
        rows.append({
            "a_id": a, "a_role": gloss1[a][0], "a_gloss": gloss1[a][1],
            "b_id": b, "b_role": gloss2[b][0], "b_gloss": gloss2[b][1],
        })
    mset = dict(mp)
    preserved = sorted({t for (a, b, t) in relset(g1)
                        if a in mset and b in mset and (mset[a], mset[b], t) in relset(g2)})
    return {"systematicity": score, "n_aligned_entities": len(mp),
            "preserved_relation_types": preserved, "rows": rows}


# ----------------------------- lexical null -----------------------------
_WORD = re.compile(r"[a-z][a-z0-9\-]+")
_STOP = set("the a an of to in and or for on with by is are we this that as at be from "
            "which can using use based our their its into between within over under".split())


def tfidf_vectors(corpus):
    docs = {}
    for p in corpus["papers"]:
        toks = [t for t in _WORD.findall(p["abstract"].lower())
                if t not in _STOP and len(t) > 2]
        docs[p["arxiv_id"]] = Counter(toks)
    df = Counter()
    for c in docs.values():
        df.update(c.keys())
    N = len(docs)
    idf = {w: math.log((N + 1) / (df[w] + 1)) + 1 for w in df}
    vecs = {}
    for aid, c in docs.items():
        v = {w: (f) * idf[w] for w, f in c.items()}
        nrm = math.sqrt(sum(x * x for x in v.values())) or 1.0
        vecs[aid] = {w: x / nrm for w, x in v.items()}
    return vecs


def cosine(a, b):
    if len(a) > len(b):
        a, b = b, a
    return sum(x * b.get(w, 0.0) for w, x in a.items())


# ----------------------------- retrieval + metrics -----------------------------
def rank_candidates(query_id, corpus_ids, score_fn):
    """Return ranked list of (cand_id, score) desc, ties -> cand_id lexicographic."""
    scored = [(cid, score_fn(query_id, cid)) for cid in corpus_ids if cid != query_id]
    scored.sort(key=lambda x: (-x[1], x[0]))
    return scored


def eval_direction(pairs, all_ids, score_fn, query_side, target_side):
    ranks = {}
    for p in pairs:
        q = p[query_side]
        t = p[target_side]
        ranked = rank_candidates(q, all_ids, score_fn)
        pos = next((i + 1 for i, (cid, _) in enumerate(ranked) if cid == t), None)
        ranks[p["id"]] = pos
    def recall_at(k):
        hits = sum(1 for r in ranks.values() if r is not None and r <= k)
        return hits / len(ranks)
    mrr = sum((1.0 / r) for r in ranks.values() if r) / len(ranks)
    return {"ranks": ranks,
            "recall@1": recall_at(1), "recall@5": recall_at(5), "recall@10": recall_at(10),
            "mrr": mrr}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--corpus", default=os.path.join(DATA, "mvp_corpus.json"))
    ap.add_argument("--schemas", default=os.path.join(DATA, "mvp_schemas.json"))
    ap.add_argument("--pairs", default=os.path.join(DATA, "feynman_10pair_papers.json"))
    ap.add_argument("--out", default=os.path.join(DATA, "sme_results.json"))
    args = ap.parse_args()

    corpus = json.load(open(args.corpus))
    schemas = json.load(open(args.schemas))["schemas"]
    feyn = json.load(open(args.pairs))
    all_ids = [p["arxiv_id"] for p in corpus["papers"]]

    ev = set(feyn["evaluable_pairs"])
    pairs = []
    for p in feyn["pairs"]:
        pid = p["id"].split("-")[0]
        if pid in ev:
            a, b = p["side_a"]["arxiv_id"], p["side_b"]["arxiv_id"]
            if a in schemas and b in schemas:
                pairs.append({"id": p["id"], "side_a": a, "side_b": b})

    # score functions per arm
    def sme_score(use_roles):
        cache = {}
        def f(q, c):
            key = (q, c, use_roles)
            if key not in cache:
                cache[key] = systematicity(schemas[q], schemas[c], use_roles=use_roles)[0]
            return cache[key]
        return f

    lex = tfidf_vectors(corpus)
    def lex_score(q, c):
        return cosine(lex[q], lex[c])

    arms = {
        "roles_on": sme_score(True),
        "roles_off": sme_score(False),
        "lexical_null": lex_score,
    }

    results = {"experiment": "EXP-RS-16", "corpus": os.path.basename(args.corpus),
               "n_corpus": len(all_ids), "n_eval_pairs": len(pairs),
               "pair_ids": [p["id"] for p in pairs], "arms": {}}
    for name, fn in arms.items():
        fwd = eval_direction(pairs, all_ids, fn, "side_a", "side_b")
        rev = eval_direction(pairs, all_ids, fn, "side_b", "side_a")
        avg = {k: (fwd[k] + rev[k]) / 2 for k in ("recall@1", "recall@5", "recall@10", "mrr")}
        results["arms"][name] = {"forward": fwd, "reverse": rev, "both_dir_avg": avg}

    results["BENCH_P10"] = results["arms"]["roles_on"]["forward"]["recall@10"]

    # alignment tables (roles-ON) for every evaluable pair
    results["alignment_tables"] = {}
    for p in pairs:
        results["alignment_tables"][p["id"]] = alignment_table(
            schemas[p["side_a"]], schemas[p["side_b"]], use_roles=True)

    json.dump(results, open(args.out, "w"), indent=1, ensure_ascii=False)

    # console summary
    print(f"corpus={results['corpus']} N={len(all_ids)} eval_pairs={len(pairs)}")
    hdr = f"{'arm':13} {'dir':8} {'R@1':>5} {'R@5':>5} {'R@10':>5} {'MRR':>6}"
    print(hdr); print("-" * len(hdr))
    for name in arms:
        for d in ("forward", "reverse"):
            a = results["arms"][name][d]
            print(f"{name:13} {d:8} {a['recall@1']:5.2f} {a['recall@5']:5.2f} "
                  f"{a['recall@10']:5.2f} {a['mrr']:6.3f}")
    print(f"\nBENCH_P10 (roles_on fwd recall@10) = {results['BENCH_P10']:.3f}")
    print("\nper-pair forward ranks (roles_on):",
          results["arms"]["roles_on"]["forward"]["ranks"])
    print("per-pair forward ranks (lexical): ",
          results["arms"]["lexical_null"]["forward"]["ranks"])
    print(f"\nwrote {args.out}")


if __name__ == "__main__":
    main()
