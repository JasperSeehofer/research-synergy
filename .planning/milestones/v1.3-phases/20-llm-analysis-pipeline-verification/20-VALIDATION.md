---
phase: 20
slug: llm-analysis-pipeline-verification
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-28
---

# Phase 20 -- Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in (`cargo test`) |
| **Config file** | Cargo.toml (workspace) |
| **Quick run command** | `cargo test` |
| **Full suite command** | `cargo test --all-targets` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test`
- **After every plan wave:** Run `cargo test --all-targets`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 20-04-01 | 04 | 3 | LLM-01 | integration | `cargo test test_analysis_pipeline_noop_nlp_only` | W0 stub | pending |
| 20-04-02 | 04 | 3 | LLM-01 | integration | `cargo test test_analysis_pipeline_noop_provider` | W0 stub | pending |
| 20-04-03 | 04 | 3 | LLM-01, LLM-02, LLM-03, LLM-04 | integration | `cargo test test_analysis_pipeline_wiremock_ollama` | W0 stub | pending |
| 20-04-04 | 04 | 3 | LLM-01 | integration | `cargo test test_analysis_pipeline_caching` | W0 stub | pending |
| 20-04-05 | 04 | 3 | LLM-01 | integration | `cargo test test_start_analysis_http` | W0 stub | pending |
| 20-04-06 | 04 | 3 | LLM-01 | integration | `cargo test --features ollama-test test_analysis_pipeline_real_ollama` | W0 stub (feature-gated) | pending |

*Status: pending / green / red / flaky*

---

## Wave 0 Requirements

- [ ] `resyn-app/src/server_fns/analysis.rs` -- `StartAnalysis` server function stub (LLM-01)
- [ ] `resyn-server/tests/analysis_pipeline_test.rs` -- 5 default + 1 feature-gated integration test stubs (LLM-01 through LLM-04)
- [ ] Analysis stage `event_type` strings documented in progress event types

*All test files are new -- no existing infrastructure covers analysis pipeline tests.*

---

## Wave 0 Stub-to-Implementation Name Mapping

Every Wave 0 stub in Plan 01 Task 0 has a 1:1 name match with Plan 04 implementations:

| Wave 0 Stub (Plan 01) | Implementation (Plan 04) | Decision |
|------------------------|--------------------------|----------|
| `test_analysis_pipeline_noop_nlp_only` | Plan 04 Test 1 | D-07 |
| `test_analysis_pipeline_noop_provider` | Plan 04 Test 2 | LLM-01 |
| `test_analysis_pipeline_wiremock_ollama` | Plan 04 Test 3 | D-11 |
| `test_analysis_pipeline_caching` | Plan 04 Test 4 | D-02 |
| `test_start_analysis_http` | Plan 04 Test 5 | D-10 |
| `test_analysis_pipeline_real_ollama` | Plan 04 Test 6 (feature-gated) | D-11 |

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Post-crawl "Run Analysis?" prompt appears | LLM-01 | UI interaction timing | After crawl completes, verify prompt appears inline on dashboard |
| Result panels auto-refresh on SSE signal | LLM-01 | Client-side SSE reactivity | Trigger analysis, watch panels update without page reload |
| NLP-only warning banner displays | LLM-01 | Visual UI element | Start server without RESYN_LLM_PROVIDER, verify warning banner |
| Method heatmap visual rendering | LLM-04 | Canvas/visual verification | Verify heatmap renders distinct colors for present vs absent combinations |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
