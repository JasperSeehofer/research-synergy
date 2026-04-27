# Phase 28: Forward-citation crawl mode (S2) - Pattern Map

**Mapped:** 2026-04-27
**Files analyzed:** 7 (6 modified + 1 new test file + 1 script + CLAUDE.md)
**Analogs found:** 7 / 8

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `resyn-core/src/data_aggregation/semantic_scholar_api.rs` | service | request-response (paginated) | itself — `fetch_references` method (lines 259–305) | exact (same file, new method mirrors existing) |
| `resyn-core/src/data_aggregation/traits.rs` | trait | request-response | itself — `PaperSource` trait (lines 1–16) | exact (same file, default-method extension) |
| `resyn-core/src/datamodels/paper.rs` | model | transform | itself — `Paper` struct + `get_arxiv_references_ids` (lines 17–79) | exact (same file, transient field + parallel method) |
| `resyn-core/src/database/queries.rs` | service | CRUD | `upsert_citations_batch` (lines 124–139) | exact (same file, inverse-direction variant) |
| `resyn-server/src/commands/crawl.rs` | command/controller | event-driven (BFS queue) | itself — worker loop (lines 354–463) | exact (same file, flag additions + new block in worker) |
| `resyn-core/tests/semantic_scholar_integration.rs` | test | request-response | itself — existing test suite (lines 55–168) | exact (same file, new test cases follow established pattern) |
| `scripts/crawl-feynman-seeds.sh` | script | batch | itself (lines 1–32) | exact (same file, flag addition) |
| `CLAUDE.md` | docs | — | itself (lines 169–180, Important Notes section) | exact (stale note removal + new entries) |

---

## Pattern Assignments

### `resyn-core/src/data_aggregation/semantic_scholar_api.rs`
**Changes:** (a) new builder fields `bidirectional` + `max_forward_citations` with builder methods; (b) new deserializer structs `S2CitationsPage` / `S2CitationItem`; (c) new `fetch_citing_papers` method implementing `PaperSource::fetch_citing_papers`.

**Analog: existing `fetch_references` pagination loop** (lines 259–305):
```rust
async fn fetch_references(&mut self, paper: &mut Paper) -> Result<(), ResynError> {
    let bare_id = crate::utils::strip_version_suffix(&paper.id);
    let mut offset: u32 = 0;
    let limit: u32 = 500;
    let mut all_refs: Vec<S2RefItem> = Vec::new();

    loop {
        let url = format!(
            "{}/paper/arXiv:{}/references?fields=externalIds,title,authors,year&limit={}&offset={}",
            self.base_url, bare_id, limit, offset
        );
        debug!(url, "Fetching references from Semantic Scholar");

        let response = self.get_with_backoff(&url).await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(());  // <-- silently swallow 404
        }
        if !response.status().is_success() {
            return Err(ResynError::SemanticScholarApi(format!(
                "references HTTP {}: {}",
                response.status(), paper.id
            )));
        }

        let body = response.text().await
            .map_err(|e| ResynError::SemanticScholarApi(format!("failed to read body: {e}")))?;

        let page = serde_json::from_str::<S2RefsPage>(&body).map_err(|e| {
            ResynError::SemanticScholarApi(format!("failed to parse response: {e}"))
        })?;

        all_refs.extend(page.data);

        match page.next {
            Some(next_offset) => offset = next_offset,
            None => break,
        }
    }

    paper.references = Self::convert_s2_refs(&all_refs);
    Ok(())
}
```
`fetch_citing_papers` is a direct mirror: replace endpoint suffix `/references` with `/citations`, replace accumulation type from `S2RefItem` to `S2CitationItem`, store into `paper.citing_papers`, and add early exit when `all_refs.len() >= self.max_forward_citations`.

**Analog: existing deserializer structs** (lines 342–352):
```rust
#[derive(Debug, Deserialize)]
struct S2RefsPage {
    data: Vec<S2RefItem>,
    next: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct S2RefItem {
    #[serde(rename = "citedPaper")]
    cited_paper: S2Paper,
}
```
New structs mirror this shape exactly, using `citingPaper` rename:
```rust
#[derive(Debug, Deserialize)]
struct S2CitationsPage {
    data: Vec<S2CitationItem>,
    next: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct S2CitationItem {
    #[serde(rename = "citingPaper")]
    citing_paper: S2Paper,
}
```

**Analog: builder methods** (lines 47–79):
```rust
pub fn with_base_url(mut self, base_url: String) -> Self {
    self.base_url = base_url;
    self
}

pub fn with_rate_limit(mut self, duration: Duration) -> Self {
    self.rate_limiter = if duration.is_zero() { None } else {
        Some(make_rate_limiter(duration))
    };
    self
}
```
New builders `with_bidirectional(mut self, val: bool) -> Self` and `with_max_forward_citations(mut self, n: usize) -> Self` follow the identical pattern.

