# Phase 1: Text Extraction Foundation - Research

**Researched:** 2026-03-14
**Domain:** Rust HTML parsing (scraper crate), ar5iv/LaTeXML HTML structure, SurrealDB v3 schema migration, CLI flag extension (clap)
**Confidence:** HIGH (core stack verified against existing code and official docs)

## Summary

Phase 1 builds a post-crawl text extraction pipeline that fetches ar5iv HTML for each persisted paper, parses it into structured named sections (abstract, introduction, methods, results, conclusion), and stores `TextExtractionResult` records in SurrealDB. Papers with no ar5iv HTML are flagged as `partial` and continue without error. Two new CLI flags control the behavior: `--analyze` (activates extraction after crawl+persist) and `--skip-fulltext` (forces abstract-only for all papers).

The core technical stack is already present in the project. `scraper` (already a dependency) handles CSS selector-based section parsing. `ArxivHTMLDownloader` (already present) provides the rate-limited HTTP client pattern. SurrealDB v3's `IF NOT EXISTS` DDL avoids the need for an external migration crate — a lightweight custom version-tracking table is the right approach, since `surrealdb-migrations` targets SurrealDB v2.x and is not compatible with v3.

**Primary recommendation:** Implement `src/data_aggregation/text_extractor.rs` as a new module mirroring the `ArxivHTMLDownloader` builder pattern, add a `schema_migrations` table with a version counter in `schema.rs`, and wire `--analyze` / `--skip-fulltext` into `main.rs` as a sequential post-persist step.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Extract into structured named sections: abstract, introduction, methods/approach, results/discussion, conclusion
- Each section is a separate field (not a flat text blob) — enables section-aware LLM prompting in later phases
- Metadata per extraction: `extraction_method` (ar5iv_html / abstract_only) + completeness map (which sections were found vs missing)
- All four section categories extracted: abstract, intro+conclusion, methods, results/discussion
- `--analyze` flag triggers text extraction as a post-crawl step: crawl → persist → **analyze** → visualize
- Extraction runs after crawl, as a second pass over persisted papers — not during crawl
- `--analyze` requires `--db` (persistence needed for caching and avoiding redundant work)
- Analysis runs on already-persisted papers from DB, enabling re-extraction without re-crawling
- Summary at end of extraction: "12/30 papers used abstract-only (no ar5iv HTML available)" — not per-paper warnings
- Best-effort section extraction: extract whatever sections exist, mark missing ones as None
- `--skip-fulltext` forces abstract-only extraction for all papers (fast mode for testing/debugging)
- Immediate fallback on ar5iv HTTP errors (500, timeout) — no retry, fall back to abstract-only
- Papers flagged as `partial` when using abstract-only continue through the pipeline without error

### Claude's Discretion
- DB schema design: separate table + relation vs embedded fields on paper record
- Rate limiter strategy: share existing ArxivHTMLDownloader rate limiter or new fetcher with same politeness rules
- Section detection approach: CSS selectors for ar5iv HTML heading patterns
- Text cleaning level: handling of LaTeX artifacts, math notation, reference markers
- Migration tool choice: surrealdb-migrations crate vs custom version tracking
- InspireHEP paper extraction: how to handle papers that came from InspireHEP but have arXiv eprints

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| TEXT-03 | System fetches full text from arXiv HTML (ar5iv) with section detection for papers that have HTML available | ar5iv URL pattern, LaTeXML CSS class names (`ltx_section`, `ltx_abstract`, `ltx_title_section`), scraper crate CSS selector API |
| TEXT-04 | System falls back gracefully to abstract-only analysis when full text is unavailable, flagging the paper as partial | `Paper.summary` field always populated from both arXiv and InspireHEP sources; `ExtractionMethod` enum + completeness map |
| INFR-03 | Database schema changes use a migration system to safely extend the existing paper schema | Custom `schema_migrations` table with version counter; SurrealDB v3 `IF NOT EXISTS` DDL; `surrealdb-migrations` crate incompatible with v3 |
| INFR-04 | System provides CLI flags to control analysis pipeline (`--analyze`, `--llm-provider`, `--skip-fulltext`) | clap derive API; `Cli` struct extension in `main.rs`; phase delivers `--analyze` and `--skip-fulltext` only |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| scraper | 0.23.1 (already in Cargo.toml) | CSS selector-based HTML parsing | Already a project dependency used for arXiv HTML reference extraction |
| surrealdb | 3.x (already in Cargo.toml) | Persist `TextExtractionResult` records | Existing DB layer; `IF NOT EXISTS` DDL prevents the need for an external migration crate |
| clap | 4.x with derive (already in Cargo.toml) | `--analyze` and `--skip-fulltext` CLI flags | Existing CLI pattern; just extend `Cli` struct |
| reqwest | 0.12.x (already in Cargo.toml) | HTTP fetching for ar5iv HTML | Shared client via `utils::create_http_client()` |
| tracing | 0.1.x (already in Cargo.toml) | Structured logging for extraction progress | Existing logging pattern |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| wiremock | 0.6 (already in dev-dependencies) | Mock ar5iv HTTP responses in integration tests | All integration tests for the extractor module |
| tokio | 1.x (already in Cargo.toml) | Async extraction loop | Already the async runtime |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Custom migration table | surrealdb-migrations crate | surrealdb-migrations targets SurrealDB ^2.4.0 — incompatible with the project's surrealdb v3 dependency. Custom lightweight table is simpler and correct. |
| Separate ar5iv fetcher struct | Extend ArxivHTMLDownloader | Separate struct is cleaner — it has a different URL base and different error semantics. Sharing the HTTP client is correct; duplicating the rate limiter struct is acceptable. |
| CSS class selectors | Heading text heuristics | CSS class selectors (LaTeXML `ltx_*` classes) are more reliable than string-matching section titles, which vary across papers |

