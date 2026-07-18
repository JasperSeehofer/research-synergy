#!/usr/bin/env python3
"""EXP-RS-28 (Phase 47) — CALIBRATION CONTROL for the RS-26/27 discovery pipeline.

Question: is the discovery pipeline's ~12.5% end-to-end genuine-bridge yield (RS-27:
40 carded -> 6 card-confirmed -> 5 adjudicated-genuine) REAL signal from the mechanism-
reduction retrieval, or would the SAME open-book-card + skeptical-adjudicator stack confirm
shared mechanisms at a similar rate on ANY surface-disjoint cross-field pair (LLM
permissiveness / confabulation)?

Control = the IDENTICAL downstream pipeline on N_CTRL RANDOM pairs drawn from the SAME
rs27 corpus that are cross-archive AND lexical<0.06 (surface-invisible, same gate as
discovery) but were NOT selected by the reduction retrieval (not in rs27 candidate set).
Only the pair-SELECTION differs (random vs reduction-top-3); card + adjudicator prompts are
byte-identical (frozen rs22_probe_openbook.md; reconstructed rs28_adjudicate.md applied to
BOTH arms). Treatment's 6 confirmed are re-adjudicated with the same reconstructed
adjudicator (harness-validity reproduction check: must recover >=4/6).

Blind constants are frozen in 47-PREREG.md (SHA recorded there). See that file for the
pre-registered PASS/FAIL/WEAK decision.

Flow:
  sample          build N_CTRL control pairs (seed) -> emit open-book card inputs
  emit-card-args  Workflow args (blind Opus card fan-out) -> data/rs28_args_openbook.json
  curate          read card outputs -> confirmed -> data/rs28_adjudicate_input.json
  emit-adj-args   Workflow args for adjudication (control + treatment re-adjudication)
  score           rates + Fisher exact + enrichment -> pre-registered verdict
"""
import argparse
import json
import math
import os
import random

from sme_lite import tfidf_vectors, cosine

HERE = os.path.dirname(os.path.abspath(__file__))
DATA = os.path.join(HERE, "data")
IN_OB = os.path.join(DATA, "rs28_in", "openbook")
OUT_OB = os.path.join(DATA, "rs28_out", "openbook")
OUT_ADJ = os.path.join(DATA, "rs28_out", "adj")
CORPUS = os.path.join(DATA, "rs27_corpus.json")
CANDS = os.path.join(DATA, "rs27_candidates.json")
OPENBOOK_PROMPT = os.path.join(HERE, "rs22_probe_openbook.md")
ADJ_PROMPT = os.path.join(HERE, "rs28_adjudicate.md")

# ---- BLIND CONSTANTS (frozen; see 47-PREREG.md) ----
N_CTRL = 80          # control pairs (2x the treatment's 40 carded, for power)
SEED = 27            # deterministic shuffle seed
LEX_MAX = 0.06       # same surface-invisible gate as discovery
# Treatment reference (fixed, from rs27_discoveries.json + rs27_adjudication.json):
T_CARDED, T_CARD_CONFIRMED, T_GENUINE = 40, 6, 5


def topcat(cat):
    c = str(cat).lower()
    return c.split(".")[0] if "." in c else c


def _pairkey(a, b):
    return tuple(sorted((a, b)))


