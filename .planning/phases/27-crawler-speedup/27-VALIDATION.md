---
phase: 27
slug: crawler-speedup
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-22
---

# Phase 27 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `cargo test` (existing 46-test suite — Rust stable) |
| **Config file** | `Cargo.toml` (workspace) |
| **Quick run command** | `cargo test -p resyn-core --lib --features ssr -- data_aggregation::openalex_bulk` |
| **Full suite command** | `cargo test --workspace --features ssr` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p resyn-core --lib --features ssr -- data_aggregation::openalex_bulk`
- **After every plan wave:** Run `cargo test --workspace --features ssr`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 27-01-01 | 01 | 1 | API key auth | T-API-01 | API key never logged; passed via header not URL | unit | `cargo test -p resyn-core --lib --features ssr -- data_aggregation::openalex_bulk` | Modify existing tests | ⬜ pending |
| 27-01-02 | 01 | 1 | --api-key clap arg / env var | T-API-01 | `OPENALEX_API_KEY` env var used; `--mailto` removed | integration | `cargo test -p resyn-server` | Add to bulk_ingest.rs | ⬜ pending |
| 27-01-03 | 01 | 1 | Hard fail on missing key | — | Clear error + URL; no unauthenticated fallback | integration | `cargo test -p resyn-server` | Add to bulk_ingest.rs | ⬜ pending |
| 27-02-01 | 02 | 1 | DEFAULT_FILTER_PHYSICS constant | — | Correct concept IDs C26873012, C121864883 | unit | `cargo test -p resyn-server -- bulk_ingest` | Add to bulk_ingest.rs | ⬜ pending |
| 27-02-02 | 02 | 1 | CLAUDE.md concept ID fix | — | C2778407487 removed; C26873012+C121864883 documented | manual | review CLAUDE.md | CLAUDE.md | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Update `openalex_bulk.rs` unit tests: replace `mailto` with `api_key` in test setup (no new test file needed — modify in-place)

*All other tasks extend existing test infrastructure or require manual review.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| CLAUDE.md concept ID corrected | D-06 | Doc-only change; no executable test | `grep "C26873012\|C121864883" CLAUDE.md` confirms presence; `grep "C2778407487" CLAUDE.md` confirms removal |
| `bulk-ingest --db surrealkv://./data-physics` end-to-end | Success Criterion 4 | Requires real OPENALEX_API_KEY + network | `OPENALEX_API_KEY=<key> cargo run --bin resyn -- bulk-ingest --db surrealkv://./data-physics --filter "primary_location.source.id:S4306400194,concepts.id:C26873012|C121864883"` |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
