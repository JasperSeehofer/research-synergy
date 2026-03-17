use leptos::prelude::*;
use leptos::web_sys;
use resyn_core::datamodels::paper::Paper;

use crate::app::SelectedPaper;
use crate::server_fns::papers::get_papers;

/// Sortable columns for the papers table.
/// Status column is not sortable in the UI (it has no clickable header).
#[derive(Clone, Copy, Debug, PartialEq)]
enum SortColumn {
    Title,
    Authors,
    Year,
    Citations,
}

/// Sort direction.
#[derive(Clone, Copy, Debug, PartialEq)]
enum SortDir {
    Asc,
    Desc,
}

impl SortDir {
    fn toggle(self) -> Self {
        match self {
            SortDir::Asc => SortDir::Desc,
            SortDir::Desc => SortDir::Asc,
        }
    }
    fn indicator(self) -> &'static str {
        match self {
            SortDir::Asc => "↑",
            SortDir::Desc => "↓",
        }
    }
    fn aria(self) -> &'static str {
        match self {
            SortDir::Asc => "ascending",
            SortDir::Desc => "descending",
        }
    }
}

/// Papers panel — sortable table of crawled papers.
#[component]
pub fn PapersPanel() -> impl IntoView {
    let papers_resource = Resource::new(|| (), |_| get_papers());

    let sort_col = RwSignal::new(SortColumn::Title);
    let sort_dir = RwSignal::new(SortDir::Asc);

    let SelectedPaper(selected_paper) = expect_context::<SelectedPaper>();

    let handle_header_click = move |col: SortColumn| {
        if sort_col.get() == col {
            sort_dir.update(|d| *d = d.toggle());
        } else {
            sort_col.set(col);
            sort_dir.set(SortDir::Asc);
        }
    };

    view! {
        <div>
            <h1 class="page-title">"Papers"</h1>
            <Suspense fallback=|| view! { <PapersTableSkeleton/> }>
                {move || papers_resource.get().map(|result| match result {
                    Err(e) => view! {
                        <div class="error-banner">
                            {format!("Failed to load papers. Check that the database is reachable and retry. ({e})")}
                        </div>
                        <PapersTableSkeleton/>
                    }.into_any(),
                    Ok(papers) if papers.is_empty() => view! {
                        <div class="table-container">
                            <table class="data-table">
                                <PapersTableHead
                                    sort_col=sort_col
                                    sort_dir=sort_dir
                                    on_click=handle_header_click
                                />
                                <tbody>
                                    <tr>
                                        <td colspan="5">
                                            <div class="empty-state">
                                                <p class="empty-state-body">
                                                    "No papers in the database. Start a crawl to add papers."
                                                </p>
                                            </div>
                                        </td>
                                    </tr>
                                </tbody>
                            </table>
                        </div>
                    }.into_any(),
                    Ok(papers) => view! {
                        <div class="table-container">
                            <table class="data-table">
                                <PapersTableHead
                                    sort_col=sort_col
                                    sort_dir=sort_dir
                                    on_click=handle_header_click
                                />
                                <tbody>
                                    <PapersTableRows
                                        papers=papers
                                        sort_col=sort_col
                                        sort_dir=sort_dir
                                        on_row_click=move |id: String| selected_paper.set(Some(id))
                                    />
                                </tbody>
                            </table>
                        </div>
                    }.into_any(),
                })}
            </Suspense>
        </div>
    }
}

/// Table header row with clickable sortable columns.
#[component]
fn PapersTableHead(
    sort_col: RwSignal<SortColumn>,
    sort_dir: RwSignal<SortDir>,
    #[prop(into)] on_click: Callback<SortColumn>,
) -> impl IntoView {
    let th_class = move |col: SortColumn| {
        if sort_col.get() == col {
            "sortable sort-active"
        } else {
            "sortable"
        }
    };
    let aria_sort = move |col: SortColumn| {
        if sort_col.get() == col {
            sort_dir.get().aria()
        } else {
            "none"
        }
    };
    let indicator = move |col: SortColumn| {
        if sort_col.get() == col {
            sort_dir.get().indicator()
        } else {
            "↑"
        }
    };

    view! {
        <thead>
            <tr>
                <th
                    class=move || th_class(SortColumn::Title)
                    aria-sort=move || aria_sort(SortColumn::Title)
                    on:click=move |_| on_click.run(SortColumn::Title)
                >
                    "Title "
                    <span class="sort-indicator">{move || indicator(SortColumn::Title)}</span>
                </th>
                <th
                    class=move || th_class(SortColumn::Authors)
                    aria-sort=move || aria_sort(SortColumn::Authors)
                    on:click=move |_| on_click.run(SortColumn::Authors)
                >
                    "Authors "
                    <span class="sort-indicator">{move || indicator(SortColumn::Authors)}</span>
                </th>
                <th
                    class=move || th_class(SortColumn::Year)
                    aria-sort=move || aria_sort(SortColumn::Year)
                    style="width: 60px;"
                    on:click=move |_| on_click.run(SortColumn::Year)
                >
                    "Year "
                    <span class="sort-indicator">{move || indicator(SortColumn::Year)}</span>
                </th>
                <th
                    class=move || th_class(SortColumn::Citations)
                    aria-sort=move || aria_sort(SortColumn::Citations)
                    style="width: 80px;"
                    on:click=move |_| on_click.run(SortColumn::Citations)
                >
                    "Citations "
                    <span class="sort-indicator">{move || indicator(SortColumn::Citations)}</span>
                </th>
                <th style="width: 120px;">"Status"</th>
            </tr>
        </thead>
    }
}