def sample():
    papers = json.load(open(CORPUS))["papers"]
    by_id = {p["arxiv_id"]: p for p in papers}
    ids = [p["arxiv_id"] for p in papers]
    # exclusion set = every reduction-selected discovery candidate pair (treatment)
    cand = json.load(open(CANDS))["candidates"]
    excluded = {_pairkey(c["query_id"], c["cand_id"]) for c in cand}
    lex = tfidf_vectors({"papers": papers})
    # enumerate all cross-archive, surface-invisible, NON-candidate pairs
    pool = []
    for i in range(len(ids)):
        for j in range(i + 1, len(ids)):
            a, b = ids[i], ids[j]
            if topcat(by_id[a]["category"]) == topcat(by_id[b]["category"]):
                continue
            key = _pairkey(a, b)
            if key in excluded:
                continue
            lx = cosine(lex[a], lex[b])
            if lx >= LEX_MAX:
                continue
            pool.append((a, b, round(float(lx), 4)))
    pool.sort(key=lambda t: (t[0], t[1]))          # deterministic base order
    rng = random.Random(SEED)
    rng.shuffle(pool)
    chosen = pool[:N_CTRL]
    os.makedirs(IN_OB, exist_ok=True)
    ctrl = []
    for k, (a, b, lx) in enumerate(chosen):
        pa, pb = by_id[a], by_id[b]
        inp = {"title_a": pa["title"], "abstract_a": pa["abstract"],
               "title_b": pb["title"], "abstract_b": pb["abstract"]}
        key = f"ctrl-{k:03d}"
        json.dump({"input": inp, "instr": "openbook", "key": key,
                   "pair": {"a_id": a, "b_id": b, "a_cat": pa["category"], "b_cat": pb["category"],
                            "lexical_cos": lx}},
                  open(os.path.join(IN_OB, f"{key}.json"), "w"), indent=1, ensure_ascii=False)
        ctrl.append({"key": key, "a_id": a, "b_id": b, "a_cat": pa["category"],
                     "b_cat": pb["category"], "a_title": pa["title"], "b_title": pb["title"],
                     "lexical_cos": lx})
    lexvals = [c["lexical_cos"] for c in ctrl]
    meta = {"n_pool": len(pool), "n_ctrl": len(ctrl), "seed": SEED, "lex_max": LEX_MAX,
            "n_excluded_candidate_pairs": len(excluded),
            "ctrl_lex_min": min(lexvals), "ctrl_lex_median": sorted(lexvals)[len(lexvals) // 2],
            "ctrl_lex_max": max(lexvals)}
    json.dump({"meta": meta, "pairs": ctrl},
              open(os.path.join(DATA, "rs28_control_pairs.json"), "w"), indent=1, ensure_ascii=False)
    print(f"sample: {len(pool)} eligible non-candidate cross-field surface-invisible pairs; "
          f"drew {len(ctrl)} (seed {SEED}) -> data/rs28_control_pairs.json")
    print(f"  control lexical_cos: min {meta['ctrl_lex_min']} median {meta['ctrl_lex_median']} "
          f"max {meta['ctrl_lex_max']}  (treatment top-40 were the extreme-low tail ~0.003-0.02,")
    print(f"  so this control is if anything biased TOWARD confirming -> conservative for 'signal real')")


def emit_card_args():
    ctrl = json.load(open(os.path.join(DATA, "rs28_control_pairs.json")))["pairs"]
    prompt_text = open(OPENBOOK_PROMPT, encoding="utf-8").read()
    os.makedirs(OUT_OB, exist_ok=True)
    records = [{"key": c["key"],
                "input_path": os.path.join(IN_OB, f"{c['key']}.json"),
                "output_path": os.path.join(OUT_OB, f"{c['key']}.json")} for c in ctrl]
    args = {"stage": "openbook", "prompt_text": prompt_text, "records": records}
    p = os.path.join(DATA, "rs28_args_openbook.json")
    json.dump(args, open(p, "w"), indent=1, ensure_ascii=False)
    print(f"emit-card-args: {len(records)} blind card calls -> {os.path.relpath(p, HERE)}")
    print(f"  (Workflow: each agent Reads .input from input_path, applies the frozen openbook "
          f"prompt, Writes JSON to output_path)")


def _load_out(path):
    raw = open(path, encoding="utf-8").read().strip()
    if raw.startswith("```"):
        raw = raw.strip("`")
    if "{" in raw:
        raw = raw[raw.find("{"):raw.rfind("}") + 1]
    d = json.loads(raw)
    return d.get("output", d) if isinstance(d, dict) else d


def curate():
    ctrl = json.load(open(os.path.join(DATA, "rs28_control_pairs.json")))["pairs"]
    papers = {p["arxiv_id"]: p for p in json.load(open(CORPUS))["papers"]}
    confirmed, carded, missing = [], 0, []
    for c in ctrl:
        p = os.path.join(OUT_OB, f"{c['key']}.json")
        if not os.path.exists(p):
            missing.append(c["key"]); continue
        carded += 1
        card = _load_out(p)
        if bool(card.get("shares_method")):
            confirmed.append({**c, "shared_mechanism": card.get("brief_justification", "")})
    if missing:
        print(f"  WARNING: {len(missing)} card outputs missing (e.g. {missing[:4]})")
    # adjudication input in the SAME schema RS-27 used (proposed_shared_mechanism included)
    adj = [{"id": f"C{i}", "field_a": c["a_cat"], "title_a": c["a_title"],
            "abstract_a": papers[c["a_id"]]["abstract"], "field_b": c["b_cat"], "title_b": c["b_title"],
            "abstract_b": papers[c["b_id"]]["abstract"], "proposed_shared_mechanism": c["shared_mechanism"],
            "lexical_cos": c["lexical_cos"]} for i, c in enumerate(confirmed)]
    json.dump({"bridges": adj}, open(os.path.join(DATA, "rs28_adjudicate_input.json"), "w"),
              indent=1, ensure_ascii=False)
    json.dump({"n_carded": carded, "n_card_confirmed": len(confirmed),
               "card_confirm_rate": round(len(confirmed) / carded, 4) if carded else None,
               "confirmed": confirmed},
              open(os.path.join(DATA, "rs28_card_confirmed.json"), "w"), indent=1, ensure_ascii=False)
    print(f"curate: {carded} carded, {len(confirmed)} card-confirmed (shares_method) "
          f"= rate {len(confirmed)/carded if carded else 0:.4f}  "
          f"(treatment {T_CARD_CONFIRMED}/{T_CARDED}={T_CARD_CONFIRMED/T_CARDED:.3f})")
    print(f"  -> data/rs28_adjudicate_input.json ({len(adj)} to blind-adjudicate)")
    for c in confirmed:
        print(f"    [{c['a_cat']} x {c['b_cat']}] lex {c['lexical_cos']}: {c['shared_mechanism'][:120]}")


def emit_adj_args():
    """Adjudicate BOTH arms with the reconstructed skeptical adjudicator (rs28_adjudicate.md):
    (1) control card-confirmed, (2) treatment's 6 (rs27_adjudicate_input.json) = reproduction check."""
    prompt_text = open(ADJ_PROMPT, encoding="utf-8").read()
    os.makedirs(OUT_ADJ, exist_ok=True)
    jobs = [{"key": "control", "input_path": os.path.join(DATA, "rs28_adjudicate_input.json"),
             "output_path": os.path.join(OUT_ADJ, "control.json")},
            {"key": "treatment_recheck", "input_path": os.path.join(DATA, "rs27_adjudicate_input.json"),
             "output_path": os.path.join(OUT_ADJ, "treatment_recheck.json")}]
    args = {"stage": "adjudicate", "prompt_text": prompt_text, "records": jobs}
    p = os.path.join(DATA, "rs28_args_adjudicate.json")
    json.dump(args, open(p, "w"), indent=1, ensure_ascii=False)
    print(f"emit-adj-args: {len(jobs)} adjudication calls (control + treatment re-check) "
          f"-> {os.path.relpath(p, HERE)}")


# ---- one-sided Fisher exact (treatment enriched vs control), hypergeometric tail ----
def _logcomb(n, k):
    if k < 0 or k > n:
        return float("-inf")
    return math.lgamma(n + 1) - math.lgamma(k + 1) - math.lgamma(n - k + 1)


def fisher_one_sided(a, b, c, d):
    """2x2 [[a,b],[c,d]]: rows=treatment(a=genuine,b=not), control(c,d).
    One-sided p that treatment is enriched = P(A >= a) with fixed margins (hypergeometric)."""
    n1, n2 = a + b, c + d
    k = a + c            # total genuine
    N = n1 + n2
    denom = _logcomb(N, n1)
    lo, hi = max(0, k - n2), min(k, n1)
    p = 0.0
    for x in range(a, hi + 1):
        p += math.exp(_logcomb(k, x) + _logcomb(N - k, n1 - x) - denom)
    return min(1.0, p), (lo, hi)


def score():
    card = json.load(open(os.path.join(DATA, "rs28_card_confirmed.json")))
    n_carded = card["n_carded"]
    k1 = card["n_card_confirmed"]
    adj = _load_out(os.path.join(OUT_ADJ, "control.json"))
    verdicts = adj.get("verdicts", [])
    k2 = sum(1 for v in verdicts if v.get("mechanism_real"))
    # reproduction check on treatment's 6
    rep = _load_out(os.path.join(OUT_ADJ, "treatment_recheck.json"))
    rep_genuine = sum(1 for v in rep.get("verdicts", []) if v.get("mechanism_real"))
    rep_n = len(rep.get("verdicts", []))

    p_card_ctrl = k1 / n_carded if n_carded else 0.0
    p_e2e_ctrl = k2 / n_carded if n_carded else 0.0
    t_card, t_e2e = T_CARD_CONFIRMED / T_CARDED, T_GENUINE / T_CARDED
    enrich_card = t_card / max(p_card_ctrl, 1.0 / n_carded)
    enrich_e2e = t_e2e / max(p_e2e_ctrl, 1.0 / n_carded)
    fisher_p, _ = fisher_one_sided(T_GENUINE, T_CARDED - T_GENUINE, k2, n_carded - k2)

    # pre-registered verdict (47-PREREG.md)
    repro_ok = rep_genuine >= 4          # harness-validity gate (expect ~5/6)
    if not repro_ok:
        verdict = "HARNESS-INVALID"
    elif p_e2e_ctrl <= 0.05 and fisher_p < 0.05:
        verdict = "PASS"
    elif p_e2e_ctrl >= 0.10 or fisher_p > 0.10:
        verdict = "FAIL"
    else:
        verdict = "WEAK"

    out = {"experiment": "EXP-RS-28", "verdict": verdict,
           "reproduction_check": {"treatment_recheck_genuine": rep_genuine, "of": rep_n,
                                  "original_rs27_genuine": T_GENUINE, "repro_ok(>=4)": repro_ok},
           "control": {"n_carded": n_carded, "card_confirmed": k1, "adjudicated_genuine": k2,
                       "card_confirm_rate": round(p_card_ctrl, 4), "e2e_genuine_rate": round(p_e2e_ctrl, 4)},
           "treatment": {"n_carded": T_CARDED, "card_confirmed": T_CARD_CONFIRMED, "genuine": T_GENUINE,
                         "card_confirm_rate": round(t_card, 4), "e2e_genuine_rate": round(t_e2e, 4)},
           "enrichment": {"card_stage": round(enrich_card, 2), "end_to_end": round(enrich_e2e, 2)},
           "fisher_one_sided_p_e2e": round(fisher_p, 5),
           "control_genuine_verdicts": [v for v in verdicts if v.get("mechanism_real")]}
    json.dump(out, open(os.path.join(DATA, "rs28_verdict.json"), "w"), indent=1, ensure_ascii=False)
    print(f"=== EXP-RS-28 calibration control — VERDICT: {verdict} ===")
    print(f"  reproduction check (my adjudicator on treatment's 6): {rep_genuine}/{rep_n} genuine "
          f"(RS-27 original 5/6; ok>=4: {repro_ok})")
    print(f"  CONTROL   card {k1}/{n_carded}={p_card_ctrl:.4f}   e2e-genuine {k2}/{n_carded}={p_e2e_ctrl:.4f}")
    print(f"  TREATMENT card {T_CARD_CONFIRMED}/{T_CARDED}={t_card:.4f}   e2e-genuine {T_GENUINE}/{T_CARDED}={t_e2e:.4f}")
    print(f"  enrichment: card {enrich_card:.2f}x   end-to-end {enrich_e2e:.2f}x")
    print(f"  Fisher one-sided p (treatment enriched, e2e): {fisher_p:.5f}")
    print(f"  -> data/rs28_verdict.json")


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    for c in ("sample", "emit-card-args", "curate", "emit-adj-args", "score"):
        sub.add_parser(c)
    a = ap.parse_args()
    {"sample": sample, "emit-card-args": emit_card_args, "curate": curate,
     "emit-adj-args": emit_adj_args, "score": score}[a.cmd]()


if __name__ == "__main__":
    main()
