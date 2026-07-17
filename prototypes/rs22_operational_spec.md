# RS-22 Instrument-Harness Operational Spec (DETERMINISTIC, BLIND)

**Experiment:** EXP-RS-22 (Phase 41) — instrument-harness stage.
**Author standpoint:** no-stake, blind research-tooling engineer. Authored with **no access**
to any answer key, ground-truth, bridge-name list, verification file, THREAD, or prior EXP-RS
result. Every rule below is derived from **general principles + the public arXiv category
taxonomy** and the frozen prompt set only. See the attestation at the end.

**Purpose.** Turn the frozen mined corpus (`prototypes/data/rs22_mined_pairs.json`, one record
per pair — schema below) plus the six frozen prompts into a fully deterministic, RNG-free
scoring harness: field labelling, the recognition menu, the K=50 retrieval pool, the judge
reference, the self-field guard, and the clean-stratum predicate. No researcher discretion
remains at execution time.

---

## 0. Inputs, constants, and global conventions

### 0.1 Frozen inputs
- `rs22_mined_pairs.json` — `{ …, pairs: [ PAIR, … ] }`, each `PAIR`:
  ```
  { pair_id: str,                                  # e.g. "rs22-000123"
    side_a: {arxiv_id, title, abstract, category}, # category = arXiv primary_category
    side_b: {arxiv_id, title, abstract, category},
    bridge_paper: {arxiv_id, asserted_analogy_snippet},
    block_id: number }
  ```
- `rs22_constants.json` (sha256 `af5ee11c7828fbec0bf9eb6a9520e82cb57dbebfcacf4f3290b108c3a8643c33`):
  `K_pool_size = 50` is the one constant this spec consumes directly.
- Frozen prompts: `rs22_probe_recall.md`, `rs22_probe_recognition.md`,
  `rs22_probe_familiarity.md`, `rs22_probe_openbook.md`, `rs22_judge_semantic.md`, and the new
  `rs22_probe_mechanism.md` (authored alongside this spec).

### 0.2 LEAKAGE GUARD (hard rule)
`bridge_paper.arxiv_id` and `bridge_paper.asserted_analogy_snippet` are **never** read by any
rule in this spec. They are bridge-side evidence and a potential answer-key leak. The harness
touches only `pair_id`, `side_a{…}`, `side_b{…}`, `block_id`.

### 0.3 Constants chosen in this spec (with justification in situ)
| Symbol | Value | Where |
|---|---|---|
| `K_pool` | `50` (= `K_pool_size`) | §3 retrieval pool |
| `K_menu` | `8` | §2 recognition menu |
| `SELF_FIELD_OVERLAP` | `0.67` | §5 self-field guard |
| `RECOG_CONF_MAX` | `0.5` | §6 clean predicate |
| `US` | `"\x1f"` (ASCII Unit Separator) | §0.4 hashing |

### 0.4 The only source of "randomness": a stable hash (no RNG)
All ordering, selection, and tie-breaking use SHA-256 hex digests compared **lexicographically**
as ordinary strings — never an RNG, never wall-clock, never set-iteration order. Two
**domain-separated** hashes are used so that *which* items get selected is statistically
independent of *where* the correct item lands (removing positional bias):

```
def H(domain, pair_id, x):          # x is a label or an arxiv_id (str)
    import hashlib
    msg = (domain + US + pair_id + US + x).encode("utf-8")
    return hashlib.sha256(msg).hexdigest()      # 64 lowercase hex chars

def Hsel(pair_id, x): return H("rs22-select", pair_id, x)   # selection
def Hord(pair_id, x): return H("rs22-order",  pair_id, x)   # presentation order
```

Every "sort by hash" below means: `sorted(items, key=lambda x: (Hxxx(pair_id, x), x))` — the raw
string `x` is the documented final tie-break (impossible in practice, ids/labels are unique, but
it makes the order total).

### 0.5 arXiv id normalisation (repo convention)
```
import re
def normalize_id(x):                 # strip trailing version suffix, per utils::strip_version_suffix
    return re.sub(r"v\d+$", "", x.strip())
```
All id comparisons, dedup, and pool membership use `normalize_id`.

