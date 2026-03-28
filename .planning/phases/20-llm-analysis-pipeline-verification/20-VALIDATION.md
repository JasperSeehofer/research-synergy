---
phase: 20
slug: llm-analysis-pipeline-verification
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-28
---

# Phase 20 — Validation Strategy

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
| 20-01-01 | 01 | 1 | LLM-01 | unit | `cargo test start_analysis` | ❌ W0 | ⬜ pending |
| 20-01-02 | 01 | 1 | LLM-01 | integration | `cargo test analysis_pipeline_noop` | ❌ W0 | ⬜ pending |
| 20-01-03 | 01 | 1 | LLM-01 | unit | `cargo test analysis_sse_events` | ❌ W0 | ⬜ pending |
| 20-02-01 | 02 | 2 | LLM-02 | integration | `cargo test get_gap_findings` | ❌ W0 | ⬜ pending |
| 20-02-02 | 02 | 2 | LLM-02 | integration | `cargo test analysis_wiremock_ollama` | ❌ W0 | ⬜ pending |
| 20-03-01 | 03 | 2 | LLM-03 | integration | `cargo test get_open_problems_ranked_after_analysis` | ❌ W0 | ⬜ pending |
| 20-04-01 | 04 | 2 | LLM-04 | integration | `cargo test get_method_matrix_after_analysis` | ❌ W0 | ⬜ pending |
| 20-05-01 | 01 | 1 | LLM-01 | unit | `cargo test analysis_noop_nlp_only` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `resyn-app/src/server_fns/analysis.rs` — `StartAnalysis` server function stub (LLM-01)
- [ ] `resyn-server/tests/analysis_pipeline_test.rs` — integration test stubs for pipeline with wiremock (LLM-01 through LLM-04)
- [ ] Analysis stage `event_type` strings documented in progress event types

*All test files are new — no existing infrastructure covers analysis pipeline tests.*

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
