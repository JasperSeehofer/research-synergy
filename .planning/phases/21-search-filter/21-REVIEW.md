---
phase: 21-search-filter
reviewed: 2026-04-07T14:30:00Z
depth: standard
files_reviewed: 13
files_reviewed_list:
  - resyn-app/Cargo.toml
  - resyn-app/src/app.rs
  - resyn-app/src/components/mod.rs
  - resyn-app/src/components/search_bar.rs
  - resyn-app/src/graph/canvas_renderer.rs
  - resyn-app/src/graph/layout_state.rs
  - resyn-app/src/graph/viewport_fit.rs
  - resyn-app/src/pages/graph.rs
  - resyn-app/src/pages/papers.rs
  - resyn-app/src/server_fns/papers.rs
  - resyn-app/style/main.css
  - resyn-core/src/database/queries.rs
  - resyn-core/src/database/schema.rs
findings:
  critical: 0
  warning: 4
  info: 3
  total: 7
status: issues_found
---

# Phase 21: Code Review Report

**Reviewed:** 2026-04-07T14:30:00Z
**Depth:** standard
**Files Reviewed:** 13
**Status:** issues_found

## Summary

Reviewed 13 files comprising the Phase 21 search and filter implementation: global search bar with BM25 full-text search, papers table inline filter with match highlighting, graph viewport pan-to-node on search result selection, and supporting database schema migration (FTS indexes). The implementation is well-structured with proper debouncing, keyboard navigation, and clean separation of concerns.

Four warnings were identified: a Unicode-safety bug in highlight text slicing, an N+1 query pattern in the paper detail endpoint, a leaked event listener closure, and an inefficient annotation lookup. Three informational items note minor improvements.

## Warnings

### WR-01: Unicode-unsafe byte slicing in highlight_text

**File:** `resyn-app/src/pages/papers.rs:55-58`
**Issue:** The `highlight_text` function finds the match position using `lower_text.find(&lower_query)` where both are produced by `to_lowercase()`. It then slices the original `text` at `start..end` where `end = start + query.len()`. If `to_lowercase()` changes the byte length of any character (e.g., German Eszett: `\u{00DF}` lowercases to `ss`, or Turkish dotted I), the byte offset from the lowered string will not correspond to the correct position in the original string. This will either produce incorrect highlight boundaries or panic on a non-UTF-8 boundary.
**Fix:** Use `char_indices` iteration or a Unicode-aware approach to map positions between the original and lowered strings. A simpler alternative for ASCII-safe academic text:
```rust
fn highlight_text(text: &str, query: &str) -> impl IntoView {
    if query.is_empty() {
        return view! { <span>{text.to_string()}</span> }.into_any();
    }
    // Find match in original text using case-insensitive char-by-char comparison
    let lower_text = text.to_lowercase();
    let lower_query = query.to_lowercase();
    if let Some(start) = lower_text.find(&lower_query) {
        // Map byte offset from lowered text back to original text using char counts
        let char_start = lower_text[..start].chars().count();
        let char_len = lower_query.chars().count();
        let byte_start = text.char_indices().nth(char_start).map(|(i, _)| i).unwrap_or(text.len());
        let byte_end = text.char_indices().nth(char_start + char_len).map(|(i, _)| i).unwrap_or(text.len());
        let before = text[..byte_start].to_string();
        let matched = text[byte_start..byte_end].to_string();
        let after = text[byte_end..].to_string();
        // ... rest unchanged
    }
    // ...
}
```

### WR-02: Inefficient N+1 annotation lookup in get_paper_detail

**File:** `resyn-app/src/server_fns/papers.rs:56-61`
**Issue:** `get_paper_detail` calls `get_all_annotations()` to fetch every annotation in the database, then filters client-side with `.find(|a| a.arxiv_id == id)`. The `LlmAnnotationRepository` already has a `get_annotation(&id)` method that does a direct record lookup by ID. With a growing database, fetching all annotations for a single paper detail view is wasteful and will degrade response times.
**Fix:**
```rust
let annotation = LlmAnnotationRepository::new(&db)
    .get_annotation(&id)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;
```

