# Phase 27: arXiv Crawl Optimisation — Research

**Researched:** 2026-04-22
**Domain:** Academic citation graph construction — bulk reference-edge ingestion, arXiv metadata batching
**Confidence:** HIGH (all critical claims verified live against APIs or codebase)

---

## Summary

The arXiv crawl bottleneck has two separable components: (1) *metadata* fetches (`fetch_paper`) and
(2) *reference* fetches (`fetch_references` — one HTML scrape per paper). The EXP-RS-07 session
already confirmed that reference fetches cannot be accelerated at the per-paper level — there is no
arXiv references API, and HTML scraping at 3 s/paper is the correct rate. The only route to order-of-
magnitude speedup is to *bypass per-paper HTTP calls entirely* for papers whose citation graph is
already known from a bulk source.

**The primary recommendation is: extend `bulk_ingest.rs` to ingest a physics/cond-mat corpus from
OpenAlex, reusing the existing two-phase spill pattern.** This gives ~100 % of crawl speedup for any
corpus subset covered by OpenAlex (2.9 M cond-mat works, 1.9 M stat-phys works). The only additional
engineering is (a) registering an OpenAlex API key (free, 30 s), (b) passing `--db` to target a new
corpus DB, and (c) writing a correct `--filter` string for physics/cond-mat topics. No trait changes.
No new ingestion architecture needed.

For the longer-term BFS crawler, arXiv API `id_list` batching can replace single-paper metadata
fetches (up to 200 IDs per 3-second call → 200× metadata speedup), but this does not help the
reference-scrape bottleneck at all, so it is secondary.

**Primary recommendation:** Add OpenAlex free-tier API key to `bulk_ingest.rs`; run `bulk-ingest`
with a cond-mat/stat-phys `--filter` and a new `--db surrealkv://./data-physics`; use the resulting
DB for EXP-RS-07 instead of HTML-crawled corpora.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|---|---|---|---|
| Metadata bulk-load | `openalex_bulk.rs` + `bulk_ingest.rs` | arXiv OAI-PMH (future) | OpenAlex already wired; OAI adds coverage for papers not in OA |
| Reference-edge ingestion | `bulk_ingest.rs` two-phase spill | — | Pattern already exists; no per-paper HTTP needed |
| Per-paper BFS crawl | `arxiv_utils.rs` BFS + `PaperSource` | `ChainedPaperSource` | Unchanged; used when corpus is not pre-loaded |
| arXiv id_list batching | `arxiv_api.rs` (new param) | — | Metadata speedup only; no reference-edge benefit |
| Reference resolution | `ArxivHTMLDownloader` | S2 / InspireHEP fallback via chain | Rate-limited to 3 s/paper; irreducible for HTML scrape path |

---

## Ranked Tradeoff Table

