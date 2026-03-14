---
phase: 6
slug: tech-debt-workspace-restructure
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-15
---

# Phase 6 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in (`cargo test`) — no external framework |
| **Config file** | none (Cargo.toml controls test targets) |
| **Quick run command** | `cargo test -p resyn-core --features ssr` |
| **Full suite command** | `cargo test -p resyn-core --features ssr && cargo test -p resyn-server && cargo build -p resyn-app --target wasm32-unknown-unknown` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p resyn-core --features ssr`
- **After every plan wave:** Run `cargo test -p resyn-core --features ssr && cargo test -p resyn-server && cargo build -p resyn-app --target wasm32-unknown-unknown`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 06-01-01 | 01 | 0 | WEB-01 | compile | `cargo build` (workspace manifests valid) | ❌ W0 | ⬜ pending |
| 06-01-02 | 01 | 0 | WEB-02 | compile | `cargo build -p resyn-app --target wasm32-unknown-unknown` | ❌ W0 | ⬜ pending |
| 06-02-01 | 02 | 1 | DEBT-01 | unit | `cargo test -p resyn-core --features ssr -- nlp` | ✅ (existing) | ⬜ pending |
| 06-02-02 | 02 | 1 | DEBT-02 | compile | `cargo build -p resyn-core --features ssr` | ✅ | ⬜ pending |
| 06-02-03 | 02 | 1 | DEBT-03 | manual | inspect TODO.md / stale checkboxes | ✅ | ⬜ pending |
| 06-03-01 | 03 | 1 | WEB-05 | compile+grep | `cargo build -p resyn-core` (no egui dep) | ✅ after deletion | ⬜ pending |
| 06-04-01 | 04 | 2 | WEB-01 | unit | `cargo test -p resyn-core --features ssr` (153 tests pass) | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `resyn-core/Cargo.toml` — workspace crate does not yet exist
- [ ] `resyn-app/Cargo.toml` + `resyn-app/src/lib.rs` — WASM stub crate does not yet exist
- [ ] `resyn-server/Cargo.toml` + `resyn-server/src/main.rs` — server crate does not yet exist
- [ ] Root `Cargo.toml` — must be converted from package manifest to workspace manifest
- [ ] `resyn-core/src/datamodels/enrichment.rs` — replaces visualization/enrichment.rs color logic

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Stale ROADMAP checkboxes removed | DEBT-03 | Content inspection | Grep for stale plan checkboxes in ROADMAP/TODO |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
