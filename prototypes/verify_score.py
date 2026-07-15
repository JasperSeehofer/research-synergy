#!/usr/bin/env python3
"""EXP-RS-20 (Phase 39) — Generate->Verify cascade: harness for the blind VERIFY stage.

Pipeline (LOCKED, CONVENTIONS.md C-38..C-40; reuses C-14/C-17/C-19/C-31..C-36):
  Stage 1 GENERATE = frozen HyDE (EXP-RS-19). Headline HyDE score(q,c) = hyde_sim(q,c) with
    LAMBDA=0, K=5, POOL=max, reused VERBATIM from hyde_score.build_scorer / hyp_vec / corpus_idf.
  Stage 2 VERIFY = a blind, benchmark-agnostic audit of each (query, candidate) transfer. Per
    candidate the verify stage sees ONLY the C-39 closure
      {query_title, query_abstract, hyp_target_field, hyp_generic_object, hyp_abstract,
       candidate_title, candidate_abstract}
    where the hypothetical shown is the WINNING one for that candidate = argmax_k
    cos(vec(h_k), tfidf(c)) (the transfer most responsible for the candidate's HyDE score).
    It returns {method_coherence:bool, object_difference:bool, rationale:str}. No paper IDs, no
    benchmark context, no sight of other candidates (C-39).
  Headline pruning (C-40): candidates with method_coherence=false are demoted to rank inf; ties
    broken by candidate arxiv_id lexicographic (inherits C-19). This is implemented as a composite
    score = hyde_score(q,c) if coherent else -inf, fed to sme_lite.eval_direction (whose sort key
    (-score, cand_id) sends -inf candidates to the bottom, lexicographically ordered among ties).
  Retrieval metric = C-19 (sme_lite.eval_direction), forward (side_a->side_b) primary.

Two subcommands:
  emit-inputs  Deterministically write the blind per-(q,c) verify INPUT records (C-39 closure) +
               a manifest, ready for dispatch to blind subagents (LLM backbone). The Feynman cheap
               gate (P1) = the 5 evaluable side_a queries x 35 candidates = 175 records (~180 calls).
  score        Collect verify OUTPUT verdicts (LLM files, or computed CAS ablation), apply C-40
               pruning, recompute C-19 recall, and evaluate the pre-registered P1 gate + P2 + P4.

Backbones (pre-registered, C-40 headline = llm):
  --backbone llm   headline. Verdicts come from data/verify_outputs/<sub>/<q>__<c>.json produced by
                   blind subagents driven by the frozen prototypes/verify_prompt.md (SHA-256, C-38).
  --backbone cas   pre-registered ABLATION (Open Risk #2 fallback): a computational keyword/rule
                   audit computed here, no LLM calls. NOT the headline; used for a cheap preview +
                   the LLM-vs-CAS backbone ablation.

Usage:
  # 1) emit the 175 blind cheap-gate inputs
  python verify_score.py emit-inputs --sub feynman
  # 2) (LLM backbone) dispatch each data/verify_inputs/feynman/*.json to a blind subagent that
  #    writes {method_coherence,object_difference,rationale} to data/verify_outputs/feynman/<name>
  # 3) score + gate
  python verify_score.py score --sub feynman --backbone llm
  python verify_score.py score --sub feynman --backbone cas   # cheap ablation preview
"""
import argparse
import glob
import json
import math
import os

from hyde_score import corpus_idf, hyp_vec, load_hyp, tokenize
from methmesh_score import load_pairs
from sme_lite import tfidf_vectors, cosine, eval_direction

HERE = os.path.dirname(os.path.abspath(__file__))
DATA = os.path.join(HERE, "data")
NEG_INF = float("-inf")

# C-17 null misses these Feynman pairs (verified EXP-RS-19); recovering >=1 is part of P1.
NULL_MISSED_FEYNMAN = {"pair01-ising-opinion", "pair04-percolation-epidemics",
                       "pair06-turing-spatial-economy"}
PAIR04 = "pair04-percolation-epidemics"

SUBS = {
    "feynman": dict(corpus="mvp_corpus.json", hyp="hyde_hypotheticals_feynman.json",
                    pairs="feynman_10pair_papers.json"),
    "modern": dict(corpus="modern_mvp_corpus.json", hyp="hyde_hypotheticals_modern.json",
                   pairs="modern_lbd_pairs.json"),
}


