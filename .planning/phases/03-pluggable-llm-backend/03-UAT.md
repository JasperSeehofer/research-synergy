---
status: complete
phase: 03-pluggable-llm-backend
source: [03-01-SUMMARY.md, 03-02-SUMMARY.md, 03-03-SUMMARY.md]
started: 2026-03-14T13:00:00Z
updated: 2026-03-14T13:15:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Cold Start Smoke Test
expected: Build from clean state and run with noop provider + local DB + --analyze flag. Application boots, crawls seed paper, persists to DB, runs NLP and LLM analysis steps without errors. `cargo run -- --db surrealkv://./test_data --analyze --llm-provider noop --max-depth 1 --paper-id 2503.18887` completes successfully with LLM summary output visible in logs.
result: pass

### 2. CLI --llm-provider Flag Accepted
expected: Running `cargo run -- --help` shows `--llm-provider` flag with accepted values (claude, ollama, noop) and `--llm-model` flag for model override.
result: pass

### 3. Unknown Provider Rejected
expected: Running `cargo run -- --db surrealkv://./test_data --analyze --llm-provider bogus --max-depth 1` exits with a non-zero exit code and an error message about unknown provider.
result: pass

### 4. Noop Provider Produces Annotations
expected: After running with `--analyze --llm-provider noop` and a DB, the noop provider produces LlmAnnotation records (paper_type "unknown", empty methods/findings/open_problems) for each paper. End-of-run summary log line shows annotated count > 0.
result: pass

### 5. Per-Paper Caching (Skip Already-Annotated)
expected: Run the same command twice with `--analyze --llm-provider noop` and same DB. Second run's LLM summary should show "cached" count equal to total papers (all cached from first run), "annotated" count = 0. No redundant API calls.
result: pass

### 6. End-of-Run LLM Summary
expected: After LLM analysis completes, a structured log summary appears showing annotated/skipped/failed/total/provider fields, matching the NLP analysis summary style.
result: pass

## Summary

total: 6
passed: 6
issues: 0
pending: 0
skipped: 0

## Gaps

[none yet]
