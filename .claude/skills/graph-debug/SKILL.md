# /graph-debug

Diagnose issues in the Research Synergy graph construction pipeline.

## Instructions

Inspect the graph construction code and report potential issues:

1. Read `src/data_processing/graph_creation.rs` and check for:
   - Duplicate node insertion (paper ID dedup correctness)
   - Version suffix stripping logic (e.g., "2301.12345v2" → "2301.12345")
   - Missing or orphaned edges (references that don't resolve to nodes)
   - Edge weight handling

2. Read `src/visualization/force_graph_app.rs` and check for:
   - Graph-to-visualization mapping issues
   - Node/edge count mismatches between petgraph and the visual layer

3. Report a structured summary:
   - **Nodes**: expected count, any duplicates found
   - **Edges**: expected count, any dangling references
   - **Known issues**: anything that looks like a bug or edge case

This is a **read-only diagnostic skill** — do not modify code unless the user explicitly asks for a fix.