def safe(aid):
    """Filesystem-safe join key for an arxiv id (cond-mat/0007235 -> cond-mat_0007235)."""
    return aid.replace("/", "_")


# ----------------------------- HyDE stage (frozen, reused) -----------------------------
def build_hyde(corpus, hyp, idf, k=5):
    """Return (hyde_score, winning_idx, lex). hyde_score/winning_idx match hyde_score.build_scorer
    headline (lam=0, K=5, max): hyde_sim(q,c)=max_k cos(vec(h_k),tfidf(c)); winner = argmax_k."""
    lex = tfidf_vectors(corpus)
    hvecs = {aid: [hyp_vec(h, idf) for h in r.get("hypotheticals", [])[:k]]
             for aid, r in hyp.items()}

    def sims(q, c):
        return [cosine(v, lex[c]) for v in hvecs.get(q, [])]

    def hyde_score(q, c):
        s = sims(q, c)
        return max(s) if s else 0.0

    def winning_idx(q, c):
        s = sims(q, c)
        if not s:
            return 0
        best, bi = s[0], 0
        for i, x in enumerate(s):  # first-index tie-break (deterministic)
            if x > best:
                best, bi = x, i
        return bi

    return hyde_score, winning_idx, lex


def load_ctx(sub):
    cfg = SUBS[sub]
    corpus = json.load(open(os.path.join(DATA, cfg["corpus"])))
    all_ids = [p["arxiv_id"] for p in corpus["papers"]]
    by_id = {p["arxiv_id"]: p for p in corpus["papers"]}
    hyp = load_hyp(os.path.join(DATA, cfg["hyp"]))
    idf = corpus_idf(corpus)
    pairs = load_pairs(os.path.join(DATA, cfg["pairs"]), set(all_ids))
    hyde_score, winning_idx, _ = build_hyde(corpus, hyp, idf)
    return corpus, all_ids, by_id, hyp, pairs, hyde_score, winning_idx


# ----------------------------- emit-inputs -----------------------------
def emit_inputs(sub, forward_only=True):
    corpus, all_ids, by_id, hyp, pairs, hyde_score, winning_idx = load_ctx(sub)
    in_dir = os.path.join(DATA, "verify_inputs", sub)
    os.makedirs(in_dir, exist_ok=True)

    # cheap gate = forward only: query = each pair's side_a; target = side_b.
    queries = []
    for p in pairs:
        queries.append((p["side_a"], p["side_b"], p["id"]))
        if not forward_only:
            queries.append((p["side_b"], p["side_a"], p["id"] + "#rev"))

    records = []
    for q, target, pid in queries:
        qp = by_id[q]
        for c in all_ids:
            if c == q:
                continue
            win = winning_idx(q, c)
            h = hyp[q]["hypotheticals"][win]
            cp = by_id[c]
            # C-39 input closure: EXACTLY these keys, NO ids/benchmark context.
            blind_input = {
                "query_title": qp["title"],
                "query_abstract": qp["abstract"],
                "hyp_target_field": h.get("target_field", ""),
                "hyp_generic_object": h.get("generic_object", ""),
                "hyp_abstract": h.get("abstract", ""),
                "candidate_title": cp["title"],
                "candidate_abstract": cp["abstract"],
            }
            fname = f"{safe(q)}__{safe(c)}.json"
            json.dump({"input": blind_input}, open(os.path.join(in_dir, fname), "w"),
                      indent=1, ensure_ascii=False)
            records.append({
                "file": fname, "query_id": q, "candidate_id": c, "pair_id": pid,
                "winning_hyp_idx": win, "is_target": (c == target),
                "query_community": qp.get("community_id"),
                "cand_community": cp.get("community_id"),
                "cand_is_benchmark": bool(cp.get("is_benchmark")),
                "hyde_score": hyde_score(q, c),
            })

    manifest = {
        "experiment": "EXP-RS-20", "convention": "C-39", "sub": sub,
        "forward_only": forward_only, "n_queries": len(queries),
        "query_ids": [q for q, _, _ in queries], "n_records": len(records),
        "input_dir": os.path.relpath(in_dir, HERE), "records": records,
    }
    mpath = os.path.join(DATA, f"verify_manifest_{sub}.json")
    json.dump(manifest, open(mpath, "w"), indent=1, ensure_ascii=False)
    print(f"emit-inputs[{sub}]: {len(queries)} queries x candidates -> {len(records)} blind "
          f"input records in {os.path.relpath(in_dir, HERE)}/")
    print(f"  manifest: {os.path.relpath(mpath, HERE)}")
    print(f"  each input file has EXACTLY the C-39 keys (no ids). Dispatch each to a blind subagent")
    print(f"  driven by the frozen verify_prompt.md; write verdicts to "
          f"verify_outputs/{sub}/<same-name>.")
    return manifest


