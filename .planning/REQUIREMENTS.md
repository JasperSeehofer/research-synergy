# Requirements: Research Synergy v1.4

**Defined:** 2026-04-06
**Core Value:** Surface research gaps and unexplored connections that no single paper reveals — by structurally analyzing and comparing papers across a citation graph

## v1.4 Requirements

Requirements for Discovery & Intelligence milestone. Each maps to roadmap phases.

### Search

- [ ] **SRCH-01**: User can search papers by title, abstract, or author from a search bar
- [ ] **SRCH-02**: Search results are ranked by relevance using SurrealDB full-text search
- [ ] **SRCH-03**: User can search from the graph page and viewport pans to the matching node with a highlight flash
- [ ] **SRCH-04**: Papers table integrates search bar for filtering displayed papers

### Similarity

- [ ] **SIM-01**: System computes pairwise cosine similarity from existing TF-IDF vectors and stores top-10 neighbors per paper
- [ ] **SIM-02**: User can view similar papers in a "Similar Papers" tab in the paper detail drawer
- [ ] **SIM-03**: User can toggle similarity edges as an overlay in the graph view
- [ ] **SIM-04**: Similarity is recomputed automatically after TF-IDF analysis completes

### Graph Analytics

- [ ] **GANA-01**: System computes PageRank on the citation graph via power iteration
- [ ] **GANA-02**: System computes betweenness centrality via Brandes' algorithm
- [ ] **GANA-03**: Graph metrics are cached in SurrealDB with corpus-fingerprint invalidation
- [ ] **GANA-04**: User can size graph nodes by metric via a "Size by" dropdown (Uniform / PageRank / Betweenness / Citations)
- [ ] **GANA-05**: Dashboard displays "Most Influential Papers" ranking by PageRank
- [ ] **GANA-06**: N+1 citation queries (`get_cited_papers`/`get_citing_papers`) replaced with single SurrealDB JOINs

### Community Detection

- [ ] **COMM-01**: System detects communities in the citation graph using Louvain modularity optimization
- [ ] **COMM-02**: User can color graph nodes by community via a "Color by" dropdown (BFS Depth / Community / Topic)
- [ ] **COMM-03**: User can view a community summary panel showing top papers, dominant keywords, and shared methods per community

### Discovery

- [ ] **DISC-01**: System generates scored paper recommendations combining similarity, centrality, and community bridge signals
- [ ] **DISC-02**: Each recommendation includes an explanation of why it was suggested
- [ ] **DISC-03**: User can access a "Discover" page with seed picker and recommendation cards
- [ ] **DISC-04**: Dashboard shows a "Suggested Reads" card with top 3 recommendations

### Export

- [ ] **EXPO-01**: User can export a single paper or selection as BibTeX from the detail drawer or papers table
- [ ] **EXPO-02**: User can export full paper metadata as CSV from the papers table
- [ ] **EXPO-03**: User can export graph data (nodes, edges, metrics) as JSON from the graph page

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Advanced Analysis

- **ADV-01**: Ensemble LLM analysis cross-validating findings across multiple models
- **ADV-02**: New gap types beyond contradiction/ABC-bridge (methodology divergence, terminology variation)
- **ADV-03**: Problem clustering (grouping variations of the same research theme)

### Data Pipeline

- **DATA-01**: PDF text extraction fallback when ar5iv unavailable
- **DATA-02**: Cross-source linking (arXiv ↔ InspireHEP via DOI/inspire_id)
- **DATA-03**: Batch crawl with multiple seed papers

### Robustness

- **ROBU-01**: GUI test coverage for Leptos pages
- **ROBU-02**: Full E2E integration test (crawl → analyze → visualize)
- **ROBU-03**: Performance benchmarks for 1000+ node graphs

## Out of Scope

| Feature | Reason |
|---------|--------|
| Semantic similarity via embeddings | TF-IDF cosine similarity sufficient for v1.4; embeddings require model hosting |
| Real-time collaborative analysis | Single-user tool |
| User accounts / bookmarks | No auth system; personal tool |
| Graph export as SVG/PNG | JSON export covers programmatic use; screenshot covers visual |
| Temporal graph evolution animation | Complex; visual temporal filtering already exists |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| SRCH-01 | Phase 21 | Pending |
| SRCH-02 | Phase 21 | Pending |
| SRCH-03 | Phase 21 | Pending |
| SRCH-04 | Phase 21 | Pending |
| SIM-01 | Phase 22 | Pending |
| SIM-02 | Phase 22 | Pending |
| SIM-03 | Phase 22 | Pending |
| SIM-04 | Phase 22 | Pending |
| GANA-01 | Phase 23 | Pending |
| GANA-02 | Phase 23 | Pending |
| GANA-03 | Phase 23 | Pending |
| GANA-04 | Phase 23 | Pending |
| GANA-05 | Phase 23 | Pending |
| GANA-06 | Phase 23 | Pending |
| COMM-01 | Phase 24 | Pending |
| COMM-02 | Phase 24 | Pending |
| COMM-03 | Phase 24 | Pending |
| DISC-01 | Phase 25 | Pending |
| DISC-02 | Phase 25 | Pending |
| DISC-03 | Phase 25 | Pending |
| DISC-04 | Phase 25 | Pending |
| EXPO-01 | Phase 26 | Pending |
| EXPO-02 | Phase 26 | Pending |
| EXPO-03 | Phase 26 | Pending |

**Coverage:**
- v1.4 requirements: 24 total
- Mapped to phases: 24
- Unmapped: 0 ✓

---
*Requirements defined: 2026-04-06*
*Last updated: 2026-04-06 after roadmap creation*
