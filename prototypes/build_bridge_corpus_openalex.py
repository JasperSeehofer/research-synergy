#!/usr/bin/env python3
"""
Phase 2 (EXP-RS-13 exploratory) — build a benchmark-centric, bridge-CONTAINING corpus
via a targeted OpenAlex fetch, to re-test dynamical LBD after EXP-RS-12 showed the
data-kuramoto corpus lacks 3/4 benchmark bridges.

Strategy: seed from the Feynman-pair ENDPOINTS, pull each endpoint's citation
neighborhood (backward references + forward citations). The cross-domain bridge papers
(e.g. "Generalized Lotka-Volterra models of stock markets") appear as forward citations
of the physics endpoints and create the missing inter-community edges. Rate-limit-free
(OpenAlex polite pool w/ key). NO giant field dump — stays benchmark-centric to avoid the
global-top-10 metric dilution.

NOTE: this is an EXPLORATORY Python reconstruction of the corpus (OpenAlex edges + networkx
Louvain + manual c-TF-IDF), NOT the resyn Rust pipeline (C-2/C-4/C-5). If it shows signal,
formalize through resyn `bulk-ingest`/`analyze`/`export-louvain-graph`. Output schema matches
`export-louvain-graph` so kuramoto_lbd_v05 consumes it unchanged.

Output: data/research_synergy_bridged.json
"""
import json, os, re, time, urllib.request, urllib.parse, math
from collections import defaultdict

API = "https://api.openalex.org/works"
SELECT = "id,doi,title,display_name,publication_date,referenced_works,locations,abstract_inverted_index"
FWD_CAP = 300          # max forward-citations fetched per endpoint (keeps corpus benchmark-scale)
UA = "resyn-research/0.1 (mailto:jasperseehofermusic@gmail.com)"

def load_key():
    k = os.environ.get("OPENALEX_API_KEY")
    if k: return k
    try:
        for line in open("/home/jasper/Repositories/research-synergy/.env"):
            s = line.strip()
            if s.startswith("OPENALEX_API_KEY=") and not s.startswith("#"):
                v = s.split("=", 1)[1].strip().strip('"').strip("'")
                if v and "your" not in v.lower(): return v
    except FileNotFoundError:
        pass
    return None  # unauthenticated polite pool (mailto UA) works fine for our volume

KEY = load_key()
print(f"OpenAlex auth: {'keyed' if KEY else 'unauthenticated (polite pool, mailto UA)'}")

def _get(url):
    headers = {"User-Agent": UA}
    if KEY: headers["Authorization"] = f"Bearer {KEY}"
    req = urllib.request.Request(url, headers=headers)
    for attempt in range(4):
        try:
            with urllib.request.urlopen(req, timeout=60) as r:
                return json.loads(r.read().decode())
        except Exception as e:
            if attempt == 3: raise
            time.sleep(1.5 * (attempt + 1))

def fetch_filter(filt, cap=None):
    """Paginate a filter, return list of work dicts (up to cap)."""
    out, cursor = [], "*"
    while True:
        url = f"{API}?filter={urllib.parse.quote(filt, safe=':|/.,')}&per-page=200&select={SELECT}&cursor={cursor}"
        page = _get(url)
        res = page.get("results", [])
        out.extend(res)
        cursor = page.get("meta", {}).get("next_cursor")
        if not res or not cursor or (cap and len(out) >= cap):
            break
        time.sleep(0.1)
    return out[:cap] if cap else out

ARXIV_DOI = "10.48550/arxiv."
def arxiv_id_of(work):
    doi = (work.get("doi") or "").lower()
    p = doi.find(ARXIV_DOI)
    if p != -1:
        return doi[p+len(ARXIV_DOI):]
    for loc in work.get("locations") or []:
        url = (loc.get("landing_page_url") or "").lower()
        m = re.search(r"arxiv\.org/abs/([^/?#\s]+(?:/[^/?#\s]+)?)", url)
        if m: return m.group(1)
    return None

def abstract_of(work):
    aii = work.get("abstract_inverted_index")
    if not aii: return ""
    pairs = []
    for tok, poss in aii.items():
        for p in poss: pairs.append((p, tok))
    pairs.sort()
    return " ".join(t for _, t in pairs)

