# Feature Landscape

**Domain:** Literature Based Discovery (LBD) — text analysis and cross-paper gap analysis
**Project:** Research Synergy (ReSyn) — milestone adding text analysis to existing citation graph tool
**Researched:** 2026-03-14
**Baseline:** Citation crawl, SurrealDB persistence, force-directed graph visualization are already shipped.

---

## Table Stakes

Features that users of an LBD/research-analysis tool expect to exist. Missing any of these makes the tool feel
incomplete or unreliable before the user even reaches the differentiating features.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Abstract-level text analysis | Abstract is always available; failing to use it when metadata already exists feels like a bug | Low | `summary` field already populated in `Paper` model from arXiv and InspireHEP — zero new data fetching |
| Keyword / term extraction per paper | Every LBD and bibliometric tool surfaces keywords; used for filtering, clustering, and display | Low–Med | TF-IDF across the crawled corpus works well for corpus-relative keywords; supplement with per-paper frequency |
| Per-paper structured summary (methods, findings, limitations) | Users expect a scan-able snapshot; reading 30 raw abstracts to find the angle is friction | Med | LLM extraction with JSON schema output; ~1 LLM call per uncached paper |
| Analysis result caching / idempotency | Re-running a crawl must not re-bill LLM calls for already-analyzed papers | Med | Store structured results in SurrealDB alongside the paper record; check existence before calling LLM |
| Graceful degradation when full text unavailable | arXiv HTML exists for ~70% of papers; tool must not fail silently or crash on the rest | Low | Fall back: abstract-only analysis, flagged as partial; never block the pipeline |
| Incremental analysis (analyze new papers only) | Corpus grows across sessions; users will not accept re-analyzing everything | Low–Med | Keyed by paper ID; SurrealDB upsert pattern already used for papers |
| Pluggable LLM backend | Users have different API access (Claude, OpenAI, Ollama); hard-coding one provider is a dealbreaker for offline or cost-sensitive use | Med | Trait-based, mirroring existing `PaperSource` pattern |

---

## Differentiators

Features that are not expected by default but provide genuine competitive advantage over Connected Papers,
Semantic Scholar, and generic citation visualization tools. These are where ReSyn wins.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Cross-paper gap detection (contradictions, unexplored method combos) | No mainstream citation tool does this; surfacing "Paper A found X, Paper B found not-X" across a graph is the core LBD value | High | Requires structured per-paper annotations first; compare method+finding pairs across connected nodes; LLM summarization of divergence |
| ABC-model bridge discovery | Classic LBD: A cites B, B cites C — but A and C have never been connected; surface these hidden links with semantic justification | High | Graph traversal (SurrealDB graph queries) plus semantic similarity scoring on extracted concepts |
| Open-problems aggregation across the graph | Synthesize what papers collectively admit they have not solved; gives the researcher a ranked list of real gaps | Med–High | Extract open-problem statements per paper via LLM; cluster and deduplicate across corpus; rank by recurrence frequency |
| Method-combination gap matrix | Show which method pairings appear in the literature vs which are conspicuously absent | Med | Build method vocabulary from extractions; compute pairwise occurrence; flag absent pairs as potential research directions |
| Graph enrichment with analysis dimensions | Color/size nodes by extracted category (e.g., methodology paper vs empirical study vs survey); immediately makes the visual graph more informative | Med | Map extracted `paper_type` and `primary_method` onto existing egui visualization layer |
| Hybrid NLP + LLM pipeline (NLP for structure, LLM for semantics) | Keeps LLM costs proportional to value: use cheap NLP for section detection and term frequency; reserve LLM for semantic interpretation | Med | Section boundary detection via heading patterns in arXiv HTML; TF-IDF for term salience; LLM only for interpretation tasks |
| Full-text extraction via arXiv LaTeX source | LaTeX source is the most structured representation available; section labels, equation context, and claim structure are explicit | Med | ar5iv/LaTeXML HTML is the easiest path (already partially used for references); LaTeX .tar.gz is richer but harder to parse in Rust |
| Analysis provenance tracking | Show which text was the source for each extracted claim (abstract vs section); enables user trust and spot-checking | Med | Store source segment + char offsets alongside extracted fields in SurrealDB |
| Temporal evolution view | Show how methods/findings shifted across the citation graph ordered by publication year | Med–High | Requires `published` field (already in `Paper` model); animate or layer the graph by year |

---

## Anti-Features

Things to deliberately not build in this milestone, with explicit rationale.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| Full-text search / indexing engine | This is a structured extraction tool, not a search engine; building keyword search would scatter focus and duplicate Semantic Scholar | Use SurrealDB graph queries on extracted structured fields |
| Paper recommendation ("you might also like") | Recommendation shifts the product away from gap surfacing toward a discovery feed; that is a different product | Stick to analyzing the user-defined corpus, not expanding it automatically |
| Real-time collaborative analysis | Adds auth, session management, conflict resolution; single-user tool is the right scope now | Leave multi-user for a future milestone after single-user is solid |
| Citation prediction / link prediction | Predicting future citations is a different ML problem; conflates with LBD hypothesis generation in confusing ways | Gap analysis on existing citations is enough value |
| Web scraping non-arXiv PDFs | Fragile, legally gray, and outside the existing data source contract | Respect the existing boundary: only papers reachable via arXiv or InspireHEP |
| Fine-tuning / training custom models | Enormous scope, requires labeled data, separate infrastructure; disproportionate to the milestone | Use off-the-shelf LLM APIs and prompt engineering; fine-tuning is a future research project |
| General-purpose NLP library bundled in Rust binary | Rust NLP ecosystem is immature relative to Python; bundling heavy models in the binary inflates compile time and binary size | Use lightweight NLP (regex-based section detection, pure-Rust TF-IDF); delegate semantic work to LLM API calls |

