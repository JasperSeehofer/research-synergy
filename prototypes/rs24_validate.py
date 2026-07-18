#!/usr/bin/env python3
"""EXP-RS-24 (Phase 43) — Orbiter-validated deep-analogy subset.

Orbiter loop (human standing directive): Mistral = EXECUTOR (bulk open-book method-sharing judgment on
all 80 mined-sample pairs); Claude = OVERSEER (decisive open-book on the surface-HARD deep tail +
mandatory audit of the Mistral overlap for over-pruning / W-SYN). Validity probe = the FROZEN
rs22_probe_openbook.md. Then re-score reduction_bge vs raw_bge vs lexical (RS-23 machinery) on the
VALIDATED-DEEP and VALIDATED-EASY splits to confirm the reduction is a deep-analogy specialist.

Subcommands:
  emit            openbook inputs for the 80 mined pairs -> data/rs24_in/openbook/<pair>.json
  run-mistral     Mistral open-book on all 80 (executor)  -> data/rs24_out_mistral/openbook/
  hard-keys       print the surface-HARD pair ids (raw-embedding rank of side_b > 10) for the Claude arm
  score           audit (Claude vs Mistral) + validated split + reduction-vs-raw recall
"""
import argparse
import concurrent.futures as cf
import json
import os

import numpy as np

from sme_lite import tfidf_vectors, cosine, rank_candidates, eval_direction
import rs22_probe as R
import rs23_reduce as RS
from rs22_probe import mistral_call

HERE = os.path.dirname(os.path.abspath(__file__))
DATA = os.path.join(HERE, "data")
N = 80
IN_OB = os.path.join(DATA, "rs24_in", "openbook")
OUT_MIS = os.path.join(DATA, "rs24_out_mistral", "openbook")
CLAUDE_OB = os.path.join(DATA, "rs24_out", "openbook")     # Claude arm (Workflow-written)
SLICE_OB = os.path.join(DATA, "rs22_out", "openbook")      # existing Claude openbook (15 slice pairs)
OB_PROMPT = os.path.join(HERE, "rs22_probe_openbook.md")


def ctx():
    all_ids, by_id, pairs = RS.load_mined(N)
    return all_ids, by_id, {p["id"]: p for p in pairs}, pairs


def emb_scorers(all_ids, by_id):
    red = {a: R._load_json_out(os.path.join(DATA, "rs23_out_mined", "mechanism", f"{R.safe(a)}.json"))
           .get("core_mechanism", "").strip() for a in all_ids}
    red_s, _ = RS._emb_scorer(all_ids, [{"title": red[a], "abstract": ""} for a in all_ids])
    raw_s, _ = RS._emb_scorer(all_ids, [{"title": by_id[a]["title"], "abstract": by_id[a]["abstract"]}
                                        for a in all_ids])
    corpus = {"papers": [by_id[a] for a in all_ids]}
    lex = tfidf_vectors(corpus)
    lex_s = lambda q, c: cosine(lex[q], lex[c])
    return red_s, raw_s, lex_s


def surface_hard(all_ids, by_id, pairs, raw_s):
    """pair_ids where side_b raw-embedding rank > 10 (surface retrieval fails = the deep tail)."""
    hard = []
    for p in pairs:
        ranked = [cid for cid, _ in rank_candidates(p["side_a"], all_ids, raw_s)]
        if ranked.index(p["side_b"]) + 1 > 10:
            hard.append(p["id"])
    return set(hard)


def emit():
    all_ids, by_id, pmap, pairs = ctx()
    os.makedirs(IN_OB, exist_ok=True)
    for p in pairs:
        a, b = by_id[p["side_a"]], by_id[p["side_b"]]
        inp = {"title_a": a["title"], "abstract_a": a["abstract"],
               "title_b": b["title"], "abstract_b": b["abstract"]}
        json.dump({"input": inp, "instr": "openbook", "key": p["id"]},
                  open(os.path.join(IN_OB, f"{p['id']}.json"), "w"), indent=1, ensure_ascii=False)
    print(f"emit: {len(pairs)} openbook inputs -> {os.path.relpath(IN_OB, HERE)}/")


