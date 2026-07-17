#!/usr/bin/env python3
"""EXP-RS-22 (Phase 41) — instrument harness (emit-inputs -> dispatch -> score).

Implements rs22_operational_spec.md (blind-authored, sha256 aecc04a0…) VERBATIM: field_label,
recognition menu (K_menu=8), K=50 retrieval pool, deterministic rank repair, self-field guard,
clean-stratum predicate. Deterministic + RNG-free modulo the pinned model's temperature-0 outputs.

Instruments (each = one frozen prompt, one blind input per unit, fresh-session subagent / Mistral):
  recall        side_a {title,abstract}                       -> {target_field,shared_mechanism,confidence,brief_reason}
  recognition   side_a {title,abstract,field_options}         -> {chosen_field,confidence,brief_reason}
  familiarity   side_b {title}                    (control)   -> {recognized,stated_result_or_null,confidence}
  openbook      side_a+side_b {title,abstract}     (control)  -> {mapping,shares_method,brief_justification}
  mechanism     side_b {title,abstract}                       -> {core_mechanism,brief_reason}
  retrieval     side_a + K=50 pool                            -> {ranking:[arxiv_id,...]}
  judge (2nd)   recall out + side_b reference                 -> {field_match,mechanism_match,overall_equivalent,rationale}

Two model families share the SAME blind input files:
  claude arm  = Workflow fan-out of fresh general-purpose subagents (pinned Opus 4.8) -> data/rs22_out/<instr>/
  mistral arm = direct Mistral REST (confirmatory) -> data/rs22_out_mistral/<instr>/

Usage (see main()):
  emit-inputs --units slice:15         # phase-1 blind inputs + manifest for the first 15 pairs
  emit-inputs --units anchors          # Feynman retrieval-reproduction inputs (incumbent pool)
  emit-inputs --units posctrl|negctrl  # control inputs
  emit-inputs --units all              # all 420 pairs
  emit-claude-args --instr recall --units slice:15   # -> data/rs22_args_<instr>_<tag>.json for Workflow
  emit-judge-inputs --units slice:15   # phase-2 judge inputs (needs recall+mechanism outputs)
  run-mistral --instr recall --units slice:15 --workers 4
  score --units slice:15               # assemble clean_records + posctrl + per-unit table
"""
import argparse
import concurrent.futures as cf
import hashlib
import json
import os
import re
import time
import urllib.error
import urllib.request

from sme_lite import _WORD, _STOP, cosine  # lexical-null tokenizer (C-17)

HERE = os.path.dirname(os.path.abspath(__file__))
DATA = os.path.join(HERE, "data")

CONST = json.load(open(os.path.join(DATA, "rs22_constants.json")))["constants"]
K_POOL = CONST["K_pool_size"]          # 50
K_MENU = 8                             # operational spec §2
SELF_FIELD_OVERLAP = 0.67              # spec §5
RECOG_CONF_MAX = 0.5                   # spec §6
US = "\x1f"

MINED = os.path.join(DATA, "rs22_mined_pairs.json")
OUT_CLAUDE = os.path.join(DATA, "rs22_out")
OUT_MISTRAL = os.path.join(DATA, "rs22_out_mistral")
IN_DIR = os.path.join(DATA, "rs22_in")

# instrument -> frozen prompt file (all SHA-frozen)
PROMPTS = {
    "recall": "rs22_probe_recall.md",
    "recognition": "rs22_probe_recognition.md",
    "familiarity": "rs22_probe_familiarity.md",
    "openbook": "rs22_probe_openbook.md",
    "mechanism": "rs22_probe_mechanism.md",
    "retrieval": "rs22_retrieval_prompt.md",
    "judge": "rs22_judge_semantic.md",
}
PHASE1 = ["recall", "recognition", "familiarity", "openbook", "mechanism", "retrieval"]


# ============================ deterministic core (spec) ============================
def H(domain, pair_id, x):
    return hashlib.sha256((domain + US + pair_id + US + x).encode("utf-8")).hexdigest()


def Hsel(pid, x):
    return H("rs22-select", pid, x)


def Hord(pid, x):
    return H("rs22-order", pid, x)


def normalize_id(x):
    return re.sub(r"v\d+$", "", str(x).strip())


