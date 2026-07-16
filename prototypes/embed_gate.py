#!/usr/bin/env python3
"""EXP-RS-21 (Phase 40) — decision machinery: the TOTAL-function verdict over both corpora.

Consumes embed_score.py per-corpus outputs (Feynman + modern) + the blind C-35 modern LLM baseline +
the per-model toytest-liveness attestation, and maps the outcome to EXACTLY ONE decision (MUST-1/2/3;
grid-verified total by embed_gate_selftest). Separated from scoring so the gate is auditable and the
verdict is a pure function of the frozen numbers.

Decision variables (C-45):
  LIVE(m)      = toytest_pass[m] AND modern_recall@10[m] >= LIVE_FLOOR   (broken-encoder guard, MUST-9;
                 LIVE_FLOOR=0.5 << MODERN_FLOOR: "the encoder works at all", distinct from no-regression)
  strictP1(m)  = m beats the 0.40 null AND pair04 in top-10 AND >=1 null-missed recovered (on Feynman)
  classPass    = EXISTS a LIVE reproducible-local model in {bge,specter,gte} with strictP1  (MUST-4: no mistral)
  headlinePass = LIVE(bge) AND strictP1(bge)
  R            = bge Feynman fwd recall@10 ;  M1mod = bge modern fwd recall@10
  P4           = >=1 other general model {gte,mistral} corroborates (pair04 top-10 AND beats null)
  P5obj        = bge pair04 objective card passes (cross-domain metadata AND cosine-margin) AND the
                 random-pair control mostly FAILS (control_pass_rate < CONTROL_MAX)

Total verdict tree (every leaf terminal; see embed_gate_selftest.py for the exhaustive grid check):
  ¬LIVE(bge)                              -> INVALID-headline-broken (investigate; do not claim a class-negative)
  LIVE(bge) ∧ headlinePass:
      M1mod < FLOOR                       -> WEAK-no-bank (anchor recovered but modern regressed)
      R>=0.80 ∧ P4 ∧ P5obj               -> ADVANCE
      else (R==0.60 tie, or missing P4/P5)-> PIVOT (bank artifact + cheap retriever; provisional)
  LIVE(bge) ∧ ¬headlinePass:
      classPass ∧ winner_modern>=FLOOR    -> WEAK-PIVOT (bank the winning local substrate; -> emb->LLM rerank)
      classPass ∧ winner_modern<FLOOR     -> WEAK-no-bank
      ¬classPass                          -> KILL (no live reproducible dense substrate recovers the
                                                    anchor beyond the ~chance lexical null; escalate to a
                                                    pure LLM-judge cascade — see note — likely the
                                                    terminal chapter verdict)
"""
import argparse
import json
import math
import os

MODERN_FLOOR = 0.833      # no-regression / banking floor = the modern lexical null (MUST-3)
LIVE_FLOOR = 0.5          # liveness floor: "encoder works at all" on the trivial modern set (MUST-9);
                          # << MODERN_FLOOR so a working-but-regressed model is NOT mislabeled broken
CONTROL_MAX = 0.30        # random-control pass-rate must stay below this for P5obj
LOCAL_CLASS = ["bge", "specter", "gte"]
GENERAL = ["gte", "mistral"]

HERE = os.path.dirname(os.path.abspath(__file__))
DATA = os.path.join(HERE, "data")


def binom_tail(n, p, k):
    """P(X >= k), X ~ Binomial(n,p)."""
    return sum(math.comb(n, i) * p**i * (1 - p)**(n - i) for i in range(k, n + 1))


def random_null(n_pairs, pool_size=35, k=10):
    """Chance-level recall@k on a pool of `pool_size` candidates (SHOULD-2)."""
    p = min(1.0, k / pool_size)
    exp = p  # expected fraction recovered
    return {"chance_recall@10_per_pair": p,
            "P(recall>=0.40)": binom_tail(n_pairs, p, math.ceil(0.40 * n_pairs)),
            "P(recall>=0.60)": binom_tail(n_pairs, p, math.ceil(0.60 * n_pairs)),
            "P(recall>=0.80)": binom_tail(n_pairs, p, math.ceil(0.80 * n_pairs)),
            "expected_recall": exp}


def fwd10(res, m):
    r = res["models"].get(m, {})
    return r.get("forward", {}).get("recall@10")


