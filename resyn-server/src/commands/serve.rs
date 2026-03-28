use anyhow::Context;
use axum::extract::State;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::{Router, routing::get, routing::post};
use clap::Args;
use futures::StreamExt;
use leptos::prelude::provide_context;
use leptos_axum::handle_server_fns_with_context;
use resyn_core::database::client::{Db, connect};
use resyn_core::datamodels::progress::ProgressEvent;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tower_http::services::{ServeDir, ServeFile};
use tracing::info;

#[derive(Args, Debug)]
pub struct ServeArgs {
    /// SurrealDB connection string
    #[arg(long, default_value = "surrealkv://./data")]
    pub db: String,

    /// Port to listen on
    #[arg(long, default_value_t = 3100)]
    pub port: u16,
}

/// SSE handler — subscribes to the progress broadcast channel and streams
/// `ProgressEvent` JSON to connected clients.
async fn sse_progress(
    State(tx): State<broadcast::Sender<ProgressEvent>>,
) -> Sse<impl futures::Stream<Item = Result<Event, std::convert::Infallible>>> {
    let rx = tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|msg| async move {
        msg.ok().and_then(|e| {
            serde_json::to_string(&e)
                .ok()
                .map(|data| Ok(Event::default().data(data)))
        })
    });
    Sse::new(stream).keep_alive(KeepAlive::new().interval(std::time::Duration::from_secs(5)))
}

pub async fn run(args: ServeArgs) -> anyhow::Result<()> {
    // Register all Leptos server functions explicitly (inventory auto-registration
    // doesn't work across crate boundaries in this setup).
    use resyn_app::server_fns::{analysis, gaps, graph, methods, papers, problems};
    use server_fn::axum::register_explicit;
    register_explicit::<papers::GetPapers>();
    register_explicit::<papers::GetPaperDetail>();
    register_explicit::<papers::GetDashboardStats>();
    register_explicit::<papers::StartCrawl>();
    register_explicit::<analysis::StartAnalysis>();
    register_explicit::<analysis::CheckLlmConfigured>();
    register_explicit::<gaps::GetGapFindings>();
    register_explicit::<problems::GetOpenProblemsRanked>();
    register_explicit::<methods::GetMethodMatrix>();
    register_explicit::<methods::GetMethodDrilldown>();
    register_explicit::<graph::GetGraphData>();

    let db: Db = connect(&args.db)
        .await
        .with_context(|| format!("Failed to connect to database at {}", args.db))?;
    let db = Arc::new(db);

    // Broadcast channel for SSE crawl progress events.
    let (progress_tx, _) = broadcast::channel::<ProgressEvent>(256);

    let db_for_fns = db.clone();
    let tx_for_fns = progress_tx.clone();
    let app = Router::new()
        // SSE progress endpoint — must be registered before the fallback.
        .route(
            "/progress",
            get(sse_progress).with_state(progress_tx.clone()),
        )
        // Leptos server functions are registered at /api/<ServerFnName>
        .route(
            "/api/{*fn_name}",
            post({
                let db = db_for_fns.clone();
                let tx = tx_for_fns.clone();
                move |req| {
                    handle_server_fns_with_context(
                        move || {
                            provide_context(db.clone());
                            provide_context(tx.clone());
                        },
                        req,
                    )
                }
            }),
        )
        // Serve WASM/JS dist files as a fallback (production mode)
        .fallback_service(
            ServeDir::new("resyn-app/dist")
                .not_found_service(ServeFile::new("resyn-app/dist/index.html")),
        );

    let addr = format!("127.0.0.1:{}", args.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .with_context(|| format!("Failed to bind to {addr}"))?;

    info!("ReSyn server listening on http://{addr}");
    axum::serve(listener, app).await.context("Server error")?;

    Ok(())
}