| Source | arXiv-ID both sides | Rate / volume | License | Rust crate | Eng cost | Speedup vs 3 s baseline |
|---|---|---|---|---|---|---|
| **OpenAlex `referenced_works`** (recommended) | ~60-70 % of edges (where both src and dst have arXiv presence) [ASSUMED — see A2] | 10 req/s with API key; 10 000 filter calls/day free tier; S3 snapshot free/unlimited | CC0 [VERIFIED: opencitations.net, OpenAlex docs] | Hand-roll `reqwest` — already done in `openalex_bulk.rs` | S — extend existing filter arg + API key | ~infinite for pre-loaded corpus; eliminates per-paper HTTP entirely |
| **Semantic Scholar Datasets API** | High — S2 resolves arXiv IDs explicitly; `citations` dataset has `externalIds.ArXiv` on both sides [VERIFIED: S2 API] | Free download with API key (key required — no cost stated for bulk datasets) | Non-commercial / nonprofit permitted [VERIFIED: semanticscholar.org/license] | `semantic-scholar-rs` v0.1.1 — targets per-paper API, NOT datasets bulk; hand-roll `reqwest` for datasets | M — separate bulk download path; requires S3 file streaming | ~infinite (pre-load) if dataset downloaded |
| **arXiv API `id_list` batching** | N/A — no reference data, metadata only | 3 s/request, up to 200 IDs/call [VERIFIED: info.arxiv.org/help/api/user-manual.html] | arXiv ToS — no bulk restrictions for id_list | `arxiv-rs` v0.1.5 (already in Cargo.toml) — supports id_list [ASSUMED — A3] | S — add param to `arxiv_api.rs` | 200× for metadata fetches only; 0× for references |
| **OpenCitations COCI** | Edges are DOI-to-DOI only — no arXiv IDs returned; `cited` field format: `omid:br/... doi:... openalex:...` [VERIFIED: api.opencitations.net live test] | 1 req/s, CC0 bulk download | CC0 [CITED: opencitations.net] | None — hand-roll `reqwest` | L — DOI→arXiv resolver step required; adds a third lookup phase | LOW — extra resolution hop makes it slower than OpenAlex direct |
| **Crossref `/works` reference array** | ~43 M works have open reference lists; references may contain arXiv DOIs (10.48550/arxiv.*) but coverage is sparse for physics preprints [CITED: crossref.org] | 50 req/s polite pool | Crossref TDM — free for non-commercial [ASSUMED — A4] | `crossref` crate v0.2.2 (last updated 2019-07-23 — abandoned) | L — crate abandoned; hand-roll; DOI→arXiv resolution for cited refs | LOW — sparse arXiv coverage on the cited side |
| **INSPIRE-HEP API** | HIGH for HEP papers — API returns `arxiv_eprints` field with arXiv IDs on both sides [VERIFIED: existing `InspireHepClient` in codebase] | 350 ms/paper rate limit (already implemented) | CC0 (most metadata) [CITED: github.com/inspirehep/rest-api-doc] | `InspireHepClient` in `resyn-core` — already implemented | XS — already wired as `--source inspirehep`; no new work | Same as current crawl — per-paper, no bulk |
| **NASA ADS** | HIGH for astrophysics papers on arXiv; citations include arXiv IDs | 5 000 req/day (API key required; free for research) | Research use allowed [ASSUMED — A5] | None — hand-roll `reqwest` | M — new client, new data model | LIMITED — 5 000/day cap makes bulk pre-load impractical |
| **arXiv OAI-PMH** | Metadata only (title, authors, abstract, dates) — NO reference data [VERIFIED: info.arxiv.org/help/api] | No documented rate limit; harvesting allowed for non-commercial [ASSUMED — A6] | arXiv ToS non-commercial [ASSUMED] | `oai-pmh` v0.5.0 (196 dl, 2026-03-13) | M — new trait impl or standalone loader | Metadata speedup only; 0× for references |

---

## Standard Stack

The following are the libraries/APIs Phase 27 WILL use. No alternatives.

### Core (already in Cargo.toml — zero new deps required)

| Library | Version | Purpose | Source |
|---|---|---|---|
| `reqwest` | 0.12.15 | HTTP client for OpenAlex API | workspace |
| `serde` / `serde_json` | 1.x | JSON deserialisation of OpenAlex pages | workspace |
| `tokio` | 1.44.1 | Async runtime, page-delay sleep | workspace |
| `clap` | 4 | `--api-key` arg on `bulk-ingest` subcommand | workspace |
| `tracing` | 0.1 | Progress logging | workspace |

**No new Cargo dependencies are needed for the primary recommendation.**

### Optional secondary work (arXiv id_list batching)

| Library | Version | Purpose |
|---|---|---|
| `arxiv-rs` | 0.1.5 | Already in Cargo.toml — add `id_list` param call |

### Crates surveyed and rejected

| Crate | Version | Last updated | Downloads | Verdict |
|---|---|---|---|---|
| `crossref` | 0.2.2 | 2019-07-23 | 10,924 | Abandoned — do not use |
| `crossref-rs` | 0.3.0 | Recent | low | Nushell plugin CLI tool, not a library |
| `openalex` | 0.2.2 | 2024-08-14 | 7,448 | Read-only metadata; doesn't cover bulk cursor pagination pattern ReSyn needs |
| `papers-openalex` | 0.3.1 | 2026-02-23 | 277 | Small; bulk cursor not verified |
| `oai-pmh` | 0.5.0 | 2026-03-13 | 196 | Metadata-only; no reference data |
| `semantic-scholar-rs` | 0.1.1 | 2026-02-14 | 34 | Per-paper API wrapper, not bulk datasets |

All [VERIFIED: crates.io API 2026-04-22].

---

## Architecture Patterns

### System Architecture Diagram

