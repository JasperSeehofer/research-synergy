#!/usr/bin/env python3
"""EXP-RS-20 — verify-backbone head-to-head: Mistral-large vs Sonnet-5 on a shared 50-input subset.

Purpose (human-directed, 2026-07-16): the pre-registered "Verify backbone: LLM vs LLM" ablation +
first orbiter/pi-migration fan-out. Both backbones run the SAME blind verify task (frozen
verify_prompt.md, C-38; C-39 input closure) on the SAME 50 (query,candidate) inputs, so verdicts
are directly comparable.

Shared 50 subset (deterministic): for each of the 5 Feynman forward queries, the top-9 HyDE-ranked
candidates UNION the true target (side_b) -> 10 per query -> 50. Always includes each true
cross-field analogue (measures each backbone's true-positive rate on the real bridges) plus the top
distractors (measures distractor-pruning / precision@10).

Backbones:
  mistral : direct Mistral API (mistral-large-latest), MISTRAL_API_KEY from env (the pi-authed key;
            orbiter/pi-migration provider family). Blind, temperature 0, JSON mode, parallel.
  sonnet  : run separately via a Workflow fan-out of 50 blind Sonnet-5 agents (emit-sonnet-args
            writes the tiny args manifest; agents Read verify_prompt.md + one input file each).

Subcommands:
  subset             write data/verify_subset_feynman.json (the shared 50).
  run-mistral        fan out the 50 over Mistral -> data/verify_outputs/feynman_mistral/<file>.
  emit-sonnet-args   write data/verify_sonnet_args.json (paths only) for the Sonnet Workflow.
  compare            load both backbones' verdicts, compute agreement/kappa/TP/pruning -> report.
"""
import argparse
import concurrent.futures as cf
import hashlib
import json
import os
import time
import urllib.error
import urllib.request

from verify_score import load_ctx, safe, DATA, HERE
from sme_lite import rank_candidates

IN_DIR = os.path.join(DATA, "verify_inputs", "feynman")
PROMPT = os.path.join(HERE, "verify_prompt.md")
PROMPT_SHA_FILE = os.path.join(HERE, "verify_prompt.sha256")
SUBSET_PATH = os.path.join(DATA, "verify_subset_feynman.json")
PAIR04 = "pair04-percolation-epidemics"


def frozen_prompt():
    """Read verify_prompt.md and enforce the C-38 freeze (must match verify_prompt.sha256)."""
    txt = open(PROMPT, encoding="utf-8").read()
    got = hashlib.sha256(txt.encode("utf-8")).hexdigest()
    want = open(PROMPT_SHA_FILE).read().strip()
    if got != want:
        raise SystemExit(f"C-38 violation: verify_prompt.md sha {got[:12]} != frozen {want[:12]}")
    return txt


# ----------------------------- shared 50 subset -----------------------------
def build_subset():
    corpus, all_ids, by_id, hyp, pairs, hyde_score, winning_idx = load_ctx("feynman")
    records = []
    for p in pairs:
        q, target, pid = p["side_a"], p["side_b"], p["id"]
        ranked = rank_candidates(q, all_ids, hyde_score)  # [(cid,score)] desc, C-19 tie-break
        rankmap = {cid: i + 1 for i, (cid, _) in enumerate(ranked)}
        top = [cid for cid, _ in ranked[:10]]
        if target not in top:                      # guarantee the true analogue is in the subset
            top = [cid for cid, _ in ranked[:9]] + [target]
        for cid in top:
            records.append({
                "file": f"{safe(q)}__{safe(cid)}.json", "pair_id": pid,
                "query_id": q, "candidate_id": cid, "is_target": (cid == target),
                "hyde_rank": rankmap[cid], "hyde_score": hyde_score(q, cid),
                "query_community": by_id[q].get("community_id"),
                "cand_community": by_id[cid].get("community_id"),
            })
    subset = {"experiment": "EXP-RS-20", "sub": "feynman", "n": len(records),
              "selection": "per query: top-9 HyDE-ranked UNION true target = 10 x 5 = 50",
              "prompt_sha256": open(PROMPT_SHA_FILE).read().strip(), "records": records}
    json.dump(subset, open(SUBSET_PATH, "w"), indent=1, ensure_ascii=False)
    print(f"subset: {len(records)} shared inputs -> {os.path.relpath(SUBSET_PATH, HERE)}")
    for p in pairs:
        rows = [r for r in records if r["query_id"] == p["side_a"]]
        tgt = next(r for r in rows if r["is_target"])
        print(f"  {p['id']:32} target rank={tgt['hyde_rank']:>2} "
              f"(subset ranks {min(r['hyde_rank'] for r in rows)}..{max(r['hyde_rank'] for r in rows)})")
    return subset


