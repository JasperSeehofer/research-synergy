#!/usr/bin/env python3
"""EXP-RS-30 (Phase 49) — SCALED NOVELTY TEST. Does the finder surface ANY plausibly-novel bridge?

Scales the RS-27 external finder (~5x, 720 papers / 18 categories) and adds the key change from the
RS-29 mechanism finding: novelty lives in the WEAK-MATCH TAIL, not the top-3. So candidates are drawn
in TWO strata — TOP (reduction ranks 1-3, canonical positive-control) and TAIL (ranks 4-12, the novelty
hunt) — card+adjudicated identically to RS-26/27/28, then every adjudicated-genuine bridge passes a
HARDENED RS-29 novelty-gate (two independent adversarial prior-art hunters + classifier). Pre-registered
KILL/PASS in 49-PREREG.md.

Flow:
  fetch        18 cats x 40, most-recent -> data/rs30_corpus.json
  emit-reduce  reduction inputs -> data/rs30_in/mechanism/            (then Workflow-reduce, blind Opus)
  extract      reduction+lexical embed; two strata; stratified card inputs -> data/rs30_in/openbook/
  curate       read cards -> confirmed -> data/rs30_adjudicate_input.json     (then Workflow-adjudicate)
  emit-novelty read adjudicated-genuine -> data/rs30_novelty_input.json       (then Workflow novelty-gate)
  score        adjudication + novelty verdicts -> strata breakdown + pre-registered verdict
"""
import argparse
import json
import os
import re
import time
import urllib.request

import numpy as np

from sme_lite import tfidf_vectors, cosine, rank_candidates
from embed_score import encode_st, MODEL_SPECS
import rs22_probe as R

HERE = os.path.dirname(os.path.abspath(__file__))
DATA = os.path.join(HERE, "data")
IN_MECH = os.path.join(DATA, "rs30_in", "mechanism")
OUT_MECH = os.path.join(DATA, "rs30_out", "mechanism")
IN_OB = os.path.join(DATA, "rs30_in", "openbook")
OUT_OB = os.path.join(DATA, "rs30_out", "openbook")
OUT_ADJ = os.path.join(DATA, "rs30_out", "adj")
CORPUS = os.path.join(DATA, "rs30_corpus.json")

# ---- BLIND CONSTANTS (frozen; see 49-PREREG.md) ----
CATS = ["cond-mat.stat-mech", "hep-th", "quant-ph", "math.AP", "q-bio.PE", "q-fin.ST",
        "cs.LG", "nlin.CD", "astro-ph.CO", "physics.soc-ph", "math-ph", "stat.ME",
        "math.PR", "cs.IT", "gr-qc", "q-bio.NC", "econ.TH", "nlin.PS"]
PER_CAT = 40
TOP_MAX = 3           # top stratum = reduction ranks 1..3
TAIL_MAX = 12         # tail stratum = ranks 4..12
LEX_MAX = 0.06
M_CARDS = 100         # total card budget
TAIL_MIN_CARDS = 40   # >= this many cards must come from the tail stratum


def topcat(cat):
    c = str(cat).lower()
    return c.split(".")[0] if "." in c else c


def _arxiv_query(cat, n):
    url = ("http://export.arxiv.org/api/query?search_query=cat:%s"
           "&start=0&max_results=%d&sortBy=submittedDate&sortOrder=descending" % (cat, n))
    req = urllib.request.Request(url, headers={"User-Agent": "resyn-research/0.1 (mailto:jasperseehofermusic@gmail.com)"})
    xml = urllib.request.urlopen(req, timeout=60).read().decode("utf-8", "replace")
    entries = re.findall(r"<entry>(.*?)</entry>", xml, re.S)
    out = []
    for e in entries:
        idm = re.search(r"<id>http://arxiv\.org/abs/([^<]+)</id>", e)
        tm = re.search(r"<title>(.*?)</title>", e, re.S)
        sm = re.search(r"<summary>(.*?)</summary>", e, re.S)
        pcm = re.search(r'<arxiv:primary_category[^>]*term="([^"]+)"', e)
        if not (idm and tm and sm):
            continue
        aid = R.normalize_id(idm.group(1).strip())
        title = " ".join(tm.group(1).split())
        abstract = " ".join(sm.group(1).split())
        cat_p = pcm.group(1) if pcm else cat
        if len(abstract) >= 200:
            out.append({"arxiv_id": aid, "title": title, "abstract": abstract, "category": cat_p})
    return out


