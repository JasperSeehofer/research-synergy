# /paper-lookup

Look up how to fetch and integrate an arXiv paper in the Research Synergy codebase.

## Instructions

Given an arXiv paper ID (e.g., `2301.12345`) or search terms from the user:

1. Read `src/data_aggregation/arxiv_api.rs` and `src/data_aggregation/arxiv_utils.rs` to understand the current fetching patterns
2. Explain how the paper would flow through the existing pipeline:
   - How to query it via the arXiv API (using the `search_query_handler` or direct ID lookup)
   - How references get resolved (`arxiv_utils.rs` recursive crawl)
   - How it ends up in the graph (`graph_creation.rs`)
3. If the user wants to add a new data source or modify the aggregation, point to the specific functions and patterns to extend

This is a **read-only diagnostic skill** — do not modify code, just explain the pipeline and integration points.
