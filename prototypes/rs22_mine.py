#!/usr/bin/env python3
"""EXP-RS-22 (Phase 41) — mechanical, deterministic cross-field analogy-pair miner.

Faithful implementation of the FROZEN protocol `rs22_mining_protocol.md`
(sha256 recorded in the snapshot). Memory-blind; no RNG; validity judgment + memory probing are
DEFERRED (§6). Every parameter below cites its protocol section.

Feasibility note (within-protocol): OpenAlex is used as the PRIMARY source because one
`abstract.search` query returns {abstract_inverted_index, referenced_works, primary_topic} together,
and a bridge's references are batch-fetched via `ids.openalex:` (the §2.2 arXiv path is the spec's
primary but OpenAlex reconstruction is an explicit allowed fallback, §2.2.3/§3.1); arXiv supplies the
final `primary_category` for the two emitted sides. TOPCAT falls back to the OpenAlex primary_topic
level-0 field id when an arXiv category is unavailable (§2.2.3).

Usage (validation slice):   python rs22_mine.py --limit-patterns 2 --cap 25 --max-bridges 8 --out-suffix _probe
Usage (full committed run):  python rs22_mine.py --full
"""
import argparse
import json
import os
import re
import time
import urllib.parse
import urllib.request
import xml.etree.ElementTree as ET
from collections import Counter

HERE = os.path.dirname(os.path.abspath(__file__))
DATA = os.path.join(HERE, "data")

# ---- frozen constants (rs22_mining_protocol.md) ----
PATTERNS = [  # §1.2 P01..P14 (order load-bearing for snippet selection §2.4)
    "mathematically equivalent to", "formally equivalent to", "formally identical to",
    "the same universality class as", "belongs to the same universality class",
    "obeys the same equations as", "governed by the same equations as", "maps onto",
    "can be mapped onto", "isomorphic to", "the same mathematical structure as",
    "in direct analogy with", "borrowed from", "is analogous to",
]
N_CAP_PER_PATTERN = 500          # §1.4
BLOCK_SIZE = 60                  # §4.3
N_COMMIT = 420                   # §4.3  (B1..B7)
DATE_FROM = "2007-01-01"         # §1.4
ARXIV_RATE_S = 3.0               # §1.1
FUNC_WORDS = {"the", "of", "and", "to", "in", "a", "is", "for", "we", "that",  # §3.2
              "with", "are", "this", "as", "by"}
OA_MAILTO = "resyn-research/0.1 (mailto:jasperseehofermusic@gmail.com)"  # §1.1

_WS = re.compile(r"\s+")
_last_arxiv = [0.0]


def norm(s):  # §1.2 whitespace-normalize + lowercase
    return _WS.sub(" ", (s or "")).strip().lower()


def has_pattern(abstract):  # §1.2 authoritative exact-substring re-check
    n = norm(abstract)
    return any(p in n for p in PATTERNS)


def first_pattern_snippet(abstract):  # §2.4
    n = norm(abstract)
    best = None
    for pi, p in enumerate(PATTERNS):
        idx = n.find(p)
        if idx != -1 and (best is None or idx < best[1]):
            best = (pi, idx)
    if best is None:
        return None, None
    _, idx = best
    start = n.rfind(". ", 0, idx)
    start = 0 if start == -1 else start + 2
    end = n.find(". ", idx)
    end = min(end + 1 if end != -1 else len(n), start + 240)
    return best[0], n[start:end].strip()


def topcat(cat):  # §0
    return cat.split(".")[0] if cat and "." in cat else (cat or "")


def english_ok(abstract):  # §3.2
    n = norm(abstract)
    nonspace = [c for c in n if not c.isspace()]
    if not nonspace:
        return False
    alpha = sum(1 for c in nonspace if "a" <= c <= "z")
    if alpha / len(nonspace) < 0.90:
        return False
    words = set(re.findall(r"[a-z]+", n))
    return len(words & FUNC_WORDS) >= 5


# ---------------- OpenAlex ----------------
OA = "https://api.openalex.org/works"
OA_SELECT = ("id,doi,title,display_name,publication_date,referenced_works,locations,"
             "abstract_inverted_index,primary_topic,language,primary_location")


_last_oa = [0.0]


