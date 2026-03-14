# Architecture Patterns

**Domain:** Hybrid NLP + LLM text analysis pipeline with 3D multidimensional visualization on an existing Rust LBD citation graph tool
**Researched:** 2026-03-14

---

## Recommended Architecture

The new milestone adds two independent capability tracks onto the existing pipeline:

1. **Text Analysis Pipeline** — extracts structured insights from paper text, stores results in SurrealDB, runs cross-paper gap analysis
2. **3D Visualization Layer** — projects high-dimensional paper embeddings into navigable 3D space using wgpu alongside the existing egui 2D view

Both tracks share the existing data model (`Paper`) and SurrealDB schema (extended). Neither requires changes to the crawl or BFS layers.

```
┌─────────────────────────────────────────────────────────────────┐
│                          main.rs                                 │
│  CLI args → validate → source → crawl → persist → graph → GUI   │
└──────────────────────────┬──────────────────────────────────────┘
                           │  Vec<Paper> (existing canonical state)
          ┌────────────────┴────────────────┐
          │                                 │
          ▼                                 ▼
┌─────────────────────┐          ┌──────────────────────┐
│  TEXT ANALYSIS      │          │  VISUALIZATION        │
│  PIPELINE (new)     │          │  (existing + new 3D)  │
│                     │          │                        │
│  text_extraction/   │          │  visualization/        │
│    traits.rs        │          │    force_graph_app.rs  │
│    arxiv_html.rs    │          │    drawers.rs          │
│    arxiv_latex.rs   │          │    settings.rs         │
│                     │          │                        │
│  nlp_analysis/      │          │  visualization_3d/ (new)
│    tfidf.rs         │          │    scatter_app.rs      │
│    section_detect.rs│          │    camera.rs           │
│    keywords.rs      │          │    wgpu_renderer.rs    │
│                     │          └──────────────────────┘
│  llm_backend/       │
│    traits.rs        │
│    claude.rs        │
│    openai.rs        │
│    ollama.rs        │
│                     │
│  gap_analysis/      │
│    cross_paper.rs   │
│    embeddings.rs    │
│    projection.rs    │
└─────────────────────┘
          │
          ▼
┌─────────────────────┐
│  database/ (extend) │
│    schema.rs +      │
│    analysis tables  │
│    vector indexes   │
│    queries.rs +     │
│    analysis repo    │
└─────────────────────┘
```

---

## Component Boundaries

### Existing Components (unchanged interface)

| Component | Responsibility | Communicates With |
|-----------|---------------|-------------------|
| `data_aggregation/` | Crawl papers via PaperSource trait | `main.rs`, `datamodels` |
| `database/` | Persist papers and citation edges | `main.rs`, `datamodels` |
| `data_processing/graph_creation.rs` | Build petgraph citation graph | `main.rs`, `datamodels` |
| `visualization/force_graph_app.rs` | 2D force-directed interactive graph | `main.rs`, `petgraph` |
| `datamodels/paper.rs` | Canonical paper domain model | All layers |

### New Components

| Component | Responsibility | Communicates With |
|-----------|---------------|-------------------|
| `text_extraction/` | Fetch and parse full paper text | `datamodels`, `database`, HTTP |
| `nlp_analysis/` | Mechanical extraction: TF-IDF, sections, keywords | `text_extraction`, `datamodels` |
| `llm_backend/` | Semantic extraction via pluggable LLM providers | `nlp_analysis`, HTTP (LLM APIs) |
| `gap_analysis/` | Cross-paper contradiction and gap detection | `database`, `llm_backend`, `datamodels` |
| `visualization_3d/` | 3D scatter of paper embeddings in wgpu | `gap_analysis` (embeddings), `datamodels` |

### Extended Components

| Component | Extension | Reason |
|-----------|-----------|--------|
| `database/schema.rs` | Add `paper_analysis` table, `similarity` relation, vector indexes | Store NLP/LLM results and embedding vectors |
| `database/queries.rs` | Add `AnalysisRepository` (upsert analysis, get embeddings, similarity search) | Analysis results need CRUD separate from paper records |
| `datamodels/` | Add `PaperAnalysis`, `TextExtractionResult`, `GapFinding` structs | New domain objects for analysis pipeline |
| `error.rs` | Add `TextExtraction`, `LlmApi`, `GapAnalysis` variants | New error sources across pipeline |

