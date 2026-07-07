#!/usr/bin/env python3
"""EXP-RS-16 (Phase 35) — build the MVP corpus for the SME-vs-baseline head-to-head.

Composition (LOCKED, CONVENTIONS.md C-14):
  * 10 benchmark endpoints = the 5 evaluable Feynman pairs (01,03,04,05,06) x side_a/side_b.
  * 26 distractors, sampled DETERMINISTICALLY from research_synergy_bridged_fine_sheaf.json:
        exclude benchmark ids; round-robin over communities by ascending community_id;
        within-community candidates sorted lexicographically; round r = r-th candidate;
        skip ids lacking a >=200-char abstract; stop at 26. No RNG.
  * Abstracts: arXiv API id_list (batched, 3 s rate-limit). OpenAlex title.search fallback.

Output: data/mvp_corpus.json  (schema in main()).
Run AFTER committing this script (commit-before-data discipline).
"""
import json
import os
import re
import sys
import time
import urllib.parse
import urllib.request
import xml.etree.ElementTree as ET

HERE = os.path.dirname(os.path.abspath(__file__))
DATA = os.path.join(HERE, "data")
TESTBED = os.path.join(DATA, "research_synergy_bridged_fine_sheaf.json")
FEYNMAN = os.path.join(DATA, "feynman_10pair_papers.json")
OUT = os.path.join(DATA, "mvp_corpus.json")

MIN_ABSTRACT_CHARS = 200
N_DISTRACTORS = 26
ARXIV_API = "http://export.arxiv.org/api/query"
OPENALEX_API = "https://api.openalex.org/works"
ATOM = "{http://www.w3.org/2005/Atom}"
RATE_LIMIT_S = 3.0
UA = "resyn-exp-rs-16 (mailto:jasperseehofermusic@gmail.com)"


def norm_id(raw: str) -> str:
    """Normalise an arXiv id: strip abs URL prefix and version suffix."""
    s = raw.strip()
    s = re.sub(r"^https?://arxiv\.org/abs/", "", s)
    s = re.sub(r"v\d+$", "", s)
    return s


def _get(url: str, timeout: int = 30) -> str:
    req = urllib.request.Request(url, headers={"User-Agent": UA})
    with urllib.request.urlopen(req, timeout=timeout) as r:
        return r.read().decode("utf-8", errors="replace")


def fetch_arxiv_batch(ids):
    """Fetch {norm_id: {title, abstract}} for a batch of arXiv ids via the API."""
    if not ids:
        return {}
    q = "id_list=" + ",".join(urllib.parse.quote(i, safe="/") for i in ids)
    url = f"{ARXIV_API}?{q}&max_results={len(ids)}"
    out = {}
    try:
        xml = _get(url)
        root = ET.fromstring(xml)
        for entry in root.findall(f"{ATOM}entry"):
            eid = entry.find(f"{ATOM}id")
            title = entry.find(f"{ATOM}title")
            summ = entry.find(f"{ATOM}summary")
            if eid is None or summ is None:
                continue
            nid = norm_id(eid.text or "")
            t = " ".join((title.text or "").split()) if title is not None else ""
            a = " ".join((summ.text or "").split())
            out[nid] = {"title": t, "abstract": a}
    except Exception as e:  # noqa: BLE001
        print(f"  [arxiv] batch error: {type(e).__name__}: {e}", file=sys.stderr)
    return out


def _reconstruct_abstract(inv):
    if not inv:
        return ""
    positions = []
    for word, idxs in inv.items():
        for i in idxs:
            positions.append((i, word))
    positions.sort()
    return " ".join(w for _, w in positions)


def fetch_openalex_by_title(title: str):
    """Fallback: resolve title -> {title, abstract} via OpenAlex title.search."""
    try:
        flt = "title.search:" + title
        url = f"{OPENALEX_API}?filter={urllib.parse.quote(flt)}&per-page=1&mailto=jasperseehofermusic@gmail.com"
        data = json.loads(_get(url))
        results = data.get("results", [])
        if not results:
            return None
        w = results[0]
        abstract = _reconstruct_abstract(w.get("abstract_inverted_index"))
        return {"title": w.get("display_name", title), "abstract": abstract}
    except Exception as e:  # noqa: BLE001
        print(f"  [openalex] error for {title!r}: {type(e).__name__}: {e}", file=sys.stderr)
        return None


def load_benchmark():
    fey = json.load(open(FEYNMAN))
    ev = set(fey["evaluable_pairs"])
    rows = []
    for p in fey["pairs"]:
        pid = p["id"].split("-")[0]
        if pid not in ev:
            continue
        for side in ("side_a", "side_b"):
            s = p[side]
            rows.append({
                "arxiv_id": s["arxiv_id"],
                "curated_title": s["title"],
                "pair_id": p["id"],
                "side": side,
                "domain": s["domain"],
            })
    return rows


