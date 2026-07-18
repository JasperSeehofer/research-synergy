#!/usr/bin/env python3
"""EXP-RS-26 (Phase 45) — Discovery run: surface HIDDEN cross-field bridges via the cascade.

The LBD payoff. Reuses the EXP-RS-25 cascade (raw ∪ reduction → Claude LLM re-rank) over the 80-pair
mined sample. A "hidden bridge" for a query paper X = a candidate paper Y such that:
  (1) Y is a DIFFERENT top-level arXiv archive from X (genuinely cross-field),
  (2) Y is NOT X's known mined partner (no bridge paper linked them = candidate-novel),
  (3) the cascade (LLM re-rank of the union) ranks Y HIGH for X (top-`RANK_MAX`),
  (4) lexical cosine(X, Y) is LOW (< `LEX_MAX`) — surface/topical retrieval would MISS it; only the
      mechanism reduction + cascade surfaced it (the whole point of LBD),
  (5) an open-book LLM judge confirms shares_method = true (a real shared mechanism, with a mapping).
These are proposed cross-field mechanism bridges for human evaluation — NOT verified novel discoveries.

Subcommands:
  extract   build + rank candidate hidden bridges from RS-25 data -> data/rs26_candidates.json;
            emit open-book transfer-card inputs for the top-M -> data/rs26_in/openbook/
  curate    read Claude transfer cards -> confirmed hidden bridges, ranked, with mechanism cards
"""
import argparse
import json
import os

from sme_lite import tfidf_vectors, cosine
import rs22_probe as R
import rs23_reduce as RS

HERE = os.path.dirname(os.path.abspath(__file__))
DATA = os.path.join(HERE, "data")
RS25_IN = os.path.join(DATA, "rs25_in", "retrieval")
RS25_OUT = os.path.join(DATA, "rs25_out", "retrieval")
IN_OB = os.path.join(DATA, "rs26_in", "openbook")
OUT_OB = os.path.join(DATA, "rs26_out", "openbook")

RANK_MAX = 3       # cascade ranks the candidate in its top-3
LEX_MAX = 0.06     # lexical cosine(query, candidate) below this = surface-invisible
M_CARDS = 40       # how many top candidates get a transfer card


def topcat(cat):
    c = str(cat).lower()
    return c.split(".")[0] if "." in c else c