### 0.6 Determinism modulo the pinned model
The pinned model `M0` (the single model id frozen for this study; not named here, and not looked
up from THREAD) is always called at **temperature 0 / greedy**, one probe per fresh session with
zero shared history (C-46 MF5), and every `(prompt_sha, input)` → output is cached under
`prototypes/data/rs22_probe_cache/` keyed by a hash of the input. All *harness* logic
(labelling, selection, ordering, repair, scoring, predicates) is pure and RNG-free and
reproduces **bit-for-bit** from the frozen corpus + cached model outputs. This is the same
"deterministic modulo the pinned model" stance the mining protocol takes.

---

## 1. `field_label(category)` — deterministic field labelling

`field_label` maps an arXiv `primary_category` string to a concise, human-readable, lowercase
field-label. It is a **pure function of public arXiv taxonomy** — no corpus statistics, no
answer key. Algorithm:

```
def field_label(category):
    c = category.strip().lower()
    c = re.sub(r"[^a-z0-9.\-]", "", c)          # keep only taxonomy chars
    if c in SUBCAT_LABELS:                        # 1) exact full-category hit
        return SUBCAT_LABELS[c]
    topcat, _, subtag = c.partition(".")          # 2) archive-level
    if topcat in ARCHIVE_LABELS:
        base = ARCHIVE_LABELS[topcat]
        return f"{base} ({prettify_tag(subtag)})" if subtag else base
    return prettify_tag(c)                         # 3) unknown archive -> prettified fallback

def prettify_tag(t):
    return " ".join(t.replace("-", " ").replace(".", " ").split()).strip().lower()
```

Fallback (steps 2–3) is fully deterministic for **any** category string, listed or not, so the
function is total. Two distinct categories can share a label only when they denote the same
field (intended); the mining `cross-field` filter guarantees `TOPCAT(side_a) != TOPCAT(side_b)`,
so `field_label(side_a.category) != field_label(side_b.category)` in practice.

### 1.1 `ARCHIVE_LABELS` — every top-level arXiv archive (fallback base)
```
astro-ph  -> "astrophysics"                     math      -> "mathematics"
cond-mat  -> "condensed matter physics"         math-ph   -> "mathematical physics"
cs        -> "computer science"                 nlin      -> "nonlinear dynamics"
econ      -> "economics"                        nucl-ex   -> "nuclear physics (experiment)"
eess      -> "electrical eng. and systems sci." nucl-th   -> "nuclear theory"
gr-qc     -> "general relativity and gravitation" physics -> "physics"
hep-ex    -> "high-energy particle physics (experiment)"  q-bio -> "quantitative biology"
hep-lat   -> "lattice field theory"             q-fin     -> "quantitative finance"
hep-ph    -> "high-energy particle phenomenology" quant-ph -> "quantum physics"
hep-th    -> "high-energy theoretical physics"  stat      -> "statistics"
```