def run_mistral(model, workers):
    key = os.environ.get("MISTRAL_API_KEY")
    if not key:
        raise SystemExit("MISTRAL_API_KEY not in env")
    prompt = open(OB_PROMPT, encoding="utf-8").read()
    os.makedirs(OUT_MIS, exist_ok=True)
    recs = [f for f in os.listdir(IN_OB) if f.endswith(".json")]

    def task(f):
        opath = os.path.join(OUT_MIS, f)
        if os.path.exists(opath):
            return "cached"
        blind = json.load(open(os.path.join(IN_OB, f)))["input"]
        v = mistral_call(prompt, blind, model, key)
        json.dump(v, open(opath, "w"), indent=1, ensure_ascii=False)
        return "ok"

    done, errs = 0, []
    with cf.ThreadPoolExecutor(max_workers=workers) as ex:
        futs = {ex.submit(task, f): f for f in recs}
        for fut in cf.as_completed(futs):
            try:
                fut.result(); done += 1
                if done % 20 == 0 or done == len(recs):
                    print(f"  mistral openbook {done}/{len(recs)}")
            except Exception as e:  # noqa: BLE001
                errs.append((futs[fut], str(e)[:120]))
    print(f"run-mistral[{model}]: {done - len(errs)} ok, {len(errs)} errs -> {OUT_MIS}")
    if errs:
        print("  errs:", errs[:5])


def hard_keys():
    all_ids, by_id, pmap, pairs = ctx()
    _, raw_s, _ = emb_scorers(all_ids, by_id)
    hard = sorted(surface_hard(all_ids, by_id, pairs, raw_s))
    print(f"surface-HARD pairs (n={len(hard)}):")
    print(json.dumps(hard))


def _shares(path):
    if not os.path.exists(path):
        return None
    d = R._load_json_out(path)
    return bool(d.get("shares_method")) if "shares_method" in d else None


def _claude_ob(pid):
    """Claude open-book verdict: prefer the RS-24 Claude arm, fall back to the RS-22 slice outputs."""
    for base in (CLAUDE_OB, SLICE_OB):
        v = _shares(os.path.join(base, f"{pid}.json"))
        if v is not None:
            return v
    return None


def kappa(pairs):
    n = len(pairs) or 1
    po = sum(1 for a, b in pairs if a == b) / n
    pa = sum(1 for a, _ in pairs if a) / n
    pb = sum(1 for _, b in pairs if b) / n
    pe = pa * pb + (1 - pa) * (1 - pb)
    return ((po - pe) / (1 - pe) if pe != 1 else 1.0), po


