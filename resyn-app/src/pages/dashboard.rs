use leptos::prelude::*;

/// Dashboard page — summary cards for the research corpus.
#[component]
pub fn Dashboard() -> impl IntoView {
    view! {
        <div>
            <h1 class="page-title">"Dashboard"</h1>
            <div class="dashboard-cards">
                <SummaryCard title="Total Papers" number="—" link_href="/papers" link_text="View all →"/>
                <SummaryCard title="Contradictions" number="—" link_href="/gaps" link_text="View all →"/>
                <SummaryCard title="ABC-Bridges" number="—" link_href="/gaps" link_text="View all →"/>
                <SummaryCard title="Open Problems" number="—" link_href="/problems" link_text="View all →"/>
                <SummaryCard title="Method Coverage" number="—" link_href="/methods" link_text="View matrix →"/>
            </div>
            <div class="empty-state">
                <p class="empty-state-heading">"No papers crawled yet"</p>
                <p class="empty-state-body">"Start a crawl from the sidebar to populate your research corpus."</p>
            </div>
        </div>
    }
}

/// A single summary card in the dashboard.
#[component]
fn SummaryCard(
    title: &'static str,
    number: &'static str,
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