def extract():
    all_ids, by_id, pairs = RS.load_mined(80)
    by_pair = {p["id"]: p for p in pairs}
    lex = tfidf_vectors({"papers": [by_id[a] for a in all_ids]})
    partner = {}
    for p in pairs:
        partner[p["side_a"]] = p["side_b"]

    cands = []
    for p in pairs:
        q = p["side_a"]
        meta = json.load(open(os.path.join(RS25_IN, f"{p['id']}.json")))["meta"]
        opath = os.path.join(RS25_OUT, f"{p['id']}.json")
        if not os.path.exists(opath):
            continue
        ranking = [R.normalize_id(x) for x in R._load_json_out(opath).get("ranking", [])]
        ranking = [c for c in ranking if c in set(meta["pool"])]     # valid union members only
        q_arch = topcat(by_id[q]["category"])
        for rank, c in enumerate(ranking[:RANK_MAX], start=1):
            if c == q or c == partner.get(q):                        # skip self + known partner
                continue
            if topcat(by_id[c]["category"]) == q_arch:               # must be cross-archive
                continue
            lx = cosine(lex[q], lex[c])
            if lx >= LEX_MAX:                                        # must be surface-invisible
                continue
            cands.append({
                "query_id": q, "query_title": by_id[q]["title"], "query_cat": by_id[q]["category"],
                "cand_id": c, "cand_title": by_id[c]["title"], "cand_cat": by_id[c]["category"],
                "cascade_rank": rank, "lexical_cos": round(lx, 4),
                "cross_archive": True, "src_pair": p["id"],
                "cand_is_some_partner": c in set(partner.values()),
            })
    # rank: cascade rank asc (1 best), then lexical asc (more hidden first)
    cands.sort(key=lambda d: (d["cascade_rank"], d["lexical_cos"], d["query_id"], d["cand_id"]))
    json.dump({"n": len(cands), "RANK_MAX": RANK_MAX, "LEX_MAX": LEX_MAX, "candidates": cands},
              open(os.path.join(DATA, "rs26_candidates.json"), "w"), indent=1, ensure_ascii=False)

    os.makedirs(IN_OB, exist_ok=True)
    top = cands[:M_CARDS]
    for i, d in enumerate(top):
        a, b = by_id[d["query_id"]], by_id[d["cand_id"]]
        inp = {"title_a": a["title"], "abstract_a": a["abstract"],
               "title_b": b["title"], "abstract_b": b["abstract"]}
        key = f"disc-{i:03d}"
        json.dump({"input": inp, "instr": "openbook", "key": key,
                   "pair": {"query_id": d["query_id"], "cand_id": d["cand_id"]}},
                  open(os.path.join(IN_OB, f"{key}.json"), "w"), indent=1, ensure_ascii=False)
    print(f"extract: {len(cands)} hidden-bridge candidates (cross-archive ∧ non-partner ∧ "
          f"cascade-top{RANK_MAX} ∧ lexical<{LEX_MAX}) -> data/rs26_candidates.json")
    print(f"  emitted top {len(top)} transfer-card inputs -> {os.path.relpath(IN_OB, HERE)}/")
    print(f"  n={M_CARDS} to dispatch (Claude open-book). Preview of the top 6:")
    for d in top[:6]:
        print(f"   [{d['cascade_rank']}|lex {d['lexical_cos']:.3f}] "
              f"{d['query_cat']:16} {d['query_title'][:44]!r}")
        print(f"        <-> {d['cand_cat']:16} {d['cand_title'][:44]!r}")


def curate():
    cand = json.load(open(os.path.join(DATA, "rs26_candidates.json")))["candidates"][:M_CARDS]
    confirmed, rejected = [], 0
    for i, d in enumerate(cand):
        key = f"disc-{i:03d}"
        p = os.path.join(OUT_OB, f"{key}.json")
        if not os.path.exists(p):
            continue
        card = R._load_json_out(p)
        rec = {**d, "shares_method": bool(card.get("shares_method")),
               "shared_mechanism": card.get("brief_justification", ""),
               "mapping": card.get("mapping", [])}
        if rec["shares_method"]:
            confirmed.append(rec)
        else:
            rejected += 1
    n_judged = len(confirmed) + rejected
    confirmed.sort(key=lambda d: (d["cascade_rank"], d["lexical_cos"]))
    out = {"experiment": "EXP-RS-26", "n_candidates_total": json.load(
               open(os.path.join(DATA, "rs26_candidates.json")))["n"],
           "n_carded": n_judged, "n_confirmed_shared_mechanism": len(confirmed),
           "hit_rate": round(len(confirmed) / n_judged, 3) if n_judged else None,
           "confirmed_bridges": confirmed}
    json.dump(out, open(os.path.join(DATA, "rs26_discoveries.json"), "w"), indent=1, ensure_ascii=False)
    print(f"=== EXP-RS-26 discovery — {len(confirmed)}/{n_judged} carded candidates confirmed as "
          f"shared-mechanism cross-field bridges (hit-rate {out['hit_rate']}) ===")
    for d in confirmed[:12]:
        print(f"\n• [{d['query_cat']}]  {d['query_title']}")
        print(f"  ↔ [{d['cand_cat']}]  {d['cand_title']}")
        print(f"  cascade-rank={d['cascade_rank']}  lexical-cos={d['lexical_cos']:.3f} (surface-invisible)")
        print(f"  shared mechanism: {d['shared_mechanism'][:300]}")
    print(f"\nwrote data/rs26_discoveries.json")
    return out


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    sub.add_parser("extract")
    sub.add_parser("curate")
    a = ap.parse_args()
    if a.cmd == "extract":
        extract()
    elif a.cmd == "curate":
        curate()


if __name__ == "__main__":
    main()
