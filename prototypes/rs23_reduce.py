#!/usr/bin/env python3
"""EXP-RS-23 (Phase 42) — Mechanism-Reduction Substrate.

Apples-to-apples with EXP-RS-21: the ONLY change is the TEXT that gets embedded — raw
`title+'. '+abstract` (RS-21, = 0.20) becomes each paper's FIELD-NEUTRAL core-mechanism reduction
(the frozen blind mechanism probe rs22_probe_mechanism.md, one LLM call per paper, no partner / no
benchmark). Same corpus (mvp_corpus, 36), same 5 pairs, same encoder (bge symmetric), same C-19
metric (forward recall@k, ties→arxiv_id lexicographic). Tests whether distilling to the mechanism
before embedding rescues the dense null RS-21 killed by field dominance.

Subcommands:
  emit-inputs   write per-paper blind mechanism-reduction inputs for all 36 Feynman papers
  score         read reductions -> embed (bge) -> forward recall@10 for reduction_bge vs raw_bge
                (RS-21 control) vs lexical_null; print the frozen 42-PREREG gate

Usage:
  python rs23_reduce.py emit-inputs
  # dispatch data/rs23_in/mechanism/*.json to blind subagents (frozen rs22_probe_mechanism.md) ->
  #   data/rs23_out/mechanism/<id>.json  (a Workflow fan-out; 36 calls)
  python rs23_reduce.py score
"""
import argparse
import hashlib
import json
import os

import numpy as np

from sme_lite import tfidf_vectors, cosine, eval_direction
from embed_score import encode_st, MODEL_SPECS
import rs22_probe as R  # _load_json_out, safe, normalize_id

HERE = os.path.dirname(os.path.abspath(__file__))
DATA = os.path.join(HERE, "data")
IN_DIR = os.path.join(DATA, "rs23_in", "mechanism")
OUT_DIR = os.path.join(DATA, "rs23_out", "mechanism")
MECH_PROMPT = os.path.join(HERE, "rs22_probe_mechanism.md")

LEXICAL_NULL = 0.40   # C-17 Feynman fwd recall@10 (verified EXP-RS-19)
RAW_BGE_RS21 = 0.20   # RS-21 bge raw-abstract fwd recall@10
LLM_CEILING = "0.60 (old) / ~1.00 (Opus 4.8 anchors)"


def load_ctx():
    corpus = json.load(open(os.path.join(DATA, "mvp_corpus.json")))
    feyn = json.load(open(os.path.join(DATA, "feynman_10pair_papers.json")))
    all_ids = [p["arxiv_id"] for p in corpus["papers"]]
    by_id = {p["arxiv_id"]: p for p in corpus["papers"]}
    ev = set(feyn["evaluable_pairs"])
    pairs = []
    for p in feyn["pairs"]:
        if p["id"].split("-")[0] in ev:
            a, b = p["side_a"]["arxiv_id"], p["side_b"]["arxiv_id"]
            if a in by_id and b in by_id:
                pairs.append({"id": p["id"], "side_a": a, "side_b": b})
    return corpus, all_ids, by_id, pairs


def emit_inputs():
    corpus, all_ids, by_id, pairs = load_ctx()
    os.makedirs(IN_DIR, exist_ok=True)
    for aid in all_ids:
        p = by_id[aid]
        inp = {"title": p["title"], "abstract": p["abstract"]}
        sha = hashlib.sha256(json.dumps(inp, sort_keys=True, ensure_ascii=False).encode()).hexdigest()
        json.dump({"input": inp, "input_sha256": sha, "instr": "mechanism", "key": aid},
                  open(os.path.join(IN_DIR, f"{R.safe(aid)}.json"), "w"), indent=1, ensure_ascii=False)
    print(f"emit-inputs: {len(all_ids)} mechanism-reduction inputs -> {os.path.relpath(IN_DIR, HERE)}/")
    print(f"  dispatch each to a blind subagent (frozen {os.path.basename(MECH_PROMPT)}) -> "
          f"{os.path.relpath(OUT_DIR, HERE)}/<id>.json")
    # emit the id list for the Workflow args
    json.dump({"promptDir": HERE, "inDir": os.path.join(DATA, "rs23_in"),
               "outDir": os.path.join(DATA, "rs23_out"),
               "prompts": {"mechanism": "rs22_probe_mechanism.md"},
               "keys": [R.safe(a) for a in all_ids], "raw_keys": all_ids},
              open(os.path.join(DATA, "rs23_args.json"), "w"), indent=1)
    print(f"  workflow args: {os.path.relpath(os.path.join(DATA,'rs23_args.json'), HERE)}")


