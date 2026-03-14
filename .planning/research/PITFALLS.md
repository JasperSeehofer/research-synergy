# Domain Pitfalls

**Domain:** Literature-Based Discovery — text analysis pipeline + 3D visualization added to existing Rust citation graph tool
**Researched:** 2026-03-14
**Applies to:** ReSyn milestone adding full-text extraction, hybrid NLP+LLM analysis, pluggable LLM backend, cross-paper gap analysis, 3D visualization

---

## Critical Pitfalls

Mistakes that cause rewrites or require fundamental architectural rework.

---

### Pitfall 1: Assuming LaTeX Source Availability for All Papers

**What goes wrong:** The pipeline is designed around LaTeX `.tar.gz` extraction as the primary path because it preserves the cleanest structure. In production, a significant fraction of arXiv papers submit PDF-only (especially older papers and some subject areas like computer science theses from certain institutions). The pipeline silently produces empty or near-empty extractions for these papers with no per-paper fallback.

**Why it happens:** LaTeX extraction is the best-case path. It is easy to prototype against, since most recent papers do have source. Coverage gaps only appear at scale or when crawling older citation networks.

**Consequences:** Cross-paper gap analysis produces skewed results because papers with successful extraction are overrepresented. The gap output is presented as if complete when it covers maybe 60-80% of the citation network. Downstream phases that assume `full_text` is populated will silently corrupt analysis.

**Warning signs:**
- Any paper citing sources before ~2010 shows zero text extraction
- `full_text` field is null or empty in DB for more than a small percentage of papers
- Gap analysis conclusions differ dramatically based on crawl depth

**Prevention:**
- Implement extraction with a priority cascade: LaTeX source → arXiv HTML (ar5iv) → abstract-only fallback
- Store `extraction_method` enum alongside extracted text (`latex_source | html | abstract_only | none`) so analysis layers know what they have
- Log extraction method distribution per crawl run
- Cross-paper analysis must weight or filter by `extraction_method`; never treat abstract-only extraction as equivalent to full-text extraction

**Phase:** Text extraction phase. Define the cascade and `extraction_method` field before writing any analysis logic.

---

### Pitfall 2: Treating Extracted Text as Clean Prose

**What goes wrong:** LaTeX source contains commands, macros, bibliography entries, figure captions, equation environments, and comment blocks. Even after running through a parser, the output includes residual LaTeX markup (`\textbf{}`, `\cite{}`, `\begin{equation}`), stray tokens from macro expansion failures, and duplicated content from `\input{}` chains. NLP pipelines trained on clean prose produce garbage section detection, incorrect keyword extraction, and inflated TF-IDF scores for LaTeX command tokens like `\phi` or `\mathrm`.

**Why it happens:** LaTeX cleaning is treated as a preprocessing step that "just works." In practice, each paper uses a different combination of packages and custom macros. Generic strippers miss paper-specific definitions.

**Consequences:** Keywords extracted from physics papers are dominated by LaTeX math tokens. Section detection misidentifies equation blocks as section boundaries. TF-IDF vectors are dominated by notation, making semantic similarity meaningless.

**Warning signs:**
- Extracted keywords contain backslash characters or curly braces
- Section count per paper is wildly inconsistent (0 sections or 50+ sections)
- Similarity scores between papers in the same subfield are near-zero

**Prevention:**
- Use arXiv's own HTML rendering (ar5iv / `arxiv.org/html/PAPER_ID`) as the primary clean-text source — LaTeXML already handles macro expansion and equation rendering to MathML, giving clean text sections with semantic markup
- If processing LaTeX source directly, strip math environments entirely rather than trying to parse them; math content is not useful for NLP-level gap analysis
- Run a validation step after extraction: check for backslash density, average token length, and section count plausibility before storing
- Keep raw extracted text and cleaned text as separate fields so the cleaning step can be re-run without re-fetching

**Phase:** Text extraction and NLP extraction phases. Define the cleaning contract before NLP pipeline is built.

---

### Pitfall 3: SurrealDB Schema Drift Without Migration Versioning