---

## Data Flow

### Text Analysis Pipeline

```
Vec<Paper> (from crawl or DB load)
    │
    ▼
text_extraction::fetch_full_text(paper)
    │  Strategy (in priority order):
    │    1. ar5iv HTML at https://ar5iv.labs.arxiv.org/html/{id}
    │       → scraper parses <section>, <h2>, <p> semantic tags
    │    2. arXiv LaTeX source at https://arxiv.org/e-print/{id}
    │       → tar.gz download, extract .tex, strip macros
    │    3. Abstract only (already in Paper.summary)
    │  Rate limiting: reuse existing ArxivHTMLDownloader rate limit config
    │
    ▼
TextExtractionResult { sections: Vec<Section>, raw_text: String, source: ExtractionMethod }
    │
    ▼
nlp_analysis::analyze(result)
    │  Runs offline (no network):
    │    - Section boundary detection (heading heuristics on HTML or LaTeX structure)
    │    - TF-IDF term weighting across paper corpus (rust-tfidf or custom)
    │    - Keyword extraction per section
    │    - Term frequency vectors for embedding basis
    │
    ▼
NlpResult { keywords: Vec<String>, tfidf_vector: Vec<f32>, section_map: HashMap<SectionType, String> }
    │
    ▼
llm_backend::extract_semantics(section_map)    [optional, requires API key]
    │  LlmBackend trait (same pattern as PaperSource):
    │    - prompt: structured JSON schema for methods, findings, open_problems, datasets
    │    - impl: ClaudeBackend, OpenAiBackend, OllamaBackend
    │    - cache: skip if paper already has LLM analysis in DB
    │
    ▼
SemanticExtraction { methods: Vec<String>, findings: Vec<String>, open_problems: Vec<String>, datasets: Vec<String> }
    │
    ▼
database::AnalysisRepository::upsert_analysis(paper_id, nlp_result, semantic_extraction)
    │  Stored as:
    │    - paper_analysis:⟨arxiv_id⟩ record (NLP + LLM fields)
    │    - embedding vector stored in paper_analysis.tfidf_embedding (SurrealDB vector field)
    │    - HNSW vector index for similarity search
    │
    ▼
gap_analysis::find_gaps(Vec<PaperAnalysis>)
    │  Cross-paper analysis:
    │    - Contradiction detection: compare findings across papers via LLM pairwise calls
    │    - Method gap detection: methods used in cited papers but absent in citing paper
    │    - Unexplored combinations: method sets that appear disjoint in citation subgraph
    │  Uses SurrealDB graph traversal + vector similarity in combined SurrealQL query
    │
    ▼
Vec<GapFinding> { paper_ids: Vec<String>, gap_type: GapType, description: String, confidence: f32 }
```

### 3D Visualization Pipeline

```
Vec<PaperAnalysis> + Vec<GapFinding>
    │
    ▼
gap_analysis::projection::compute_embeddings(papers)
    │  Dimensionality reduction:
    │    - Input: tfidf_vector per paper (from NLP step above)
    │    - Algorithm: PCA via linfa-reduction (pure Rust, no Python dependency)
    │      → project to 3 principal components
    │    - Output: Vec<(paper_id, [x, y, z])>
    │  Note: UMAP (fast-umap crate) is available but depends on burn ML framework —
    │        prefer PCA for simplicity unless cluster separation is inadequate
    │
    ▼
visualization_3d::ScatterApp::new(points, gap_findings)
    │  Rendering stack:
    │    - wgpu for GPU-accelerated 3D rendering (Vulkan/Metal/DX12/WebGPU)
    │    - egui-wgpu integration crate for UI panels alongside 3D viewport
    │    - Camera: orbit + zoom with mouse drag (custom implementation, ~200 LOC)
    │    - Nodes: instanced sphere rendering, colored by topic cluster or gap type
    │    - Edges: line primitives for citation edges in 3D space
    │    - Sidebar: egui panel with paper metadata on node hover/click
    │
    ▼
Interactive 3D scatter with:
    - Orbit/zoom camera controls
    - Node hover → paper title + key finding tooltip
    - Gap finding highlights (colored edges or halos)
    - Toggle: show/hide citation edges in 3D space
    - Axis labels for principal components
```

