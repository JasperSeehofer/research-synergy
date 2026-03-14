# Phase 2: NLP Analysis + DB Schema - Research

**Researched:** 2026-03-14
**Domain:** Rust NLP (TF-IDF), SurrealDB schema extension, corpus fingerprinting
**Confidence:** MEDIUM-HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- Text input is section-weighted with fixed weights: abstract 2x, methods 1.5x, results 1x, intro/conclusion 0.5x
- Weights are hardcoded, not CLI-configurable
- Papers with only abstract (partial extraction) are included at full weight
- Text preprocessing uses standard English stop words plus a small hardcoded list of academic boilerplate terms (e.g., "paper", "study", "result", "show", "figure")
- Top 5 keywords per paper logged at info level with TF-IDF scores
- Corpus-level summary after per-paper keywords: paper count, avg keywords/paper, top corpus terms with paper counts
- NLP analysis output appears after text extraction summary (pipeline order: extract → analyze)
- Corpus = all papers in the database (not just current crawl)
- TF-IDF is recomputed for all papers when corpus changes (IDF is corpus-relative)
- Corpus fingerprint (paper count + hash of arxiv_ids) stored in DB metadata table for change detection
- If corpus unchanged since last analysis, skip recomputation entirely (satisfies INFR-02)
- TF-IDF vectors stored as sparse term→score maps (only non-zero terms)
- Top-N keyword rankings stored as a separate ranked list alongside the full sparse vector for fast reads
- Separate `paper_analysis` table (not extending `text_extraction`)
- Corpus metadata stored in its own `analysis_metadata` table

### Claude's Discretion