**What goes wrong:** The existing codebase auto-initializes schema on connection with no version tracking (`src/database/schema.rs`). The new milestone adds structured analysis fields to the `paper` table (extracted text, NLP results, LLM annotations, analysis timestamps). When a user runs the new version against a database populated by the old version, either: (a) fields are silently absent and queries return null, or (b) schema `DEFINE FIELD` statements conflict with existing data if type constraints tighten.

**Why it happens:** The auto-init pattern works fine when schema only grows by addition. It breaks the moment any field gets a type constraint, a default value, or is removed.

**Consequences:** Users who have invested hours of crawl time in a populated DB must either delete and re-crawl, or run manual SurrealQL patches. The `db-only` mode (load from existing DB without re-crawling) becomes unreliable.

**Warning signs:**
- Any `DEFINE FIELD ... TYPE` statement added to a table that already has records
- A field renamed between versions
- Analysis results appearing for some papers but not others after an upgrade

**Prevention:**
- Adopt the `surrealdb-migrations` crate (Rust-native, integrates with the existing SurrealDB 3.x stack) before adding any new schema fields
- Assign a schema version at connection time; refuse to run on a DB at a lower version without an explicit `--migrate` flag
- New analysis fields should be `FLEXIBLE TYPE` or `OPTION<T>` to allow backward-compatible addition
- Document the migration in code comments, not just commit messages

**Phase:** Database extension phase must begin with migration infrastructure, not field additions.

---

### Pitfall 4: LLM Abstraction That Leaks Provider Semantics

**What goes wrong:** A `LlmBackend` trait is defined with a single `analyze(prompt: &str) -> String` method. Within weeks, the implementation reveals that Claude and OpenAI have different context window limits, different token counting APIs, different rate limiting behaviors, different structured output enforcement mechanisms, and different batch endpoint availability. The abstraction that was meant to hide providers now forces every call site to special-case providers anyway, or the trait grows into an unmanageable set of optional methods.

**Why it happens:** The `PaperSource` trait worked cleanly because fetching a paper is semantically identical across arXiv and InspireHEP. LLM providers have different operational envelopes that matter for the analysis pipeline.

**Consequences:** The trait becomes a leaky abstraction. Adding Ollama support (local models with no batching, no structured outputs, different context limits) requires either duplicating logic or punching holes in the interface. Caching is impossible to implement at the trait level without knowing how each provider counts tokens for cache keys.

**Warning signs:**
- `match provider { Claude => ..., OpenAI => ... }` appearing anywhere outside the provider implementation
- Trait methods acquiring `Option<ModelCapabilities>` or `if self.supports_structured_output()` guards
- Prompt construction logic branching on provider type

**Prevention:**
- Design the trait to include capability reporting: `fn max_context_tokens(&self) -> usize`, `fn supports_structured_output(&self) -> bool`, `fn supports_batch(&self) -> bool`
- Accept a `PromptConfig` struct (not a raw string) so callers can express intent (e.g., "extract JSON matching this schema") and providers decide how to implement it
- Define a `LlmResponse` struct that carries both the raw response and a structured parse result, so the caller doesn't need to know whether the provider natively enforced the schema or whether the client-side parsed it
- Keep prompt construction inside the calling layer, not inside the provider implementation — providers should be transport, not business logic

**Phase:** LLM backend design phase. Get the trait signature right before writing two provider implementations or you will refactor both.

---

## Moderate Pitfalls

Mistakes that cause significant rework but not a full rewrite.

---

### Pitfall 5: Caching LLM Results by Prompt String Alone

**What goes wrong:** Analysis results for already-processed papers are cached by hashing the prompt string. When the prompt template changes (e.g., to extract a new field like "datasets used"), all cached results are invalid but the cache key still matches if the paper text was unchanged. Papers processed before the template change silently lack the new field.

**Why it happens:** Prompt-as-cache-key is the obvious first implementation.

**Consequences:** Partial results in the database — some papers have the new field, others do not, with no way to tell which without checking every record. Cross-paper gap analysis using the new field silently drops papers that were processed before the change.

**Prevention:**
- Cache key must include a prompt template version, not just a hash of the paper content + raw prompt string
- Store `analysis_schema_version` on every paper analysis record in SurrealDB
- Implement a `--reanalyze` flag that re-runs LLM analysis for papers below the current schema version
- Rate-limit-aware batch re-analysis should be possible without a full re-crawl