**Analog: `convert_s2_refs` — S2Paper → Reference conversion** (lines 176–217):
The existing helper already converts an `S2Paper` reference. D-44 from CONTEXT discretion suggests extracting the per-item conversion into a private `convert_s2_paper_to_ref(s2: &S2Paper) -> Reference` helper that both `convert_s2_refs` and the new citations converter call. Pattern to follow:
```rust
fn convert_s2_refs(items: &[S2RefItem]) -> Vec<Reference> {
    items.iter().map(|item| {
        let cited = &item.cited_paper;
        // ... field extraction, arxiv_eprint handling, Link construction ...
        Reference { author, title, links, doi, arxiv_eprint, ..Default::default() }
    }).collect()
}
```

---

### `resyn-core/src/data_aggregation/traits.rs`
**Change:** Add `fetch_citing_papers` as a default (no-op) async method.

**Analog: existing default method** (lines 12–16):
```rust
#[async_trait]
pub trait PaperSource: Send + Sync {
    async fn fetch_paper(&self, id: &str) -> Result<Paper, ResynError>;
    async fn fetch_references(&mut self, paper: &mut Paper) -> Result<(), ResynError>;
    fn source_name(&self) -> &'static str;
    /// For chained sources, returns the inner source that resolved the last fetch.
    /// Default: same as `source_name()`.
    fn last_resolving_source(&self) -> &'static str {
        self.source_name()
    }
}
```
New method follows the same default-impl pattern:
```rust
/// Fetch papers that cite `paper` and store them in `paper.citing_papers`.
/// Default: no-op — only `SemanticScholarSource` overrides this.
async fn fetch_citing_papers(&mut self, _paper: &mut Paper) -> Result<(), ResynError> {
    Ok(())
}
```

---

### `resyn-core/src/datamodels/paper.rs`
**Changes:** Add `citing_papers: Vec<Reference>` transient field; add `get_citing_arxiv_ids` method.

**Analog: existing transient-safe field approach** — `Paper` struct (lines 17–32):
The struct derives `Default`, so a new field with `#[serde(default, skip_serializing)]` gets zero-initialized without DB schema changes. Existing `references: Vec<Reference>` shows the precedent for `Vec<Reference>` fields.

**Analog: `get_arxiv_references_ids`** (lines 63–79):
```rust
pub fn get_arxiv_references_ids(&self) -> Vec<String> {
    self.references
        .iter()
        .filter_map(|r| r.get_arxiv_id().ok())
        .filter(|id| {
            if id.is_empty() {
                tracing::warn!(
                    "Skipping empty arXiv ID in references for paper {}",
                    self.id
                );
                false
            } else {
                true
            }
        })
        .collect()
}
```
`get_citing_arxiv_ids` is a direct copy-and-adapt: replace `self.references` with `self.citing_papers`, update warning string to say "citing_papers".

**New field placement** (after line 27, inside `Paper` struct):
```rust
/// Transient: populated during crawl, never written to DB or JSON export.
#[serde(default, skip_serializing)]
pub citing_papers: Vec<Reference>,
```
Because `Paper` derives `Default`, `PaperRecord::to_paper()` (queries.rs lines 67–88) already sets `references: Vec::new()` explicitly; `citing_papers` will default-to-empty automatically — no change needed there.

---

### `resyn-core/src/database/queries.rs`
**Change:** Add `upsert_inverse_citations_batch(&self, cited_arxiv_id: &str, citing_papers: &[Reference]) -> Result<usize, ResynError>`.

**Analog: `upsert_citations_batch`** (lines 122–139):
```rust
/// Insert citation edges for (from_arxiv_id, to_arxiv_id) pairs without checking
/// whether the target paper exists — dangling edges are acceptable for bulk ingest.
pub async fn upsert_citations_batch(
    &self,
    pairs: &[(String, String)],
) -> Result<usize, ResynError> {
    for (from_id, to_id) in pairs {
        let from_rid = paper_record_id(from_id);
        let to_rid = paper_record_id(to_id);
        self.db
            .query("RELATE $from->cites->$to")
            .bind(("from", from_rid))
            .bind(("to", to_rid))
            .await
            .map_err(|e| ResynError::Database(format!("batch citation failed: {e}")))?;
    }
    Ok(pairs.len())
}
```
The new method has signature `upsert_inverse_citations_batch(&self, cited_arxiv_id: &str, citing_papers: &[Reference])`. For each `Reference` in `citing_papers`, extract `get_arxiv_id()`, then emit `RELATE $from->cites->$to` where `from = citing_arxiv_id` and `to = cited_arxiv_id`. The same dangling-edge-OK approach applies (no existence check on citing paper).