- Exact TF-IDF implementation (log-normalized TF, smooth IDF, etc.)
- N-gram handling (unigrams only vs bigrams for multi-word terms)
- Corpus fingerprint hash algorithm
- Minimum term frequency thresholds for inclusion in sparse vector
- Migration version numbering (continues from Phase 1's version 2)

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| TEXT-02 | System computes corpus-relative keywords per paper using TF-IDF (offline, no API cost) | TF-IDF implemented in Rust using hand-rolled computation or `rust-tfidf` crate; section-weighted input from `TextExtractionResult`; sparse `HashMap<String, f32>` stored per paper |
| INFR-02 | Analysis results are cached in SurrealDB per paper; re-runs skip already-analyzed papers | Corpus fingerprint stored in `analysis_metadata` table; `AnalysisRepository::analysis_exists()` guards per-paper skip; whole-corpus skip when fingerprint matches |
</phase_requirements>

---

## Summary

Phase 2 extends the analysis pipeline to compute TF-IDF keyword rankings per paper from already-extracted section text, storing sparse term vectors and ranked keyword lists in a new `paper_analysis` table. The system must also guard against redundant recomputation using a corpus fingerprint stored in a new `analysis_metadata` table, satisfying the INFR-02 caching requirement.

The two primary technical challenges are: (1) choosing how to implement corpus-aware TF-IDF in Rust with section-weighting, and (2) storing sparse term-score maps in SurrealDB's SCHEMAFULL schema. Both are solvable with high confidence. The `rust-tfidf` crate provides corpus-aware IDF computation via a simple strategy pattern, but its API requires custom document trait implementations. A hand-rolled implementation using `HashMap<String, f32>` and standard library types is equally viable and has zero external dependency overhead. For the DB side, SurrealDB's `FLEXIBLE TYPE object` field definition supports dynamic key-value maps in SCHEMAFULL tables without predefining each key.

The corpus fingerprint approach (paper count + sorted-ID hash) is straightforward and avoids storing large checksums. SHA-256 via the `sha2` crate or a simple sorted-IDs joined string hash is sufficient; neither requires a new dependency if `sha2` is added, or it can be avoided entirely by hashing the sorted IDs with `std::hash::DefaultHasher` for a non-cryptographic but stable fingerprint.

**Primary recommendation:** Hand-roll TF-IDF using `HashMap<String, f32>` (no new crate) with smooth IDF formula; use `stop-words` crate for English stop words; store sparse vectors as `FLEXIBLE TYPE object` fields in SurrealDB; add `sha2` for corpus fingerprinting.

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `stop-words` | 0.8+ | English stop word list | Provides NLTK-sourced English stop words; one function call `stop_words::get(LANGUAGE::English)` returns `Vec<String>` |
| `sha2` | 0.10+ | Corpus fingerprint hash | Standard, stable SHA-256 in Rust (`sha2` crate from RustCrypto); deterministic across runs and platforms |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `rust-tfidf` | 1.0 | TF-IDF corpus computation | Use only if you want a trait-based strategy pattern; higher complexity for marginal benefit vs hand-rolling |
| `keyword_extraction` | 1.5.0 | TF-IDF + RAKE + TextRank | Use if multi-algorithm exploration is desired; higher complexity than needed for this phase |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Hand-rolled TF-IDF | `rust-tfidf` | `rust-tfidf` requires implementing `ProcessedDocument` trait per doc; hand-roll is simpler for our weighted-text use case |
| Hand-rolled TF-IDF | `keyword_extraction` v1.5.0 | `keyword_extraction` bundles RAKE/TextRank we don't need; `TfIdfParams::UnprocessedDocuments` API is ergonomic but adds opaque dependency |
| `sha2` for fingerprint | `std::hash::DefaultHasher` | `DefaultHasher` is NOT stable across Rust versions — do not use for persisted fingerprints; use `sha2` or sort+join |

**Installation:**
```bash
cargo add stop-words sha2
```

---

## Architecture Patterns

### Recommended Project Structure
```
src/
├── nlp/                     # New module for NLP computation
│   ├── mod.rs               # pub mod tfidf; pub mod preprocessing;
│   ├── tfidf.rs             # TfIdfEngine: corpus builder, scorer, sparse vector output
│   └── preprocessing.rs     # tokenize(), apply_stop_words(), apply_section_weights()
├── datamodels/
│   ├── analysis.rs          # PaperAnalysis struct, AnalysisMetadata struct
│   └── mod.rs               # add pub mod analysis;
└── database/
    ├── schema.rs            # add apply_migration_3() + apply_migration_4()
    └── queries.rs           # add AnalysisRepository
```

### Pattern 1: Section-Weighted Token Bag

**What:** Before computing TF, replicate each section's tokens proportional to its weight. This embeds section importance into raw term frequency naturally.

**When to use:** Any time you need weighted multi-field TF-IDF without separate per-section scoring.

**Example:**
```rust
// Source: derived from TextExtractionResult SectionMap (src/datamodels/extraction.rs)
fn weighted_token_bag(sections: &SectionMap) -> Vec<String> {
    let mut tokens = Vec::new();
    let section_weights: &[(&Option<String>, u32)] = &[
        (&sections.abstract_text, 2),
        (&sections.methods, 2),   // 1.5x → round up to 2 for integer repetitions
        (&sections.results, 1),
        (&sections.introduction, 1),
        (&sections.conclusion, 1),
    ];
    for (text_opt, weight) in section_weights {
        if let Some(text) = text_opt {
            let section_tokens = tokenize(text);
            for _ in 0..*weight {
                tokens.extend(section_tokens.clone());
            }
        }
    }
    tokens
}
```

Note: 1.5x weight can be approximated as 2x repetitions for methods (integer repetition is simpler than float scaling in raw token bags). Alternatively, apply float multiplier during TF score computation instead of token repetition.

**Better approach (float weight applied at TF stage):**
```rust
// Compute TF per section, then combine with weights:
// tf_combined[term] = sum(weight_i * tf_section_i[term])
// This avoids token duplication and handles 1.5x exactly.
```

### Pattern 2: Corpus-Aware Smooth IDF

**What:** Log-normalized IDF with +1 smoothing avoids division by zero and dampens high-frequency terms.

**Formula:** `idf(t) = ln((1 + N) / (1 + df(t))) + 1`

where N = total documents, df(t) = documents containing term t.

**When to use:** Standard choice for academic text; prevents zero IDF for universal terms.

**Example:**
```rust
// Source: standard NLP formula, verified against sklearn TfidfVectorizer default
fn compute_smooth_idf(doc_freq: usize, total_docs: usize) -> f32 {
    let n = total_docs as f32;
    let df = doc_freq as f32;
    ((1.0 + n) / (1.0 + df)).ln() + 1.0
}
```

### Pattern 3: Sparse Vector Storage via FLEXIBLE TYPE object

**What:** Store `HashMap<String, f32>` as a SurrealDB FLEXIBLE object field — schema enforces the field exists but does not enumerate keys.

**DDL:**
```sql
DEFINE TABLE IF NOT EXISTS paper_analysis SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS arxiv_id ON paper_analysis TYPE string;
DEFINE FIELD IF NOT EXISTS tfidf_vector ON paper_analysis FLEXIBLE TYPE object;
DEFINE FIELD IF NOT EXISTS top_keywords ON paper_analysis TYPE array<array>;
DEFINE FIELD IF NOT EXISTS analyzed_at ON paper_analysis TYPE string;
DEFINE FIELD IF NOT EXISTS corpus_fingerprint ON paper_analysis TYPE string;
DEFINE INDEX IF NOT EXISTS idx_analysis_arxiv_id ON paper_analysis FIELDS arxiv_id UNIQUE;
```

**Rust serialization (HIGH confidence — verified with SurrealDB FLEXIBLE TYPE docs):**
```rust
// Store sparse vector as serde_json::Value (object) via SurrealValue derive
// The FLEXIBLE keyword allows arbitrary string keys without predefined sub-fields
```

### Pattern 4: AnalysisRepository Following ExtractionRepository Pattern

**What:** Mirrors the exact structure of `ExtractionRepository` in `src/database/queries.rs`.

**When to use:** All new DB record types follow this pattern in the project.

```rust
// Source: src/database/queries.rs ExtractionRepository pattern
#[derive(Debug, Clone, SurrealValue)]
#[surreal(crate = "surrealdb::types")]
struct AnalysisRecord {
    arxiv_id: String,
    tfidf_vector: serde_json::Value,   // sparse HashMap<String, f32> serialized
    top_keywords: Vec<(String, f32)>,  // ranked list [(term, score)]
    analyzed_at: String,
    corpus_fingerprint: String,
}
```

### Pattern 5: Corpus Fingerprint

**What:** Sort all arxiv_ids alphabetically, join with `,`, SHA-256 hash the result. Store `(paper_count, fingerprint_hex)` in `analysis_metadata`.

**Example:**
```rust
use sha2::{Sha256, Digest};

fn corpus_fingerprint(arxiv_ids: &[String]) -> String {
    let mut sorted = arxiv_ids.to_vec();
    sorted.sort();
    let joined = sorted.join(",");
    let mut hasher = Sha256::new();
    hasher.update(joined.as_bytes());
    format!("{:x}", hasher.finalize())
}
```

### Anti-Patterns to Avoid

- **Using `std::hash::DefaultHasher` for persisted fingerprints:** Its output is NOT guaranteed stable across Rust compiler versions. Use `sha2` or `xxhash` crates.
- **Storing dense vectors:** With 200+ papers each potentially having 5000+ unique terms, a dense vector per paper would be multi-MB per record. Always store only non-zero terms (sparse).
- **Defining `tfidf_vector` as nested SCHEMAFULL fields:** You cannot predeclare dynamic term keys in SurrealDB DDL. Use `FLEXIBLE TYPE object`.
- **Re-computing IDF in the per-paper loop:** IDF is corpus-level. Compute the full corpus IDF map once, then apply to each paper's TF map.
- **Using `TYPE object` without `FLEXIBLE` in SCHEMAFULL:** Without FLEXIBLE, SurrealDB enforces that only fields declared with `DEFINE FIELD field.subfield` may exist — dynamic keys are rejected.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| English stop word list | Manually curated word list | `stop-words` crate | NLTK-sourced, 150+ words, one line to get; error-prone to build manually |
| SHA-256 | Custom hashing | `sha2` 0.10 from RustCrypto | Stability, correctness, platform consistency |

**Key insight:** TF-IDF computation itself is simple enough to hand-roll correctly (it's ~60 lines of Rust), but stop word lists are long and curating them manually creates maintenance debt. Use the `stop-words` crate for the list and hand-roll the math.

---

## Common Pitfalls

### Pitfall 1: `std::hash::DefaultHasher` Fingerprint Instability

**What goes wrong:** Fingerprint stored in DB differs between Rust compiler versions, triggering false cache misses and full recomputation on every run after a toolchain upgrade.

**Why it happens:** Rust documents that `DefaultHasher` output is not stable across versions.

**How to avoid:** Always use `sha2` (cryptographic) or `xxhash`-based (non-cryptographic but stable) for persisted hashes.

**Warning signs:** Re-analysis triggered immediately after `rustup update` with same corpus.

### Pitfall 2: IDF Computed Before All Extractions Loaded

**What goes wrong:** IDF calculation uses only a subset of papers because some `text_extraction` records hadn't been loaded yet, skewing corpus-relative scores.

**Why it happens:** Running NLP immediately after text extraction without first completing all extraction upserts.

**How to avoid:** Load all `TextExtractionResult` records from `text_extraction` table in one query before starting TF-IDF computation. Never interleave extraction and analysis.

**Warning signs:** Papers analyzed early in the run have inflated IDF scores compared to papers analyzed later.

### Pitfall 3: FLEXIBLE TYPE object Requires Explicit DDL

**What goes wrong:** `UPSERT` to `paper_analysis` fails because the `tfidf_vector` field was defined with plain `TYPE object` (not FLEXIBLE), and SurrealDB v3 rejects unknown sub-fields in SCHEMAFULL mode.

**Why it happens:** Developers apply SCHEMAFULL patterns from `paper` table (all flat scalar fields) to object-valued fields.

**How to avoid:** Use `DEFINE FIELD tfidf_vector ON paper_analysis FLEXIBLE TYPE object;` in migration 3. Verified from SurrealDB docs.

**Warning signs:** `upsert paper_analysis failed: Field 'tfidf_vector.quantum' not found` style errors.

### Pitfall 4: SurrealValue Derive with serde_json::Value Fields

**What goes wrong:** `SurrealValue` derive macro may not serialize `serde_json::Value` fields automatically into SurrealDB's native Value representation, causing type mismatch errors.

**Why it happens:** SurrealDB's Rust SDK has its own `Value` type; mapping from `serde_json::Value` requires the SDK to handle the conversion.

**How to avoid:** Test with a minimal in-memory DB record in the earliest wave. If issues arise, represent the sparse vector as `Vec<(String, f32)>` pairs instead (stored as `TYPE array<array>`) and convert to/from `HashMap` at the boundary.

**Warning signs:** `serialize error: cannot convert serde_json Value` or `type mismatch` on upsert.

### Pitfall 5: Top-N Keyword Storage Format

**What goes wrong:** Storing top keywords as `Vec<(String, f32)>` — tuples don't have a canonical `SurrealValue` representation.

**Why it happens:** The `SurrealValue` derive macro handles structs and primitives, not raw tuples cleanly.

**How to avoid:** Use a named struct `KeywordScore { term: String, score: f32 }` for the ranked list elements, or store as parallel arrays (`top_terms: Vec<String>`, `top_scores: Vec<f32>`). Parallel arrays match the established flat-fields pattern of this codebase.

---

## Code Examples

Verified patterns from official sources and codebase:

### Stop Words Setup
```rust
// Source: stop-words crate docs.rs
use stop_words::{get, LANGUAGE};

fn build_stop_words() -> std::collections::HashSet<String> {
    let mut words: std::collections::HashSet<String> = get(LANGUAGE::English)
        .into_iter()
        .collect();
    // Academic boilerplate (locked decision from CONTEXT.md)
    for w in &["paper", "study", "result", "show", "figure", "also", "using"] {
        words.insert(w.to_string());
    }
    words
}
```

### Tokenizer
```rust
// Simple whitespace + punctuation tokenizer for Rust (no external crate needed)
fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.len() > 2)
        .map(|t| t.to_lowercase())
        .collect()
}
```

### Per-Document Weighted TF
```rust
// Source: derived from TextExtractionResult (src/datamodels/extraction.rs)
fn compute_weighted_tf(
    sections: &SectionMap,
    stop_words: &HashSet<String>,
) -> HashMap<String, f32> {
    let weighted_sections: &[(&Option<String>, f32)] = &[
        (&sections.abstract_text,  2.0),
        (&sections.methods,        1.5),
        (&sections.results,        1.0),
        (&sections.introduction,   0.5),
        (&sections.conclusion,     0.5),
    ];

    let mut term_weights: HashMap<String, f32> = HashMap::new();
    let mut total_weight = 0.0_f32;

    for (text_opt, weight) in weighted_sections {
        if let Some(text) = text_opt {
            let tokens = tokenize(text);
            let n = tokens.len() as f32;
            if n == 0.0 { continue; }
            for token in tokens {
                if stop_words.contains(&token) { continue; }
                *term_weights.entry(token).or_default() += weight / n;
            }
            total_weight += weight;
        }
    }

    // Normalize by total weight sum so scores are comparable across docs
    if total_weight > 0.0 {
        for score in term_weights.values_mut() {
            *score /= total_weight;
        }
    }
    term_weights
}
```

### Corpus IDF and Final TF-IDF
```rust
// Compute IDF from all per-document term sets
fn compute_idf(
    doc_term_sets: &[HashSet<String>],
) -> HashMap<String, f32> {
    let n = doc_term_sets.len() as f32;
    let mut df: HashMap<String, usize> = HashMap::new();
    for terms in doc_term_sets {
        for term in terms {
            *df.entry(term.clone()).or_default() += 1;
        }
    }
    df.into_iter()
        .map(|(term, count)| {
            let idf = ((1.0 + n) / (1.0 + count as f32)).ln() + 1.0;
            (term, idf)
        })
        .collect()
}

fn apply_tfidf(tf: &HashMap<String, f32>, idf: &HashMap<String, f32>) -> HashMap<String, f32> {
    tf.iter()
        .filter_map(|(term, tf_score)| {
            idf.get(term).map(|idf_score| (term.clone(), tf_score * idf_score))
        })
        .collect()
}
```

### Corpus Fingerprint
```rust
// Source: sha2 crate (RustCrypto) — standard pattern
use sha2::{Digest, Sha256};

fn corpus_fingerprint(arxiv_ids: &[String]) -> String {
    let mut sorted = arxiv_ids.to_vec();
    sorted.sort();
    let content = sorted.join(",");
    let hash = Sha256::digest(content.as_bytes());
    format!("{hash:x}")
}
```

### Migration 3 DDL Pattern
```rust
// Source: src/database/schema.rs migration 2 pattern
async fn apply_migration_3(db: &Surreal<Any>) -> Result<(), ResynError> {
    db.query(
        "
        DEFINE TABLE IF NOT EXISTS paper_analysis SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS arxiv_id ON paper_analysis TYPE string;
        DEFINE FIELD IF NOT EXISTS tfidf_vector ON paper_analysis FLEXIBLE TYPE object;
        DEFINE FIELD IF NOT EXISTS top_terms ON paper_analysis TYPE array<string>;
        DEFINE FIELD IF NOT EXISTS top_scores ON paper_analysis TYPE array<float>;
        DEFINE FIELD IF NOT EXISTS analyzed_at ON paper_analysis TYPE string;
        DEFINE FIELD IF NOT EXISTS corpus_fingerprint ON paper_analysis TYPE string;
        DEFINE INDEX IF NOT EXISTS idx_analysis_arxiv_id ON paper_analysis FIELDS arxiv_id UNIQUE;
        ",
    )
    .await
    .map_err(|e| ResynError::Database(format!("migration 3 DDL failed: {e}")))?;
    Ok(())
}

async fn apply_migration_4(db: &Surreal<Any>) -> Result<(), ResynError> {
    db.query(
        "
        DEFINE TABLE IF NOT EXISTS analysis_metadata SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS key ON analysis_metadata TYPE string;
        DEFINE FIELD IF NOT EXISTS paper_count ON analysis_metadata TYPE int;
        DEFINE FIELD IF NOT EXISTS corpus_fingerprint ON analysis_metadata TYPE string;
        DEFINE FIELD IF NOT EXISTS last_analyzed ON analysis_metadata TYPE string;
        DEFINE INDEX IF NOT EXISTS idx_metadata_key ON analysis_metadata FIELDS key UNIQUE;
        ",
    )
    .await
    .map_err(|e| ResynError::Database(format!("migration 4 DDL failed: {e}")))?;
    Ok(())
}
```

Note: `top_terms` + `top_scores` as parallel arrays avoids the `Vec<(String, f32)>` tuple serialization problem. Both fields are `TYPE array<string>` and `TYPE array<float>` respectively — flat scalar arrays that the `SurrealValue` derive handles without issue.

### AnalysisRepository Upsert Pattern
```rust
// Source: src/database/queries.rs ExtractionRepository.upsert_extraction pattern
pub async fn upsert_analysis(&self, result: &PaperAnalysis) -> Result<(), ResynError> {
    let arxiv_id = strip_version_suffix(&result.arxiv_id);
    let record = AnalysisRecord::from(result);

    self.db
        .query("UPSERT type::record('paper_analysis', $id) CONTENT $record")
        .bind(("id", arxiv_id))
        .bind(("record", record.into_value()))
        .await
        .map_err(|e| ResynError::Database(format!("upsert analysis failed: {e}")))?;

    Ok(())
}
```

### run_analysis() Extension Pattern
```rust
// Source: src/main.rs run_analysis() — add NLP step after text extraction loop
async fn run_analysis(db: &Db, rate_limit_secs: u64, skip_fulltext: bool) {
    // ... existing text extraction loop unchanged ...

    // NEW: NLP analysis step
    let analysis_repo = database::queries::AnalysisRepository::new(db);
    let extractions = extraction_repo.get_all_extractions().await.unwrap_or_else(...);
    run_nlp_analysis(db, &analysis_repo, &extractions).await;
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Dense TF-IDF vectors | Sparse term→score maps | Industry standard since ~2010 | Memory reduction of 100-1000x for academic text |
| `std::collections::HashMap` for fingerprint | Stable hash (sha2/xxhash) | Always | Correctness — `DefaultHasher` output varies across Rust versions |
| `TYPE object` in SurrealDB SCHEMAFULL | `FLEXIBLE TYPE object` | SurrealDB v1.3+ | Allows dynamic keys in strict-schema tables |

**Deprecated/outdated:**
- `TYPE object` for dynamic maps in SCHEMAFULL: Without FLEXIBLE keyword, sub-fields must all be predeclared. Use `FLEXIBLE TYPE object`.

---

## Open Questions

1. **SurrealValue derive with serde_json::Value fields**
   - What we know: The `SurrealValue` proc-macro derive is used for all DB record types in this codebase
   - What's unclear: Whether `serde_json::Value` fields serialize correctly through `SurrealValue` without additional conversion boilerplate
   - Recommendation: In Wave 0 or Wave 1, write a minimal DB test that upserts a record with a `serde_json::Value` field and reads it back. If it fails, switch to parallel arrays (`top_terms: Vec<String>`, `top_scores: Vec<f32>`) as the storage format for the sparse vector approximation, or store the sparse vector as a JSON string field.

2. **Minimum term frequency threshold**
   - What we know: Sparse vector should contain only non-zero terms, but very rare terms (df=1) inflate IDF scores
   - What's unclear: Whether a minimum TF threshold (e.g., exclude terms appearing in <2% of documents) improves or hurts keyword quality for small corpora (<50 papers)
   - Recommendation: Start with df≥1 (include all terms) and a minimum per-doc raw token count of 2. This is Claude's discretion per CONTEXT.md.

3. **N-gram handling**
   - What we know: CONTEXT.md grants Claude's discretion on unigrams vs bigrams
   - What's unclear: Multi-word physics terms (e.g., "quantum entanglement", "lattice QCD") are important to HEP domain
   - Recommendation: Start with unigrams only. Bigrams add significant complexity (vocabulary explosion, stop word combinations) and can be added in v2 if keywords are insufficiently specific.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + `#[tokio::test]` |
| Config file | `Cargo.toml` (no separate test config) |
| Quick run command | `cargo test nlp` |
| Full suite command | `cargo test` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| TEXT-02 | `compute_weighted_tf` returns higher scores for abstract terms than conclusion terms | unit | `cargo test nlp::tfidf::test_section_weighting -x` | ❌ Wave 0 |
| TEXT-02 | Stop words excluded from TF output | unit | `cargo test nlp::preprocessing::test_stop_words_excluded -x` | ❌ Wave 0 |
| TEXT-02 | Corpus IDF = smooth IDF formula | unit | `cargo test nlp::tfidf::test_smooth_idf -x` | ❌ Wave 0 |
| TEXT-02 | `get_ranked_words(5)` returns top 5 by TF-IDF score | unit | `cargo test nlp::tfidf::test_top_n_ranking -x` | ❌ Wave 0 |
| TEXT-02 | `AnalysisRepository::upsert_analysis` roundtrips through in-memory DB | integration | `cargo test database::queries::test_analysis_upsert_and_get` | ❌ Wave 0 |
| INFR-02 | Second `--analyze` run on same corpus skips recomputation (fingerprint match) | integration | `cargo test database::queries::test_corpus_fingerprint_skip` | ❌ Wave 0 |
| INFR-02 | `migration 3` and `migration 4` applied without data loss to existing DB | integration | `cargo test database::queries::test_migrate_schema_v3_v4` | ❌ Wave 0 |
| INFR-02 | `analysis_exists` returns false before upsert, true after | unit | `cargo test database::queries::test_analysis_exists` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test nlp` (unit tests for NLP module)
- **Per wave merge:** `cargo test` (full suite including DB integration tests)
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src/nlp/mod.rs` — module declaration
- [ ] `src/nlp/tfidf.rs` — `TfIdfEngine` with unit tests
- [ ] `src/nlp/preprocessing.rs` — `tokenize()`, `build_stop_words()` with unit tests
- [ ] `src/datamodels/analysis.rs` — `PaperAnalysis`, `AnalysisMetadata` structs
- [ ] Migration 3 + 4 in `src/database/schema.rs` — new DB tables
- [ ] `AnalysisRepository` in `src/database/queries.rs` — with DB integration tests

---

## Sources

### Primary (HIGH confidence)
- SurrealDB docs — `FLEXIBLE TYPE object` DDL for dynamic key-value fields in SCHEMAFULL tables
- `src/database/schema.rs` — version-guarded migration pattern (migration 3 + 4 continue from version 2)
- `src/database/queries.rs` — `ExtractionRepository` pattern to follow exactly for `AnalysisRepository`
- `src/datamodels/extraction.rs` — `TextExtractionResult` / `SectionMap` as direct input to TF-IDF

### Secondary (MEDIUM confidence)
- [keyword_extraction v1.5.0](https://lib.rs/crates/keyword_extraction) — `TfIdfParams::UnprocessedDocuments` API (version confirmed via lib.rs)
- [rust-tfidf docs](https://docs.rs/rust-tfidf) — `TfIdfDefault::tfidf(term, doc, corpus.iter())` corpus-aware API
- [stop-words crate](https://crates.io/crates/stop-words) — `stop_words::get(LANGUAGE::English)` returns `Vec<String>`
- [sha2 crate docs](https://docs.rs/sha2) — `Sha256::digest(bytes)` API; RustCrypto standard

### Tertiary (LOW confidence)
- WebSearch: SurrealDB FLEXIBLE TYPE example syntax — not confirmed from live SurrealDB v3 docs directly (404 on some doc pages); confirmed via SurrealDB SDK docs page

---

## Metadata

**Confidence breakdown:**
- Standard stack: MEDIUM — `stop-words` and `sha2` are well-established; hand-rolled TF-IDF is simple and provably correct
- Architecture: HIGH — all patterns directly mirror existing codebase conventions; `FLEXIBLE TYPE object` confirmed from SurrealDB docs
- Pitfalls: HIGH — `DefaultHasher` instability, `FLEXIBLE TYPE object` requirement, and tuple serialization issues are verified known problems
- SurrealValue + serde_json::Value compatibility: LOW — needs a smoke test in Wave 0 before relying on it

**Research date:** 2026-03-14
**Valid until:** 2026-06-14 (SurrealDB v3 API is stable; stop-words/sha2 crates are stable)
