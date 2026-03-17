---
phase: 8
slug: leptos-web-shell-analysis-panels
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-17
---

# Phase 8 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | none — inline `#[test]` and `#[cfg(test)]` |
| **Quick run command** | `cargo check -p resyn-app --target wasm32-unknown-unknown && cargo check -p resyn-server` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo check -p resyn-app --target wasm32-unknown-unknown && cargo check -p resyn-server`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 08-01-01 | 01 | 1 | WEB-03 | compile | `cargo build -p resyn-app --target wasm32-unknown-unknown` | ❌ W0 | ⬜ pending |
| 08-02-01 | 02 | 1 | WEB-04 | integration | `cargo test -p resyn-server server_fn` | ❌ W0 | ⬜ pending |
| 08-03-01 | 03 | 2 | AUI-01 | unit | `cargo test -p resyn-core gap_finding_repository` | ❌ W0 | ⬜ pending |
| 08-04-01 | 04 | 2 | AUI-02 | unit | `cargo test -p resyn-core aggregate_open_problems` | ❌ W0 | ⬜ pending |
| 08-05-01 | 05 | 2 | AUI-03 | unit | `cargo test -p resyn-core build_method_matrix` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `resyn-core/src/datamodels/progress.rs` — move ProgressEvent from resyn-server to resyn-core (WASM-safe, no ssr gate)
- [ ] Unit test for `aggregate_open_problems()` aggregation logic — covers AUI-02
- [ ] Unit test for `build_method_matrix()` pair counting — covers AUI-03
- [ ] Verify existing `GapFindingRepository::get_all_gap_findings()` test coverage — covers AUI-01
- [ ] `cargo build -p resyn-app --target wasm32-unknown-unknown` must pass — covers WEB-03

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `trunk serve` renders paper list in browser | WEB-03 | Visual rendering requires browser | Run `trunk serve`, open localhost:8080, verify paper list table renders |
| Sidebar collapse/expand animation | WEB-03 | CSS transition is visual | Click sidebar collapse button, verify icon-only rail with tooltips |
| Method heatmap cell colors and drill-down | AUI-03 | Visual color rendering | Open Methods panel, verify blue gradient, click cell for sub-matrix |
| Crawl progress bar real-time update | WEB-04 | SSE live connection | Start crawl from UI, verify progress bar updates without page refresh |
| Dark theme visual consistency | WEB-03 | Visual styling | Check all panels for dark background, muted colors, consistent typography |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