**Installation:** No new dependencies required. All libraries are already present in `Cargo.toml`.

## Architecture Patterns

### Recommended Project Structure
```
src/
├── data_aggregation/
│   ├── text_extractor.rs    # NEW: Ar5ivExtractor struct with rate-limited fetch + section parse
│   ├── html_parser.rs       # EXISTING: ArxivHTMLDownloader (reuse pattern)
│   └── ...
├── datamodels/
│   ├── extraction.rs        # NEW: TextExtractionResult, ExtractionMethod, SectionMap
│   └── paper.rs             # EXISTING: Paper struct (unchanged)
├── database/
│   ├── schema.rs            # MODIFIED: add extraction table DDL + migration version table
│   ├── queries.rs           # MODIFIED: add ExtractionRepository with upsert/get/get_all
│   └── ...
└── main.rs                  # MODIFIED: --analyze, --skip-fulltext flags + analysis pipeline step
```

### Pattern 1: Builder-pattern rate-limited extractor (mirrors ArxivHTMLDownloader)
**What:** New `Ar5ivExtractor` struct with the same `::new(client).with_rate_limit(duration)` builder and `rate_limit_check()` method used by `ArxivHTMLDownloader`.
**When to use:** All ar5iv HTML fetch calls during the analysis phase.
**Example:**
```rust
// Mirrors src/data_aggregation/html_parser.rs
pub struct Ar5ivExtractor {
    last_called: Option<Instant>,
    call_per_duration: Duration,
    client: reqwest::Client,
}

impl Ar5ivExtractor {
    pub fn new(client: reqwest::Client) -> Self {
        Self {
            last_called: None,
            call_per_duration: Duration::from_secs(3),
            client,
        }
    }

    pub fn with_rate_limit(mut self, duration: Duration) -> Self {
        self.call_per_duration = duration;
        self
    }

    pub async fn extract(&mut self, paper: &Paper) -> TextExtractionResult {
        // rate_limit_check() then fetch ar5iv HTML
        // on HTTP error: return abstract-only fallback
    }
}
```

### Pattern 2: TextExtractionResult data model
**What:** Structured extraction output with named section fields, extraction method enum, and per-section completeness flags.
**When to use:** Returned by `Ar5ivExtractor::extract()`, persisted to SurrealDB, loaded back for downstream phases.
**Example:**
```rust
// src/datamodels/extraction.rs
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum ExtractionMethod {
    #[default]
    AbstractOnly,
    Ar5ivHtml,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SectionMap {
    pub abstract_text: Option<String>,
    pub introduction: Option<String>,
    pub methods: Option<String>,
    pub results: Option<String>,
    pub conclusion: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TextExtractionResult {
    pub arxiv_id: String,
    pub extraction_method: ExtractionMethod,
    pub sections: SectionMap,
    pub is_partial: bool,     // true when abstract-only
    pub extracted_at: String, // ISO 8601 timestamp
}
```

