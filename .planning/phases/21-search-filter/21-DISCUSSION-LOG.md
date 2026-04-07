# Phase 21: Search & Filter - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-07
**Phase:** 21-search-filter
**Areas discussed:** Search placement & UX, Graph search integration, Search backend strategy, Papers table filtering

---

## Search placement & UX

| Option | Description | Selected |
|--------|-------------|----------|
| Global top bar | Persistent search bar in app header, accessible from every page | ✓ |
| Per-page inline | Each page gets its own search input | |
| Command palette | Ctrl+K modal overlay | |

**User's choice:** Global top bar
**Notes:** None

| Option | Description | Selected |
|--------|-------------|----------|
| Dropdown list | Inline dropdown below search bar, top 8-10 matches | ✓ |
| Full results page | Dedicated results page | |
| Side panel | Results slide in from side | |

**User's choice:** Dropdown list
**Notes:** None

| Option | Description | Selected |
|--------|-------------|----------|
| Open detail drawer | Click opens paper detail drawer | |
| Navigate to papers page | Click goes to papers table | |
| Context-sensitive | Different action per page | ✓ |

**User's choice:** Context-sensitive, but additionally always open the detail drawer
**Notes:** User wanted both context-specific actions AND drawer opening on every result click

| Option | Description | Selected |
|--------|-------------|----------|
| Ctrl+K / Cmd+K | Standard modern shortcut | ✓ |
| / (slash) | GitHub-style | |
| No shortcut | Click only | |

**User's choice:** Ctrl+K / Cmd+K
**Notes:** None

---

## Graph search integration

| Option | Description | Selected |
|--------|-------------|----------|
| Smooth lerp pan+zoom | Lerp animation reusing viewport_fit pattern | ✓ |
| Instant jump | Snap viewport immediately | |
| You decide | Claude picks | |

**User's choice:** Smooth lerp pan+zoom
**Notes:** None

| Option | Description | Selected |
|--------|-------------|----------|
| Pulse glow + fade | 2-3 pulses over ~2s then fades | ✓ |
| Persistent highlight ring | Stays until user clicks elsewhere | |
| Color flash | Brief color change then fade | |

**User's choice:** Pulse glow + fade
**Notes:** None

| Option | Description | Selected |
|--------|-------------|----------|
| Pan to first, dim others | Viewport centers on top result, non-matches dimmed | ✓ |
| Highlight all matches | All matches highlighted, zoom out to fit | |
| Dropdown only | Graph only reacts on explicit click | |

**User's choice:** Pan to first, dim others — but graph must NOT pan while typing
**Notes:** User raised concern about chaotic panning during typing. Agreed: graph only pans on explicit result selection (click or Enter), not during live typing. Dropdown updates live with debounce.

| Option | Description | Selected |
|--------|-------------|----------|
| Pan on selection only | Graph stays still while typing, pans on click/Enter | ✓ |
| Pan on selection + dim while typing | Same + dim non-matches in real-time | |

**User's choice:** Pan on selection only
**Notes:** Cleaner approach — no viewport or visual changes until user commits

---

## Search backend strategy

| Option | Description | Selected |
|--------|-------------|----------|
| SurrealDB full-text search | DEFINE ANALYZER + search index, server fn query | ✓ |
| Client-side filter | Filter loaded papers in WASM | |
| Hybrid | SurrealDB for global, client-side for table | |

**User's choice:** SurrealDB full-text search
**Notes:** None

| Option | Description | Selected |
|--------|-------------|----------|
| Title + Abstract + Authors | Three fields per SRCH-01 | ✓ |
| Title + Authors only | Faster but misses abstract | |
| All text fields | Maximum recall, noisier | |

**User's choice:** Title + Abstract + Authors
**Notes:** None

| Option | Description | Selected |
|--------|-------------|----------|
| SurrealDB native scoring | search::score() BM25-based | ✓ |
| Custom weighted scoring | Title 3x, author 2x, abstract 1x | |
| You decide | Claude picks | |

**User's choice:** SurrealDB native scoring
**Notes:** None

---

## Papers table filtering

| Option | Description | Selected |
|--------|-------------|----------|
| Client-side filter | Filter already-loaded papers in WASM | |
| Server-side search | Same SurrealDB full-text search as global bar | ✓ |
| You decide | Claude picks | |

**User's choice:** Server-side search
**Notes:** User preferred consistency with global search over client-side simplicity

| Option | Description | Selected |
|--------|-------------|----------|
| Separate inline filter bar | Own search input above papers table | ✓ |
| Global bar drives table | Global search filters table when on papers page | |
| Both | Global dropdown + inline filter | |

**User's choice:** Separate inline filter bar
**Notes:** Independent from global search bar per SRCH-04

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, highlight matches | Bold/color matching text in cells | ✓ |
| No highlighting | Just filter rows | |

**User's choice:** Yes, highlight matches
**Notes:** None

---

## Claude's Discretion

- Debounce timing details
- Dropdown styling and positioning
- Pulse glow implementation approach
- Empty state messaging
- Search result count badge

## Deferred Ideas

None — discussion stayed within phase scope