def fetch():
    papers, seen = [], set()
    for cat in CATS:
        try:
            got = _arxiv_query(cat, PER_CAT + 8)
        except Exception as e:  # noqa: BLE001
            print(f"  [fetch {cat} failed: {e}]"); got = []
        k = 0
        for p in got:
            if p["arxiv_id"] in seen:
                continue
            seen.add(p["arxiv_id"]); papers.append(p); k += 1
            if k >= PER_CAT:
                break
        print(f"  {cat}: {k}")
        time.sleep(3.0)
    json.dump({"papers": papers, "cats": CATS}, open(CORPUS, "w"), indent=1, ensure_ascii=False)
    print(f"fetch: {len(papers)} papers across {len(CATS)} categories -> data/rs30_corpus.json")


def emit_reduce():
    papers = json.load(open(CORPUS))["papers"]
    os.makedirs(IN_MECH, exist_ok=True)
    for p in papers:
        inp = {"title": p["title"], "abstract": p["abstract"]}
        json.dump({"input": inp, "instr": "mechanism", "key": p["arxiv_id"]},
                  open(os.path.join(IN_MECH, f"{R.safe(p['arxiv_id'])}.json"), "w"), indent=1, ensure_ascii=False)
    print(f"emit-reduce: {len(papers)} reduction inputs -> {os.path.relpath(IN_MECH, HERE)}/ (n={len(papers)})")


def _bge(texts_papers):
    hf_id, _ = MODEL_SPECS["bge"]
    V, _, _, _ = encode_st(hf_id, texts_papers, "bge")
    return np.asarray(V)


def extract():
    papers = json.load(open(CORPUS))["papers"]
    by_id = {p["arxiv_id"]: p for p in papers}
    ids = [p["arxiv_id"] for p in papers]
    red = {}
    for aid in ids:
        p = os.path.join(OUT_MECH, f"{R.safe(aid)}.json")
        red[aid] = R._load_json_out(p).get("core_mechanism", "").strip() if os.path.exists(p) else ""
    missing = [a for a in ids if not red[a]]
    if missing:
        raise SystemExit(f"{len(missing)} reductions missing (e.g. {missing[:4]}) — reduce first")
    V = _bge([{"title": red[a], "abstract": ""} for a in ids])
    idx = {a: i for i, a in enumerate(ids)}
    lex = tfidf_vectors({"papers": papers})

    def red_s(q, c):
        return float(np.dot(V[idx[q]], V[idx[c]]))

    cands, seen_pair = [], {}
    for q in ids:
        ranked = [c for c, _ in rank_candidates(q, ids, red_s)]
        q_arch = topcat(by_id[q]["category"])
        for rank, c in enumerate(ranked[:TAIL_MAX], start=1):
            if topcat(by_id[c]["category"]) == q_arch:
                continue
            lx = cosine(lex[q], lex[c])
            if lx >= LEX_MAX:
                continue
            key = tuple(sorted((q, c)))
            stratum = "top" if rank <= TOP_MAX else "tail"
            rec = {"query_id": q, "query_title": by_id[q]["title"], "query_cat": by_id[q]["category"],
                   "cand_id": c, "cand_title": by_id[c]["title"], "cand_cat": by_id[c]["category"],
                   "reduction_rank": rank, "lexical_cos": round(lx, 4), "stratum": stratum}
            if key not in seen_pair or rank < seen_pair[key]["reduction_rank"]:
                seen_pair[key] = rec
    cands = list(seen_pair.values())
    cands.sort(key=lambda d: (d["reduction_rank"], d["lexical_cos"], d["query_id"], d["cand_id"]))
    top = [c for c in cands if c["stratum"] == "top"]
    tail = [c for c in cands if c["stratum"] == "tail"]
    json.dump({"n": len(cands), "n_top": len(top), "n_tail": len(tail), "candidates": cands},
              open(os.path.join(DATA, "rs30_candidates.json"), "w"), indent=1, ensure_ascii=False)

    # stratified card budget: >= TAIL_MIN_CARDS from tail, fill to M_CARDS from top, then remainder
    n_tail = min(TAIL_MIN_CARDS, len(tail))
    n_top = min(M_CARDS - n_tail, len(top))
    carded = top[:n_top] + tail[:n_tail]
    # fill any remaining budget from whichever stratum has leftovers
    if len(carded) < M_CARDS:
        extra = top[n_top:] + tail[n_tail:]
        extra.sort(key=lambda d: (d["reduction_rank"], d["lexical_cos"]))
        carded += extra[:M_CARDS - len(carded)]
    carded.sort(key=lambda d: (d["stratum"], d["reduction_rank"], d["lexical_cos"]))
    os.makedirs(IN_OB, exist_ok=True)
    for i, d in enumerate(carded):
        a, b = by_id[d["query_id"]], by_id[d["cand_id"]]
        inp = {"title_a": a["title"], "abstract_a": a["abstract"],
               "title_b": b["title"], "abstract_b": b["abstract"]}
        json.dump({"input": inp, "instr": "openbook", "key": f"s-{i:03d}",
                   "pair": {"query_id": d["query_id"], "cand_id": d["cand_id"], "stratum": d["stratum"],
                            "reduction_rank": d["reduction_rank"], "lexical_cos": d["lexical_cos"]}},
                  open(os.path.join(IN_OB, f"s-{i:03d}.json"), "w"), indent=1, ensure_ascii=False)
    json.dump({"carded": carded}, open(os.path.join(DATA, "rs30_carded.json"), "w"), indent=1, ensure_ascii=False)
    n_ct = sum(1 for c in carded if c["stratum"] == "top")
    n_cta = sum(1 for c in carded if c["stratum"] == "tail")
    print(f"extract: {len(cands)} candidates (top {len(top)}, tail {len(tail)}); "
          f"carded {len(carded)} = {n_ct} top + {n_cta} tail -> data/rs30_in/openbook/ (n={len(carded)})")


