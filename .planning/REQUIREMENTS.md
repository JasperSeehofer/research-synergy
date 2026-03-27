# Requirements: Research Synergy (ReSyn)

**Defined:** 2026-03-27
**Core Value:** Surface research gaps and unexplored connections that no single paper reveals — by structurally analyzing and comparing papers across a citation graph

## v1.3 Requirements

Requirements for v1.3 Data Pipeline Fixes. Each maps to roadmap phases.

### arXiv Crawl

- [ ] **ARXIV-01**: User can crawl arXiv papers and see citation edges stored for references that mention arXiv IDs in plain text (not just hyperlinked)
- [ ] **ARXIV-02**: User can see published dates for all crawled papers (backfilled from arXiv API for reference-only papers)
- [ ] **ARXIV-03**: User can run an arXiv crawl and get comparable edge density to InspireHEP for the same seed paper

### Orphan Nodes

- [ ] **ORPH-01**: User can identify why specific nodes appear disconnected after an InspireHEP crawl
- [ ] **ORPH-02**: User sees zero orphan nodes in the graph for a standard depth-2+ crawl (every node has at least one edge)

### LLM Analysis

- [ ] **LLM-01**: User can trigger LLM analysis from the web UI and see it complete successfully
- [ ] **LLM-02**: User can view gap findings (contradictions, ABC-bridges) in the web UI after analysis
- [ ] **LLM-03**: User can view open problems panel with results ranked by recurrence
- [ ] **LLM-04**: User can view method heatmap showing existing vs absent pairings

## Future Requirements

### Data Enrichment

- **ENRICH-01**: Temporal slider filtering shows visible effect with backfilled publication dates
- **ENRICH-02**: Papers crawled via reference parsing get full metadata enrichment from arXiv API

## Out of Scope

| Feature | Reason |
|---------|--------|
| New data sources beyond arXiv/InspireHEP | Focus is fixing existing pipelines, not adding new ones |
| Custom LLM fine-tuning | Use off-the-shelf APIs with prompt engineering |
| Real-time collaborative analysis | Single-user tool |
| New visualization features | v1.2 just shipped graph rendering overhaul |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| ARXIV-01 | TBD | Pending |
| ARXIV-02 | TBD | Pending |
| ARXIV-03 | TBD | Pending |
| ORPH-01 | TBD | Pending |
| ORPH-02 | TBD | Pending |
| LLM-01 | TBD | Pending |
| LLM-02 | TBD | Pending |
| LLM-03 | TBD | Pending |
| LLM-04 | TBD | Pending |

**Coverage:**
- v1.3 requirements: 9 total
- Mapped to phases: 0
- Unmapped: 9 ⚠️

---
*Requirements defined: 2026-03-27*
*Last updated: 2026-03-27 after initial definition*
