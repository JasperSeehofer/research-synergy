# Technology Stack

**Project:** Research Synergy (ReSyn) — Text Analysis + 3D Visualization Milestone
**Researched:** 2026-03-14
**Scope:** New libraries only. Existing stack (tokio, petgraph, egui/eframe, surrealdb, reqwest, scraper, serde) is not re-researched.

---

## New Dependencies for This Milestone

### Full-Text Extraction

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| `flate2` | 1.x (already common transitive dep) | Decompress arXiv LaTeX source `.tar.gz` archives | Rust-native, zero external dependency, the standard gzip crate. Used via `GzDecoder`. |
| `tar` | 0.4.x | Unpack `.tar` archives after gzip decompression | Pairs with flate2 for arXiv source archives. Pure Rust. Straightforward API. |
| `pdf-extract` | 0.10.x | Extract plain text from PDF when LaTeX source unavailable | Active development (v0.10.0 released Oct 2025). Pure Rust. Best simple option for PDF fallback path. Not recommended as primary path — arXiv LaTeX/HTML is structurally richer. |
| `scraper` (existing) | 0.23.x | Extract full-text sections from ar5iv HTML | Already in use for HTML parsing. ar5iv HTML5 uses semantic tags (`<section>`, `<h2>`, `<p>`) and LaTeXML CSS classes (`ltx_section`, `ltx_para`, `ltx_bibblock`). Reuse existing infrastructure. |

**Text extraction priority order (implement as fallback chain):**

1. **arXiv LaTeX source** — `https://arxiv.org/src/{id}` returns `.tar.gz`. Best structure. Sections are explicit `\section{}` commands. Extract with `flate2` + `tar`, parse with a small custom LaTeX tokenizer (regex-based is sufficient; full LaTeX parsing is overkill).
2. **ar5iv HTML** — `https://ar5iv.labs.arxiv.org/html/{id}`. Already partially scraped for references. Extend the existing `html_parser.rs` with section-aware selectors. CSS class `ltx_section` marks sections; `ltx_para` marks paragraphs. HIGH confidence this works; existing `ArxivHTMLDownloader` handles rate limits.
3. **PDF via `pdf-extract`** — `https://arxiv.org/pdf/{id}`. Last resort. Loses structure. Accept for papers where LaTeX source and HTML both fail.

**What NOT to use:**
- `lopdf` — Low-level PDF manipulation library. Designed for creating/editing PDFs, not text extraction. Use `pdf-extract` instead (built on lopdf internally but provides the text extraction layer).
- `pdfium-render` / `poppler-rs` — Require system-level native libraries (pdfium, poppler). Violates the "standalone Rust binary" constraint from existing architecture.
- Full LaTeX parsers (e.g. `latex` crate) — Over-engineered for section extraction. Regex matching on `\section`, `\subsection`, and `\begin{abstract}` is sufficient and far simpler to maintain.

---

### NLP Processing (Offline, Mechanical Extraction)

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| `keyword_extraction` | 1.x | TF-IDF, RAKE, TextRank keyword extraction | Implements multiple algorithms behind feature flags. No external deps. Works offline. Directly provides scored keyword lists per paper — exactly what the pipeline needs for per-paper annotation. |
| `fastembed` | 5.x | Local sentence embeddings for semantic similarity | Bundles quantized ONNX sentence-transformer models (all-MiniLM-L6-v2 default). Uses `ort` under the hood. No Python, no API key. Produces 384-dim vectors usable for cosine similarity across papers. v5.12.0 confirmed active. SurrealDB integration docs exist. |

**What NOT to use:**
- `rust-bert` — Requires `tch` (PyTorch C++ bindings via LibTorch, ~2GB download). Too heavy for a desktop tool. Maintenance has slowed vs. fastembed/candle ecosystem.
- `candle` (directly) — HuggingFace's pure-Rust ML framework is excellent but lower-level than fastembed. Use fastembed (which can use candle backend) for the sentence embedding use case. Direct candle usage is warranted only if custom fine-tuning is needed, which it isn't here.
- `nlprule` — Rule-based grammar correction. Wrong tool for this domain.
- `rsnltk` — Shells out to Python. Violates offline constraint and Rust-only constraint.
- `ort` (directly) — fastembed wraps ort correctly. Direct ort usage requires managing ONNX model files and session initialization manually.

---

### LLM Backend (Semantic Extraction)

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| `genai` | 0.5.x | Multi-provider LLM API client | Single crate covers Claude (Anthropic), OpenAI, Ollama, Gemini, Groq, DeepSeek out of the box. v0.5 switched to reqwest 0.13 (matches existing reqwest 0.12 — **pin genai to reqwest 0.12 feature or handle dep version mismatch**). Ergonomic async API. Native protocol support means no translation layer. Actively maintained. |