# ---- §1 field_label ----
ARCHIVE_LABELS = {
    "astro-ph": "astrophysics", "cond-mat": "condensed matter physics",
    "cs": "computer science", "econ": "economics",
    "eess": "electrical eng. and systems sci.", "gr-qc": "general relativity and gravitation",
    "hep-ex": "high-energy particle physics (experiment)", "hep-lat": "lattice field theory",
    "hep-ph": "high-energy particle phenomenology", "hep-th": "high-energy theoretical physics",
    "math": "mathematics", "math-ph": "mathematical physics", "nlin": "nonlinear dynamics",
    "nucl-ex": "nuclear physics (experiment)", "nucl-th": "nuclear theory",
    "physics": "physics", "q-bio": "quantitative biology", "q-fin": "quantitative finance",
    "quant-ph": "quantum physics", "stat": "statistics",
}
SUBCAT_LABELS = {
    "cond-mat.stat-mech": "statistical mechanics", "cond-mat.dis-nn": "disordered systems and neural networks",
    "cond-mat.mes-hall": "mesoscopic and nanoscale physics", "cond-mat.mtrl-sci": "materials science",
    "cond-mat.soft": "soft condensed matter", "cond-mat.str-el": "strongly correlated electrons",
    "cond-mat.supr-con": "superconductivity", "cond-mat.quant-gas": "quantum gases",
    "cond-mat.other": "condensed matter physics",
    "physics.optics": "optics", "physics.flu-dyn": "fluid dynamics",
    "physics.soc-ph": "physics of social systems", "physics.bio-ph": "biological physics",
    "physics.chem-ph": "chemical physics", "physics.plasm-ph": "plasma physics",
    "physics.geo-ph": "geophysics", "physics.ao-ph": "atmospheric and oceanic physics",
    "physics.atom-ph": "atomic physics", "physics.comp-ph": "computational physics",
    "physics.data-an": "data analysis and statistics", "physics.class-ph": "classical physics",
    "physics.med-ph": "medical physics", "physics.app-ph": "applied physics",
    "physics.acc-ph": "accelerator physics", "physics.space-ph": "space physics",
    "physics.ins-det": "instrumentation and detectors", "physics.atm-clus": "atomic and molecular clusters",
    "nlin.CD": "chaotic dynamics", "nlin.PS": "pattern formation and solitons",
    "nlin.AO": "adaptation and self-organizing systems", "nlin.SI": "exactly solvable and integrable systems",
    "nlin.CG": "cellular automata and lattice gases",
    "q-bio.PE": "population biology and evolution", "q-bio.NC": "neuroscience (neurons and cognition)",
    "q-bio.BM": "biomolecules", "q-bio.MN": "molecular networks",
    "q-bio.QM": "quantitative methods in biology", "q-bio.CB": "cell behavior",
    "q-bio.GN": "genomics", "q-bio.SC": "subcellular processes", "q-bio.TO": "tissues and organs",
    "q-fin.ST": "statistical finance / econophysics", "q-fin.MF": "mathematical finance",
    "q-fin.PR": "derivative pricing", "q-fin.RM": "risk management",
    "q-fin.PM": "portfolio management", "q-fin.TR": "trading and market microstructure",
    "q-fin.CP": "computational finance", "q-fin.EC": "financial economics", "q-fin.GN": "general finance",
    "stat.ML": "machine learning (statistics)", "stat.ME": "statistical methodology",
    "stat.TH": "statistics theory", "stat.AP": "applied statistics", "stat.CO": "computational statistics",
    "cs.LG": "machine learning", "cs.AI": "artificial intelligence", "cs.CV": "computer vision",
    "cs.CL": "computational linguistics / NLP", "cs.NE": "neural and evolutionary computing",
    "cs.IT": "information theory", "cs.DS": "data structures and algorithms",
    "cs.SI": "social and information networks", "cs.GT": "algorithmic game theory",
    "cs.CC": "computational complexity", "cs.RO": "robotics", "cs.DC": "distributed and parallel computing",
    "cs.SY": "systems and control", "cs.NI": "networking and internet architecture",
    "math.PR": "probability theory", "math.ST": "mathematical statistics",
    "math.OC": "optimization and control", "math.DS": "dynamical systems", "math.QA": "quantum algebra",
    "math.AG": "algebraic geometry", "math.CO": "combinatorics", "math.NA": "numerical analysis",
    "math.AP": "analysis of PDEs", "math.NT": "number theory", "math.DG": "differential geometry",
    "math.GT": "geometric topology", "math.RT": "representation theory", "math.FA": "functional analysis",
    "math.MP": "mathematical physics", "math.GR": "group theory", "math.RA": "rings and algebras",
    "math.CA": "classical analysis (real/complex)", "math.SG": "symplectic geometry",
    "eess.SP": "signal processing", "eess.SY": "systems and control",
    "eess.IV": "image and video processing", "eess.AS": "audio and speech processing",
    "econ.EM": "econometrics", "econ.TH": "economic theory", "econ.GN": "general economics",
    "astro-ph.CO": "cosmology", "astro-ph.GA": "astrophysics of galaxies",
    "astro-ph.HE": "high-energy astrophysics", "astro-ph.SR": "solar and stellar astrophysics",
    "astro-ph.EP": "earth and planetary astrophysics", "astro-ph.IM": "astronomical instrumentation and methods",
}