**Phase:** LLM analysis phase, before any results are persisted.

---

### Pitfall 6: 3D Projection Treated as Ground Truth Layout

**What goes wrong:** Papers are embedded into a high-dimensional space (topic similarity, method overlap, temporal axis, citation density) and projected to 3D via PCA or UMAP for visualization. Users interpret the 3D position of nodes as if proximity means semantic similarity. In reality, PCA projections preserve global variance but distort local neighborhoods; UMAP preserves local structure but distances between clusters are not meaningful. Users draw false conclusions from apparent clustering.

**Why it happens:** 3D graph layouts look authoritative. The algorithm choice and its distortion properties are not surfaced to the user.

**Consequences:** Researchers act on false gap hypotheses that are artifacts of projection distortion rather than real semantic relationships. Trust in the tool erodes when conclusions cannot be replicated.

**Warning signs:**
- UI shows no information about which projection method or parameters were used
- Cluster boundaries change dramatically with different UMAP `n_neighbors` values
- Two papers that cite each other appear far apart in 3D

**Prevention:**
- Display the projection method and key parameters in the UI (e.g., "UMAP n_neighbors=15, min_dist=0.1")
- Offer at least two projection options so users can verify that a cluster persists across algorithms
- 3D positions should be a navigation aid, not the primary output — the primary output is the textual gap analysis; the 3D view is an entry point into it
- Consider t-SNE for local structure exploration and PCA for global overview, and make the distinction explicit in the UI

**Phase:** 3D visualization phase. Embed projection metadata into the UI from day one, not as a later addition.

---

### Pitfall 7: Rate Limiting Not Extended to Text Extraction Paths

**What goes wrong:** The existing codebase enforces rate limiting on arXiv metadata (3s) and InspireHEP (350ms). Full-text extraction adds two new request types: arXiv HTML downloads (ar5iv) and arXiv source archive downloads (`.tar.gz`). These reuse `create_http_client()` but have no rate limiting applied, because the existing `ArxivHTMLDownloader` rate limiter is only wired to bibliography scraping, not to a new extraction client.

**Why it happens:** The rate limiter is embedded in `ArxivHTMLDownloader` rather than being a shared pipeline-level concern. New fetchers are easy to add without realizing they need the same delay.

**Consequences:** Rapid fetching of dozens of full-text HTML pages or source archives triggers arXiv's bot detection. The IP gets throttled or blocked, halting a multi-hour crawl mid-way through. This is particularly damaging because source archive downloads are much larger than metadata API calls.

**Warning signs:**
- HTTP 429 or 403 responses during extraction phase
- Extraction works for 5-10 papers then silently fails for the rest
- `create_http_client()` called in new fetcher code without any `tokio::time::sleep`

**Prevention:**
- Extract rate limiting into a shared `RateLimiter` struct (token bucket or simple sleep-based) that any fetcher can reference, rather than embedding it in `ArxivHTMLDownloader`
- Apply conservative defaults: treat source archive downloads (`.tar.gz`) at 5s intervals since they are large
- The existing `InspireHepClient` rate-limiting bug (only enforced in `fetch_references`, not `fetch_paper`) should be fixed before adding more fetcher types, to avoid the same pattern repeating

**Phase:** Text extraction phase, before any new HTTP fetchers are written.

---

### Pitfall 8: Entity Ambiguity in Cross-Paper Gap Analysis

**What goes wrong:** The gap analysis compares methods and findings across papers. Paper A mentions "transformer," paper B mentions "attention mechanism," paper C mentions "self-attention." The NLP/LLM extraction treats these as three distinct methods. The gap detector concludes that no paper has compared "attention mechanism" with "transformer," even though they are the same concept.

**Why it happens:** LLM extraction produces surface-form strings, not normalized concept identifiers. Without canonicalization, the same concept appears under dozens of synonyms.

**Consequences:** Gap analysis produces high false-positive rates — reporting non-existent gaps between papers that cover the same ground under different terminology. Users lose confidence in the output.

**Warning signs:**
- Synonymous terms (e.g., "GNN" and "graph neural network") appearing as separate extracted methods
- Gap count increases roughly linearly with number of papers processed, rather than stabilizing as the concept space fills in
- The same paper's extracted methods differ between two analysis runs

