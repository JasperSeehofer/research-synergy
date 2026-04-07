---
phase: 21
slug: search-filter
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-07
---

# Phase 21 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo test --lib` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 21-01-01 | 01 | 1 | SRCH-01 | — | N/A | integration | `cargo test search` | ❌ W0 | ⬜ pending |
| 21-01-02 | 01 | 1 | SRCH-04 | — | N/A | integration | `cargo test search` | ❌ W0 | ⬜ pending |
| 21-02-01 | 02 | 2 | SRCH-02 | — | N/A | manual | Visual graph pan check | N/A | ⬜ pending |
| 21-02-02 | 02 | 2 | SRCH-03 | — | N/A | integration | `cargo test papers_filter` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] SurrealDB full-text search index integration test stubs
- [ ] `search_papers` server fn unit test stubs
- [ ] Existing test infrastructure covers framework needs

*If none: "Existing infrastructure covers all phase requirements."*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Graph pan + pulse glow on search result click | SRCH-02 | Canvas rendering requires visual inspection | 1. Search for a known paper 2. Click result on graph page 3. Verify viewport pans smoothly to node 4. Verify pulse glow ring appears (2-3 pulses) |
| Ctrl+K shortcut focuses search bar | SRCH-01 | Keyboard shortcut requires browser testing | 1. Navigate to any page 2. Press Ctrl+K / Cmd+K 3. Verify search bar receives focus |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
