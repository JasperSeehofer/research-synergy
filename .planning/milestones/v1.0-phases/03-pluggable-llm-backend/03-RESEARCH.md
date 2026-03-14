# Phase 3: Pluggable LLM Backend - Research

**Researched:** 2026-03-14
**Domain:** LLM API integration (Anthropic Claude, Ollama), Rust async trait patterns, SurrealDB schema extension
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Findings as structured objects: `{text: "...", strength: "strong_evidence"}` — enables Phase 4 contradiction detection
- Paper type as fixed enum: experimental, theoretical, review, computational — feeds VIS-01 color-by-type
- Results in new `llm_annotation` table — separate from `paper_analysis`, consistent with Phase 2 separate-tables decision
- API keys via environment variables only: `ANTHROPIC_API_KEY`, `OLLAMA_URL`, etc.
- `--llm-model` flag for model selection within provider, sensible defaults per provider
- Ollama defaults to `http://localhost:11434`, overridable via `OLLAMA_URL`
- LLM trait mirrors `PaperSource` pattern: `#[async_trait]`, `Send + Sync`
- Noop provider: `methods: [], findings: [], open_problems: [], paper_type: "unknown", provider: "noop"`, persists to DB
- Noop logs constructed prompt at debug level
- Single paper LLM failure: skip and continue
- Parse failure: retry once with JSON-only nudge, log raw response at debug, then skip
- Rate limiting: baked into each provider, no CLI flag
- End-of-run summary matching NLP analysis summary style
- `reqwest` 0.12 already a dependency — direct API calls, no SDK crate