---

## Feature Dependencies

Understanding which features must exist before others can be built.

```
Abstract available (already in Paper.summary)
  └── Abstract-level LLM extraction (methods, findings, open problems)
        ├── Analysis result caching in SurrealDB
        ├── Keyword/term extraction (TF-IDF on abstract corpus)
        └── Graph enrichment with paper_type / primary_method
              └── Temporal evolution view

arXiv HTML / ar5iv full-text retrieval
  └── NLP section detection (heading patterns → section map)
        └── Section-aware LLM extraction (methods section → methods, conclusion → limitations)
              └── Analysis provenance tracking (source segment stored)

Per-paper structured extractions (all papers in corpus)
  ├── Open-problems aggregation (cluster open_problems across graph)
  ├── Method-combination gap matrix (build method vocabulary)
  └── Cross-paper gap detection (compare findings, contradictions)
        └── ABC-model bridge discovery (graph traversal + semantic bridge)

Pluggable LLM backend trait
  └── (All LLM-dependent features above)
```

---

## MVP Recommendation

For the first iteration of this milestone, prioritize features in dependency order:

**Phase 1 — Per-paper analysis foundation (builds the data model everything else needs)**
1. Abstract-level LLM extraction: methods, findings, open problems, paper type — output stored as structured JSON in SurrealDB
2. Analysis result caching and idempotency — prevents redundant API calls across sessions
3. Pluggable LLM backend trait — Claude and local-model (Ollama) as the two initial implementations
4. Keyword/TF-IDF extraction — pure NLP, no API cost, works offline

**Phase 2 — Full-text enrichment (deepens the per-paper data)**
5. arXiv HTML (ar5iv) full-text retrieval and NLP section detection
6. Section-aware LLM extraction using identified section boundaries
7. Analysis provenance tracking

**Phase 3 — Cross-paper gap analysis (the core differentiator)**
8. Open-problems aggregation across the citation graph
9. Method-combination gap matrix
10. Cross-paper contradiction / divergence detection
11. Graph enrichment: color/size nodes by extracted dimensions

**Defer:**
- ABC-model bridge discovery: high complexity, requires all of Phase 1-3 data to be meaningful
- Temporal evolution view: useful but not required for gap surfacing
- 3D visualization: significant egui/fdg effort; deliver 2D enrichment first

---

## Sources

- [Literature-Based Discovery (LBD) survey — Medinformatics 2025](https://ojs.bonviewpress.com/index.php/MEDIN/article/view/5348)
- [A Hybrid Approach to LBD: Traditional Methods with LLMs — MDPI 2025](https://www.mdpi.com/2076-3417/15/16/8785)
- [Recent Advances and Future Directions in LBD — arXiv 2506.12385](https://arxiv.org/abs/2506.12385)
- [Make LBD Great Again Through Reproducible Pipelines — arXiv 2502.16450](https://arxiv.org/abs/2502.16450)
- [Leveraging LLMs for Enhancing LBD — MDPI 2025](https://www.mdpi.com/2504-2289/8/11/146)
- [Literature-based discovery — Wikipedia](https://en.wikipedia.org/wiki/Literature-based_discovery)
- [Structured information extraction from scientific text with LLMs — Nature Communications 2024](https://www.nature.com/articles/s41467-024-45563-x)
- [Extracting accurate materials data with LLMs and prompt engineering — Nature Communications 2024](https://www.nature.com/articles/s41467-024-45914-8)
- [ar5iv 04.2024 HTML5 dataset for arXiv — SIGMathLing](https://sigmathling.kwarc.info/resources/ar5iv-dataset-2024/)
- [arXiv HTML papers: why it matters — arXiv 2402.08954](https://arxiv.org/html/2402.08954v1)
- [Document Parsing Unveiled: Techniques for Structured Extraction — arXiv 2410.21169](https://arxiv.org/html/2410.21169v1)
- [Litmaps vs ResearchRabbit vs Connected Papers comparison — The Effortless Academic 2025](https://effortlessacademic.com/litmaps-vs-researchrabbit-vs-connected-papers-the-best-literature-review-tool-in-2025/)
- [Open Research Knowledge Graph (ORKG)](https://orkg.org/)
- [Text clustering with LLM embeddings — arXiv 2403.15112](https://arxiv.org/html/2403.15112v1)
- [Mining Scientific Papers: NLP-enhanced Bibliometrics — Frontiers](https://www.frontiersin.org/journals/research-metrics-and-analytics/articles/10.3389/frma.2019.00002/full)