def _get(url):
    """OpenAlex GET with polite rate-limit + 429-aware backoff; returns {} on persistent failure (never raises)."""
    dt = time.monotonic() - _last_oa[0]
    if dt < 0.15:
        time.sleep(0.15 - dt)
    req = urllib.request.Request(url, headers={"User-Agent": OA_MAILTO})
    key = os.environ.get("OPENALEX_API_KEY")
    if key:
        req.add_header("Authorization", f"Bearer {key}")
    for attempt in range(6):
        try:
            r = json.load(urllib.request.urlopen(req, timeout=60))
            _last_oa[0] = time.monotonic()
            return r
        except urllib.error.HTTPError as e:
            if attempt == 5:
                break
            time.sleep((12 if e.code == 429 else 2) * (attempt + 1))
        except Exception:  # noqa: BLE001
            if attempt == 5:
                break
            time.sleep(2 * (attempt + 1))
    _last_oa[0] = time.monotonic()
    return {}


def oa_search(phrase, cap):
    """abstract.search for a phrase → list of work dicts, date asc, id asc (§1.3/§1.4)."""
    out, cursor = [], "*"
    filt = (f"abstract.search:{phrase},language:en,from_publication_date:{DATE_FROM},"
            f"type:article")
    while cursor and len(out) < cap:
        url = (f"{OA}?filter={urllib.parse.quote(filt)}&select={OA_SELECT}"
               f"&per-page=200&sort=publication_date:asc&cursor={cursor}")
        j = _get(url)
        out.extend(j.get("results", []))
        cursor = j.get("meta", {}).get("next_cursor")
    return out[:cap]


def oa_by_ids(oa_ids):
    """Batch-fetch works by OpenAlex short ids (§2.2)."""
    works = {}
    for i in range(0, len(oa_ids), 50):
        grp = oa_ids[i:i + 50]
        filt = "openalex:" + "|".join(grp)
        url = f"{OA}?filter={urllib.parse.quote(filt)}&select={OA_SELECT}&per-page=50"
        for w in _get(url).get("results", []):
            works[short_id(w["id"])] = w
    return works


def short_id(u):
    return u.rsplit("/", 1)[-1] if u else u


def arxiv_id_of(work):  # mirror build_bridge_corpus_openalex.py §2.1
    doi = (work.get("doi") or "").lower()
    m = re.search(r"10\.48550/arxiv\.(.+)$", doi)
    if m:
        return strip_v(m.group(1))
    for loc in work.get("locations") or []:
        u = (loc.get("landing_page_url") or "")
        m = re.search(r"arxiv\.org/abs/([^v?#]+)", u)
        if m:
            return strip_v(m.group(1))
    return None


def strip_v(a):
    return re.sub(r"v\d+$", "", a)


def oa_abstract(work):  # abstract_inverted_index → text
    aii = work.get("abstract_inverted_index")
    if not aii:
        return ""
    pos = {}
    for word, idxs in aii.items():
        for i in idxs:
            pos[i] = word
    return " ".join(pos[i] for i in sorted(pos))


def oa_topcat(work):  # §2.2.3 fallback: primary_topic level-0 field id
    pt = work.get("primary_topic") or {}
    fld = (pt.get("field") or {})
    return "oa:" + short_id(fld.get("id", "")) if fld.get("id") else ""


# ---------------- arXiv (final category for the two sides) ----------------
def arxiv_meta(ids):
    """Fetch {arxiv_id: (title, abstract, primary_category)} via arXiv API, 3 s rate-limit (§1.1)."""
    out = {}
    for i in range(0, len(ids), 50):
        grp = ids[i:i + 50]
        dt = time.monotonic() - _last_arxiv[0]
        if dt < ARXIV_RATE_S:
            time.sleep(ARXIV_RATE_S - dt)
        url = ("http://export.arxiv.org/api/query?id_list=" + ",".join(grp)
               + "&max_results=50")
        try:
            raw = urllib.request.urlopen(urllib.request.Request(
                url, headers={"User-Agent": OA_MAILTO}), timeout=60).read()
        except Exception:  # noqa: BLE001
            _last_arxiv[0] = time.monotonic()
            continue
        _last_arxiv[0] = time.monotonic()
        ns = {"a": "http://www.w3.org/2005/Atom", "arxiv": "http://arxiv.org/schemas/atom"}
        for e in ET.fromstring(raw).findall("a:entry", ns):
            idu = (e.findtext("a:id", "", ns) or "")
            m = re.search(r"arxiv\.org/abs/([^v]+)", idu)
            if not m:
                continue
            aid = strip_v(m.group(1))
            pc = e.find("arxiv:primary_category", ns)
            out[aid] = (
                _WS.sub(" ", e.findtext("a:title", "", ns)).strip(),
                _WS.sub(" ", e.findtext("a:summary", "", ns)).strip(),
                pc.get("term") if pc is not None else None,
            )
    return out


