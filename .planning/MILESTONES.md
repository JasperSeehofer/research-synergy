# Milestones

## v1.0 Analysis Pipeline (Shipped: 2026-03-14)

**Phases completed:** 5 phases, 12 plans, 6 tasks

**Key accomplishments:**
- Full text extraction from ar5iv HTML with section detection and abstract-only graceful degradation
- Offline TF-IDF keyword extraction with corpus fingerprint caching and section-weighted scoring
- Pluggable LLM backend (Claude, Ollama, Noop) with per-paper caching via SurrealDB
- Cross-paper contradiction detection and ABC-bridge discovery with LLM-verified justifications
- Enriched citation graph visualization with paper-type coloring, finding-strength sizing, edge tinting, and hover tooltips

**Stats:** 22 feat commits, 32 files modified, 5,528 lines added, 8,749 total Rust LOC
**Git range:** `c4a6e69..HEAD` (feat(01-01) → feat(05-02))

**Known tech debt:**
- `nlp` module not exported in `lib.rs` (only accessible from binary)
- Phase 4 SUMMARY frontmatter missing `requirements_completed` for GAPS-01/GAPS-02
- Gap findings not wired into visualization layer (stdout only)
- ROADMAP plan checkboxes stale for phases 2-5
- Stale stub comment in `src/llm/ollama.rs:2`

---

