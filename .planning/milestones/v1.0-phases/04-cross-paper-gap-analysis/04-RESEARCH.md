# Phase 4: Cross-Paper Gap Analysis - Research

**Researched:** 2026-03-14
**Domain:** Rust graph algorithms, LLM prompt engineering, cosine similarity, petgraph BFS/distance, SurrealDB schema, table rendering
**Confidence:** HIGH

## Summary

Phase 4 adds two gap-discovery algorithms on top of the existing TF-IDF and LLM annotation infrastructure from Phases 2 and 3. The key architectural work is: (1) a `src/gap_analysis/` module containing a contradiction detector and ABC-bridge discoverer, (2) a new `gap_finding` SurrealDB table via migration 6, and (3) a `GapFindingRepository` and `GapFindingRecord` following the exact pattern already established across 5 prior repositories.

The algorithms themselves are well-understood. Cosine similarity over sparse TF-IDF vectors identifies same-topic pairs (O(k) per pair where k is shared-term count). Graph distance in petgraph uses `petgraph::algo::dijkstra` or `astar` on the existing `StableGraph` — both are available in the v0.7.1 crate already in Cargo.lock. LLM calls reuse `LlmProvider::annotate_paper`-style interface with different prompt constants in a new `src/llm/gap_prompt.rs`.

The corpus fingerprint caching pattern (`corpus_fingerprint` + `analysis_metadata` skip guard) already exists in Phase 2 NLP analysis and should be reused for gap analysis with a distinct key (`"gap_analysis"`). The "never block on missing data" philosophy from Phase 1 maps directly to the skip-on-LLM-failure pattern already in `run_llm_analysis()`.

