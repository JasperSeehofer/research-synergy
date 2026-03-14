---
phase: 5
slug: visualization-enrichment
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-14
---

# Phase 5 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in (`cargo test`) |
| **Config file** | none (Cargo.toml test configuration) |
| **Quick run command** | `cargo test visualization` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test visualization`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 05-01-01 | 01 | 0 | VIS-01 | unit | `cargo test visualization::tests::test_paper_type_to_color` | ❌ W0 | ⬜ pending |
| 05-01-02 | 01 | 0 | VIS-01 | unit | `cargo test visualization::tests::test_finding_strength_radius` | ❌ W0 | ⬜ pending |
| 05-01-03 | 01 | 0 | VIS-01 | unit | `cargo test visualization::tests::test_unenriched_node_defaults` | ❌ W0 | ⬜ pending |
| 05-01-04 | 01 | 0 | VIS-02 | unit | `cargo test visualization::tests::test_settings_analysis_default` | ❌ W0 | ⬜ pending |
| 05-01-05 | 01 | 0 | VIS-02 | unit | `cargo test visualization::tests::test_enriched_view_empty_maps` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/visualization/enrichment.rs` (or inline `#[cfg(test)]` module) — test stubs for VIS-01 color/size logic
- [ ] `src/visualization/tests.rs` — test stubs for VIS-02 toggle and fallback behavior

*Existing `cargo test` infrastructure covers all framework needs.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Node colors render correctly in GUI | VIS-01 | egui rendering requires visual inspection | Run app with analyzed papers, verify muted color palette on nodes |
| Tooltip shows keywords + method on hover | VIS-02 | Hover interaction requires GUI | Hover over enriched nodes, verify tooltip content |
| Edge tinting by source node color | VIS-01 | Visual rendering | Run enriched view, verify edge colors match source node type |
| Toggle transition is instant (no animation) | VIS-02 | Visual behavior | Toggle enriched view checkbox, verify instant color/size snap |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
