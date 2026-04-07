use leptos::prelude::*;
use leptos_router::hooks::use_location;
use leptos_use::signal_debounced;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use crate::app::{DrawerOpenRequest, SearchPanRequest, SearchPanTrigger, SelectedPaper};
use crate::server_fns::papers::{search_papers, SearchResult};

/// Global search bar rendered in the app shell top bar.
///
/// Features:
/// - Ctrl+K / Cmd+K global shortcut focuses the input from any page
/// - 300ms debounced server fn calls to avoid flooding
/// - Dropdown showing up to 10 ranked BM25 results
/// - Keyboard navigation (ArrowUp/ArrowDown/Enter/Escape)
/// - Result selection opens the paper drawer and sets SearchPanTrigger (if on /graph)
/// - Empty-string guard: no server call when query is blank
#[component]
pub fn GlobalSearchBar() -> impl IntoView {
    let input_ref = NodeRef::<leptos::html::Input>::new();

    // Raw query updated on every keystroke
    let query: RwSignal<String> = RwSignal::new(String::new());
    // Debounced version — triggers server fetch after 300ms of inactivity
    let debounced_query: Signal<String> = signal_debounced(query, 300.0);

    // Whether the input is currently focused (drives outline + kbd hint visibility)
    let focused: RwSignal<bool> = RwSignal::new(false);
    // Whether the dropdown is open
    let dropdown_open: RwSignal<bool> = RwSignal::new(false);
    // Keyboard-navigation index into results (None = nothing highlighted)
    let focused_idx: RwSignal<Option<usize>> = RwSignal::new(None);

    // Server resource: re-fetches whenever debounced_query changes
    let results: Resource<Vec<SearchResult>> = Resource::new(
        move || debounced_query.get(),
        |q| async move {
            if q.trim().is_empty() {
                return vec![];
            }
            search_papers(q, None).await.unwrap_or_default()
        },
    );

    // Context signals provided by App
    let selected_paper =
        use_context::<SelectedPaper>().expect("SelectedPaper context must be provided");
    let search_pan_trigger =
        use_context::<SearchPanTrigger>().expect("SearchPanTrigger context must be provided");

    // Router location — used to decide whether to set SearchPanTrigger
    let location = use_location();

    // ── Ctrl+K / Cmd+K global shortcut ────────────────────────────────────────
    Effect::new(move |_| {
        let doc = web_sys::window().unwrap().document().unwrap();
        let input_ref_clone = input_ref.clone();
        let cb = Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(
            move |e: web_sys::KeyboardEvent| {
                if (e.ctrl_key() || e.meta_key()) && e.key() == "k" {
                    e.prevent_default();
                    if let Some(el) = input_ref_clone.get() {
                        let _ = el.focus();
                    }
                }
            },
        );
        doc.add_event_listener_with_callback("keydown", cb.as_ref().unchecked_ref())
            .unwrap();
        cb.forget(); // Intentional: lives for app lifetime
    });

    // ── Result selection handler ───────────────────────────────────────────────
    let select_result = move |arxiv_id: String| {
        // Always open the drawer
        selected_paper.0.set(Some(DrawerOpenRequest {
            paper_id: arxiv_id.clone(),
            ..Default::default()
        }));

        // Only trigger graph pan if we're on the graph page (D-08)
        let path = location.pathname.get();
        if path == "/graph" {
            search_pan_trigger
                .0
                .set(Some(SearchPanRequest { paper_id: arxiv_id }));
        }

        // Close dropdown and clear query
        dropdown_open.set(false);
        focused_idx.set(None);
        query.set(String::new());
    };

    // ── Keyboard navigation handler ────────────────────────────────────────────
    let on_keydown = move |e: web_sys::KeyboardEvent| {
        let key = e.key();
        match key.as_str() {
            "ArrowDown" => {
                e.prevent_default();
                let current_results = results.get().unwrap_or_default();
                let len = current_results.len();
                if len == 0 {
                    return;
                }
                focused_idx.update(|idx| {
                    *idx = Some(match *idx {
                        None => 0,
                        Some(i) => (i + 1).min(len - 1),
                    });
                });
            }
            "ArrowUp" => {
                e.prevent_default();
                focused_idx.update(|idx| {
                    *idx = match *idx {
                        None | Some(0) => None,
                        Some(i) => Some(i - 1),
                    };
                });
            }
            "Enter" => {
                e.prevent_default();
                let current_results = results.get().unwrap_or_default();
                if let Some(idx) = focused_idx.get() {
                    if let Some(result) = current_results.get(idx) {
                        select_result(result.arxiv_id.clone());
                    }
                } else if let Some(first) = current_results.into_iter().next() {
                    select_result(first.arxiv_id);
                }
            }
            "Escape" => {
                dropdown_open.set(false);
                focused_idx.set(None);
                query.set(String::new());
                if let Some(el) = input_ref.get() {
                    let _ = el.blur();
                }
            }
            _ => {}
        }
    };

    view! {
        <div class="global-search-wrapper">
            <div
                class="global-search-bar"
                class:search-focused=move || focused.get()
            >
                <span class="search-icon">"🔍"</span>
                <input
                    type="text"
                    placeholder="Search papers..."
                    node_ref=input_ref
                    prop:value=move || query.get()
                    on:input=move |e| {
                        let val = event_target_value(&e);
                        query.set(val);
                        focused_idx.set(None);
                        dropdown_open.set(true);
                    }
                    on:focus=move |_| {
                        focused.set(true);
                        if !query.get().is_empty() {
                            dropdown_open.set(true);
                        }
                    }
                    on:blur=move |_| {
                        // Delay so mousedown on a result row fires before we close
                        focused.set(false);
                        set_timeout(
                            move || {
                                dropdown_open.set(false);
                                focused_idx.set(None);
                            },
                            std::time::Duration::from_millis(200),
                        );
                    }
                    on:keydown=on_keydown
                />
                <Show when=move || !focused.get()>
                    <kbd class="search-kbd-hint">"Ctrl+K"</kbd>
                </Show>
            </div>

            <Show when=move || dropdown_open.get() && !query.get().is_empty()>
                <div class="search-dropdown">
                    <Suspense fallback=|| view! {
                        <div class="search-empty-state">"Searching..."</div>
                    }>
                        {move || {
                            let res = results.get().unwrap_or_default();
                            if res.is_empty() {
                                view! {
                                    <div>
                                        <div class="search-empty-state">"No papers found"</div>
                                        <div class="search-empty-hint" style="padding: 0 var(--space-md) var(--space-md)">"Try a different title, author, or keyword"</div>
                                    </div>
                                }.into_any()
                            } else {
                                let items = res
                                    .into_iter()
                                    .enumerate()
                                    .map(|(i, result)| {
                                        let arxiv_id = result.arxiv_id.clone();
                                        let title = result.title.clone();
                                        let authors_year = {
                                            let author_str = result.authors.first()
                                                .cloned()
                                                .unwrap_or_default();
                                            let year = result.year.clone();
                                            if author_str.is_empty() {
                                                year
                                            } else {
                                                format!("{} · {}", author_str, year)
                                            }
                                        };
                                        let select_result = select_result.clone();
                                        view! {
                                            <div
                                                class="search-result-row"
                                                class:search-result-focused=move || focused_idx.get() == Some(i)
                                                on:mousedown=move |e| {
                                                    e.prevent_default();
                                                    select_result(arxiv_id.clone());
                                                }
                                            >
                                                <div class="search-result-title">{title}</div>
                                                <div class="search-result-meta">{authors_year}</div>
                                            </div>
                                        }
                                    })
                                    .collect::<Vec<_>>();
                                view! { <div>{items}</div> }.into_any()
                            }
                        }}
                    </Suspense>
                </div>
            </Show>
        </div>
    }
}