### GUI Mode Selection

The application will launch one of two visualization modes based on CLI flag:

```
--view 2d   → existing force_graph_app (default, preserves current behavior)
--view 3d   → new visualization_3d::ScatterApp (requires analysis to have run)
--analyze   → run text analysis pipeline then launch selected view
```

---

## Key Abstractions

### LlmBackend Trait (new, mirrors PaperSource pattern)

```rust
#[async_trait]
pub trait LlmBackend: Send + Sync {
    async fn extract_semantics(
        &self,
        sections: &HashMap<SectionType, String>,
    ) -> Result<SemanticExtraction, ResynError>;

    fn backend_name(&self) -> &'static str;
    fn is_available(&self) -> bool; // check env var / connectivity
}
```

Implementations: `ClaudeBackend` (Anthropic Messages API), `OpenAiBackend` (chat completions), `OllamaBackend` (local HTTP at localhost:11434). Trait object dispatched at runtime via `Box<dyn LlmBackend>`.

Structured output: prompt includes JSON schema definition; response parsed with serde_json. If LLM returns malformed JSON, retry once with stricter prompt, then fall back to NLP-only result.

### TextExtractor Trait (new)

```rust
#[async_trait]
pub trait TextExtractor: Send + Sync {
    async fn extract(&self, paper: &Paper) -> Result<TextExtractionResult, ResynError>;
    fn extraction_method(&self) -> ExtractionMethod;
}
```

Implementations: `Ar5ivExtractor` (HTML scraping, preferred), `LatexSourceExtractor` (tar.gz download + parsing), `AbstractOnlyExtractor` (fallback, always available).

The analysis pipeline tries extractors in order and takes the first successful result. No changes to existing `ArxivHTMLDownloader` — `Ar5ivExtractor` creates its own instance with the same rate limiting infrastructure.

### PaperAnalysis Domain Model (new in `datamodels/`)

```rust
pub struct PaperAnalysis {
    pub paper_id: String,           // normalized arxiv ID
    pub extraction_method: ExtractionMethod,
    pub keywords: Vec<String>,
    pub tfidf_vector: Vec<f32>,     // stored in SurrealDB as vector field
    pub section_map: HashMap<SectionType, String>,
    pub methods: Vec<String>,       // LLM-extracted (optional)
    pub findings: Vec<String>,      // LLM-extracted (optional)
    pub open_problems: Vec<String>, // LLM-extracted (optional)
    pub datasets: Vec<String>,      // LLM-extracted (optional)
    pub analyzed_at: DateTime<Utc>,
    pub llm_backend_used: Option<String>,
}
```

### SurrealDB Schema Extensions

```surql
-- New table: paper analysis results
DEFINE TABLE paper_analysis SCHEMAFULL;
DEFINE FIELD paper_id ON paper_analysis TYPE string;
DEFINE FIELD tfidf_vector ON paper_analysis TYPE array<float>;
DEFINE FIELD keywords ON paper_analysis TYPE array<string>;
DEFINE FIELD methods ON paper_analysis TYPE array<string>;
DEFINE FIELD findings ON paper_analysis TYPE array<string>;
DEFINE FIELD open_problems ON paper_analysis TYPE array<string>;
DEFINE FIELD analyzed_at ON paper_analysis TYPE datetime;

-- Vector index for similarity search
DEFINE INDEX paper_analysis_vector
  ON paper_analysis FIELDS tfidf_vector
  MTREE DIMENSION 512 DIST COSINE;

-- Gap findings table
DEFINE TABLE gap_finding SCHEMAFULL;
DEFINE FIELD paper_ids ON gap_finding TYPE array<string>;
DEFINE FIELD gap_type ON gap_finding TYPE string;   -- contradiction | method_gap | unexplored_combination
DEFINE FIELD description ON gap_finding TYPE string;
DEFINE FIELD confidence ON gap_finding TYPE float;
```

---

## Suggested Build Order

Component dependencies determine phase ordering. Each phase produces a working, testable deliverable.

### Phase 1: Text Extraction Layer

Build first because all analysis depends on having text.

