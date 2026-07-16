#!/usr/bin/env python3
"""EXP-RS-21 (Phase 40) — Dense embedding substrate scorer (per-corpus, per-model).

Method (LOCKED, CONVENTIONS.md C-41..C-45): replace the C-17 TF-IDF vector with a dense embedding
    emb(p) = L2norm(model.encode(TEXT_m(p)))
    score(q,c) = cos(emb(q), emb(c)) = dot (L2-normalized)
Retrieval metric = C-19 (sme_lite.eval_direction / rank_candidates), forward primary, ties ->
candidate arxiv_id lexicographic. Every arm is apples-to-apples with the C-17 lexical null and every
prior EXP-RS-16..20 arm (identical corpus / pairs / metric / tie-break).

Per-model text (C-41 revised + C-43 table):
  bge/gte/mistral: TEXT = title + '. ' + abstract
  specter:         native title + [SEP] + abstract via tokenize(text=title, text_pair=abstract)  (MUST-7)

Encoding (C-42 revised):
  HEADLINE M1 = bge, SYMMETRIC (no instruction) — canonical for a symmetric abstract<->abstract analogy
  task; the BGE retrieval-instruction ("Represent this sentence for searching relevant passages: ")
  ASYMMETRIC arm is a DESCRIPTIVE ablation only (--directional), NOT the headline (MUST-8).

Frozen model set (C-43): M1 bge-large-en-v1.5 (headline, MTEB-retrieval designation), M2 specter2
(scientific/citation, P3 contrast), M4 gte-large (2nd general open, robustness) — the REPRODUCIBLE
LOCAL class↑ vote. M3 mistral-embed (API) is DESCRIPTIVE-only, EXCLUDED from the class↑/KILL vote (MUST-4).

This module computes per-corpus per-model numbers ONLY (ranks, recall, mrr, revision, seq-len /
truncation, objective P5 cards, per-model strict-P1 flags, liveness assertions). The cross-corpus
DECISION (class↑, headlinePass, the total-function verdict) is embed_gate.py.

Usage:
  python embed_score.py --corpus data/mvp_corpus.json --pairs data/feynman_10pair_papers.json \
      --models bge,gte,specter,mistral --out data/embed_results_feynman.json
"""
import argparse
import json
import os
import sys
import urllib.request
import urllib.error

import numpy as np

from sme_lite import eval_direction  # C-19 harness, verbatim
from methmesh_score import load_pairs

HERE = os.path.dirname(os.path.abspath(__file__))
DATA = os.path.join(HERE, "data")

NULL_MISSED_FEYNMAN = {"pair01-ising-opinion", "pair04-percolation-epidemics",
                       "pair06-turing-spatial-economy"}  # C-17 null misses these (verified)
BGE_QUERY_INSTRUCTION = "Represent this sentence for searching relevant passages: "
P5_MARGIN_DELTA = 0.02  # C-45 objective cross-domain margin, frozen before scoring
MAX_SEQ = {"bge": 512, "gte": 512, "specter": 512, "mistral": 8192}  # pinned (SHOULD-4)

MODEL_SPECS = {
    "bge":     ("BAAI/bge-large-en-v1.5", "st"),
    "gte":     ("thenlper/gte-large",     "st"),
    "specter": ("allenai/specter2_base",  "specter"),
    "mistral": ("mistral-embed",          "api"),
}
LOCAL_CLASS = ["bge", "specter", "gte"]  # reproducible class↑ vote (C-43); mistral excluded


def field(p):
    """Frozen 'field' for the objective P5 same-field / cross-domain rule (works both corpora)."""
    for k in ("community_id", "domain", "primary_category"):
        if p.get(k) not in (None, ""):
            return f"{k}:{p[k]}"
    aid = p.get("arxiv_id", "")
    return "cat:" + (aid.split("/")[0] if "/" in aid else aid.split(".")[0])


def st_text(p):
    return f"{(p.get('title') or '').strip()}. {(p.get('abstract') or '').strip()}".strip()


def l2norm_rows(mat):
    mat = np.asarray(mat, dtype=np.float64)
    n = np.linalg.norm(mat, axis=1, keepdims=True)
    n[n == 0] = 1.0
    return mat / n


def _assert_live(vecs, n_expected, tag):
    """Liveness assertions (MUST-9): shape + non-degenerate embeddings."""
    assert vecs.shape[0] == n_expected, f"{tag}: got {vecs.shape[0]} vecs, expected {n_expected}"
    assert np.isfinite(vecs).all(), f"{tag}: non-finite embeddings"
    # not all rows identical (a broken encoder often returns a constant)
    assert np.max(np.std(vecs, axis=0)) > 1e-6, f"{tag}: degenerate (all-identical) embeddings"


