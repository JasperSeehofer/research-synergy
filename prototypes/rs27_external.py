#!/usr/bin/env python3
"""EXP-RS-27 (Phase 46) — external-corpus discovery: run the reduction finder on a FRESH, un-mined
cross-field arXiv corpus (papers fetched by category, NOT via bridge papers, so any bridge is genuinely
un-mined). Simpler than RS-26: the discovery signal is "the mechanism reduction surfaces C for Q while
lexical does NOT" — no benchmark re-rank needed.

Flow:
  fetch        stratified arXiv fetch across diverse categories -> data/rs27_corpus.json
  emit-reduce  reduction inputs for every paper -> data/rs27_in/mechanism/   (then Workflow-reduce)
  extract      reduction+lexical embed; hidden bridges = reduction top-K ∧ cross-archive ∧ lexical<LEX;
               emit open-book card inputs -> data/rs27_in/openbook/           (then Workflow-card)
  curate       read cards -> confirmed cross-field bridges, ranked
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
IN_MECH = os.path.join(DATA, "rs27_in", "mechanism")
OUT_MECH = os.path.join(DATA, "rs27_out", "mechanism")
IN_OB = os.path.join(DATA, "rs27_in", "openbook")
OUT_OB = os.path.join(DATA, "rs27_out", "openbook")
CORPUS = os.path.join(DATA, "rs27_corpus.json")

CATS = ["cond-mat.stat-mech", "hep-th", "quant-ph", "math.AP", "q-bio.PE", "q-fin.ST",
        "cs.LG", "nlin.CD", "astro-ph.CO", "physics.soc-ph", "math-ph", "stat.ME"]
PER_CAT = 12
RANK_MAX = 3
LEX_MAX = 0.06
M_CARDS = 40


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
            got = _arxiv_query(cat, PER_CAT + 4)
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
    print(f"fetch: {len(papers)} papers across {len(CATS)} categories -> data/rs27_corpus.json")


def emit_reduce():
    papers = json.load(open(CORPUS))["papers"]
    os.makedirs(IN_MECH, exist_ok=True)
    for p in papers:
        inp = {"title": p["title"], "abstract": p["abstract"]}
        json.dump({"input": inp, "instr": "mechanism", "key": p["arxiv_id"]},
                  open(os.path.join(IN_MECH, f"{R.safe(p['arxiv_id'])}.json"), "w"), indent=1, ensure_ascii=False)
    print(f"emit-reduce: {len(papers)} reduction inputs -> {os.path.relpath(IN_MECH, HERE)}/ "
          f"(n={len(papers)} for the index-based Workflow)")


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

    cands, seen_pair = [], set()
    for q in ids:
        ranked = [c for c, _ in rank_candidates(q, ids, red_s)]
        q_arch = topcat(by_id[q]["category"])
        for rank, c in enumerate(ranked[:RANK_MAX], start=1):
            if topcat(by_id[c]["category"]) == q_arch:
                continue
            key = tuple(sorted((q, c)))
            if key in seen_pair:
                continue
            lx = cosine(lex[q], lex[c])
            if lx >= LEX_MAX:
                continue
            seen_pair.add(key)
            cands.append({"query_id": q, "query_title": by_id[q]["title"], "query_cat": by_id[q]["category"],
                          "cand_id": c, "cand_title": by_id[c]["title"], "cand_cat": by_id[c]["category"],
                          "reduction_rank": rank, "lexical_cos": round(lx, 4)})
    cands.sort(key=lambda d: (d["reduction_rank"], d["lexical_cos"], d["query_id"], d["cand_id"]))
    json.dump({"n": len(cands), "candidates": cands}, open(os.path.join(DATA, "rs27_candidates.json"), "w"),
              indent=1, ensure_ascii=False)
    os.makedirs(IN_OB, exist_ok=True)
    top = cands[:M_CARDS]
    for i, d in enumerate(top):
        a, b = by_id[d["query_id"]], by_id[d["cand_id"]]
        inp = {"title_a": a["title"], "abstract_a": a["abstract"],
               "title_b": b["title"], "abstract_b": b["abstract"]}
        json.dump({"input": inp, "instr": "openbook", "key": f"ext-{i:03d}",
                   "pair": {"query_id": d["query_id"], "cand_id": d["cand_id"]}},
                  open(os.path.join(IN_OB, f"ext-{i:03d}.json"), "w"), indent=1, ensure_ascii=False)
    print(f"extract: {len(cands)} hidden-bridge candidates -> data/rs27_candidates.json; "
          f"emitted top {len(top)} card inputs (n={len(top)}). Top 6:")
    for d in top[:6]:
        print(f"  [{d['reduction_rank']}|lex {d['lexical_cos']:.3f}] {d['query_cat']:16} {d['query_title'][:40]!r}")
        print(f"       <-> {d['cand_cat']:16} {d['cand_title'][:40]!r}")


def curate():
    cand = json.load(open(os.path.join(DATA, "rs27_candidates.json")))["candidates"][:M_CARDS]
    papers = {p["arxiv_id"]: p for p in json.load(open(CORPUS))["papers"]}
    conf, rej = [], 0
    for i, d in enumerate(cand):
        p = os.path.join(OUT_OB, f"ext-{i:03d}.json")
        if not os.path.exists(p):
            continue
        card = R._load_json_out(p)
        if bool(card.get("shares_method")):
            conf.append({**d, "shared_mechanism": card.get("brief_justification", ""),
                         "q_abs": papers[d["query_id"]]["abstract"], "c_abs": papers[d["cand_id"]]["abstract"]})
        else:
            rej += 1
    nj = len(conf) + rej
    conf.sort(key=lambda d: (d["reduction_rank"], d["lexical_cos"]))
    json.dump({"experiment": "EXP-RS-27", "n_carded": nj, "n_confirmed": len(conf),
               "hit_rate": round(len(conf) / nj, 3) if nj else None,
               "confirmed_bridges": [{k: v for k, v in c.items() if k not in ("q_abs", "c_abs")} for c in conf]},
              open(os.path.join(DATA, "rs27_discoveries.json"), "w"), indent=1, ensure_ascii=False)
    # adjudication input (with abstracts + mechanism)
    adj = [{"id": f"E{i}", "field_a": c["query_cat"], "title_a": c["query_title"], "abstract_a": c["q_abs"],
            "field_b": c["cand_cat"], "title_b": c["cand_title"], "abstract_b": c["c_abs"],
            "proposed_shared_mechanism": c["shared_mechanism"], "lexical_cos": c["lexical_cos"]}
           for i, c in enumerate(conf)]
    json.dump({"bridges": adj}, open(os.path.join(DATA, "rs27_adjudicate_input.json"), "w"), indent=1, ensure_ascii=False)
    print(f"=== EXP-RS-27 external discovery — {len(conf)}/{nj} carded confirmed shared-method "
          f"(hit-rate {conf and round(len(conf)/nj,3)}) ===")
    for d in conf[:12]:
        print(f"\n• [{d['query_cat']}] {d['query_title']}")
        print(f"  ↔ [{d['cand_cat']}] {d['cand_title']}  (lex {d['lexical_cos']:.3f})")
        print(f"  {d['shared_mechanism'][:260]}")
    print(f"\nwrote data/rs27_discoveries.json + rs27_adjudicate_input.json ({len(adj)} for blind adjudication)")


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    for c in ("fetch", "emit-reduce", "extract", "curate"):
        sub.add_parser(c)
    a = ap.parse_args()
    {"fetch": fetch, "emit-reduce": emit_reduce, "extract": extract, "curate": curate}[a.cmd]()


if __name__ == "__main__":
    main()
