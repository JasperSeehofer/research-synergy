use codee::string::JsonSerdeCodec;
use leptos::prelude::*;
use leptos_use::{UseEventSourceReturn, use_event_source};
use resyn_core::datamodels::progress::ProgressEvent;

use crate::server_fns::analysis::start_analysis;
use crate::server_fns::papers::start_crawl;

/// SSE-connected crawl progress display with inline crawl launcher form.
///
/// Receives live `ProgressEvent` updates from `/progress` (the Phase 7 SSE endpoint).
/// Shows:
///  - **Analysis running state**: analysis progress bar with stage label (D-03).
///  - **Crawl running state**: crawl progress bar, stats line, current paper title.
///  - **Idle state**: "No active crawl" + optional post-crawl analysis prompt + crawl start form.
///  - **Collapsed (rail) state**: spinner icon only.
#[component]
pub fn CrawlProgress(
    /// Whether the sidebar is in collapsed (rail) mode.
    collapsed: RwSignal<bool>,
) -> impl IntoView {
    // Connect to the SSE endpoint.
    let UseEventSourceReturn {
        message: sse_message,
        ready_state: _,
        ..
    } = use_event_source::<ProgressEvent, JsonSerdeCodec>("/progress");

    // Track the latest event — None means no events yet (idle).
    let last_event: RwSignal<Option<ProgressEvent>> = RwSignal::new(None);

    // Update last_event whenever a new SSE message arrives.
    Effect::new(move |_| {
        if let Some(msg) = sse_message.get() {
            last_event.set(Some(msg.data));
        }
    });

    // Derived signal: is a crawl or analysis currently running?
    let is_running = move || {
        last_event
            .get()
            .as_ref()
            .map(|e| e.event_type != "complete" && e.event_type != "idle")
            .unwrap_or(false)
    };

    // Derived signal: is analysis specifically running (not complete/error)?
    let is_analysis_running = move || {
        last_event
            .get()
            .as_ref()
            .map(|e| {
                e.event_type.starts_with("analysis_")
                    && e.event_type != "analysis_complete"
                    && e.event_type != "analysis_error"
            })
            .unwrap_or(false)
    };

    // Progress percentage (0–100) based on depth progress.
    let progress_pct = move || {
        last_event
            .get()
            .as_ref()
            .map(|e| {
                if e.max_depth == 0 {
                    0u32
                } else {
                    ((e.current_depth as f64 / e.max_depth as f64) * 100.0) as u32
                }
            })
            .unwrap_or(0)
    };

    // Action for the post-crawl "Run now" prompt.
    let analysis_action = Action::new(move |_: &()| async move { start_analysis().await });

    view! {
        {move || {
            if collapsed.get() {
                // Collapsed (rail) mode: show spinner if running, or idle dot.
                if is_running() {
                    view! {
                        <div style="display:flex;flex-direction:column;align-items:center;gap:4px;">
                            <div class="crawl-spinner"></div>
                            <span class="crawl-percent-label">{progress_pct().to_string()}{"%"}</span>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div style="display:flex;align-items:center;justify-content:center;">
                            <span title="No active crawl" style="font-size:16px;color:var(--color-text-muted);">"·"</span>
                        </div>
                    }.into_any()
                }
            } else if is_analysis_running() {
                // Analysis progress display (per D-03, UI-SPEC Component 4).
                let event = last_event.get().unwrap_or_else(|| ProgressEvent {
                    event_type: "idle".to_string(),
                    papers_found: 0,
                    papers_pending: 0,
                    papers_failed: 0,
                    current_depth: 0,
                    max_depth: 0,
                    elapsed_secs: 0.0,
                    current_paper_id: None,
                    current_paper_title: None,
                    analysis_stage: None,
                });
                let stage_label = match event.event_type.as_str() {
                    "analysis_extracting" => "Extracting text",
                    "analysis_nlp" => "Running TF-IDF",
                    "analysis_llm" => "Annotating papers",
                    "analysis_gaps" => "Finding gaps",
                    _ => "Analyzing",
                };
                let stats_text = format!("{} / {}", event.papers_found, event.papers_pending);

                view! {
                    <div>
                        <p class="analysis-stage-label">{format!("Analysis: {}", stage_label)}</p>
                        <div class="crawl-progress-bar">
                            <div class="crawl-progress-fill" style="width:50%"></div>
                        </div>
                        <p class="crawl-progress-stats">{stats_text}</p>
                    </div>
                }.into_any()
            } else if is_running() {
                // Expanded crawl running state: progress bar + stats.
                let event = last_event.get().unwrap_or_else(|| ProgressEvent {
                    event_type: "idle".to_string(),
                    papers_found: 0,
                    papers_pending: 0,
                    papers_failed: 0,
                    current_depth: 0,
                    max_depth: 0,
                    elapsed_secs: 0.0,
                    current_paper_id: None,
                    current_paper_title: None,
                    analysis_stage: None,
                });
                let pct = progress_pct();
                let stats_text = format!(
                    "{} found  \u{2022}  {} failed  \u{2022}  depth {}/{}",
                    event.papers_found,
                    event.papers_failed,
                    event.current_depth,
                    event.max_depth
                );
                let current_title = event
                    .current_paper_title
                    .clone()
                    .or_else(|| event.current_paper_id.clone())
                    .unwrap_or_default();

                view! {
                    <div>
                        <p class="crawl-progress-running">"Crawl in progress"</p>
                        <div class="crawl-progress-bar">
                            <div
                                class="crawl-progress-fill"
                                style=format!("width:{}%", pct)
                            ></div>
                        </div>
                        <p class="crawl-progress-stats">{stats_text}</p>
                        {if !current_title.is_empty() {
                            let title_attr = current_title.clone();
                            view! {
                                <p class="crawl-current-paper" title=title_attr>{current_title}</p>
                            }.into_any()
                        } else {
                            view! { <span></span> }.into_any()
                        }}
                    </div>
                }.into_any()
            } else {
                // Expanded idle state: last summary + optional analysis prompt + start form.
                let summary = last_event.get().as_ref().map(|e| {
                    format!(
                        "Last crawl: {} papers found, {} failed ({:.1}s)",
                        e.papers_found, e.papers_failed, e.elapsed_secs
                    )
                });
                // Show post-crawl prompt only when the last event was a crawl completion (D-01).
                let show_prompt = last_event.get()
                    .map(|e| e.event_type == "complete")
                    .unwrap_or(false);

                view! {
                    <div>
                        {if let Some(s) = summary {
                            view! { <p class="crawl-progress-idle">{s}</p> }.into_any()
                        } else {
                            view! { <p class="crawl-progress-idle">"No active crawl"</p> }.into_any()
                        }}
                        // Post-crawl inline analysis prompt (UI-SPEC Component 3).
                        {if show_prompt {
                            view! {
                                <div class="analysis-prompt">
                                    "Analysis available \u{2014}"
                                    <button
                                        class="analysis-prompt-btn"
                                        on:click=move |_| { analysis_action.dispatch(()); }
                                    >
                                        "Run now"
                                    </button>
                                </div>
                            }.into_any()
                        } else {
                            view! { <span></span> }.into_any()
                        }}
                        <CrawlForm/>
                    </div>
                }.into_any()
            }
        }}
    }
}

