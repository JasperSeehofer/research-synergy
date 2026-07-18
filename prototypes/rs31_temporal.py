#!/usr/bin/env python3
"""EXP-RS-31 (E1, Phase 50) — Temporal-holdout, degree-controlled novelty benchmark.

Does the field-neutral REDUCTION cosine between two pre-T papers predict whether they get BRIDGED
after T, BEYOND node degree — specifically on the graph-distant Poor-Poor (both-obscure) stratum?
Substrate: the 420-pair mined benchmark (each pair has a timestamped bridge_paper). At T=2010: 634
pre-T pool papers, 165 future-bridge positives. See 50-PREREG.md for the pre-registered KILL/PASS.

Stages:
  build         parse years, build positives/pool/negatives + union reduction index -> data/rs31_state.json
  emit-reduce   reduction inputs for pool papers missing a reduction -> data/rs31_in/mechanism/  (Workflow-reduce)
  fetch-degree  Semantic Scholar citationCount per pool paper -> data/rs31_degree.json
  score         reduction_cos vs pa-null AUC per stratum + bootstrap dAUC -> pre-registered verdict
"""
import argparse
import glob
import json
import math
import os
import re
import time
import urllib.request

import numpy as np

from sme_lite import _WORD, _STOP
from embed_score import encode_st, MODEL_SPECS

HERE = os.path.dirname(os.path.abspath(__file__))
DATA = os.path.join(HERE, "data")
IN_MECH = os.path.join(DATA, "rs31_in", "mechanism")
MINED = os.path.join(DATA, "rs22_mined_pairs.json")
RED_DIRS = ["rs23_out/mechanism", "rs23_out_mined/mechanism", "rs27_out/mechanism",
            "rs30_out/mechanism", "rs31_out/mechanism"]

# ---- BLIND CONSTANTS (frozen; see 50-PREREG.md) ----
T = 2010
LEX_MAX = 0.06
BOOT = 2000
SEED = 31
N_MIN = 20              # min Poor-Poor positives for adequate power
AUC_PASS = 0.60
AUC_KILL = 0.55


def year(aid):
    if not aid:
        return None
    m = re.search(r"(\d{2})(\d{2})\.\d{4,5}", str(aid)) or re.search(r"/(\d{2})(\d{2})\d{3}", str(aid))
    if not m:
        return None
    yy = int(m.group(1))
    return (1900 + yy) if yy >= 91 else (2000 + yy)   # arXiv started 1991


def safe(a):
    return str(a).replace("/", "_")


def topcat(cat):
    c = str(cat).lower()
    return c.split(".")[0] if "." in c else c


def red_index():
    idx = {}
    for d in RED_DIRS:
        for f in glob.glob(os.path.join(DATA, d, "*.json")):
            idx.setdefault(os.path.basename(f)[:-5], f)
    return idx


def _load_red(path):
    try:
        raw = open(path, encoding="utf-8").read().strip()
        if raw.startswith("```"):
            raw = raw.strip("`")
        if "{" in raw:
            raw = raw[raw.find("{"):raw.rfind("}") + 1]
        d = json.loads(raw)
        d = d.get("output", d)
        return str(d.get("core_mechanism", "")).strip()
    except Exception:  # noqa: BLE001
        return ""