**Analog: `get_citing_papers` SurrealQL pattern** (lines 225–242):
```rust
// Edge direction: from = citing paper (in), to = cited paper (out)
"SELECT ... FROM paper WHERE id IN (SELECT VALUE in FROM cites WHERE out = $rid)"
```
Confirms the `in → cites → out` convention: the `in` end is the paper doing the citing, `out` is the paper being cited. `RELATE $citing->cites->$cited` is therefore correct direction for the inverse-batch method.

---

### `resyn-server/src/commands/crawl.rs`
**Changes:** New flags on `CrawlArgs`; source construction passes new builder calls; worker loop gains a `fetch_citing_papers` block + `upsert_inverse_citations_batch` call.

**Analog: existing `CrawlArgs` flags** (lines 46–100):
```rust
/// Enable parallel crawling; optional value sets max concurrency (default: 4)
#[arg(long, num_args = 0..=1, default_missing_value = "4")]
pub parallel: Option<usize>,

/// Run text extraction and analysis after crawl
#[arg(long, default_value_t = false)]
pub analyze: bool,
```
New flags follow identical clap attribute style:
```rust
/// Fetch both backward (references) and forward (citations) links (S2 source only)
#[arg(long, default_value_t = false)]
pub bidirectional: bool,

/// Maximum forward citations to fetch per paper (default: 500)
#[arg(long, default_value_t = 500)]
pub max_forward_citations: usize,
```

**Analog: `make_single_source` — S2 builder chain** (lines 109–119):
```rust
"semantic_scholar" => {
    let s2 = SemanticScholarSource::from_env(client);
    let s2 = match s2_limiter {
        Some(rl) => s2.with_shared_rate_limiter(rl),
        None => s2,
    };
    Box::new(s2)
}
```
The new builder calls extend this chain before `Box::new(s2)`:
```rust
let s2 = s2.with_bidirectional(args_bidirectional)
            .with_max_forward_citations(args_max_forward_citations);
```
`make_single_source` must receive these as parameters (or capture from `args`).

**Analog: worker loop — `fetch_references` block** (lines 378–385):
```rust
if let Err(e) = source.fetch_references(&mut paper).await {
    warn!(
        paper_id = entry.paper_id.as_str(),
        error = %e,
        "Failed to fetch references"
    );
}
```
The new `fetch_citing_papers` block mirrors this pattern immediately after:
```rust
if args.bidirectional {
    if let Err(e) = source.fetch_citing_papers(&mut paper).await {
        warn!(
            paper_id = entry.paper_id.as_str(),
            error = %e,
            "Failed to fetch citing papers"
        );
    } else {
        // persist inverse edges
        if let Err(e) = paper_repo_task
            .upsert_inverse_citations_batch(&paper.id, &paper.citing_papers)
            .await
        {
            warn!(paper_id = entry.paper_id.as_str(), error = %e,
                  "Failed to upsert inverse citations");
        }
        // enqueue citing papers into BFS queue (same depth logic as forward refs)
        for arxiv_id in paper.get_citing_arxiv_ids() {
            if let Err(e) = queue
                .enqueue_if_absent(&arxiv_id, &seed_id, entry.depth_level + 1)
                .await
            {
                warn!(arxiv_id, error = %e, "Failed to enqueue citing paper");
            }
        }
    }
}
```
The non-S2 source `tracing::warn!` for `--bidirectional` on unsupported sources should be emitted in `fetch_citing_papers` default impl or at the call site when `source_name() != "semantic_scholar"`.

---

### `resyn-core/tests/semantic_scholar_integration.rs`
**Change:** New test cases for the `/citations` endpoint (happy path, 404 silent, pagination, cap).

**Analog: `source_with` helper + happy-path test structure** (lines 55–98):
```rust
fn source_with(mock_uri: String) -> SemanticScholarSource {
    SemanticScholarSource::new(reqwest::Client::new())
        .with_base_url(mock_uri)
        .with_rate_limit(Duration::ZERO)
        .with_backoff_base(Duration::from_millis(10))
}

#[tokio::test]
async fn test_fetch_references_extracts_arxiv_ids() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string(refs_response_mixed()))
        .mount(&server)
        .await;

    let mut source = source_with(server.uri());
    let mut paper = Paper { id: "cond-mat/0010317".to_string(), ..Default::default() };
    source.fetch_references(&mut paper).await.unwrap();

    assert_eq!(paper.references.len(), 3);
    let arxiv_ids = paper.get_arxiv_references_ids();
    assert_eq!(arxiv_ids.len(), 2);
}
```
New tests for `fetch_citing_papers` follow the same structure: add a `citations_response()` helper returning JSON with `citingPaper` objects, then assert on `paper.citing_papers.len()` and `paper.get_citing_arxiv_ids()`.