def _hf_revision(hf_id):
    try:
        import huggingface_hub as hh
        return hh.model_info(hf_id).sha or "unknown"
    except Exception:  # noqa: BLE001
        return "unknown"


# ------------------------------- encoders -------------------------------
def encode_st(hf_id, papers, model_key, query_instruction=None):
    import torch
    from sentence_transformers import SentenceTransformer
    torch.manual_seed(0)
    model = SentenceTransformer(hf_id, device="cpu")
    model.max_seq_length = MAX_SEQ[model_key]
    model.eval()
    texts = [st_text(p) for p in papers]
    inp = [(query_instruction + t) if query_instruction else t for t in texts]
    trunc = _truncation_report(model.tokenizer, texts, MAX_SEQ[model_key])
    with torch.no_grad():
        vecs = model.encode(inp, normalize_embeddings=True, convert_to_numpy=True,
                            batch_size=16, show_progress_bar=False)
    return l2norm_rows(vecs), _hf_revision(hf_id), MAX_SEQ[model_key], trunc


def encode_specter(hf_id, papers, model_key, query_instruction=None):
    """SPECTER2 (base + proximity adapter) or specter-v1; native title[SEP]abstract + CLS (MUST-7)."""
    import torch
    from transformers import AutoTokenizer
    titles = [(p.get("title") or "").strip() for p in papers]
    absts = [(p.get("abstract") or "").strip() for p in papers]
    variant = "specter2_base+proximity"
    try:
        from adapters import AutoAdapterModel
        tok = AutoTokenizer.from_pretrained(hf_id)
        model = AutoAdapterModel.from_pretrained(hf_id)
        model.load_adapter("allenai/specter2", source="hf", load_as="proximity", set_active=True)
        model.set_active_adapters("proximity")  # explicit — the P3 contrast MUST use the proximity adapter
        assert "proximity" in str(model.active_adapters), "specter2 proximity adapter not active"
    except Exception as e:  # noqa: BLE001 — objective within-spec fallback (nice-to-have #1)
        print(f"  [specter2 adapter unavailable: {type(e).__name__}: {e}; -> specter-v1]",
              file=sys.stderr)
        from transformers import AutoModel
        hf_id = "allenai/specter"
        tok = AutoTokenizer.from_pretrained(hf_id)
        model = AutoModel.from_pretrained(hf_id)
        variant = "specter-v1-FALLBACK"
    torch.manual_seed(0)
    model.eval()
    trunc = _truncation_report(tok, [f"{t} {tok.sep_token} {a}" for t, a in zip(titles, absts)],
                               MAX_SEQ[model_key])
    out = []
    with torch.no_grad():
        for i in range(0, len(papers), 16):
            enc = tok(titles[i:i + 16], absts[i:i + 16], padding=True, truncation=True,
                      return_tensors="pt", max_length=MAX_SEQ[model_key])
            emb = model(**enc).last_hidden_state[:, 0, :]  # CLS
            out.append(emb.cpu().numpy())
    return l2norm_rows(np.vstack(out)), f"{variant}@{_hf_revision(hf_id)}", MAX_SEQ[model_key], trunc


def encode_mistral(model_id, papers, model_key, query_instruction=None):
    key = os.environ.get("MISTRAL_API_KEY")
    if not key:
        raise RuntimeError("MISTRAL_API_KEY not set")
    texts = [st_text(p) for p in papers]
    out = []
    for i in range(0, len(texts), 32):
        batch = texts[i:i + 32]
        body = json.dumps({"model": model_id, "input": batch}).encode()
        req = urllib.request.Request(
            "https://api.mistral.ai/v1/embeddings", data=body,
            headers={"Authorization": f"Bearer {key}", "Content-Type": "application/json"})
        got = None
        for attempt in range(5):
            try:
                r = json.load(urllib.request.urlopen(req, timeout=90))
                got = [d["embedding"] for d in r["data"]]
                break
            except (urllib.error.HTTPError, urllib.error.URLError):
                if attempt == 4:
                    raise
                import time
                time.sleep(2 * (attempt + 1))
        assert got is not None and len(got) == len(batch), \
            f"mistral batch misalignment: {None if got is None else len(got)} != {len(batch)}"
        out.extend(got)
    vecs = l2norm_rows(out)  # idempotent belt-and-suspenders vs API-side norm
    return vecs, f"mistral-embed-api@{model_id}", MAX_SEQ[model_key], {"note": "api tokens not counted"}