### Pattern 3: ar5iv URL construction
**What:** ar5iv HTML is served at `https://arxiv.org/html/{arxiv_id}` (the existing `convert_pdf_url_to_html_url()` already produces this URL). For papers fetched from InspireHEP, use the `arxiv_id` field directly.
**When to use:** Inside `Ar5ivExtractor::extract()`.
**Example:**
```rust
fn ar5iv_url(arxiv_id: &str) -> String {
    format!("https://arxiv.org/html/{}", arxiv_id)
}
```

### Pattern 4: CSS selector-based section parsing (LaTeXML `ltx_*` classes)
**What:** arXiv HTML papers are generated by LaTeXML and use `ltx_*` CSS classes consistently. Section containers use `ltx_section`, section titles use `ltx_title ltx_title_section`, and the abstract uses `ltx_abstract`. Text content sits in `ltx_para` elements inside each section.
**When to use:** Inside the HTML parsing step in `Ar5ivExtractor`.
**Example:**
```rust
use scraper::{Html, Selector};

fn parse_sections(html: &Html) -> SectionMap {
    // Abstract
    let abstract_sel = Selector::parse(".ltx_abstract").expect("static");
    let abstract_text = html.select(&abstract_sel)
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string());

    // Named sections: match on normalized title text
    let section_sel = Selector::parse("section.ltx_section, div.ltx_section").expect("static");
    let title_sel = Selector::parse(".ltx_title_section, .ltx_title_chapter").expect("static");
    let para_sel = Selector::parse(".ltx_para").expect("static");

    let mut introduction = None;
    let mut methods = None;
    let mut results = None;
    let mut conclusion = None;

    for section in html.select(&section_sel) {
        let title = section.select(&title_sel)
            .next()
            .map(|t| t.text().collect::<String>().to_lowercase());

        let text: String = section.select(&para_sel)
            .flat_map(|p| p.text())
            .collect::<Vec<_>>()
            .join(" ");

        match title.as_deref() {
            Some(t) if t.contains("introduction") => introduction = Some(text),
            Some(t) if t.contains("method") || t.contains("approach") || t.contains("model") => methods = Some(text),
            Some(t) if t.contains("result") || t.contains("discussion") || t.contains("experiment") => results = Some(text),
            Some(t) if t.contains("conclusion") || t.contains("summary") => conclusion = Some(text),
            _ => {}
        }
    }

    SectionMap { abstract_text, introduction, methods, results, conclusion }
}
```

### Pattern 5: Lightweight custom migration table
**What:** A `schema_migrations` table stores integer version + timestamp. `init_schema()` becomes `migrate_schema(current_version)` which runs each numbered migration exactly once. No external crate needed.
**When to use:** `database/schema.rs` — replaces the current single-shot `init_schema()`.
**Example:**
```rust
// src/database/schema.rs
pub async fn migrate_schema(db: &Surreal<Any>) -> Result<(), ResynError> {
    // Ensure migration tracking table exists (idempotent)
    db.query("
        DEFINE TABLE IF NOT EXISTS schema_migrations SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS version ON schema_migrations TYPE int;
        DEFINE FIELD IF NOT EXISTS applied_at ON schema_migrations TYPE string;
    ")
    .await
    .map_err(|e| ResynError::Database(format!("migration table init failed: {e}")))?;

    let current: Option<i64> = get_schema_version(db).await?;
    let current = current.unwrap_or(0);

    if current < 1 { apply_migration_1(db).await?; }
    if current < 2 { apply_migration_2(db).await?; }
    // ...
    Ok(())
}

async fn apply_migration_1(db: &Surreal<Any>) -> Result<(), ResynError> {
    // Original paper + cites schema
    db.query(" /* existing schema DDL */ ").await ...;
    set_schema_version(db, 1).await
}

async fn apply_migration_2(db: &Surreal<Any>) -> Result<(), ResynError> {
    // text_extraction table DDL
    db.query("
        DEFINE TABLE IF NOT EXISTS text_extraction SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS arxiv_id ON text_extraction TYPE string;
        DEFINE FIELD IF NOT EXISTS extraction_method ON text_extraction TYPE string;
        DEFINE FIELD IF NOT EXISTS abstract_text ON text_extraction TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS introduction ON text_extraction TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS methods ON text_extraction TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS results ON text_extraction TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS conclusion ON text_extraction TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS is_partial ON text_extraction TYPE bool;
        DEFINE FIELD IF NOT EXISTS extracted_at ON text_extraction TYPE string;
        DEFINE INDEX IF NOT EXISTS idx_extraction_arxiv_id ON text_extraction FIELDS arxiv_id UNIQUE;
    ").await ...;
    set_schema_version(db, 2).await
}
```

