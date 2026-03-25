# Requirements: Research Synergy (ReSyn)

**Defined:** 2026-03-24
**Core Value:** Surface research gaps and unexplored connections that no single paper reveals — by structurally analyzing and comparing papers across a citation graph

## v1.2 Requirements

Requirements for v1.2 Graph Rendering Overhaul. Each maps to roadmap phases.

### Force Layout

- [x] **FORCE-01**: Graph nodes spread into visible clusters reflecting citation structure instead of collapsing to a central blob
- [x] **FORCE-02**: Force coefficients (repulsion, attraction, damping, ideal distance) tuned to produce readable layouts for 300-400 node citation graphs
- [x] **FORCE-03**: Nodes initialized in concentric rings by BFS depth from seed paper for better simulation warm start

### Edge Rendering

- [ ] **EDGE-01**: Regular citation edges visible at-a-glance on the dark (#0d1117) background
- [ ] **EDGE-02**: WebGL2 edges rendered via quad-based triangle geometry instead of 1px-capped LINES primitive
- [ ] **EDGE-03**: Edge color and alpha consistent between Canvas 2D and WebGL2 renderers

### Node Rendering

- [ ] **NODE-01**: Node circles sharp at all sizes using resolution-independent anti-aliasing (fwidth in WebGL2)
- [ ] **NODE-02**: Node borders crisp at all zoom levels (line width scaled by inverse viewport scale)
- [ ] **NODE-03**: Seed paper node visually distinct with gold/amber color and outer ring

### Viewport & Labels

- [ ] **VIEW-01**: Graph auto-fits into viewport after force layout stabilizes
- [ ] **VIEW-02**: Auto-fit does not re-trigger after user manually pans or zooms
- [ ] **LABEL-01**: Labels rendered with priority-ordered collision avoidance (seed first, then by citation count)
- [ ] **LABEL-02**: Convergence indicator shows stabilization status in graph controls

## Future Requirements

### Runtime Configuration

- **CONFIG-01**: User can tune force parameters (repulsion, ideal distance) via UI sliders
- **CONFIG-02**: User can toggle label visibility independently of zoom level

## Out of Scope

| Feature | Reason |
|---------|--------|
| Edge bundling | Obscures individual citation paths; expensive for Canvas 2D |
| Curved/bezier edges | No analytical benefit for citation edges; tangent computation per tick |
| 3D graph rendering | Unnecessary cognitive overhead; camera adds complexity without benefit |
| Custom node shapes | Visual noise; distinguish via color and ring, not shape |
| Per-edge weight controls | Citation graphs are unweighted |
| JavaScript graph libraries | Full Rust/WASM stack — PROJECT.md constraint |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| FORCE-01 | Phase 15 | Complete |
| FORCE-02 | Phase 15 | Complete |
| FORCE-03 | Phase 15 | Complete |
| EDGE-01 | Phase 16 | Pending |
| EDGE-02 | Phase 16 | Pending |
| EDGE-03 | Phase 16 | Pending |
| NODE-01 | Phase 16 | Pending |
| NODE-02 | Phase 16 | Pending |
| NODE-03 | Phase 16 | Pending |
| VIEW-01 | Phase 17 | Pending |
| VIEW-02 | Phase 17 | Pending |
| LABEL-01 | Phase 17 | Pending |
| LABEL-02 | Phase 17 | Pending |

**Coverage:**
- v1.2 requirements: 13 total
- Mapped to phases: 13
- Unmapped: 0

---
*Requirements defined: 2026-03-24*
*Last updated: 2026-03-24 after roadmap creation*