def _truncation_report(tokenizer, texts, max_len):
    over = []
    try:
        for i, t in enumerate(texts):
            n = len(tokenizer.encode(t, add_special_tokens=True))
            if n > max_len:
                over.append((i, n))
    except Exception:  # noqa: BLE001
        return {"note": "token count unavailable"}
    return {"n_truncated": len(over), "over": over[:20], "max_len": max_len}


ENCODERS = {"st": encode_st, "specter": encode_specter, "api": encode_mistral}


def get_embeddings(model_key, papers, cache_dir, corpus_name, directional=False):
    hf_id, kind = MODEL_SPECS[model_key]
    tag = f"{model_key}{'_dir' if directional else ''}"
    cache = os.path.join(cache_dir, f"embed_{tag}_{corpus_name}.npz")
    if os.path.exists(cache):
        z = np.load(cache, allow_pickle=True)
        return z["q"], z["p"], str(z["rev"]), int(z["seq"]), json.loads(str(z["trunc"]))
    enc = ENCODERS[kind]
    passage, rev, seq, trunc = enc(hf_id, papers, model_key)
    _assert_live(passage, len(papers), f"{tag}/passage")
    if directional and model_key == "bge":
        query, _, _, _ = enc(hf_id, papers, model_key, query_instruction=BGE_QUERY_INSTRUCTION)
        _assert_live(query, len(papers), f"{tag}/query")
    else:
        query = passage
    os.makedirs(cache_dir, exist_ok=True)
    np.savez(cache, q=query, p=passage, rev=rev, seq=seq, trunc=json.dumps(trunc))
    return query, passage, rev, seq, trunc


def objective_cards(pairs, papers, sc):
    """C-45 objective P5 card: cross-domain (metadata-diff) + cosine-margin over same-field median."""
    pmap = {p["arxiv_id"]: p for p in papers}
    all_ids = [p["arxiv_id"] for p in papers]
    cards = {}
    for p in pairs:
        q, t = p["side_a"], p["side_b"]
        fq = field(pmap[q])
        sims = {c: sc(q, c) for c in all_ids if c != q}
        true_sim = sims.get(t)
        same_field = [v for c, v in sims.items() if c != t and field(pmap[c]) == fq]
        sf_median = float(np.median(same_field)) if same_field else 0.0
        cross_domain = field(pmap[t]) != fq
        margin_ok = (true_sim is not None) and (true_sim > sf_median + P5_MARGIN_DELTA)
        cards[p["id"]] = {
            "true_pair_cosine": true_sim, "same_field_median": sf_median,
            "cross_domain_metadata": cross_domain, "margin_over_same_field": margin_ok,
            "objective_pass": bool(cross_domain and margin_ok),
        }
    return cards


def random_control(pairs, papers, sc, n=200):
    """Random-pair control (MUST-6): non-benchmark cross-field pairs should FAIL the objective rule."""
    pmap = {p["arxiv_id"]: p for p in papers}
    all_ids = [p["arxiv_id"] for p in papers]
    bench = {x for p in pairs for x in (p["side_a"], p["side_b"])}
    pool = [a for a in all_ids if a not in bench]
    passes = 0
    tried = 0
    # deterministic pseudo-random pairing by lexicographic index stride (no RNG per C-14 discipline)
    for i in range(len(pool)):
        q = pool[i]
        c = pool[(i * 7 + 3) % len(pool)]
        if c == q or field(pmap[c]) == field(pmap[q]):
            continue
        fq = field(pmap[q])
        sims = {x: sc(q, x) for x in all_ids if x != q}
        same_field = [v for x, v in sims.items() if x != c and field(pmap[x]) == fq]
        sf_med = float(np.median(same_field)) if same_field else 0.0
        if sims[c] > sf_med + P5_MARGIN_DELTA:
            passes += 1
        tried += 1
        if tried >= n:
            break
    return {"control_pairs_tried": tried, "control_pass_rate": (passes / tried) if tried else None}


