# Roadmap: Research Synergy (ReSyn)

## Overview

ReSyn's next milestone adds a full text analysis pipeline to the existing citation graph tool, then uses per-paper structured extractions to surface cross-paper research gaps. The work proceeds in five phases: establish text extraction infrastructure with graceful degradation, run offline NLP analysis, add pluggable LLM semantic extraction, perform cross-paper gap analysis, and finally enrich the visualization with analysis dimensions. Each phase delivers a working, independently testable capability on top of the stable existing foundation.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Text Extraction Foundation** - Fetch and parse paper text with ar5iv HTML, LaTeX fallback, and abstract-only graceful degradation (completed 2026-03-14)
- [ ] **Phase 2: NLP Analysis + DB Schema** - Offline keyword/TF-IDF extraction persisted to SurrealDB with migration infrastructure
- [ ] **Phase 3: Pluggable LLM Backend** - Semantic extraction (methods, findings, open problems) via provider-agnostic LLM trait
- [ ] **Phase 4: Cross-Paper Gap Analysis** - Contradiction detection and ABC-bridge discovery across the citation graph
- [ ] **Phase 5: Visualization Enrichment** - Citation graph nodes colored/sized by analysis dimensions with raw vs enriched toggle

## Phase Details

### Phase 1: Text Extraction Foundation
**Goal**: Papers have extracted text available in a structured form, with every paper annotated by how its text was obtained, and the pipeline never blocks on unavailable full text
**Depends on**: Nothing (builds on existing stable pipeline)
**Requirements**: TEXT-03, TEXT-04, INFR-03, INFR-04
**Success Criteria** (what must be TRUE):
  1. Running `--analyze` on a corpus produces a `TextExtractionResult` for every paper, including papers with no ar5iv HTML
  2. Each extraction record carries an `extraction_method` field (ar5iv_html / abstract_only) visible in logs
  3. Papers without ar5iv HTML are flagged as `partial` and continue through the pipeline without error
  4. `--skip-fulltext` flag causes all papers to use abstract-only extraction
  5. New HTTP fetchers respect the shared rate limiter — no additional per-fetcher sleep logic required
**Plans:** 2/2 plans complete

Plans:
- [x] 01-01-PLAN.md — Data models (TextExtractionResult, SectionMap) + DB migration system + ExtractionRepository
- [x] 01-02-PLAN.md — Ar5ivExtractor with section parsing + CLI flags (--analyze, --skip-fulltext) + pipeline wiring

### Phase 2: NLP Analysis + DB Schema
**Goal**: Every paper in the corpus has keyword rankings and TF-IDF vectors stored in SurrealDB, computed offline without any API calls, and future schema changes apply via migrations rather than manual DDL
**Depends on**: Phase 1
**Requirements**: TEXT-02, INFR-02
**Success Criteria** (what must be TRUE):
  1. Running `--analyze` on a corpus populates a `paper_analysis` table with per-paper keyword rankings and TF-IDF vectors
  2. Re-running `--analyze` on an already-analyzed corpus skips existing records without re-computing them
  3. Applying the migration to an existing populated SurrealDB database succeeds without data loss
  4. Keyword rankings are visible in CLI output (e.g., top-5 keywords per paper logged at info level)
**Plans:** 1/2 plans executed

Plans:
- [ ] 02-01-PLAN.md — PaperAnalysis/AnalysisMetadata data models + DB migrations 3+4 + AnalysisRepository
- [ ] 02-02-PLAN.md — NLP preprocessing + TF-IDF engine + pipeline wiring with corpus fingerprint caching

### Phase 3: Pluggable LLM Backend
**Goal**: Each paper receives structured semantic annotations (methods, findings, open problems) extracted by an LLM, the backend is swappable via CLI flag, and results are cached so re-runs never re-bill API costs for already-analyzed papers
**Depends on**: Phase 2
**Requirements**: TEXT-01, INFR-01
**Success Criteria** (what must be TRUE):
  1. `--llm-provider claude` produces structured annotations (methods, findings, open problems) for each paper and stores them in SurrealDB
  2. `--llm-provider ollama` produces the same structured output using a local model, no internet required beyond the Ollama endpoint
  3. Re-running analysis on an already-annotated corpus skips LLM calls for cached papers (verified by zero API requests in logs)
  4. `--llm-provider noop` runs the full pipeline without any LLM calls, producing empty-but-valid annotation records
**Plans:** 1/3 plans executed

Plans:
- [ ] 03-01-PLAN.md — Data models (LlmAnnotation, Finding, Method) + LLM trait + NoopProvider + migration 5 + LlmAnnotationRepository
- [ ] 03-02-PLAN.md — ClaudeProvider + OllamaProvider with wiremock integration tests
- [ ] 03-03-PLAN.md — CLI flags (--llm-provider, --llm-model) + run_llm_analysis() pipeline wiring

### Phase 4: Cross-Paper Gap Analysis
**Goal**: The system surfaces contradictions between papers and hidden ABC-bridge connections across the citation graph, stored as structured gap findings that can be reviewed by the user
**Depends on**: Phase 3
**Requirements**: GAPS-01, GAPS-02
**Success Criteria** (what must be TRUE):
  1. Running `--analyze` on a corpus with divergent findings produces at least one `contradiction` gap finding in SurrealDB
  2. Running `--analyze` on a corpus produces `abc_bridge` gap findings where a non-obvious A-C connection exists via a shared B intermediary
  3. Gap findings are printed to stdout at analysis completion, grouped by type (contradiction / abc_bridge)
  4. Each gap finding includes the paper IDs involved and a human-readable justification string
**Plans:** 3 plans

Plans:
- [x] 04-01-PLAN.md — GapFinding data model + migration 6 + GapFindingRepository + LlmProvider verify_gap trait extension + gap prompts (completed 2026-03-14)
- [ ] 04-02-PLAN.md — Similarity module + contradiction detector + ABC-bridge discoverer with graph distance
- [ ] 04-03-PLAN.md — Output formatter + CLI flags (--full-corpus, --verbose) + run_gap_analysis() pipeline wiring

### Phase 5: Visualization Enrichment
**Goal**: The citation graph visually encodes analysis dimensions so users can see paper type, primary method, and finding strength at a glance, and can switch between the raw citation view and the enriched view
**Depends on**: Phase 4
**Requirements**: VIS-01, VIS-02
**Success Criteria** (what must be TRUE):
  1. After analysis, graph nodes are colored by paper type (empirical / theoretical / survey / methodology) and sized by finding strength
  2. A toggle control in the graph UI switches between raw citation view (current behavior) and analysis-enriched view
  3. Switching to enriched view with no analysis data present shows the raw graph unchanged, not an error
  4. Hovering a node in enriched view shows extracted keywords and primary method in the tooltip
**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 → 5

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Text Extraction Foundation | 2/2 | Complete   | 2026-03-14 |
| 2. NLP Analysis + DB Schema | 1/2 | In Progress|  |
| 3. Pluggable LLM Backend | 1/3 | In Progress|  |
| 4. Cross-Paper Gap Analysis | 0/3 | Not started | - |
| 5. Visualization Enrichment | 0/TBD | Not started | - |