def prettify_tag(t):
    return " ".join(t.replace("-", " ").replace(".", " ").split()).strip().lower()


# Faithful bug-fix over rs22_operational_spec.md §1 (documented deviation): the spec lowercases the
# category before the SUBCAT_LABELS lookup, but its table keys carry uppercase arXiv subtags
# (math.QA, q-fin.ST, cs.LG, …) — so under the spec-as-written EVERY letter-code subcategory misses
# the table and falls to the cryptic "archive (subtag)" fallback (e.g. math.QA -> "mathematics (qa)"
# instead of "quantum algebra"). The table's existence shows intent = the nice labels. Fix: match the
# authoritative table CASE-INSENSITIVELY (lowercased keys are collision-free). This realizes the
# spec's evident intent, is applied consistently everywhere field_label is used (so all invariants
# hold), and only makes field labels clearer — it cannot bias the REASON direction. Noted in
# 41-VERIFICATION.
SUBCAT_LC = {k.lower(): v for k, v in SUBCAT_LABELS.items()}


def field_label(category):
    c = re.sub(r"[^a-z0-9.\-]", "", str(category).strip().lower())
    if c in SUBCAT_LC:
        return SUBCAT_LC[c]
    topcat, _, subtag = c.partition(".")
    if topcat in ARCHIVE_LABELS:
        base = ARCHIVE_LABELS[topcat]
        return f"{base} ({prettify_tag(subtag)})" if subtag else base
    return prettify_tag(c)


ARCHIVE_FALLBACK = sorted(set(ARCHIVE_LABELS.values()))


# ---- corpus tables ----
def load_corpus():
    return json.load(open(MINED))["pairs"]


def corpus_field_labels(pairs):
    L = set()
    for p in pairs:
        L.add(field_label(p["side_a"]["category"]))
        L.add(field_label(p["side_b"]["category"]))
    return sorted(L)


def build_papers(pairs):
    papers = {}
    for p in sorted(pairs, key=lambda p: p["pair_id"]):
        for side in (p["side_a"], p["side_b"]):
            pid = normalize_id(side["arxiv_id"])
            if pid not in papers:
                papers[pid] = {"arxiv_id": pid, "title": side["title"],
                               "abstract": side["abstract"], "category": side["category"]}
    return papers


# ---- §2 recognition menu ----
def field_options(pair, labels):
    pid = pair["pair_id"]
    correct = field_label(pair["side_b"]["category"])
    a_label = field_label(pair["side_a"]["category"])
    D = [lab for lab in labels if lab != correct and lab != a_label]
    D_sorted = sorted(D, key=lambda lab: (Hsel(pid, lab), lab))
    distractors = D_sorted[:(K_MENU - 1)]
    if len(distractors) < K_MENU - 1:
        for lab in ARCHIVE_FALLBACK:
            if len(distractors) >= K_MENU - 1:
                break
            if lab == correct or lab == a_label or lab in distractors:
                continue
            distractors.append(lab)
    menu = distractors + [correct]
    return sorted(menu, key=lambda lab: (Hord(pid, lab), lab))


# ---- §3 retrieval pool ----
def retrieval_pool(pair, papers):
    pid = pair["pair_id"]
    a_id = normalize_id(pair["side_a"]["arxiv_id"])
    b_id = normalize_id(pair["side_b"]["arxiv_id"])
    cand = [q for q in papers if q != a_id and q != b_id]
    cand_sorted = sorted(cand, key=lambda q: (Hsel(pid, q), q))
    distractors = cand_sorted[:(K_POOL - 1)]
    pool_ids = distractors + [b_id]
    K = len(pool_ids)
    presented = sorted(pool_ids, key=lambda q: (Hord(pid, q), q))
    return presented, K


def rank_side_b(pair, presented, K, model_ranked_ids):
    pool = set(presented)
    seen, cleaned = set(), []
    for rid in model_ranked_ids:
        nid = normalize_id(rid)
        if nid in pool and nid not in seen:
            cleaned.append(nid)
            seen.add(nid)
    missing = sorted(q for q in pool if q not in seen)
    full = cleaned + missing
    assert len(full) == K and set(full) == pool
    b_id = normalize_id(pair["side_b"]["arxiv_id"])
    rank_1 = full.index(b_id) + 1
    pctile = 100.0 * (rank_1 - 1) / (K - 1)
    return rank_1, pctile


# ---- §5 self-field guard ----
STOP = {"and", "of", "the", "in", "for", "a", "an", "to", "on", "with"}


def normalize_label(s):
    return " ".join(re.sub(r"[^a-z0-9]+", " ", str(s).lower()).split()).strip()


def content_tokens(s):
    return {t for t in normalize_label(s).split() if t not in STOP}


