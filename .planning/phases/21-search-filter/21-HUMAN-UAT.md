---
status: partial
phase: 21-search-filter
source: [21-VERIFICATION.md]
started: 2026-04-07
updated: 2026-04-07
---

## Current Test

[awaiting human testing]

## Tests

### 1. Global search bar renders on all pages
expected: Search bar visible in top bar (48px min-height) above content area on Dashboard, Papers, Graph, Gaps, Problems, Methods pages
result: [pending]

### 2. Ctrl+K focuses search input
expected: Pressing Ctrl+K (or Cmd+K on Mac) from any page moves focus to the search input
result: [pending]

### 3. Search results dropdown appears after 300ms debounce
expected: Typing 'quantum' shows a dropdown of ranked results with title, authors, year; 'Searching...' shown during fetch
result: [pending]

### 4. Graph viewport pans to search result node
expected: Selecting a result on the graph page triggers smooth lerp pan to center the matched node
result: [pending]

### 5. Pulse glow ring after pan
expected: Matched node shows blue (#58a6ff) ring that pulses 2-3 times over ~2s then fades
result: [pending]

### 6. Papers table filter
expected: Table rows filter in real time (300ms debounce); matching text in title and author columns is bold with blue accent color
result: [pending]

## Summary

total: 6
passed: 0
issues: 0
pending: 6
skipped: 0
blocked: 0

## Gaps
