---
phase: 10
slug: analysis-ui-polish-scale
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-18
---

# Phase 10 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust) + wasm-pack test (WASM) |
| **Config file** | Cargo.toml (workspace) |
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
| 10-01-01 | 01 | 1 | DEBT-04 | unit | `cargo test -p resyn-core llm::prompt` | ❌ W0 | ⬜ pending |
| 10-01-02 | 01 | 1 | AUI-04 | unit | `cargo test -p resyn-core datamodels::llm_annotation` | ✅ | ⬜ pending |
| 10-02-01 | 02 | 1 | AUI-04 | integration | `cargo test -p resyn-app server_fns` | ❌ W0 | ⬜ pending |
| 10-03-01 | 03 | 2 | SCALE-02 | unit | `cargo test -p resyn-app graph::layout_state` | ❌ W0 | ⬜ pending |
| 10-03-02 | 03 | 2 | SCALE-03 | unit | `cargo test -p resyn-app graph::layout_state` | ❌ W0 | ⬜ pending |
| 10-04-01 | 04 | 3 | SCALE-01 | integration | `cargo test --workspace -- scale` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `resyn-core/src/llm/prompt_tests.rs` — stubs for section-aware prompt (DEBT-04)
- [ ] `resyn-core/src/datamodels/llm_annotation_tests.rs` — serde round-trip with source_section/source_snippet (AUI-04)
- [ ] `resyn-app/src/graph/layout_state_tests.rs` — LOD visibility + temporal filter (SCALE-02, SCALE-03)

*Existing infrastructure covers test framework — no new deps needed.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Gap finding click opens drawer with highlighted source text | AUI-04 | Browser interaction, visual highlighting | Click gap finding → verify drawer opens with Source tab, snippet highlighted |
| LOD progressive reveal on zoom | SCALE-02 | Visual WebGL rendering behavior | Zoom in/out on 1000+ node graph → verify nodes appear/disappear smoothly |
| Temporal slider dims out-of-range papers | SCALE-03 | Visual CSS/WebGL opacity behavior | Drag slider handles → verify papers outside range dim to ~10% opacity |
| Node count indicator updates | SCALE-02 | Visual UI element | Zoom in/out → verify "Showing X of Y nodes" updates |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