def _bge(papers_list):
    hf_id, _ = MODEL_SPECS["bge"]
    V, rev, _, _ = encode_st(hf_id, papers_list, "bge")   # symmetric, L2-normed
    return np.asarray(V), rev


def _emb_scorer(all_ids, papers_list):
    V, rev = _bge(papers_list)
    idx = {a: i for i, a in enumerate(all_ids)}

    def f(q, c):
        return float(np.dot(V[idx[q]], V[idx[c]]))
    return f, rev


def score():
    corpus, all_ids, by_id, pairs = load_ctx()
    # load reductions
    reductions, missing = {}, []
    for aid in all_ids:
        p = os.path.join(OUT_DIR, f"{R.safe(aid)}.json")
        if not os.path.exists(p):
            missing.append(aid)
            continue
        d = R._load_json_out(p)
        reductions[aid] = {"core": (d.get("core_mechanism") or "").strip(),
                           "reason": (d.get("brief_reason") or "").strip()}
    if missing:
        raise SystemExit(f"missing {len(missing)} reductions (e.g. {missing[:5]}); dispatch them first.")

    # arm text builders (feed encode_st via st_text = 'title. abstract')
    raw_list = [{"title": by_id[a]["title"], "abstract": by_id[a]["abstract"]} for a in all_ids]
    red_list = [{"title": reductions[a]["core"], "abstract": ""} for a in all_ids]
    redfull_list = [{"title": reductions[a]["core"], "abstract": reductions[a]["reason"]} for a in all_ids]

    lex = tfidf_vectors(corpus)
    lex_score = lambda q, c: cosine(lex[q], lex[c])
    red_score, red_rev = _emb_scorer(all_ids, red_list)
    raw_score, raw_rev = _emb_scorer(all_ids, raw_list)
    redfull_score, _ = _emb_scorer(all_ids, redfull_list)

    arms = {"reduction_bge": red_score, "raw_bge": raw_score,
            "reduction_bge_full": redfull_score, "lexical_null": lex_score}
    results = {}
    for name, fn in arms.items():
        results[name] = eval_direction(pairs, all_ids, fn, "side_a", "side_b")

    red10 = results["reduction_bge"]["recall@10"]
    raw10 = results["raw_bge"]["recall@10"]
    lex10 = results["lexical_null"]["recall@10"]
    # frozen 42-PREREG predictions
    P1 = red10 > 0.40            # beats the lexical null
    P2 = red10 > raw10           # beats raw-abstract embedding (isolates the reduction)
    if P1 and P2:
        verdict = "ADVANCE — field-neutral reduction rescues cheap retrieval; build reduction->LLM cascade"
    elif P2 and not P1:
        verdict = "PARTIAL — reduction concentrates signal (beats raw embeddings) but ties/below the lexical null"
    else:
        verdict = "KILL — structure cannot be compressed even by an LLM reduction; raw-text LLM is necessary"

    out = {"experiment": "EXP-RS-23", "corpus": "mvp_corpus.json (Feynman)", "n_corpus": len(all_ids),
           "n_pairs": len(pairs), "pair_ids": [p["id"] for p in pairs],
           "encoder": "bge-large-en-v1.5 symmetric", "bge_revision": red_rev,
           "reduction_prompt": "rs22_probe_mechanism.md",
           "arms_forward_recall@10": {k: results[k]["recall@10"] for k in arms},
           "arms_full": results,
           "floors": {"lexical_null": LEXICAL_NULL, "raw_bge_rs21": RAW_BGE_RS21, "llm_ceiling": LLM_CEILING},
           "predictions": {"P1_beats_lexical_null(>0.40)": P1, "P2_beats_raw_bge": P2},
           "verdict": verdict,
           "per_pair_reduction_ranks": results["reduction_bge"]["ranks"],
           "per_pair_raw_ranks": results["raw_bge"]["ranks"]}
    json.dump(out, open(os.path.join(DATA, "rs23_results.json"), "w"), indent=1, ensure_ascii=False)

    print(f"=== EXP-RS-23 (Feynman, n_pairs={len(pairs)}, bge {red_rev[:12]}) ===")
    hdr = f"{'arm':22} {'R@1':>5} {'R@5':>5} {'R@10':>5} {'MRR':>6}"
    print(hdr); print("-" * len(hdr))
    for name in ("reduction_bge", "reduction_bge_full", "raw_bge", "lexical_null"):
        m = results[name]
        print(f"{name:22} {m['recall@1']:5.2f} {m['recall@5']:5.2f} {m['recall@10']:5.2f} {m['mrr']:6.3f}")
    print(f"\nfloors: lexical_null=0.40  raw_bge(RS-21)=0.20  llm_rawtext={LLM_CEILING}")
    print(f"reduction ranks (fwd): {results['reduction_bge']['ranks']}")
    print(f"raw_bge   ranks (fwd): {results['raw_bge']['ranks']}")
    print(f"\nP1 reduction>0.40 lexical null: {P1}  |  P2 reduction>raw_bge({raw10:.2f}): {P2}")
    print(f"\n==> {verdict}")
    print(f"wrote {os.path.relpath(os.path.join(DATA,'rs23_results.json'), HERE)}")
    return out