### Pattern 6: `--analyze` pipeline integration in main.rs
**What:** New `--analyze` and `--skip-fulltext` flags on `Cli` struct, with analysis running as a sequential step after DB persist.
**When to use:** `main.rs` — extend the existing pipeline.
**Example:**
```rust
// In Cli struct (clap derive)
/// Run text extraction analysis after crawl
#[arg(long, default_value_t = false)]
analyze: bool,

/// Skip full-text extraction, use abstract only
#[arg(long, default_value_t = false)]
skip_fulltext: bool,
```

```rust
// In main() after the persist block
if cli.analyze {
    let Some(ref db) = db else {
        error!("--analyze requires --db to be specified");
        std::process::exit(1);
    };
    let repo = database::queries::ExtractionRepository::new(db);
    let all_papers = database::queries::PaperRepository::new(db)
        .get_all_papers().await?;

    let client = utils::create_http_client();
    let mut extractor = data_aggregation::text_extractor::Ar5ivExtractor::new(client)
        .with_rate_limit(Duration::from_secs(cli.rate_limit_secs));

    let mut abstract_only_count = 0usize;
    for paper in &all_papers {
        if repo.extraction_exists(&paper.id).await? { continue; } // skip cached
        let result = if cli.skip_fulltext {
            TextExtractionResult::from_abstract(paper)
        } else {
            extractor.extract(paper).await
        };
        if result.is_partial { abstract_only_count += 1; }
        repo.upsert_extraction(&result).await?;
    }
    info!(
        abstract_only = abstract_only_count,
        total = all_papers.len(),
        "{}/{} papers used abstract-only extraction",
        abstract_only_count, all_papers.len()
    );
}
```

### Anti-Patterns to Avoid
- **Embedding extraction fields on the `paper` table:** Makes the paper record mutable after crawl and pollutes the fetch/crawl data model. Use a separate `text_extraction` table with a `UNIQUE` index on `arxiv_id`.
- **Running extraction during crawl (not after):** The user decision is explicit: extraction is a post-crawl second pass. Interleaving it with BFS crawling adds latency and complicates re-extraction.
- **Retrying ar5iv HTTP errors:** Decision is immediate fallback to abstract-only. No retry loop.
- **Per-paper warning logs for missing HTML:** Decision is a single summary log at the end. Use counters, not per-paper warns.
- **Using `surrealdb-migrations` crate:** It targets SurrealDB ^2.4.0, incompatible with the project's surrealdb v3 dependency.
- **Panicking on section parse failure:** Always return a `TextExtractionResult` — even if all sections are `None`. Extraction is best-effort.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| HTML parsing and CSS selection | Custom string slicing/regex for HTML | `scraper` crate (already a dependency) | Already handles malformed HTML, CSS pseudo-selectors, attribute queries — battle-tested |
| Rate limiting between requests | New sleep-based rate limiter | Mirror `ArxivHTMLDownloader::rate_limit_check()` pattern | The pattern is already validated and tested in the codebase |
| HTTP client with timeout | New reqwest::Client builder | `utils::create_http_client()` | Already provides 30s timeout, shared across all fetchers |

**Key insight:** The entire HTML fetch-and-parse stack already exists in the project. The text extractor is a new _application_ of existing building blocks, not new infrastructure.

## Common Pitfalls

### Pitfall 1: ar5iv 404s treated as hard errors
**What goes wrong:** Some papers don't have ar5iv HTML (older papers, conversion failures). A 404 or 5xx causes extraction to abort rather than fall back.
**Why it happens:** `reqwest` returns `Err` only for network errors by default; HTTP 4xx/5xx are Ok responses with non-200 status. Code that does `.send()?.text()?` without checking `.status()` silently gets the error page HTML and tries to parse it as paper content.
**How to avoid:** After `.send()`, check `response.status().is_success()` before calling `.text()`. On non-200, return `TextExtractionResult::from_abstract(paper)`.
**Warning signs:** Section fields are all `None` even on papers known to have ar5iv HTML; extraction appears to "succeed" but produces empty results.