def curate():
    carded = json.load(open(os.path.join(DATA, "rs30_carded.json")))["carded"]
    papers = {p["arxiv_id"]: p for p in json.load(open(CORPUS))["papers"]}
    conf = []
    for i, d in enumerate(carded):
        p = os.path.join(OUT_OB, f"s-{i:03d}.json")
        if not os.path.exists(p):
            continue
        card = R._load_json_out(p)
        if bool(card.get("shares_method")):
            conf.append({**d, "shared_mechanism": card.get("brief_justification", ""),
                         "q_abs": papers[d["query_id"]]["abstract"], "c_abs": papers[d["cand_id"]]["abstract"]})
    by_strat = {"top": [c for c in conf if c["stratum"] == "top"],
                "tail": [c for c in conf if c["stratum"] == "tail"]}
    adj = [{"id": f"S{i}", "stratum": c["stratum"], "field_a": c["query_cat"], "title_a": c["query_title"],
            "abstract_a": c["q_abs"], "field_b": c["cand_cat"], "title_b": c["cand_title"], "abstract_b": c["c_abs"],
            "proposed_shared_mechanism": c["shared_mechanism"], "lexical_cos": c["lexical_cos"],
            "reduction_rank": c["reduction_rank"], "query_id": c["query_id"], "cand_id": c["cand_id"]}
           for i, c in enumerate(conf)]
    json.dump({"bridges": adj}, open(os.path.join(DATA, "rs30_adjudicate_input.json"), "w"), indent=1, ensure_ascii=False)
    print(f"curate: {len(carded)} carded -> {len(conf)} card-confirmed "
          f"(top {len(by_strat['top'])}, tail {len(by_strat['tail'])}) -> data/rs30_adjudicate_input.json")


def emit_novelty():
    """After adjudication: emit the genuine bridges as novelty-gate inputs (RS-29 hunter/classifier)."""
    adj = R._load_json_out(os.path.join(OUT_ADJ, "adjudication.json"))
    inp = {b["id"]: b for b in json.load(open(os.path.join(DATA, "rs30_adjudicate_input.json")))["bridges"]}
    genuine = [v for v in adj.get("verdicts", []) if v.get("mechanism_real")]
    bridges = []
    for v in genuine:
        b = inp.get(v["id"])
        if not b:
            continue
        bridges.append({"id": v["id"], "stratum": b["stratum"], "reduction_rank": b["reduction_rank"],
                        "fieldA": b["field_a"], "fieldB": b["field_b"], "titleA": b["title_a"],
                        "titleB": b["title_b"], "arxivA": b["query_id"], "arxivB": b["cand_id"],
                        "mechanism": b["proposed_shared_mechanism"], "adj_novelty": v.get("novelty")})
    json.dump({"bridges": bridges}, open(os.path.join(DATA, "rs30_novelty_input.json"), "w"), indent=1, ensure_ascii=False)
    n_top = sum(1 for b in bridges if b["stratum"] == "top")
    n_tail = sum(1 for b in bridges if b["stratum"] == "tail")
    print(f"emit-novelty: {len(bridges)} adjudicated-genuine bridges (top {n_top}, tail {n_tail}) "
          f"-> data/rs30_novelty_input.json (run the RS-29 two-hunter novelty-gate on these)")


