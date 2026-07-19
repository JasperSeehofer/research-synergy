#!/usr/bin/env python3
"""Tier-validation: does a CHEAPER reduction model (Mistral, off the Claude window) preserve the E1
temporal-novelty signal? Mistral-reduce the 634 E1-pool papers with the FROZEN rs22_probe_mechanism
prompt, then re-run the E1 AUC on the Mistral reductions and compare to Opus.

  reduce   Mistral-reduce the pool (<=4 workers, idempotent) -> data/rs31_out_mistral/mechanism/
  score    E1 AUC on Mistral reductions vs Opus vs degree-null; agreement corr; adopt/keep verdict
"""
import argparse
import concurrent.futures as cf
import json
import math
import os
import time
import urllib.error
import urllib.request

import numpy as np

import rs31_temporal as R31
from embed_score import encode_st, MODEL_SPECS

HERE = os.path.dirname(os.path.abspath(__file__))
DATA = os.path.join(HERE, "data")
OUT_MISTRAL = os.path.join(DATA, "rs31_out_mistral", "mechanism")
PROMPT = os.path.join(HERE, "rs22_probe_mechanism.md")
MODEL = "mistral-large-latest"


def safe(a):
    return str(a).replace("/", "_")


def _mistral(prompt, blind, key, retries=6):
    body = json.dumps({"model": MODEL,
                       "messages": [{"role": "system", "content": prompt},
                                    {"role": "user", "content": json.dumps(blind, ensure_ascii=False)}],
                       "response_format": {"type": "json_object"}, "temperature": 0, "max_tokens": 1200}).encode()
    last = None
    for att in range(retries):
        try:
            req = urllib.request.Request("https://api.mistral.ai/v1/chat/completions", data=body,
                                         headers={"Authorization": f"Bearer {key}", "Content-Type": "application/json"})
            r = json.load(urllib.request.urlopen(req, timeout=120))
            return json.loads(r["choices"][0]["message"]["content"])
        except (urllib.error.HTTPError, urllib.error.URLError, KeyError, ValueError, TypeError) as e:
            last = e
            time.sleep(3 * (att + 1))
    raise RuntimeError(f"mistral failed after {retries}: {last}")


def reduce(workers=4):
    key = os.environ.get("MISTRAL_API_KEY")
    if not key:
        raise SystemExit("MISTRAL_API_KEY not set")
    st = json.load(open(os.path.join(DATA, "rs31_state.json")))
    pool, meta = st["pool"], st["meta"]
    prompt = open(PROMPT, encoding="utf-8").read()
    os.makedirs(OUT_MISTRAL, exist_ok=True)
    todo = [a for a in pool if not os.path.exists(os.path.join(OUT_MISTRAL, f"{safe(a)}.json"))]
    print(f"reduce(mistral): {len(pool)-len(todo)} cached, {len(todo)} to do (workers={workers})")

    def task(a):
        out = os.path.join(OUT_MISTRAL, f"{safe(a)}.json")
        if os.path.exists(out):
            return "cached"
        v = _mistral(prompt, {"title": meta[a]["title"], "abstract": meta[a]["abstract"]}, key)
        json.dump(v, open(out, "w"), indent=1, ensure_ascii=False)
        return "ok"

    done, errs = 0, []
    with cf.ThreadPoolExecutor(max_workers=workers) as ex:
        futs = {ex.submit(task, a): a for a in todo}
        for fut in cf.as_completed(futs):
            try:
                fut.result(); done += 1
                if done % 50 == 0 or done == len(todo):
                    print(f"  mistral {done}/{len(todo)}", flush=True)
            except Exception as e:  # noqa: BLE001
                errs.append((futs[fut], str(e)[:80]))
    open(os.path.join(DATA, "rs33_reduce.done"), "w").write(f"{done} ok, {len(errs)} errs\n")
    print(f"reduce(mistral): {done} ok, {len(errs)} errs -> {os.path.relpath(OUT_MISTRAL, HERE)}/")
    if errs:
        print("  errs:", errs[:5])


def _mistral_matrix(pool):
    reds = []
    for a in pool:
        p = os.path.join(OUT_MISTRAL, f"{safe(a)}.json")
        if os.path.exists(p):
            try:
                d = json.load(open(p)); d = d.get("output", d)
                reds.append(str(d.get("core_mechanism", "")).strip())
            except Exception:  # noqa: BLE001
                reds.append("")
        else:
            reds.append("")
    hf, _ = MODEL_SPECS["bge"]
    V, _, _, _ = encode_st(hf, [{"title": r, "abstract": ""} for r in reds], "bge")
    V = np.asarray(V, dtype=np.float32)
    V /= (np.linalg.norm(V, axis=1, keepdims=True) + 1e-9)
    return V, [bool(r) for r in reds]