def load_subset():
    if not os.path.exists(SUBSET_PATH):
        return build_subset()
    return json.load(open(SUBSET_PATH))


# ----------------------------- Mistral backbone -----------------------------
def mistral_call(prompt, blind_input, model, key, retries=4):
    body = json.dumps({
        "model": model,
        "messages": [{"role": "system", "content": prompt},
                     {"role": "user", "content": json.dumps(blind_input, ensure_ascii=False)}],
        "response_format": {"type": "json_object"}, "temperature": 0, "max_tokens": 600,
    }).encode()
    last = None
    for attempt in range(retries):
        try:
            req = urllib.request.Request(
                "https://api.mistral.ai/v1/chat/completions", data=body,
                headers={"Authorization": f"Bearer {key}", "Content-Type": "application/json"})
            r = json.load(urllib.request.urlopen(req, timeout=90))
            content = r["choices"][0]["message"]["content"]
            d = json.loads(content)
            return {"method_coherence": bool(d["method_coherence"]),
                    "object_difference": bool(d["object_difference"]),
                    "rationale": str(d.get("rationale", "")), "backbone": model}
        except (urllib.error.HTTPError, urllib.error.URLError, KeyError, ValueError, TypeError) as e:
            last = e
            time.sleep(2 * (attempt + 1))  # backoff (also eases rate limits)
    raise RuntimeError(f"mistral_call failed after {retries}: {last}")


def run_mistral(model, workers, full=False):
    key = os.environ.get("MISTRAL_API_KEY")
    if not key:
        raise SystemExit("MISTRAL_API_KEY not in env")
    prompt = frozen_prompt()
    if full:  # full 175-input headline P1 gate (reuses the 50 subset files already written)
        manifest = json.load(open(os.path.join(DATA, "verify_manifest_feynman.json")))
        records = manifest["records"]
    else:
        records = load_subset()["records"]
    out_dir = os.path.join(DATA, "verify_outputs", "feynman_mistral")
    os.makedirs(out_dir, exist_ok=True)
    n_total = len(records)

    def task(rec):
        opath = os.path.join(out_dir, rec["file"])
        if os.path.exists(opath):
            return rec["file"], "cached"
        blind = json.load(open(os.path.join(IN_DIR, rec["file"])))["input"]
        v = mistral_call(prompt, blind, model, key)
        json.dump(v, open(opath, "w"), indent=1, ensure_ascii=False)
        return rec["file"], "ok"

    done, errs = 0, []
    with cf.ThreadPoolExecutor(max_workers=workers) as ex:
        futs = {ex.submit(task, r): r for r in records}
        for fut in cf.as_completed(futs):
            rec = futs[fut]
            try:
                _, st = fut.result()
                done += 1
                if done % 25 == 0 or done == n_total:
                    print(f"  mistral {done}/{n_total}")
            except Exception as e:  # noqa: BLE001 — report, don't abort the fleet
                errs.append((rec["file"], str(e)))
    meta = {"backbone": "mistral", "model": model, "n": n_total, "full": full, "errors": errs,
            "prompt_sha256": open(PROMPT_SHA_FILE).read().strip()}
    json.dump(meta, open(os.path.join(out_dir, "_meta.json"), "w"), indent=1, ensure_ascii=False)
    print(f"run-mistral[{model}]: {done - len(errs)} ok, {len(errs)} errors -> {out_dir}")
    if errs:
        print("  errors:", errs[:5])


