# Phase 23: Graph Analytics — Centrality & Metrics - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-09
**Phase:** 23-graph-analytics-centrality-metrics
**Areas discussed:** Node sizing controls, Influential papers panel, Computation & caching strategy, N+1 query optimization

---

## Node sizing controls

| Option | Description | Selected |
|--------|-------------|----------|
| Graph controls overlay | Add to existing controls overlay alongside edge toggles, force mode, and label mode | ✓ |
| Separate floating panel | Distinct metrics panel with top-5 values | |
| Sidebar settings section | Under expandable in the sidebar | |

**User's choice:** Graph controls overlay
**Notes:** Keeps all graph settings in one place, consistent with existing pattern.

| Option | Description | Selected |
|--------|-------------|----------|
| Animated lerp (~300ms) | Smooth size interpolation, consistent with viewport_fit lerp | ✓ |
| Instant snap | Immediate resize, simpler | |
| You decide | Claude picks | |

**User's choice:** Animated lerp
**Notes:** Consistent with existing animation patterns in the codebase.

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, show score | Tooltip shows active metric's value on hover | ✓ |
| No, size is enough | Keep tooltips as-is | |

**User's choice:** Yes, show raw metric score on hover

---

## Influential papers panel

| Option | Description | Selected |
|--------|-------------|----------|
| New dashboard card | 6th card showing top-5 by PageRank, with "View all →" link | ✓ |
| Dedicated page/panel | New '/analytics' page with full sortable ranking | |
| Both | Dashboard card plus full ranking page | |

**User's choice:** New dashboard card
**Notes:** Consistent with existing 5-card dashboard layout.

| Option | Description | Selected |
|--------|-------------|----------|
| Score + title + year | Compact, consistent with other cards | ✓ |
| Score + title + authors + year | Richer but more space | |
| You decide | Claude picks | |

**User's choice:** Score + title + year

| Option | Description | Selected |
|--------|-------------|----------|
| Top 5 | Fits card well, full list elsewhere | ✓ |
| Top 10 | More comprehensive but taller card | |

**User's choice:** Top 5

---

## Computation & caching strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Background after crawl | Auto-recompute, silent, same as similarity | |
| On-demand button | User clicks to trigger | |
| Both | Auto-compute plus manual "Recompute" button | ✓ |

**User's choice:** Both — auto-compute after crawl plus manual recompute button
**Notes:** Dual trigger model gives both convenience and user control.

| Option | Description | Selected |
|--------|-------------|----------|
| Silent | No progress indicator | |
| Subtle indicator | Small spinner/badge on Size by dropdown | ✓ |
| You decide | Claude picks | |

**User's choice:** Subtle indicator

| Option | Description | Selected |
|--------|-------------|----------|
| Disable metric options | Grayed out with "Computing..." label | ✓ |
| Hide metric options | Options appear dynamically | |
| You decide | Claude picks | |

**User's choice:** Disable metric options (grayed out, not hidden)

---

## N+1 query optimization

| Option | Description | Selected |
|--------|-------------|----------|
| Just get_cited/get_citing | Replace N+1 loops with JOINs, minimal scope | |
| All citation query paths | Also refactor get_citation_graph BFS | |
| You decide | Claude assesses which queries cause issues | ✓ |

**User's choice:** You decide — Claude assesses performance impact and refactors accordingly

| Option | Description | Selected |
|--------|-------------|----------|
| Replace in-place | Same signatures, better implementation | ✓ |
| New functions alongside | Keep old, add _optimized variants | |

**User's choice:** Replace in-place

---

## Claude's Discretion

- PageRank score formatting (percentage vs decimal)
- Recompute button placement and styling
- Spinner/badge design for computing indicator
- Scope of N+1 refactor beyond get_cited/get_citing
- PageRank convergence parameters
- Betweenness centrality algorithm variant

## Deferred Ideas

None — discussion stayed within phase scope
