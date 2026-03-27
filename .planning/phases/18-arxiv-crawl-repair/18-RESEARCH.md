# Phase 18: arXiv Crawl Repair - Research

**Researched:** 2026-03-28
**Domain:** Rust text extraction, regex pattern matching, arXiv HTML bibliography parsing
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Extract arXiv IDs in both new format (`YYMM.NNNNN`) and old format (`category/NNNNNNN`) from reference plain text, in addition to existing hyperlink extraction
- **D-02:** Also extract DOI patterns (`10.NNNN/...`) from reference plain text and store as `Reference.doi`. DOIs won't create graph edges but enrich metadata for future use
- **D-03:** When both a hyperlink and a plain-text arXiv ID are found in the same reference, merge both as Links on the Reference and dedup by ID (same arXiv ID not added twice)
- **D-04:** Verify "comparable edge density" via an automated integration test using wiremock — crawl the same seed paper via both arXiv and InspireHEP sources and assert comparable edge counts
- **D-05:** Use a real arXiv HTML page snapshot as the wiremock fixture (not synthetic HTML) to test against actual HTML structure
- **D-06:** Only new crawls benefit from the fix — no backfill or re-crawl mechanism. Document that users should delete their local DB (`data/` directory) and re-crawl from scratch after upgrading

### Claude's Discretion

- Regex pattern design for arXiv ID and DOI extraction from plain text
- Where in the parsing pipeline to inject text-based extraction (within `aggregate_references_for_arxiv_paper` or as a separate pass)
- Threshold for "comparable" edge density in the integration test assertion

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| ARXIV-01 | User can crawl arXiv papers and see citation edges stored for references that mention arXiv IDs in plain text (not just hyperlinked) | Parser fix in `aggregate_references_for_arxiv_paper` + updated `get_arxiv_id()` |
| ARXIV-03 | User can run an arXiv crawl and get comparable edge density to InspireHEP for the same seed paper | Integration test comparing edge counts from both sources against same HTML fixture |
</phase_requirements>

---

## Summary

The arXiv HTML bibliography parser in `resyn-core/src/data_aggregation/arxiv_utils.rs` extracts references by iterating `<span class="ltx_bibblock">` elements and only records arXiv IDs found in `<a href="...">` tags. References that cite papers using plain text like `arXiv:2301.12345` without a hyperlink are silently ignored — the Reference is created but has no Links, so `get_arxiv_id()` fails and the BFS crawler never follows those references. This means arXiv crawls produce sparse graphs while InspireHEP crawls of the same paper are dense.

The fix has two parts: (1) add text-based pattern matching inside `aggregate_references_for_arxiv_paper` to extract arXiv IDs and DOIs from the plain text already collected in `reference_string`, and (2) ensure `get_arxiv_id()` also falls back to the `arxiv_eprint` field so text-extracted IDs participate in edge creation. No new external crates are needed — Rust's standard library `find`/`split` methods suffice for simple patterns, but adding `regex` to `resyn-core` (behind the `ssr` feature) is recommended for correctness and maintainability given the variety of real-world arXiv bibliography formats.

The integration tests for the fix must live in `resyn-core/tests/` (the workspace-level `tests/` directory is orphaned and not compiled). The D-05 requirement for a real HTML snapshot fixture is important: synthetic HTML in prior tests only covers the `<a>` tag path; a real snapshot will exercise the plain-text path with authentic `arXiv:YYMM.NNNNN` patterns.

**Primary recommendation:** Add `regex` to `resyn-core` under the `ssr` feature, extract IDs from `reference_string` after the child-iteration loop, construct an arXiv `Link` and populate `arxiv_eprint` on the Reference, dedup by ID before pushing Links, and update `get_arxiv_id()` to fall back to `arxiv_eprint` when no Link is present.

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `regex` | 1.11 (latest stable) | Compile-time pattern matching for arXiv IDs and DOIs in plain text | Correctness over hand-rolled splits; handles version suffixes, old-format categories, DOI namespaces cleanly |
| `scraper` | 0.23.1 (already in workspace) | HTML DOM traversal — already in use | Existing dependency, no change |
| `wiremock` | 0.6 (already in workspace) | HTTP mocking for integration tests | Already used for arXiv/InspireHEP tests |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| Rust stdlib `str` methods | — | Simple boundary checks, prefix matching | Where regex is overkill (e.g., `starts_with("arXiv:")`) |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `regex` crate | Hand-rolled string splits | Splits are fragile against `arXiv: 2301.12345` (space after colon), versioned IDs, old-format `hep-ph/0601234`. Regex handles all variants cleanly. |
| `regex` crate | `fancy-regex` | `fancy-regex` supports lookaheads but adds ~2MB to binary. Not needed here — no lookaheads required. |

