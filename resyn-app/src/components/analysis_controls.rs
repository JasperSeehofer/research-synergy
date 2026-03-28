use codee::string::JsonSerdeCodec;
use leptos::prelude::*;
use leptos_use::{UseEventSourceReturn, use_event_source};
use resyn_core::datamodels::progress::ProgressEvent;

use crate::server_fns::analysis::{check_llm_configured, start_analysis};

/// Analysis controls: LLM warning banner + Run Analysis button.
///
/// Shows:
/// - LLM warning banner (D-07) when `RESYN_LLM_PROVIDER` is not configured.
/// - "Run Analysis" button (D-01) that triggers the analysis pipeline.
/// - Disables the button while analysis SSE events are active.
#[component]
pub fn AnalysisControls() -> impl IntoView {
    // Server resource: is LLM provider configured?
    let llm_configured = Resource::new(|| (), |_| check_llm_configured());

    // Own SSE subscription — lightweight; multiple connections to the same broadcast are fine.
    let UseEventSourceReturn {
        message: sse_message,
        ..
    } = use_event_source::<ProgressEvent, JsonSerdeCodec>("/progress");

    let last_event: RwSignal<Option<ProgressEvent>> = RwSignal::new(None);
    Effect::new(move |_| {
        if let Some(msg) = sse_message.get() {
            last_event.set(Some(msg.data));
        }
    });

    // Derived: is analysis currently running (analysis_* event active, not complete/error)?
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

    let analysis_action = Action::new(move |_: &()| async move { start_analysis().await });

    let is_pending = analysis_action.pending();

    view! {
        // LLM Warning Banner (D-07): shown when RESYN_LLM_PROVIDER is unset.
        <Suspense fallback=|| ()>
            {move || llm_configured.get().map(|result| {
                match result {
                    Ok(false) => view! {
                        <div class="warning-banner">
                            "LLM provider not configured \u{2014} showing NLP-only results. Set RESYN_LLM_PROVIDER for full analysis."
                        </div>
                    }.into_any(),
                    _ => view! { <span></span> }.into_any(),
                }
            })}
        </Suspense>

        // Run Analysis Button (D-01)
        <div style="margin-top: var(--space-lg);">
            <button
                class="btn-primary"
                disabled=move || is_pending.get() || is_analysis_running()
                on:click=move |_| { analysis_action.dispatch(()); }
            >
                {move || if is_pending.get() || is_analysis_running() {
                    "Analysis running\u{2026}"
                } else {
                    "Run Analysis"
                }}
            </button>
        </div>

        // Result feedback — success or error after action resolves.
        {move || analysis_action.value().get().map(|r| match r {
            Ok(msg) => view! {
                <p class="status-text success">{msg}</p>
            }.into_any(),
            Err(e) => view! {
                <p class="status-text error">{format!("Error: {}", e)}</p>
            }.into_any(),
        })}
    }
}