def same_field(x, y):
    if x is None or normalize_label(x) == "":
        return False
    if normalize_label(x) == normalize_label(y):
        return True
    Tx, Ty = content_tokens(x), content_tokens(y)
    if not Tx or not Ty:
        return False
    return (len(Tx & Ty) / min(len(Tx), len(Ty))) >= SELF_FIELD_OVERLAP


def recall_fires(a_label, judge_out, recall_out):
    if not judge_out or not judge_out.get("overall_equivalent"):
        return False
    return not same_field(recall_out.get("target_field"), a_label)


def is_clean(pair, judge_out, recall_out, recog_out):
    a_label = field_label(pair["side_a"]["category"])
    b_label = field_label(pair["side_b"]["category"])
    fails_recall = not recall_fires(a_label, judge_out, recall_out)
    recog_wrong = recog_out.get("chosen_field") != b_label
    recog_low = float(recog_out.get("confidence", 1.0)) <= RECOG_CONF_MAX
    return fails_recall and recog_wrong and recog_low


# ---- lexical null over a pool (C-17) ----
def tok(text):
    return [t for t in _WORD.findall(str(text).lower()) if t not in _STOP and len(t) > 2]


def build_lexical_space(papers):
    """TF-IDF over all mined papers (title+abstract), returns per-id normalized vec + idf."""
    import math
    from collections import Counter
    docs = {q: Counter(tok(p["title"] + ". " + p["abstract"])) for q, p in papers.items()}
    df = Counter()
    for c in docs.values():
        df.update(c.keys())
    N = len(docs)
    idf = {w: math.log((N + 1) / (df[w] + 1)) + 1 for w in df}
    vecs = {}
    for q, c in docs.items():
        v = {w: f * idf[w] for w, f in c.items()}
        nrm = math.sqrt(sum(x * x for x in v.values())) or 1.0
        vecs[q] = {w: x / nrm for w, x in v.items()}
    return vecs


def lexical_pctile(pair, presented, K, vecs):
    """Rank side_b in the pool by lexical cosine(side_a, candidate); return (rank1, pctile)."""
    a_id = normalize_id(pair["side_a"]["arxiv_id"])
    b_id = normalize_id(pair["side_b"]["arxiv_id"])
    qv = vecs.get(a_id, {})
    scored = sorted(presented, key=lambda cid: (-cosine(qv, vecs.get(cid, {})), cid))
    rank_1 = scored.index(b_id) + 1
    return rank_1, 100.0 * (rank_1 - 1) / (K - 1)


# ============================ unit sources ============================
def parse_units(spec, pairs):
    """Return list of unit dicts {utype, key, ...}. spec: all | slice:N | anchors | posctrl | negctrl."""
    if spec == "all":
        return [{"utype": "pair", "key": p["pair_id"], "pair": p} for p in pairs]
    if spec.startswith("slice:"):
        n = int(spec.split(":")[1])
        return [{"utype": "pair", "key": p["pair_id"], "pair": p} for p in pairs[:n]]
    if spec == "posctrl":
        d = json.load(open(os.path.join(DATA, "rs22_posctrl_set.json")))
        return [{"utype": "posctrl", "key": it["id"], "item": it} for it in d["items"]]
    if spec == "negctrl":
        d = json.load(open(os.path.join(DATA, "rs22_negctrl_set.json")))
        return [{"utype": "negctrl", "key": it["id"], "item": it} for it in d["items"]]
    if spec == "anchors":
        return anchor_units()
    raise SystemExit(f"unknown --units {spec}")


def anchor_units():
    """5 Feynman evaluable pairs over the 36-paper MVP pool (reproduce incumbent recall@10=0.60)."""
    feyn = json.load(open(os.path.join(DATA, "feynman_10pair_papers.json")))
    corpus = json.load(open(os.path.join(DATA, "mvp_corpus.json")))
    ev = set(feyn["evaluable_pairs"])
    all_ids = [p["arxiv_id"] for p in corpus["papers"]]
    by_id = {p["arxiv_id"]: p for p in corpus["papers"]}
    units = []
    for p in feyn["pairs"]:
        if p["id"].split("-")[0] not in ev:
            continue
        a, b = p["side_a"]["arxiv_id"], p["side_b"]["arxiv_id"]
        if a not in by_id or b not in by_id:
            continue
        units.append({"utype": "anchor", "key": p["id"], "side_a": by_id[a], "side_b_id": b,
                      "pool_ids": [c for c in all_ids if c != a], "by_id": by_id})
    return units


# ============================ input builders (phase 1) ============================
def _corpus_labels_cache(pairs, _c={}):
    if "L" not in _c:
        _c["L"] = corpus_field_labels(pairs)
    return _c["L"]


