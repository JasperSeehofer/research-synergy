#!/usr/bin/env python3
"""EXP-RS-32 (E3, Phase 51) — Method/Object asymmetric off-diagonal retrieval.

A cross-field analogy = SAME method, DIFFERENT object. Split each paper into method_atom + object_atom
(frozen rs32_methobj.md), rank on method-similarity under an object-distance gate, and test whether that
un-buries the "off-diagonal" transfers the symmetric whole-reduction cosine hides. See 51-PREREG.md.

Stages:
  emit-feynman  method/object inputs for the 10 Feynman deep-analogy endpoints  (then Workflow-reduce)
  gate          split-validation on the 5 Feynman pairs -> PASS/KILL precondition
  emit-pool     method/object inputs for the 634 E1 pool papers                 (then Workflow-reduce)
  score         AUC(method) vs AUC(symmetric) on the E1 benchmark, off-diagonal lift -> verdict
"""
import argparse
import glob
import json
import math
import os

import numpy as np

from embed_score import encode_st, MODEL_SPECS
import rs31_temporal as R31

HERE = os.path.dirname(os.path.abspath(__file__))
DATA = os.path.join(HERE, "data")
IN_FEYN = os.path.join(DATA, "rs32_in", "feynman")
OUT_FEYN = os.path.join(DATA, "rs32_out", "feynman")
IN_POOL = os.path.join(DATA, "rs32_in", "pool")
OUT_POOL = os.path.join(DATA, "rs32_out", "pool")

# ---- BLIND CONSTANTS (frozen; see 51-PREREG.md) ----
TAU = 0.5              # object-distance gate: object cosine < TAU = "different object"
SEP_GATE = 3           # >=3/5 Feynman pairs need method-sim > object-sim
NONDEGEN = 0.85        # mean within-paper cos(method,object) must be < this
EPS = 0.02
SEED = 32
BOOT = 2000
OFFDIAG_MIN = 10


def safe(a):
    return str(a).replace("/", "_")


def _feyn_endpoints():
    f = json.load(open(os.path.join(DATA, "feynman_10pair_papers.json")))
    corp = {x["arxiv_id"]: x for x in json.load(open(os.path.join(DATA, "mvp_corpus.json")))["papers"]}
    ev = set(f["evaluable_pairs"])
    pairs = []
    for p in f["pairs"]:
        if p["id"].split("-")[0] in ev:
            a, b = p["side_a"]["arxiv_id"], p["side_b"]["arxiv_id"]
            if a in corp and b in corp:
                pairs.append((p["id"], a, b))
    return pairs, corp


def emit_feynman():
    pairs, corp = _feyn_endpoints()
    os.makedirs(IN_FEYN, exist_ok=True)
    ids = sorted({x for _, a, b in pairs for x in (a, b)})
    for aid in ids:
        m = corp[aid]
        json.dump({"input": {"title": m["title"], "abstract": m["abstract"]}, "instr": "methobj", "key": aid},
                  open(os.path.join(IN_FEYN, f"{safe(aid)}.json"), "w"), indent=1, ensure_ascii=False)
    print(f"emit-feynman: {len(ids)} method/object inputs ({len(pairs)} pairs) -> {os.path.relpath(IN_FEYN, HERE)}/")


def _load_mo(path):
    raw = open(path, encoding="utf-8").read().strip()
    if raw.startswith("```"):
        raw = raw.strip("`")
    if "{" in raw:
        raw = raw[raw.find("{"):raw.rfind("}") + 1]
    d = json.loads(raw)
    d = d.get("output", d)
    return str(d.get("method_atom", "")).strip(), str(d.get("object_atom", "")).strip()


def _embed(texts):
    hf, _ = MODEL_SPECS["bge"]
    V, _, _, _ = encode_st(hf, [{"title": t, "abstract": ""} for t in texts], "bge")
    V = np.asarray(V, dtype=np.float32)
    return V / (np.linalg.norm(V, axis=1, keepdims=True) + 1e-9)


