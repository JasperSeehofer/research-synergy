# Concerns

## Technical Debt

### Excessive Cloning
- `Paper` and related types are cloned frequently across layers. The BFS crawler clones `new_referenced_papers` every iteration (`referenced_paper_ids = new_referenced_papers.clone()` at `src/data_aggregation/arxiv_utils.rs:132`).
- `PaperRecord::to_paper()` clones every field individually (`src/database/queries.rs:55-76`).
- Consider using `Arc<Paper>` or passing references where ownership isn't needed.

### InspireHEP Rate Limiting Architecture
- `InspireHepClient::fetch_paper` takes `&self` (immutable) but rate limiting requires mutable state. The rate limiter is only enforced in `fetch_references(&mut self)`, not in `fetch_paper`. This is an architectural workaround that bypasses rate limiting on some API calls.

### `main.rs` Module Visibility
- All module declarations in `main.rs` use `#[allow(dead_code)]` — a workaround because `main.rs` re-declares modules that `lib.rs` also exports. The `#[allow(dead_code)]` masks genuinely unused code warnings.

### URL Conversion Fragility
- `convert_pdf_url_to_html_url` at `src/data_aggregation/arxiv_utils.rs:70-72` uses naive string replacement (`.replace(".pdf", "").replace("pdf", "html")`) — could break on unexpected URL formats.

## Known Issues

### GUI Event Serialization
- `handle_events()` at `src/visualization/force_graph_app.rs:133` uses `.unwrap()` on `serde_json::to_string(&e)` — could panic on non-serializable event payloads.

### FPS Divide-by-Zero Risk
- `update_fps()` divides by `elapsed.as_secs_f32()` which is safe (returns fraction), but the integer check `elapsed.as_secs() >= 1` means FPS updates only once per second, leading to stale values.

## Security Considerations

### No API Response Validation
- arXiv and InspireHEP API responses are trusted without validation. Malformed responses could cause unexpected behavior beyond deserialization errors.

### HTTP Client Timeout
- Single 30-second timeout (`src/utils.rs:14`) may be insufficient for large graph operations, or too generous for simple API calls. No per-request timeout differentiation.

## Performance Concerns

### N+1 HTML Downloads (arXiv Source)
- Each paper requires a separate HTML download for reference extraction. For a BFS crawl of depth 3, this means hundreds of sequential HTTP requests with 3-second rate limiting between each.

### O(n) Citation Retrieval in DB
- `get_cited_papers` and `get_citing_papers` make N+1 queries: one to get IDs, then one `get_paper` per ID (`src/database/queries.rs:167-188`). Could use a single JOIN query.

### Force-Directed Graph Scaling
- Fruchterman-Reingold layout is O(n²) per iteration. Large citation graphs (100+ nodes) will slow the GUI significantly.

### BFS Crawler Memory
- All papers are held in memory as `Vec<Paper>` during crawl. No streaming or pagination — entire citation graph must fit in memory.

## Fragile Areas

### HTML Reference Parsing
- Relies on arXiv HTML structure with `span.ltx_bibblock` CSS selectors (`src/data_aggregation/arxiv_utils.rs:17-18`). Any arXiv HTML redesign will break reference extraction silently (returns 0 references rather than erroring).

### Paper ID Deduplication via Regex
- Version suffix stripping uses `rfind('v')` with digit check (`src/utils.rs:4-9`). Works for arXiv IDs but could misfire on IDs containing 'v' followed by digits in non-version contexts.

### Database Schema Initialization
- Schema is auto-initialized on connection (`src/database/schema.rs`). No migration system or version tracking — schema changes require manual intervention or data loss.

## Test Coverage Gaps

- **No GUI tests** — visualization layer is entirely untested
- **No concurrent DB tests** — all DB tests are sequential single-connection
- **No real arXiv HTML tests** — integration tests use synthetic HTML, real page structure changes won't be caught
- **No full integration test** — no test exercises the complete CLI → crawl → persist → visualize pipeline
- **No performance benchmarks** — no way to detect regressions in crawl speed or graph rendering
