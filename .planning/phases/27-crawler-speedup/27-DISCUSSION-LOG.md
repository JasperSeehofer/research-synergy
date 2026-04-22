# Phase 27: Crawler Speedup - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-22
**Phase:** 27-crawler-speedup
**Areas discussed:** T3 scope, --mailto removal, API key enforcement, CLAUDE.md update scope

---

## T3 — arXiv id_list Batching

| Option | Description | Selected |
|--------|-------------|----------|
| Defer to later phase | Phase 27 primary goal is pre-ingest via OpenAlex; BFS crawler is a separate concern. Keep T3 out of scope. | ✓ |
| Include in Phase 27 | Add fetch_papers_batch() to arxiv_api.rs and wire into BFS queue. ~1 day of extra work. | |

**User's choice:** Defer to later phase
**Notes:** T3 provides 200× speedup for metadata fetches only (0× for reference scraping). Not the right phase for it.

---

## --mailto Removal

| Option | Description | Selected |
|--------|-------------|----------|
| Hard remove | Delete --mailto arg and DEFAULT_MAILTO constant entirely. Clean code. | ✓ |
| Deprecate with warning | Keep as hidden arg; print warning if used. | |

**User's choice:** Hard remove
**Notes:** Single-user CLI; no backwards compatibility concern. Clean break preferred.

---

## API Key Enforcement

| Option | Description | Selected |
|--------|-------------|----------|
| Hard fail with clear error | Exit immediately with actionable error message pointing to openalex.org/settings/api | ✓ |
| Warn and continue unauthenticated | Proceed but warn; fails silently at bulk scale | |
| Warn and continue, log 409s | Proceed; abort on 3 consecutive 409s | |

**User's choice:** Hard fail with clear error
**Notes:** Prevents the silent bulk-ingest failure mode where 409s drop papers after the first 100 pages.

---

## CLAUDE.md Update Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Full update | Fix wrong concept ID + update bulk-ingest example + note --mailto removal | ✓ |
| Concept IDs only | Just replace the wrong concept ID | |

**User's choice:** Full update
**Notes:** C2778407487 is Altmetrics, not Statistical Physics. Correct IDs are C26873012 (cond-mat) and C121864883 (stat-phys). Example command should show --api-key and physics filter.

---

## Claude's Discretion

- Exact error message wording for missing API key
- Whether to add/update a unit test for Authorization header injection

## Deferred Ideas

- arXiv id_list batching (T3) — future crawler optimization phase