def gate():
    pairs, _ = _feyn_endpoints()
    ids = sorted({x for _, a, b in pairs for x in (a, b)})
    mo = {}
    for aid in ids:
        p = os.path.join(OUT_FEYN, f"{safe(aid)}.json")
        if not os.path.exists(p):
            raise SystemExit(f"missing method/object output for {aid} — reduce first")
        mo[aid] = _load_mo(p)
    methods = _embed([mo[a][0] for a in ids])
    objects = _embed([mo[a][1] for a in ids])
    idx = {a: i for i, a in enumerate(ids)}
    within = float(np.mean([float(np.dot(methods[idx[a]], objects[idx[a]])) for a in ids]))
    rows, sep = [], 0
    for pid, a, b in pairs:
        ms = float(np.dot(methods[idx[a]], methods[idx[b]]))
        os_ = float(np.dot(objects[idx[a]], objects[idx[b]]))
        rows.append({"pair": pid, "method_sim": round(ms, 3), "object_sim": round(os_, 3),
                     "method_gt_object": ms > os_})
        sep += int(ms > os_)
    ok_sep = sep >= SEP_GATE
    ok_deg = within < NONDEGEN
    verdict = "GATE-PASS" if (ok_sep and ok_deg) else "KILL"
    out = {"experiment": "EXP-RS-32", "stage": "feynman-gate", "verdict": verdict,
           "separability": f"{sep}/5 (need >={SEP_GATE})", "sep_ok": ok_sep,
           "within_paper_method_object_cos_mean": round(within, 3), "nondegen_ok": ok_deg,
           "pairs": rows}
    json.dump(out, open(os.path.join(DATA, "rs32_gate.json"), "w"), indent=1, ensure_ascii=False)
    print(f"=== EXP-RS-32 Feynman split-validation gate — {verdict} ===")
    for r in rows:
        print(f"  {r['pair']:26} method_sim {r['method_sim']:.3f}  object_sim {r['object_sim']:.3f}  "
              f"method>object={r['method_gt_object']}")
    print(f"  separability {sep}/5 (need >={SEP_GATE}) -> {ok_sep};  "
          f"within-paper method↔object mean {within:.3f} (<{NONDEGEN}) -> {ok_deg}")
    print(f"  VERDICT: {verdict}" + ("" if verdict == "GATE-PASS" else "  -> KILL E3, move to E2"))


def emit_pool():
    st = json.load(open(os.path.join(DATA, "rs31_state.json")))
    os.makedirs(IN_POOL, exist_ok=True)
    for a in st["pool"]:
        m = st["meta"][a]
        json.dump({"input": {"title": m["title"], "abstract": m["abstract"]}, "instr": "methobj", "key": a},
                  open(os.path.join(IN_POOL, f"{safe(a)}.json"), "w"), indent=1, ensure_ascii=False)
    print(f"emit-pool: {len(st['pool'])} method/object inputs -> {os.path.relpath(IN_POOL, HERE)}/")


def _auc(scores, labels):
    s = np.asarray(scores, float); y = np.asarray(labels, bool)
    npos, nneg = int(y.sum()), int((~y).sum())
    if npos == 0 or nneg == 0:
        return float("nan")
    order = np.argsort(s, kind="mergesort"); ranks = np.empty(len(s)); sr = s[order]; i = 0
    while i < len(s):
        j = i
        while j + 1 < len(s) and sr[j + 1] == sr[i]:
            j += 1
        ranks[order[i:j + 1]] = (i + j) / 2.0 + 1.0; i = j + 1
    return (ranks[y].sum() - npos * (npos + 1) / 2.0) / (npos * nneg)


