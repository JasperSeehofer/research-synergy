use leptos::prelude::*;
use resyn_core::analysis::aggregation::MethodMatrix;

/// Determine the CSS class for a cell based on its count.
fn cell_class(count: u32) -> &'static str {
    match count {
        0 => "heatmap-cell cell-empty",
        1..=3 => "heatmap-cell cell-low",
        4..=9 => "heatmap-cell cell-medium",
        _ => "heatmap-cell cell-high",
    }
}

/// A CSS grid heatmap for a `MethodMatrix`.
///
/// - Column header labels are rotated 45 degrees via `.heatmap-col-header`.
/// - Row labels appear to the left of each row.
/// - Empty cells (count == 0) use `.cell-empty` (dark gray, dashed border).
/// - Non-empty cells use `.cell-low`, `.cell-medium`, `.cell-high` based on count.
/// - Clicking a non-empty cell fires `on_cell_click` with `(row_category, col_category)`.
#[component]
pub fn Heatmap(
    /// The matrix data to render.
    matrix: MethodMatrix,
    /// Optional callback fired when a non-empty cell is clicked.
    /// Receives `(row_category, col_category)`.
    #[prop(optional)]
    on_cell_click: Option<Callback<(String, String)>>,
) -> impl IntoView {
    let categories = matrix.categories.clone();
    let pair_counts = matrix.pair_counts.clone();
    let n = categories.len();

    if n == 0 {
        return view! {
            <div class="empty-state">
                <p class="empty-state-body">"No method categories found."</p>
            </div>
        }
        .into_any();
    }

    // Grid layout: 1 label column + N data columns.
    let grid_cols = format!("auto repeat({}, 40px)", n);

    let cats_for_header = categories.clone();
    let header_row = view! {
        // Top-left empty corner cell
        <div></div>
        // Column headers (rotated)
        {cats_for_header.into_iter().map(|cat| {
            let title = cat.clone();
            let label = cat.clone();
            view! {
                <div class="heatmap-col-header" title=title>{label}</div>
            }
        }).collect_view()}
    };

    let rows = categories
        .iter()
        .map(|row_cat| {
            let row_cat = row_cat.clone();
            // Pre-clone the row label for use after cells_view consumes row_cat.
            let row_label = row_cat.clone();
            let col_cats = categories.clone();
            let pair_counts_ref = pair_counts.clone();
            let on_click = on_cell_click.clone();

            let cells_view = col_cats.into_iter().map(move |col_cat| {
                // Normalize key: always store with alphabetically smaller category first.
                let key = if row_cat <= col_cat {
                    (row_cat.clone(), col_cat.clone())
                } else {
                    (col_cat.clone(), row_cat.clone())
                };
                let count = *pair_counts_ref.get(&key).unwrap_or(&0);
                let class = cell_class(count);
                let title = format!("{} × {} : {} paper(s)", row_cat, col_cat, count);
                let row_for_click = row_cat.clone();
                let col_for_click = col_cat.clone();
                let on_click_inner = on_click.clone();

                if count > 0 {
                    view! {
                        <div
                            class=class
                            title=title
                            role="button"
                            tabindex="0"
                            on:click=move |_| {
                                if let Some(cb) = &on_click_inner {
                                    cb.run((row_for_click.clone(), col_for_click.clone()));
                                }
                            }
                        >
                            <span style="font-size:10px;color:#0d1117;font-weight:600;">{count.to_string()}</span>
                        </div>
                    }
                    .into_any()
                } else {
                    view! {
                        <div class=class title=title></div>
                    }
                    .into_any()
                }
            });

            view! {
                // Row label
                <div class="heatmap-axis-label" style="display:flex;align-items:center;padding-right:8px;white-space:nowrap;">
                    {row_label}
                </div>
                // Row cells
                {cells_view.collect_view()}
            }
        })
        .collect_view();

    view! {
        <div
            class="heatmap-grid"
            style=format!("grid-template-columns:{};", grid_cols)
            role="grid"
            aria-label="Method co-occurrence heatmap"
        >
            {header_row}
            {rows}
        </div>
    }
    .into_any()
}
