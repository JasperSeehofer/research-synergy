use leptos::prelude::*;

/// Papers panel — sortable table of crawled papers.
#[component]
pub fn PapersPanel() -> impl IntoView {
    view! {
        <div>
            <h1 class="page-title">"Papers"</h1>
            <div class="table-container">
                <table>
                    <thead>
                        <tr>
                            <th class="sortable">"Title " <span class="sort-indicator">"↑"</span></th>
                            <th class="sortable">"Authors " <span class="sort-indicator">"↑"</span></th>
                            <th class="sortable" style="width: 60px;">"Year " <span class="sort-indicator">"↑"</span></th>
                            <th class="sortable" style="width: 80px;">"Citations " <span class="sort-indicator">"↑"</span></th>
                            <th style="width: 120px;">"Status"</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr>
                            <td colspan="5">
                                <div class="empty-state">
                                    <p class="empty-state-body">"No papers in the database. Start a crawl to add papers."</p>
                                </div>
                            </td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </div>
    }
}
