use anyhow::Context;
use axum::{Router, routing::post};
use clap::Args;
use leptos::prelude::provide_context;
use leptos_axum::handle_server_fns_with_context;
use std::sync::Arc;
use tower_http::services::ServeDir;
use tracing::info;

use resyn_core::database::client::{connect_local, Db};

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
    let db: Db = connect_local(&args.db)
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
