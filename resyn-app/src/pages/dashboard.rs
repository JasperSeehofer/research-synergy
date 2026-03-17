use leptos::prelude::*;

use crate::server_fns::papers::{DashboardStats, get_dashboard_stats};

/// Dashboard page — summary cards for the research corpus.
#[component]
pub fn Dashboard() -> impl IntoView {
    let stats = Resource::new(|| (), |_| get_dashboard_stats());

    view! {
        <div>
            <h1 class="page-title">"Dashboard"</h1>
            <Suspense fallback=|| view! {
                // Loading state: skeleton numbers on all 5 cards
                <div class="dashboard-cards">
                    <SkeletonCard title="Total Papers" link_href="/papers" link_text="View all →"/>
                    <SkeletonCard title="Contradictions" link_href="/gaps" link_text="View all →"/>
                    <SkeletonCard title="ABC-Bridges" link_href="/gaps" link_text="View all →"/>
                    <SkeletonCard title="Open Problems" link_href="/problems" link_text="View all →"/>
                    <SkeletonCard title="Method Coverage" link_href="/methods" link_text="View matrix →"/>
                </div>
            }>
                {move || stats.get().map(|result| match result {
                    Err(e) => view! {
                        <div class="dashboard-cards">
                            <ErrorCard title="Total Papers" link_href="/papers" link_text="View all →"/>
                            <ErrorCard title="Contradictions" link_href="/gaps" link_text="View all →"/>
                            <ErrorCard title="ABC-Bridges" link_href="/gaps" link_text="View all →"/>
                            <ErrorCard title="Open Problems" link_href="/problems" link_text="View all →"/>
                            <ErrorCard title="Method Coverage" link_href="/methods" link_text="View matrix →"/>
                        </div>
                        <div class="error-banner">
                            {format!("Failed to load dashboard stats: {e}")}
                        </div>
                    }.into_any(),
                    Ok(s) => view! {
                        <DashboardCards stats=s/>
                    }.into_any(),
                })}
            </Suspense>
        </div>
    }
}

/// Renders the 5 summary cards given loaded stats.
#[component]
fn DashboardCards(stats: DashboardStats) -> impl IntoView {
    let coverage_str = format!("{:.0}%", stats.method_coverage_pct);
    view! {
        <div class="dashboard-cards">
            <SummaryCard
                title="Total Papers"
                number=stats.total_papers.to_string()
                link_href="/papers"
                link_text="View all →"
            />
            <SummaryCard
                title="Contradictions"
                number=stats.contradiction_count.to_string()
                link_href="/gaps"
                link_text="View all →"
            />
            <SummaryCard
                title="ABC-Bridges"
                number=stats.bridge_count.to_string()
                link_href="/gaps"
                link_text="View all →"
            />
            <SummaryCard
                title="Open Problems"
                number=stats.open_problems_count.to_string()
                link_href="/problems"
                link_text="View all →"
            />
            <SummaryCard
                title="Method Coverage"
                number=coverage_str
                link_href="/methods"
                link_text="View matrix →"
            />
        </div>
        {(stats.total_papers == 0).then(|| view! {
            <div class="empty-state">
                <p class="empty-state-heading">"No papers crawled yet"</p>
                <p class="empty-state-body">"Start a crawl from the sidebar to populate your research corpus."</p>
            </div>
        })}
    }
}

/// A single summary card in the dashboard.
#[component]
fn SummaryCard(
    title: &'static str,
    number: String,
    link_href: &'static str,
    link_text: &'static str,
) -> impl IntoView {
    view! {
        <div class="dashboard-card">
            <p class="dashboard-card-title">{title}</p>
            <p class="dashboard-card-number">{number}</p>
            <a class="dashboard-card-link" href=link_href>{link_text}</a>
        </div>
    }
}

/// A summary card in the skeleton loading state.
#[component]
fn SkeletonCard(
    title: &'static str,
    link_href: &'static str,
    link_text: &'static str,
) -> impl IntoView {
    view! {
        <div class="dashboard-card">
            <p class="dashboard-card-title">{title}</p>
            <p class="skeleton skeleton-display dashboard-card-number"></p>
            <a class="dashboard-card-link" href=link_href>{link_text}</a>
        </div>
    }
}

/// A summary card in the error state.
#[component]
fn ErrorCard(
    title: &'static str,
    link_href: &'static str,
    link_text: &'static str,
) -> impl IntoView {
    view! {
        <div class="dashboard-card">
            <p class="dashboard-card-title">{title}</p>
            <p class="dashboard-card-number" style="font-size: var(--font-size-body); color: var(--color-text-muted);">
                "—"
            </p>
            <p class="text-label text-muted" style="margin-bottom: var(--space-xs);">"Failed to load"</p>
            <a class="dashboard-card-link" href=link_href>{link_text}</a>
        </div>
    }
}