# ----------------------------- mined-corpus confirmation (cross-archive) -----------------------------
MIN_IN = os.path.join(DATA, "rs23_in_mined", "mechanism")
MIN_OUT = os.path.join(DATA, "rs23_out_mined", "mechanism")


def load_mined(n):
    """Self-contained sub-benchmark from the first n mined pairs: 2n UNIQUE papers (endpoints are
    globally unique per the mining dedup), rank-all-others metric like the Feynman testbed."""
    pairs_raw = json.load(open(os.path.join(DATA, "rs22_mined_pairs.json")))["pairs"][:n]
    by_id, all_ids, pairs = {}, [], []
    for p in pairs_raw:
        a = R.normalize_id(p["side_a"]["arxiv_id"])
        b = R.normalize_id(p["side_b"]["arxiv_id"])
        for side, sid in ((p["side_a"], a), (p["side_b"], b)):
            if sid not in by_id:
                by_id[sid] = {"arxiv_id": sid, "title": side["title"], "abstract": side["abstract"],
                              "category": side["category"]}
                all_ids.append(sid)
        pairs.append({"id": p["pair_id"], "side_a": a, "side_b": b})
    return all_ids, by_id, pairs


def emit_mined(n):
    all_ids, by_id, pairs = load_mined(n)
    os.makedirs(MIN_IN, exist_ok=True)
    for aid in all_ids:
        p = by_id[aid]
        inp = {"title": p["title"], "abstract": p["abstract"]}
        json.dump({"input": inp, "instr": "mechanism", "key": aid},
                  open(os.path.join(MIN_IN, f"{R.safe(aid)}.json"), "w"), indent=1, ensure_ascii=False)
    json.dump({"promptDir": HERE, "inDir": os.path.join(DATA, "rs23_in_mined"),
               "outDir": os.path.join(DATA, "rs23_out_mined"),
               "prompts": {"mechanism": "rs22_probe_mechanism.md"},
               "keys": [R.safe(a) for a in all_ids]},
              open(os.path.join(DATA, "rs23_mined_args.json"), "w"), indent=1)
    print(f"emit-mined[n={n}]: {len(all_ids)} unique papers ({len(pairs)} pairs) -> "
          f"{os.path.relpath(MIN_IN, HERE)}/ ; args data/rs23_mined_args.json")