def build_input(instr, unit, pairs, papers):
    """Return (input_dict, meta_dict) for a phase-1 instrument on a unit, or None if N/A."""
    ut = unit["utype"]
    if ut == "pair":
        p = unit["pair"]
        a, b = p["side_a"], p["side_b"]
        if instr == "recall":
            return {"title": a["title"], "abstract": a["abstract"]}, {}
        if instr == "recognition":
            menu = field_options(p, _corpus_labels_cache(pairs))
            return ({"title": a["title"], "abstract": a["abstract"], "field_options": menu},
                    {"field_options": menu, "correct": field_label(b["category"])})
        if instr == "familiarity":
            return {"title": b["title"]}, {}
        if instr == "openbook":
            return {"title_a": a["title"], "abstract_a": a["abstract"],
                    "title_b": b["title"], "abstract_b": b["abstract"]}, {}
        if instr == "mechanism":
            return {"title": b["title"], "abstract": b["abstract"]}, {}
        if instr == "retrieval":
            presented, K = retrieval_pool(p, papers)
            cands = [{"arxiv_id": q, "title": papers[q]["title"], "abstract": papers[q]["abstract"]}
                     for q in presented]
            return ({"query_title": a["title"], "query_abstract": a["abstract"], "candidates": cands},
                    {"presented": presented, "K": K, "target": normalize_id(b["arxiv_id"])})
    elif ut == "anchor":
        if instr != "retrieval":
            return None
        sa = unit["side_a"]
        cands = [{"arxiv_id": c, "title": unit["by_id"][c]["title"],
                  "abstract": unit["by_id"][c]["abstract"]} for c in unit["pool_ids"]]
        return ({"query_title": sa["title"], "query_abstract": sa["abstract"], "candidates": cands},
                {"presented": unit["pool_ids"], "K": len(unit["pool_ids"]), "target": unit["side_b_id"]})
    elif ut == "posctrl":
        it = unit["item"]
        if instr == "recall":
            return {"title": it["side_a_title"], "abstract": it["side_a_abstract"]}, {}
        if instr == "recognition":
            menu = posctrl_menu(it, pairs)
            return ({"title": it["side_a_title"], "abstract": it["side_a_abstract"], "field_options": menu},
                    {"field_options": menu, "correct": it["side_b_field_label"]})
    elif ut == "negctrl":
        it = unit["item"]
        if instr == "recall":
            return {"title": it["title"], "abstract": it["abstract"]}, {}
    return None


def posctrl_menu(it, pairs):
    """Deterministic 8-option menu for a positive-control item: correct side_b_field_label + distractors."""
    pid = it["id"]
    correct = it["side_b_field_label"]
    a_label = it.get("field_a", "")
    L = _corpus_labels_cache(pairs)
    D = [lab for lab in L if lab != correct and normalize_label(lab) != normalize_label(a_label)]
    D_sorted = sorted(D, key=lambda lab: (Hsel(pid, lab), lab))[:(K_MENU - 1)]
    menu = D_sorted + [correct]
    return sorted(menu, key=lambda lab: (Hord(pid, lab), lab))


# ============================ emit-inputs ============================
def safe(k):
    return str(k).replace("/", "_")


def emit_inputs(units_spec):
    pairs = load_corpus()
    papers = build_papers(pairs)
    units = parse_units(units_spec, pairs)
    manifest_records = []
    for u in units:
        instrs = PHASE1 if u["utype"] in ("pair",) else \
            (["retrieval"] if u["utype"] == "anchor" else
             (["recall", "recognition"] if u["utype"] == "posctrl" else ["recall"]))
        for instr in instrs:
            built = build_input(instr, u, pairs, papers)
            if built is None:
                continue
            inp, meta = built
            d = os.path.join(IN_DIR, instr)
            os.makedirs(d, exist_ok=True)
            path = os.path.join(d, f"{safe(u['key'])}.json")
            sha = hashlib.sha256(json.dumps(inp, sort_keys=True, ensure_ascii=False).encode()).hexdigest()
            json.dump({"input": inp, "meta": meta, "input_sha256": sha, "instr": instr,
                       "key": u["key"], "utype": u["utype"]},
                      open(path, "w"), indent=1, ensure_ascii=False)
            manifest_records.append({"instr": instr, "key": u["key"], "utype": u["utype"],
                                     "input_path": path, "input_sha256": sha,
                                     "out_claude": os.path.join(OUT_CLAUDE, instr, f"{safe(u['key'])}.json"),
                                     "out_mistral": os.path.join(OUT_MISTRAL, instr, f"{safe(u['key'])}.json")})
    tag = units_spec.replace(":", "")
    mpath = os.path.join(DATA, f"rs22_manifest_{tag}.json")
    json.dump({"units_spec": units_spec, "n_units": len(units), "n_records": len(manifest_records),
               "records": manifest_records}, open(mpath, "w"), indent=1, ensure_ascii=False)
    from collections import Counter
    by_instr = Counter(r["instr"] for r in manifest_records)
    print(f"emit-inputs[{units_spec}]: {len(units)} units -> {len(manifest_records)} phase-1 inputs")
    print("  by instrument:", dict(by_instr))
    print(f"  manifest: {os.path.relpath(mpath, HERE)}")
    return mpath