### 1.2 `SUBCAT_LABELS` — common sub-categories (exact full-category keys)
```
# cond-mat
cond-mat.stat-mech -> "statistical mechanics"        cond-mat.dis-nn    -> "disordered systems and neural networks"
cond-mat.mes-hall  -> "mesoscopic and nanoscale physics" cond-mat.mtrl-sci -> "materials science"
cond-mat.soft      -> "soft condensed matter"        cond-mat.str-el    -> "strongly correlated electrons"
cond-mat.supr-con  -> "superconductivity"            cond-mat.quant-gas -> "quantum gases"
cond-mat.other     -> "condensed matter physics"
# physics
physics.optics   -> "optics"                    physics.flu-dyn -> "fluid dynamics"
physics.soc-ph   -> "physics of social systems" physics.bio-ph  -> "biological physics"
physics.chem-ph  -> "chemical physics"          physics.plasm-ph-> "plasma physics"
physics.geo-ph   -> "geophysics"                physics.ao-ph   -> "atmospheric and oceanic physics"
physics.atom-ph  -> "atomic physics"            physics.comp-ph -> "computational physics"
physics.data-an  -> "data analysis and statistics" physics.class-ph -> "classical physics"
physics.med-ph   -> "medical physics"           physics.app-ph  -> "applied physics"
physics.acc-ph   -> "accelerator physics"       physics.space-ph-> "space physics"
physics.ins-det  -> "instrumentation and detectors" physics.atm-clus -> "atomic and molecular clusters"
# nlin
nlin.CD -> "chaotic dynamics"                   nlin.PS -> "pattern formation and solitons"
nlin.AO -> "adaptation and self-organizing systems" nlin.SI -> "exactly solvable and integrable systems"
nlin.CG -> "cellular automata and lattice gases"
# q-bio
q-bio.PE -> "population biology and evolution"  q-bio.NC -> "neuroscience (neurons and cognition)"
q-bio.BM -> "biomolecules"                      q-bio.MN -> "molecular networks"
q-bio.QM -> "quantitative methods in biology"   q-bio.CB -> "cell behavior"
q-bio.GN -> "genomics"                          q-bio.SC -> "subcellular processes"
q-bio.TO -> "tissues and organs"
# q-fin
q-fin.ST -> "statistical finance / econophysics" q-fin.MF -> "mathematical finance"
q-fin.PR -> "derivative pricing"                q-fin.RM -> "risk management"
q-fin.PM -> "portfolio management"              q-fin.TR -> "trading and market microstructure"
q-fin.CP -> "computational finance"             q-fin.EC -> "financial economics"
q-fin.GN -> "general finance"
# stat
stat.ML -> "machine learning (statistics)"      stat.ME -> "statistical methodology"
stat.TH -> "statistics theory"                  stat.AP -> "applied statistics"
stat.CO -> "computational statistics"
# cs
cs.LG -> "machine learning"                      cs.AI -> "artificial intelligence"
cs.CV -> "computer vision"                       cs.CL -> "computational linguistics / NLP"
cs.NE -> "neural and evolutionary computing"     cs.IT -> "information theory"
cs.DS -> "data structures and algorithms"        cs.SI -> "social and information networks"
cs.GT -> "algorithmic game theory"               cs.CC -> "computational complexity"
cs.RO -> "robotics"                              cs.DC -> "distributed and parallel computing"
cs.SY -> "systems and control"                   cs.NI -> "networking and internet architecture"
# math
math.PR -> "probability theory"                  math.ST -> "mathematical statistics"
math.OC -> "optimization and control"            math.DS -> "dynamical systems"
math.QA -> "quantum algebra"                      math.AG -> "algebraic geometry"
math.CO -> "combinatorics"                        math.NA -> "numerical analysis"
math.AP -> "analysis of PDEs"                     math.NT -> "number theory"
math.DG -> "differential geometry"                math.GT -> "geometric topology"
math.RT -> "representation theory"                math.FA -> "functional analysis"
math.MP -> "mathematical physics"                 math.GR -> "group theory"
math.RA -> "rings and algebras"                   math.CA -> "classical analysis (real/complex)"
math.SG -> "symplectic geometry"
# eess / econ / astro-ph
eess.SP -> "signal processing"                    eess.SY -> "systems and control"
eess.IV -> "image and video processing"           eess.AS -> "audio and speech processing"
econ.EM -> "econometrics"                          econ.TH -> "economic theory"
econ.GN -> "general economics"
astro-ph.CO -> "cosmology"                         astro-ph.GA -> "astrophysics of galaxies"
astro-ph.HE -> "high-energy astrophysics"          astro-ph.SR -> "solar and stellar astrophysics"
astro-ph.EP -> "earth and planetary astrophysics"  astro-ph.IM -> "astronomical instrumentation and methods"
```

Any category not in `SUBCAT_LABELS` is handled by the archive+prettified-subtag fallback (e.g. a
future `cond-mat.newtag` → `"condensed matter physics (newtag)"`, an unknown `zz.yy` →
`"zz yy"`). Deterministic and total.

---

## 2. Recognition `field_options` menu

Given a `PAIR` and the frozen corpus, build the list of `K_menu = 8` field-label strings shown to
`rs22_probe_recognition.md`.

