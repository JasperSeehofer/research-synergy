use leptos::prelude::*;
use leptos::web_sys;
use resyn_core::analysis::highlight::find_highlight_range;
use resyn_core::datamodels::extraction::{ExtractionMethod, TextExtractionResult};

use crate::app::{DrawerTab, SelectedPaper};
use crate::server_fns::papers::{PaperDetail, get_paper_detail};

/// Paper detail side drawer. Slides in from the right when a paper is selected.
///
/// Controlled by the `SelectedPaper` context signal. Setting to `None` closes the drawer.
#[component]
pub fn Drawer() -> impl IntoView {
    let SelectedPaper(selected) = expect_context::<SelectedPaper>();

    let drawer_class = move || {
        if selected.get().is_some() {
            "drawer open"
        } else {
            "drawer closed"
        }
    };

    let close = move |_: web_sys::MouseEvent| selected.set(None);

    view! {
        // Backdrop — click to close
        {move || selected.get().is_some().then(|| view! {
            <div
                class="drawer-backdrop"
                on:click=close
                aria-hidden="true"
            ></div>
        })}

        <aside
            class=drawer_class
            role="dialog"
            aria-modal="true"
            aria-label="Paper details"
        >
            {move || {
                if let Some(req) = selected.get() {
                    view! {
                        <DrawerContent
                            paper_id=req.paper_id.clone()
                            initial_tab=req.initial_tab.clone()
                            highlight_snippet=req.highlight_snippet.clone()
                            highlight_section=req.highlight_section.clone()
                            on_close=close
                        />
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }
            }}
        </aside>
    }
}

/// Content rendered inside the open drawer for a given paper_id.
#[component]
fn DrawerContent(
    paper_id: String,
    initial_tab: DrawerTab,
    highlight_snippet: Option<String>,
    highlight_section: Option<String>,
    #[prop(into)] on_close: Callback<web_sys::MouseEvent>,
) -> impl IntoView {
    let id = paper_id.clone();
    let detail_resource = Resource::new(move || id.clone(), get_paper_detail);

    // Tab state — initialized from the open request, resets when drawer is re-opened
    let active_tab = RwSignal::new(initial_tab);

    view! {
        <Suspense fallback=move || view! {
            <DrawerSkeleton paper_id=paper_id.clone() on_close=on_close/>
        }>
            {move || detail_resource.get().map(|result| match result {
                Err(e) => view! {
                    <div class="drawer-header">
                        <h2 class="drawer-title">"Error loading paper"</h2>
                        <button
                            class="drawer-close"
                            on:click=move |e| on_close.run(e)
                            aria-label="Close"
                        >
                            "✕"
                        </button>
                    </div>
                    <div class="drawer-body">
                        <div class="error-banner">
                            {format!("Failed to load paper details: {e}")}
                        </div>
                    </div>
                }.into_any(),
                Ok(detail) => view! {
                    <DrawerBody
                        detail=detail
                        active_tab=active_tab
                        highlight_snippet=highlight_snippet.clone()
                        highlight_section=highlight_section.clone()
                        on_close=on_close
                    />
                }.into_any(),
            })}
        </Suspense>
    }
}

/// Drawer body once paper detail is loaded.
#[component]
fn DrawerBody(
    detail: PaperDetail,
    active_tab: RwSignal<DrawerTab>,
    highlight_snippet: Option<String>,
    highlight_section: Option<String>,
    #[prop(into)] on_close: Callback<web_sys::MouseEvent>,
) -> impl IntoView {
    let paper = detail.paper;
    let annotation = detail.annotation;
    let extraction = detail.extraction;

    let year = if paper.published.len() >= 4 {
        paper.published[..4].to_string()
    } else {
        String::new()
    };
    let authors_str = paper.authors.join(", ");
    let meta = if year.is_empty() {
        authors_str.clone()
    } else if authors_str.is_empty() {
        year.clone()
    } else {
        format!("{authors_str} · {year}")
    };

    let methods: Vec<String> = annotation
        .as_ref()
        .map(|a| a.methods.iter().map(|m| m.name.clone()).collect())
        .unwrap_or_default();

    let findings: Vec<String> = annotation
        .as_ref()
        .map(|a| a.findings.iter().map(|f| f.text.clone()).collect())
        .unwrap_or_default();

    let open_problems: Vec<String> = annotation
        .as_ref()
        .map(|a| a.open_problems.clone())
        .unwrap_or_default();

    let arxiv_id = paper.id.clone();

    view! {
        <div class="drawer-header">
            <h2 class="drawer-title">{paper.title.clone()}</h2>
            <button
                class="drawer-close"
                on:click=move |e| on_close.run(e)
                aria-label="Close"
            >
                "✕"
            </button>
        </div>

        // Tab strip
        <div class="drawer-tab-strip">
            <button
                class=move || if active_tab.get() == DrawerTab::Overview {
                    "drawer-tab active"
                } else {
                    "drawer-tab"
                }
                on:click=move |_| active_tab.set(DrawerTab::Overview)
            >
                "Overview"
            </button>
            <button
                class=move || if active_tab.get() == DrawerTab::Source {
                    "drawer-tab active"
                } else {
                    "drawer-tab"
                }
                on:click=move |_| active_tab.set(DrawerTab::Source)
            >
                "Source"
            </button>
        </div>

        // Tab content
        {move || {
            if active_tab.get() == DrawerTab::Overview {
                view! {
                    <div class="drawer-body">
                        // Authors + year
                        <p class="drawer-meta">{meta.clone()}</p>

                        // Abstract
                        <section>
                            <h3 class="drawer-section-title">"Abstract"</h3>
                            <p class="drawer-abstract">{paper.summary.clone()}</p>
                        </section>

                        // Methods
                        <section>
                            <h3 class="drawer-section-title">"Methods"</h3>
                            {if methods.is_empty() {
                                view! {
                                    <p class="text-body text-muted">"No methods annotated."</p>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="tags-list">
                                        {methods.iter().map(|m| view! {
                                            <span class="tag">{m.clone()}</span>
                                        }).collect_view()}
                                    </div>
                                }.into_any()
                            }}
                        </section>

                        // Findings
                        <section>
                            <h3 class="drawer-section-title">"Findings"</h3>
                            {if findings.is_empty() {
                                view! {
                                    <p class="text-body text-muted">"No findings annotated."</p>
                                }.into_any()
                            } else {
                                view! {
                                    <ul style="list-style: none; display: flex; flex-direction: column; gap: var(--space-xs);">
                                        {findings.iter().map(|f| view! {
                                            <li class="text-body">{f.clone()}</li>
                                        }).collect_view()}
                                    </ul>
                                }.into_any()
                            }}
                        </section>

                        // Open problems
                        <section>
                            <h3 class="drawer-section-title">"Open Problems"</h3>
                            {if open_problems.is_empty() {
                                view! {
                                    <p class="text-body text-muted">"No open problems annotated."</p>
                                }.into_any()
                            } else {
                                view! {
                                    <ul style="list-style: none; display: flex; flex-direction: column; gap: var(--space-xs);">
                                        {open_problems.iter().map(|p| view! {
                                            <li class="text-body">{p.clone()}</li>
                                        }).collect_view()}
                                    </ul>
                                }.into_any()
                            }}
                        </section>

                        // arXiv ID reference
                        <p class="text-label text-muted">"arXiv: " {arxiv_id.clone()}</p>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="drawer-body">
                        <SourceTabBody
                            extraction=extraction.clone()
                            highlight_snippet=highlight_snippet.clone()
                            highlight_section=highlight_section.clone()
                        />
                    </div>
                }.into_any()
            }
        }}
    }
}

/// Source tab body — shows extraction sections with optional snippet highlighting.
#[component]
fn SourceTabBody(
    extraction: Option<TextExtractionResult>,
    highlight_snippet: Option<String>,
    highlight_section: Option<String>,
) -> impl IntoView {
    match extraction {
        None => view! {
            <div class="source-empty-state">
                <p class="text-body text-muted">"Not yet analyzed"</p>
                <p class="text-label text-muted">"Run analysis on this paper to see source text."</p>
            </div>
        }.into_any(),
        Some(extraction) => {
            let is_abstract_only = extraction.extraction_method == ExtractionMethod::AbstractOnly;

            // Build section list: (section_key, display_name, text)
            let sections: Vec<(&'static str, &'static str, Option<String>)> = vec![
                ("abstract", "Abstract", extraction.sections.abstract_text.clone()),
                ("introduction", "Introduction", extraction.sections.introduction.clone()),
                ("methods", "Methods", extraction.sections.methods.clone()),
                ("results", "Results", extraction.sections.results.clone()),
                ("conclusion", "Conclusion", extraction.sections.conclusion.clone()),
            ];

            let snippet = highlight_snippet.clone();
            let hl_section = highlight_section.clone();

            view! {
                <div>
                    {if is_abstract_only {
                        view! {
                            <p class="abstract-only-label">"Abstract only \u{2014} full text unavailable"</p>
                        }.into_any()
                    } else {
                        view! { <div></div> }.into_any()
                    }}

                    {sections.into_iter().filter_map(|(key, name, text_opt)| {
                        let text = text_opt?;
                        let snippet_clone = snippet.clone();
                        let hl_section_clone = hl_section.clone();
                        Some(view! {
                            <div class="source-section">
                                <div class="source-section-header">{name}</div>
                                <SourceSectionText
                                    section_key=key.to_string()
                                    text=text
                                    highlight_snippet=snippet_clone
                                    highlight_section=hl_section_clone
                                />
                            </div>
                        })
                    }).collect_view()}
                </div>
            }.into_any()
        }
    }
}

/// Renders a single section's text, optionally with a highlighted snippet.
#[component]
fn SourceSectionText(
    section_key: String,
    text: String,
    highlight_snippet: Option<String>,
    highlight_section: Option<String>,
) -> impl IntoView {
    // Check if this section should be highlighted
    let should_highlight = highlight_snippet.is_some()
        && highlight_section
            .as_deref()
            .map(|s| s.to_lowercase() == section_key.to_lowercase())
            .unwrap_or(false);

    if should_highlight {
        let snippet = highlight_snippet.as_deref().unwrap_or("");
        match find_highlight_range(&text, snippet) {
            Some((start, end)) => {
                let before = text[..start].to_string();
                let matched = text[start..end].to_string();
                let after = text[end..].to_string();
                view! {
                    <p class="source-section-text">
                        {before}
                        <mark class="snippet-highlight">{matched}</mark>
                        {after}
                    </p>
                }.into_any()
            }
            None => view! {
                <p class="source-section-text">
                    {text}
                    <span class="text-muted text-label">" (Snippet not found in section text)"</span>
                </p>
            }.into_any(),
        }
    } else {
        view! {
            <p class="source-section-text">{text}</p>
        }.into_any()
    }
}

/// Skeleton loading state for the drawer.
#[component]
fn DrawerSkeleton(
    paper_id: String,
    #[prop(into)] on_close: Callback<web_sys::MouseEvent>,
) -> impl IntoView {
    view! {
        <div class="drawer-header">
            <h2 class="drawer-title skeleton skeleton-heading" style="width: 80%; min-height: 24px;">
            </h2>
            <button
                class="drawer-close"
                on:click=move |e| on_close.run(e)
                aria-label="Close"
            >
                "✕"
            </button>
        </div>

        // Tab strip skeleton
        <div class="drawer-tab-strip">
            <button class="drawer-tab active">"Overview"</button>
            <button class="drawer-tab">"Source"</button>
        </div>

        <div class="drawer-body">
            // Authors + year skeleton
            <div class="drawer-meta skeleton skeleton-text" style="width: 60%;"></div>

            // Abstract skeleton
            <section>
                <h3 class="drawer-section-title">"Abstract"</h3>
                <div class="skeleton skeleton-text" style="width: 100%; margin-bottom: 6px;"></div>
                <div class="skeleton skeleton-text" style="width: 95%; margin-bottom: 6px;"></div>
                <div class="skeleton skeleton-text" style="width: 88%;"></div>
            </section>

            // Methods skeleton
            <section>
                <h3 class="drawer-section-title">"Methods"</h3>
                <div class="tags-list">
                    <span class="skeleton tag" style="width: 64px; height: 24px;"></span>
                    <span class="skeleton tag" style="width: 80px; height: 24px;"></span>
                    <span class="skeleton tag" style="width: 56px; height: 24px;"></span>
                </div>
            </section>

            // Findings skeleton
            <section>
                <h3 class="drawer-section-title">"Findings"</h3>
                <div class="skeleton skeleton-text" style="width: 100%; margin-bottom: 6px;"></div>
                <div class="skeleton skeleton-text" style="width: 90%;"></div>
            </section>

            // Open problems skeleton
            <section>
                <h3 class="drawer-section-title">"Open Problems"</h3>
                <div class="skeleton skeleton-text" style="width: 85%; margin-bottom: 6px;"></div>
                <div class="skeleton skeleton-text" style="width: 70%;"></div>
            </section>

            // Paper ID reference (not a skeleton — always available)
            <p class="text-label text-muted">"arXiv: " {paper_id}</p>
        </div>
    }
}