**Pluggable LLM trait pattern:**

The project already uses the `PaperSource` trait pattern (defined in `src/data_aggregation/traits.rs`). Mirror this exactly for the LLM backend:

```rust
#[async_trait]
pub trait LlmBackend: Send + Sync {
    async fn analyze_paper(&self, prompt: &str, context: &str) -> Result<String, ResynError>;
    fn backend_name(&self) -> &str;
}
```

Implementations:
- `GenAiBackend` — wraps `genai` crate, selects model via config
- `NoopBackend` — returns empty structured JSON, enables offline-only mode

**What NOT to use:**
- `async-openai` alone — Only covers OpenAI. Ollama support requires a separate crate. `genai` subsumes it.
- `clust` / `anthropic-api` — Claude-only. Same problem.
- `llm-connector` — Less mature ecosystem, fewer providers, less active than genai.

**Reqwest version conflict mitigation:**

`genai` 0.5 uses `reqwest 0.13`. The existing codebase uses `reqwest 0.12`. Cargo will attempt to compile both versions side-by-side (allowed since they are different major versions). Verify this compiles cleanly before committing. If it causes link issues, pin `genai` to an older 0.4.x release that used `reqwest 0.12`, or migrate the project to `reqwest 0.13`. LOW confidence on exact resolution — **verify in Phase implementation**.

---

### 3D Visualization

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| `egui-wgpu` (via existing eframe) | 0.31.x (bundled with eframe) | Custom 3D rendering panel inside existing egui app | eframe already uses wgpu as its backend. `egui-wgpu` exposes `CallbackTrait` for injecting custom wgpu render passes into an egui panel. No new window framework needed. Official egui demo (`custom3d_wgpu.rs`) shows the pattern. |
| `wgpu` | 22.x (bundled with eframe) | GPU draw calls for 3D node/edge rendering | Already a transitive dependency through eframe. Custom wgpu render pass handles: 3D node spheres, edge lines, camera projection. Avoids adding a heavy game engine. |
| `glam` | 0.29.x | 3D math: Vec3, Mat4, camera transforms | Lightweight, no_std-compatible, the de-facto standard for game/graphics math in Rust. No alternative worth considering at this scope. |

**3D visualization architecture:**

Do NOT use a game engine (Bevy) or a separate 3D framework (three-d, rend3). The existing app is an egui app. The 3D view should be one panel in the existing egui layout, rendered via `egui::PaintCallback` into a wgpu render pass. This is exactly what `egui_wgpu::CallbackTrait` enables.

Implementation approach:
1. Add a new `Visualization3dPanel` struct implementing `egui_wgpu::CallbackTrait`
2. Maintain a `Vec<Node3d>` (position in 3D, color, size from analysis dimensions) and `Vec<Edge3d>` (pairs of node indices)
3. On each frame, upload node/edge data as vertex buffers via wgpu
4. Use a simple vertex shader for nodes (billboarded quads or point sprites) and edges (line list)
5. Camera: orbit camera with mouse drag, scroll zoom — implement manually using `glam` transforms

**Why NOT Bevy:** Bevy is an ECS game engine. Embedding it alongside an existing egui app requires the `bevy_egui` integration crate and fundamentally restructures the app around Bevy's scheduler. Disproportionate complexity for adding one visualization panel.

**Why NOT three-d:** Uses OpenGL 3.3, not wgpu. Would introduce a second graphics backend alongside eframe's wgpu. Conflicts likely. Abandoned in favor of native wgpu inside eframe.

**Why NOT rend3:** Last meaningful commit was 2023. MEDIUM confidence it still compiles with current wgpu. Maintenance risk unacceptable.

---

### Supporting Utilities

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `tiktoken-rs` | 0.6.x | Token counting before LLM calls | Use to estimate token count of paper text before sending to LLM API. Prevents exceeding context windows and enables cost estimation. Claude/OpenAI use different tokenizers — use as an approximation. |
| `base64` | 0.22.x | Encode binary content for LLM multimodal API calls | If future phases send PDF pages as images to vision-capable LLMs. Optional — only add if needed. |
| `ndarray` | 0.16.x | Dense matrix operations for embedding similarity | Computing cosine similarity matrix across N paper embedding vectors. Standard for numerical arrays in Rust. |