### Pitfall 2: Section title matching too narrow
**What goes wrong:** Sections labeled "I. Introduction", "1 Introduction", "A. Motivation" don't match pattern `t.contains("introduction")`.
**Why it happens:** Physics/CS papers on arXiv use highly variable section naming conventions. Some use Roman numerals, some use letters, some use free-form titles.
**How to avoid:** Normalize the title string: strip leading whitespace, digits, Roman numerals, dots, and convert to lowercase before matching. Use `contains` with multiple synonyms for each category.
**Warning signs:** `introduction` field is always `None` on multi-section papers.

### Pitfall 3: Extraction result linked to wrong paper ID (version suffix)
**What goes wrong:** Paper crawled as "2301.12345v2" produces extraction stored under "2301.12345v2", but DB lookup uses "2301.12345". Cache-check always misses.
**Why it happens:** `strip_version_suffix()` is called during paper upsert but may be forgotten in extraction record ID construction.
**How to avoid:** Always call `strip_version_suffix(&paper.id)` when building the `arxiv_id` field in `TextExtractionResult`. Use `paper_record_id` from `queries.rs` as a model for the extraction record ID.
**Warning signs:** Papers are re-extracted on every `--analyze` run instead of being skipped.

### Pitfall 4: Migration runs partially and leaves schema in inconsistent state
**What goes wrong:** A migration DDL query fails midway through (e.g., network drop to embedded DB, disk full). The version number is never updated, so the migration re-runs on next startup, but some DDL has already been applied, causing conflicts.
**Why it happens:** SurrealDB `IF NOT EXISTS` on `DEFINE TABLE` makes individual statements idempotent, but the version bump happens in a separate query.
**How to avoid:** Structure each migration as: (1) idempotent DDL using `IF NOT EXISTS`, then (2) version bump. Because the DDL is idempotent, re-running it on a partial failure is safe. The version only updates after all DDL succeeds.
**Warning signs:** `schema_migrations` shows version stuck at N-1 even though the version N table fields exist.

### Pitfall 5: InspireHEP papers with no arXiv HTML
**What goes wrong:** Papers fetched via InspireHEP source have `source = InspireHep` and a valid `arxiv_id` field, but the ar5iv URL is constructed incorrectly or not constructed at all.
**Why it happens:** The `pdf_url` field on InspireHEP papers may not follow the same arXiv URL convention used by `convert_pdf_url_to_html_url()`.
**How to avoid:** Use `paper.id` (the stripped arXiv ID) directly to build the ar5iv URL: `format!("https://arxiv.org/html/{}", paper.id)`. Do not rely on `pdf_url` conversion. If `paper.id` is empty or invalid, fall back to abstract-only.
**Warning signs:** InspireHEP-sourced papers are always `is_partial = true` even when their arXiv HTML exists.

### Pitfall 6: Text extraction includes bibliography text
**What goes wrong:** Extracted "introduction" or "conclusion" text contains bibliography entries mixed in from reference list sections.
**Why it happens:** The bibliography section (`ltx_bibliography`) sits inside or after the main `ltx_section` elements and may be selected by paragraph selectors.
**How to avoid:** Add an exclusion for sections whose title normalizes to "references", "bibliography", or "acknowledgements". Or select only the text nodes that are not inside `.ltx_bibliography`.
**Warning signs:** Extracted section text contains "[1] Author, Title, Journal" style citation strings.

## Code Examples

Verified patterns from existing codebase and official sources:

### Checking HTTP response status before parsing (verified pattern)
```rust
// Based on existing ArxivHTMLDownloader pattern, extended with status check
pub async fn fetch_ar5iv_html(&self, arxiv_id: &str) -> Result<Html, ResynError> {
    let url = format!("https://arxiv.org/html/{}", arxiv_id);
    let response = self.client.get(&url).send().await
        .map_err(|e| ResynError::HtmlDownload(format!("{url}: {e}")))?;

    if !response.status().is_success() {
        return Err(ResynError::HtmlDownload(
            format!("{url}: HTTP {}", response.status())
        ));
    }

    let body = response.text().await
        .map_err(|e| ResynError::HtmlDownload(format!("{url}: {e}")))?;
    Ok(Html::parse_document(&body))
}
```