def strict_p1(fwd):
    """Per-model strict-P1 (C-45): beats null (R>0.40) AND pair04 in top10 AND >=1 null-missed."""
    ranks = fwd["ranks"]
    recovered = {pid for pid, r in ranks.items() if r is not None and r <= 10}
    null_missed = sorted(recovered & NULL_MISSED_FEYNMAN)
    pair04 = ranks.get("pair04-percolation-epidemics")
    pair04_top10 = pair04 is not None and pair04 <= 10
    return {
        "recall@10": fwd["recall@10"], "beats_null_0.40": fwd["recall@10"] > 0.40,
        "pair04_rank": pair04, "pair04_top10": pair04_top10,
        "null_missed_recovered": null_missed,
        "strict_P1_pass": fwd["recall@10"] > 0.40 and pair04_top10 and len(null_missed) >= 1,
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--corpus", default=os.path.join(DATA, "mvp_corpus.json"))
    ap.add_argument("--pairs", default=os.path.join(DATA, "feynman_10pair_papers.json"))
    ap.add_argument("--models", default="bge,gte,specter,mistral")
    ap.add_argument("--out", default=os.path.join(DATA, "embed_results_feynman.json"))
    ap.add_argument("--cache-dir", default=os.path.join(DATA, "embed_cache"))
    ap.add_argument("--directional", action="store_true", help="ablation: BGE query instruction ON")
    ap.add_argument("--forward-only", action="store_true")
    args = ap.parse_args()

    corpus = json.load(open(args.corpus))
    papers = corpus["papers"]
    corpus_name = os.path.splitext(os.path.basename(args.corpus))[0]
    all_ids = [p["arxiv_id"] for p in papers]
    present = set(all_ids)
    pairs = load_pairs(args.pairs, present)
    is_feynman = "pair01-ising-opinion" in {p["id"] for p in pairs}
    model_keys = [m.strip() for m in args.models.split(",") if m.strip()]

    results = {"experiment": "EXP-RS-21", "corpus": os.path.basename(args.corpus),
               "corpus_name": corpus_name, "n_corpus": len(all_ids), "n_eval_pairs": len(pairs),
               "pair_ids": [p["id"] for p in pairs], "directional": args.directional,
               "p5_margin_delta": P5_MARGIN_DELTA, "models": {}}

    for mk in model_keys:
        try:
            q, p, rev, seq, trunc = get_embeddings(mk, papers, args.cache_dir, corpus_name,
                                                   directional=args.directional and mk == "bge")
        except Exception as e:  # noqa: BLE001
            results["models"][mk] = {"error": f"{type(e).__name__}: {e}"}
            print(f"[{mk}] ERROR {e}", file=sys.stderr); continue
        eq = {a: q[i] for i, a in enumerate(all_ids)}
        ep = {a: p[i] for i, a in enumerate(all_ids)}

        def sc(a, b, eq=eq, ep=ep):
            return float(np.dot(eq[a], ep[b]))

        fwd = eval_direction(pairs, all_ids, sc, "side_a", "side_b")
        rec = {"revision": rev, "max_seq_length": seq, "truncation": trunc, "forward": fwd}
        if not args.forward_only:
            rv = eval_direction(pairs, all_ids, sc, "side_b", "side_a")
            rec["reverse"] = rv
            rec["both_dir_avg"] = {k: (fwd[k] + rv[k]) / 2
                                   for k in ("recall@1", "recall@5", "recall@10", "mrr")}
        rec["cards"] = objective_cards(pairs, papers, sc)
        rec["random_control"] = random_control(pairs, papers, sc)
        if is_feynman:
            rec["strict_P1"] = strict_p1(fwd)
        results["models"][mk] = rec

    json.dump(results, open(args.out, "w"), indent=1, ensure_ascii=False,
              default=lambda o: float(o) if isinstance(o, np.floating) else o)

    # console summary
    print(f"corpus={corpus_name} N={len(all_ids)} pairs={len(pairs)} models={model_keys} "
          f"directional={args.directional} (feynman={is_feynman})")
    hdr = f"{'model':9} {'dir':8} {'R@1':>5} {'R@5':>5} {'R@10':>5} {'MRR':>6}  strictP1"
    print(hdr); print("-" * len(hdr))
    for mk in model_keys:
        rec = results["models"].get(mk, {})
        if "error" in rec:
            print(f"{mk:9} ERROR: {rec['error']}"); continue
        dirs = ("forward",) if args.forward_only else ("forward", "reverse")
        for d in dirs:
            m = rec.get(d)
            if m:
                sp = rec.get("strict_P1", {}).get("strict_P1_pass", "") if d == "forward" else ""
                print(f"{mk:9} {d:8} {m['recall@1']:5.2f} {m['recall@5']:5.2f} "
                      f"{m['recall@10']:5.2f} {m['mrr']:6.3f}  {sp}")
    print(f"wrote {args.out}")


if __name__ == "__main__":
    main()
