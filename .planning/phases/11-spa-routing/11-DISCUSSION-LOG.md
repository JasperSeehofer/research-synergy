# Phase 11: SPA Routing - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-23
**Phase:** 11-spa-routing
**Areas discussed:** SPA fallback strategy

---

## Discussion Skipped (Clear-Cut Fix)

| Option | Description | Selected |
|--------|-------------|----------|
| Create context now | Root cause is clear — ServeDir needs SPA fallback. Go straight to CONTEXT.md. | ✓ |
| Discuss first | Walk through gray areas even though the fix pattern is standard. | |

**User's choice:** Create context now (Recommended)
**Notes:** Phase is a well-understood SPA routing fix with no user-facing design decisions. Root cause identified during codebase scout: `ServeDir` fallback in `serve.rs` doesn't serve `index.html` for non-file routes.

---

## Claude's Discretion

- Choice of tower-http API for SPA fallback (ServeDir::fallback, ServeFile, or catch-all handler)
- Optional cache control headers for index.html vs static assets

## Deferred Ideas

None.