def emit_judge_inputs(units_spec, arm="claude"):
    """Phase 2: build judge inputs from recall + mechanism outputs (pairs + posctrl)."""
    pairs = load_corpus()
    by_pid = {p["pair_id"]: p for p in pairs}
    posctrl = {it["id"]: it for it in json.load(open(os.path.join(DATA, "rs22_posctrl_set.json")))["items"]} \
        if os.path.exists(os.path.join(DATA, "rs22_posctrl_set.json")) else {}
    units = parse_units(units_spec, pairs)
    out_root = OUT_CLAUDE if arm == "claude" else OUT_MISTRAL
    recs, skipped = [], 0
    for u in units:
        if u["utype"] not in ("pair", "posctrl"):
            continue
        rec_p = os.path.join(out_root, "recall", f"{safe(u['key'])}.json")
        if not os.path.exists(rec_p):
            skipped += 1
            continue
        recall_out = _load_json_out(rec_p)
        if u["utype"] == "pair":
            p = by_pid[u["key"]]
            ref_field = field_label(p["side_b"]["category"])
            mech_p = os.path.join(out_root, "mechanism", f"{safe(u['key'])}.json")
            if not os.path.exists(mech_p):
                skipped += 1
                continue
            ref_mech = _load_json_out(mech_p).get("core_mechanism", "")
        else:
            it = posctrl[u["key"]]
            ref_field = it["side_b_field_label"]
            ref_mech = it["shared_mechanism"]
        inp = {"target_field": recall_out.get("target_field"),
               "shared_mechanism": recall_out.get("shared_mechanism"),
               "reference_field": ref_field, "reference_mechanism": ref_mech}
        d = os.path.join(IN_DIR, "judge")
        os.makedirs(d, exist_ok=True)
        path = os.path.join(d, f"{safe(u['key'])}.json")
        sha = hashlib.sha256(json.dumps(inp, sort_keys=True, ensure_ascii=False).encode()).hexdigest()
        json.dump({"input": inp, "meta": {}, "input_sha256": sha, "instr": "judge",
                   "key": u["key"], "utype": u["utype"]}, open(path, "w"), indent=1, ensure_ascii=False)
        recs.append({"instr": "judge", "key": u["key"], "utype": u["utype"], "input_path": path,
                     "input_sha256": sha,
                     "out_claude": os.path.join(OUT_CLAUDE, "judge", f"{safe(u['key'])}.json"),
                     "out_mistral": os.path.join(OUT_MISTRAL, "judge", f"{safe(u['key'])}.json")})
    tag = units_spec.replace(":", "")
    mpath = os.path.join(DATA, f"rs22_manifest_judge_{tag}_{arm}.json")
    json.dump({"units_spec": units_spec, "arm": arm, "n_records": len(recs), "records": recs},
              open(mpath, "w"), indent=1, ensure_ascii=False)
    print(f"emit-judge-inputs[{units_spec},{arm}]: {len(recs)} judge inputs "
          f"({skipped} skipped for missing recall/mechanism) -> {os.path.relpath(mpath, HERE)}")
    return mpath


def _load_json_out(path):
    """Read a model output file that may be raw JSON or {output:...} wrapped; tolerate fences."""
    raw = open(path, encoding="utf-8").read().strip()
    if raw.startswith("```"):
        raw = raw.strip("`")
        raw = raw[raw.find("{"):raw.rfind("}") + 1]
    try:
        d = json.loads(raw)
    except ValueError:
        d = json.loads(raw[raw.find("{"):raw.rfind("}") + 1])
    return d.get("output", d) if isinstance(d, dict) else d


# ============================ emit-claude-args (Workflow) ============================
def emit_claude_args(instr, units_spec):
    tag = units_spec.replace(":", "")
    if instr == "judge":
        mpath = os.path.join(DATA, f"rs22_manifest_judge_{tag}_claude.json")
    else:
        mpath = os.path.join(DATA, f"rs22_manifest_{tag}.json")
    manifest = json.load(open(mpath))
    prompt_path = os.path.join(HERE, PROMPTS[instr])
    recs = [r for r in manifest["records"] if r["instr"] == instr]
    args = {"instr": instr, "prompt_path": prompt_path, "units_spec": units_spec,
            "records": [{"key": r["key"], "input_path": r["input_path"], "output_path": r["out_claude"]}
                        for r in recs]}
    p = os.path.join(DATA, f"rs22_args_{instr}_{tag}.json")
    json.dump(args, open(p, "w"), indent=1, ensure_ascii=False)
    print(f"emit-claude-args[{instr},{units_spec}]: {len(recs)} calls -> {os.path.relpath(p, HERE)}")
    print(f"  prompt: {PROMPTS[instr]}  output dir: {os.path.relpath(os.path.join(OUT_CLAUDE, instr), HERE)}/")
    return p