**Installation (workspace Cargo.toml and resyn-core Cargo.toml):**
```bash
# Add to workspace [workspace.dependencies]
regex = "1"

# Add to resyn-core [dependencies] under ssr feature
regex = { workspace = true, optional = true }

# Add to resyn-core [features] ssr list
"dep:regex",
```

**Version verification (confirmed 2026-03-28):**
`regex` 1.11.x is the current stable release. The workspace currently has no `regex` dependency — it must be added.

---

## Architecture Patterns

### Recommended Project Structure

No new files are needed. Changes are confined to:

```
resyn-core/src/
├── data_aggregation/
│   └── arxiv_utils.rs        # Primary change: text extraction in aggregate_references_for_arxiv_paper
├── datamodels/
│   └── paper.rs              # get_arxiv_id() fallback to arxiv_eprint
└── (Cargo.toml)              # Add regex dependency

resyn-core/tests/
└── arxiv_html_parsing.rs     # New integration test file (replaces orphaned workspace tests/)
```

### Pattern 1: Text Extraction Inline in the Reference Loop

**What:** After the child-iteration loop in `aggregate_references_for_arxiv_paper` collects `reference_string`, apply regex patterns to extract arXiv IDs and DOIs before constructing the `Reference`.

**When to use:** Inline is correct here because `reference_string` is fully assembled and DOI/ID candidates need to be extracted before `Reference` is built.

**Example (Rust):**
```rust
// After the child iteration loop, before pushing to references:
use regex::Regex;

// Compile once outside the loop (or use lazy_static / std::sync::OnceLock)
static ARXIV_NEW: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
static ARXIV_OLD: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
static DOI_PAT:   std::sync::OnceLock<Regex> = std::sync::OnceLock::new();

let arxiv_new_re = ARXIV_NEW.get_or_init(|| {
    Regex::new(r"\b(\d{4}\.\d{4,5}(?:v\d+)?)\b").unwrap()
});
let arxiv_old_re = ARXIV_OLD.get_or_init(|| {
    Regex::new(r"\b([a-zA-Z][a-zA-Z0-9\-]+/\d{7}(?:v\d+)?)\b").unwrap()
});
let doi_re = DOI_PAT.get_or_init(|| {
    Regex::new(r"\b(10\.\d{4,}/\S+)").unwrap()
});

// Collect IDs already captured from <a> tags
let mut seen_arxiv_ids: HashSet<String> = links
    .iter()
    .filter_map(|url| {
        if url.contains("arxiv") {
            url.split('/').last().map(|s| strip_version_suffix(s).to_string())
        } else {
            None
        }
    })
    .collect();

// Extract new-format arXiv IDs from text
for cap in arxiv_new_re.captures_iter(&reference_string) {
    let id = strip_version_suffix(&cap[1]);
    if seen_arxiv_ids.insert(id.clone()) {
        links.push(format!("https://arxiv.org/abs/{}", id));
        // also record as arxiv_eprint below
    }
}
// ... similarly for old-format
```

**Key detail:** The dedup set is built from existing `<a>`-tag links first, so text-extracted duplicates are suppressed (D-03).

### Pattern 2: get_arxiv_id() Fallback to arxiv_eprint

**What:** Update `Reference::get_arxiv_id()` to also check `self.arxiv_eprint` when no arXiv Link is found. This is the mechanism by which text-extracted IDs become graph edges.

**Current code:**
```rust
// resyn-core/src/datamodels/paper.rs — Reference::get_arxiv_id()
pub fn get_arxiv_id(&self) -> Result<String, ResynError> {
    let link = self.links.iter().find(|link| matches!(link.journal, Journal::Arxiv));
    match link {
        Some(existing_link) => existing_link.url.split("/").last()...
        None => Err(ResynError::NoArxivLink),
    }
}
```

**After fix:** The primary approach (D-01) is to add an arXiv `Link` to the Reference during parsing, so `get_arxiv_id()` finds it through the existing link-based path. The `arxiv_eprint` field serves as a secondary store for metadata. However, as a belt-and-suspenders fallback, `get_arxiv_id()` should also check `self.arxiv_eprint`:

```rust
pub fn get_arxiv_id(&self) -> Result<String, ResynError> {
    // Primary: check Links for a Journal::Arxiv entry
    if let Some(link) = self.links.iter().find(|l| matches!(l.journal, Journal::Arxiv)) {
        return link.url.split('/').last()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .ok_or(ResynError::NoArxivLink);
    }
    // Fallback: check arxiv_eprint field
    self.arxiv_eprint.clone().ok_or(ResynError::NoArxivLink)
}
```

### Pattern 3: OnceLock for Regex Compilation

**What:** Use `std::sync::OnceLock<Regex>` (stable since Rust 1.70) to compile regex patterns once rather than on every call to `aggregate_references_for_arxiv_paper`.

**Why:** Regex compilation is expensive (~microseconds per compile). The function is called for every paper in a BFS crawl. `OnceLock` is in stdlib — no additional dependency.

**Alternative:** `lazy_static` macro. Not recommended since `OnceLock` is now stable stdlib and `lazy_static` is an extra dependency.

### Pattern 4: Real HTML Fixture via Inline String or File

**What:** For D-05, the wiremock integration test uses a real arXiv HTML page snapshot rather than synthetic HTML.

**How to obtain fixture:** Fetch a known paper's HTML page (e.g., `https://arxiv.org/html/2503.18887`) and save the bibliography section as a string constant or file in `resyn-core/tests/fixtures/`. Include at least 20-30 reference entries to get statistical coverage.

**Fixture storage:** `resyn-core/tests/fixtures/arxiv_2503_18887_biblio.html` — a trimmed HTML document containing only the bibliography `<span class="ltx_bibblock">` entries from the real page.

**Integration test structure:**
```rust
// resyn-core/tests/arxiv_html_parsing.rs
#[tokio::test]
async fn test_arXiv_and_inspirehep_comparable_edge_density() {
    // Mount the same seed paper's data on both mock servers
    // Crawl via ArxivSource → count arxiv_reference_ids
    // Crawl via InspireHepClient → count arxiv_reference_ids
    // Assert: arxiv_count >= inspirehep_count * 0.7  (70% threshold — discretion item)
}
```

### Anti-Patterns to Avoid

- **Recompiling regex on every call:** Compile each regex pattern to a `static OnceLock<Regex>`, not `Regex::new(...)` inside the loop.
- **Extracting IDs from the full HTML text:** Only extract from `reference_string` (the text of a single `<span class="ltx_bibblock">`), not the whole page. Otherwise false positives from abstract/body text will create phantom edges.
- **Not stripping version suffixes on text-extracted IDs:** `strip_version_suffix()` must be applied to text-extracted IDs before dedup, just as it is for IDs from `<a>` tags and from InspireHEP.
- **Forgetting the ssr feature gate:** `regex` is server-side only. Add it under `[features] ssr = [..., "dep:regex"]` and guard any usage with `#[cfg(feature = "ssr")]`.
- **Moving tests to workspace-level tests/:** The `tests/html_parsing.rs` and `tests/inspirehep_integration.rs` at the workspace root are orphaned — they do not belong to any crate and are never compiled. New tests must go in `resyn-core/tests/`.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| arXiv ID pattern matching | Custom split-and-validate logic | `regex` crate | Old-format IDs (`hep-ph/0601234`) have categories with hyphens; new-format needs 4-digit YYMM prefix; version suffixes must be handled. Split logic grows fragile. |
| DOI extraction | Custom string walking | `regex` crate | DOI suffixes can contain slashes, dots, hyphens — any split approach will misidentify boundaries |
| Regex lazy initialization | `lazy_static` macro | `std::sync::OnceLock` | stdlib, no extra dep, stable since Rust 1.70 |

**Key insight:** The arXiv bibliography is unstructured text from LaTeX rendering. Real papers contain patterns like `arXiv:2301.12345`, `arXiv: 2301.12345`, `[arXiv:2301.12345v2]`, and old-format `hep-ph/0601234`. A single well-tested regex handles all of these; hand-rolled splits will miss edge cases.

---

## Common Pitfalls

### Pitfall 1: DOI Pattern Matching Too Eagerly