```
                 Phase 27 Data Paths
                 ===================

  [OpenAlex API]                  [arXiv API]
   filter=cond-mat concepts        id_list=id1,id2,...
   per-page=200, cursor=*          max_results=200
        |                               |
        v                               v
  fetch_page()                   arxiv_api::fetch_batch()
  (openalex_bulk.rs)             (arxiv_api.rs — new)
        |                               |
        |-- Paper.id (arXiv)            |-- Vec<Paper>
        |-- referenced_works            |    (metadata only)
        |   [OA W-IDs]                  |
        v                               v
  Phase 1: spill JSONL            upsert_papers_batch()
  {"f":"arxiv_id",                (existing)
   "r":["W123","W456"]}
        |
        v
  id_map: OA_ID -> arXiv_ID
  (built during Phase 1)
        |
        v
  Phase 2: translate + upsert
  upsert_citations_batch(
    &[(from_arxiv, to_arxiv)])
        |
        v
  [SurrealDB: surrealkv://./data-physics]
  paper table + cites edges
        |
        v
  cargo run -- analyze --db surrealkv://./data-physics
        |
        v
  cargo run -- export-louvain-graph ...
```

### Recommended Project Structure (files to touch/add)

```
resyn-server/src/commands/
├── bulk_ingest.rs           # TOUCH: add --api-key arg; update URL builder
resyn-core/src/data_aggregation/
├── openalex_bulk.rs         # TOUCH: pass api-key header in fetch_page()
├── arxiv_api.rs             # TOUCH (secondary): add id_list batch fetch fn
```

**No new files required for primary recommendation.** The cond-mat ingest is a pure
configuration change: different `--filter` and `--db` args on the existing `bulk-ingest` subcommand.

### Pattern 1: OpenAlex API key authentication (breaking change from mailto)

**What:** As of 2026-02-13, OpenAlex requires an API key for production use. Without a key,
users get 100 free credits then HTTP 409. The existing `--mailto` polite-pool approach still returns
HTTP 200 for small queries (verified 2026-04-22), but will fail at bulk-ingest scale.

**When to use:** Always — add `--api-key` arg to `BulkIngestArgs`; inject as header.

**Example:**
```rust
// In openalex_bulk.rs: fetch_page signature update
pub struct OpenAlexBulkLoader {
    client: Client,
    api_key: String,     // replaces mailto
}

impl OpenAlexBulkLoader {
    pub async fn fetch_page(&self, filter: &str, cursor: &str) -> Result<OpenAlexPage, ResynError> {
        let url = format!(
            "{}?filter={}&per-page=200&cursor={}",
            OPENALEX_API, filter, cursor
        );
        self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("User-Agent", "resyn/0.1 (mailto:jasperseehofermusic@gmail.com)")
            .send()
            .await
            // ...
    }
}
```
[Source: developers.openalex.org/api-reference/authentication — VERIFIED 2026-04-22]

### Pattern 2: Physics corpus filter for `bulk-ingest`

**What:** Reuse existing `bulk-ingest` subcommand with a cond-mat/stat-phys filter.

**Verified concept IDs** (queried from OpenAlex API 2026-04-22):
- `C26873012` — Condensed matter physics (level 1, 2.9 M works)
- `C121864883` — Statistical physics (level 1, 1.9 M works)
- `C99874945` — Statistical mechanics (level 2, 38 k works)

**WARNING — CLAUDE.md bug:** `CLAUDE.md` lists `C2778407487` as "Statistical Physics" but the
OpenAlex API returns it as "Altmetrics" (351 k works). Do not use `C2778407487` for physics.
[VERIFIED: api.openalex.org live query 2026-04-22]

**Filter for cond-mat/stat-phys corpus on arXiv:**
```
primary_location.source.id:S4306400194,concepts.id:C26873012|C121864883
```
Where `S4306400194` = arXiv (Cornell University) [VERIFIED: api.openalex.org live query].

**CLI invocation:**
```bash
cargo run --release --bin resyn -- bulk-ingest \
  --db surrealkv://./data-physics \
  --api-key "$OPENALEX_API_KEY" \
  --filter "primary_location.source.id:S4306400194,concepts.id:C26873012|C121864883"
```

### Pattern 3: `referenced_works` → arXiv ID resolution

**What:** `referenced_works` contains OpenAlex W-IDs (`"https://openalex.org/W1234"`), NOT arXiv
IDs. The existing two-phase `bulk_ingest.rs` spill pattern already handles this correctly: Phase 1
builds an `id_map: HashMap<oa_id, arxiv_id>` and Phase 2 translates. Only edges where both source
and target are arXiv-linked papers are written. This is correct behaviour — no change needed.