def build():
    pairs = json.load(open(MINED))["pairs"]
    meta = {}         # arxiv_id -> {title, abstract, category, year}
    for p in pairs:
        for s in ("side_a", "side_b"):
            sd = p[s]
            aid = sd["arxiv_id"]
            if aid not in meta:
                meta[aid] = {"title": sd["title"], "abstract": sd["abstract"],
                             "category": sd["category"], "year": year(aid)}
    pool = sorted(a for a, m in meta.items() if m["year"] is not None and m["year"] <= T)
    positives = []
    for p in pairs:
        a, b = p["side_a"]["arxiv_id"], p["side_b"]["arxiv_id"]
        bp = p.get("bridge_paper", {}).get("arxiv_id")
        ya, yb, yg = meta[a]["year"], meta[b]["year"], year(bp)
        if None in (ya, yb, yg):
            continue
        if max(ya, yb) <= T < yg:
            positives.append(sorted((a, b)))
    pos_set = {tuple(x) for x in positives}
    ridx = red_index()
    have = {a for a in pool if safe(a) in ridx}
    missing = [a for a in pool if a not in have]
    state = {"T": T, "pool": pool, "positives": [list(x) for x in sorted(pos_set)],
             "n_pool": len(pool), "n_positives": len(pos_set),
             "reduced": len(have), "missing_reductions": missing,
             "meta": {a: meta[a] for a in pool}}
    json.dump(state, open(os.path.join(DATA, "rs31_state.json"), "w"), indent=1, ensure_ascii=False)
    print(f"build: T={T} | pool={len(pool)} | positives={len(pos_set)} | "
          f"reduced={len(have)}/{len(pool)} | missing={len(missing)}")
    return state


def emit_reduce():
    st = json.load(open(os.path.join(DATA, "rs31_state.json")))
    os.makedirs(IN_MECH, exist_ok=True)
    for a in st["missing_reductions"]:
        m = st["meta"][a]
        json.dump({"input": {"title": m["title"], "abstract": m["abstract"]},
                   "instr": "mechanism", "key": a},
                  open(os.path.join(IN_MECH, f"{safe(a)}.json"), "w"), indent=1, ensure_ascii=False)
    print(f"emit-reduce: {len(st['missing_reductions'])} reduction inputs -> {os.path.relpath(IN_MECH, HERE)}/ "
          f"(reduce, then re-run build to refresh coverage)")


def fetch_degree():
    st = json.load(open(os.path.join(DATA, "rs31_state.json")))
    pool = st["pool"]
    key = os.environ.get("S2_API_KEY")
    hdr = {"Content-Type": "application/json"}
    if key:
        hdr["x-api-key"] = key
    deg = {}
    for i in range(0, len(pool), 400):
        chunk = pool[i:i + 400]
        body = json.dumps({"ids": [f"ARXIV:{re.sub(r'v[0-9]+$', '', a)}" for a in chunk]}).encode()
        url = "https://api.semanticscholar.org/graph/v1/paper/batch?fields=citationCount,year"
        for attempt in range(5):
            try:
                req = urllib.request.Request(url, data=body, headers=hdr)
                r = json.load(urllib.request.urlopen(req, timeout=120))
                for a, rec in zip(chunk, r):
                    deg[a] = (rec or {}).get("citationCount")
                break
            except Exception as e:  # noqa: BLE001
                print(f"  [batch {i} attempt {attempt}: {str(e)[:80]}]")
                time.sleep(4 * (attempt + 1))
        time.sleep(1.2)
        print(f"  degree {min(i + 400, len(pool))}/{len(pool)}")
    got = sum(1 for a in pool if deg.get(a) is not None)
    json.dump({"degree": deg, "n_pool": len(pool), "n_resolved": got},
              open(os.path.join(DATA, "rs31_degree.json"), "w"), indent=1, ensure_ascii=False)
    print(f"fetch-degree: resolved {got}/{len(pool)} citation counts -> data/rs31_degree.json")


# ---- lexical (tf-idf) dense matrix, L2-normed ----
def tok(t):
    return [w for w in _WORD.findall(str(t).lower()) if w not in _STOP and len(w) > 2]


def lexical_matrix(pool, meta):
    from collections import Counter
    docs = [Counter(tok(meta[a]["title"] + ". " + meta[a]["abstract"])) for a in pool]
    df = Counter()
    for c in docs:
        df.update(c.keys())
    vocab = {w: k for k, w in enumerate(sorted(df))}
    N = len(docs)
    idf = {w: math.log((N + 1) / (df[w] + 1)) + 1 for w in df}
    L = np.zeros((N, len(vocab)), dtype=np.float32)
    for i, c in enumerate(docs):
        for w, f in c.items():
            L[i, vocab[w]] = f * idf[w]
    L /= (np.linalg.norm(L, axis=1, keepdims=True) + 1e-9)
    return L