def score_mined(n):
    all_ids, by_id, pairs = load_mined(n)
    red, missing = {}, []
    for aid in all_ids:
        p = os.path.join(MIN_OUT, f"{R.safe(aid)}.json")
        if not os.path.exists(p):
            missing.append(aid); continue
        d = R._load_json_out(p)
        red[aid] = (d.get("core_mechanism") or "").strip()
    if missing:
        raise SystemExit(f"missing {len(missing)} mined reductions (e.g. {missing[:5]})")
    corpus = {"papers": [by_id[a] for a in all_ids]}
    raw_list = [{"title": by_id[a]["title"], "abstract": by_id[a]["abstract"]} for a in all_ids]
    red_list = [{"title": red[a], "abstract": ""} for a in all_ids]
    lex = tfidf_vectors(corpus)
    lex_score = lambda q, c: cosine(lex[q], lex[c])
    red_score, rev = _emb_scorer(all_ids, red_list)
    raw_score, _ = _emb_scorer(all_ids, raw_list)
    arms = {"reduction_bge": red_score, "raw_bge": raw_score, "lexical_null": lex_score}
    res = {k: eval_direction(pairs, all_ids, fn, "side_a", "side_b") for k, fn in arms.items()}
    r10, raw10, lex10 = (res["reduction_bge"]["recall@10"], res["raw_bge"]["recall@10"],
                         res["lexical_null"]["recall@10"])
    P1, P2 = r10 > lex10, r10 > raw10
    verdict = ("CONFIRM — reduction beats BOTH lexical + raw embedding on the cross-archive set" if (P1 and P2)
               else "PARTIAL — reduction beats raw embedding but not the lexical null here" if P2
               else "NEGATIVE — reduction does not beat raw embedding on the cross-archive set")
    out = {"experiment": "EXP-RS-23-mined", "n_pairs": len(pairs), "n_corpus": len(all_ids),
           "encoder": f"bge {rev[:12]}", "arms_forward": {k: res[k]["recall@10"] for k in arms},
           "recall@5": {k: res[k]["recall@5"] for k in arms}, "mrr": {k: res[k]["mrr"] for k in arms},
           "P1_beats_lexical": P1, "P2_beats_raw_bge": P2, "verdict": verdict}
    json.dump(out, open(os.path.join(DATA, "rs23_results_mined.json"), "w"), indent=1, ensure_ascii=False)
    print(f"=== EXP-RS-23 mined confirmation (cross-archive, n_pairs={len(pairs)}, N={len(all_ids)}) ===")
    hdr = f"{'arm':16} {'R@5':>5} {'R@10':>5} {'MRR':>6}"
    print(hdr); print("-" * len(hdr))
    for k in ("reduction_bge", "raw_bge", "lexical_null"):
        m = res[k]; print(f"{k:16} {m['recall@5']:5.2f} {m['recall@10']:5.2f} {m['mrr']:6.3f}")
    print(f"\nP1 reduction>lexical({lex10:.2f}): {P1}  |  P2 reduction>raw_bge({raw10:.2f}): {P2}")
    print(f"==> {verdict}\nwrote data/rs23_results_mined.json")
    return out


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    sub.add_parser("emit-inputs")
    sub.add_parser("score")
    em = sub.add_parser("emit-mined"); em.add_argument("--n", type=int, default=80)
    sm = sub.add_parser("score-mined"); sm.add_argument("--n", type=int, default=80)
    a = ap.parse_args()
    if a.cmd == "emit-inputs":
        emit_inputs()
    elif a.cmd == "score":
        score()
    elif a.cmd == "emit-mined":
        emit_mined(a.n)
    elif a.cmd == "score-mined":
        score_mined(a.n)


if __name__ == "__main__":
    main()