**What goes wrong:** The DOI regex `10\.\d{4,}/\S+` matches trailing punctuation (commas, periods, closing brackets) as part of the DOI.
**Why it happens:** DOIs can legitimately end in characters like `)` or `.` in some databases, so `\S+` captures them.
**How to avoid:** After extraction, strip trailing punctuation: `id.trim_end_matches(|c: char| !c.is_alphanumeric())`. Alternatively, use `[^\s,;)\]]+` instead of `\S+` in the pattern.
**Warning signs:** Stored DOI values end with `.`, `)`, or `]` characters.

### Pitfall 2: False-Positive arXiv IDs from Non-Reference Text

**What goes wrong:** The arXiv new-format pattern `\d{4}\.\d{4,5}` also matches year.number strings like publication years in journal names (e.g., `Phys. Rev. D 2301.12345` is fine, but `Phys. Rev. 91.1` would not match due to digit count).
**Why it happens:** The pattern is applied to the entire reference text block which may include journal volume/page numbers.
**How to avoid:** Anchor the pattern with `\b` word boundaries and require the 4-digit prefix to look like YYMM (e.g., first two digits `0-2` for year since 2000). The pattern `\b((?:0[0-9]|1[0-9]|2[0-9])\d{2}\.\d{4,5}(?:v\d+)?)\b` limits false positives.
**Warning signs:** Papers suddenly have many more edges than expected, pointing to non-existent arXiv papers.

### Pitfall 3: Old-Format arXiv IDs Contain Slashes

**What goes wrong:** `category/NNNNNNN` patterns (e.g., `hep-ph/0601234`) contain a `/` which also appears in URLs and DOIs.
**Why it happens:** The old-format regex `[a-zA-Z][a-zA-Z0-9\-]+/\d{7}` may match fragments of DOI strings or URLs if not properly anchored.
**How to avoid:** Apply the old-format regex only to the plain-text portion of `reference_string`, after stripping URLs. Alternatively, use a word-boundary anchor before the category name `\b[a-zA-Z][a-zA-Z0-9\-]+/\d{7}`.
**Warning signs:** Old-format IDs match inside URL strings like `inspirehep.net/literature/hep-ph/0601234`.

### Pitfall 4: Orphaned workspace-level tests/ Directory

**What goes wrong:** Writing new integration tests in `/tests/html_parsing.rs` (workspace root) — they are never compiled and silently ignored.
**Why it happens:** The workspace-level `tests/` is not wired to any crate in any `Cargo.toml`. It appears to be a leftover from a pre-workspace single-crate layout.
**How to avoid:** All new integration tests go in `resyn-core/tests/`. The existing orphaned files in `tests/` can be ignored or deleted.
**Warning signs:** `cargo test --test html_parsing` says "no test target named html_parsing".

### Pitfall 5: regex Crate Not Behind ssr Feature Gate

**What goes wrong:** Adding `regex` as a non-optional dependency causes the WASM build (`resyn-app`) to fail or bloat — `regex` is large and compiles for WASM.
**Why it happens:** `resyn-core` has two compilation contexts: native (with `ssr` feature, used by `resyn-server`) and WASM (used by `resyn-app`). Only native needs `regex`.
**How to avoid:** Add `regex = { workspace = true, optional = true }` in `resyn-core/Cargo.toml` and add `"dep:regex"` to the `ssr` feature list. Guard usage with `#[cfg(feature = "ssr")]` where needed (the functions that use it are already ssr-only).

### Pitfall 6: Dedup Set Not Seeded from Existing Links

**What goes wrong:** If a reference has both `<a href="https://arxiv.org/abs/2301.12345">` and plain text `arXiv:2301.12345`, the same ID gets added twice to `links`, creating duplicate Links.
**Why it happens:** Text extraction runs after link collection; without seeding the dedup set from existing links, it adds a duplicate.
**How to avoid:** Initialize the `seen_arxiv_ids: HashSet<String>` by iterating `links` (already collected from `<a>` tags) before running the regex extraction loop. (D-03 requirement.)

---

## Code Examples

Verified patterns from codebase inspection:

### Current aggregate_references_for_arxiv_paper Shape (the function to modify)
```rust
// resyn-core/src/data_aggregation/arxiv_utils.rs lines 10-72
// Key structure:
// - Iterates <span class="ltx_bibblock"> elements
// - For each span, builds reference_string (all text) and links Vec<String> (only <a> hrefs)
// - After loop: builds Reference with links.iter().map(Link::from_url)
// Text extraction inserts between "links collected" and "Reference built"
```