def arxiv_search(phrase, cap):
    """§1.3 S1: arXiv abs phrase search → [(arxiv_id, abstract, category)], date-filtered, paginated."""
    out, start = [], 0
    ns = {"a": "http://www.w3.org/2005/Atom", "arxiv": "http://arxiv.org/schemas/atom"}
    qp = urllib.parse.quote(phrase)
    while len(out) < cap and start < cap:
        dt = time.monotonic() - _last_arxiv[0]
        if dt < ARXIV_RATE_S:
            time.sleep(ARXIV_RATE_S - dt)
        sq = f'abs:%22{qp}%22+AND+submittedDate:[200701010000+TO+299901012359]'
        url = (f"http://export.arxiv.org/api/query?search_query={sq}"
               f"&start={start}&max_results=100&sortBy=submittedDate&sortOrder=ascending")
        try:
            raw = urllib.request.urlopen(urllib.request.Request(
                url, headers={"User-Agent": OA_MAILTO}), timeout=60).read()
        except Exception:  # noqa: BLE001
            _last_arxiv[0] = time.monotonic(); break
        _last_arxiv[0] = time.monotonic()
        entries = ET.fromstring(raw).findall("a:entry", ns)
        if not entries:
            break
        for e in entries:
            m = re.search(r"arxiv\.org/abs/([^v]+)", e.findtext("a:id", "", ns) or "")
            if not m:
                continue
            pc = e.find("arxiv:primary_category", ns)
            out.append((strip_v(m.group(1)),
                        _WS.sub(" ", e.findtext("a:summary", "", ns)).strip(),
                        pc.get("term") if pc is not None else None))
        start += 100
    return out[:cap]


def oa_work_by_arxiv(aid):
    """Fetch the OpenAlex work for an arXiv id (via the 10.48550 DOI) to get referenced_works."""
    doi = urllib.parse.quote(f"10.48550/arxiv.{aid}")
    r = _get(f"{OA}?filter=doi:{doi}&select={OA_SELECT}&per-page=1").get("results", [])
    return r[0] if r else None


_last_s2 = [0.0]


def s2_refs(aid):
    """§2.2.1 fallback: reference arXiv ids of a bridge via Semantic Scholar (returns arXiv ids directly)."""
    dt = time.monotonic() - _last_s2[0]
    if dt < 1.1:
        time.sleep(1.1 - dt)
    key = os.environ.get("S2_API_KEY")
    hdr = {"User-Agent": OA_MAILTO}
    if key:
        hdr["x-api-key"] = key
    url = (f"https://api.semanticscholar.org/graph/v1/paper/arXiv:{aid}/references"
           f"?fields=externalIds&limit=500")
    out = []
    for attempt in range(4):
        try:
            j = json.load(urllib.request.urlopen(urllib.request.Request(url, headers=hdr), timeout=60))
            for r in j.get("data", []):
                ext = ((r.get("citedPaper") or {}).get("externalIds") or {})
                if ext.get("ArXiv"):
                    out.append(strip_v(ext["ArXiv"]))
            break
        except urllib.error.HTTPError as e:
            if e.code == 429 and attempt < 3:
                time.sleep(2 * (attempt + 1)); continue
            break
        except Exception:  # noqa: BLE001
            break
    _last_s2[0] = time.monotonic()
    return out


