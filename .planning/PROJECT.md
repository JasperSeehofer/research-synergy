# Research Synergy (ReSyn)

## What This Is

A Rust-powered Literature Based Discovery tool that aggregates academic papers from arXiv and InspireHEP, builds citation graphs, and surfaces hidden connections between research. It crawls paper references via BFS, persists them to SurrealDB, and visualizes citation networks as interactive force-directed graphs. The next evolution adds full text analysis — extracting structured insights from abstracts and PDFs, then performing cross-paper gap analysis to identify contradictions, unexplored method combinations, and open problems across a citation network.

## Core Value

Surface research gaps and unexplored connections that no single paper reveals — by structurally analyzing and comparing papers across a citation graph.

## Requirements

### Validated

- ✓ Fetch paper metadata from arXiv API — existing
- ✓ Fetch paper metadata from InspireHEP API — existing
- ✓ BFS crawl citation references to configurable depth — existing
- ✓ Parse arXiv HTML bibliography for reference extraction — existing
- ✓ Persist papers and citation edges to SurrealDB — existing
- ✓ Load citation graph from database (db-only mode) — existing
- ✓ Build directed citation graph with petgraph — existing
- ✓ Interactive force-directed graph visualization with pan/zoom — existing
- ✓ Paper ID validation (new and old arXiv formats) — existing
- ✓ Rate limiting for arXiv (3s) and InspireHEP (350ms) — existing
- ✓ Pluggable data source architecture via PaperSource trait — existing
- ✓ Version suffix deduplication across all layers — existing

### Active

- [ ] Extract text from paper abstracts (already available in Paper model)
- [ ] Extract full text from paper PDFs/source (best available method per paper)
- [ ] NLP-based extraction: section detection, keyword extraction, TF-IDF
- [ ] LLM-powered semantic extraction: methods, findings, open problems, datasets/tools
- [ ] Pluggable LLM backend trait (Claude, OpenAI, local models)
- [ ] Store structured analysis results in SurrealDB (extend paper schema)
- [ ] Cross-paper gap analysis: contradictions, method gaps, unexplored combinations
- [ ] Enrich citation graph with analysis dimensions (color/size by category)
- [ ] Multidimensional analysis with 3D projection visualization
- [ ] Per-data-source analysis pipeline (arXiv text vs InspireHEP text)

### Out of Scope

- Real-time collaborative analysis — single-user tool for now
- Citation prediction / paper recommendation — focus is on gap surfacing, not suggesting new papers
- Full-text indexing / search engine — analysis is structured extraction, not free-text search
- Non-arXiv PDF sources — only papers reachable through existing data sources

## Context

ReSyn is a brownfield Rust project with ~25 source files across 8 modules. The existing pipeline (crawl → persist → graph → visualize) is stable with 44 tests. The `Paper` model already contains abstracts (`summary` field) from both arXiv and InspireHEP, so abstract analysis can begin immediately without new data fetching.

For full text, arXiv offers three extraction paths: LaTeX source archives (.tar.gz), HTML renderings (ar5iv, already partially scraped for references), and PDF downloads. The best approach should be researched considering availability, structure preservation, and performance.

The hybrid NLP + LLM approach means: NLP handles mechanical extraction (section boundaries, keywords, term frequencies) while LLM handles semantic understanding (interpreting findings, categorizing open problems, identifying methodological approaches). This keeps LLM API costs proportional to insight value rather than text volume.

The LLM backend should be trait-based (similar to existing `PaperSource` pattern) so users can plug in Claude, OpenAI, local models (Ollama), or future providers.

Cross-paper analysis is the core differentiator — individual paper annotations are a means to the end of surfacing gaps across the citation network.

The 3D visualization for multidimensional analysis is a significant new capability beyond the existing 2D force-directed graph. Papers would be positioned based on extracted dimensions (topic similarity, methodological overlap, temporal evolution) and projected into navigable 3D space.

## Constraints

- **Language**: Rust — maintain consistency with existing codebase
- **Database**: SurrealDB — extend existing schema rather than introducing new storage
- **API costs**: LLM calls should be batched/cached to avoid redundant analysis of already-processed papers
- **Rate limits**: Respect arXiv and InspireHEP rate limits during text extraction (same as crawling)
- **Offline capability**: NLP extraction should work fully offline; LLM analysis requires API access

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Hybrid NLP + LLM analysis | NLP for structure/cost efficiency, LLM for semantic depth | — Pending |
| Pluggable LLM backend via trait | Same pattern as PaperSource; future-proofs provider choice | — Pending |
| SurrealDB for analysis storage | Extend existing schema; graph queries natural for cross-paper analysis | — Pending |
| Cross-paper over per-paper focus | Per-paper extraction is a stepping stone; gap analysis is the real value | — Pending |
| 3D visualization for multidimensional analysis | 2D force-directed graph insufficient for multi-category comparison | — Pending |
| Best-available text extraction per paper | Research LaTeX source vs HTML vs PDF; availability varies by paper | — Pending |

---
*Last updated: 2026-03-14 after initialization*