**Prevention:**
- Prompt engineering: instruct the LLM to normalize extracted concepts to their canonical form ("prefer the full name over the abbreviation; prefer the widely-used name over paper-specific jargon")
- Build a concept normalization pass after extraction: group extracted terms by embedding similarity before comparing across papers
- Store both the raw extracted string and a normalized form in SurrealDB
- For the physics domain specifically (this is a physics/HEP tool), consider leveraging InspireHEP's existing keyword taxonomy as a normalization target

**Phase:** Cross-paper analysis phase design, before the gap detection algorithm is implemented.

---

### Pitfall 9: Fruchterman-Reingold O(n²) Blocker for 3D

**What goes wrong:** The existing 2D visualization uses Fruchterman-Reingold layout, which is already noted in CONCERNS.md as problematic for 100+ nodes. The 3D visualization adds a new dimension that multiplies force calculation cost. A citation graph at depth 3 easily reaches 200-500 nodes. The 3D layout will freeze the GUI on any non-trivial crawl.

**Why it happens:** The fdg crate (used for 2D layout) implements naive O(n²) per iteration. Adding a third dimension does not change the algorithm complexity, but increases the constant factor and worsens the user-visible frame rate.

**Consequences:** The 3D visualization is unusable for the graphs ReSyn is designed to produce. This is discovered late in the milestone when the full pipeline is integrated.

**Warning signs:**
- Frame rate drops below 10 FPS with 150+ nodes in 2D already
- The fdg crate does not offer Barnes-Hut or other spatial subdivision options

**Prevention:**
- Before implementing 3D layout, benchmark the current 2D implementation with 200+ nodes and measure FPS
- For 3D, prefer a layout algorithm with spatial subdivision (Barnes-Hut tree, O(n log n)) — verify whether the chosen 3D library provides this
- Consider pre-computing the 3D layout offline (not interactively in the render loop) and storing positions in SurrealDB, then loading them for visualization
- Alternatively, implement level-of-detail: only simulate physics for nodes visible in the current view frustum

**Phase:** 3D visualization phase planning. Benchmark before committing to a library.

---

## Minor Pitfalls

Issues that cause friction but are contained.

---

### Pitfall 10: LLM Context Window Overflow on Long Papers

**What goes wrong:** Full-text extraction produces 10,000-50,000 tokens for a long physics paper. Sending the entire text to a Claude or OpenAI API call exceeds the effective context window for structured extraction, or incurs high cost. The LLM truncates mid-paper and returns analysis of only the first portion, silently.

**Prevention:**
- Implement chunked extraction: section-by-section rather than full paper at once
- Use NLP pre-processing to identify the most information-dense sections (abstract, introduction, conclusion, methods) and prioritize those for LLM calls
- Log token estimates before API calls; warn if estimated tokens exceed 50% of model context

**Phase:** LLM analysis phase.

---

### Pitfall 11: arXiv HTML Structure Changes Break Section Detection

**What goes wrong:** The existing codebase already relies on `span.ltx_bibblock` CSS selectors for reference extraction and notes that "any arXiv HTML redesign will break reference extraction silently." Section detection for full-text extraction will use similar structural selectors on the arXiv HTML rendering. When arXiv updates LaTeXML or changes their HTML template (which has happened before), section detection silently fails, returning zero sections.

**Prevention:**
- Abstract selector-based parsing into a `HtmlSectionExtractor` with a clear contract and version string
- Add a validation step that checks extraction plausibility (minimum expected sections for a research paper: abstract, body, references)
- Consider the ar5iv standalone dataset as a more stable offline alternative for large batch processing

**Phase:** Text extraction phase.

---

### Pitfall 12: Parallel LLM Calls Without Backpressure

**What goes wrong:** A batch of 50 papers is sent to the LLM backend concurrently using `tokio::join_all` or equivalent. The API responds with rate limit errors (HTTP 429) for calls beyond the provider's per-minute token limit. Error handling retries immediately, amplifying the rate limit pressure. The analysis phase becomes unreliable and hard to resume.