- `text_extraction/traits.rs` — `TextExtractor` trait
- `text_extraction/arxiv_html.rs` — ar5iv scraper (reuse scraper crate already in tree)
- `text_extraction/arxiv_latex.rs` — tar.gz download + .tex strip
- `text_extraction/abstract_only.rs` — always-available fallback
- Extend `error.rs` with `TextExtraction` variant
- Unit tests with wiremock (same pattern as existing arXiv HTML tests)

No DB changes yet. Output: `TextExtractionResult` in memory.

### Phase 2: NLP Analysis

Build on top of Phase 1 text. Pure offline, no API calls, fast iteration.

- `nlp_analysis/section_detect.rs` — heading-based section boundary parser
- `nlp_analysis/tfidf.rs` — TF-IDF scoring using `rust-tfidf` crate
- `nlp_analysis/keywords.rs` — top-N keywords per section and per paper
- Add `NlpResult` to `datamodels/`
- Extend `database/schema.rs` with `paper_analysis` table (no vector index yet)
- Extend `database/queries.rs` with `AnalysisRepository::upsert_nlp`
- Integration tests against in-memory SurrealDB

### Phase 3: LLM Backend

Build after NLP so the LLM receives pre-processed structured sections, not raw text.

- `llm_backend/traits.rs` — `LlmBackend` trait
- `llm_backend/claude.rs` — Anthropic Messages API via reqwest
- `llm_backend/openai.rs` — OpenAI chat completions
- `llm_backend/ollama.rs` — Ollama local HTTP
- Extend `database/schema.rs` with LLM result fields on `paper_analysis`
- Extend `AnalysisRepository` with `upsert_semantic` method
- Cache check: skip LLM call if `paper_analysis.analyzed_at` exists and `llm_backend_used` matches
- Integration tests: wiremock for LLM API responses

### Phase 4: Gap Analysis + Embeddings

Depends on both NLP (for tfidf_vector) and LLM (for findings/methods). Adds vector indexing.

- `gap_analysis/embeddings.rs` — normalize and store tfidf vectors, add HNSW index
- `gap_analysis/cross_paper.rs` — contradiction and method gap detection logic
- `gap_analysis/projection.rs` — PCA via `linfa-reduction` to produce 3D coordinates
- Extend DB schema with `gap_finding` table and HNSW vector index on `paper_analysis`
- Output: `Vec<GapFinding>` + `Vec<(String, [f32; 3])>` coordinates

### Phase 5: 3D Visualization

Depends on Phase 4 for 3D coordinates. Can be developed alongside Phase 4 with mock data.

- `visualization_3d/wgpu_renderer.rs` — wgpu device setup, shader compilation, instanced sphere rendering
- `visualization_3d/camera.rs` — orbit + zoom camera with mouse input
- `visualization_3d/scatter_app.rs` — main app struct integrating wgpu + egui panels
- Extend CLI with `--view 2d|3d` and `--analyze` flags
- Existing 2D visualization remains untouched

---

## Anti-Patterns to Avoid

### Anti-Pattern 1: Using rsnltk or rust-bert for Core NLP
**What:** Importing Python-binding NLP toolkits (rsnltk) or transformer model runners (rust-bert) for keyword/TF-IDF tasks.
**Why bad:** rsnltk requires a Python runtime. rust-bert requires libtorch (1+ GB download, C++ build). Both are heavyweight for what TF-IDF needs.
**Instead:** Use `rust-tfidf` crate for term frequency calculations. Reserve heavy model inference to the LLM backend trait — the LLM API already provides semantic understanding over HTTP.

### Anti-Pattern 2: Single Visualization Process for Both 2D and 3D
**What:** Embedding wgpu into the existing eframe event loop.
**Why bad:** eframe owns the window and event loop; wgpu needs its own surface. Mixing them requires unsafe GPU resource sharing.
**Instead:** Launch visualization_3d as a separate window via wgpu + winit directly, using the `egui-wgpu` crate for UI panels embedded in the 3D viewport. Keep the existing eframe 2D app completely separate.

### Anti-Pattern 3: Running LLM Calls on Full Paper Text
**What:** Sending entire PDF text or HTML dump to the LLM API.
**Why bad:** Expensive (large token counts), slow, and LLMs perform worse on unstructured walls of text than on focused section excerpts.
**Instead:** NLP section detection runs first. LLM receives only the structured section map (`methods_section`, `results_section`, `conclusion_section`), typically 1–3k tokens per paper.