# ----------------------------- Sonnet Workflow args -----------------------------
def emit_sonnet_args():
    frozen_prompt()  # enforce freeze before dispatch
    subset = load_subset()
    args = {"prompt_path": PROMPT, "input_dir": IN_DIR,
            "records": [{"file": r["file"], "path": os.path.join(IN_DIR, r["file"])}
                        for r in subset["records"]]}
    p = os.path.join(DATA, "verify_sonnet_args.json")
    json.dump(args, open(p, "w"), indent=1, ensure_ascii=False)
    print(f"emit-sonnet-args: {len(args['records'])} records -> {os.path.relpath(p, HERE)}")
    print("  pass this file's contents as the Workflow `args`; agents Read prompt_path + their path.")


# ----------------------------- compare -----------------------------
def load_backbone(name):
    d = os.path.join(DATA, "verify_outputs", f"feynman_{name}")
    out = {}
    if not os.path.isdir(d):
        return out
    for r in load_subset()["records"]:
        fp = os.path.join(d, r["file"])
        if os.path.exists(fp):
            out[r["file"]] = json.load(open(fp))
    return out


def kappa(pairs):
    """Cohen's kappa on a list of (a_bool, b_bool)."""
    n = len(pairs) or 1
    po = sum(1 for a, b in pairs if a == b) / n
    pa1 = sum(1 for a, _ in pairs if a) / n
    pb1 = sum(1 for _, b in pairs if b) / n
    pe = pa1 * pb1 + (1 - pa1) * (1 - pb1)
    return (po - pe) / (1 - pe) if pe != 1 else 1.0, po


