# Testing

## Framework

- **Test runner:** `cargo test` (built-in Rust test framework)
- **Async tests:** `#[tokio::test]` for async test functions
- **HTTP mocking:** `wiremock` crate for integration tests
- **CI:** fmt check + clippy + tests + tarpaulin coverage

## Test Count and Distribution

**44 tests total:**

| Category | Count | Location |
|---|---|---|
| Unit tests | 30 | Inline `#[cfg(test)]` modules in source files |
| Integration tests (arXiv) | 3 | `tests/html_parsing.rs` |
| Integration tests (InspireHEP) | 5 | `tests/inspirehep_integration.rs` |
| Database tests | 6 | `src/database/queries.rs` |

## Unit Tests by Module

| Module | Tests | What's Tested |
|---|---|---|
| `src/datamodels/paper.rs` | Paper construction, reference handling, arXiv ID extraction |
| `src/data_processing/graph_creation.rs` | Graph building from papers, edge creation, dedup |
| `src/data_aggregation/arxiv_utils.rs` | Reference string parsing, URL conversion (4 tests) |
| `src/data_aggregation/search_query_handler.rs` | Query builder patterns |
| `src/data_aggregation/inspirehep_api.rs` | JSON deserialization, Paper conversion |
| `src/validation.rs` | arXiv ID validation (new/old format), URL validation (4 tests) |
| `src/utils.rs` | Version suffix stripping (7 cases in 1 test) |

## Integration Test Patterns

### wiremock-based HTTP Mocking

Tests start a `MockServer`, mount expectations, and pass the mock URL to clients:

```rust
let mock_server = MockServer::start().await;
Mock::given(method("GET"))
    .and(path("/html/2301.99999"))
    .respond_with(ResponseTemplate::new(200).set_body_string(html))
    .mount(&mock_server)
    .await;
```

### Rate Limit Disabling

All integration tests disable rate limiting for speed:
```rust
ArxivHTMLDownloader::new(client).with_rate_limit(Duration::from_millis(0))
InspireHepClient::new(client).with_rate_limit(Duration::from_millis(0))
```

### Base URL Override

InspireHEP tests redirect API calls to mock server:
```rust
InspireHepClient::new(client).with_base_url(mock_server.uri())
```

## Database Tests

All database tests use in-memory SurrealDB — no external server required:

```rust
let db = connect_memory().await.unwrap();
let repo = PaperRepository::new(&db);
```

**Tests cover:**
- `test_upsert_and_get_paper` — basic CRUD
- `test_upsert_is_idempotent` — duplicate upserts produce 1 record
- `test_paper_exists` — existence check before/after insert
- `test_version_suffix_dedup` — `v1` and `v2` map to same record
- `test_upsert_citations` — citation edge creation + traversal
- `test_get_citation_graph` — multi-hop BFS graph retrieval (A→B→C chain)

## Test Helpers

- `make_test_paper(id, ref_ids)` in `src/database/queries.rs` — creates `Paper` with arXiv reference links
- `mock_arxiv_html_with_references()` in `tests/html_parsing.rs` — returns HTML with mixed arXiv/non-arXiv refs
- `sample_literature_response()` in `tests/inspirehep_integration.rs` — returns InspireHEP JSON fixture
- `Paper::new()` / `Paper { ..Default::default() }` — used throughout for partial construction

## Error Case Testing

- HTTP 404 responses: `test_aggregate_references_html_not_found`, `test_inspirehep_paper_not_found`
- Malformed JSON: `test_inspirehep_malformed_json`
- Empty results: `test_inspirehep_empty_hits`
- Invalid inputs: `test_invalid_arxiv_ids` (7 invalid cases)

## Coverage Gaps

- No GUI/visualization tests (eframe/egui apps are hard to unit test)
- No end-to-end test of full pipeline (CLI → crawl → persist → visualize)
- No concurrent database access tests
- No performance benchmarks
- HTML parsing tests use synthetic HTML, not real arXiv pages