# ============================ mistral arm ============================
def frozen_prompt(instr):
    return open(os.path.join(HERE, PROMPTS[instr]), encoding="utf-8").read()


def mistral_call(prompt, blind_input, model, key, retries=5):
    body = json.dumps({
        "model": model,
        "messages": [{"role": "system", "content": prompt},
                     {"role": "user", "content": json.dumps(blind_input, ensure_ascii=False)}],
        "response_format": {"type": "json_object"}, "temperature": 0, "max_tokens": 4000,
    }).encode()
    last = None
    for attempt in range(retries):
        try:
            req = urllib.request.Request(
                "https://api.mistral.ai/v1/chat/completions", data=body,
                headers={"Authorization": f"Bearer {key}", "Content-Type": "application/json"})
            r = json.load(urllib.request.urlopen(req, timeout=120))
            return json.loads(r["choices"][0]["message"]["content"])
        except (urllib.error.HTTPError, urllib.error.URLError, KeyError, ValueError, TypeError) as e:
            last = e
            time.sleep(3 * (attempt + 1))
    raise RuntimeError(f"mistral_call failed after {retries}: {last}")


def run_mistral(instr, units_spec, model, workers):
    key = os.environ.get("MISTRAL_API_KEY")
    if not key:
        raise SystemExit("MISTRAL_API_KEY not in env")
    tag = units_spec.replace(":", "")
    mpath = os.path.join(DATA, f"rs22_manifest_judge_{tag}_mistral.json") if instr == "judge" \
        else os.path.join(DATA, f"rs22_manifest_{tag}.json")
    manifest = json.load(open(mpath))
    recs = [r for r in manifest["records"] if r["instr"] == instr]
    prompt = frozen_prompt(instr)
    out_dir = os.path.join(OUT_MISTRAL, instr)
    os.makedirs(out_dir, exist_ok=True)

    def task(rec):
        opath = os.path.join(out_dir, f"{safe(rec['key'])}.json")
        if os.path.exists(opath):
            return "cached"
        blind = json.load(open(rec["input_path"]))["input"]
        v = mistral_call(prompt, blind, model, key)
        json.dump(v, open(opath, "w"), indent=1, ensure_ascii=False)
        return "ok"

    done, errs = 0, []
    with cf.ThreadPoolExecutor(max_workers=workers) as ex:
        futs = {ex.submit(task, r): r for r in recs}
        for fut in cf.as_completed(futs):
            rec = futs[fut]
            try:
                fut.result()
                done += 1
                if done % 20 == 0 or done == len(recs):
                    print(f"  mistral {instr} {done}/{len(recs)}")
            except Exception as e:  # noqa: BLE001
                errs.append((rec["key"], str(e)[:120]))
    print(f"run-mistral[{instr},{units_spec}]: {done - len(errs)} ok, {len(errs)} errs -> {out_dir}")
    if errs:
        print("  errs:", errs[:5])


# ============================ score ============================
def _out(root, instr, key):
    p = os.path.join(root, instr, f"{safe(key)}.json")
    return _load_json_out(p) if os.path.exists(p) else None


