#!/usr/bin/env python3
"""EXP-RS-25 (Phase 44) — raw ∪ reduction -> LLM-rerank cascade (the scalable LBD retriever).

Two cheap retrievers (raw-abstract bge + mechanism-reduction bge) each surface top-K; UNION the
candidate sets (dedup); the LLM re-ranks the small union (frozen rs22_retrieval_prompt.md). Evaluated
on the RS-23/24 mixed 80-pair cross-archive sample. Orbiter: Claude re-rank (overseer/primary) +
Mistral re-rank (executor cross-family) — the re-rank is a synthesis task, so Claude is authoritative
and Mistral is the descriptive comparison (W-SYN check).

Subcommands:
  emit         build union pools (top-K raw ∪ top-K reduction) + retrieval inputs -> data/rs25_in/retrieval/
  run-mistral  Mistral re-rank of each union (executor)   -> data/rs25_out_mistral/retrieval/
  score        Claude (+Mistral) re-rank recall@10/MRR vs union-ceiling / raw-alone / reduction-alone
"""
import argparse
import concurrent.futures as cf
import json
import os

from sme_lite import rank_candidates, eval_direction
import rs22_probe as R
import rs23_reduce as RS
from rs22_probe import mistral_call

HERE = os.path.dirname(os.path.abspath(__file__))
DATA = os.path.join(HERE, "data")
N = 80
K_EACH = 15                       # top-K from each retriever before the union
IN_DIR = os.path.join(DATA, "rs25_in", "retrieval")
OUT_CLAUDE = os.path.join(DATA, "rs25_out", "retrieval")
OUT_MIS = os.path.join(DATA, "rs25_out_mistral", "retrieval")
RET_PROMPT = os.path.join(HERE, "rs22_retrieval_prompt.md")


def ctx():
    all_ids, by_id, pairs = RS.load_mined(N)
    red = {a: R._load_json_out(os.path.join(DATA, "rs23_out_mined", "mechanism", f"{R.safe(a)}.json"))
           .get("core_mechanism", "").strip() for a in all_ids}
    red_s, _ = RS._emb_scorer(all_ids, [{"title": red[a], "abstract": ""} for a in all_ids])
    raw_s, _ = RS._emb_scorer(all_ids, [{"title": by_id[a]["title"], "abstract": by_id[a]["abstract"]}
                                        for a in all_ids])
    return all_ids, by_id, pairs, red_s, raw_s


def union_pool(q, all_ids, raw_s, red_s, k=K_EACH):
    raw_top = [c for c, _ in rank_candidates(q, all_ids, raw_s)[:k]]
    red_top = [c for c, _ in rank_candidates(q, all_ids, red_s)[:k]]
    seen, union = set(), []
    for c in raw_top + red_top:               # deterministic: raw first, then reduction, dedup
        if c not in seen:
            seen.add(c); union.append(c)
    return union


def emit():
    all_ids, by_id, pairs, red_s, raw_s = ctx()
    os.makedirs(IN_DIR, exist_ok=True)
    sizes = []
    for p in pairs:
        q = p["side_a"]
        pool = union_pool(q, all_ids, raw_s, red_s)
        sizes.append(len(pool))
        cands = [{"arxiv_id": c, "title": by_id[c]["title"], "abstract": by_id[c]["abstract"]}
                 for c in pool]
        inp = {"query_title": by_id[q]["title"], "query_abstract": by_id[q]["abstract"],
               "candidates": cands}
        json.dump({"input": inp, "meta": {"pool": pool, "target": p["side_b"],
                                          "target_in_union": p["side_b"] in pool},
                   "instr": "retrieval", "key": p["id"]},
                  open(os.path.join(IN_DIR, f"{p['id']}.json"), "w"), indent=1, ensure_ascii=False)
    ceil = sum(1 for p in pairs if p["side_b"] in union_pool(p["side_a"], all_ids, raw_s, red_s)) / len(pairs)
    print(f"emit: {len(pairs)} union retrieval inputs -> {os.path.relpath(IN_DIR, HERE)}/  "
          f"(K_each={K_EACH}, union size {min(sizes)}-{max(sizes)}, mean {sum(sizes)/len(sizes):.1f})")
    print(f"  union membership ceiling (side_b in union) = {ceil:.3f}")


def run_mistral(model, workers):
    key = os.environ.get("MISTRAL_API_KEY")
    if not key:
        raise SystemExit("MISTRAL_API_KEY not in env")
    prompt = open(RET_PROMPT, encoding="utf-8").read()
    os.makedirs(OUT_MIS, exist_ok=True)
    recs = [f for f in os.listdir(IN_DIR) if f.endswith(".json")]

    def task(f):
        opath = os.path.join(OUT_MIS, f)
        if os.path.exists(opath):
            return "cached"
        blind = json.load(open(os.path.join(IN_DIR, f)))["input"]
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
                    print(f"  mistral rerank {done}/{len(recs)}")
            except Exception as e:  # noqa: BLE001
                errs.append((futs[fut], str(e)[:120]))
    print(f"run-mistral[{model}]: {done - len(errs)} ok, {len(errs)} errs -> {OUT_MIS}")
    if errs:
        print("  errs:", errs[:5])


