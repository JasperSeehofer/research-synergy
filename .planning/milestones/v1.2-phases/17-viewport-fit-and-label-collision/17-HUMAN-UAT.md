---
status: partial
phase: 17-viewport-fit-and-label-collision
source: [17-VERIFICATION.md]
started: 2026-03-26T00:00:00Z
updated: 2026-03-26T00:00:00Z
---

## Current Test

[awaiting human testing]

## Tests

### 1. Auto-Fit Animation on Convergence
expected: Viewport smoothly animates (~0.5s) to frame all nodes with ~10% margin on each side. Status badge changes from "Simulating..." to "Settled" in green.
result: [pending]

### 2. User Interaction Latch
expected: After panning/scrolling before convergence, auto-fit does NOT fire. Badge still transitions to "Settled" but viewport stays where user left it.
result: [pending]

### 3. Fit Button Manual Trigger
expected: Pressing Fit button (U+2922) smoothly animates viewport to frame all visible nodes, regardless of user_has_interacted state.
result: [pending]

### 4. Label Collision Avoidance at Medium Zoom
expected: Labels appear as pill badges. No overlaps. Seed paper always labeled. Hidden nodes (LOD-culled) have no labels.
result: [pending]

### 5. Hover Label Override
expected: Hovering over a node whose label was culled by collision avoidance reveals its label as a pill badge.
result: [pending]

## Summary

total: 5
passed: 0
issues: 0
pending: 5
skipped: 0
blocked: 0

## Gaps
