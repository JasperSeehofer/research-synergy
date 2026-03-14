# Requirements: Research Synergy (ReSyn)

**Defined:** 2026-03-14
**Core Value:** Surface research gaps and unexplored connections that no single paper reveals

## v1 Requirements

### Text Extraction

- [ ] **TEXT-01**: System extracts structured fields (methods, findings, open problems, paper type) from paper abstracts via LLM
- [ ] **TEXT-02**: System computes corpus-relative keywords per paper using TF-IDF (offline, no API cost)
- [ ] **TEXT-03**: System fetches full text from arXiv HTML (ar5iv) with section detection for papers that have HTML available
- [ ] **TEXT-04**: System falls back gracefully to abstract-only analysis when full text is unavailable, flagging the paper as partial

### Infrastructure

- [ ] **INFR-01**: LLM backend is pluggable via a trait, supporting at least two providers (e.g., Claude API and Ollama)
- [ ] **INFR-02**: Analysis results are cached in SurrealDB per paper; re-runs skip already-analyzed papers
- [x] **INFR-03**: Database schema changes use a migration system to safely extend the existing paper schema
- [ ] **INFR-04**: System provides CLI flags to control analysis pipeline (e.g., `--analyze`, `--llm-provider`, `--skip-fulltext`)

### Cross-Paper Analysis

- [ ] **GAPS-01**: System detects contradictions between papers (divergent findings on the same topic across connected papers)
- [ ] **GAPS-02**: System discovers ABC-model bridges (hidden A↔C connections via shared B intermediaries with semantic justification)

### Visualization

- [ ] **VIS-01**: Citation graph nodes are colored/sized by extracted analysis dimensions (paper type, primary method, finding strength)
- [ ] **VIS-02**: User can toggle between raw citation view and analysis-enriched view

## v2 Requirements

### Cross-Paper Analysis

- **GAPS-03**: Open-problems aggregation across citation graph ranked by recurrence frequency
- **GAPS-04**: Method-combination gap matrix showing existing vs absent method pairings

### Text Extraction

- **TEXT-05**: Section-aware LLM extraction using detected section boundaries for deeper analysis
- **TEXT-06**: Analysis provenance tracking (store which text segment sourced each extraction)

### Visualization

- **VIS-03**: 3D multidimensional projection of paper embeddings (PCA/UMAP) into navigable space
- **VIS-04**: Temporal evolution view layering the graph by publication year

## Out of Scope

| Feature | Reason |
|---------|--------|
| Full-text search / indexing engine | Structured extraction tool, not a search engine; duplicates Semantic Scholar |
| Paper recommendation | Shifts product away from gap surfacing toward discovery feed |
| Real-time collaboration | Adds auth, sessions, conflict resolution; single-user tool is right scope |
| Citation prediction | Different ML problem; conflates with LBD hypothesis generation |
| Non-arXiv PDF scraping | Fragile, legally gray, outside existing data source contract |
| Fine-tuning custom models | Enormous scope; use off-the-shelf LLM APIs with prompt engineering |
| LaTeX source parsing | ar5iv HTML is simpler and sufficient; LaTeX parsing in Rust is high complexity for marginal gain |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| TEXT-01 | Phase 3 | Pending |
| TEXT-02 | Phase 2 | Pending |
| TEXT-03 | Phase 1 | Pending |
| TEXT-04 | Phase 1 | Pending |
| INFR-01 | Phase 3 | Pending |
| INFR-02 | Phase 2 | Pending |
| INFR-03 | Phase 1 | Complete |
| INFR-04 | Phase 1 | Pending |
| GAPS-01 | Phase 4 | Pending |
| GAPS-02 | Phase 4 | Pending |
| VIS-01 | Phase 5 | Pending |
| VIS-02 | Phase 5 | Pending |

**Coverage:**
- v1 requirements: 12 total
- Mapped to phases: 12
- Unmapped: 0

---
*Requirements defined: 2026-03-14*
*Last updated: 2026-03-14 — all 12 v1 requirements mapped to phases 1-5*
