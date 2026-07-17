# RS-22 Cross-Field Analogy-Pair Mining Protocol (FROZEN)

**Experiment:** EXP-RS-22 (Phase 41) — mechanical mining stage.
**Status:** to be SHA-256-frozen and committed BEFORE any execution.
**Author standpoint:** neutral research-tooling engineer. This protocol was authored
with **no access** to any existing benchmark, ground-truth, or answer key. It is written
purely from general principles and describes a fully deterministic pipeline.
**Determinism guarantee:** no RNG, no model calls, no "iterate until N clean." Every step is a
pure function of (a) this spec, (b) the frozen snapshot artifact, (c) the frozen constants
(`prototypes/data/rs22_constants.json`, sha256 `af5ee11c7828fbec0bf9eb6a9520e82cb57dbebfcacf4f3290b108c3a8643c33`).

Downstream conventions this feeds: C-46 (two-format memory probe), C-47 (blind constants,
`n_floor = 110` clean pairs), C-48 (one-directional metric + total gate; "Deterministic frozen
mining (snapshot+SHA before the probe split; pre-specified expansion blocks, never
iterate-until-n; mining VALIDITY judgment = Claude/panel not Mistral").

---

## 0. Definitions

- **Cross-field analogy pair** — two papers `side_a`, `side_b` from **different scientific
  fields** whose methods/mechanisms are the **same underlying mathematics** (e.g. a
  phase-transition / renormalization / percolation method reused across fields).
- **Bridge paper** — a *third* paper that **explicitly asserts** such a cross-field
  equivalence in its abstract (using one of the frozen phrase patterns, §1.2) **and** cites
  papers on both sides. The bridge is **never** emitted as `side_a` or `side_b`; it is only the
  evidence that a real cross-field link exists.
- **TOPCAT(cat)** — the arXiv top-level archive: the substring of an arXiv category before the
  first `.`; if the category has no `.`, the category itself.
  Examples: `cond-mat.stat-mech → cond-mat`, `physics.soc-ph → physics`, `q-bio.PE → q-bio`,
  `cs.LG → cs`, `q-fin.ST → q-fin`, `hep-th → hep-th`, `math-ph → math-ph`.
- **Clean domain paper** — a real arXiv paper whose abstract does **NOT** contain any frozen
  bridge-assertion pattern (§1.2). Domain papers do their own single-field science; the bridge
  assertion is the *bridge's* job, not a side's.

---

## 1. Sources & frozen queries

### 1.1 Sources (in priority order)

| # | Source | Role | Access |
|---|---|---|---|
| S1 | **arXiv API** (`http://export.arxiv.org/api/query`) | canonical `{title, abstract, primary_category}` for any arXiv id; bridge discovery via `abs:` phrase search | `id_list` batches ≤ 50, **3.0 s** rate-limit (matches repo `ArxivHTMLDownloader` default / C-14 practice) |
| S2 | **OpenAlex** (`https://api.openalex.org/works`) | bridge discovery via `abstract.search`; bridge **references** (`referenced_works`); abstract/category fallback | polite pool, `mailto` UA `resyn-research/0.1 (mailto:jasperseehofermusic@gmail.com)`, optional `OPENALEX_API_KEY` bearer; `abstract_inverted_index` reconstruction as in `build_bridge_corpus_openalex.py` |
| S3 | **Semantic Scholar** (`https://api.semanticscholar.org/graph/v1`) | tertiary bridge discovery + `references` fallback when OpenAlex lacks `referenced_works` | key under non-commercial terms; used read-only |

The repo already ships arXiv + OpenAlex access (`data_aggregation/arxiv_api.rs`,
`bulk-ingest`, `build_bridge_corpus_openalex.py`); S3 is optional and only fills reference gaps.

### 1.2 Frozen bridge-assertion phrase patterns (P)

14 case-insensitive substring patterns. Matching is performed on the **whitespace-normalized,
lowercased** abstract (collapse every run of whitespace — including newlines — to a single
space, then `.lower()`), so line-wrapped phrases still match. The list is **ordered**; order
is load-bearing only for snippet selection (§2.4).

```
P01  "mathematically equivalent to"
P02  "formally equivalent to"
P03  "formally identical to"
P04  "the same universality class as"
P05  "belongs to the same universality class"
P06  "obeys the same equations as"
P07  "governed by the same equations as"
P08  "maps onto"
P09  "can be mapped onto"
P10  "isomorphic to"
P11  "the same mathematical structure as"
P12  "in direct analogy with"
P13  "borrowed from"
P14  "is analogous to"
```

A paper is a **bridge candidate** iff its whitespace-normalized/lowercased abstract contains
≥ 1 pattern in P as an exact substring. This exact-substring re-check is the **authoritative**
gate: the API-side phrase searches below are only a coarse pre-filter, so any looseness in an
API's phrase handling cannot leak non-matching abstracts through.

### 1.3 Frozen query strings (one per source per pattern)

Substitute each `P` verbatim (quoted). Freeze `SNAPSHOT_DATE` = the UTC date the mine is run
(recorded in the snapshot manifest, §4.2).

**S1 arXiv** — one request per pattern:
```
search_query=abs:"<P>"+AND+submittedDate:[200701010000+TO+<SNAPSHOT_DATE>2359]
&start=0&max_results=500&sortBy=submittedDate&sortOrder=ascending
```
(Paginate `start` in steps of 100 up to `max_results`. arXiv supports `abs:"..."` phrase and
boolean `AND`.)

**S2 OpenAlex** — one cursor-paginated request per pattern:
```
filter=abstract.search:"<P>",language:en,from_publication_date:2007-01-01,
       to_publication_date:<SNAPSHOT_DATE>,type:article
&select=id,doi,title,display_name,publication_date,primary_topic,topics,
        referenced_works,locations,abstract_inverted_index,language
&per-page=200&sort=publication_date:asc&cursor=<cursor>
```
Cap at 500 results per pattern (`N_CAP_PER_PATTERN = 500`).

**S3 Semantic Scholar** (optional tertiary) — one request per pattern:
```
GET /paper/search?query=<P>&fields=externalIds,title,abstract,fieldsOfStudy
    &publicationDateOrYear=2007:<SNAPSHOT_YEAR>&limit=100
```

### 1.4 Frozen date range & caps

- **Date range:** `2007-01-01 … SNAPSHOT_DATE` (the new-format arXiv era; captures the full
  cross-field-analogy literature the corpus is drawn from).
- **Per (source × pattern) cap:** `N_CAP_PER_PATTERN = 500` (S1/S2), `100` (S3).
- **Ordering inside each API result set:** publication/submitted date **ascending**, ties
  broken by native id ascending. This makes each raw result list a deterministic, reproducible
  sequence that is snapshotted verbatim (§4.2).

---

## 2. Bridge → pair extraction (deterministic)

Process bridge candidates one at a time. Each surviving bridge yields **exactly one** pair.

### 2.1 Resolve the bridge's arXiv id
A bridge candidate must itself be arXiv-resolvable (`bridge_arxiv_id` via arXiv id, OpenAlex
DOI `10.48550/arxiv.*`, or a `locations[].landing_page_url` matching `arxiv.org/abs/…` — reuse
`arxiv_id_of()` from `build_bridge_corpus_openalex.py`). If not arXiv-resolvable → **reject
bridge** (reason `bridge_not_arxiv`). Strip version suffix (`utils::strip_version_suffix`
semantics: drop trailing `vN`).

### 2.2 Collect the bridge's candidate domain references R
1. Fetch the bridge's outgoing references: OpenAlex `referenced_works` (primary); if empty, S3
   `/paper/{arxiv_id}/references` (`externalIds.ArXiv`).
2. For each reference, resolve to an `arxiv_id` (drop non-arXiv references).
3. Fetch `{title, abstract, primary_category}` per reference from **S1 arXiv** (`id_list`,
   3 s rate-limit); OpenAlex reconstruction fallback for missing fields; `TOPCAT` from
   `primary_category` (arXiv) or, if absent, the OpenAlex `primary_topic` **level-0** field id.
4. Keep a reference in **R** iff it passes ALL of:
   - `arxiv_id != bridge_arxiv_id`;
   - abstract is a **clean domain** abstract (contains **no** pattern in P, §1.2);
   - abstract ≥ **200 characters** and **English** (§3.2);
   - `arxiv_id` and `TOPCAT` both resolved.
5. Sort **R** by `arxiv_id` **lexicographic ascending** (canonical order for all tie-breaks).

### 2.3 Select the two sides (fully deterministic)
1. Compute the multiset of `TOPCAT` values over **R**. Let `C` = the list of **distinct**
   TOPCATs, ranked by **descending count** (how many refs the bridge cites from that field),
   ties broken by TOPCAT string **ascending**.
2. If `|C| < 2` → **reject bridge** (reason `not_cross_field`): the bridge does not cite two
   distinct fields, so no cross-field pair can be formed mechanically.
3. The two bridged fields = `c1 = C[0]`, `c2 = C[1]` (the two most-cited distinct fields; a
   robust field-agnostic proxy for "the two named fields," and independent of any answer key).
4. `paper_c1` = the **lexicographically-first** `arxiv_id` in R with `TOPCAT == c1`.
   `paper_c2` = the **lexicographically-first** `arxiv_id` in R with `TOPCAT == c2`.
5. **side_a / side_b assignment (the ambiguity tie-break):** order `{paper_c1, paper_c2}` by
   `arxiv_id` **lexicographic ascending**; the **smaller** becomes `side_a`, the larger
   `side_b`. This is a total order, so the assignment is unique and deterministic.
   - `side_a` (the paper used as the retrieval query downstream) is by construction a **clean
     domain paper** (its abstract contains none of P) — this is the load-bearing requirement
     from the task; `side_b` is likewise clean (both were filtered in §2.2.4).

> The mine does **not** claim `paper_c1`/`paper_c2` are the *true* two sides the bridge names,
> nor that they truly share a method — only that they are two clean papers from the two fields
> the bridge cites most and that a bridge explicitly asserted a cross-field equivalence linking
> those fields. The **validity judgment is DEFERRED** (§6).

### 2.4 Extract the asserted-analogy snippet
On the bridge's whitespace-normalized/lowercased abstract, find the **first** pattern match by
scanning P in list order `P01…P14` and taking the earliest character position among matches.
`asserted_analogy_snippet` = the **verbatim (original-case)** substring of the normalized
abstract from the start of the sentence containing the match (previous `". "` boundary or
string start) to the end of that sentence (next `". "` boundary or +240 chars, whichever comes
first). Store the triggering pattern id alongside in the snapshot log.

---

## 3. Frozen filters (a pair is emitted only if ALL pass)

Applied after §2 produces a `(side_a, side_b, bridge)` triple.

1. **Cross-field.** `TOPCAT(side_a) != TOPCAT(side_b)` (arXiv top-level archives differ).
   If TOPCAT is unavailable for either side, fall back to OpenAlex **level-0** concept ids and
   require they differ. (Guaranteed by §2.3 but re-asserted here as a hard gate.)
2. **Both real & fetchable.** Each side has a resolvable, version-stripped `arxiv_id` and
   non-empty `title`.
3. **Abstract length.** Each side's abstract ≥ **200 characters** (post whitespace-normalize).
4. **English.** Each side's abstract passes the deterministic English gate (§3.2).
5. **Clean side_a (and side_b).** Neither side's abstract contains any pattern in P (§1.2).
   (side_a cleanliness is mandatory; side_b cleanliness is applied identically.)
6. **Independence from prior benchmarks.** Drop the pair if `side_a.arxiv_id` **or**
   `side_b.arxiv_id` appears in the frozen exclusion file
   `prototypes/data/rs22_exclusion_ids.txt` (union of all prior EXP-RS benchmark endpoints,
   supplied as bare ids at execution time). This keeps the mined stratum independent of
   already-studied pairs. *(An id list of things to EXCLUDE is not an answer key.)*
7. **Dedup by arxiv_id.** Maintain a running set of already-emitted `arxiv_id`s. Reject a pair
   if `side_a.arxiv_id` or `side_b.arxiv_id` is already used by an earlier emitted pair, and
   reject exact duplicate unordered `{side_a, side_b}` pairs. First occurrence in canonical
   order (§4.1) wins.

### 3.2 Deterministic English gate
On the lowercased abstract: (a) ratio of `[a-z]` characters to total non-space characters
`≥ 0.90`, **and** (b) it contains `≥ 5` **distinct** words from the frozen function-word set
`{the, of, and, to, in, a, is, for, we, that, with, are, this, as, by}`. No language-ID model
(keeps it deterministic). When the abstract came from OpenAlex, also require `language == "en"`.

---

## 4. Deterministic draw, snapshot & blocks

### 4.1 Canonical ordering
1. Union all bridge-candidate ids from all (source × pattern) raw result lists; **dedup** by
   `bridge_arxiv_id`.
2. Sort the deduped bridge-candidate list by `bridge_arxiv_id` **lexicographic ascending**
   (the deterministic draw key). Ties impossible (ids unique).
3. Process bridges in this order through §2–§3. Each surviving pair is appended to the
   **emitted-pairs sequence** in processing order. A pair's **rank** = its 0-based index in
   this sequence.
4. `pair_id = "rs22-%06d" % rank`. `block_id = (rank // BLOCK_SIZE) + 1`.
   Because the snapshot (§4.2) freezes the exact raw id lists, this ordering is fully
   reproducible from the frozen artifact.

### 4.2 Snapshot (frozen BEFORE the probe split — C-48)
Write `prototypes/data/rs22_mining_snapshot.json` and record its sha256 in the commit message
and in `rs22_mined_pairs.json.snapshot_ref`:
```
{
  "protocol_sha256": "<sha256 of THIS file, rs22_mining_protocol.md>",
  "constants_sha256": "af5ee11c7828fbec0bf9eb6a9520e82cb57dbebfcacf4f3290b108c3a8643c33",
  "snapshot_date_utc": "<SNAPSHOT_DATE>",
  "source_index_names": {
    "arxiv": "export.arxiv.org/api/query",
    "openalex": "api.openalex.org/works (snapshot <OpenAlex data-version header>)",
    "semantic_scholar": "api.semanticscholar.org/graph/v1"
  },
  "raw_hits": {                     // the RAW returned id lists, verbatim, per source×pattern
    "arxiv":     { "P01": ["...","..."], "...": [] },
    "openalex":  { "P01": ["W..."],      "...": [] },
    "s2":        { "P01": ["..."],       "...": [] }
  },
  "bridge_candidates_sorted": ["<arxiv_id>", "..."],   // §4.1 step 2, the frozen draw order
  "rejections": [ {"bridge_arxiv_id":"...","reason":"not_cross_field"} , ... ]
}
```

### 4.3 Blocks (NEVER iterate-until-clean-hits-110)
- `BLOCK_SIZE = 60`. Block *k* = emitted pairs with `rank ∈ [ (k-1)·60 , k·60 )`.
- **Stopping rule (deterministic, fixed count):** the mine emits the **first
  `N_COMMIT = 420` filter-passing pairs** in canonical order — i.e. blocks **B1…B7**. This is a
  fixed count on a frozen ordered list; it is computed **without any reference to memory /
  clean-stratum outcomes** (the mine is memory-blind). If < 420 pairs survive from the mined
  bridge set, raise `N_CAP_PER_PATTERN` in a single pre-registered re-freeze — not adaptively.
- **Expansion contract:** the downstream memory probe (§6, C-46) partitions these pairs into
  clean / non-clean. If, after probing the committed blocks, the clean-stratum count is
  `< n_floor = 110`, the **sole sanctioned expansion** is to mine and emit the **next whole
  pre-specified block B8** (`rank ∈ [420, 480)`), re-freeze the snapshot, and re-probe. This is
  a **discrete, logged, pre-committed unit** — repeat with B9, B10… only as whole blocks. At no
  point does mining or block-drawing stop at "exactly 110 clean"; blocks are drawn in full,
  in order, and the clean count never steers the mining loop.

### 4.4 Block-size sizing rationale (state the yield assumption)
- Downstream requires `n_floor = 110` **clean (memory-absent)** pairs (C-47/`rs22_constants`).
- **Assumed memory-absent yield `f_mem = 0.30`** of emitted valid pairs. Rationale: bridge
  papers assert *known* cross-field analogies, and the famous ones (e.g. spin-glass ↔ neural
  nets, Ising ↔ opinion dynamics) will be **recalled** by the model → NOT memory-absent; only
  the more obscure / recent cross-field reuses land in the clean stratum. `0.30` is a
  deliberately conservative planning figure.
- **Implied total:** `ceil(110 / 0.30) = 367` valid pairs minimum; with ~15% headroom for the
  clean-stratum false-miss ceiling (`falsemiss_max = 0.10`, C-47) and stratum losses →
  **commit `N_COMMIT = 420`** = **7 blocks × 60**. Expected clean at `f_mem = 0.30` ≈ **126**
  (13% above the 110 floor); at `f_mem = 0.25` ≈ 105 (short → triggers the B8 expansion by the
  §4.3 contract); at `f_mem = 0.40` ≈ 168 (comfortable). The block mechanism is exactly the
  designed remedy for a low realized yield.

---

## 5. Output schema — `prototypes/data/rs22_mined_pairs.json`

```json
{
  "protocol_sha256": "<sha256 of rs22_mining_protocol.md>",
  "snapshot_ref": "rs22_mining_snapshot.json@<sha256>",
  "block_size": 60,
  "n_blocks_committed": 7,
  "n_pairs": 420,
  "pairs": [
    {
      "pair_id": "rs22-000000",
      "side_a": {
        "arxiv_id": "0803.1234",
        "title": "…",
        "abstract": "…(≥200 chars, English, contains no bridge-assertion phrase)…",
        "category": "cond-mat.stat-mech"
      },
      "side_b": {
        "arxiv_id": "1104.5678",
        "title": "…",
        "abstract": "…(≥200 chars, English, contains no bridge-assertion phrase)…",
        "category": "q-fin.ST"
      },
      "bridge_paper": {
        "arxiv_id": "1207.9012",
        "asserted_analogy_snippet": "…verbatim sentence containing the first matched pattern…"
      },
      "block_id": 1
    }
  ]
}
```

- **Exactly** the requested per-pair record shape:
  `{pair_id, side_a:{arxiv_id,title,abstract,category}, side_b:{…}, bridge_paper:{arxiv_id,
  asserted_analogy_snippet}, block_id}`.
- `category` = the paper's arXiv `primary_category` (full, not just TOPCAT).
- `pairs` is ordered by `pair_id` ascending (= canonical rank order = block order).
- The top-level manifest fields make the file self-freezing (protocol + snapshot SHAs).

---

## 6. Explicitly DEFERRED (NOT part of this mechanical mine)

This protocol is a **memory-blind, mechanical** miner. Two things are **out of scope** and are
carried out by **separate, downstream steps**:

1. **Method-sharing VALIDITY judgment — a SYNTHESIS task.** Deciding whether `side_a` and
   `side_b` *really* share the same underlying mathematics/mechanism (vs. the bridge merely
   citing two loosely related papers, or the mechanical §2.3 field-count heuristic having
   picked the wrong two references) is **NOT** mechanized here. Per C-48 (panel MF9) this
   judgment is made by **Claude / the human panel**, never by the mechanical executor
   (Mistral). The mine only guarantees the *structural* facts: a bridge paper explicitly
   asserted a cross-field equivalence (frozen phrases), it cites both fields, and the two sides
   are clean single-field papers from the two fields it cites most. Invalid pairs are dropped
   in that separate validity pass; this protocol deliberately over-produces (§4.4 headroom) to
   survive that attrition.

2. **Memory-probing / clean-stratum assignment.** The two-format memory probe (free-recall +
   recognition), the confidence scoring, and the **clean (memory-absent) stratum** definition
   (C-46: "fails BOTH formats ∧ LOW-confidence non-match") are a **separate downstream step**
   run in independent, fresh sessions with zero shared history and per-call input hashing
   (C-46 panel MF5). The mine emits its pairs **before and independently of** any memory
   outcome; the probe never feeds back into mining, and mining never reads memory results
   except through the discrete, pre-specified block-expansion contract (§4.3).

Everything above the line (surface bridges → extract two clean cross-field sides → apply frozen
filters → snapshot → partition into fixed blocks → emit the fixed-schema JSON) is the entire
mechanical mine and is 100% deterministic.

---

## 7. Freeze & reproducibility checklist

1. Fill `SNAPSHOT_DATE`; run §1 queries; write `rs22_mining_snapshot.json` (§4.2).
2. Run §2–§4; write `rs22_mined_pairs.json` (§5).
3. `sha256sum rs22_mining_protocol.md rs22_mining_snapshot.json rs22_mined_pairs.json` →
   record all three in the commit and in `.planning/research/THREAD.md` (same-day update).
4. Commit **before** the probe split. Any later change to P, caps, block size, or the §2.3 rule
   is a **new** protocol version with a new SHA and a fresh snapshot — never an in-place edit
   (append-only, C-conventions discipline).
5. A re-run from the frozen `rs22_mining_snapshot.json` (steps §2–§4 replayed over the frozen
   `raw_hits` / `bridge_candidates_sorted`) MUST reproduce `rs22_mined_pairs.json` byte-for-byte
   modulo live abstract text (which is itself pinnable by caching the arXiv/OpenAlex responses
   in `prototypes/data/rs22_abstract_cache/`).
```