def _rerank_rank(pool, target, ranked_ids):
    """Rank of target after LLM re-rank of the union, with deterministic repair (missing appended)."""
    poolset = set(pool)
    seen, cleaned = set(), []
    for rid in ranked_ids:
        nid = R.normalize_id(rid)
        if nid in poolset and nid not in seen:
            cleaned.append(nid); seen.add(nid)
    full = cleaned + sorted(c for c in poolset if c not in seen)
    return full.index(target) + 1 if target in poolset else None   # None = target not in union (ceiling miss)


def _cascade_recall(pairs, metas, out_dir):
    ranks = {}
    for p in pairs:
        m = metas[p["id"]]
        path = os.path.join(out_dir, f"{p['id']}.json")
        if m["target"] not in m["pool"]:
            ranks[p["id"]] = None            # union ceiling miss (can't recover)
            continue
        if not os.path.exists(path):
            ranks[p["id"]] = "MISSING"; continue
        rr = R._load_json_out(path).get("ranking", [])
        ranks[p["id"]] = _rerank_rank(m["pool"], m["target"], rr)
    valid = {k: v for k, v in ranks.items() if v != "MISSING"}
    n = len(valid)
    r10 = sum(1 for v in valid.values() if v is not None and v <= 10) / n
    r5 = sum(1 for v in valid.values() if v is not None and v <= 5) / n
    mrr = sum(1.0 / v for v in valid.values() if v) / n
    missing = sum(1 for v in ranks.values() if v == "MISSING")
    return {"recall@10": r10, "recall@5": r5, "mrr": mrr, "n": n, "missing": missing, "ranks": valid}


def score():
    all_ids, by_id, pairs, red_s, raw_s = ctx()
    metas = {p["id"]: json.load(open(os.path.join(IN_DIR, f"{p['id']}.json")))["meta"] for p in pairs}
    ceil = sum(1 for p in pairs if metas[p["id"]]["target_in_union"]) / len(pairs)
    raw_only = eval_direction(pairs, all_ids, raw_s, "side_a", "side_b")["recall@10"]
    red_only = eval_direction(pairs, all_ids, red_s, "side_a", "side_b")["recall@10"]

    claude = _cascade_recall(pairs, metas, OUT_CLAUDE) if os.path.isdir(OUT_CLAUDE) else None
    mistral = _cascade_recall(pairs, metas, OUT_MIS) if os.path.isdir(OUT_MIS) else None

    out = {"experiment": "EXP-RS-25", "n_pairs": len(pairs), "K_each": K_EACH,
           "baselines": {"raw_only_R@10": raw_only, "reduction_only_R@10": red_only,
                         "union_membership_ceiling": ceil},
           "cascade_claude": claude, "cascade_mistral": mistral}
    json.dump(out, open(os.path.join(DATA, "rs25_results.json"), "w"), indent=1, ensure_ascii=False)
    print(f"=== EXP-RS-25 cascade (mixed 80-pair, K_each={K_EACH}) ===")
    print(f"baselines: raw-alone R@10={raw_only:.3f}  reduction-alone R@10={red_only:.3f}  "
          f"union-ceiling={ceil:.3f}")
    for name, r in (("CASCADE (Claude re-rank)", claude), ("cascade (Mistral re-rank)", mistral)):
        if r:
            print(f"{name}: R@10={r['recall@10']:.3f}  R@5={r['recall@5']:.3f}  MRR={r['mrr']:.3f}  "
                  f"(n={r['n']}, missing={r['missing']})")
    if claude:
        lift = claude["recall@10"] - raw_only
        print(f"\n==> cascade (Claude) vs raw-alone: {claude['recall@10']:.3f} vs {raw_only:.3f} "
              f"({'+' if lift>=0 else ''}{lift*100:.1f} pts)")
    print(f"wrote data/rs25_results.json")
    return out


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    sub.add_parser("emit")
    rm = sub.add_parser("run-mistral"); rm.add_argument("--model", default="mistral-large-latest")
    rm.add_argument("--workers", type=int, default=4)
    sub.add_parser("score")
    a = ap.parse_args()
    if a.cmd == "emit":
        emit()
    elif a.cmd == "run-mistral":
        run_mistral(a.model, a.workers)
    elif a.cmd == "score":
        score()


if __name__ == "__main__":
    main()