def short_id(oa_url):  # https://openalex.org/W123 -> W123
    return oa_url.rsplit("/", 1)[-1]

# ---- 1. endpoints from the benchmark ----
fp = json.load(open("data/feynman_10pair_papers.json"))
endpoints = {}   # arxiv_id -> (pair_id, side)
for pr in fp["pairs"]:
    for side in ("side_a", "side_b"):
        aid = pr[side].get("arxiv_id")
        if aid: endpoints[aid] = (pr["id"], side)
print(f"Benchmark endpoints with arXiv IDs: {len(endpoints)}")

# ---- 2. resolve endpoints -> OpenAlex works (DOI, then title fallback) ----
def resolve_endpoint(aid, title):
    # try arXiv DOI (new + old format) — works for post-2022 papers
    for doi in (f"10.48550/arxiv.{aid}", f"10.48550/arXiv.{aid}"):
        r = fetch_filter(f"doi:{doi}", cap=1)
        if r and arxiv_id_of(r[0]) == aid: return r[0]
    # fallback: title search (fetch_filter does the single URL-encode — do NOT pre-quote)
    if title:
        r = fetch_filter(f"title.search:{title[:90]}", cap=8)
        for w in r:                       # exact arXiv-location match is ground truth
            if arxiv_id_of(w) == aid: return w
        # accept the top title hit only if it is itself an arXiv paper (avoid non-arXiv noise)
        for w in r:
            if arxiv_id_of(w): return w
    return None

works = {}   # arxiv_id -> work dict
oa2arxiv = {}  # openalex short id -> arxiv_id
resolved_endpoints = {}
for aid, (pid, side) in endpoints.items():
    title = None
    for pr in fp["pairs"]:
        if pr["id"] == pid: title = pr[side].get("title")
    w = resolve_endpoint(aid, title)
    if w:
        rid = aid   # key by the benchmark's canonical arXiv id so scoring finds it
        works[rid] = w; oa2arxiv[short_id(w["id"])] = rid
        resolved_endpoints[aid] = short_id(w["id"])
        print(f"  resolved {aid:22} -> {short_id(w['id'])} (arxiv_of={arxiv_id_of(w)})")
    else:
        print(f"  UNRESOLVED {aid}")
print(f"Resolved {len(resolved_endpoints)}/{len(endpoints)} endpoints")

# ---- 3. expand: forward citations + backward refs of each resolved endpoint ----
seed_oa = list(resolved_endpoints.values())
# forward: papers citing any endpoint (bridges live here). Batch OR-filter in groups.
def batched(xs, n):
    for i in range(0, len(xs), n): yield xs[i:i+n]

fwd = []
for grp in batched(seed_oa, 25):
    fwd += fetch_filter("cites:" + "|".join(grp), cap=FWD_CAP*len(grp))
print(f"Forward-citation works fetched: {len(fwd)}")

# backward: referenced works of endpoints (resolve their arxiv ids)
ref_oa = set()
for oa in seed_oa:
    w = works.get(oa2arxiv[oa])
    for rw in (w.get("referenced_works") or []):
        ref_oa.add(short_id(rw))
bwd = []
for grp in batched(list(ref_oa), 50):
    try:
        bwd += fetch_filter("ids.openalex:" + "|".join("https://openalex.org/"+g for g in grp))
    except Exception as e:
        print(f"  [warn] backward batch failed ({e}); continuing (forward citations carry the bridges)")
print(f"Backward-reference works fetched: {len(bwd)}")

# ---- 4. assemble node set (arXiv-resolvable only) ----
for w in fwd + bwd:
    aid = arxiv_id_of(w)
    if aid:
        works.setdefault(aid, w)
        oa2arxiv[short_id(w["id"])] = aid
print(f"Total arXiv-resolvable works (nodes): {len(works)}")

# ---- 5. citation edges among the node set ----
node_ids = set(works.keys())
edges = set()
for aid, w in works.items():
    for rw in (w.get("referenced_works") or []):
        tgt = oa2arxiv.get(short_id(rw))
        if tgt and tgt in node_ids and tgt != aid:
            edges.add((aid, tgt))