def score():
    st = json.load(open(os.path.join(DATA, "rs31_state.json")))
    pool, meta = st["pool"], st["meta"]
    pos_set = {tuple(x) for x in st["positives"]}
    deg = json.load(open(os.path.join(DATA, "rs31_degree.json")))["degree"]
    # method/object reductions for the pool
    mo = {}
    for a in pool:
        p = os.path.join(OUT_POOL, f"{safe(a)}.json")
        mo[a] = _load_mo(p) if os.path.exists(p) else ("", "")
    have = [a for a in pool if mo[a][0] and mo[a][1] and deg.get(a) is not None]
    M = _embed([mo[a][0] for a in have]); O = _embed([mo[a][1] for a in have])
    # symmetric reduction matrix (reuse rs31)
    Vred, has_red = R31.reduction_matrix(pool)
    ri = {a: i for i, a in enumerate(pool)}
    hi = {a: i for i, a in enumerate(have)}
    dvals = {a: deg.get(a) for a in have}
    med = float(np.median([dvals[a] for a in have]))
    poor = {a for a in have if dvals[a] < med}
    Lset = set(have)

    # lexical gate reuse: recompute lexical among pool (small)
    L = R31.lexical_matrix(pool, meta); pli = {a: i for i, a in enumerate(pool)}

    def topcat(c):
        c = str(c).lower(); return c.split(".")[0] if "." in c else c

    rows = []
    hv = have
    for ii in range(len(hv)):
        a = hv[ii]
        for jj in range(ii + 1, len(hv)):
            b = hv[jj]
            if topcat(meta[a]["category"]) == topcat(meta[b]["category"]):
                continue
            if float(np.dot(L[pli[a]], L[pli[b]])) >= R31.LEX_MAX:
                continue
            lab = tuple(sorted((a, b))) in pos_set
            msim = float(np.dot(M[hi[a]], M[hi[b]]))
            osim = float(np.dot(O[hi[a]], O[hi[b]]))
            sym = float(np.dot(Vred[ri[a]], Vred[ri[b]])) if (has_red[ri[a]] and has_red[ri[b]]) else float("nan")
            pp = (a in poor) and (b in poor)
            rows.append((msim, osim, sym, lab, pp))
    msim = np.array([r[0] for r in rows]); osim = np.array([r[1] for r in rows])
    sym = np.array([r[2] for r in rows]); lab = np.array([r[3] for r in rows], bool)
    ppm = np.array([r[4] for r in rows], bool)
    valid = ~np.isnan(sym)

    def strat(mask, name):
        m = mask & valid
        return {"stratum": name, "n": int(m.sum()), "n_pos": int(lab[m].sum()),
                "auc_method": round(float(_auc(msim[m], lab[m])), 4),
                "auc_symmetric": round(float(_auc(sym[m], lab[m])), 4)}

    overall = strat(np.ones(len(lab), bool), "overall")
    pp = strat(ppm, "poor_poor")
    # off-diagonal (object-distant) positives on poor-poor
    offdiag = ppm & valid & (osim < TAU)
    od = {"n": int(offdiag.sum()), "n_pos": int(lab[offdiag].sum()),
          "auc_method": round(float(_auc(msim[offdiag], lab[offdiag])), 4) if offdiag.sum() else None,
          "auc_symmetric": round(float(_auc(sym[offdiag], lab[offdiag])), 4) if offdiag.sum() else None}
    dauc_off = (od["auc_method"] - od["auc_symmetric"]) if (od["auc_method"] is not None) else None
    # bootstrap dAUC_offdiag
    rng = np.random.default_rng(SEED); dboot = []
    oi = np.where(offdiag)[0]; y = lab[oi]; pos_i = np.where(y)[0]; neg_i = np.where(~y)[0]
    if len(pos_i) and len(neg_i):
        for _ in range(BOOT):
            sel = np.concatenate([rng.choice(pos_i, len(pos_i)), rng.choice(neg_i, len(neg_i))])
            am = _auc(msim[oi][sel], y[sel]); asy = _auc(sym[oi][sel], y[sel])
            if not (math.isnan(am) or math.isnan(asy)):
                dboot.append(am - asy)
    ci = [round(float(np.percentile(dboot, 2.5)), 4), round(float(np.percentile(dboot, 97.5)), 4)] if dboot else [None, None]

    if od["n_pos"] < OFFDIAG_MIN:
        verdict = "WEAK-underpowered"
    elif pp["auc_method"] >= pp["auc_symmetric"] - EPS and dauc_off is not None and dauc_off > 0 and ci[0] is not None and ci[0] > 0:
        verdict = "PASS"
    elif pp["auc_method"] < pp["auc_symmetric"] - EPS or (dauc_off is not None and dauc_off <= 0):
        verdict = "KILL"
    else:
        verdict = "WEAK"
    out = {"experiment": "EXP-RS-32", "stage": "e1-graded", "verdict": verdict,
           "overall": overall, "poor_poor": pp, "offdiag_poor_poor": od,
           "dAUC_offdiag": dauc_off, "dAUC_offdiag_boot95CI": ci, "tau": TAU}
    json.dump(out, open(os.path.join(DATA, "rs32_verdict.json"), "w"), indent=1, ensure_ascii=False)
    print(f"=== EXP-RS-32 method/object graded on E1 — VERDICT: {verdict} ===")
    for s in (overall, pp):
        print(f"  {s['stratum']:10} n={s['n']:6} pos={s['n_pos']:3} AUC(method)={s['auc_method']} AUC(symmetric)={s['auc_symmetric']}")
    print(f"  off-diag PP (object<{TAU}): n={od['n']} pos={od['n_pos']} AUC(method)={od['auc_method']} AUC(sym)={od['auc_symmetric']} dAUC={dauc_off} CI={ci}")


def main():
    ap = argparse.ArgumentParser(); sub = ap.add_subparsers(dest="cmd", required=True)
    for c in ("emit-feynman", "gate", "emit-pool", "score"):
        sub.add_parser(c)
    a = ap.parse_args()
    {"emit-feynman": emit_feynman, "gate": gate, "emit-pool": emit_pool, "score": score}[a.cmd]()


if __name__ == "__main__":
    main()