/// Crawl start form — paper ID input, depth select, source select, submit button.
/// Calls the `start_crawl` server function via a Leptos Action.
#[component]
fn CrawlForm() -> impl IntoView {
    let paper_id: RwSignal<String> = RwSignal::new(String::new());
    let depth: RwSignal<u8> = RwSignal::new(3);
    let source: RwSignal<String> = RwSignal::new("arxiv".to_string());

    // Action wrapping the start_crawl server function.
    let crawl_action = Action::new(move |(pid, d, src): &(String, usize, String)| {
        let pid = pid.clone();
        let d = *d;
        let src = src.clone();
        async move { start_crawl(pid, d, src).await }
    });

    let is_pending = crawl_action.pending();
    let result = crawl_action.value();

    let on_submit = move |e: leptos::ev::SubmitEvent| {
        e.prevent_default();
        let pid = paper_id.get();
        if pid.is_empty() {
            return;
        }
        crawl_action.dispatch((pid, depth.get() as usize, source.get()));
    };

    view! {
        <form class="crawl-form" on:submit=on_submit>
            <div>
                <label class="form-label" for="crawl-paper-id">"Paper ID"</label>
                <input
                    id="crawl-paper-id"
                    class="form-input"
                    type="text"
                    placeholder="e.g. 2503.18887"
                    prop:value=paper_id
                    on:input=move |e| paper_id.set(event_target_value(&e))
                />
            </div>
            <div>
                <label class="form-label" for="crawl-depth">"Depth"</label>
                <select
                    id="crawl-depth"
                    class="form-select"
                    on:change=move |e| {
                        if let Ok(v) = event_target_value(&e).parse::<u8>() {
                            depth.set(v);
                        }
                    }
                >
                    {(1u8..=10).map(|d| view! {
                        <option value=d.to_string() selected=move || depth.get() == d>
                            {d.to_string()}
                        </option>
                    }).collect_view()}
                </select>
            </div>
            <div>
                <label class="form-label" for="crawl-source">"Source"</label>
                <select
                    id="crawl-source"
                    class="form-select"
                    on:change=move |e| source.set(event_target_value(&e))
                >
                    <option value="arxiv" selected=move || source.get() == "arxiv">"arXiv"</option>
                    <option value="inspirehep" selected=move || source.get() == "inspirehep">"InspireHEP"</option>
                </select>
            </div>
            <button
                type="submit"
                class="btn-primary"
                disabled=move || paper_id.get().is_empty() || is_pending.get()
            >
                {move || if is_pending.get() { "Starting\u{2026}" } else { "Start Crawl" }}
            </button>

            // Result feedback
            {move || result.get().map(|r| match r {
                Ok(msg) => view! {
                    <p class="status-text success">{msg}</p>
                }.into_any(),
                Err(e) => view! {
                    <p class="status-text error">{format!("Error: {}", e)}</p>
                }.into_any(),
            })}
        </form>
    }
}