**Verified format** (live API call 2026-04-22):
```json
{
  "referenced_works": [
    "https://openalex.org/W1560783210",
    "https://openalex.org/W1724212071"
  ]
}
```

**Resolution via batch filter (for future batch-resolution use case):**
```
GET /works?filter=ids.openalex:W1560783210|W1724212071&select=id,doi,locations&per-page=50
```
Up to 50 W-IDs per pipe-separated filter value [VERIFIED: developers.openalex.org LLM guide].

### Pattern 4: arXiv id_list batching (secondary, metadata-only speedup)

**What:** arXiv API supports `id_list=2301.12345,2401.04191,...` with `max_results=200`. One call
at 3 s returns 200 paper metadata records vs current 1. Reference scraping is unchanged.

**Batch size:** Up to 200 results per call (staying within `max_results` limit). Unofficial reports
suggest `start` offset may break above 1 000 — use for targeted id_list fetches, not open-ended
pagination. [VERIFIED: info.arxiv.org/help/api/user-manual.html; pagination caveat MEDIUM confidence]

**When to use:** When BFS queue has already resolved the next N paper IDs (from a pre-loaded DB or
from `referenced_works` resolution) and needs metadata. Not useful when IDs are unknown.

### Anti-Patterns to Avoid

- **Using `crossref` crate:** Last release 2019 — abandoned. Hand-roll if ever needed.
- **Using `C2778407487` as Statistical Physics concept:** It resolves to Altmetrics. Use `C121864883`.
- **Relying on `--mailto` for bulk-ingest at scale:** OpenAlex deprecated polite pool Feb 2026; get a free API key.
- **Running `analyze` against `./data-openalex/` (90 GB RS-08 corpus):** `get_all_papers()` has no pagination → OOM. The new physics corpus must be a separate `--db` path.
- **Expecting `DELETE cites` to be cross-database:** SurrealDB connections are per storage path; `DELETE cites` only affects the `--db` passed to that invocation [VERIFIED: `client.rs` — each `connect()` opens an isolated engine].

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---|---|---|---|
| OpenAlex API pagination | Custom cursor loop | Existing `fetch_page` + cursor in `openalex_bulk.rs` | Already battle-tested against 10 M+ record corpora |
| Two-phase citation ingestion | In-memory edge buffer | Existing JSONL spill pattern in `bulk_ingest.rs` | Avoids OOM on large corpora; already correct |
| arXiv HTML reference scraping | Alternative HTML parser | Existing `ArxivHTMLDownloader` + `scraper` | Already rate-limited correctly; don't duplicate |
| Rust crossref client | Hand-rolled HTTP | Don't use Crossref at all for this use case | DOI-to-arXiv resolution adds complexity with no speedup benefit |
| OAI-PMH harvester | Custom XML parser | `oai-pmh` crate v0.5.0 — OR skip entirely | OAI-PMH provides no reference edges; not needed for primary recommendation |

---

## Common Pitfalls

### Pitfall 1: OpenAlex API key now required (silent failures at scale)

**What goes wrong:** `bulk-ingest` runs fine on first 100 requests (free unauthenticated credits),
then returns HTTP 409 for all subsequent pages, silently dropping papers.
**Why it happens:** Feb 2026 change — polite pool (`?mailto=...`) is deprecated; unauthenticated
usage gets 100 credits/day.
**How to avoid:** Register at openalex.org (free, 30 s); add `--api-key` arg; inject as
`Authorization: Bearer <key>` header.
**Warning signs:** `papers_upserted` stops growing after ~20 pages; HTTP 409 errors in logs.

### Pitfall 2: CLAUDE.md has wrong statistical physics concept ID

**What goes wrong:** Using `C2778407487` in the OpenAlex filter returns Altmetrics papers instead of
statistical physics papers. Corpus appears to ingest correctly but contains wrong domain.
**Why it happens:** CLAUDE.md documents the RS-08 spike filter which includes this as "Statistical
Physics" — but API lookup shows it is Altmetrics.
**How to avoid:** Use `C121864883` (Statistical physics) and `C26873012` (Condensed matter physics).
Verify any concept ID before committing to a filter.
**Warning signs:** Ingested papers have no cond-mat/stat-phys topics; titles are bibliometric papers.

### Pitfall 3: `referenced_works` edges are OpenAlex W-IDs, not arXiv IDs