/// Sorted table rows.
#[component]
fn PapersTableRows(
    papers: Vec<Paper>,
    sort_col: RwSignal<SortColumn>,
    sort_dir: RwSignal<SortDir>,
    #[prop(into)] on_row_click: Callback<String>,
) -> impl IntoView {
    let sorted = move || {
        let mut sorted = papers.clone();
        let col = sort_col.get();
        let dir = sort_dir.get();

        sorted.sort_by(|a, b| {
            let ord = match col {
                SortColumn::Title => a.title.to_lowercase().cmp(&b.title.to_lowercase()),
                SortColumn::Authors => a.authors.join(", ").to_lowercase().cmp(&b.authors.join(", ").to_lowercase()),
                SortColumn::Year => a.published.cmp(&b.published),
                SortColumn::Citations => {
                    a.citation_count.unwrap_or(0).cmp(&b.citation_count.unwrap_or(0))
                }
            };
            match dir {
                SortDir::Asc => ord,
                SortDir::Desc => ord.reverse(),
            }
        });

        sorted
    };

    view! {
        <For
            each=sorted
            key=|p| p.id.clone()
            children=move |paper| {
                let paper_id = paper.id.clone();
                let on_click = on_row_click;
                view! {
                    <PaperRow paper=paper on_click=move |_| on_click.run(paper_id.clone())/>
                }
            }
        />
    }
}

/// A single paper row.
#[component]
fn PaperRow(
    paper: Paper,
    #[prop(into)] on_click: Callback<web_sys::MouseEvent>,
) -> impl IntoView {
    let year = year_from_published(&paper.published).to_string();
    let authors_str = if paper.authors.is_empty() {
        "—".to_string()
    } else if paper.authors.len() == 1 {
        paper.authors[0].clone()
    } else {
        format!("{} et al.", paper.authors[0])
    };
    let status = status_str(&paper);
    let citations = paper.citation_count.map(|c| c.to_string()).unwrap_or_else(|| "—".to_string());
    let status_class = if status == "Analyzed" { "status-analyzed" } else { "status-pending" };

    view! {
        <tr on:click=move |e| on_click.run(e)>
            <td>{paper.title.clone()}</td>
            <td>{authors_str}</td>
            <td>{year}</td>
            <td>{citations}</td>
            <td><span class=status_class>{status}</span></td>
        </tr>
    }
}

/// 8 skeleton rows for the loading state.
#[component]
fn PapersTableSkeleton() -> impl IntoView {
    view! {
        <div class="table-container">
            <table class="data-table">
                <thead>
                    <tr>
                        <th>"Title"</th>
                        <th>"Authors"</th>
                        <th style="width: 60px;">"Year"</th>
                        <th style="width: 80px;">"Citations"</th>
                        <th style="width: 120px;">"Status"</th>
                    </tr>
                </thead>
                <tbody>
                    {(0..8_u8).map(|_| view! {
                        <tr>
                            <td><div class="skeleton skeleton-text" style="width: 80%;"></div></td>
                            <td><div class="skeleton skeleton-text" style="width: 60%;"></div></td>
                            <td><div class="skeleton skeleton-text" style="width: 40px;"></div></td>
                            <td><div class="skeleton skeleton-text" style="width: 40px;"></div></td>
                            <td><div class="skeleton skeleton-text" style="width: 60px;"></div></td>
                        </tr>
                    }).collect_view()}
                </tbody>
            </table>
        </div>
    }
}

/// Extract 4-digit year from a published string like "2023-01-01T...".
fn year_from_published(published: &str) -> &str {
    if published.len() >= 4 {
        &published[..4]
    } else {
        "—"
    }
}

/// Determine status string for a paper based on whether it has annotations.
/// Since papers don't carry annotation status directly, we use citation_count
/// as a proxy (None = not enriched = Pending; Some = at least partially processed).
/// The real status will be derived from annotation presence once the annotation
/// endpoint is wired up in later plans.
fn status_str(paper: &Paper) -> &'static str {
    // TODO: derive from LLM annotation presence (Plan 05 wires this up).
    // For now, papers with a citation count have been processed by InspireHEP.
    match paper.citation_count {
        Some(_) => "Analyzed",
        None => "Pending",
    }
}