def score():
    all_ids, by_id, pmap, pairs = ctx()
    red_s, raw_s, lex_s = emb_scorers(all_ids, by_id)
    hard = surface_hard(all_ids, by_id, pairs, raw_s)

    mis = {p["id"]: _shares(os.path.join(OUT_MIS, f"{p['id']}.json")) for p in pairs}
    cla = {p["id"]: _claude_ob(p["id"]) for p in pairs}

    # --- AUDIT: Claude vs Mistral on the overlap ---
    overlap = [(cla[k], mis[k]) for k in mis if cla[k] is not None and mis[k] is not None]
    kap, agree = kappa(overlap) if overlap else (float("nan"), float("nan"))
    over_prune = sum(1 for k in mis if cla[k] is True and mis[k] is False)   # Claude keep, Mistral drop
    under_prune = sum(1 for k in mis if cla[k] is False and mis[k] is True)

    # --- validated set: deep = surface-hard ∧ Claude-shares (authoritative); fall back to Mistral ---
    def validated(pid):
        v = cla[pid] if cla[pid] is not None else mis[pid]
        return bool(v)
    deep = [p for p in pairs if p["id"] in hard and validated(p["id"])]
    easy = [p for p in pairs if p["id"] not in hard and validated(p["id"])]

    def rc(sub, s):
        m = eval_direction(sub, all_ids, s, "side_a", "side_b")
        return m["recall@10"], m["recall@5"], m["mrr"]

    out = {"experiment": "EXP-RS-24", "n_pairs": len(pairs), "n_surface_hard": len(hard),
           "audit": {"n_overlap": len(overlap), "cohens_kappa": kap, "raw_agreement": agree,
                     "mistral_over_prune(Claude_keep_Mistral_drop)": over_prune,
                     "mistral_under_prune": under_prune,
                     "n_mistral_validated": sum(1 for v in mis.values() if v),
                     "n_claude_validated_overlap": sum(1 for v in cla.values() if v)},
           "validated_deep": {"n": len(deep), "ids": [p["id"] for p in deep]},
           "validated_easy": {"n": len(easy)}}
    for name, sub in (("VALIDATED_DEEP", deep), ("VALIDATED_EASY", easy)):
        if sub:
            out[name] = {"n": len(sub),
                         "reduction_bge": rc(sub, red_s)[0], "raw_bge": rc(sub, raw_s)[0],
                         "lexical": rc(sub, lex_s)[0]}
    P1 = ("VALIDATED_DEEP" in out and out["VALIDATED_DEEP"]["reduction_bge"] > out["VALIDATED_DEEP"]["raw_bge"])
    P2 = ("VALIDATED_EASY" in out and out["VALIDATED_EASY"]["raw_bge"] >= out["VALIDATED_EASY"]["reduction_bge"])
    ndeep = out["validated_deep"]["n"]
    if P1 and ndeep >= 8:
        verdict = "CONFIRM — reduction is a validated deep-analogy specialist (n>=8); build the router/cascade"
    elif P1:
        verdict = f"WEAK — reduction>raw on validated-deep but n={ndeep}<8; expand the sample before building"
    else:
        verdict = "REFUTE — raw >= reduction even on validated-deep; Feynman win likely small-n; scaling unsolved"
    out["predictions"] = {"P1_reduction>raw_on_deep": P1, "P2_raw>=reduction_on_easy": P2}
    out["verdict"] = verdict
    json.dump(out, open(os.path.join(DATA, "rs24_results.json"), "w"), indent=1, ensure_ascii=False)

    print(f"=== EXP-RS-24 (orbiter-validated deep subset, n_pairs={len(pairs)}) ===")
    print(f"AUDIT (Claude vs Mistral open-book, overlap n={len(overlap)}): kappa={kap:.2f} agree={agree:.2f}"
          f" | Mistral over-prune (Claude keep/Mistral drop)={over_prune}  under-prune={under_prune}")
    print(f"  Mistral validated {out['audit']['n_mistral_validated']}/{len(pairs)}; "
          f"surface-hard n={len(hard)}")
    for name in ("VALIDATED_DEEP", "VALIDATED_EASY"):
        if name in out:
            d = out[name]
            print(f"{name} (n={d['n']}): reduction R@10={d['reduction_bge']:.2f}  "
                  f"raw R@10={d['raw_bge']:.2f}  lexical R@10={d['lexical']:.2f}")
    print(f"\nP1 reduction>raw on deep: {P1}  |  P2 raw>=reduction on easy: {P2}")
    print(f"==> {verdict}\nwrote data/rs24_results.json")
    return out


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    sub.add_parser("emit")
    rm = sub.add_parser("run-mistral"); rm.add_argument("--model", default="mistral-large-latest")
    rm.add_argument("--workers", type=int, default=4)
    sub.add_parser("hard-keys")
    sub.add_parser("score")
    a = ap.parse_args()
    if a.cmd == "emit":
        emit()
    elif a.cmd == "run-mistral":
        run_mistral(a.model, a.workers)
    elif a.cmd == "hard-keys":
        hard_keys()
    elif a.cmd == "score":
        score()


if __name__ == "__main__":
    main()