# ----------------------------- verdict backbones -----------------------------
def load_llm_verdicts(sub, manifest):
    """Read blind subagent outputs -> {(q,c): verdict}. Requires one file per manifest record."""
    out_dir = os.path.join(DATA, "verify_outputs", sub)
    verdicts, missing, bad = {}, [], []
    for rec in manifest["records"]:
        path = os.path.join(out_dir, rec["file"])
        if not os.path.exists(path):
            missing.append(rec["file"])
            continue
        try:
            d = json.load(open(path))
            v = {"method_coherence": bool(d["method_coherence"]),
                 "object_difference": bool(d["object_difference"]),
                 "rationale": d.get("rationale", "")}
        except (KeyError, ValueError, TypeError):
            bad.append(rec["file"])
            continue
        verdicts[(rec["query_id"], rec["candidate_id"])] = v
    return verdicts, missing, bad


def cas_verdict(blind_input, hyde_score_qc, object_sim_qc):
    """Pre-registered CAS ABLATION (Open Risk #2): a computational keyword/rule audit over the same
    C-39 closure — NOT the headline. Rules (deterministic, no LLM):
      method_coherence = the winning hypothetical actually retrieved this candidate (hyde_score>0)
        AND the candidate shares >=2 content tokens with the hypothetical's method text
        (target_field + generic_object + hyp_abstract) -> genuine shared machinery, not zero overlap.
      object_difference = low lexical overlap between query and candidate abstracts
        (object_sim < 0.15) -> a different object/substrate, not the same-field same-object paper.
    """
    hyp_txt = " ".join([blind_input["hyp_target_field"], blind_input["hyp_generic_object"],
                        blind_input["hyp_abstract"]])
    cand_txt = f"{blind_input['candidate_title']} {blind_input['candidate_abstract']}"
    shared = set(tokenize(hyp_txt)) & set(tokenize(cand_txt))
    method_coherence = (hyde_score_qc > 0.0) and (len(shared) >= 2)
    object_difference = object_sim_qc < 0.15
    return {"method_coherence": bool(method_coherence),
            "object_difference": bool(object_difference),
            "rationale": f"CAS: hyde_score={hyde_score_qc:.3f}, shared_method_tokens={len(shared)}, "
                         f"object_sim={object_sim_qc:.3f}"}


def compute_cas_verdicts(sub):
    corpus, all_ids, by_id, hyp, pairs, hyde_score, winning_idx = load_ctx(sub)
    lex = tfidf_vectors(corpus)
    verdicts = {}
    for p in pairs:
        q = p["side_a"]
        qp = by_id[q]
        for c in all_ids:
            if c == q:
                continue
            win = winning_idx(q, c)
            h = hyp[q]["hypotheticals"][win]
            cp = by_id[c]
            bi = {"query_title": qp["title"], "query_abstract": qp["abstract"],
                  "hyp_target_field": h.get("target_field", ""),
                  "hyp_generic_object": h.get("generic_object", ""),
                  "hyp_abstract": h.get("abstract", ""),
                  "candidate_title": cp["title"], "candidate_abstract": cp["abstract"]}
            verdicts[(q, c)] = cas_verdict(bi, hyde_score(q, c), cosine(lex[q], lex[c]))
    return verdicts


# ----------------------------- score + gate -----------------------------
def pruned_scorer(hyde_score, verdicts, default_coherent=True):
    """C-40: coherent -> hyde_score; method_coherence=false -> -inf (rank inf, lexicographic ties)."""
    def f(q, c):
        v = verdicts.get((q, c))
        coherent = v["method_coherence"] if v is not None else default_coherent
        return hyde_score(q, c) if coherent else NEG_INF
    return f