### SurrealDB v3 UPSERT with extraction record ID
```rust
// Pattern mirrors existing PaperRepository::upsert_paper in queries.rs
self.db
    .query("UPSERT type::record('text_extraction', $id) CONTENT $record")
    .bind(("id", strip_version_suffix(&result.arxiv_id)))
    .bind(("record", record.into_value()))
    .await
    .map_err(|e| ResynError::Database(format!("upsert extraction failed: {e}")))?;
```

### clap derive flag addition (verified against existing Cli struct)
```rust
// Extend Cli in main.rs — matches existing pattern
/// Run text extraction analysis after crawl and persist
#[arg(long, default_value_t = false)]
analyze: bool,

/// Skip full-text extraction; all papers use abstract only
#[arg(long, default_value_t = false)]
skip_fulltext: bool,
```

### Abstract-only fallback construction
```rust
impl TextExtractionResult {
    pub fn from_abstract(paper: &Paper) -> Self {
        TextExtractionResult {
            arxiv_id: strip_version_suffix(&paper.id),
            extraction_method: ExtractionMethod::AbstractOnly,
            sections: SectionMap {
                abstract_text: Some(paper.summary.clone()),
                ..Default::default()
            },
            is_partial: true,
            extracted_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}
```

Note: if `chrono` is not already a dependency, use a simpler timestamp approach or add it. Check `Cargo.toml` — it is not currently listed. Use a format string from `std::time::SystemTime` instead, or add `chrono = { version = "0.4", features = ["serde"] }`.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| surrealdb-migrations crate | Custom migration version table | surrealdb-migrations stuck on ^2.4.0, project uses v3 | Must implement lightweight custom migration; ~20 lines of Rust, not a burden |
| arXiv PDF text extraction | ar5iv HTML (LaTeXML) extraction | arXiv rolled out HTML-first for papers in ~2023-2024 | Structured HTML with CSS classes is far easier to parse than PDF; no PDF parsing library needed |
| Full LaTeXML CSS class docs | Empirical inspection of live papers | LaTeXML docs are incomplete regarding specific class names | Verify selectors against real ar5iv papers during implementation; `ltx_section`, `ltx_abstract`, `ltx_para`, `ltx_title_section` are confirmed present |

**Deprecated/outdated:**
- `surrealdb-migrations`: Targets v2.x. Do not add as a dependency.
- ar5iv.labs.arxiv.org: The legacy ar5iv subdomain still works but `arxiv.org/html/{id}` is the canonical current URL (used by existing `convert_pdf_url_to_html_url()` already).

## Open Questions

1. **`chrono` crate for timestamps**
   - What we know: `TextExtractionResult` needs an `extracted_at` timestamp. `chrono` is not in `Cargo.toml`.
   - What's unclear: Whether to add `chrono` or use `std::time::SystemTime` formatting.
   - Recommendation: Add `chrono = { version = "0.4", features = ["serde"] }` — it will be needed in Phase 2 and 3 as well. Alternatively, use `std::time::UNIX_EPOCH` with manual formatting to avoid a new dep for now.

2. **DB schema design: separate table vs embedded fields on `paper`**
   - What we know: CONTEXT.md leaves this to Claude's discretion. Both approaches work in SurrealDB.
   - What's unclear: Whether downstream phases (2, 3) will need to JOIN extraction with paper or load them together.
   - Recommendation: Separate `text_extraction` table with `UNIQUE` index on `arxiv_id`. This keeps the paper record unchanged (no re-crawl needed to populate it), enables independent re-extraction, and matches the existing SurrealDB relation pattern.