def score():
    st = json.load(open(os.path.join(DATA, "rs31_state.json")))
    pool, meta = st["pool"], st["meta"]
    pos_set = {tuple(x) for x in st["positives"]}
    deg = json.load(open(os.path.join(DATA, "rs31_degree.json")))["degree"]
    Vmis, hm = _mistral_matrix(pool)
    Vop, ho = R31.reduction_matrix(pool)
    L = R31.lexical_matrix(pool, meta)
    idx = {a: i for i, a in enumerate(pool)}
    dvals = {a: deg.get(a) for a in pool}
    valid = [a for a in pool if hm[idx[a]] and ho[idx[a]] and dvals[a] is not None]
    med = float(np.median([dvals[a] for a in valid]))
    poor = {a for a in valid if dvals[a] < med}

    def topcat(c):
        c = str(c).lower(); return c.split(".")[0] if "." in c else c

    mis, op, pa, lab, ppm = [], [], [], [], []
    vi = valid
    for ii in range(len(vi)):
        a = vi[ii]
        for jj in range(ii + 1, len(vi)):
            b = vi[jj]
            if topcat(meta[a]["category"]) == topcat(meta[b]["category"]):
                continue
            if float(np.dot(L[idx[a]], L[idx[b]])) >= R31.LEX_MAX:
                continue
            mis.append(float(np.dot(Vmis[idx[a]], Vmis[idx[b]])))
            op.append(float(np.dot(Vop[idx[a]], Vop[idx[b]])))
            pa.append(math.log1p(dvals[a]) + math.log1p(dvals[b]))
            lab.append(tuple(sorted((a, b))) in pos_set)
            ppm.append((a in poor) and (b in poor))
    mis = np.array(mis); op = np.array(op); pa = np.array(pa)
    lab = np.array(lab, bool); ppm = np.array(ppm, bool)
    corr = float(np.corrcoef(mis, op)[0, 1])

    def strat(mask, name):
        return {"stratum": name, "n": int(mask.sum()), "n_pos": int(lab[mask].sum()),
                "auc_mistral": round(float(R31._auc(mis[mask], lab[mask])[0]), 4),
                "auc_opus": round(float(R31._auc(op[mask], lab[mask])[0]), 4),
                "auc_degree_null": round(float(R31._auc(pa[mask], lab[mask])[0]), 4)}

    overall = strat(np.ones(len(lab), bool), "overall")
    pp = strat(ppm, "poor_poor")
    aurm = pp["auc_mistral"]; degn = pp["auc_degree_null"]
    p_m = R31._mannwhitney_p(aurm, pp["n_pos"], pp["n"] - pp["n_pos"])
    # ADOPT iff Mistral still passes E1 on Poor-Poor (>=0.60, beats degree, p<0.05)
    adopt = (aurm >= R31.AUC_PASS and aurm > degn and p_m < 0.05)
    verdict = "ADOPT-MISTRAL" if adopt else "KEEP-OPUS"
    out = {"validation": "mistral-reduction-tier", "verdict": verdict,
           "pairwise_cos_corr_mistral_vs_opus": round(corr, 4),
           "overall": overall, "poor_poor": pp, "poor_poor_mistral_mwp": round(p_m, 5),
           "opus_reference_poor_poor_auc": 0.754}
    json.dump(out, open(os.path.join(DATA, "rs33_verdict.json"), "w"), indent=1, ensure_ascii=False)
    print(f"=== Tier-validation (Mistral reductions vs Opus) — VERDICT: {verdict} ===")
    print(f"  pairwise reduction-cosine correlation Mistral vs Opus: {corr:.3f}")
    for s in (overall, pp):
        print(f"  {s['stratum']:10} n={s['n']:6} pos={s['n_pos']:3} | "
              f"AUC(mistral)={s['auc_mistral']}  AUC(opus)={s['auc_opus']}  AUC(degree-null)={s['auc_degree_null']}")
    print(f"  poor-poor Mistral Mann-Whitney p={p_m:.5f}  (Opus ref PP AUC 0.754)")
    print(f"  -> {verdict}: " + ("Mistral preserves the E1 signal -> route O(N) reductions to Mistral (off Claude window)"
                                  if adopt else "Mistral degrades the signal -> keep Opus (or try Haiku)"))


def main():
    ap = argparse.ArgumentParser(); sub = ap.add_subparsers(dest="cmd", required=True)
    r = sub.add_parser("reduce"); r.add_argument("--workers", type=int, default=4)
    sub.add_parser("score")
    a = ap.parse_args()
    if a.cmd == "reduce":
        reduce(a.workers)
    else:
        score()


if __name__ == "__main__":
    main()