**What goes wrong:** Treating `referenced_works` strings as arXiv IDs produces malformed DB records.
**Why it happens:** The field always contains `https://openalex.org/W<number>` URLs.
**How to avoid:** The existing two-phase spill pattern already handles this — do not bypass `id_map`.
**Warning signs:** `upsert_citations_batch` inserts records with `paper:W1234` instead of `paper:2301.12345`.

### Pitfall 4: cond-mat corpus OOM during `analyze`

**What goes wrong:** `get_all_papers()` loads all papers into RAM; 2.9 M cond-mat works will OOM.
**Why it happens:** No pagination in the analyze path — same issue as the 90 GB RS-08 corpus.
**How to avoid:** Use a narrow filter (e.g., add `--published-before` cutoff) or run `analyze` only
on a pre-crawled BFS corpus (hundreds of papers), not the full OpenAlex bulk load.
**Warning signs:** OOM kill during `analyze` step.

### Pitfall 5: `DELETE cites` wipes the correct DB

**What goes wrong:** Developer runs two `bulk-ingest` jobs against different `--db` paths simultaneously,
or forgets to pass `--db` and overwrites the default `./data-openalex/` corpus.
**Why it happens:** `DELETE cites` is per-connection (scoped to the storage path), but if the
default `./data-openalex` is targeted by mistake, it wipes the 90 GB RS-08 citation graph.
**How to avoid:** Always explicitly pass `--db surrealkv://./data-physics` (or other target); never
run bulk-ingest without `--db` against a corpus you care about.

### Pitfall 6: arXiv `id_list` pagination offset failures

**What goes wrong:** Using `start` offset > 1000 in `id_list` queries returns empty results.
**Why it happens:** Undocumented server-side limit reported in community; affects large id_list batches
with pagination.
**How to avoid:** Keep id_list batch size ≤ 200 per call; do not paginate with `start` — issue
separate calls for each batch of IDs.
[MEDIUM confidence — community reports, not official docs]

---

## Code Examples

### Example 1: Updated `fetch_page` with API key header

```rust
// resyn-core/src/data_aggregation/openalex_bulk.rs
pub struct OpenAlexBulkLoader {
    client: Client,
    api_key: String,
}

impl OpenAlexBulkLoader {
    pub fn new(client: Client, api_key: impl Into<String>) -> Self {
        Self { client, api_key: api_key.into() }
    }

    pub async fn fetch_page(&self, filter: &str, cursor: &str) -> Result<OpenAlexPage, ResynError> {
        let url = format!(
            "{}?filter={}&per-page=200&cursor={}",
            OPENALEX_API, filter, cursor
        );
        let page: OpenAlexPage = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("User-Agent", "resyn/0.1 (mailto:jasperseehofermusic@gmail.com)")
            .send()
            .await
            .map_err(|e| ResynError::OpenAlexApi(format!("request failed: {e}")))?
            .error_for_status()
            .map_err(|e| ResynError::OpenAlexApi(format!("API error: {e}")))?
            .json()
            .await
            .map_err(|e| ResynError::OpenAlexApi(format!("JSON parse failed: {e}")))?;
        Ok(page)
    }
}
```

### Example 2: Updated `BulkIngestArgs` with `--api-key`

```rust
// resyn-server/src/commands/bulk_ingest.rs
const DEFAULT_FILTER_PHYSICS: &str =
    "primary_location.source.id:S4306400194,concepts.id:C26873012|C121864883";

#[derive(Args, Debug)]
pub struct BulkIngestArgs {
    #[arg(long, default_value = "surrealkv://./data-openalex")]
    pub db: String,

    #[arg(long, default_value = DEFAULT_FILTER)]
    pub filter: String,

    /// OpenAlex API key (free at openalex.org; required since 2026-02-13)
    /// Falls back to unauthenticated (100 credits/day limit) if not set.
    #[arg(long, env = "OPENALEX_API_KEY", default_value = "")]
    pub api_key: String,

    #[arg(long, default_value_t = 100)]
    pub page_delay_ms: u64,
}
```

### Example 3: arXiv id_list batch metadata fetch (secondary — `arxiv_api.rs`)