### WR-03: Leaked Closure for Ctrl+K global shortcut (no cleanup)

**File:** `resyn-app/src/components/search_bar.rs:69-71`
**Issue:** The `keydown` event listener closure is created inside an `Effect::new` and intentionally leaked with `cb.forget()`. The comment says "Intentional: lives for app lifetime", which is acceptable for a truly global singleton. However, `Effect::new` may re-run (e.g., on component re-mount during navigation), registering duplicate event listeners each time. Each re-run adds another listener that can never be removed.
**Fix:** Guard against duplicate registration. Either use a static `AtomicBool` to track if the listener was already installed, or move the listener registration to the `App` component (which mounts once) instead of `GlobalSearchBar`. Alternatively, use `on_cleanup` to remove the listener:
```rust
Effect::new(move |_| {
    let doc = web_sys::window().unwrap().document().unwrap();
    let input_ref_clone = input_ref.clone();
    let cb = Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(/* ... */);
    doc.add_event_listener_with_callback("keydown", cb.as_ref().unchecked_ref()).unwrap();
    let func = cb.as_ref().unchecked_ref::<js_sys::Function>().clone();
    std::mem::forget(cb); // prevent Drop
    on_cleanup(move || {
        let doc = web_sys::window().unwrap().document().unwrap();
        let _ = doc.remove_event_listener_with_callback("keydown", &func);
    });
});
```

### WR-04: Papers filter fetches all papers then filters client-side

**File:** `resyn-app/src/pages/papers.rs:96-109`
**Issue:** When the filter query is non-empty, the resource calls `search_papers(q, Some(100))` to get matching IDs, then calls `get_papers()` to fetch ALL papers, then filters the full list by matching IDs. This means every filtered search loads the entire paper table from the server. This works for small datasets but is an unnecessary double-fetch that will scale poorly.
**Fix:** Either return full `Paper` objects from the search server function directly, or add a server function that accepts a list of IDs and returns only those papers. The simplest fix is to extend `search_papers` to return `Paper` objects directly:
```rust
// In the resource closure:
if q.is_empty() {
    get_papers().await
} else {
    // Use a dedicated server fn that returns Paper objects for search results
    search_papers_full(q.clone(), Some(100)).await
}
```

## Info

### IN-01: Leaked ResizeObserver and closure in graph page

**File:** `resyn-app/src/pages/graph.rs:449-451`
**Issue:** `std::mem::forget(cb)` and `std::mem::forget(observer)` are used to keep the ResizeObserver alive. While functional, these leak memory on page navigation. The `on_cleanup` pattern used for the RAF handle could also be applied here.
**Fix:** Store the observer in the `RenderState` or a dedicated cleanup struct and disconnect in `on_cleanup`.

### IN-02: frame_counter wrapping behavior

**File:** `resyn-app/src/graph/layout_state.rs:74`
**Issue:** `frame_counter` is a `u32` that uses `wrapping_add(1)`. After ~71.5 million seconds (~2.27 years at 60fps), it wraps. The pulse timing logic in `canvas_renderer.rs:295` uses `saturating_sub` which would produce incorrect results on wrap, causing a stuck or missing pulse animation. This is extremely unlikely to occur in practice.
**Fix:** No action needed -- this is a theoretical concern. If desired, reset `frame_counter` to 0 when no pulse animation is active.

### IN-03: Search result score field sent but unused on client

**File:** `resyn-app/src/server_fns/papers.rs:142` and `resyn-app/src/components/search_bar.rs:42`
**Issue:** The `SearchResult` struct includes a `score: f32` field that is populated by the server and deserialized on the client, but never used in the UI. The results are already sorted by score server-side.
**Fix:** Consider removing the `score` field from `SearchResult` to reduce payload size, or display it as a relevance indicator in the dropdown.

---

_Reviewed: 2026-04-07T14:30:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