### Anti-Pattern 4: Storing Embeddings Outside SurrealDB
**What:** Writing a separate vector store (Qdrant, Pinecone) alongside SurrealDB.
**Why bad:** Adds operational complexity, breaks the "single DB" constraint, and SurrealDB 3.0 natively supports HNSW vector indexes with combined graph + vector queries in SurrealQL.
**Instead:** Store `tfidf_vector` as a `array<float>` field on `paper_analysis` with an MTREE/HNSW index. Use SurrealQL `<|k|>` nearest-neighbor operator for similarity search.

### Anti-Pattern 5: Rewriting PCA from Scratch
**What:** Implementing eigenvector decomposition manually.
**Why bad:** Numerically fragile, high implementation cost for a well-solved problem.
**Instead:** Use `linfa-reduction` (part of linfa, the Rust scikit-learn equivalent). It provides PCA with optional BLAS backend and is actively maintained.

---

## Scalability Considerations

| Concern | At 50 papers (typical crawl) | At 500 papers | At 5000 papers |
|---------|------------------------------|---------------|----------------|
| Text extraction | Sequential with rate limiting, fine | Fine | May need parallelism with semaphore |
| TF-IDF computation | In-memory across corpus, <100ms | In-memory, <1s | Batch processing, stream from DB |
| LLM API calls | ~$0.05–0.50 depending on provider | Budget becomes significant | Cache strictly; add `--reanalyze` flag |
| PCA projection | Trivial at 50×512 matrix | <100ms | May need incremental PCA |
| SurrealDB vector search | Instant | <10ms | <100ms with HNSW index |
| 3D rendering | 50 nodes instant | 500 nodes fine | 5000 needs LOD or culling |

At typical ReSyn crawl depth (3 BFS levels from one seed), 50–200 papers is the expected corpus. All approaches above are valid at this scale with no modifications.

---

## Sources

- [egui_graphs — petgraph-based graph widget](https://github.com/blitzarx1/egui_graphs) — MEDIUM confidence (WebSearch)
- [egui-wgpu integration crate](https://docs.rs/egui-wgpu) — MEDIUM confidence (WebSearch, official docs.rs)
- [wgpu cross-platform graphics](https://wgpu.rs/) — HIGH confidence (official site)
- [rend3-egui 3D renderer + egui](https://crates.io/crates/rend3-egui) — MEDIUM confidence (WebSearch)
- [rust-tfidf crate](https://crates.io/crates/rust-tfidf) — MEDIUM confidence (crates.io)
- [linfa-reduction PCA](https://crates.io/crates/linfa-reduction) — MEDIUM confidence (crates.io, docs.rs)
- [fast-umap Rust UMAP](https://github.com/eugenehp/fast-umap) — LOW confidence (WebSearch only; depends on burn ML framework)
- [SurrealDB vector embeddings + HNSW](https://surrealdb.com/docs/surrealdb/models/vector) — HIGH confidence (official docs)
- [SurrealDB 3.0 multi-model + vector](https://venturebeat.com/data/surrealdb-3-0-wants-to-replace-your-five-database-rag-stack-with-one/) — MEDIUM confidence (WebSearch)
- [ar5iv HTML arXiv papers](https://ar5iv.labs.arxiv.org/) — HIGH confidence (official arXiv initiative)
- [arXiv LaTeX source e-print download](https://info.arxiv.org/help/view.html) — HIGH confidence (official arXiv docs)
- [arXiv HTML accessibility initiative](https://arxiv.org/html/2402.08954v1) — HIGH confidence (arXiv paper)
- [LLM structured output for scientific papers](https://arxiv.org/abs/2510.04749) — MEDIUM confidence (academic paper)
- [cloudllm pluggable LLM providers Rust](https://lib.rs/crates/cloudllm) — LOW confidence (WebSearch only)
- [Rust AI agent trait-based LLM abstraction](https://dev.to/rajmandaliya/building-a-rust-ai-agent-framework-from-scratch-what-i-learned-3o23) — MEDIUM confidence (WebSearch)

---

*Architecture research: 2026-03-14*
