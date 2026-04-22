---
phase: 27-crawler-speedup
verified: 2026-04-22T16:05:00Z
status: passed
score: 8/8 must-haves verified
overrides_applied: 0
---

# Phase 27: Crawler Speedup Verification Report

**Phase Goal:** Eliminate per-paper HTML scrapes via OpenAlex bulk reference-edge pre-ingest; wire OpenAlex API key; fix concept IDs in CLAUDE.md
**Verified:** 2026-04-22T16:05:00Z
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | bulk-ingest sends Authorization: Bearer header to OpenAlex, not mailto query param | VERIFIED | `openalex_bulk.rs:161` — `.header("Authorization", format!("Bearer {}", self.api_key))`; URL at line 154-157 contains no `mailto=` param |
| 2 | bulk-ingest fails immediately with a clear error if OPENALEX_API_KEY is not set | VERIFIED | `bulk_ingest.rs:67-75` — match guard exits with `tracing::error!` + `std::process::exit(1)` and message includes `openalex.org/settings/api` URL |
| 3 | The --mailto arg is completely removed from BulkIngestArgs (no deprecation shim) | VERIFIED | `grep "DEFAULT_MAILTO\|pub mailto\|args\.mailto" bulk_ingest.rs` — zero matches |
| 4 | Existing tests pass with the new api_key field in place of mailto | VERIFIED | `cargo test -p resyn-core --lib --features ssr -- data_aggregation::openalex_bulk` — 6 passed; `cargo test -p resyn-server --lib` — 2 new tests pass |
| 5 | DEFAULT_FILTER_PHYSICS constant is present in bulk_ingest.rs with exact concept IDs C26873012 and C121864883 | VERIFIED | `bulk_ingest.rs:16-17` — `const DEFAULT_FILTER_PHYSICS: &str = "primary_location.source.id:S4306400194,concepts.id:C26873012\|C121864883"` |
| 6 | CLAUDE.md no longer contains C2778407487 as a concept ID | VERIFIED | `grep "C2778407487" CLAUDE.md` — zero matches |
| 7 | CLAUDE.md documents C26873012 (Condensed matter) and C121864883 (Statistical physics) as the correct physics concept IDs | VERIFIED | `CLAUDE.md:174` — `C26873012`=Condensed matter physics, `C121864883`=Statistical physics in Important Notes |
| 8 | CLAUDE.md bulk-ingest example uses --api-key and not --mailto | VERIFIED | `CLAUDE.md:50,53` — both bulk-ingest examples use `--api-key "$OPENALEX_API_KEY"`; `grep -- "--mailto" CLAUDE.md` — zero matches |

**Score:** 8/8 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `resyn-core/src/data_aggregation/openalex_bulk.rs` | OpenAlexBulkLoader with api_key field and Authorization: Bearer header | VERIFIED | `api_key: String` field at line 142; Bearer header at line 161; no `mailto` in URL or field |
| `resyn-server/src/commands/bulk_ingest.rs` | BulkIngestArgs with --api-key clap arg backed by OPENALEX_API_KEY env var | VERIFIED | `api_key: Option<String>` at line 34 with `#[arg(long, env = "OPENALEX_API_KEY")]`; `DEFAULT_FILTER_PHYSICS` at lines 16-17; hard-fail guard at lines 67-75 |
| `CLAUDE.md` | Corrected concept IDs and updated bulk-ingest documentation | VERIFIED | C2778407487 absent; C26873012 + C121864883 labelled correctly; --api-key examples in both commands block and Important Notes |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `bulk_ingest.rs` run() | `openalex_bulk.rs` OpenAlexBulkLoader::new | `OpenAlexBulkLoader::new(client, &api_key)` | WIRED | `bulk_ingest.rs:89` — `let loader = OpenAlexBulkLoader::new(client, &api_key);` using the extracted local `api_key` variable, not `args.api_key` |
| `openalex_bulk.rs` fetch_page | OpenAlex API | Authorization header | WIRED | Line 161 sets `Authorization: Bearer {api_key}`; URL has no `mailto=` param |
| `CLAUDE.md` bulk-ingest example | DEFAULT_FILTER_PHYSICS constant | same filter string value | WIRED | `CLAUDE.md:54` and `bulk_ingest.rs:17` both carry `concepts.id:C26873012\|C121864883` |

### Data-Flow Trace (Level 4)

Not applicable — phase modifies an HTTP authentication header and CLI arg routing, not data-rendering components. The data flow (fetch pages → upsert papers → spill citations → translate → write edges) was pre-existing and unchanged by this phase.

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| api_key flag parsed correctly | `cargo test -p resyn-server --lib -- test_api_key_flag_parsed` | ok | PASS |
| absent api_key is None | `cargo test -p resyn-server --lib -- test_no_api_key_is_none` | ok | PASS |
| openalex_bulk unit tests (regression) | `cargo test -p resyn-core --lib --features ssr -- data_aggregation::openalex_bulk` | 6 passed | PASS |
| Workspace compiles clean | `cargo check --workspace --features ssr` | Finished dev profile, 0 errors | PASS |

### Requirements Coverage

No requirement IDs declared for this phase. Goal fully implemented across Plans 01 and 02.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `bulk_ingest.rs` | 15 | `#[allow(dead_code)]` on `DEFAULT_FILTER_PHYSICS` | Info | Constant defined but not yet referenced from a clap default; suppressed the compiler warning deliberately. No functional impact — constant is the documented physics filter value for operator use via `--filter`. |

No blocking anti-patterns. The `#[allow(dead_code)]` is correct: `DEFAULT_FILTER_PHYSICS` is a named constant for documentation and copy-paste use, not yet wired as a clap `default_value`. This matches the plan intent ("no more relying on a commented-out filter string").

### Human Verification Required

None. All acceptance criteria are mechanically verifiable and confirmed.

### Gaps Summary

No gaps. All 8 must-have truths verified against the actual codebase:

- Plan 01: `OpenAlexBulkLoader.api_key` replaces `.mailto`; `Authorization: Bearer` header wired; `--mailto` arg and `DEFAULT_MAILTO` constant fully removed; hard-fail guard on absent key; 2 new unit tests green; 6 regression tests green.
- Plan 02: `DEFAULT_FILTER_PHYSICS` present with exact value; CLAUDE.md carries zero occurrences of `C2778407487`; correct physics concept IDs (`C26873012`, `C121864883`) documented with accurate labels; bulk-ingest examples updated to `--api-key`/`OPENALEX_API_KEY`.

---

_Verified: 2026-04-22T16:05:00Z_
_Verifier: Claude (gsd-verifier)_