def build_distractor_order(benchmark_ids):
    """Deterministic round-robin candidate order across communities (C-14)."""
    tb = json.load(open(TESTBED))
    by_comm = {}
    for n in tb["nodes"]:
        by_comm.setdefault(n["community_id"], []).append(n["id"])
    for cid in by_comm:
        by_comm[cid] = sorted(x for x in by_comm[cid] if x not in benchmark_ids)
    max_round = max((len(v) for v in by_comm.values()), default=0)
    order = []
    for r in range(max_round):
        for cid in sorted(by_comm):
            if r < len(by_comm[cid]):
                order.append((by_comm[cid][r], cid))
    return order  # list of (arxiv_id, community_id) in round-robin order


def main():
    benchmark = load_benchmark()
    benchmark_ids = {b["arxiv_id"] for b in benchmark}
    id2comm = {n["id"]: n["community_id"]
               for n in json.load(open(TESTBED))["nodes"]}

    # --- benchmark abstracts (arXiv, then OpenAlex fallback) ---
    print(f"Fetching {len(benchmark)} benchmark abstracts via arXiv...")
    fetched = fetch_arxiv_batch([b["arxiv_id"] for b in benchmark])
    time.sleep(RATE_LIMIT_S)
    corpus = []
    for b in benchmark:
        info = fetched.get(b["arxiv_id"])
        source = "arxiv"
        if not info or len(info.get("abstract", "")) < MIN_ABSTRACT_CHARS:
            alt = fetch_openalex_by_title(b["curated_title"])
            time.sleep(1.0)
            if alt and len(alt.get("abstract", "")) >= MIN_ABSTRACT_CHARS:
                info, source = alt, "openalex"
        if not info or len(info.get("abstract", "")) < MIN_ABSTRACT_CHARS:
            print(f"  !! benchmark {b['arxiv_id']} ({b['pair_id']}/{b['side']}) "
                  f"missing abstract (len={len(info.get('abstract','')) if info else 0})",
                  file=sys.stderr)
        corpus.append({
            "arxiv_id": b["arxiv_id"],
            "title": (info or {}).get("title") or b["curated_title"],
            "abstract": (info or {}).get("abstract", ""),
            "community_id": id2comm.get(b["arxiv_id"]),
            "is_benchmark": True,
            "pair_id": b["pair_id"],
            "side": b["side"],
            "domain": b["domain"],
            "source": source,
        })

    # --- distractors (deterministic round-robin, stop at 26 valid) ---
    order = build_distractor_order(benchmark_ids)
    print(f"Distractor candidate pool: {len(order)} ids; selecting {N_DISTRACTORS} valid...")
    chosen = []
    i = 0
    while len(chosen) < N_DISTRACTORS and i < len(order):
        batch = order[i:i + 30]
        i += 30
        got = fetch_arxiv_batch([a for a, _ in batch])
        time.sleep(RATE_LIMIT_S)
        for aid, cid in batch:
            if len(chosen) >= N_DISTRACTORS:
                break
            info = got.get(aid)
            if info and len(info.get("abstract", "")) >= MIN_ABSTRACT_CHARS:
                chosen.append({
                    "arxiv_id": aid,
                    "title": info["title"],
                    "abstract": info["abstract"],
                    "community_id": cid,
                    "is_benchmark": False,
                    "pair_id": None,
                    "side": None,
                    "domain": None,
                    "source": "arxiv",
                })
    corpus.extend(chosen)

    n_bench_ok = sum(1 for c in corpus if c["is_benchmark"] and len(c["abstract"]) >= MIN_ABSTRACT_CHARS)
    out = {
        "experiment": "EXP-RS-16",
        "schema_version": "v0.1",
        "convention": "C-14",
        "min_abstract_chars": MIN_ABSTRACT_CHARS,
        "n_benchmark": sum(1 for c in corpus if c["is_benchmark"]),
        "n_benchmark_with_abstract": n_bench_ok,
        "n_distractors": len(chosen),
        "n_total": len(corpus),
        "source_testbed": "research_synergy_bridged_fine_sheaf.json",
        "papers": corpus,
    }
    json.dump(out, open(OUT, "w"), indent=1, ensure_ascii=False)
    print(f"\nWrote {OUT}")
    print(f"  benchmark: {out['n_benchmark']} ({n_bench_ok} with abstract), "
          f"distractors: {out['n_distractors']}, total: {out['n_total']}")
    if n_bench_ok < 10 or len(chosen) < N_DISTRACTORS:
        print("  WARNING: incomplete corpus — inspect stderr misses above.", file=sys.stderr)


if __name__ == "__main__":
    main()