**Prevention:**
- Implement a semaphore-bounded concurrency pattern: at most N concurrent LLM calls, tunable per provider
- Implement exponential backoff on 429 responses, not immediate retry
- Track which papers have been successfully analyzed in SurrealDB so the analysis phase is resumable after interruption

**Phase:** LLM analysis phase.

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|---|---|---|
| Text extraction (LaTeX) | Assumes LaTeX availability for all papers (Pitfall 1) | Implement cascade: LaTeX → HTML → abstract, store `extraction_method` |
| Text extraction (HTML) | LaTeX artifacts in cleaned text (Pitfall 2) | Use ar5iv HTML as primary; validate extracted text before storing |
| Text extraction (HTTP) | Rate limiting gap for new fetchers (Pitfall 7) | Shared `RateLimiter` struct before writing new fetcher code |
| DB schema extension | Schema drift without migrations (Pitfall 3) | Adopt `surrealdb-migrations` crate before adding fields |
| LLM backend trait design | Leaky abstraction (Pitfall 4) | Capability-aware trait with `PromptConfig` and `LlmResponse` |
| LLM analysis caching | Stale cache after prompt updates (Pitfall 5) | Cache key includes prompt schema version |
| LLM analysis calls | Context window overflow (Pitfall 10) | Chunked, section-prioritized extraction |
| LLM analysis calls | Parallel calls without backpressure (Pitfall 12) | Semaphore-bounded concurrency + exponential backoff |
| Gap analysis | Entity ambiguity across papers (Pitfall 8) | LLM canonicalization prompt + normalized concept field in DB |
| 3D visualization layout | O(n²) layout freeze (Pitfall 9) | Benchmark first, consider offline layout precomputation |
| 3D visualization UX | Projection treated as ground truth (Pitfall 6) | Show projection method/params in UI; present text analysis as primary output |
| arXiv HTML parsing | Structure changes (Pitfall 11) | Abstract into versioned extractor with plausibility validation |

---

## Sources

- [Stop Misusing t-SNE and UMAP for Visual Analytics (2025)](https://arxiv.org/html/2506.08725v2) — projection distortion pitfalls
- [Benchmarking Document Parsers on Mathematical Formula Extraction from PDFs](https://arxiv.org/html/2512.09874v1) — PDF extraction limitations
- [ar5iv 04.2024 Dataset — SIGMathLing](https://sigmathling.kwarc.info/resources/ar5iv-dataset-2024/) — ar5iv HTML as structured text source
- [HTML papers on arXiv: why it's important, and how we made it happen](https://arxiv.org/html/2402.08954v1) — arXiv HTML rendering pipeline via LaTeXML
- [Bad Schemas could break your LLM Structured Outputs — Instructor](https://python.useinstructor.com/blog/2024/09/26/bad-schemas-could-break-your-llm-structured-outputs/) — schema design pitfalls for LLM extraction
- [LLM Structured Output Benchmarks are Riddled with Mistakes — Cleanlab](https://cleanlab.ai/blog/structured-output-benchmark/) — validation requirements for structured extraction
- [Caching LLM Responses: When It Helps and When It Hurts — Particula](https://particula.tech/blog/when-to-cache-llm-responses-decision-guide) — caching risks in production
- [Building Bridges to LLMs: Moving Beyond Over Abstraction — HatchWorks](https://hatchworks.com/blog/gen-ai/llm-projects-production-abstraction/) — LLM abstraction layer pitfalls
- [Leveraging Large Language Models for Enhancing Literature-Based Discovery — MDPI](https://www.mdpi.com/2504-2289/8/11/146) — LBD with LLMs, entity ambiguity challenges
- [unarXive 2022: All arXiv Publications Pre-Processed for NLP](https://ar5iv.labs.arxiv.org/html/2303.14957) — structured full-text and citation network extraction patterns
- [surrealdb-migrations crate](https://github.com/Odonno/surrealdb-migrations) — migration tooling for SurrealDB
- [egui_graphs — petgraph-based graph widget](https://github.com/blitzarx1/egui_graphs) — performance notes on Fruchterman-Reingold scaling
- [Comprehensive review of dimensionality reduction algorithms — PMC](https://pmc.ncbi.nlm.nih.gov/articles/PMC12453773/) — limitations of projection techniques