def reduction_matrix(pool):
    ridx = red_index()
    reds = [_load_red(ridx[safe(a)]) if safe(a) in ridx else "" for a in pool]
    hf, _ = MODEL_SPECS["bge"]
    V, _, _, _ = encode_st(hf, [{"title": r, "abstract": ""} for r in reds], "bge")
    V = np.asarray(V, dtype=np.float32)
    V /= (np.linalg.norm(V, axis=1, keepdims=True) + 1e-9)
    return V, [bool(r) for r in reds]


def _auc(scores, labels):
    """AUC = P(score_pos > score_neg) via ranks (ties=0.5). Returns (auc, n_pos, n_neg)."""
    s = np.asarray(scores, dtype=float)
    y = np.asarray(labels)
    npos, nneg = int(y.sum()), int((~y.astype(bool)).sum())
    if npos == 0 or nneg == 0:
        return float("nan"), npos, nneg
    order = np.argsort(s, kind="mergesort")
    ranks = np.empty(len(s))
    sr = s[order]
    i = 0
    while i < len(s):
        j = i
        while j + 1 < len(s) and sr[j + 1] == sr[i]:
            j += 1
        ranks[order[i:j + 1]] = (i + j) / 2.0 + 1.0
        i = j + 1
    auc = (ranks[y.astype(bool)].sum() - npos * (npos + 1) / 2.0) / (npos * nneg)
    return auc, npos, nneg


def _mannwhitney_p(auc, npos, nneg):
    if npos == 0 or nneg == 0 or math.isnan(auc):
        return float("nan")
    U = auc * npos * nneg
    mu = npos * nneg / 2.0
    sd = math.sqrt(npos * nneg * (npos + nneg + 1) / 12.0)
    z = (U - mu) / (sd + 1e-9)
    return 0.5 * math.erfc(z / math.sqrt(2))          # one-sided (auc>0.5)