def prune_stats(sub, manifest, verdicts):
    """P4: per query, distractors pruned (method_coherence=false); same-community share."""
    per_q = {}
    for rec in manifest["records"]:
        q, c = rec["query_id"], rec["candidate_id"]
        v = verdicts.get((q, c))
        if v is None:
            continue
        d = per_q.setdefault(q, {"n": 0, "pruned": 0, "pruned_same_comm": 0,
                                 "pruned_target": 0, "obj_diff_false": 0})
        d["n"] += 1
        if not v["method_coherence"]:
            d["pruned"] += 1
            if rec.get("cand_community") is not None and \
                    rec.get("cand_community") == rec.get("query_community"):
                d["pruned_same_comm"] += 1
            if rec["is_target"]:
                d["pruned_target"] += 1
        if not v["object_difference"]:
            d["obj_diff_false"] += 1
    n_q = len(per_q) or 1
    total_pruned = sum(d["pruned"] for d in per_q.values())
    total_same = sum(d["pruned_same_comm"] for d in per_q.values())
    return {
        "per_query": per_q,
        "mean_pruned_per_query": total_pruned / n_q,
        "total_pruned": total_pruned,
        "same_community_share_of_pruned": (total_same / total_pruned) if total_pruned else 0.0,
        "targets_pruned": sum(d["pruned_target"] for d in per_q.values()),
    }


