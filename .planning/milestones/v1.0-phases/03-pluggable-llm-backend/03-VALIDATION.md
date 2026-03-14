---
phase: 3
slug: pluggable-llm-backend
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-14
---

# Phase 3 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness + `wiremock` (already in dev-dependencies) |
| **Config file** | Cargo.toml (existing test configuration) |
| **Quick run command** | `cargo test llm` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test llm`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 3-01-01 | 01 | 1 | INFR-01 | unit | `cargo test llm::tests` | ❌ W0 | ⬜ pending |
| 3-01-02 | 01 | 1 | INFR-01 | unit | `cargo test noop_provider` | ❌ W0 | ⬜ pending |
| 3-01-03 | 01 | 1 | TEXT-01 | unit | `cargo test llm_annotation_serde` | ❌ W0 | ⬜ pending |
| 3-02-01 | 02 | 1 | TEXT-01 | unit (DB) | `cargo test llm_annotation_repository` | ❌ W0 | ⬜ pending |
| 3-03-01 | 03 | 2 | INFR-01 | integration (wiremock) | `cargo test claude_integration` | ❌ W0 | ⬜ pending |
| 3-03-02 | 03 | 2 | INFR-01 | integration (wiremock) | `cargo test ollama_integration` | ❌ W0 | ⬜ pending |
| 3-04-01 | 04 | 3 | TEXT-01 | integration | `cargo test llm_cache_skip` | ❌ W0 | ⬜ pending |
| 3-04-02 | 04 | 3 | TEXT-01 | unit | `cargo test llm_parse_retry` | ❌ W0 | ⬜ pending |
| 3-04-03 | 04 | 3 | INFR-01 | unit | `cargo test cli_llm_provider` | ❌ W0 | ⬜ pending |
| 3-04-04 | 04 | 3 | TEXT-01 | unit | `cargo test llm_summary_log` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/llm/mod.rs` — module declarations
- [ ] `src/llm/traits.rs` — `LlmProvider` trait definition
- [ ] `src/llm/noop.rs` — `NoopProvider` implementation with unit tests
- [ ] `src/llm/claude.rs` — `ClaudeProvider` with wiremock tests
- [ ] `src/llm/ollama.rs` — `OllamaProvider` with wiremock tests
- [ ] `src/datamodels/llm_annotation.rs` — `LlmAnnotation`, `Finding`, `Method` structs with serde roundtrip tests
- [ ] Migration 5 in `src/database/schema.rs` — `llm_annotation` table DDL
- [ ] `LlmAnnotationRepository` in `src/database/queries.rs` — with in-memory DB tests

*No new test frameworks needed — `wiremock` and `tokio::test` already present in dev-dependencies.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Claude API returns valid annotations with real key | INFR-01 | Requires live API key + billing | Run `cargo run -- --llm-provider claude --paper-id 2503.18887 --db mem://` with `ANTHROPIC_API_KEY` set |
| Ollama produces annotations with local model | INFR-01 | Requires Ollama running locally | Start Ollama, `ollama pull llama3.2`, run `cargo run -- --llm-provider ollama --paper-id 2503.18887 --db mem://` |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