def ref_arxiv_ids(b_aid):
    """Reference arXiv ids of a bridge (§2.2). Protocol §2.2.1 names OpenAlex referenced_works primary
    with a Semantic Scholar fallback; in practice OpenAlex lacks the 10.48550/arXiv DOI for many (esp.
    pre-2015) papers AND 429-rate-limits the long full run, so the sanctioned S2 fallback — which returns
    reference arXiv ids DIRECTLY in one call — is the operative fast path (documented deviation)."""
    seen, out = set(), []
    for a in s2_refs(b_aid):
        if a not in seen:
            seen.add(a); out.append(a)
    return out


# ---------------- pipeline ----------------
def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--full", action="store_true", help="full committed run (N_COMMIT=420)")
    ap.add_argument("--limit-patterns", type=int, default=None, help="validation: only first N patterns")
    ap.add_argument("--cap", type=int, default=N_CAP_PER_PATTERN, help="validation: per-pattern cap")
    ap.add_argument("--max-bridges", type=int, default=None, help="validation: process only first K bridges")
    ap.add_argument("--out-suffix", default="", help="validation: suffix for output files")
    args = ap.parse_args()

    excl = set(l.strip() for l in open(os.path.join(DATA, "rs22_exclusion_ids.txt")) if l.strip())
    proto_sha = _sha(os.path.join(HERE, "rs22_mining_protocol.md"))
    patterns = PATTERNS[: args.limit_patterns] if args.limit_patterns else PATTERNS

    # ---- §1 bridge discovery: arXiv abs phrase search (all hits arXiv-native), authoritative re-check §1.2 ----
    raw_hits = {"arxiv": {}}
    bridge_meta = {}   # arxiv_id -> (abstract, category)
    for p in patterns:
        hits = arxiv_search(p, args.cap)
        raw_hits["arxiv"][p] = [h[0] for h in hits]
        for aid, ab, cat in hits:
            if has_pattern(ab):                    # §1.2 authoritative exact substring re-check
                bridge_meta.setdefault(aid, (ab, cat))
    # §4.1 canonical order: sort by bridge arxiv_id asc
    bridges_sorted = sorted(bridge_meta.items())   # (bridge_arxiv_id, (abstract, cat))
    if args.max_bridges:
        bridges_sorted = bridges_sorted[: args.max_bridges]

    # ---- §2/§3 bridge → pair (references via OpenAlex) ----
    emitted, used_ids, seen_pairs, rejections = [], set(), set(), []
    def _emit_snapshot_and_out():  # incremental checkpoint (survives a duration-kill)
        _write_outputs(args, proto_sha, bridges_sorted, raw_hits, emitted, rejections)

    for bi, (b_aid, (b_abstract, b_cat)) in enumerate(bridges_sorted):
        try:
            ref_ids = [a for a in ref_arxiv_ids(b_aid) if a != b_aid]
            if len(ref_ids) < 2:
                rejections.append({"bridge": b_aid, "reason": "no_or_too_few_arxiv_refs"}); continue
            meta = arxiv_meta(ref_ids)   # {arxiv_id: (title, abstract, primary_category)} (§2.2.3)
            R = []  # (arxiv_id, TOPCAT, (title, abstract, category)) — clean domain refs only (§2.2.4)
            for aid, (title, ab, cat) in meta.items():
                if not (title and ab and cat):
                    continue
                if len(norm(ab)) < 200 or not english_ok(ab) or has_pattern(ab):
                    continue
                R.append((aid, topcat(cat), (title, ab, cat)))
            if len({r[0] for r in R}) < 2:
                rejections.append({"bridge": b_aid, "reason": "too_few_clean_refs"}); continue
            R.sort(key=lambda x: x[0])  # §2.2.5 arxiv_id asc
            cnt = Counter(tc for _, tc, _ in R)                   # §2.3 two most-cited distinct fields
            C = sorted(cnt, key=lambda t: (-cnt[t], t))
            if len(C) < 2:
                rejections.append({"bridge": b_aid, "reason": "not_cross_field"}); continue
            c1, c2 = C[0], C[1]
            paper_c1 = next(a for a, tc, _ in R if tc == c1)
            paper_c2 = next(a for a, tc, _ in R if tc == c2)
            s_lo, s_hi = sorted([paper_c1, paper_c2])             # §2.3.5 side_a = smaller arxiv_id
            metamap = {a: m for a, _, m in R}
            pair_sides, ok = {}, True
            for role, aid in (("side_a", s_lo), ("side_b", s_hi)):
                title, ab, cat = metamap[aid]
                if not (title and ab and len(norm(ab)) >= 200 and english_ok(ab) and not has_pattern(ab)):
                    ok = False; break
                pair_sides[role] = {"arxiv_id": aid, "title": title, "abstract": ab, "category": cat}
            if not ok:
                rejections.append({"bridge": b_aid, "reason": "side_filter_fail"}); continue
            if pair_sides["side_a"]["category"] == pair_sides["side_b"]["category"]:  # §3.1 cross-field
                rejections.append({"bridge": b_aid, "reason": "same_field_final"}); continue
            if s_lo in excl or s_hi in excl:
                rejections.append({"bridge": b_aid, "reason": "excluded_prior_benchmark"}); continue
            if s_lo in used_ids or s_hi in used_ids or frozenset([s_lo, s_hi]) in seen_pairs:
                rejections.append({"bridge": b_aid, "reason": "dup"}); continue
            _, snippet = first_pattern_snippet(b_abstract)
            rank = len(emitted)
            emitted.append({
                "pair_id": "rs22-%06d" % rank, "side_a": pair_sides["side_a"],
                "side_b": pair_sides["side_b"],
                "bridge_paper": {"arxiv_id": b_aid, "asserted_analogy_snippet": snippet},
                "block_id": (rank // BLOCK_SIZE) + 1,
            })
            used_ids.update([s_lo, s_hi]); seen_pairs.add(frozenset([s_lo, s_hi]))
        except Exception as e:  # noqa: BLE001 — one bad bridge never crashes the mine
            rejections.append({"bridge": b_aid, "reason": "exc_" + type(e).__name__}); continue
        if len(emitted) and len(emitted) % 20 == 0:
            _emit_snapshot_and_out()
        if bi % 50 == 0:
            print(f"  [{bi}/{len(bridges_sorted)} bridges → {len(emitted)} pairs]", flush=True)
        if args.full and len(emitted) >= N_COMMIT:
            break

    # ---- §4.2/§5 final snapshot + output ----
    out_path, snap_path = _write_outputs(args, proto_sha, bridges_sorted, raw_hits, emitted, rejections)
    print(f"bridges considered={len(bridges_sorted)}  emitted pairs={len(emitted)}  "
          f"rejections={len(rejections)}")
    for e in emitted[:5]:
        print(f"  {e['pair_id']}: {e['side_a']['arxiv_id']} [{e['side_a']['category']}] <-> "
              f"{e['side_b']['arxiv_id']} [{e['side_b']['category']}]  bridge={e['bridge_paper']['arxiv_id']}")
    print(f"wrote {out_path} + {snap_path}")


def _write_outputs(args, proto_sha, bridges_sorted, raw_hits, emitted, rejections):
    suffix = args.out_suffix
    snap = {"protocol_sha256": proto_sha, "constants_sha256":
            "af5ee11c7828fbec0bf9eb6a9520e82cb57dbebfcacf4f3290b108c3a8643c33",
            "snapshot_date_utc": "live", "n_bridges_considered": len(bridges_sorted),
            "raw_hits_counts": {p: len(v) for p, v in raw_hits["arxiv"].items()},
            "bridge_candidates_sorted": [a for a, _ in bridges_sorted],
            "n_emitted": len(emitted), "rejections": rejections[:400]}
    snap_path = os.path.join(DATA, f"rs22_mining_snapshot{suffix}.json")
    json.dump(snap, open(snap_path, "w"), indent=1, ensure_ascii=False)
    out = {"protocol_sha256": proto_sha, "snapshot_ref": os.path.basename(snap_path),
           "block_size": BLOCK_SIZE, "n_blocks_committed": (len(emitted) + BLOCK_SIZE - 1) // BLOCK_SIZE,
           "n_pairs": len(emitted), "validation_slice": not args.full, "pairs": emitted}
    out_path = os.path.join(DATA, f"rs22_mined_pairs{suffix}.json")
    json.dump(out, open(out_path, "w"), indent=1, ensure_ascii=False)
    return out_path, snap_path


def _sha(path):
    import hashlib
    return hashlib.sha256(open(path, "rb").read()).hexdigest()


if __name__ == "__main__":
    main()
