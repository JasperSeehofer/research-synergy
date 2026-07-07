#!/usr/bin/env python3
"""EXP-RS-17 (Phase 36) — build the MODERN held-out MVP corpus (the leakage-controlled bar).

Composition (LOCKED, CONVENTIONS.md C-24):
  * 12 endpoints = the 6 evaluable modern_lbd_pairs (m01,m02,m03,m04,m06,m08) x side_a/side_b.
  * 24 distractors sampled DETERMINISTICALLY from post-2018 arXiv:
        take each endpoint's primary arXiv category (dedup, preserve first-seen order); for each
        category query `cat:<C> AND submittedDate:[20180101 TO *]` sorted by submittedDate ASCENDING;
        round-robin across the categories, taking the earliest-submitted ids not already in the
        corpus (lexicographic arxiv_id tie-break); skip ids lacking a >=200-char abstract; stop at 24.
        No RNG.
  * Abstracts + primary categories via the arXiv API (3 s rate-limit).

Output: data/modern_mvp_corpus.json  (same schema family as C-14 mvp_corpus.json).
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
MODERN = os.path.join(DATA, "modern_lbd_pairs.json")
OUT = os.path.join(DATA, "modern_mvp_corpus.json")

MIN_ABSTRACT_CHARS = 200
N_DISTRACTORS = 24
ARXIV_API = "http://export.arxiv.org/api/query"
ATOM = "{http://www.w3.org/2005/Atom}"
ARXIV_NS = "{http://arxiv.org/schemas/atom}"
RATE_LIMIT_S = 3.0
UA = "resyn-exp-rs-17 (mailto:jasperseehofermusic@gmail.com)"


def norm_id(raw: str) -> str:
    s = raw.strip()
    s = re.sub(r"^https?://arxiv\.org/abs/", "", s)
    s = re.sub(r"v\d+$", "", s)
    return s


def _get(url: str, timeout: int = 30) -> str:
    req = urllib.request.Request(url, headers={"User-Agent": UA})
    with urllib.request.urlopen(req, timeout=timeout) as r:
        return r.read().decode("utf-8", errors="replace")


def _parse_entries(xml: str):
    """Return list of {arxiv_id, title, abstract, primary_category, submitted}."""
    out = []
    root = ET.fromstring(xml)
    for entry in root.findall(f"{ATOM}entry"):
        eid = entry.find(f"{ATOM}id")
        title = entry.find(f"{ATOM}title")
        summ = entry.find(f"{ATOM}summary")
        pub = entry.find(f"{ATOM}published")
        prim = entry.find(f"{ARXIV_NS}primary_category")
        if eid is None or summ is None:
            continue
        out.append({
            "arxiv_id": norm_id(eid.text or ""),
            "title": " ".join((title.text or "").split()) if title is not None else "",
            "abstract": " ".join((summ.text or "").split()),
            "primary_category": (prim.get("term") if prim is not None else None),
            "submitted": (pub.text or "") if pub is not None else "",
        })
    return out


def fetch_by_ids(ids):
    if not ids:
        return {}
    q = "id_list=" + ",".join(urllib.parse.quote(i, safe="/") for i in ids)
    url = f"{ARXIV_API}?{q}&max_results={len(ids)}"
    try:
        return {e["arxiv_id"]: e for e in _parse_entries(_get(url))}
    except Exception as e:  # noqa: BLE001
        print(f"  [arxiv] id batch error: {type(e).__name__}: {e}", file=sys.stderr)
        return {}


def fetch_category_post2018(cat: str, max_results: int = 100):
    """Earliest-submitted post-2018 papers in a category, submittedDate ASC."""
    sq = f"cat:{cat} AND submittedDate:[201801010000 TO 209912312359]"
    url = (f"{ARXIV_API}?search_query={urllib.parse.quote(sq)}"
           f"&sortBy=submittedDate&sortOrder=ascending&max_results={max_results}")
    try:
        entries = _parse_entries(_get(url))
        # deterministic: submittedDate asc (API order), lexicographic arxiv_id tie-break
        entries.sort(key=lambda e: (e["submitted"], e["arxiv_id"]))
        return entries
    except Exception as e:  # noqa: BLE001
        print(f"  [arxiv] cat {cat} error: {type(e).__name__}: {e}", file=sys.stderr)
        return []


def load_endpoints():
    m = json.load(open(MODERN))
    ev = set(m["evaluable_pairs"])
    rows = []
    for p in m["pairs"]:
        if p["id"] not in ev:
            continue
        for side in ("side_a", "side_b"):
            s = p[side]
            rows.append({
                "arxiv_id": norm_id(s["arxiv_id"]),
                "curated_title": s["title"],
                "pair_id": p["id"],
                "side": side,
                "domain": s["domain"],
            })
    return rows


def main():
    endpoints = load_endpoints()
    endpoint_ids = {e["arxiv_id"] for e in endpoints}
    print(f"Endpoints: {len(endpoints)} ({len(endpoint_ids)} unique ids)")

    # --- endpoint metadata (abstract + primary category) ---
    meta = fetch_by_ids([e["arxiv_id"] for e in endpoints])
    time.sleep(RATE_LIMIT_S)

    corpus = []
    cat_order = []  # first-seen order of primary categories for round-robin
    for e in endpoints:
        info = meta.get(e["arxiv_id"], {})
        cat = info.get("primary_category")
        if cat and cat not in cat_order:
            cat_order.append(cat)
        abstract = info.get("abstract", "")
        if len(abstract) < MIN_ABSTRACT_CHARS:
            print(f"  !! endpoint {e['arxiv_id']} ({e['pair_id']}/{e['side']}) "
                  f"abstract len={len(abstract)}", file=sys.stderr)
        corpus.append({
            "arxiv_id": e["arxiv_id"],
            "title": info.get("title") or e["curated_title"],
            "abstract": abstract,
            "primary_category": cat,
            "is_benchmark": True,
            "pair_id": e["pair_id"],
            "side": e["side"],
            "domain": e["domain"],
            "source": "arxiv",
        })
    print(f"Primary categories (round-robin order): {cat_order}")

    # --- distractor pools per category (post-2018, submittedDate asc) ---
    pools = {}
    for cat in cat_order:
        pools[cat] = fetch_category_post2018(cat, max_results=100)
        time.sleep(RATE_LIMIT_S)
        print(f"  pool[{cat}]: {len(pools[cat])} candidates")

    # --- deterministic round-robin selection ---
    chosen = []
    chosen_ids = set()
    ptr = {cat: 0 for cat in cat_order}
    exhausted = set()
    while len(chosen) < N_DISTRACTORS and len(exhausted) < len(cat_order):
        for cat in cat_order:
            if len(chosen) >= N_DISTRACTORS:
                break
            if cat in exhausted:
                continue
            pool = pools[cat]
            # advance ptr to next usable candidate
            picked = None
            while ptr[cat] < len(pool):
                cand = pool[ptr[cat]]
                ptr[cat] += 1
                cid = cand["arxiv_id"]
                if (cid in endpoint_ids or cid in chosen_ids
                        or len(cand["abstract"]) < MIN_ABSTRACT_CHARS):
                    continue
                picked = cand
                break
            if picked is None:
                exhausted.add(cat)
                continue
            chosen_ids.add(picked["arxiv_id"])
            chosen.append({
                "arxiv_id": picked["arxiv_id"],
                "title": picked["title"],
                "abstract": picked["abstract"],
                "primary_category": picked["primary_category"] or cat,
                "is_benchmark": False,
                "pair_id": None,
                "side": None,
                "domain": None,
                "source": "arxiv",
            })
    corpus.extend(chosen)

    n_bench_ok = sum(1 for c in corpus if c["is_benchmark"]
                     and len(c["abstract"]) >= MIN_ABSTRACT_CHARS)
    out = {
        "experiment": "EXP-RS-17",
        "schema_version": "v0.1",
        "convention": "C-24",
        "min_abstract_chars": MIN_ABSTRACT_CHARS,
        "n_benchmark": sum(1 for c in corpus if c["is_benchmark"]),
        "n_benchmark_with_abstract": n_bench_ok,
        "n_distractors": len(chosen),
        "n_total": len(corpus),
        "source_pairs": "modern_lbd_pairs.json (evaluable: m01,m02,m03,m04,m06,m08)",
        "distractor_categories": cat_order,
        "papers": corpus,
    }
    json.dump(out, open(OUT, "w"), indent=1, ensure_ascii=False)
    print(f"\nWrote {OUT}")
    print(f"  benchmark: {out['n_benchmark']} ({n_bench_ok} with abstract), "
          f"distractors: {out['n_distractors']}, total: {out['n_total']}")
    if n_bench_ok < 12 or len(chosen) < N_DISTRACTORS:
        print("  WARNING: incomplete corpus — inspect stderr misses above.", file=sys.stderr)


if __name__ == "__main__":
    main()