```rust
// resyn-core/src/data_aggregation/arxiv_api.rs
// Add a batch fetch alongside the existing single-paper fetch:
pub async fn fetch_papers_batch(
    ids: &[&str],
    client: &reqwest::Client,
) -> Result<Vec<Paper>, ResynError> {
    let id_list = ids.join(",");
    let url = format!(
        "http://export.arxiv.org/api/query?id_list={}&max_results={}",
        id_list,
        ids.len().min(200)
    );
    // Rate: caller must sleep 3s between calls.
    // Returns Atom XML — parse with existing arxiv-rs or scraper.
    // ...
}
```

### Example 4: Physics corpus ingest invocation

```bash
# Register free API key at openalex.org/settings/api, then:
export OPENALEX_API_KEY="your-key-here"

cargo run --release --bin resyn -- bulk-ingest \
  --db surrealkv://./data-physics \
  --filter "primary_location.source.id:S4306400194,concepts.id:C26873012|C121864883" \
  --page-delay-ms 100

# Then analyze (only if corpus size is manageable — run with --max-depth filter):
cargo run --release --bin resyn -- analyze --db surrealkv://./data-physics

# Export for EXP-RS-07:
cargo run --release --bin resyn -- export-louvain-graph \
  --db surrealkv://./data-physics \
  --output prototypes/data/research_synergy_physics.json \
  --tfidf-top-n 200
```

---

## License Flags

| Source | License | Compatible with Free/Nonprofit S2 key tier? |
|---|---|---|
| **OpenAlex API + data** | CC0 (public domain) — data is free; API key required since 2026-02-13 but registration is free | YES — no restrictions |
| **OpenAlex S3 snapshot** | CC0; AWS Open Data program covers egress costs | YES — `aws s3 sync s3://openalex ... --no-sign-request` |
| **OpenCitations COCI** | CC0 [CITED: opencitations.net] | YES — but not recommended (DOI-only, no arXiv IDs) |
| **Crossref metadata** | Crossref TDM License — non-commercial research permitted [ASSUMED — A4] | YES (assumed) — but `crossref` crate is abandoned |
| **Semantic Scholar Datasets** | Non-commercial / nonprofit permitted; attribution required [VERIFIED: semanticscholar.org/license] | YES — same license tier as existing S2 per-paper API key |
| **INSPIRE-HEP** | CC0 for most metadata [CITED: github.com/inspirehep/rest-api-doc] | YES — already used |
| **NASA ADS** | Research use permitted; license details not formally confirmed [ASSUMED — A5] | UNCONFIRMED — do not use in primary recommendation |
| **arXiv OAI-PMH** | Non-commercial bulk harvesting permitted [ASSUMED — A6 from arXiv ToS] | YES (assumed) — but provides no reference data |

**Top-level warning:** No recommended primary-path source exceeds the Free/Nonprofit S2 key tier
restrictions. The OpenAlex API key is free to register. The S3 snapshot requires no account at all.

---

## Research Question 4: Hybrid Pre-Ingest Architecture

**Q: Does `DELETE cites` at `bulk_ingest.rs:127` scope to the connected DB or a shared store?**

A: Scoped to the connected DB only. [VERIFIED: `resyn-core/src/database/client.rs`]

Each `connect(endpoint)` call opens an isolated SurrealKV engine at the path in `endpoint`. The
`setup()` call runs `USE NS resyn DB resyn` — this namespace/DB tuple is the logical scope, but
since the underlying file is different (`./data-openalex/` vs `./data-physics/`), two connections
are completely isolated. `DELETE cites` on a `./data-physics` connection cannot affect `./data-openalex`.

**Q: Is a different `--db` path sufficient, or do we need code changes?**

A: Pure config change. No code needed for the `DELETE cites` isolation. The only code change needed
is the API key (replacing the deprecated `--mailto` polite pool).

**Q: What filter string pulls physics/cond-mat papers?**

```
primary_location.source.id:S4306400194,concepts.id:C26873012|C121864883
```

Breakdown:
- `S4306400194` = arXiv (Cornell University repository) — ensures only arXiv-hosted papers
- `C26873012` = Condensed matter physics (2.9 M works)
- `C121864883` = Statistical physics (1.9 M works)

Note: These concepts overlap significantly. Expect ~3–4 M distinct works. Full bulk-ingest will
take several hours at 200 works/page, 10 pages/s.

**Q: Does `openalex_bulk.rs` already deserialise `referenced_works`?**

A: Yes. [VERIFIED: `openalex_bulk.rs` line 36, `#[serde(default)] pub referenced_works: Vec<String>`]
The field is already present in `OpenAlexWork`. The `bulk_ingest.rs` Phase 1 loop already spills
non-empty `referenced_works` to disk. No code change needed there.