### get_arxiv_id() Current Implementation
```rust
// resyn-core/src/datamodels/paper.rs lines 110-125
// Only checks self.links for Journal::Arxiv — misses arxiv_eprint field
// Must add fallback: self.arxiv_eprint.clone().ok_or(ResynError::NoArxivLink)
```

### InspireHEP Pattern for arxiv_eprint + Link Construction (reference)
```rust
// resyn-core/src/data_aggregation/inspirehep_api.rs lines 182-184
// When arxiv_eprint is present, InspireHEP creates a Link:
if let Some(ref eprint) = arxiv_eprint {
    links.push(Link::from_url(&format!("https://arxiv.org/abs/{}", eprint)));
}
// arXiv source should follow this same pattern for text-extracted IDs
```

### Existing Integration Test Pattern (for new tests to follow)
```rust
// resyn-core/src/data_aggregation/text_extractor.rs lines 430-473
// Uses wiremock MockServer, mounts HTML fixture, calls function directly
// New tests in resyn-core/tests/arxiv_html_parsing.rs follow this pattern
// But use aggregate_references_for_arxiv_paper (not Ar5ivExtractor)
```

### OnceLock Regex Initialization Pattern (stdlib, no extra dep)
```rust
use std::sync::OnceLock;
use regex::Regex;

static ARXIV_NEW_RE: OnceLock<Regex> = OnceLock::new();

fn get_arxiv_new_re() -> &'static Regex {
    ARXIV_NEW_RE.get_or_init(|| {
        Regex::new(r"\b((?:0[0-9]|1[0-9]|2[0-9])\d{2}\.\d{4,5}(?:v\d+)?)\b").unwrap()
    })
}
```

---

## Runtime State Inventory

> Greenfield parser fix — no rename/refactor involved.

Not applicable for this phase.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust stable toolchain | All builds | ✓ | pinned via rust-toolchain.toml | — |
| `regex` crate | Text extraction | ✗ (not yet added) | — | Must add to Cargo.toml |
| `wiremock` | Integration tests | ✓ | 0.6 (workspace) | — |
| `cargo test -p resyn-core --features ssr` | Running new tests | ✓ | — | — |

**Missing dependencies with no fallback:**
- `regex` crate must be added to workspace `Cargo.toml` and `resyn-core/Cargo.toml` before implementation.

**Missing dependencies with fallback:**
- None.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness + `tokio::test` for async |
| Config file | `rust-toolchain.toml` (pinned toolchain) |
| Quick run command | `cargo test -p resyn-core --features ssr 2>&1` |
| Full suite command | `cargo test --workspace 2>&1` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| ARXIV-01 | Plain text arXiv IDs extracted as Links and arxiv_eprint | unit | `cargo test -p resyn-core --features ssr extract_arxiv_id_from_plain_text` | ❌ Wave 0 |
| ARXIV-01 | get_arxiv_id() falls back to arxiv_eprint | unit | `cargo test -p resyn-core --features ssr get_arxiv_id_fallback` | ❌ Wave 0 |
| ARXIV-01 | DOI patterns extracted and stored in Reference.doi | unit | `cargo test -p resyn-core --features ssr extract_doi_from_plain_text` | ❌ Wave 0 |
| ARXIV-01 | Dedup: same arXiv ID from hyperlink + text not duplicated | unit | `cargo test -p resyn-core --features ssr dedup_arxiv_links` | ❌ Wave 0 |
| ARXIV-03 | Real HTML fixture yields >= 70% of InspireHEP edge count | integration | `cargo test -p resyn-core --features ssr --test arxiv_html_parsing` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p resyn-core --features ssr`
- **Per wave merge:** `cargo test --workspace`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `resyn-core/tests/arxiv_html_parsing.rs` — integration tests for ARXIV-01 and ARXIV-03
- [ ] `resyn-core/tests/fixtures/arxiv_2503_18887_biblio.html` — real HTML snippet for D-05 fixture (fetch from `https://arxiv.org/html/2503.18887` and trim to bibliography section)
- [ ] Framework dependency install: add `regex` to `Cargo.toml` workspace and `resyn-core/Cargo.toml`

---

## Project Constraints (from CLAUDE.md)

All of the following apply to this phase:

