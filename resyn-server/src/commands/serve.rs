use anyhow::Context;
use axum::{Router, routing::post};
use clap::Args;
use leptos::prelude::provide_context;
use leptos_axum::handle_server_fns_with_context;
use std::sync::Arc;
use tower_http::services::ServeDir;
use tracing::info;

use resyn_core::database::client::{Db, connect};

#[derive(Args, Debug)]
pub struct ServeArgs {
    /// SurrealDB connection string
    #[arg(long, default_value = "surrealkv://./data")]
    pub db: String,

    /// Port to listen on
    #[arg(long, default_value_t = 3000)]
    pub port: u16,
}

pub async fn run(args: ServeArgs) -> anyhow::Result<()> {
    // Register all Leptos server functions explicitly (inventory auto-registration
    // doesn't work across crate boundaries in this setup).
    use resyn_app::server_fns::{gaps, graph, methods, papers, problems};
    use server_fn::axum::register_explicit;
    register_explicit::<papers::GetPapers>();
    register_explicit::<papers::GetPaperDetail>();
    register_explicit::<papers::GetDashboardStats>();
    register_explicit::<papers::StartCrawl>();
    register_explicit::<gaps::GetGapFindings>();
    register_explicit::<problems::GetOpenProblemsRanked>();
    register_explicit::<methods::GetMethodMatrix>();
    register_explicit::<methods::GetMethodDrilldown>();
    register_explicit::<graph::GetGraphData>();

    let db: Db = connect(&args.db)
        .await
        .with_context(|| format!("Failed to connect to database at {}", args.db))?;
    let db = Arc::new(db);

    let db_for_fns = db.clone();
    let app = Router::new()
        // Leptos server functions are registered at /api/<ServerFnName>
        .route(
            "/api/{*fn_name}",
            post({
                let db = db_for_fns.clone();
                move |req| {
                    handle_server_fns_with_context(
                        move || {
                            provide_context(db.clone());
                        },
                        req,
                    )
                }
            }),
        )
        // Serve WASM/JS dist files as a fallback (production mode)
        .fallback_service(ServeDir::new("resyn-app/dist"));

    let addr = format!("127.0.0.1:{}", args.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .with_context(|| format!("Failed to bind to {addr}"))?;

    info!("ReSyn server listening on http://{addr}");
    axum::serve(listener, app).await.context("Server error")?;

    Ok(())
}