def score():
    carded = json.load(open(os.path.join(DATA, "rs30_carded.json")))["carded"]
    adj = R._load_json_out(os.path.join(OUT_ADJ, "adjudication.json"))
    genuine = {v["id"] for v in adj.get("verdicts", []) if v.get("mechanism_real")}
    inp = {b["id"]: b for b in json.load(open(os.path.join(DATA, "rs30_adjudicate_input.json")))["bridges"]}
    nov = json.load(open(os.path.join(DATA, "rs30_novelty_verdicts.json")))["verdicts"] \
        if os.path.exists(os.path.join(DATA, "rs30_novelty_verdicts.json")) else []
    novmap = {v["id"]: v for v in nov}

    def strat_counts(pred):
        return {s: sum(1 for c in carded if c["stratum"] == s and pred(c)) for s in ("top", "tail")}

    carded_by = strat_counts(lambda c: True)
    # genuine per stratum
    gen_by = {"top": 0, "tail": 0}
    breakdown = {"known_crossfield": {"top": 0, "tail": 0}, "specialist_known": {"top": 0, "tail": 0},
                 "candidate_novel": {"top": 0, "tail": 0}, "robust_novel": {"top": 0, "tail": 0}}
    robust_novel_bridges, candidate_novel_bridges = [], []
    for gid in genuine:
        b = inp.get(gid)
        if not b:
            continue
        s = b["stratum"]
        gen_by[s] += 1
        v = novmap.get(gid)
        if not v:
            continue
        cls = v.get("novelty_class")
        both_notfound = v.get("both_hunters_not_found", False)
        if cls == "novel_looking" and both_notfound:
            breakdown["robust_novel"][s] += 1
            robust_novel_bridges.append({**b, **v})
        elif cls == "novel_looking":
            breakdown["candidate_novel"][s] += 1
            candidate_novel_bridges.append({**b, **v})
        elif cls in ("known_crossfield", "specialist_known"):
            breakdown[cls][s] += 1

    n_robust = sum(breakdown["robust_novel"].values())
    n_cand = sum(breakdown["candidate_novel"].values())
    top_genuine = gen_by["top"]
    if top_genuine == 0:
        verdict = "INVALID"
    elif n_robust >= 1:
        verdict = "PASS"
    elif n_cand >= 1:
        verdict = "WEAK"
    else:
        verdict = "KILL"

    out = {"experiment": "EXP-RS-30", "verdict": verdict,
           "carded": carded_by, "genuine": gen_by, "novelty_breakdown": breakdown,
           "n_robust_novel": n_robust, "n_candidate_novel": n_cand,
           "robust_novel_bridges": robust_novel_bridges, "candidate_novel_bridges": candidate_novel_bridges}
    json.dump(out, open(os.path.join(DATA, "rs30_verdict.json"), "w"), indent=1, ensure_ascii=False)
    print(f"=== EXP-RS-30 scaled novelty test — VERDICT: {verdict} ===")
    print(f"  carded: top {carded_by['top']}, tail {carded_by['tail']}")
    print(f"  adjudicated-genuine: top {gen_by['top']}, tail {gen_by['tail']} (positive control: top>=1 = {top_genuine>=1})")
    print(f"  novelty breakdown (top/tail):")
    for k in ("known_crossfield", "specialist_known", "candidate_novel", "robust_novel"):
        print(f"    {k:18} top {breakdown[k]['top']}  tail {breakdown[k]['tail']}")
    print(f"  robust_novel={n_robust}  candidate_novel={n_cand}  -> {verdict}")
    if robust_novel_bridges:
        for b in robust_novel_bridges:
            print(f"    ROBUST-NOVEL {b['id']} [{b['stratum']}] {b['fieldA'][:30]} <-> {b['fieldB'][:30]}")
    print(f"  -> data/rs30_verdict.json")


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    for c in ("fetch", "emit-reduce", "extract", "curate", "emit-novelty", "score"):
        sub.add_parser(c)
    a = ap.parse_args()
    {"fetch": fetch, "emit-reduce": emit_reduce, "extract": extract, "curate": curate,
     "emit-novelty": emit_novelty, "score": score}[a.cmd]()


if __name__ == "__main__":
    main()