def compute(feyn, modern, toy_pass, c35=None):
    def modern10(m):
        v = fwd10(modern, m)
        return v if v is not None else -1.0

    def strictp1(m):
        return feyn["models"].get(m, {}).get("strict_P1", {}).get("strict_P1_pass", False)

    live = {m: bool(toy_pass.get(m)) and modern10(m) >= LIVE_FLOOR for m in LOCAL_CLASS}
    class_pass = any(live[m] and strictp1(m) for m in LOCAL_CLASS)
    headline_pass = live.get("bge", False) and strictp1("bge")
    R = fwd10(feyn, "bge")
    M1mod = modern10("bge")

    # P4 corroboration
    def beats_and_anchor(m):
        sp = feyn["models"].get(m, {}).get("strict_P1")
        if not sp:  # non-feynman-strictP1 model (mistral has strict_P1 too since feynman run computes it)
            f = feyn["models"].get(m, {}).get("forward", {})
            return f.get("recall@10", 0) > 0.40  # weaker corroboration if no strictP1 block
        return sp["beats_null_0.40"] and sp["pair04_top10"]
    p4 = any(beats_and_anchor(m) for m in GENERAL)

    # P5obj (anchored on pair04 for the headline)
    bge = feyn["models"].get("bge", {})
    card04 = bge.get("cards", {}).get("pair04-percolation-epidemics", {})
    ctrl = bge.get("random_control", {}).get("control_pass_rate")
    p5obj = bool(card04.get("objective_pass")) and (ctrl is not None and ctrl < CONTROL_MAX)

    # verdict tree (total)
    winner = None
    if not live.get("bge", False):
        verdict = "INVALID-headline-broken"
        rationale = ("bge failed the liveness gate (toytest and/or modern>=%.3f); a class-negative "
                     "cannot be reported from a possibly-broken headline encoder — investigate." % LIVE_FLOOR)
    elif headline_pass:
        if M1mod < MODERN_FLOOR:
            verdict = "WEAK-no-bank"
            rationale = "Headline recovers the anchor but modern regressed below the no-regression floor."
        elif R >= 0.80 and p4 and p5obj:
            verdict = "ADVANCE"
            rationale = "Strict double-beat on the discriminating corpus + family corroboration + objective card."
        else:
            verdict = "PIVOT"
            rationale = ("Headline ties/near-beats the incumbent WITH the anchor; bank the auditable card "
                         "+ cheap retriever; provisional (single-corpus, effective n~1 pair04). "
                         "Next build = embedding->LLM re-rank cascade.")
    else:
        if class_pass:
            winner = next(m for m in LOCAL_CLASS if live[m] and strictp1(m))
            if modern10(winner) >= MODERN_FLOOR:
                verdict = "WEAK-PIVOT"
                rationale = ("Headline bge did not cleanly recover the anchor, but LIVE local model '%s' "
                             "did (strict-P1); bank it as the cheap first stage; next = embedding->LLM "
                             "re-rank. Headline underperforms — provisional." % winner)
            else:
                verdict = "WEAK-no-bank"
                rationale = "A local model passes strict-P1 but regressed on modern -> do not bank."
        else:
            verdict = "KILL"
            rationale = ("No LIVE reproducible dense substrate recovers the pair04 anchor beyond the ~chance "
                         "lexical null. Static embedding geometry does not surface cross-domain mechanism "
                         "analogy; only full-context LLM reasoning does. Escalate to a pure LLM-judge "
                         "cascade that STRUCTURALLY differs from the C-20 one-shot 35-way baseline "
                         "(e.g. pairwise/tournament judging, or scaling beyond the 36-paper pool); if no "
                         "such structural difference is available, this KILL is the terminal chapter verdict.")

    n_pairs = feyn["n_eval_pairs"]
    return {
        "verdict": verdict, "rationale": rationale,
        "R_bge_feynman_recall@10": R, "M1_bge_modern_recall@10": M1mod,
        "MODERN_FLOOR": MODERN_FLOOR, "live": live, "toytest_pass": toy_pass,
        "class_pass": class_pass, "headline_pass": headline_pass, "winner_banked": winner,
        "P4_family_corroboration": p4, "P5obj_pair04_card_and_control": p5obj,
        "pair04_card_bge": card04, "random_control_pass_rate_bge": ctrl,
        "strict_P1_by_model": {m: feyn["models"].get(m, {}).get("strict_P1") for m in feyn["models"]},
        "random_null_feynman": random_null(n_pairs),
        "c35_modern_bar": c35, "note_c35": "modern is a NO-REGRESSION FLOOR only (null already 0.833); "
        "C-35 is descriptive context, NOT a strict-beat requirement (MUST-3).",
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--feynman", default=os.path.join(DATA, "embed_results_feynman.json"))
    ap.add_argument("--modern", default=os.path.join(DATA, "embed_results_modern.json"))
    ap.add_argument("--toytest", default=os.path.join(DATA, "embed_toytest_pass.json"))
    ap.add_argument("--c35", default=os.path.join(DATA, "baseline_results_modern.json"))
    ap.add_argument("--out", default=os.path.join(DATA, "embed_verdict.json"))
    args = ap.parse_args()

    feyn = json.load(open(args.feynman))
    modern = json.load(open(args.modern))
    toy = json.load(open(args.toytest)) if os.path.exists(args.toytest) else {}
    c35 = None
    if os.path.exists(args.c35):
        c35 = json.load(open(args.c35)).get("recall@10")

    out = compute(feyn, modern, toy, c35)
    json.dump(out, open(args.out, "w"), indent=1, ensure_ascii=False)

    print("=== EXP-RS-21 VERDICT ===")
    print(f"  R (bge Feynman recall@10) = {out['R_bge_feynman_recall@10']}")
    print(f"  M1 (bge modern recall@10) = {out['M1_bge_modern_recall@10']}  (floor {MODERN_FLOOR})")
    print(f"  live = {out['live']}")
    print(f"  headlinePass={out['headline_pass']}  classPass={out['class_pass']}  "
          f"P4={out['P4_family_corroboration']}  P5obj={out['P5obj_pair04_card_and_control']}")
    rn = out["random_null_feynman"]
    print(f"  random null: chance recall@10/pair={rn['chance_recall@10_per_pair']:.3f}  "
          f"P(R>=0.40)={rn['P(recall>=0.40)']:.3f}  P(R>=0.60)={rn['P(recall>=0.60)']:.3f}")
    print(f"\n  ==> {out['verdict']}")
    print(f"      {out['rationale']}")
    print(f"\nwrote {args.out}")


if __name__ == "__main__":
    main()
