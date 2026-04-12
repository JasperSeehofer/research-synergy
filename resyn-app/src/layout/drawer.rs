use leptos::prelude::*;
use leptos::web_sys;
use resyn_core::analysis::highlight::find_highlight_range;
use resyn_core::datamodels::extraction::{ExtractionMethod, TextExtractionResult};

use crate::app::{DrawerOpenRequest, DrawerTab, SelectedPaper};
use crate::server_fns::papers::{PaperDetail, get_paper_detail};
use crate::server_fns::similarity::get_similar_papers;

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
            <button
                class=move || if active_tab.get() == DrawerTab::Similar {
                    "drawer-tab active"
                } else {
                    "drawer-tab"
                }
                on:click=move |_| active_tab.set(DrawerTab::Similar)
            >
                "Similar"
            </button>
        </div>

        // Tab content
        {move || {
            match active_tab.get() {
                DrawerTab::Overview => view! {
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
                }.into_any(),
                DrawerTab::Source => view! {
                    <div class="drawer-body">
                        <SourceTabBody
                            extraction=extraction.clone()
                            highlight_snippet=highlight_snippet.clone()
                            highlight_section=highlight_section.clone()
                        />
                    </div>
                }.into_any(),
                DrawerTab::Similar => view! {
                    <div class="drawer-body">
                        <SimilarTabBody paper_id=arxiv_id.clone() />
                    </div>
                }.into_any(),
            }
        }}
    }
}

/// Similar tab body — shows a ranked list of similar papers with score, metadata, and shared keywords.
/// When no similarity data exists yet (empty vec from server fn), shows a waiting spinner (D-08).
#[component]
fn SimilarTabBody(paper_id: String) -> impl IntoView {
    let pid = paper_id.clone();
    let similar_resource = Resource::new(move || pid.clone(), |id| get_similar_papers(id));
    let selected = expect_context::<SelectedPaper>();

    view! {
        <Suspense fallback=move || view! {
            <div class="similar-waiting-state">
                <div class="spinner"></div>
                <p class="text-body text-muted">"Loading similar papers..."</p>
            </div>
        }>
            {move || {
                similar_resource.get().map(|result| match result {
                    Ok(entries) if entries.is_empty() => {
                        // D-08: No similarity data yet — show waiting spinner
                        view! {
                            <div class="similar-waiting-state">
                                <div class="spinner"></div>
                                <p class="text-body text-muted">"Waiting for TF-IDF analysis..."</p>
                                <p class="text-label text-muted">"Run analysis to compute paper similarity."</p>
                            </div>
                        }.into_any()
                    }
                    Ok(entries) => {
                        // D-01: Ranked list with score %, title, authors, year, shared keywords
                        view! {
                            <div class="similar-papers-list">
                                {entries.into_iter().map(|entry| {
                                    let arxiv_id = entry.arxiv_id.clone();
                                    let score_pct = format!("{:.0}%", entry.score * 100.0);
                                    let authors_str = if entry.authors.len() > 2 {
                                        format!("{} et al.", entry.authors[0])
                                    } else {
                                        entry.authors.join(", ")
                                    };
                                    let shared = entry.shared_terms.join(", ");

                                    // D-02: Click navigates to the similar paper's drawer
                                    let on_click = {
                                        let sel = selected.0;
                                        let id = arxiv_id.clone();
                                        move |_: web_sys::MouseEvent| {
                                            sel.set(Some(DrawerOpenRequest {
                                                paper_id: id.clone(),
                                                initial_tab: DrawerTab::Overview,
                                                highlight_snippet: None,
                                                highlight_section: None,
                                            }));
                                        }
                                    };

                                    view! {
                                        <div class="similar-paper-item" on:click=on_click>
                                            <div class="similar-paper-header">
                                                <span class="similar-score">{score_pct}</span>
                                                <span class="similar-title">{entry.title}</span>
                                            </div>
                                            <div class="similar-paper-meta">
                                                <span class="similar-authors">{authors_str}</span>
                                                {(!entry.year.is_empty()).then(|| view! {
                                                    <span class="similar-year">{entry.year}</span>
                                                })}
                                            </div>
                                            {(!shared.is_empty()).then(|| view! {
                                                <div class="similar-shared-terms">
                                                    <span class="shared-label">"Shared: "</span>
                                                    <span class="shared-terms">{shared}</span>
                                                </div>
                                            })}
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        }.into_any()
                    }
                    Err(_) => view! {
                        <p class="text-body text-muted">"Failed to load similar papers."</p>
                    }.into_any(),
                })
            }}
        </Suspense>
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

                    {
                        let section_views: Vec<_> = sections.into_iter().filter_map(|(key, name, text_opt)| {
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
                        }).collect();
                        if section_views.is_empty() {
                            view! {
                                <p class="text-body text-muted">"No source text available for this paper."</p>
                            }.into_any()
                        } else {
                            section_views.collect_view().into_any()
                        }
                    }
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
        }
        .into_any()
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