print(f"Citation edges (arXiv-to-arXiv, within node set): {len(edges)}")

# ---- 6. Louvain communities on the undirected citation graph ----
import networkx as nx
G = nx.Graph()
G.add_nodes_from(node_ids)
G.add_edges_from((a, b) for a, b in edges)
# keep giant component for a well-posed substrate (C-13 spirit), but export full too
comms = nx.community.louvain_communities(G, seed=42, resolution=1.0)
comm_of = {}
for ci, cset in enumerate(comms):
    for n in cset: comm_of[n] = ci
print(f"Louvain: {len(comms)} communities on {G.number_of_nodes()} nodes / {G.number_of_edges()} edges")

# ---- 7. c-TF-IDF (top-50 terms/node) from title+abstract ----
STOP = set("the a an and or of to in for on with is are as by we this that from at be it its our their can using use based model models method methods results result study paper approach show new via between into over under also more most such which these those than then when where while these each other both".split())
def toks(text):
    return [t for t in re.findall(r"[a-z]{3,}", text.lower()) if t not in STOP]
docs = {aid: toks((w.get("title") or w.get("display_name") or "") + " " + abstract_of(w)) for aid, w in works.items()}
df = defaultdict(int)
for aid, ts in docs.items():
    for t in set(ts): df[t] += 1
Ndoc = max(len(docs), 1)
def tfidf_vec(ts):
    tf = defaultdict(int)
    for t in ts: tf[t] += 1
    n = max(len(ts), 1)
    v = {t: (c/n) * math.log(Ndoc/(1+df[t])) for t, c in tf.items()}
    return sorted(v.items(), key=lambda x: -x[1])[:50]

# ---- 8. export in export-louvain-graph schema ----
nodes_out = [{"id": aid, "community_id": comm_of.get(aid, 999999),
              "tfidf_vec": [[t, round(s, 6)] for t, s in tfidf_vec(docs[aid])]}
             for aid in node_ids if aid in comm_of]
edges_out = [{"src": a, "dst": b, "weight": 1.0} for a, b in edges
             if a in comm_of and b in comm_of]
out = {
    "louvain_params": {"seed": 42, "resolution": 1.0, "min_community_size": 1},
    "corpus_fingerprint": "openalex-bridge-exploratory",
    "nodes": nodes_out,
    "communities": [],
    "edges": edges_out,
}
os.makedirs("data", exist_ok=True)
json.dump(out, open("data/research_synergy_bridged.json", "w"))
print(f"\nWrote data/research_synergy_bridged.json: {len(nodes_out)} nodes, {len(edges_out)} edges")

# ---- 9. benchmark coverage report ----
EVAL = set(fp["evaluable_pairs"])
comm = {n["id"]: n["community_id"] for n in nodes_out}
idset = set(comm)
# inter-community edge counts
cedge = defaultdict(int)
for e in edges_out:
    ca, cb = comm[e["src"]], comm[e["dst"]]
    if ca != cb: cedge[frozenset((ca, cb))] += 1
print("\nBenchmark evaluable pairs in bridged corpus:")
n_eval = 0; n_bridged = 0
for pr in fp["pairs"]:
    k = pr["id"][:6]
    if k not in EVAL: continue
    a, b = pr["side_a"]["arxiv_id"], pr["side_b"]["arxiv_id"]
    pa, pb = a in idset, b in idset
    ca, cb = comm.get(a), comm.get(b)
    de = cedge.get(frozenset((ca, cb)), 0) if (pa and pb and ca != cb) else 0
    ok = pa and pb and ca != cb
    if ok: n_eval += 1
    if de > 0: n_bridged += 1
    print(f"  {pr['id'][:24]:24} A={'Y' if pa else 'n'}(c{ca}) B={'Y' if pb else 'n'}(c{cb})  inter-comm edges={de}  {'EVAL' if ok else '-'}")
print(f"\nn_eval={n_eval}  pairs-with-bridge-edges={n_bridged}  (was 1/4 in data-kuramoto)")