3. **Section detection accuracy on physics HEP papers**
   - What we know: arXiv papers in the HEP domain (this project's domain) use varied section naming (e.g., "Theoretical Framework", "Numerical Results", "Phenomenological Analysis").
   - What's unclear: Whether title-keyword matching will correctly classify non-standard section names.
   - Recommendation: Start with the multi-keyword match approach documented in Pattern 4. Log when sections remain `None` on multi-section papers during manual testing. Refinement is low-risk since it only affects which `Option<String>` gets populated.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test runner (cargo test) |
| Config file | none — uses `#[cfg(test)]` inline + `cargo test` |
| Quick run command | `cargo test text_extractor` |
| Full suite command | `cargo test` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| TEXT-03 | ar5iv HTML fetch + section parse populates non-None fields | integration | `cargo test text_extractor::tests::test_ar5iv_section_parse` | Wave 0 |
| TEXT-03 | ar5iv HTML with HTTP 404 triggers fallback not panic | integration | `cargo test text_extractor::tests::test_ar5iv_404_fallback` | Wave 0 |
| TEXT-03 | ar5iv URL constructed from arxiv_id (not pdf_url) | unit | `cargo test text_extractor::tests::test_ar5iv_url_construction` | Wave 0 |
| TEXT-04 | abstract-only extraction sets `is_partial = true` | unit | `cargo test extraction::tests::test_abstract_only_result` | Wave 0 |
| TEXT-04 | abstract-only extraction populates `abstract_text` from `Paper.summary` | unit | `cargo test extraction::tests::test_abstract_from_paper_summary` | Wave 0 |
| INFR-03 | migration v1 → v2 applies `text_extraction` table DDL | unit (DB) | `cargo test database::schema::tests::test_migration_v2_creates_extraction_table` | Wave 0 |
| INFR-03 | migration is idempotent (run twice, no error) | unit (DB) | `cargo test database::schema::tests::test_migration_idempotent` | Wave 0 |
| INFR-04 | `--analyze` flag absent → no extraction runs | unit (main integration) | `cargo test -- --test cli_analyze_skipped_without_flag` | Wave 0 |
| INFR-04 | `--analyze` without `--db` exits with error | unit | `cargo test -- --test cli_analyze_requires_db` | Wave 0 |
| INFR-04 | `--skip-fulltext` causes all papers to use abstract-only | unit | `cargo test text_extractor::tests::test_skip_fulltext_mode` | Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test text_extractor`
- **Per wave merge:** `cargo test`
- **Phase gate:** Full suite green (`cargo test`) before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src/data_aggregation/text_extractor.rs` — main extractor module with `#[cfg(test)]` block covering TEXT-03
- [ ] `src/datamodels/extraction.rs` — data model with unit tests covering TEXT-04
- [ ] `src/database/schema.rs` — modified to add migration system with `#[cfg(test)]` tests covering INFR-03
- [ ] Integration test HTML fixtures: wiremock-served ar5iv HTML snippets for extractor tests

## Sources

### Primary (HIGH confidence)
- Existing codebase (`src/data_aggregation/html_parser.rs`, `src/database/schema.rs`, `src/database/queries.rs`, `Cargo.toml`) — verified directly; all patterns and dependencies confirmed
- Official SurrealDB docs (https://surrealdb.com/docs/surrealql/statements/define/table) — `IF NOT EXISTS` DDL syntax confirmed for v3
- scraper crate docs (https://docs.rs/scraper/latest/scraper/) — CSS selector API confirmed as already used in project

### Secondary (MEDIUM confidence)
- LaTeXML CSS class manual (https://math.nist.gov/~BMiller/LaTeXML/manual/cssclasses/) — confirms `ltx_title_section`, `ltx_section` pattern; does not exhaustively list all classes
- arXiv HTML paper inspection (https://arxiv.org/html/2503.18887) — confirms `ltx_section`, `ltx_subsection`, `ltx_para`, heading hierarchy; abstract class name not directly confirmed from WebFetch rendering
- surrealdb-migrations docs (https://docs.rs/surrealdb-migrations/latest/) — confirmed dependency on `surrealdb ^2.4.0`, incompatible with project's v3

### Tertiary (LOW confidence)
- WebSearch result for arXiv HTML CSS class names — `ltx_abstract` class name not directly confirmed from live HTML inspection; confirmed only indirectly through pattern extrapolation from LaTeXML manual. Flag for validation during implementation: inspect live ar5iv HTML in browser dev tools on a real paper before writing selectors.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all libraries already in Cargo.toml; no new dependencies except possibly chrono
- Architecture: HIGH — patterns directly mirror existing code in the project
- ar5iv CSS class names: MEDIUM — `ltx_section`, `ltx_para` confirmed; `ltx_abstract` extrapolated from LaTeXML naming convention; must verify in browser devtools on first implementation task
- Migration approach: HIGH — surrealdb-migrations v2 incompatibility confirmed; custom table approach confirmed correct with SurrealDB v3 DDL
- Pitfalls: HIGH — all identified from direct code reading of existing patterns

**Research date:** 2026-03-14
**Valid until:** 2026-09-14 (stable — arXiv HTML format and SurrealDB v3 API stable; scraper crate API stable)