def score():
    st = json.load(open(os.path.join(DATA, "rs31_state.json")))
    pool, meta = st["pool"], st["meta"]
    pos_set = {tuple(x) for x in st["positives"]}
    deg = json.load(open(os.path.join(DATA, "rs31_degree.json")))["degree"]
    V, has_red = reduction_matrix(pool)
    L = lexical_matrix(pool, meta)
    idx = {a: i for i, a in enumerate(pool)}
    Rcos = V @ V.T
    Lcos = L @ L.T
    # degree + median split (papers with a resolved degree)
    dvals = np.array([deg.get(a) if deg.get(a) is not None else np.nan for a in pool], dtype=float)
    med = np.nanmedian(dvals)
    poor = {i for i, a in enumerate(pool) if not np.isnan(dvals[i]) and dvals[i] < med}

    rows = []
    n = len(pool)
    for i in range(n):
        if not has_red[i] or np.isnan(dvals[i]):
            continue
        for j in range(i + 1, n):
            if not has_red[j] or np.isnan(dvals[j]):
                continue
            if topcat(meta[pool[i]]["category"]) == topcat(meta[pool[j]]["category"]):
                continue
            if Lcos[i, j] >= LEX_MAX:
                continue
            key = tuple(sorted((pool[i], pool[j])))
            lab = key in pos_set
            pa = math.log1p(dvals[i]) + math.log1p(dvals[j])
            pp = (i in poor) and (j in poor)
            rr = (i not in poor) and (j not in poor)
            rows.append((float(Rcos[i, j]), pa, lab, pp, rr))
    rows = np.array(rows, dtype=object)
    rcos = np.array([r[0] for r in rows], float)
    pa = np.array([r[1] for r in rows], float)
    lab = np.array([r[2] for r in rows], bool)
    ppm = np.array([r[3] for r in rows], bool)
    rrm = np.array([r[4] for r in rows], bool)

    def strat(mask, name):
        r, p, y = rcos[mask], pa[mask], lab[mask]
        a_r, npos, nneg = _auc(r, y)
        a_p, _, _ = _auc(p, y)
        pv = _mannwhitney_p(a_r, npos, nneg)
        return {"stratum": name, "n_pairs": int(mask.sum()), "n_pos": npos, "n_neg": nneg,
                "auc_reduction": None if math.isnan(a_r) else round(a_r, 4),
                "auc_pa_null": None if math.isnan(a_p) else round(a_p, 4),
                "dAUC": None if (math.isnan(a_r) or math.isnan(a_p)) else round(a_r - a_p, 4),
                "mannwhitney_p": None if math.isnan(pv) else round(pv, 5)}

    overall = strat(np.ones(len(lab), bool), "overall")
    pp = strat(ppm, "poor_poor")
    rr = strat(rrm, "rich_rich")

    # paired bootstrap dAUC on poor-poor
    rng = np.random.default_rng(SEED)
    dboot = []
    ridx_pp = np.where(ppm)[0]
    if pp["n_pos"] and pp["n_neg"]:
        rp, pp_, yp = rcos[ridx_pp], pa[ridx_pp], lab[ridx_pp]
        pos_i = np.where(yp)[0]; neg_i = np.where(~yp)[0]
        for _ in range(BOOT):
            bp = rng.choice(pos_i, len(pos_i)); bn = rng.choice(neg_i, len(neg_i))
            sel = np.concatenate([bp, bn]); yy = yp[sel]
            ar, _, _ = _auc(rp[sel], yy); apa, _, _ = _auc(pp_[sel], yy)
            if not (math.isnan(ar) or math.isnan(apa)):
                dboot.append(ar - apa)
    dboot = np.array(dboot)
    ci = [round(float(np.percentile(dboot, 2.5)), 4), round(float(np.percentile(dboot, 97.5)), 4)] if len(dboot) else [None, None]

    # verdict
    aur = pp["auc_reduction"]; dau = pp["dAUC"]; pv = pp["mannwhitney_p"]
    if overall["auc_reduction"] is not None and overall["auc_reduction"] <= 0.5:
        verdict = "INVALID"
    elif pp["n_pos"] < N_MIN:
        verdict = "WEAK-underpowered"
    elif aur is not None and aur >= AUC_PASS and pv is not None and pv < 0.05 and dau is not None and dau > 0 and ci[0] is not None and ci[0] > 0:
        verdict = "PASS"
    elif aur is not None and (aur <= AUC_KILL or (dau is not None and dau <= 0)):
        verdict = "KILL"
    else:
        verdict = "WEAK"

    out = {"experiment": "EXP-RS-31", "T": T, "verdict": verdict, "degree_median": float(med),
           "overall": overall, "poor_poor": pp, "rich_rich": rr, "poor_poor_dAUC_boot95CI": ci,
           "n_pool_scored": int(sum(1 for i in range(n) if has_red[i] and not np.isnan(dvals[i])))}
    json.dump(out, open(os.path.join(DATA, "rs31_verdict.json"), "w"), indent=1, ensure_ascii=False)
    print(f"=== EXP-RS-31 temporal novelty benchmark — VERDICT: {verdict} ===")
    for s in (overall, pp, rr):
        print(f"  {s['stratum']:10} n={s['n_pairs']:6} pos={s['n_pos']:4} | "
              f"AUC(reduction)={s['auc_reduction']} AUC(pa-null)={s['auc_pa_null']} dAUC={s['dAUC']} p={s['mannwhitney_p']}")
    print(f"  poor-poor dAUC bootstrap 95% CI: {ci}")
    print(f"  -> data/rs31_verdict.json")


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    for c in ("build", "emit-reduce", "fetch-degree", "score"):
        sub.add_parser(c)
    a = ap.parse_args()
    {"build": build, "emit-reduce": emit_reduce, "fetch-degree": fetch_degree, "score": score}[a.cmd]()


if __name__ == "__main__":
    main()