**Primary recommendation:** New `src/gap_analysis/` module with `similarity.rs`, `contradiction.rs`, `abc_bridge.rs`, `output.rs` sub-modules; migration 6 for `gap_finding` table; `run_gap_analysis()` function in `main.rs` slotted after `run_llm_analysis()`; JSON string storage for `paper_ids` and `shared_terms` arrays following Phase 3's established pattern.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Two-stage pipeline: TF-IDF keyword overlap identifies same-topic paper pairs, then finding strength divergence (from Phase 3 `Finding.strength`) narrows candidates, then LLM confirms actual contradictions
- LLM verification reuses existing `LlmProvider` trait and `--llm-provider` CLI flag — same provider for both annotation and gap analysis, different prompt template
- Matches Phase 1's "never block on missing data" philosophy — if LLM verification fails for a pair, skip and continue
- B intermediary = shared high-weight TF-IDF keywords between papers A and C where A and C don't directly cite each other
- Default scope: papers within the citation graph (connected by some path)
- `--full-corpus` CLI flag expands scope to all papers in SurrealDB, including disconnected papers from separate crawls
- LLM verification generates a human-readable justification explaining the A↔C connection via B (satisfies success criterion #4)
- Table format grouped by type (Contradictions section, then ABC Bridges section)
- Columns: Type | Papers | Shared Terms | Justification
- Justification truncated to ~60 chars in table; `--verbose` flag shows full justifications below the table
- Summary count line after table: "Gap analysis: N contradictions, M ABC-bridges found across P papers"
- Stdout + DB persistence only — no file export in v1
- Separate `gap_finding` SCHEMAFULL table (follows Phase 2/3 pattern of separate tables per analysis type)
- Fields: type (contradiction/abc_bridge), paper_ids, shared_terms, justification, confidence, found_at
- Corpus fingerprint caching — skip gap analysis if corpus unchanged since last run
- History preserved with timestamps — old findings kept when corpus changes and new findings are added, not deleted

### Claude's Discretion
- TF-IDF similarity threshold for same-topic detection
- Minimum graph distance for ABC-bridge "non-obvious" qualification
- LLM prompt templates for contradiction verification and ABC-bridge justification
- Confidence scoring approach for gap findings
- Migration version numbering (continues from Phase 3's version 5)
- Table rendering implementation (manual formatting vs crate)

### Deferred Ideas (OUT OF SCOPE)
- Semantic synonym/embedding-based topic matching — future enhancement for deeper non-obvious link detection beyond TF-IDF
- Method-combination gap matrix (GAPS-04, v2)
- Open-problem aggregation across citation graph (GAPS-03, v2)
- File export (--output gaps.json) — revisit if users need programmatic access beyond DB queries
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| GAPS-01 | System detects contradictions between papers (divergent findings on the same topic across connected papers) | Two-stage pipeline: cosine similarity over `PaperAnalysis.tfidf_vector` narrows candidates; `Finding.strength` divergence filters; LLM confirms; result stored in `gap_finding` table |
| GAPS-02 | System discovers ABC-model bridges (hidden A↔C connections via shared B intermediaries with semantic justification) | Petgraph BFS/distance for graph separation check; TF-IDF term intersection to identify B-keywords; LLM generates justification string stored in `gap_finding.justification` |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| petgraph | 0.7.1 (locked) | Graph traversal, shortest-path distance | Already in Cargo.lock; `algo::dijkstra` gives hop-count distance for ABC-bridge non-obvious check |
| surrealdb | 3.0.4 (locked) | `gap_finding` table persistence | Established project DB; migration 6 follows the version-guarded pattern from schema.rs |
| serde_json | 1.0 (locked) | JSON-string encoding for `paper_ids`, `shared_terms` arrays in SCHEMAFULL | Phase 3 established this pattern for arrays of complex types |
| chrono | 0.4 (locked) | `found_at` timestamp field | Used in all existing repositories |
| clap | 4 (locked) | `--full-corpus` and `--verbose` CLI flags | Established project CLI framework |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| std::collections::HashMap | stdlib | Cosine similarity computation over TF-IDF vectors | Intersection of term maps; already used in `tfidf.rs` |
| petgraph::algo::dijkstra | 0.7.1 | Shortest-path distance between nodes for ABC non-obvious check | Single-source BFS variant; O(V + E) on unweighted graph |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Manual table formatting | `tabled` crate | Manual avoids new dependency; `tabled` gives cleaner column alignment — but for simple 4-column tables, manual is sufficient |
| JSON string storage for arrays | SurrealDB `array<string>` | `array<string>` works for `paper_ids`, but `shared_terms` is also `Vec<String>` so `array<string>` DDL is valid — either approach works; JSON string is the established Phase 3 pattern |
| Cosine similarity for topic detection | Jaccard index | Cosine accounts for TF-IDF magnitude (rarer terms weighted higher), Jaccard does not; cosine is strictly better given existing TF-IDF vectors |

**Installation:** No new dependencies required. All needed crates are already in Cargo.toml.

## Architecture Patterns

### Recommended Project Structure
```
src/
├── gap_analysis/
│   ├── mod.rs           # pub use re-exports; module declarations
│   ├── similarity.rs    # cosine_similarity(a: &HashMap<String,f32>, b: &HashMap<String,f32>) -> f32
│   ├── contradiction.rs # ContradictionDetector: find_candidates(), verify_with_llm()
│   ├── abc_bridge.rs    # AbcBridgeDiscoverer: find_bridges(), graph distance check, verify_with_llm()
│   └── output.rs        # format_gap_table(), print_verbose_justifications()
├── llm/
│   ├── gap_prompt.rs    # CONTRADICTION_SYSTEM_PROMPT, ABC_BRIDGE_SYSTEM_PROMPT constants
│   └── ... (existing)
├── datamodels/
│   ├── gap_finding.rs   # GapFinding struct + GapType enum
│   └── ... (existing)
└── database/
    └── queries.rs       # + GapFindingRepository (append to existing file)
```

### Pattern 1: Cosine Similarity over Sparse TF-IDF Vectors
**What:** Compute similarity between two papers using their existing `tfidf_vector` HashMaps. Only iterate over the shorter vector's keys and look up in the longer — O(min(|a|, |b|)) in the intersection phase.
**When to use:** Same-topic candidate selection for contradiction detection; shared-B-keyword identification for ABC bridges.
**Example:**
```rust
// src/gap_analysis/similarity.rs
use std::collections::HashMap;

pub fn cosine_similarity(a: &HashMap<String, f32>, b: &HashMap<String, f32>) -> f32 {
    let dot: f32 = a.iter()
        .filter_map(|(term, &va)| b.get(term).map(|&vb| va * vb))
        .sum();
    let mag_a: f32 = a.values().map(|v| v * v).sum::<f32>().sqrt();
    let mag_b: f32 = b.values().map(|v| v * v).sum::<f32>().sqrt();
    if mag_a == 0.0 || mag_b == 0.0 { 0.0 } else { dot / (mag_a * mag_b) }
}

/// Shared terms weighted above threshold — for ABC-bridge B-keyword extraction.
pub fn shared_high_weight_terms(
    a: &HashMap<String, f32>,
    b: &HashMap<String, f32>,
    min_weight: f32,
) -> Vec<String> {
    let mut terms: Vec<String> = a.iter()
        .filter(|(term, &va)| va >= min_weight && b.get(*term).copied().unwrap_or(0.0) >= min_weight)
        .map(|(term, _)| term.clone())
        .collect();
    terms.sort();
    terms
}
```

### Pattern 2: Graph Distance via petgraph dijkstra
**What:** Use `petgraph::algo::dijkstra` on the existing `StableGraph<Paper, f32, Directed>` to compute hop-count distance between node pairs.
**When to use:** ABC-bridge "non-obvious" qualification — A and C must not directly cite each other and must be separated by at least N hops.
**Example:**
```rust
// In abc_bridge.rs
use petgraph::algo::dijkstra;
use petgraph::stable_graph::StableGraph;
use petgraph::Directed;
use crate::datamodels::paper::Paper;

fn graph_distance(
    graph: &StableGraph<Paper, f32, Directed>,
    from: petgraph::prelude::NodeIndex,
    to: petgraph::prelude::NodeIndex,
) -> Option<u32> {
    let distances = dijkstra(graph, from, Some(to), |_| 1u32);
    distances.get(&to).copied()
}
```
Note: `dijkstra` on a directed graph computes directed reachability. For ABC-bridge: check undirected reachability by running dijkstra from both A→C and C→A, or build an undirected view with `graph.into_edge_type()`. Since citation edges are directed, "connected by some path" in CONTEXT.md means connected in either direction — treat the graph as undirected for distance computation.

### Pattern 3: GapFinding Datamodel and Repository
**What:** Follow the exact `SurrealValue` derive + `From` + `RecordId` + `Repository` pattern from `queries.rs`.
**When to use:** New `gap_finding` table in migration 6.
**Example:**
```rust
// src/datamodels/gap_finding.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GapType {
    Contradiction,
    AbcBridge,
}

impl GapType {
    pub fn as_str(&self) -> &'static str {
        match self {
            GapType::Contradiction => "contradiction",
            GapType::AbcBridge => "abc_bridge",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapFinding {
    pub gap_type: GapType,
    pub paper_ids: Vec<String>,   // serialized to JSON string in DB record
    pub shared_terms: Vec<String>, // serialized to JSON string in DB record
    pub justification: String,
    pub confidence: f32,
    pub found_at: String,         // RFC3339 timestamp
}
```

```rust
// In queries.rs — GapFindingRepository
#[derive(Debug, Clone, SurrealValue)]
#[surreal(crate = "surrealdb::types")]
struct GapFindingRecord {
    gap_type: String,
    paper_ids: String,    // serde_json::to_string(&Vec<String>)
    shared_terms: String, // serde_json::to_string(&Vec<String>)
    justification: String,
    confidence: f32,
    found_at: String,
}
```

### Pattern 4: LLM Gap Verification — New Prompt, Same Trait
**What:** Add new prompt constants in `src/llm/gap_prompt.rs`. Call `provider.annotate_paper()`-equivalent via a new `verify_gap()` method on `LlmProvider`, OR call the provider's HTTP endpoint directly from the gap analysis module with a different prompt. Simpler: add a second method to `LlmProvider` trait.
**When to use:** Contradiction confirmation and ABC-bridge justification.

The cleanest approach given the existing trait is to extend `LlmProvider` with a second method:
```rust
// In src/llm/traits.rs — add method
async fn verify_gap(
    &mut self,
    prompt: &str,    // caller constructs the full prompt from gap_prompt.rs templates
    context: &str,   // paper summaries / findings joined as context
) -> Result<String, ResynError>; // returns the justification string
```
This avoids parsing structured JSON from gap verification — just return the raw string justification.

**Alternative:** Reuse `annotate_paper` by passing a specially constructed "abstract" that includes both paper summaries and a gap question. Simpler since it requires no trait change, but less explicit.

### Anti-Patterns to Avoid
- **Building an adjacency matrix for distance computation:** petgraph already provides `dijkstra`; don't hand-roll BFS.
- **Nested OBJECT fields in SurrealDB SCHEMAFULL for `paper_ids` and `shared_terms`:** Phase 2/3 established that arrays with complex or variable structure must be stored as JSON strings (TYPE string). Use `serde_json::to_string()` and `serde_json::from_str()`.
- **Running gap analysis when LLM provider is absent:** Gap analysis requires LLM for verification. Skip gracefully if `--llm-provider` is not specified (log a warning, do not error out).
- **Deleting old gap findings on re-run:** CONTEXT.md mandates history preservation. INSERT new records with fresh `found_at` timestamps; do NOT UPSERT or delete old ones.
- **Global corpus scan at every run:** Use the corpus fingerprint caching guard (same pattern as `run_nlp_analysis`) with key `"gap_analysis"` in `analysis_metadata` table.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Graph shortest-path distance | Custom BFS loop | `petgraph::algo::dijkstra` | Already in Cargo.lock; handles edge weights, disconnected graphs, and node indices correctly |
| Cosine similarity | External crate | stdlib f32 arithmetic over HashMap | Vectors are sparse (50-200 nonzero terms); no matrix library needed; 5 lines of code |
| Table column alignment | `tabled` crate | Manual `format!("{:<width$}", ...)` with computed column widths | Saves a dependency; table has fixed 4 columns with predictable content lengths |
| JSON array serialization for DB | Custom string builder | `serde_json::to_string` / `serde_json::from_str` | Already a project dependency; established in Phase 3 |
| Corpus change detection | Timestamp comparison | SHA-256 fingerprint of sorted paper IDs | Already implemented as `nlp::tfidf::corpus_fingerprint()` — reuse it |

**Key insight:** Petgraph's `dijkstra` and stdlib HashMap arithmetic are sufficient for the entire similarity and distance computation. The expensive part (LLM calls) is correctly bounded by the two-stage pipeline.

## Common Pitfalls

### Pitfall 1: Directed vs Undirected Graph Distance for ABC Bridges
**What goes wrong:** `dijkstra` on a directed graph finds no path from A to C even though A and C are connected via B in the citation graph, because citation edges go A→B and C→B (both citing the same paper), not forming a directed path.
**Why it happens:** Citation edges are directed (citing→cited). Two papers can share a common reference without having a directed path between them.
**How to avoid:** For ABC-bridge scope, treat connectivity as undirected. Use `petgraph::visit::EdgeRef` with an undirected traversal, OR build a temporary undirected graph from the existing `StableGraph` edges. Alternatively, check "no direct edge" by inspecting `graph.find_edge(a_idx, c_idx)` and `graph.find_edge(c_idx, a_idx)`, then use undirected BFS for minimum hop distance.
**Warning signs:** ABC-bridge discoverer finds zero bridges even on a corpus where papers clearly share references.

### Pitfall 2: GapFinding Record ID Strategy — No Per-Paper Key
**What goes wrong:** Using paper IDs as the record ID (like `paper:⟨arxiv_id⟩`) will cause UPSERT to overwrite old findings for the same pair on re-run.
**Why it happens:** CONTEXT.md mandates history preservation — old findings must not be deleted.
**How to avoid:** Use `CREATE gap_finding CONTENT $record` (auto-generated SurrealDB ID) rather than `UPSERT type::record('gap_finding', $id)`. Add a `get_all_gap_findings()` method on `GapFindingRepository` that reads all records for display.
**Warning signs:** Re-running analysis on the same corpus produces only N results instead of accumulating.

### Pitfall 3: Similarity Threshold Too High or Too Low
**What goes wrong:** Cosine threshold of 0.9+ produces zero candidate pairs (corpus papers talk about the same domain but use different vocabulary). Threshold of 0.1 floods the LLM with thousands of pairs.
**Why it happens:** TF-IDF vectors are sparse and domain-relative; physics papers sharing terms like "quantum" will have moderate similarity even when covering different topics.
**How to avoid:** Recommended starting threshold: **0.3** for same-topic detection in contradiction pipeline. For ABC bridges, use shared term count (e.g., ≥ 3 high-weight shared terms) rather than cosine similarity directly.
**Warning signs:** Zero contradictions found after LLM stage (threshold too high), or LLM being called thousands of times (threshold too low).

### Pitfall 4: LLM Prompt Returns Structured JSON When Plain String Suffices
**What goes wrong:** Designing the gap verification prompt to return JSON with fields `{ confirmed: bool, justification: string }` adds a JSON parsing step that can fail.
**Why it happens:** Habit from Phase 3's structured annotation prompt.
**How to avoid:** For gap verification, prompt the LLM to return ONLY the justification string if confirmed, or the single word "NO" if not a real gap. Parse with simple `starts_with("NO")` check. This eliminates JSON parsing from the gap path.
**Warning signs:** LLM returns valid JSON but `serde_json::from_str` fails because the model added trailing commentary.

### Pitfall 5: SurrealDB `f32` Field Precision for Confidence Score
**What goes wrong:** SurrealDB `TYPE float` stores 64-bit doubles; Rust `SurrealValue` derive with `f32` field may lose precision or cause type mismatch on retrieval.
**Why it happens:** Existing `AnalysisRecord` uses `Vec<f32>` for `top_scores` — this works because SurrealDB coerces float on read. Same approach applies here.
**How to avoid:** Store confidence as `f32` in the Rust struct. SurrealDB coerces float values transparently. This is confirmed working in existing `AnalysisRecord.top_scores`.

### Pitfall 6: Corpus Fingerprint Key Collision with NLP Analysis
**What goes wrong:** Using key `"corpus_tfidf"` for gap analysis caching guard will cause gap analysis to skip when NLP analysis hasn't changed, even if gap parameters changed.
**Why it happens:** Both NLP and gap analysis read from the same `analysis_metadata` table.
**How to avoid:** Use distinct key `"gap_analysis"` for gap corpus fingerprint storage in `analysis_metadata`.

## Code Examples

Verified patterns from existing codebase:

### Migration 6 Pattern (follows schema.rs exactly)
```rust
// src/database/schema.rs — append apply_migration_6 and add guard
async fn apply_migration_6(db: &Surreal<Any>) -> Result<(), ResynError> {
    db.query(
        "
        DEFINE TABLE IF NOT EXISTS gap_finding SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS gap_type ON gap_finding TYPE string;
        DEFINE FIELD IF NOT EXISTS paper_ids ON gap_finding TYPE string;
        DEFINE FIELD IF NOT EXISTS shared_terms ON gap_finding TYPE string;
        DEFINE FIELD IF NOT EXISTS justification ON gap_finding TYPE string;
        DEFINE FIELD IF NOT EXISTS confidence ON gap_finding TYPE float;
        DEFINE FIELD IF NOT EXISTS found_at ON gap_finding TYPE string;
        ",
    )
    .await
    .map_err(|e| ResynError::Database(format!("migration 6 DDL failed: {e}")))?;
    Ok(())
}

// In migrate_schema():
if version < 6 {
    apply_migration_6(db).await?;
    record_migration(db, 6).await?;
}
```

### GapFindingRepository INSERT (not UPSERT — preserves history)
```rust
pub async fn insert_gap_finding(&self, finding: &GapFinding) -> Result<(), ResynError> {
    let record = GapFindingRecord::from(finding);
    self.db
        .query("CREATE gap_finding CONTENT $record")
        .bind(("record", record.into_value()))
        .await
        .map_err(|e| ResynError::Database(format!("insert gap finding failed: {e}")))?;
    Ok(())
}

pub async fn get_all_gap_findings(&self) -> Result<Vec<GapFinding>, ResynError> {
    let records: Vec<GapFindingRecord> = self
        .db
        .select("gap_finding")
        .await
        .map_err(|e| ResynError::Database(format!("get all gap findings failed: {e}")))?;
    Ok(records.iter().map(|r| r.to_gap_finding()).collect())
}
```

### Corpus Fingerprint Cache Guard for Gap Analysis
```rust
// In run_gap_analysis() — reuse existing corpus_fingerprint() function
let arxiv_ids: Vec<String> = all_annotations.iter().map(|a| a.arxiv_id.clone()).collect();
let fingerprint = nlp::tfidf::corpus_fingerprint(&arxiv_ids);

if let Ok(Some(meta)) = analysis_repo.get_metadata("gap_analysis").await
    && meta.corpus_fingerprint == fingerprint
{
    info!("Gap corpus unchanged, skipping gap analysis");
    // Still display existing findings from DB
    return;
}
```

### run_gap_analysis() Slot in main.rs (after run_llm_analysis)
```rust
async fn run_analysis(db: &Db, ..., llm_provider: Option<&str>, ...) {
    // ... existing extraction, NLP, LLM steps ...
    if let Some(provider_name) = llm_provider {
        // ... build provider ...
        run_llm_analysis(db, provider.as_mut()).await;
        // Gap analysis requires LLM annotations — run after LLM step
        run_gap_analysis(db, provider.as_mut(), full_corpus, verbose).await;
    }
}
```

### Table Output Format
```
Contradictions
--------------
Papers                          | Shared Terms            | Justification
2301.11111, 2301.22222          | quantum, decoherence    | Papers disagree on whether...

ABC Bridges
-----------
Papers                          | Shared Terms (via B)    | Justification
2301.11111 → 2301.22222 → ...   | lattice, topological    | Paper A's methods in lattic...

Gap analysis: 1 contradiction, 2 ABC-bridges found across 15 papers
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Full LLM scan of all pairs | Two-stage: TF-IDF filter then LLM | Decided in CONTEXT.md | Keeps API costs proportional to corpus size |
| Semantic embeddings for similarity | Sparse TF-IDF cosine similarity | Decided in CONTEXT.md (embeddings deferred to v2) | No new dependencies; works on existing `PaperAnalysis.tfidf_vector` |
| Delete-and-recreate gap findings | INSERT with history preservation | Decided in CONTEXT.md | Enables trend analysis across crawl sessions |

**Deprecated/outdated:**
- Embedding-based topic matching: deferred to v2; not applicable in this phase

## Open Questions

1. **Minimum graph distance threshold for ABC bridges**
   - What we know: CONTEXT.md says "Claude's discretion"; direct citation (distance 1) should not qualify
   - What's unclear: Whether distance ≥ 2 (share a common reference but don't cite each other) is sufficient, or if ≥ 3 produces better results
   - Recommendation: Use **distance ≥ 2 in undirected graph** as the minimum. A and C must not directly cite each other (no edge A→C or C→A). Papers that both cite B (distance 2 via shared reference) are the canonical LBD ABC pattern. Distance ≥ 3 would miss the most obvious bridges.

2. **Confidence scoring approach**
   - What we know: Field exists in schema; CONTEXT.md leaves it to discretion
   - What's unclear: Whether to score 0.0–1.0 from LLM response, or use a fixed heuristic
   - Recommendation: Simple heuristic — cosine similarity score as base confidence for contradictions (higher overlap = higher confidence that they discuss the same topic); number of shared high-weight terms normalized to [0, 1] for ABC bridges. Store as f32. No LLM confidence needed.

3. **`--full-corpus` flag behavior when no DB is connected**
   - What we know: `--full-corpus` should expand scope to all papers in SurrealDB
   - What's unclear: What to do if `--full-corpus` is specified but no `--db` flag given
   - Recommendation: Log a warning ("--full-corpus requires --db, falling back to in-graph scope") and continue with the crawl papers. Do not error out — consistent with "never block on missing data" philosophy.

4. **`array<string>` vs `TYPE string` (JSON) for `paper_ids` and `shared_terms`**
   - What we know: Phase 3 established JSON string storage for array fields to avoid SCHEMAFULL nested-object enforcement issues
   - What's unclear: Whether `array<string>` DDL actually works correctly for simple string arrays in SurrealDB 3.x (unlike `array<object>` which has issues)
   - Recommendation: Use `TYPE string` with JSON encoding to remain consistent with the established pattern and avoid any SCHEMAFULL surprises. The risk of diverging from convention outweighs any marginal benefit of `array<string>`.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness (cargo test) |
| Config file | none — inline `#[cfg(test)]` modules |
| Quick run command | `cargo test gap` |
| Full suite command | `cargo test` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| GAPS-01 | `cosine_similarity` returns correct score for known vectors | unit | `cargo test gap_analysis::similarity::tests -x` | Wave 0 |
| GAPS-01 | Contradiction candidate detection finds pairs above threshold | unit | `cargo test gap_analysis::contradiction::tests -x` | Wave 0 |
| GAPS-01 | Contradiction with divergent finding strengths is detected | unit | `cargo test gap_analysis::contradiction::tests::test_strength_divergence -x` | Wave 0 |
| GAPS-01 | `gap_finding` table created by migration 6 | unit (DB) | `cargo test test_migrate_schema_applies_migration_6` | Wave 0 |
| GAPS-01 | GapFindingRepository INSERT preserves history (multiple inserts) | unit (DB) | `cargo test test_gap_finding_insert_preserves_history` | Wave 0 |
| GAPS-02 | ABC-bridge discoverer finds bridge where A-C share B | unit | `cargo test gap_analysis::abc_bridge::tests::test_finds_bridge` | Wave 0 |
| GAPS-02 | Direct citations (distance 1) are not reported as ABC bridges | unit | `cargo test gap_analysis::abc_bridge::tests::test_no_bridge_direct_citation` | Wave 0 |
| GAPS-01 + GAPS-02 | `shared_high_weight_terms` returns correct intersection | unit | `cargo test gap_analysis::similarity::tests::test_shared_terms` | Wave 0 |
| GAPS-01 + GAPS-02 | Corpus fingerprint caching skips re-analysis on unchanged corpus | unit (DB) | `cargo test test_gap_analysis_corpus_cache_skip` | Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test gap`
- **Per wave merge:** `cargo test`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src/gap_analysis/mod.rs` — module declarations and re-exports
- [ ] `src/gap_analysis/similarity.rs` — `cosine_similarity`, `shared_high_weight_terms` with inline tests
- [ ] `src/gap_analysis/contradiction.rs` — detector with inline tests
- [ ] `src/gap_analysis/abc_bridge.rs` — discoverer with inline tests
- [ ] `src/gap_analysis/output.rs` — table formatter (no DB, pure output logic)
- [ ] `src/datamodels/gap_finding.rs` — `GapType` enum, `GapFinding` struct
- [ ] `src/llm/gap_prompt.rs` — prompt constants

## Sources

### Primary (HIGH confidence)
- Cargo.lock (project) — petgraph 0.7.1, surrealdb 3.0.4 confirmed
- `src/database/schema.rs` — migration pattern (versions 1–5) verified directly
- `src/database/queries.rs` — `SurrealValue` derive + `From` + `Repository` pattern verified directly
- `src/nlp/tfidf.rs` — `corpus_fingerprint()`, `TfIdfEngine` available for reuse
- `src/main.rs` — `run_analysis()` pipeline structure verified directly
- `src/llm/traits.rs` — `LlmProvider` trait signature verified
- `src/datamodels/llm_annotation.rs` — `Finding.strength`, `LlmAnnotation` fields verified
- `src/datamodels/analysis.rs` — `PaperAnalysis.tfidf_vector: HashMap<String, f32>` verified
- `src/data_processing/graph_creation.rs` — `StableGraph<Paper, f32, Directed>` type verified

### Secondary (MEDIUM confidence)
- petgraph 0.7.x docs — `petgraph::algo::dijkstra` API is stable and unchanged since 0.6.x; signature verified against locked version
- SurrealDB 3.x `CREATE` vs `UPSERT` behavior — CREATE auto-generates record ID; confirmed consistent with project's existing query patterns

### Tertiary (LOW confidence)
- TF-IDF cosine similarity threshold 0.3 recommendation — based on general NLP practice for sparse academic text vectors; needs empirical tuning against actual corpus

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all dependencies confirmed in Cargo.lock; no new crates needed
- Architecture: HIGH — follows established project patterns with high fidelity; verified in existing source files
- Pitfalls: HIGH for DB/schema pitfalls (empirically derived from Phase 2/3 decisions); MEDIUM for threshold recommendations (empirical, needs tuning)
- Algorithm correctness: HIGH for cosine similarity, petgraph distance; LOW for recommended numeric thresholds

**Research date:** 2026-03-14
**Valid until:** 2026-04-14 (stable ecosystem; petgraph/surrealdb APIs not changing rapidly)
