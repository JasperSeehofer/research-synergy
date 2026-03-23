# Requirements: Research Synergy (ReSyn)

**Defined:** 2026-03-23
**Core Value:** Surface research gaps and unexplored connections that no single paper reveals — by structurally analyzing and comparing papers across a citation graph.

## v1.1.1 Requirements

Bug fixes for v1.1 web UI features confirmed broken via agent-browser testing.

### Routing

- [ ] **ROUTE-01**: User can navigate to any page via sidebar links without full page reload
- [ ] **ROUTE-02**: User can directly load any route (e.g. `/graph`, `/papers`) via URL or browser refresh

### Graph Rendering

- [ ] **GRAPH-01**: Force-directed layout produces visible node animation spreading nodes apart
- [ ] **GRAPH-02**: Graph nodes render with crisp edges (no DPR blur)
- [ ] **GRAPH-03**: Graph edges (citation links) are visually rendered between connected nodes

### Graph Interaction

- [ ] **INTERACT-01**: User can drag individual nodes to reposition them
- [ ] **INTERACT-02**: User can pan the graph viewport by dragging empty space
- [ ] **INTERACT-03**: User can zoom in/out with scroll wheel

### Temporal Controls

- [ ] **TEMPORAL-01**: Both slider thumbs are visible and draggable independently

## Future Requirements

None — this is a bugfix milestone.

## Out of Scope

| Feature | Reason |
|---------|--------|
| New graph features (search, filtering by analysis) | Bugfix milestone only |
| Performance optimization / profiling | Deferred to next feature milestone |
| New analysis capabilities | No new features in this milestone |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| ROUTE-01 | Phase 11 | Pending |
| ROUTE-02 | Phase 11 | Pending |
| GRAPH-01 | Phase 12 | Pending |
| GRAPH-02 | Phase 12 | Pending |
| GRAPH-03 | Phase 12 | Pending |
| INTERACT-01 | Phase 13 | Pending |
| INTERACT-02 | Phase 13 | Pending |
| INTERACT-03 | Phase 13 | Pending |
| TEMPORAL-01 | Phase 14 | Pending |

**Coverage:**
- v1.1.1 requirements: 9 total
- Mapped to phases: 9
- Unmapped: 0

---
*Requirements defined: 2026-03-23*
*Last updated: 2026-03-23 after roadmap creation*
