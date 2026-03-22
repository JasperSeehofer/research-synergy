---
phase: 7
slug: incremental-crawl-infrastructure
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-15
---

# Phase 7 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness + `#[tokio::test]` |
| **Config file** | none (cargo-native) |
| **Quick run command** | `cargo test crawl_queue -- --nocapture` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test crawl_queue -- --nocapture`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 7-01-01 | 01 | 1 | CRAWL-01 | unit | `cargo test test_queue_enqueue_dedup -- --nocapture` | ❌ W0 | ⬜ pending |
| 7-01-02 | 01 | 1 | CRAWL-01 | unit | `cargo test test_queue_claim -- --nocapture` | ❌ W0 | ⬜ pending |
| 7-01-03 | 01 | 1 | CRAWL-01 | unit | `cargo test test_queue_claim_empty -- --nocapture` | ❌ W0 | ⬜ pending |
| 7-01-04 | 01 | 1 | CRAWL-02 | unit | `cargo test test_queue_reset_stale -- --nocapture` | ❌ W0 | ⬜ pending |
| 7-02-01 | 02 | 1 | CRAWL-02 | integration | `cargo test test_crawl_resume -- --nocapture` | ❌ W0 | ⬜ pending |
| 7-03-01 | 03 | 2 | CRAWL-03 | unit | `cargo test test_progress_event -- --nocapture` | ❌ W0 | ⬜ pending |
| 7-04-01 | 04 | 2 | CRAWL-04 | unit | `cargo test test_parallel_concurrency -- --nocapture` | ❌ W0 | ⬜ pending |
| 7-04-02 | 04 | 2 | CRAWL-04 | unit | `cargo test test_rate_limiter_blocks -- --nocapture` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `resyn-core/src/database/crawl_queue.rs` — stubs for CRAWL-01, CRAWL-02
- [ ] `resyn-core/src/data_aggregation/rate_limiter.rs` — stubs for CRAWL-04
- [ ] `governor = "0.10"` added to resyn-core Cargo.toml
- [ ] `axum = "0.8"` added to resyn-server Cargo.toml

*Existing test infrastructure (cargo test, wiremock, connect_memory) covers framework needs.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| SSE stream readable via curl | CRAWL-03 | Requires running server + curl client | Start crawl with `--progress`, run `curl -N localhost:3001/progress`, verify JSON events appear |
| Parallel crawl faster than sequential | CRAWL-04 | Timing comparison across runs | Run same crawl with and without `--parallel`, compare elapsed times |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