**Why `K_menu = 8`.** Chance accuracy `1/8 = 12.5%` — low enough that a correct recognition
carries signal, yet a bare forced-choice menu of 8 stays short and answerable without a reasoning
scaffold (matching the probe's "just select" instruction). 8 is the task's recommended value and
gives 7 distractors, enough to be non-trivial while remaining well within the pool of distinct
corpus field-labels.

```
def corpus_field_labels(corpus):                       # computed once, frozen
    L = set()
    for p in corpus.pairs:
        L.add(field_label(p.side_a.category))
        L.add(field_label(p.side_b.category))
    return sorted(L)                                    # lexicographic, deterministic

ARCHIVE_FALLBACK = sorted(set(ARCHIVE_LABELS.values())) # frozen pad list (lexicographic)

def field_options(pair, corpus):
    correct = field_label(pair.side_b.category)         # MUST appear exactly once
    a_label = field_label(pair.side_a.category)
    D = [lab for lab in corpus_field_labels(corpus)
             if lab != correct and lab != a_label]      # distractor pool (excl. both sides)
    D_sorted = sorted(D, key=lambda lab: (Hsel(pair.pair_id, lab), lab))
    distractors = D_sorted[:(K_menu - 1)]               # first 7 by selection hash
    # underfill: pad from frozen archive list if corpus has < 7 distinct distractors
    if len(distractors) < K_menu - 1:
        for lab in ARCHIVE_FALLBACK:
            if len(distractors) >= K_menu - 1: break
            if lab == correct or lab == a_label or lab in distractors: continue
            distractors.append(lab)
    menu = distractors + [correct]                      # <= 8 labels, correct exactly once
    menu = sorted(menu, key=lambda lab: (Hord(pair.pair_id, lab), lab))  # position not fixed
    return menu
```

**Guarantees.** (a) `correct` is excluded from `D` and the pad loop, so it appears **exactly
once**. (b) distractors are distinct (`D` set-derived; pad checks membership) and exclude both
`field_label(side_a.category)` and `field_label(side_b.category)`. (c) selection uses `Hsel`,
final order uses `Hord` — independent, so the correct option lands at an **unpredictable index**
per pair with no positional bias. (d) **More distinct fields than needed:** take the 7 lowest
`Hsel`. **Fewer than needed:** use all of `D`, then pad from `ARCHIVE_FALLBACK` (lexicographic)
skipping `correct`/`a_label`/duplicates; only if the union `D ∪ ARCHIVE_FALLBACK` still yields
< 7 distractors (impossible for real arXiv data — ≥ 20 archive labels alone) does the menu shrink
below 8, which is logged as `menu_underfilled`.

---

## 3. K=50 retrieval pool

Query = `side_a`. Build a pool of exactly `K_pool = 50` candidate papers to be ranked.

### 3.1 Canonical paper table (built once, frozen)
```
def build_papers(corpus):
    papers = {}                                          # insertion-ordered dict
    for p in sorted(corpus.pairs, key=lambda p: p.pair_id):     # pair_id ascending
        for side in (p.side_a, p.side_b):                # side_a before side_b
            pid = normalize_id(side.arxiv_id)
            if pid not in papers:                        # first occurrence wins (dedup by id)
                papers[pid] = {"arxiv_id": pid, "title": side.title,
                               "abstract": side.abstract, "category": side.category}
    return papers
```

### 3.2 Pool construction
```
def retrieval_pool(pair, papers):
    a_id = normalize_id(pair.side_a.arxiv_id)
    b_id = normalize_id(pair.side_b.arxiv_id)            # the TARGET, present in `papers`
    cand = [pid for pid in papers                        # OTHER corpus papers, excl. both sides
                if pid != a_id and pid != b_id]
    cand_sorted = sorted(cand, key=lambda pid: (Hsel(pair.pair_id, pid), pid))
    distractors = cand_sorted[:(K_pool - 1)]             # first 49 by selection hash
    pool_ids = distractors + [b_id]                      # target included exactly once -> 50
    K = len(pool_ids)                                    # 50 normally
    presented = sorted(pool_ids, key=lambda pid: (Hord(pair.pair_id, pid), pid))  # order not fixed
    return presented, K
```

**Guarantees.** `side_b` is included exactly once; `side_a` and `side_b` are excluded from the
distractor set; dedup is by `normalize_id` (no duplicate arxiv_id in `papers`, so none in the
pool). Selection (`Hsel`) is independent of presentation order (`Hord`), so the target's position
is **not fixed**. **Fewer than 49 distinct distractors available:** include all of them, set
`K = len(pool_ids) < 50`, flag the pair `pool_underfilled`, and use this actual `K` in
`pctile_rank`. (For the committed 420-pair corpus, `|papers| ≈ 800` ≫ 50, so this branch is
defensive only; the study's pinned `K_pool_size = 50` holds for every real pair.)

### 3.3 The retrieval call
Present to `M0` (temperature 0): the query `side_a {title, abstract}` and the `presented` list of
candidates, each candidate as `{arxiv_id, title, abstract}` in `presented` order. Ask for a
**full ranking best→worst** — a JSON array of all `K` candidate `arxiv_id`s, most-analogous
first. (The candidate is identified to the model by its `arxiv_id`; the model returns arxiv_ids.)

### 3.4 Deterministic repair + percentile rank
```
def rank_side_b(pair, presented, K, model_ranked_ids):
    pool = set(presented)
    seen, cleaned = set(), []
    for rid in model_ranked_ids:                         # keep valid, first-seen only
        nid = normalize_id(rid)
        if nid in pool and nid not in seen:
            cleaned.append(nid); seen.add(nid)
    missing = sorted(pid for pid in pool if pid not in seen)   # lexicographic ascending
    full = cleaned + missing                             # exact permutation of the K pool ids
    assert len(full) == K and set(full) == pool
    b_id = normalize_id(pair.side_b.arxiv_id)
    rank_1based = full.index(b_id) + 1                   # 1 = best
    pctile_rank = 100.0 * (rank_1based - 1) / (K - 1)    # 0 = ranked best, 100 = worst
    return rank_1based, pctile_rank
```
Repair rule (deterministic): drop ids not in the pool; drop duplicate occurrences after the
first; append every **missing** pool id at the **bottom in lexicographic order**. The result is
always a full permutation of the 50 pool ids, so `pctile_rank` is defined for any model output.

---

## 4. Judge reference `{reference_field, reference_mechanism}`

For each `PAIR`, construct the held-out reference fed to `rs22_judge_semantic.md` **from side_b
only**:

```
reference_field     = field_label(pair.side_b.category)                       # deterministic (§1)
reference_mechanism = mechanism_probe(pair.side_b.title, pair.side_b.abstract)["core_mechanism"]
```

- `mechanism_probe` is the new `rs22_probe_mechanism.md`, run on `side_b {title, abstract}`
  **only**, in a **fresh session with no other context** (no side_a, no bridge, no pair_id),
  `M0` at temperature 0, output cached per §0.6. Its strict output is
  `{core_mechanism: str, brief_reason: str}`; `reference_mechanism` is the `core_mechanism`
  string verbatim.
- The judge is then called with `{target_field, shared_mechanism}` from the recall probe and
  `{reference_field, reference_mechanism}` as above, returning
  `{field_match, mechanism_match, overall_equivalent, rationale}`. The judge remains the sole
  semantic authority (§5 only subtracts self-field paraphrase, never overrides a `false`).

---

## 5. Self-field guard — "subtract side_a's native vocabulary"

The recall probe **fires** (recalled side_b's analogue) only if the judge accepts it **and** the
model did not merely re-name the query's own field.

```
STOP = {"and","of","the","in","for","a","an","to","on","with"}

def normalize_label(s):
    s = re.sub(r"[^a-z0-9]+", " ", str(s).lower())
    return " ".join(s.split()).strip()

def content_tokens(s):
    return {t for t in normalize_label(s).split() if t not in STOP}

def same_field(x, y):                       # is free-text x the SAME field as label y?
    if x is None or normalize_label(x) == "":
        return False
    if normalize_label(x) == normalize_label(y):
        return True                         # exact (normalised) equality
    Tx, Ty = content_tokens(x), content_tokens(y)
    if not Tx or not Ty:
        return False
    overlap = len(Tx & Ty) / min(len(Tx), len(Ty))   # overlap coefficient (min-based)
    return overlap >= SELF_FIELD_OVERLAP    # 0.67

def recall_fires(pair, judge_out, recall_out):
    if not judge_out["overall_equivalent"]:
        return False                        # judge is the authority
    a_label = field_label(pair.side_a.category)
    return not same_field(recall_out["target_field"], a_label)   # guard: not side_a's own field
```

**The "different field" check is exact:** a HIT requires `overall_equivalent == true` AND
`same_field(target_field, field_label(side_a.category)) == false`. `same_field` is normalised-
string inequality **OR** a content-token **overlap coefficient ≥ 0.67** (intersection over the
smaller token set, stopwords `STOP` removed). Threshold `0.67` is deliberately conservative: it
fires only when a clear majority (≥ 2/3) of the shorter label's content tokens coincide — so it
catches "condensed matter" ⊂ "condensed matter physics" (self-paraphrase, overlap 1.0) but does
**not** wrongly flag genuinely different fields that merely share one modifier, e.g. "statistical
mechanics" vs "statistical finance" (overlap 0.5 < 0.67). The guard only removes self-field
paraphrase; it never turns a judge `false` into a hit.

---

## 6. Clean-stratum (memory-absent) membership — canonical definition

Restating C-46 (two-format memory probe) + the frozen prompts precisely. A pair is **CLEAN
(memory-absent)** iff it **fails BOTH formats with a low-confidence non-match**:

```
def is_clean(pair, judge_out, recall_out, recog_out):
    # (i) recall does NOT fire (per §5)
    fails_recall = not recall_fires(pair, judge_out, recall_out)
    # (ii) recognition picks the WRONG field (exact string compare vs the correct menu option)
    b_label     = field_label(pair.side_b.category)
    recog_wrong = (recog_out["chosen_field"] != b_label)
    # (iii) recognition confidence is LOW
    recog_low   = (recog_out["confidence"] <= RECOG_CONF_MAX)      # 0.5
    return fails_recall and recog_wrong and recog_low
```

Exact predicate:

> `CLEAN(pair) ⇔ ¬recall_fires(pair) ∧ (recognition.chosen_field ≠ field_label(side_b.category))
> ∧ (recognition.confidence ≤ 0.5)`

- `recall_fires` is §5 (judge `overall_equivalent` ∧ not-self-field).
- `recognition.chosen_field` is copied **verbatim** from `field_options` (probe constraint), and
  the correct option **is** `field_label(side_b.category)`, so `!=` is a direct exact string
  comparison — no fuzzy matching needed.
- `RECOG_CONF_MAX = 0.5` (recommended): "low confidence" = at or below the midpoint of `[0,1]`.
- The familiarity (`rs22_probe_familiarity.md`) and open-book (`rs22_probe_openbook.md`) probes
  are **controls**; they do **not** enter the CLEAN predicate (they characterise *why* a pair is
  clean — memorised-source vs can't-reason — downstream).

---

## 7. End-to-end per-pair harness (assembled)

```
papers = build_papers(corpus)                                  # §3.1, once
Lcorpus = corpus_field_labels(corpus)                          # §2, once

def score_pair(pair):
    # --- instruments (M0, temp 0, fresh sessions, cached) ---
    recall_out  = run(rs22_probe_recall,      side_a={title,abstract})
    menu        = field_options(pair, corpus)                  # §2
    recog_out   = run(rs22_probe_recognition, side_a={title,abstract}, field_options=menu)
    _fam        = run(rs22_probe_familiarity, title=side_b.title)          # control
    _open       = run(rs22_probe_openbook,    A=side_a, B=side_b)          # control
    presented,K = retrieval_pool(pair, papers)                 # §3
    ranked_ids  = run_retrieval(side_a={title,abstract}, candidates=presented)   # §3.3
    rank1, pct  = rank_side_b(pair, presented, K, ranked_ids)  # §3.4
    # --- reference + judge ---
    ref_field   = field_label(side_b.category)                 # §4
    ref_mech    = run(rs22_probe_mechanism, side_b={title,abstract})["core_mechanism"]
    judge_out   = run(rs22_judge_semantic,
                      target_field=recall_out.target_field, shared_mechanism=recall_out.shared_mechanism,
                      reference_field=ref_field, reference_mechanism=ref_mech)
    # --- derived ---
    fired       = recall_fires(pair, judge_out, recall_out)    # §5
    clean       = is_clean(pair, judge_out, recall_out, recog_out)          # §6
    return {pair_id, recall_out, recog_out, menu, judge_out, ref_field, ref_mech,
            rank_1based:rank1, pctile_rank:pct, K, recall_fired:fired, clean}
```

Every step is a pure function of the frozen corpus + the pinned model's cached, temperature-0
outputs. No RNG, no wall-clock, no set-iteration dependence, no researcher choice.

---

## Determinism & blindness attestation

I authored this spec **blind**: from general principles and the **public arXiv category
taxonomy** only, plus the six frozen RS-22 prompt templates, `rs22_mining_protocol.md`,
`rs22_prompts_README.md`, `rs22_constants.json`, and the **key structure of the first pair only**
of `rs22_mined_pairs.json` (to confirm the record schema). I did **not** read, open, or reference
`THREAD.md`, any verification file, any `bridge_names`/`cross_bridges_ground_truth`, any verdict,
any prior EXP-RS result, or any answer key about which pairs share a method, and no rule here uses
`bridge_paper.arxiv_id` or `bridge_paper.asserted_analogy_snippet` (leakage guard, §0.2). Every
ordering, selection, and tie-break is resolved by lexicographic string comparison or by a
documented domain-separated SHA-256 hash of `(pair_id, item)` — there is **no RNG** anywhere. The
harness is therefore fully deterministic and reproducible bit-for-bit from the frozen corpus and
the pinned model's cached temperature-0 outputs, and its construction is independent of the
study's outcome.
