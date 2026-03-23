# /gsd-verify

Browser-based UI verification using agent-browser after GSD phase execution.
TRIGGER when: user runs /gsd-verify, or after /gsd:verify-work completes with UI-related changes.

## Instructions

Detect whether the completed phase touched UI files, and if so, launch agent-browser to visually verify the running application.

### Step 1: Detect UI changes

Determine the phase directory from $ARGUMENTS (phase number) or from the most recent phase in `.planning/phases/`.

Check git diff for the phase's commits to identify changed files. A phase has UI changes if any modified file matches:

- `resyn-app/src/**` (frontend components, pages, layout, graph rendering)
- `resyn-app/style/**` (CSS/stylesheets)
- `resyn-app/index.html`
- Any file under `components/`, `pages/`, `layout/`, `graph/` directories

If no UI changes are detected, report:

```
No UI changes detected in this phase. Skipping browser verification.
```

And exit.

### Step 2: Check services are running

Verify the app and server are accessible:

```bash
# Check if the server (port 3100) and app (port 8080) are running
curl -s -o /dev/null -w "%{http_code}" http://localhost:8080 2>/dev/null
curl -s -o /dev/null -w "%{http_code}" http://localhost:3100 2>/dev/null
```

If either is not responding, prompt the user:

```
Services not running. Start them before browser verification:

  Terminal 1: cd resyn-server && cargo run
  Terminal 2: cd resyn-app && trunk serve

Then re-run /gsd-verify
```

And exit.

### Step 3: Browser verification

Use `agent-browser` to verify the UI. Run commands via Bash tool.

**3a. Open the app and take a baseline screenshot:**

```bash
agent-browser open http://localhost:8080 && agent-browser wait --load networkidle && agent-browser screenshot --full --annotate /tmp/resyn-verify-full.png
```

Read the screenshot file to visually inspect it.

**3b. Take an accessibility snapshot to verify interactive elements:**

```bash
agent-browser snapshot -i
```

**3c. Run targeted checks based on what changed:**

For each category of UI change, run the relevant checks:

- **Graph/visualization changes** (`graph/`, `force_graph_app`, `layout/`):
  - Verify the graph canvas is rendered and visible
  - Check that nodes and edges appear (snapshot should show interactive graph elements)
  - Screenshot the graph area specifically

- **Component changes** (`components/`):
  - Navigate to pages that use the changed components
  - Verify components render without visual errors
  - Check interactive elements respond (click, hover)

- **Page/routing changes** (`pages/`):
  - Navigate to each affected page route
  - Verify page loads and renders content
  - Screenshot each page

- **Style changes** (`style/`):
  - Full-page screenshot to catch layout regressions
  - Compare element visibility and spacing

For each check:
```bash
agent-browser screenshot --annotate /tmp/resyn-verify-{check_name}.png
```

Read each screenshot to visually verify correctness.

### Step 4: Report results

Present a verification report:

```
## Browser Verification: Phase {N}

| Check | Status | Notes |
|-------|--------|-------|
| App loads | PASS/FAIL | ... |
| Graph renders | PASS/FAIL | ... |
| {Component} visible | PASS/FAIL | ... |

### Screenshots
[Describe what was observed in each screenshot]

### Issues Found
[List any visual regressions, broken layouts, missing elements]
```

If issues are found, write them to the phase's UAT file (if one exists) as additional gaps, or report them inline for the user to act on.

### Step 5: Cleanup

```bash
agent-browser close
```