- **Rust edition 2024** — use edition 2024 features (let-chains already used in codebase)
- **Single `#[tokio::main]` in main.rs** — tests use `#[tokio::test]`, not a new runtime
- **`cargo fmt --all -- --check`** — all code must pass before commit
- **`cargo clippy --all-targets --all-features -- -Dwarnings`** — CI fails on warnings; no unused imports, no dead code
- **`ArxivHTMLDownloader::with_rate_limit(Duration::from_millis(0))`** — disable rate limiting in tests
- **`cargo test`** — all 186 existing tests currently pass; regressions not acceptable
- **ssr feature gate** — server-side-only code (reqwest, scraper, regex) must be behind `#[cfg(feature = "ssr")]`
- **Run binary as `cargo run --bin resyn`** — two binaries in workspace; bare `cargo run` is ambiguous
- **Error handling via `ResynError` with `?` propagation** — no panics in production code paths; crawler logs warnings for individual failures and continues
- **`strip_version_suffix()`** applied at all dedup boundaries — text-extracted IDs are no exception

---

## Open Questions

1. **Threshold for "comparable" edge density (D-04 / ARXIV-03)**
   - What we know: D-04 requires an integration test asserting comparable edge counts between arXiv and InspireHEP sources. "Comparable" is left to Claude's discretion.
   - What's unclear: The right threshold. arXiv HTML may genuinely miss some references that InspireHEP has (e.g., papers only in InspireHEP's database with no arXiv presence). A 100% match is unrealistic.
   - Recommendation: Use 70% as the lower bound (`arxiv_count >= inspirehep_count * 0.70`). This is aggressive enough to catch a regression back to the broken state (where arXiv produces 0 edges) while allowing for genuine gaps. Document the threshold explicitly in the test.

2. **Orphaned workspace-level tests/**
   - What we know: `tests/html_parsing.rs` and `tests/inspirehep_integration.rs` at workspace root are never compiled. They duplicate tests that already exist inline in `inspirehep_api.rs` and `text_extractor.rs`.
   - What's unclear: Whether to delete them or leave them.
   - Recommendation: The planner should include a task to delete the orphaned files (or leave them — they cause no test failures, just dead code). At minimum, do not add new tests there.

3. **regex crate WASM impact**
   - What we know: `regex` is listed as optional in the plan, to be added under `ssr` feature only.
   - What's unclear: Whether `regex` is already transitively pulled into the WASM build by another dependency.
   - Recommendation: After adding `regex` as `optional = true`, run `cargo build --target wasm32-unknown-unknown -p resyn-app` to verify no WASM regressions. If it fails, investigate the feature tree.

---

## Sources

### Primary (HIGH confidence)
- Direct source code inspection: `resyn-core/src/data_aggregation/arxiv_utils.rs` — identified the exact gap (no text extraction after link collection)
- Direct source code inspection: `resyn-core/src/datamodels/paper.rs` — confirmed `get_arxiv_id()` only checks `links`, not `arxiv_eprint`; confirmed `Reference` already has `doi` and `arxiv_eprint` fields
- Direct source code inspection: `resyn-core/src/data_aggregation/inspirehep_api.rs` — confirmed the InspireHEP pattern for building Links from `arxiv_eprint` (the pattern arXiv should mirror)
- Direct source code inspection: `resyn-core/src/validation.rs` — confirmed arXiv ID format rules (new: `YYMM.NNNNN`, old: `category/NNNNNNN`)
- Direct source code inspection: `resyn-core/Cargo.toml` + workspace `Cargo.toml` — confirmed `regex` is not present; `scraper` and `wiremock` are
- Test run: `cargo test -p resyn-core --features ssr` — 186 tests pass; confirmed test harness setup

### Secondary (MEDIUM confidence)
- Rust stdlib docs: `std::sync::OnceLock` available since Rust 1.70; stable and preferred over `lazy_static` for static regex patterns
- `regex` crate 1.11.x — current stable, no known API breaks from the pattern syntax used here

### Tertiary (LOW confidence)
- Assumed: 70% edge density threshold is reasonable; no empirical data from actual arXiv vs InspireHEP crawl comparison was collected during research. This threshold should be validated by the implementer when writing the fixture test.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — codebase directly inspected; only new dependency is `regex`, a well-known crate
- Architecture: HIGH — all integration points identified from source; patterns verified against working InspireHEP code
- Pitfalls: HIGH — derived from reading the exact code that will change, not speculation
- Test threshold (ARXIV-03): LOW — 70% is an educated estimate, not empirically validated

**Research date:** 2026-03-28
**Valid until:** 2026-06-28 (stable domain — arXiv HTML format rarely changes; `regex` API is stable)