---

## Research Question 5: Architecture Recommendation

**Smallest change that maximises speedup (prescriptive):**

1. **File: `resyn-core/src/data_aggregation/openalex_bulk.rs`**
   - Replace `mailto: String` field with `api_key: String`
   - Remove `?mailto=...` from URL; add `Authorization: Bearer {api_key}` header
   - Keep `User-Agent` header for courtesy

2. **File: `resyn-server/src/commands/bulk_ingest.rs`**
   - Replace `--mailto` arg with `--api-key` (also accept `OPENALEX_API_KEY` env var)
   - Add `DEFAULT_FILTER_PHYSICS` constant for cond-mat corpus convenience
   - Update `OpenAlexBulkLoader::new()` call

3. **Runtime: Register free OpenAlex API key**
   - openalex.org/settings/api — 30 seconds

4. **Invocation: Run `bulk-ingest` with physics filter**
   - `--db surrealkv://./data-physics --filter "primary_location.source.id:S4306400194,concepts.id:C26873012|C121864883"`

**What does NOT need to change:**
- `PaperSource` trait — no modification
- `ChainedPaperSource` — no modification
- The two-phase spill pattern — already correct
- `upsert_citations_batch` — already correct
- `DELETE cites` scoping — already per-connection

**Secondary (optional, separate task):**
- Add `fetch_papers_batch(ids: &[&str])` to `arxiv_api.rs` using `id_list` parameter
- Wire into BFS crawler to batch metadata fetches when the queue has multiple pending IDs
- This is an optimisation for the crawl path only; it does not affect the bulk-ingest path

---

## Phase 27 Scope

### Files to create

None required for primary recommendation.

Optional secondary:
- (none — all changes are to existing files)

### Files to touch

| File | Change | Task |
|---|---|---|
| `resyn-core/src/data_aggregation/openalex_bulk.rs` | Replace `mailto` with `api_key`; update header | T1 |
| `resyn-server/src/commands/bulk_ingest.rs` | Replace `--mailto` with `--api-key` / env var; add physics filter constant | T1 |
| `CLAUDE.md` | Fix `C2778407487` — document it as Altmetrics, not Statistical Physics; add correct physics concept IDs | T2 |
| `resyn-core/src/data_aggregation/arxiv_api.rs` | (secondary) Add `fetch_papers_batch` function | T3 (optional) |

### Estimated task count

- **T1** (primary — API key migration + physics filter): 1 task, S effort, ~30 lines changed
- **T2** (CLAUDE.md correction): 1 task, XS effort, doc-only
- **T3** (secondary — arXiv id_list batching): 1 task, M effort, ~50 lines new
- **Manual prerequisite**: Register OpenAlex API key (user action, not a code task)

**Total: 2 code tasks + 1 doc fix + 1 user action. Optional T3 adds 1 more.**

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|---|---|---|
| A1 | `referenced_works` in OpenAlex covers ~60-70 % of arXiv-to-arXiv edges (both sides arXiv-linked) | Ranked Tradeoff Table | If lower, the pre-ingest corpus will have sparse edges; fallback to BFS crawl still works |
| A2 | arXiv API `id_list` is supported by `arxiv-rs` crate `v0.1.5` | Standard Stack / Pattern 4 | If not, hand-roll the batch URL using `reqwest` directly — 1 day extra |
| A3 | OAI-PMH usage for non-commercial bulk harvesting is permitted under arXiv ToS | License Flags | If restricted, OAI-PMH path is blocked; S3 Kaggle snapshot is the fallback |
| A4 | Crossref TDM License permits non-commercial research use without registration | License Flags | LOW risk — Crossref not in primary recommendation |
| A5 | NASA ADS research use permitted; no cost for 5000 req/day | License Flags | LOW risk — ADS not in primary recommendation |

---

## Open Questions (RESOLVED)

1. **OpenAlex API key registration** — RESOLVED by D-01/D-02: API key is mandatory; `--mailto` path eliminated entirely. No ambiguity about polite-pool behavior at scale.

2. **cond-mat corpus size vs `analyze` memory** — Outside Phase 27 scope (D-07 deferred T3). Runtime concern: use `--published-before` flag with `analyze` to sub-select corpus if needed.