**Analog: 404 silent test** (lines 100–111):
```rust
#[tokio::test]
async fn test_fetch_paper_not_found() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let source = source_with(server.uri());
    let result = source.fetch_paper("cond-mat/9999999").await;
    assert!(matches!(result, Err(ResynError::PaperNotFound(_))));
}
```
For `fetch_citing_papers` 404, assert `result.is_ok()` and `paper.citing_papers.is_empty()` (mirrors `fetch_references` 404 → `Ok(())`).

**New `source_with` needs builder args for bidirectional tests:**
```rust
fn bidir_source_with(mock_uri: String) -> SemanticScholarSource {
    SemanticScholarSource::new(reqwest::Client::new())
        .with_base_url(mock_uri)
        .with_rate_limit(Duration::ZERO)
        .with_backoff_base(Duration::from_millis(10))
        .with_bidirectional(true)
        .with_max_forward_citations(500)
}
```

---

### `scripts/crawl-feynman-seeds.sh`
**Change:** Add `--bidirectional` to each invocation.

**Analog: existing per-seed invocation** (lines 23–27):
```bash
for ID in "${seeds[@]}"; do
  echo ""
  echo "=== Seeding $ID ==="
  "$BINARY" crawl --paper-id "$ID" --max-depth 2 --source semantic_scholar --parallel 1 --db "$DB"
done
```
Updated form:
```bash
  "$BINARY" crawl --paper-id "$ID" --max-depth 2 --source semantic_scholar --parallel 1 --bidirectional --db "$DB"
```

---

### `CLAUDE.md`
**Changes:** (a) Remove stale `ChainedPaperSource` empty-refs bug note (lines 173) — bug fixed in chained_source.rs:52-85; (b) add `--bidirectional` and `--max-forward-citations` rows to the crawl argument table; (c) add note under Important Notes about bidirectional mode.

**Analog: crawl argument table rows** (CLAUDE.md lines ~108–115, existing table format):
```markdown
| `--source` | `arxiv` | Data source: `arxiv` or `inspirehep` |
| `--db` | `surrealkv://./data` | DB connection string |
```
New rows:
```markdown
| `--bidirectional` | false | Fetch forward citations (citing papers) in addition to references; S2 source only |
| `--max-forward-citations` | 500 | Cap on citing papers fetched per seed paper |
```

---

## Shared Patterns

### Error handling — ResynError propagation
**Source:** `resyn-core/src/data_aggregation/semantic_scholar_api.rs` (fetch_references, lines 274–291)
**Apply to:** `fetch_citing_papers` and `upsert_inverse_citations_batch`
```rust
// 404 → Ok(()) (silent skip)
if response.status() == reqwest::StatusCode::NOT_FOUND {
    return Ok(());
}
// Other non-2xx → Err with endpoint context
if !response.status().is_success() {
    return Err(ResynError::SemanticScholarApi(format!(
        "citations HTTP {}: {}",
        response.status(), paper.id
    )));
}
// Body parse error
serde_json::from_str::<S2CitationsPage>(&body).map_err(|e| {
    ResynError::SemanticScholarApi(format!("failed to parse response: {e}"))
})?;
```

### Database RELATE pattern
**Source:** `resyn-core/src/database/queries.rs` (lines 131–136)
**Apply to:** `upsert_inverse_citations_batch`
```rust
self.db
    .query("RELATE $from->cites->$to")
    .bind(("from", from_rid))
    .bind(("to", to_rid))
    .await
    .map_err(|e| ResynError::Database(format!("batch citation failed: {e}")))?;
```

### Worker loop warn-and-continue pattern
**Source:** `resyn-server/src/commands/crawl.rs` (lines 378–385)
**Apply to:** all new worker-loop blocks in crawl.rs
```rust
if let Err(e) = source.fetch_references(&mut paper).await {
    warn!(
        paper_id = entry.paper_id.as_str(),
        error = %e,
        "Failed to fetch references"
    );
}
```

### Serde transient field pattern
**Source:** `resyn-core/src/datamodels/paper.rs` (implied by existing `references` field + `PaperRecord::to_paper()` not restoring it)
**Apply to:** `Paper::citing_papers` field
```rust
#[serde(default, skip_serializing)]
pub citing_papers: Vec<Reference>,
```

---

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| None | — | — | All phase-28 changes have close analogs in the existing codebase |

---

## Metadata

**Analog search scope:** `resyn-core/src/`, `resyn-server/src/commands/`, `resyn-core/tests/`, `scripts/`
**Files scanned:** 8 (semantic_scholar_api.rs, traits.rs, paper.rs, queries.rs, crawl.rs, semantic_scholar_integration.rs, chained_source.rs, crawl-feynman-seeds.sh)
**Pattern extraction date:** 2026-04-27