def score(units_spec, arm="claude"):
    pairs = load_corpus()
    papers = build_papers(pairs)
    vecs = build_lexical_space(papers)
    units = parse_units(units_spec, pairs)
    root = OUT_CLAUDE if arm == "claude" else OUT_MISTRAL
    # RS-21 dense-embedding null (optional): fold into maxnull via min(pctile) when present
    emb_null_path = os.path.join(DATA, "rs22_embed_null.json")
    emb_null = json.load(open(emb_null_path))["ranks"] if os.path.exists(emb_null_path) else {}

    per_unit, clean_records, missing = [], [], []
    anchor_ranks = {}
    for u in units:
        key, ut = u["key"], u["utype"]
        if ut == "anchor":
            ret = _out(root, "retrieval", key)
            if not ret:
                missing.append((key, "retrieval"))
                continue
            pool = u["pool_ids"]
            fake_pair = {"pair_id": key, "side_a": u["side_a"],
                         "side_b": {"arxiv_id": u["side_b_id"]}}
            r1, _ = rank_side_b(fake_pair, pool, len(pool), ret.get("ranking", []))
            anchor_ranks[key] = r1
            continue
        if ut == "pair":
            p = u["pair"]
            recall_o = _out(root, "recall", key)
            recog_o = _out(root, "recognition", key)
            judge_o = _out(root, "judge", key)
            ret_o = _out(root, "retrieval", key)
            for nm, o in (("recall", recall_o), ("recognition", recog_o),
                          ("judge", judge_o), ("retrieval", ret_o)):
                if o is None:
                    missing.append((key, nm))
            if None in (recall_o, recog_o, judge_o, ret_o):
                continue
            presented, K = retrieval_pool(p, papers)
            r1_llm, pct_llm = rank_side_b(p, presented, K, ret_o.get("ranking", []))
            r1_lex, pct_lex = lexical_pctile(p, presented, K, vecs)
            # bestnull = better (lower pctile) of lexical (and embedding when available)
            pct_emb = emb_null.get(key, {}).get("pctile")
            pct_bestnull = min(pct_lex, pct_emb) if pct_emb is not None else pct_lex
            a_label = field_label(p["side_a"]["category"])
            fired = recall_fires(a_label, judge_o, recall_o)
            clean = is_clean(p, judge_o, recall_o, recog_o)
            fp = f"{field_label(p['side_a']['category'])} || {field_label(p['side_b']['category'])}"
            row = {"key": key, "field_pair": fp, "recall_fired": fired, "clean": clean,
                   "rank_llm": r1_llm, "pctile_llm": pct_llm, "rank_lex": r1_lex, "pctile_lex": pct_lex,
                   "pctile_emb": pct_emb, "pctile_bestnull": pct_bestnull,
                   "recog_conf": recog_o.get("confidence"), "recall_conf": recall_o.get("confidence")}
            per_unit.append(row)
            if clean:
                clean_records.append({"pair_id": key, "field_pair": fp,
                                      "rank_llm_pctile": pct_llm, "rank_bestnull_pctile": pct_bestnull,
                                      "rank_lex_pctile": pct_lex, "rank_emb_pctile": pct_emb})
    # anchor recall@10 (reproduction)
    anchor_recall10 = (sum(1 for r in anchor_ranks.values() if r <= 10) / len(anchor_ranks)) \
        if anchor_ranks else None

    n_clean = len(clean_records)
    n_pairs_scored = sum(1 for r in per_unit)
    field_pairs = {r["field_pair"] for r in clean_records}
    summary = {"units_spec": units_spec, "arm": arm, "n_pairs_scored": n_pairs_scored,
               "n_clean": n_clean, "clean_yield": (n_clean / n_pairs_scored) if n_pairs_scored else None,
               "n_distinct_field_pairs_clean": len(field_pairs),
               "anchor_ranks": anchor_ranks, "anchor_recall@10": anchor_recall10,
               "n_missing": len(missing), "missing_examples": missing[:10]}
    tag = units_spec.replace(":", "")
    json.dump({"summary": summary, "per_unit": per_unit},
              open(os.path.join(DATA, f"rs22_score_{tag}_{arm}.json"), "w"), indent=1, ensure_ascii=False)
    json.dump({"records": clean_records},
              open(os.path.join(DATA, f"rs22_clean_records_{tag}_{arm}.json"), "w"), indent=1, ensure_ascii=False)
    print(f"=== score[{units_spec},{arm}] ===")
    print(f"  pairs scored={n_pairs_scored}  CLEAN={n_clean}  yield={summary['clean_yield']}")
    print(f"  distinct clean field-pairs={len(field_pairs)}  missing={len(missing)}")
    if anchor_recall10 is not None:
        print(f"  anchor recall@10 (reproduction, want [0.50,0.90])={anchor_recall10:.2f}  ranks={anchor_ranks}")
    return summary


# ============================ main ============================
def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    e = sub.add_parser("emit-inputs"); e.add_argument("--units", required=True)
    j = sub.add_parser("emit-judge-inputs"); j.add_argument("--units", required=True)
    j.add_argument("--arm", default="claude")
    c = sub.add_parser("emit-claude-args"); c.add_argument("--instr", required=True)
    c.add_argument("--units", required=True)
    m = sub.add_parser("run-mistral"); m.add_argument("--instr", required=True)
    m.add_argument("--units", required=True); m.add_argument("--model", default="mistral-large-latest")
    m.add_argument("--workers", type=int, default=4)
    s = sub.add_parser("score"); s.add_argument("--units", required=True); s.add_argument("--arm", default="claude")
    a = ap.parse_args()
    if a.cmd == "emit-inputs":
        emit_inputs(a.units)
    elif a.cmd == "emit-judge-inputs":
        emit_judge_inputs(a.units, a.arm)
    elif a.cmd == "emit-claude-args":
        emit_claude_args(a.instr, a.units)
    elif a.cmd == "run-mistral":
        run_mistral(a.instr, a.units, a.model, a.workers)
    elif a.cmd == "score":
        score(a.units, a.arm)


if __name__ == "__main__":
    main()