def compare():
    subset = load_subset()
    son, mis = load_backbone("sonnet"), load_backbone("mistral")
    both = [r for r in subset["records"] if r["file"] in son and r["file"] in mis]
    missing = {"sonnet": [r["file"] for r in subset["records"] if r["file"] not in son],
               "mistral": [r["file"] for r in subset["records"] if r["file"] not in mis]}

    mc_pairs = [(son[r["file"]]["method_coherence"], mis[r["file"]]["method_coherence"]) for r in both]
    od_pairs = [(son[r["file"]]["object_difference"], mis[r["file"]]["object_difference"]) for r in both]
    mc_k, mc_po = kappa(mc_pairs)
    od_k, od_po = kappa(od_pairs)
    both_agree = sum(1 for r in both
                     if son[r["file"]]["method_coherence"] == mis[r["file"]]["method_coherence"]
                     and son[r["file"]]["object_difference"] == mis[r["file"]]["object_difference"])

    # true-positive on the 5 real analogues: does each backbone judge the TRUE target method-coherent?
    targets = [r for r in both if r["is_target"]]
    tp = {"sonnet": {}, "mistral": {}}
    for r in targets:
        tp["sonnet"][r["pair_id"]] = son[r["file"]]["method_coherence"]
        tp["mistral"][r["pair_id"]] = mis[r["file"]]["method_coherence"]

    # per-query pruning within the top-10 (precision proxy): how many judged incoherent (pruned)
    per_query = {}
    for r in both:
        d = per_query.setdefault(r["pair_id"], {"n": 0, "sonnet_pruned": 0, "mistral_pruned": 0,
                                                "sonnet_same_comm_pruned": 0, "mistral_same_comm_pruned": 0})
        d["n"] += 1
        same = r["cand_community"] is not None and r["cand_community"] == r["query_community"]
        if not son[r["file"]]["method_coherence"]:
            d["sonnet_pruned"] += 1
            d["sonnet_same_comm_pruned"] += int(same)
        if not mis[r["file"]]["method_coherence"]:
            d["mistral_pruned"] += 1
            d["mistral_same_comm_pruned"] += int(same)

    disagreements = []
    for r in both:
        s, m = son[r["file"]], mis[r["file"]]
        if s["method_coherence"] != m["method_coherence"] or s["object_difference"] != m["object_difference"]:
            disagreements.append({
                "pair_id": r["pair_id"], "candidate": r["candidate_id"], "hyde_rank": r["hyde_rank"],
                "is_target": r["is_target"],
                "sonnet": {"mc": s["method_coherence"], "od": s["object_difference"], "why": s.get("rationale", "")[:200]},
                "mistral": {"mc": m["method_coherence"], "od": m["object_difference"], "why": m.get("rationale", "")[:200]},
            })

    report = {
        "experiment": "EXP-RS-20", "comparison": "sonnet-5 vs mistral-large", "n_subset": subset["n"],
        "n_compared": len(both), "missing": {k: len(v) for k, v in missing.items()},
        "agreement": {
            "method_coherence": {"raw": mc_po, "cohens_kappa": mc_k,
                                 "sonnet_coherent": sum(1 for a, _ in mc_pairs if a),
                                 "mistral_coherent": sum(1 for _, b in mc_pairs if b)},
            "object_difference": {"raw": od_po, "cohens_kappa": od_k,
                                  "sonnet_diff": sum(1 for a, _ in od_pairs if a),
                                  "mistral_diff": sum(1 for _, b in od_pairs if b)},
            "both_fields_agree": both_agree, "both_fields_agree_rate": both_agree / (len(both) or 1),
        },
        "true_positive_on_real_analogues": tp,
        "per_query_pruning": per_query,
        "n_disagreements": len(disagreements), "disagreements": disagreements,
    }
    out = os.path.join(DATA, "verify_compare_feynman.json")
    json.dump(report, open(out, "w"), indent=1, ensure_ascii=False)

    # console summary
    print(f"compared {len(both)}/{subset['n']} (missing: sonnet={len(missing['sonnet'])} "
          f"mistral={len(missing['mistral'])})")
    print(f"\nmethod_coherence  agree={mc_po:.2f}  kappa={mc_k:.2f}  "
          f"(coherent: sonnet={report['agreement']['method_coherence']['sonnet_coherent']} "
          f"mistral={report['agreement']['method_coherence']['mistral_coherent']} of {len(both)})")
    print(f"object_difference agree={od_po:.2f}  kappa={od_k:.2f}")
    print(f"both fields agree  {both_agree}/{len(both)} ({100*both_agree/(len(both) or 1):.0f}%)")
    print("\ntrue analogue judged method-coherent (want TRUE for all 5):")
    for pid in sorted(tp["sonnet"]):
        print(f"  {pid:32} sonnet={tp['sonnet'][pid]!s:5} mistral={tp['mistral'][pid]!s:5}"
              + ("   <-- pair04 (GATE-B anchor)" if pid == PAIR04 else ""))
    print("\nper-query pruned within top-10 (incoherent -> pruned):")
    for pid, d in sorted(per_query.items()):
        print(f"  {pid:32} sonnet={d['sonnet_pruned']:>2} (same-field {d['sonnet_same_comm_pruned']})  "
              f"mistral={d['mistral_pruned']:>2} (same-field {d['mistral_same_comm_pruned']})  of {d['n']}")
    print(f"\ndisagreements: {len(disagreements)}/{len(both)}  (full list in {os.path.relpath(out, HERE)})")
    return report


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    sub.add_parser("subset")
    rm = sub.add_parser("run-mistral")
    rm.add_argument("--model", default="mistral-large-latest")
    rm.add_argument("--workers", type=int, default=8)
    rm.add_argument("--full", action="store_true", help="run all 175 manifest inputs (P1 gate)")
    sub.add_parser("emit-sonnet-args")
    sub.add_parser("compare")
    args = ap.parse_args()
    if args.cmd == "subset":
        build_subset()
    elif args.cmd == "run-mistral":
        run_mistral(args.model, args.workers, full=args.full)
    elif args.cmd == "emit-sonnet-args":
        emit_sonnet_args()
    elif args.cmd == "compare":
        compare()


if __name__ == "__main__":
    main()