---

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| LLM client | `genai` 0.5 | `async-openai` | Single-provider; no Ollama/Claude without extra crates |
| LLM client | `genai` 0.5 | `clust` | Claude-only, lower activity |
| Embeddings | `fastembed` | `rust-bert` | ~2GB LibTorch download, slowing maintenance |
| Embeddings | `fastembed` | `candle` direct | Lower-level; fastembed provides ready-made models |
| PDF extraction | `pdf-extract` | `lopdf` | lopdf is manipulation, not extraction |
| PDF extraction | `pdf-extract` | `pdfium-render` | Requires native pdfium binary, breaks standalone binary |
| LaTeX extraction | Custom regex | `latex` crate | Section extraction doesn't need a full LaTeX parser |
| 3D visualization | egui wgpu callback | Bevy | ECS engine; requires restructuring entire app |
| 3D visualization | egui wgpu callback | `three-d` | OpenGL backend conflicts with eframe's wgpu |
| 3D visualization | egui wgpu callback | `rend3` | Effectively unmaintained since 2023 |
| NLP keywords | `keyword_extraction` | `tfidf-text-summarizer` | Summarizer, not keyword extraction; wrong primitive |
| NLP keywords | `keyword_extraction` | `rs-natural` | Less active, fewer algorithms |

---

## Installation

```toml
# Cargo.toml additions

[dependencies]
# Full-text extraction
flate2 = "1"
tar = "0.4"
pdf-extract = "0.10"

# NLP
keyword_extraction = { version = "1", features = ["tf_idf", "rake", "text_rank"] }
fastembed = "5"

# LLM API
genai = "0.5"

# 3D math
glam = "0.29"

# Token counting (LLM cost/window management)
tiktoken-rs = "0.6"

# Embedding similarity
ndarray = "0.16"

[dev-dependencies]
# No new dev deps — existing wiremock/tokio-test cover new modules
```

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| LaTeX source extraction (flate2 + tar) | HIGH | Standard Rust crates, well-documented, arXiv source endpoint is stable |
| ar5iv HTML extraction extension | HIGH | Existing scraper infra already targets ar5iv; CSS class structure confirmed from ar5iv docs |
| PDF extraction (pdf-extract) | MEDIUM | Active development confirmed (v0.10.0, Oct 2025); quality on academic PDFs with complex math layouts is variable |
| NLP keyword extraction (keyword_extraction) | MEDIUM | Confirmed on crates.io with multiple algorithms; community adoption is modest; API may need validation |
| Local embeddings (fastembed) | HIGH | v5.12.0 confirmed, SurrealDB integration docs exist, ONNX-based, no Python required, strong community |
| LLM multi-provider client (genai) | HIGH | v0.5 confirmed, 10+ providers, actively maintained, reqwest version conflict needs implementation verification |
| reqwest version conflict (genai 0.5 vs existing 0.12) | LOW | Cargo may compile both; could also require project migration to reqwest 0.13 — must verify during implementation |
| 3D visualization via egui wgpu callback | HIGH | Official egui demo (`custom3d_wgpu.rs`) demonstrates the pattern; wgpu already bundled with eframe |
| 3D math (glam) | HIGH | De-facto standard, stable API, widely used with wgpu |

---

## Sources

- [genai crate — crates.io](https://crates.io/crates/genai)
- [rust-genai GitHub — jeremychone/rust-genai](https://github.com/jeremychone/rust-genai)
- [fastembed crate — crates.io](https://crates.io/crates/fastembed)
- [fastembed-rs GitHub — Anush008/fastembed-rs](https://github.com/Anush008/fastembed-rs)
- [pdf-extract crate — crates.io](https://crates.io/crates/pdf-extract)
- [lopdf GitHub — J-F-Liu/lopdf](https://github.com/J-F-Liu/lopdf)
- [keyword_extraction — lib.rs](https://lib.rs/crates/keyword_extraction)
- [egui custom3d_wgpu demo — GitHub emilk/egui](https://github.com/emilk/egui/blob/main/crates/egui_demo_app/src/apps/custom3d_wgpu.rs)
- [egui_wgpu CallbackTrait docs](https://docs.rs/egui-wgpu)
- [ar5iv HTML5 service](https://ar5iv.labs.arxiv.org/)
- [ar5iv GitHub — dginev/ar5iv](https://github.com/dginev/ar5iv)
- [flate2 crate — GitHub rust-lang/flate2-rs](https://github.com/rust-lang/flate2-rs)
- [ort crate (ONNX Runtime) — GitHub pykeio/ort](https://github.com/pykeio/ort)
- [Building Sentence Transformers in Rust — DEV Community](https://dev.to/mayu2008/building-sentence-transformers-in-rust-a-practical-guide-with-burn-onnx-runtime-and-candle-281k)
- [async-openai crate — crates.io](https://crates.io/crates/async-openai)
- [three-d crate — crates.io](https://crates.io/crates/three-d)
- [arXiv HTML accessibility announcement — arXiv blog](https://blog.arxiv.org/2023/12/21/accessibility-update-arxiv-now-offers-papers-in-html-format/)

---

*Stack research: 2026-03-14*
