---
phase: 18
slug: arxiv-crawl-repair
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-28
---

# Phase 18 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | resyn-core/Cargo.toml |
| **Quick run command** | `cargo test -p resyn-core` |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p resyn-core`
- **After every plan wave:** Run `cargo test --workspace`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 18-01-01 | 01 | 1 | ARXIV-01 | unit | `cargo test -p resyn-core extract_arxiv_id_from_text` | ❌ W0 | ⬜ pending |
| 18-01-02 | 01 | 1 | ARXIV-01 | unit | `cargo test -p resyn-core get_arxiv_id_fallback` | ❌ W0 | ⬜ pending |
| 18-02-01 | 02 | 2 | ARXIV-03 | integration | `cargo test -p resyn-core arxiv_edge_density` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `resyn-core/tests/arxiv_html_parsing.rs` — integration test stubs for arXiv ID extraction from plain text
- [ ] Test fixture HTML files with plain-text arXiv citations (no hyperlinks)
- [ ] `regex` crate added to resyn-core/Cargo.toml

*Existing cargo test infrastructure covers framework needs.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Visual edge density comparison | ARXIV-03 | Requires running real crawl against live APIs | Run `cargo run --bin resyn -- --source arxiv --paper-id <id>` and `cargo run --bin resyn -- --source inspirehep --paper-id <id>`, compare graph edge counts |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