### Claude's Discretion
- Methods field granularity and structure (likely structured with category for Phase 4 compatibility)
- Open problems format
- Exact prompt template design for annotation extraction
- Migration version numbering (continues from Phase 2's version 4, so migration 5)
- Caching strategy: per-paper existence check (like `extraction_exists()` / `analysis_exists()` pattern)

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| TEXT-01 | System extracts structured fields (methods, findings, open problems, paper type) from paper abstracts via LLM | LLM trait + Claude/Ollama providers + serde_json deserialization of structured JSON response |
| INFR-01 | LLM backend is pluggable via a trait, supporting at least two providers (Claude API and Ollama) | `LlmProvider` async trait with `#[async_trait]`, `Send + Sync`, mirroring `PaperSource` pattern |
</phase_requirements>

---

## Summary

Phase 3 adds a pluggable LLM annotation layer to the existing pipeline. The architecture is well-defined by CONTEXT.md: an async trait (`LlmProvider`) with three concrete implementations (claude, ollama, noop), a new `llm_annotation` SurrealDB table via migration 5, and a `run_llm_analysis()` function inserted after `run_nlp_analysis()` in `main.rs`.

The key technical constraint is that the `genai` SDK crate (v0.5+) requires reqwest 0.13, which conflicts with the project's reqwest 0.12. This is confirmed as a real blocker. The solution is direct HTTP calls via the existing `reqwest::Client` — identical to how `InspireHepClient` works today. Both Anthropic and Ollama have clean REST APIs requiring no SDK.

Both providers support structured JSON output. Ollama accepts a `format` field in the request body containing a JSON Schema object that constrains model output. Anthropic's API requires prompt engineering (include schema in system prompt, instruct "respond ONLY with valid JSON"), then `serde_json` deserialization. Ollama's schema enforcement is more reliable for structured extraction.

**Primary recommendation:** Use `reqwest` 0.12 direct HTTP calls for both providers, mirroring the `InspireHepClient` builder pattern. Define the `LlmAnnotation` struct with fixed fields. Use Ollama JSON schema enforcement; use prompt-based JSON for Claude. Cache per-paper via `annotation_exists()` check at the start of the analysis loop.

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `reqwest` | 0.12.x (already in Cargo.toml) | HTTP calls to Anthropic API and Ollama | Already present; avoids genai/reqwest 0.13 conflict |
| `async-trait` | 0.1 (already in Cargo.toml) | `#[async_trait]` on `LlmProvider` trait | Project-established pattern for `PaperSource` |
| `serde` / `serde_json` | 1.x (already in Cargo.toml) | Deserialize structured LLM JSON responses | Already present; typed deserialization is idiomatic |
| `tokio::time::sleep` | (tokio full, already present) | Rate limiting between LLM calls | Same mechanism as `InspireHepClient` |
| `tracing` | 0.1 (already present) | Structured logging, debug-level prompt logging | Project standard |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `chrono` | 0.4 (already present) | `annotated_at` timestamp on `LlmAnnotation` | Same as `analyzed_at` in `PaperAnalysis` |
| `surrealdb` | 3 (already present) | `llm_annotation` table CRUD via `LlmAnnotationRepository` | Consistent with all other analysis tables |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Direct reqwest HTTP | `genai` 0.5 SDK | SDK requires reqwest 0.13 — incompatible with project; not viable |
| Direct reqwest HTTP | `anthropic-sdk-rust` (unofficial) | Would need audit for reqwest version; direct HTTP is simpler and fully controlled |
| Direct reqwest HTTP | `clust` crate | Also depends on reqwest; adds dep without benefit given simple API |

**Installation:** No new dependencies required. All needed libraries are already in `Cargo.toml`.

---

## Architecture Patterns

### Recommended Project Structure
```
src/
├── llm/
│   ├── mod.rs          # pub use declarations
│   ├── traits.rs       # LlmProvider async trait
│   ├── claude.rs       # ClaudeProvider (reqwest + ANTHROPIC_API_KEY)
│   ├── ollama.rs       # OllamaProvider (reqwest + OLLAMA_URL)
│   └── noop.rs         # NoopProvider (instant, logs prompt at debug)
├── datamodels/
│   └── llm_annotation.rs  # LlmAnnotation, Finding, Method, PaperType structs
├── database/
│   ├── schema.rs       # migration 5: llm_annotation table
│   └── queries.rs      # LlmAnnotationRepository (upsert/get/exists/get_all)
└── main.rs             # --llm-provider / --llm-model flags, run_llm_analysis()
```

### Pattern 1: LlmProvider Trait (mirrors PaperSource)

**What:** Async trait with `Send + Sync` for swappable LLM backends.
**When to use:** All LLM calls go through this trait; main.rs constructs the concrete impl based on `--llm-provider` flag.

```rust
// Source: mirrors src/data_aggregation/traits.rs pattern
use async_trait::async_trait;
use crate::datamodels::llm_annotation::LlmAnnotation;
use crate::error::ResynError;

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn annotate_paper(
        &self,
        arxiv_id: &str,
        abstract_text: &str,
    ) -> Result<LlmAnnotation, ResynError>;
    fn provider_name(&self) -> &'static str;
}
```

Note: Unlike `PaperSource`, `annotate_paper` takes `&self` (not `&mut self`) because rate limiting is internal state that can use `tokio::Mutex` or an `Arc<Mutex<Instant>>` — or alternatively `&mut self` like `InspireHepClient` if the provider is not shared. Given `run_llm_analysis()` calls annotate sequentially (one paper at a time), `&mut self` is fine and simpler — matches `InspireHepClient`.

### Pattern 2: ClaudeProvider HTTP Request

**What:** Direct reqwest POST to Anthropic Messages API.
**When to use:** `--llm-provider claude`.

```rust
// Source: verified from https://platform.claude.com/docs/en/api/getting-started
// Required headers:
//   x-api-key: <ANTHROPIC_API_KEY env var>
//   anthropic-version: 2023-06-01
//   content-type: application/json

let body = serde_json::json!({
    "model": self.model,        // default: "claude-haiku-4-5"
    "max_tokens": 1024,
    "system": SYSTEM_PROMPT,    // instruct JSON-only output with schema
    "messages": [
        {"role": "user", "content": abstract_text}
    ]
});

let resp = self.client
    .post("https://api.anthropic.com/v1/messages")
    .header("x-api-key", &self.api_key)
    .header("anthropic-version", "2023-06-01")
    .header("content-type", "application/json")
    .json(&body)
    .send()
    .await?;

// Response: resp.json::<ClaudeResponse>() where content[0].text is the JSON string
// Parse: serde_json::from_str::<LlmAnnotation>(&content_text)
```

### Pattern 3: OllamaProvider HTTP Request with JSON Schema

**What:** POST to `http://localhost:11434/api/chat` with `format` field containing JSON Schema.
**When to use:** `--llm-provider ollama`.

```rust
// Source: verified from https://docs.ollama.com/api/chat
let body = serde_json::json!({
    "model": self.model,     // default: "llama3.2"
    "messages": [
        {"role": "system", "content": SYSTEM_PROMPT},
        {"role": "user", "content": abstract_text}
    ],
    "stream": false,
    "format": LLM_ANNOTATION_JSON_SCHEMA,  // full JSON Schema object
    "options": {"temperature": 0}           // deterministic for structured output
});

// Response: resp.message.content is a JSON string
// Parse: serde_json::from_str::<LlmAnnotation>(&resp_body["message"]["content"])
```

Ollama's `format` schema enforcement is more reliable than prompt-only approaches. Temperature 0 is recommended by Ollama docs for schema-constrained output.

### Pattern 4: Annotation Repository (mirrors AnalysisRepository)

**What:** `LlmAnnotationRepository` struct wrapping `&Db`, with methods matching the established pattern.
**When to use:** All DB reads/writes for `llm_annotation` table.

```rust
// Source: mirrors src/database/queries.rs AnalysisRepository pattern
pub struct LlmAnnotationRepository<'a> {
    db: &'a Db,
}

impl<'a> LlmAnnotationRepository<'a> {
    pub fn new(db: &'a Db) -> Self { Self { db } }
    pub async fn upsert_annotation(&self, ann: &LlmAnnotation) -> Result<(), ResynError> { ... }
    pub async fn get_annotation(&self, arxiv_id: &str) -> Result<Option<LlmAnnotation>, ResynError> { ... }
    pub async fn annotation_exists(&self, arxiv_id: &str) -> Result<bool, ResynError> { ... }
    pub async fn get_all_annotations(&self) -> Result<Vec<LlmAnnotation>, ResynError> { ... }
}
```

### Pattern 5: Caching in run_llm_analysis()

**What:** Per-paper skip guard at the start of the annotation loop, mirroring `extraction_exists()` pattern.
**When to use:** Every invocation of LLM analysis.

```rust
// Source: mirrors run_analysis() in src/main.rs
for paper in &all_papers {
    let stripped_id = utils::strip_version_suffix(&paper.id);
    if llm_repo.annotation_exists(&stripped_id).await.unwrap_or(false) {
        skipped_count += 1;
        continue;  // zero API calls for cached papers
    }
    // ... annotate and persist
}
```

Unlike TF-IDF (corpus fingerprint guard), LLM annotations are per-paper — a corpus fingerprint guard would invalidate all cached work on each new paper. Per-paper `annotation_exists()` check is correct.

### Pattern 6: SurrealDB Schema for LlmAnnotation

**What:** SCHEMAFULL table for scalar fields; FLEXIBLE for complex LLM output arrays.
**When to use:** Migration 5.

The `methods` and `findings` arrays contain nested objects. SurrealDB SCHEMAFULL cannot type-check nested object fields within arrays. Use `TYPE array FLEXIBLE` or store as `serde_json::Value` (like `tfidf_vector` in migration 3).

```sql
-- Migration 5
DEFINE TABLE IF NOT EXISTS llm_annotation SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS arxiv_id ON llm_annotation TYPE string;
DEFINE FIELD IF NOT EXISTS paper_type ON llm_annotation TYPE string;
DEFINE FIELD IF NOT EXISTS methods ON llm_annotation TYPE array;
DEFINE FIELD IF NOT EXISTS findings ON llm_annotation TYPE array;
DEFINE FIELD IF NOT EXISTS open_problems ON llm_annotation TYPE array<string>;
DEFINE FIELD IF NOT EXISTS provider ON llm_annotation TYPE string;
DEFINE FIELD IF NOT EXISTS model_name ON llm_annotation TYPE string;
DEFINE FIELD IF NOT EXISTS annotated_at ON llm_annotation TYPE string;
DEFINE INDEX IF NOT EXISTS idx_llm_annotation_arxiv_id ON llm_annotation FIELDS arxiv_id UNIQUE;
```

Note: `methods` and `findings` stored as `array` without element type constraint — avoids SCHEMAFULL vs nested-object incompatibility (Phase 2 lesson: `tfidf_vector` uses FLEXIBLE). The `SurrealValue` derive macro on `LlmAnnotationRecord` will serialize nested structs as JSON, use `serde_json::Value` for these fields in the DB record struct.

### Anti-Patterns to Avoid

- **Using `genai` or any SDK crate:** All Rust LLM SDKs found use reqwest 0.12 or 0.13 — the confirmed genai 0.5 uses 0.13, incompatible with project's reqwest 0.12.
- **Corpus-level cache guard for LLM:** A fingerprint guard (like TF-IDF) would re-analyze all papers whenever one paper is added. LLM calls are expensive; per-paper caching is essential.
- **Blocking on all LLM failures:** Matches Phase 1/2 philosophy — individual paper failures are logged and skipped, never block the pipeline.
- **Nested SCHEMAFULL OBJECTs in SurrealDB:** Confirmed pitfall from Phase 2 — use `serde_json::Value` for complex nested fields in DB record structs.
- **`&self` with mutable rate-limit state:** The rate-limit `last_called: Option<Instant>` field requires mutation. Use `&mut self` like `InspireHepClient` since `run_llm_analysis()` owns the provider and calls it sequentially.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| JSON parsing of LLM response | Custom parser | `serde_json::from_str::<LlmAnnotation>()` | serde handles all edge cases; already a dep |
| HTTP request building | Manual string concat | `reqwest::Client` builder with `.json()` | Already present, handles encoding/headers |
| Rate limiting | Custom sleep logic | `tokio::time::sleep` + `Instant` pattern | Identical to `InspireHepClient::rate_limit_check()` — copy the pattern |
| DB record type bridging | Manual SurrealDB serialization | `SurrealValue` derive + `serde_json::Value` for complex fields | Phase 2 established this pattern for `AnalysisRecord` |
| Schema migration | Custom DDL runner | `migrate_schema()` `if version < N` pattern | Already established and idempotent |
| Timestamp generation | Custom | `chrono::Utc::now().to_rfc3339()` | Project-wide standard |

**Key insight:** The entire LLM provider infrastructure pattern already exists in the codebase — it's a near-direct port of `InspireHepClient` (HTTP) combined with `AnalysisRepository` (DB). No novel patterns needed.

---

## Common Pitfalls

### Pitfall 1: Anthropic API `anthropic-version` Header Missing
**What goes wrong:** API returns 400 or 401 with "missing version header".
**Why it happens:** Unlike most REST APIs, Anthropic requires `anthropic-version: 2023-06-01` in every request — not just `x-api-key` and `content-type`.
**How to avoid:** Always set all three required headers: `x-api-key`, `anthropic-version: 2023-06-01`, `content-type: application/json`.
**Warning signs:** HTTP 400 responses from `api.anthropic.com` with no apparent auth error.

### Pitfall 2: Ollama `stream: false` Omission
**What goes wrong:** Response comes back as streaming NDJSON chunks instead of a single JSON object; parsing fails.
**Why it happens:** Ollama streams by default. Omitting `"stream": false` in the request body causes a streaming response.
**How to avoid:** Always include `"stream": false` in Ollama request body.
**Warning signs:** `serde_json::from_str` fails with "trailing characters" error on the response body.

### Pitfall 3: SurrealDB Nested Array Objects in SCHEMAFULL
**What goes wrong:** Migration succeeds, but UPSERT of records with nested structs (methods, findings) fails at runtime.
**Why it happens:** SurrealDB SCHEMAFULL validates field types strictly; array items with object subfields require FLEXIBLE treatment.
**How to avoid:** Declare `methods` and `findings` as `TYPE array` (no item type), and use `serde_json::Value` in the Rust record struct — exact same pattern as `tfidf_vector` in `AnalysisRecord`.
**Warning signs:** "Field 'methods[0].category' not allowed" SurrealDB errors.

### Pitfall 4: LLM Response Not Pure JSON
**What goes wrong:** `serde_json::from_str` fails even with JSON instructions because Claude wraps the JSON in prose ("Here is the JSON: ...").
**Why it happens:** LLMs tend to narrate unless forcefully constrained. Ollama's `format` field eliminates this; Claude requires strong prompt discipline.
**How to avoid:** For Claude: include in system prompt "Respond ONLY with a JSON object. No text before or after." For retry: include "IMPORTANT: respond ONLY with valid JSON, nothing else." Log raw response at debug level on parse failure.
**Warning signs:** `serde_json::Error` with "expected value at line 1 column 1" or parse error at first non-JSON character.

### Pitfall 5: Environment Variable Not Set at Runtime
**What goes wrong:** `std::env::var("ANTHROPIC_API_KEY")` returns `Err`, causing a panic or unhelpful error.
**Why it happens:** User forgot to export env var before running.
**How to avoid:** Check env vars at startup of `run_llm_analysis()` before the paper loop, return `ResynError::LlmApi("ANTHROPIC_API_KEY not set".into())` with a clear message. Do not panic.
**Warning signs:** Silent skip of entire LLM analysis or panic at thread boundary.

### Pitfall 6: reqwest 0.13 Dep Introduced via Transitive Dependency
**What goes wrong:** `cargo build` fails with "mismatched reqwest versions" or features conflict.
**Why it happens:** Adding any LLM SDK crate that depends on reqwest 0.13 causes conflict.
**How to avoid:** Add no new HTTP or LLM SDK crates. Use direct reqwest 0.12 calls only.
**Warning signs:** `cargo build` output showing two incompatible versions of reqwest in dependency tree.

---

## Code Examples

### Anthropic API Call Pattern

```rust
// Source: https://platform.claude.com/docs/en/api/getting-started (verified 2026-03-14)

// Required headers for every request:
// x-api-key: $ANTHROPIC_API_KEY
// anthropic-version: 2023-06-01
// content-type: application/json

#[derive(Deserialize)]
struct ClaudeResponseContent {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

#[derive(Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeResponseContent>,
}

async fn call_claude(client: &Client, api_key: &str, model: &str, system: &str, user: &str)
    -> Result<String, ResynError>
{
    let body = serde_json::json!({
        "model": model,
        "max_tokens": 1024,
        "system": system,
        "messages": [{"role": "user", "content": user}]
    });

    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| ResynError::LlmApi(format!("Claude request failed: {e}")))?;

    if resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
        return Err(ResynError::LlmApi("Claude rate limit hit (429)".into()));
    }
    if !resp.status().is_success() {
        return Err(ResynError::LlmApi(format!("Claude HTTP {}", resp.status())));
    }

    let parsed: ClaudeResponse = resp.json().await
        .map_err(|e| ResynError::LlmApi(format!("Claude response parse failed: {e}")))?;
    Ok(parsed.content.into_iter()
        .find(|c| c.content_type == "text")
        .map(|c| c.text)
        .unwrap_or_default())
}
```

### Ollama API Call Pattern

```rust
// Source: https://docs.ollama.com/api/chat (verified 2026-03-14)
// Endpoint: POST http://localhost:11434/api/chat
// stream: false to get single response object
// format: JSON Schema object for structured output

#[derive(Deserialize)]
struct OllamaMessage { content: String }
#[derive(Deserialize)]
struct OllamaResponse { message: OllamaMessage }

async fn call_ollama(client: &Client, base_url: &str, model: &str, system: &str, user: &str, schema: &serde_json::Value)
    -> Result<String, ResynError>
{
    let body = serde_json::json!({
        "model": model,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": user}
        ],
        "stream": false,
        "format": schema,
        "options": {"temperature": 0}
    });

    let resp = client
        .post(format!("{base_url}/api/chat"))
        .json(&body)
        .send()
        .await
        .map_err(|e| ResynError::LlmApi(format!("Ollama request failed: {e}")))?;

    if !resp.status().is_success() {
        return Err(ResynError::LlmApi(format!("Ollama HTTP {}", resp.status())));
    }

    let parsed: OllamaResponse = resp.json().await
        .map_err(|e| ResynError::LlmApi(format!("Ollama response parse: {e}")))?;
    Ok(parsed.message.content)
}
```

### LlmAnnotation Struct Design

```rust
// Recommended struct — designed for Phase 4 contradiction detection
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Finding {
    pub text: String,
    pub strength: String,   // "strong_evidence" | "moderate_evidence" | "preliminary" | "inconclusive"
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Method {
    pub name: String,
    pub category: String,   // "experimental" | "theoretical" | "computational" | "statistical"
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LlmAnnotation {
    pub arxiv_id: String,
    pub paper_type: String,         // "experimental" | "theoretical" | "review" | "computational"
    pub methods: Vec<Method>,
    pub findings: Vec<Finding>,
    pub open_problems: Vec<String>,
    pub provider: String,           // "claude" | "ollama" | "noop"
    pub model_name: String,
    pub annotated_at: String,       // chrono::Utc::now().to_rfc3339()
}
```

### JSON Schema for Ollama format field

```rust
// Embed as a const or lazy_static — passed directly as the `format` field
pub const LLM_ANNOTATION_SCHEMA: &str = r#"{
  "type": "object",
  "properties": {
    "paper_type": {"type": "string", "enum": ["experimental", "theoretical", "review", "computational"]},
    "methods": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "name": {"type": "string"},
          "category": {"type": "string"}
        },
        "required": ["name", "category"]
      }
    },
    "findings": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "text": {"type": "string"},
          "strength": {"type": "string", "enum": ["strong_evidence", "moderate_evidence", "preliminary", "inconclusive"]}
        },
        "required": ["text", "strength"]
      }
    },
    "open_problems": {"type": "array", "items": {"type": "string"}}
  },
  "required": ["paper_type", "methods", "findings", "open_problems"]
}"#;
```

### Parse-with-Retry Pattern

```rust
// First attempt: parse directly
let json_text = provider_call(abstract_text).await?;
let annotation = match serde_json::from_str::<LlmAnnotationRaw>(&json_text) {
    Ok(a) => a,
    Err(e) => {
        debug!(raw_response = json_text.as_str(), "First parse failed: {e}");
        // Retry with stricter prompt
        let retry_text = provider_retry(abstract_text).await?;
        match serde_json::from_str::<LlmAnnotationRaw>(&retry_text) {
            Ok(a) => a,
            Err(e2) => {
                debug!(raw_response = retry_text.as_str(), "Retry parse also failed: {e2}");
                warn!(arxiv_id, "LLM parse failed after retry, skipping paper");
                skipped_count += 1;
                continue;
            }
        }
    }
};
```

### run_llm_analysis() Skeleton

```rust
// Mirrors run_nlp_analysis() in src/main.rs
async fn run_llm_analysis(db: &Db, provider: &mut dyn LlmProvider) {
    let paper_repo = database::queries::PaperRepository::new(db);
    let llm_repo = database::queries::LlmAnnotationRepository::new(db);

    let all_papers = paper_repo.get_all_papers().await.unwrap_or_else(|e| {
        error!(error = %e, "Failed to load papers for LLM analysis");
        std::process::exit(1);
    });

    let (mut annotated, mut skipped) = (0usize, 0usize);

    for paper in &all_papers {
        let id = utils::strip_version_suffix(&paper.id);
        if llm_repo.annotation_exists(&id).await.unwrap_or(false) {
            skipped += 1;
            continue;
        }
        // Use abstract_text (or full text if available from text_extraction table)
        match provider.annotate_paper(&id, &paper.summary).await {
            Ok(ann) => {
                if let Err(e) = llm_repo.upsert_annotation(&ann).await {
                    error!(paper_id = id.as_str(), error = %e, "Failed to persist annotation");
                }
                annotated += 1;
            }
            Err(e) => {
                warn!(paper_id = id.as_str(), error = %e, "LLM annotation failed, skipping");
                skipped += 1;
            }
        }
    }

    info!(
        annotated, skipped, total = all_papers.len(), provider = provider.provider_name(),
        "LLM analysis: {}/{} papers annotated ({} skipped), provider: {}",
        annotated, all_papers.len() - skipped, skipped, provider.provider_name()
    );
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `genai` SDK for multi-provider LLM | Direct reqwest HTTP calls | Now (genai 0.5 uses reqwest 0.13) | Must use raw HTTP — actually simpler for this use case |
| Prompt-only JSON from LLMs | Ollama `format` JSON Schema enforcement | Ollama 2024 structured outputs feature | Ollama output is schema-constrained, not just requested |
| Single retry or fail-fast on parse error | Parse → retry with nudge → skip | Established CONTEXT.md decision | Balances reliability with pipeline continuity |

**Deprecated/outdated:**
- `genai` 0.1-0.4 (reqwest 0.12): Would have been compatible, but project adopts direct HTTP anyway for simplicity and no SDK dep.

---

## Open Questions

1. **Text source for annotation: abstract vs. full text**
   - What we know: `run_llm_analysis()` can read from `text_extraction` table (Ar5ivHtml full text) or from `paper.summary` (abstract)
   - What's unclear: Whether full text should be used when available (richer extraction) or always abstract (consistent, smaller, cheaper)
   - Recommendation: Start with abstract (`paper.summary`) — cheaper, consistent, and avoids per-paper branching logic; can upgrade to full text in v2 (TEXT-05)

2. **Anthropic rate limit tier for new API keys**
   - What we know: Tier 1 (new accounts) has low RPM/TPM limits; 429 responses include `retry-after` header
   - What's unclear: Whether the baked-in rate limit delay is sufficient for all tiers
   - Recommendation: Default delay of 1s for Claude (conservative for Tier 1); `InspireHepClient` pattern applies — configurable delay per provider constant

3. **Ollama model availability validation**
   - What we know: If the requested model isn't pulled, Ollama returns an error
   - What's unclear: Whether to validate model exists before the loop or handle per-request
   - Recommendation: First call will fail and be caught per the skip-and-continue policy; log a clear warning pointing to `ollama pull <model>`

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness + `wiremock` (already in dev-dependencies) |
| Config file | none (Cargo.toml test configuration) |
| Quick run command | `cargo test llm` |
| Full suite command | `cargo test` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| INFR-01 | `LlmProvider` trait is `Send + Sync`, all three providers implement it | unit | `cargo test llm::tests` | ❌ Wave 0 |
| INFR-01 | NoopProvider returns empty-but-valid `LlmAnnotation` instantly | unit | `cargo test noop_provider` | ❌ Wave 0 |
| INFR-01 | ClaudeProvider sends correct headers (x-api-key, anthropic-version) | integration (wiremock) | `cargo test claude_integration` | ❌ Wave 0 |
| INFR-01 | OllamaProvider sends `stream: false` and `format` schema in body | integration (wiremock) | `cargo test ollama_integration` | ❌ Wave 0 |
| INFR-01 | `--llm-provider` flag selects correct provider at CLI level | unit | `cargo test cli_llm_provider` | ❌ Wave 0 |
| TEXT-01 | `LlmAnnotation` serializes/deserializes via serde roundtrip | unit | `cargo test llm_annotation_serde` | ❌ Wave 0 |
| TEXT-01 | `LlmAnnotationRepository` upsert/get/exists/get_all work against in-memory SurrealDB | unit (DB) | `cargo test llm_annotation_repository` | ❌ Wave 0 |
| TEXT-01 | Per-paper caching: second call returns cached result without LLM invocation | integration | `cargo test llm_cache_skip` | ❌ Wave 0 |
| TEXT-01 | Parse failure triggers retry, second failure skips and continues | unit | `cargo test llm_parse_retry` | ❌ Wave 0 |
| TEXT-01 | End-of-run summary log emits correct annotated/skipped/total counts | unit | `cargo test llm_summary_log` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test llm`
- **Per wave merge:** `cargo test`
- **Phase gate:** Full suite (`cargo test`) green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src/llm/mod.rs` — module declarations
- [ ] `src/llm/traits.rs` — `LlmProvider` trait definition
- [ ] `src/llm/noop.rs` — `NoopProvider` implementation
- [ ] `src/llm/claude.rs` — `ClaudeProvider` with wiremock tests
- [ ] `src/llm/ollama.rs` — `OllamaProvider` with wiremock tests
- [ ] `src/datamodels/llm_annotation.rs` — `LlmAnnotation`, `Finding`, `Method` structs with serde roundtrip tests
- [ ] Migration 5 in `src/database/schema.rs` — `llm_annotation` table DDL
- [ ] `LlmAnnotationRepository` in `src/database/queries.rs` — with in-memory DB tests

No new test frameworks needed — `wiremock` and `tokio::test` already present in dev-dependencies.

---

## Sources

### Primary (HIGH confidence)
- `https://platform.claude.com/docs/en/api/getting-started` — Confirmed required headers: `x-api-key`, `anthropic-version: 2023-06-01`, `content-type: application/json`; request/response format for Messages API
- `https://docs.ollama.com/api/chat` — Confirmed `/api/chat` endpoint, `stream: false` parameter, `format` JSON Schema field for structured output, response shape `message.content`
- Existing codebase: `src/data_aggregation/traits.rs`, `src/data_aggregation/inspirehep_api.rs`, `src/database/queries.rs`, `src/database/schema.rs`, `src/main.rs` — patterns to mirror directly

### Secondary (MEDIUM confidence)
- WebSearch confirming `genai` 0.5 uses reqwest 0.13 (multiple sources agree, matches STATE.md blocker)
- Ollama structured outputs blog post — confirms `temperature: 0` recommendation for schema-constrained output; JSON Schema in `format` field is the stable API

### Tertiary (LOW confidence)
- Specific rate limit delays (1s for Claude default) — based on Tier 1 conservative estimate; should be validated against actual API key tier

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all dependencies already present in Cargo.toml; no new crates needed
- Architecture: HIGH — directly mirrors established patterns in codebase; trait, repository, migration patterns are all established
- API integration: HIGH — verified from official Anthropic and Ollama documentation
- reqwest conflict: HIGH — STATE.md blocker confirmed by multiple web sources (genai 0.5 → reqwest 0.13)
- Pitfalls: HIGH — Anthropic header requirement verified from official docs; Ollama stream:false and SurrealDB nested objects from Phase 2 learnings
- Test architecture: HIGH — mirrors existing wiremock integration test patterns from InspireHEP tests

**Research date:** 2026-03-14
**Valid until:** 2026-06-14 (APIs are stable; Anthropic version header `2023-06-01` has been stable for 3 years)
