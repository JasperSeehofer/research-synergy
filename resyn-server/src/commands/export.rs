use clap::Args;
use tracing::{error, info};

#[derive(Args, Debug)]
pub struct ExportArgs {
    /// Database connection string (e.g. "surrealkv://./data")
    #[arg(long, default_value = "surrealkv://./data")]
    pub db: String,

    /// Output JSON file path
    #[arg(long)]
    pub output: String,

    /// Exclude papers published after this date (ISO-8601: "YYYY-MM-DD").
    /// Required for a reproducible pre-2015 Kuramoto-LBD v03 corpus slice.
    #[arg(long)]
    pub published_before: Option<String>,

    /// Maximum number of TF-IDF terms to export per node (default: 50)
    #[arg(long, default_value_t = 50)]
    pub tfidf_top_n: usize,
}

pub async fn run(args: ExportArgs) -> anyhow::Result<()> {
    let db = match resyn_core::database::client::connect(&args.db).await {
        Ok(db) => {
            info!(endpoint = args.db.as_str(), "Connected to database");
            db
        }
        Err(e) => {
            error!(error = %e, "Failed to connect to database");
            std::process::exit(1);
        }
    };

    let cutoff = args.published_before.as_deref();
    info!(
        published_before = cutoff.unwrap_or("(none)"),
        tfidf_top_n = args.tfidf_top_n,
        "Exporting Louvain community graph"
    );

    let graph = resyn_core::graph_analytics::community::export_community_graph(
        &db,
        cutoff,
        args.tfidf_top_n,
    )
    .await
    .map_err(|e| anyhow::anyhow!("Export failed: {e}"))?;

    info!(
        nodes = graph.nodes.len(),
        edges = graph.edges.len(),
        fingerprint = graph.corpus_fingerprint.as_str(),
        "Graph assembled"
    );

    let json = serde_json::to_string_pretty(&graph)
        .map_err(|e| anyhow::anyhow!("JSON serialization failed: {e}"))?;

    std::fs::write(&args.output, json)
        .map_err(|e| anyhow::anyhow!("Failed to write {}: {e}", args.output))?;

    info!(path = args.output.as_str(), "Export written");
    Ok(())
}