def run_score(sub, backbone, forward_only, allow_missing):
    corpus, all_ids, by_id, hyp, pairs, hyde_score, winning_idx = load_ctx(sub)
    mpath = os.path.join(DATA, f"verify_manifest_{sub}.json")
    if not os.path.exists(mpath):
        raise SystemExit(f"missing {mpath} — run: python verify_score.py emit-inputs --sub {sub}")
    manifest = json.load(open(mpath))

    if backbone == "cas":
        verdicts = compute_cas_verdicts(sub)
        missing, bad = [], []
    else:
        verdicts, missing, bad = load_llm_verdicts(sub, manifest)
        if (missing or bad) and not allow_missing:
            raise SystemExit(
                f"verify outputs incomplete: {len(missing)} missing, {len(bad)} malformed "
                f"(of {len(manifest['records'])}). First missing: {missing[:5]}. "
                f"Re-run the blind subagents, or pass --allow-missing (missing -> coherent, logged).")

    # HyDE-alone baseline (no prune) and the C-40 pruned cascade.
    hyde_fwd = eval_direction(pairs, all_ids, hyde_score, "side_a", "side_b")
    psc = pruned_scorer(hyde_score, verdicts, default_coherent=True)
    pruned_fwd = eval_direction(pairs, all_ids, psc, "side_a", "side_b")

    ranks = pruned_fwd["ranks"]
    recovered = {pid for pid, r in ranks.items() if r is not None and r <= 10}
    null_missed_recovered = sorted(recovered & NULL_MISSED_FEYNMAN)
    R = pruned_fwd["recall@10"]

    # P1 (cheap forward gate — decisive): R > 0.40 AND pair04 in top-10 AND >=1 null-missed pair.
    p1_beats_null = R > 0.40
    p1_pair04 = ranks.get(PAIR04) is not None and ranks[PAIR04] <= 10
    p1_null_missed = len(null_missed_recovered) >= 1
    P1 = p1_beats_null and p1_pair04 and p1_null_missed
    # P2: cascade beats HyDE-alone K=5 max-pool (0.20 for the frozen Feynman headline).
    P2 = R > 0.20
    stats = prune_stats(sub, manifest, verdicts)
    p4_ge2 = stats["mean_pruned_per_query"] >= 2.0

    results = {
        "experiment": "EXP-RS-20", "sub": sub, "backbone": backbone,
        "forward_only": forward_only, "n_corpus": len(all_ids),
        "n_eval_pairs": len(pairs), "pair_ids": [p["id"] for p in pairs],
        "n_verdicts": len(verdicts), "n_missing": len(missing), "n_malformed": len(bad),
        "missing_examples": missing[:10],
        "hyde_alone_forward": hyde_fwd,
        "cascade_forward": pruned_fwd,
        "gate": {
            "P1_cheap_forward_gate": {
                "R_recall@10": R,
                "beats_C17_null_gt_0.40": p1_beats_null,
                "pair04_recovered": p1_pair04, "pair04_rank": ranks.get(PAIR04),
                "null_missed_recovered": null_missed_recovered,
                "recovers_null_missed_pair": p1_null_missed,
                "PASS": P1,
            },
            "P2_beats_hyde_alone_gt_0.20": {"cascade_R": R, "hyde_alone_R": hyde_fwd["recall@10"],
                                            "PASS": P2},
            "P4_prune_ge2_per_query": {"mean_pruned_per_query": stats["mean_pruned_per_query"],
                                       "same_community_share": stats["same_community_share_of_pruned"],
                                       "targets_pruned": stats["targets_pruned"], "PASS": p4_ge2},
        },
        "prune_stats": stats,
        "verdicts": {f"{q}__{c}": v for (q, c), v in sorted(verdicts.items())},
    }
    out = os.path.join(DATA, f"verify_results_{sub}_{backbone}.json")
    json.dump(results, open(out, "w"), indent=1, ensure_ascii=False)

    # console summary
    print(f"corpus={SUBS[sub]['corpus']} N={len(all_ids)} pairs={len(pairs)} backbone={backbone} "
          f"verdicts={len(verdicts)}/{len(manifest['records'])} "
          f"(missing={len(missing)} malformed={len(bad)})")
    hdr = f"{'stage':20} {'R@1':>5} {'R@5':>5} {'R@10':>5} {'MRR':>6}"
    print(hdr); print("-" * len(hdr))
    for name, m in (("hyde-alone (K5 max)", hyde_fwd), ("cascade (verify-pruned)", pruned_fwd)):
        print(f"{name:20} {m['recall@1']:5.2f} {m['recall@5']:5.2f} {m['recall@10']:5.2f} "
              f"{m['mrr']:6.3f}")
    print("\ncascade forward ranks:", ranks)
    print(f"pruned/query (mean)={stats['mean_pruned_per_query']:.2f}  "
          f"same-community share={stats['same_community_share_of_pruned']:.2f}  "
          f"targets_pruned={stats['targets_pruned']}")
    g = results["gate"]
    print(f"\nP1 (R>0.40 AND pair04@<=10 AND >=1 null-missed): R={R:.3f} "
          f"beats_null={p1_beats_null} pair04_rank={ranks.get(PAIR04)} "
          f"null_missed_recovered={null_missed_recovered} -> {'PASS' if P1 else 'FAIL (KILL)'}")
    print(f"P2 (cascade {R:.2f} > hyde-alone {hyde_fwd['recall@10']:.2f}): {'PASS' if P2 else 'FAIL'}")
    print(f"P4 (>=2 pruned/query): {stats['mean_pruned_per_query']:.2f} -> {'PASS' if p4_ge2 else 'FAIL'}")
    if backbone == "llm":
        print(f"\n==> P1 CHEAP GATE (headline LLM backbone): "
              f"{'PASS' if P1 else 'FAIL -> KILL (licenses embedding-substrate escalation)'}")
    else:
        print(f"\n==> CAS ablation preview (NOT the headline; P1 verdict pins to the LLM backbone)")
    print(f"wrote {out}")
    return results


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    e = sub.add_parser("emit-inputs")
    e.add_argument("--sub", choices=list(SUBS), default="feynman")
    e.add_argument("--both-directions", action="store_true",
                   help="also emit reverse (side_b->side_a) inputs (ablation; default forward-only)")
    s = sub.add_parser("score")
    s.add_argument("--sub", choices=list(SUBS), default="feynman")
    s.add_argument("--backbone", choices=["llm", "cas"], default="llm")
    s.add_argument("--both-directions", action="store_true")
    s.add_argument("--allow-missing", action="store_true",
                   help="treat missing LLM verdicts as method_coherence=true (logged); default errors")
    args = ap.parse_args()

    if args.cmd == "emit-inputs":
        emit_inputs(args.sub, forward_only=not args.both_directions)
    elif args.cmd == "score":
        run_score(args.sub, args.backbone, forward_only=not args.both_directions,
                  allow_missing=args.allow_missing)


if __name__ == "__main__":
    main()