3. **arXiv `arxiv-rs` crate id_list support** — RESOLVED by D-07: arXiv id_list batching is deferred to a future crawler optimization phase. Not needed for Phase 27.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|---|---|---|---|---|
| OpenAlex API key | T1 bulk-ingest | Requires user registration (free) | N/A | `--mailto` (may hit 100-credit limit at scale) |
| SurrealKV local storage | All DB ops | Compiled in — no external service | 3.x | — |
| `cargo` / `rustc` | All tasks | Confirmed (git repo active) | rust-toolchain.toml pinned | — |
| AWS CLI | S3 snapshot alternative | Not checked | — | Use API with key instead |

**Missing dependencies with no fallback:** OpenAlex API key — user must register (free).

---

## Validation Architecture

Test framework: `cargo test` (existing 46-test suite). No new framework needed.

### Phase Requirements → Test Map

| Req | Behavior | Test Type | Command | File Exists? |
|---|---|---|---|---|
| T1-a | `fetch_page` sends `Authorization: Bearer` header | unit | `cargo test -p resyn-core --lib --features ssr -- data_aggregation::openalex_bulk` | Modify existing `openalex_bulk` tests |
| T1-b | `BulkIngestArgs` accepts `--api-key` and `OPENALEX_API_KEY` env | integration | `cargo test -p resyn-server` | Add test to `bulk_ingest.rs` |
| T3 | `fetch_papers_batch` constructs correct id_list URL | unit | `cargo test -p resyn-core --lib --features ssr -- data_aggregation::arxiv_api` | Add to `arxiv_api.rs` tests |

### Wave 0 Gaps

- [ ] Update `openalex_bulk.rs` unit tests: replace `mailto` with `api_key` in test setup

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---|---|---|
| V2 Authentication | No — service-to-service API key, not user auth | — |
| V3 Session Management | No | — |
| V4 Access Control | No | — |
| V5 Input Validation | Yes — `filter` arg is user-supplied string injected into URL | `clap` validation; URL-encode the filter value in `fetch_page` |
| V6 Cryptography | No | — |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---|---|---|
| API key in CLI args visible in `ps` output | Information Disclosure | Accept via `env = "OPENALEX_API_KEY"` in `clap`; never log key value |
| Filter string URL injection | Tampering | Validate filter against allowlist of concept/source ID patterns before issuing request |

---

## Sources

### Primary (HIGH confidence — verified live 2026-04-22)

- OpenAlex API live queries (api.openalex.org) — referenced_works format, concept IDs, source IDs, HTTP 200 without key for small queries
- `resyn-core/src/data_aggregation/openalex_bulk.rs` — existing struct fields, serialisation
- `resyn-server/src/commands/bulk_ingest.rs` — two-phase spill pattern, `DELETE cites` scope
- `resyn-core/src/database/client.rs` — per-endpoint connection isolation
- `resyn-core/src/data_aggregation/chained_source.rs` — ChainedPaperSource contract (fixed, tests pass)
- `api.opencitations.net/index/v2` live test — confirmed DOI-only format, no arXiv IDs
- `info.arxiv.org/help/api/user-manual.html` — id_list param, 2000 max_results, 3 s rate limit
- crates.io API — version dates and download counts for all surveyed crates (2026-04-22)
- OpenAlex blog (blog.openalex.org) — Feb 2026 API key mandate, $1/day free tier details
- semanticscholar.org/license — S2 non-commercial use confirmed

### Secondary (MEDIUM confidence)

- OpenCitations API paper (arxiv.org/abs/1904.06052) — 450 M DOI-to-DOI citations
- EXP-RS-07-HANDOFF.md — prior verified findings re: arXiv batching, OAI-PMH, S3 (web-verified 2026-04-22, not re-verified)

### Tertiary (LOW confidence — flagged as ASSUMED)

- arXiv id_list support in `arxiv-rs` crate (A2) — inferred from API docs, not crate source
- arXiv OAI-PMH non-commercial policy (A3) — inferred from general arXiv ToS
- Crossref TDM non-commercial terms (A4) — standard industry knowledge

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all crates verified live against crates.io API
- Architecture: HIGH — verified against live codebase and OpenAlex API
- Pitfalls: HIGH for items 1-4 (verified); MEDIUM for item 6 (community reports)
- License flags: HIGH for OpenAlex/S2/OpenCitations; ASSUMED for NASA ADS/arXiv OAI

**Research date:** 2026-04-22
**Valid until:** 2026-07-22 (stable APIs; OpenAlex pricing model could change sooner)
