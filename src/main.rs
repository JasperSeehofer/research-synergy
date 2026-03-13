#[allow(dead_code)]
mod data_aggregation;
#[allow(dead_code)]
mod data_processing;
#[allow(dead_code)]
mod datamodels;
#[allow(dead_code)]
mod error;
#[allow(dead_code)]
mod utils;
#[allow(dead_code)]
mod validation;
#[allow(dead_code)]
mod visualization;

use clap::Parser;
use datamodels::paper::Paper;
use eframe::run_native;
use tracing::{error, info};
use visualization::force_graph_app::DemoApp;

#[derive(Parser, Debug)]
#[command(
    name = "resyn",
    about = "Research Synergy - Literature Based Discovery"
)]
struct Cli {
    /// arXiv paper ID to use as seed (e.g. 2503.18887)
    #[arg(short, long, default_value = "2503.18887")]
    paper_id: String,

    /// Maximum depth for recursive reference search
    #[arg(short = 'd', long, default_value_t = 3)]
    max_depth: usize,

    /// Rate limit delay between requests in seconds
    #[arg(short, long, default_value_t = 3)]
    rate_limit_secs: u64,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    if let Err(e) = validation::validate_arxiv_id(&cli.paper_id) {
        error!(error = %e, "Invalid paper ID");
        std::process::exit(1);
    }

    let client = utils::create_http_client();
    let mut downloader = data_aggregation::html_parser::ArxivHTMLDownloader::new(client)
        .with_rate_limit(std::time::Duration::from_secs(cli.rate_limit_secs));

    let arxiv_paper = match data_aggregation::arxiv_api::get_paper_by_id(&cli.paper_id).await {
        Ok(paper) => paper,
        Err(e) => {
            error!(error = %e, "Failed to fetch seed paper");
            std::process::exit(1);
        }
    };

    let paper = match Paper::from_arxiv_paper(&arxiv_paper) {
        Ok(p) => p,
        Err(e) => {
            error!(error = %e, "Failed to parse seed paper");
            std::process::exit(1);
        }
    };

    let referenced_papers = data_aggregation::arxiv_utils::recursive_paper_search_by_references(
        &paper.id,
        cli.max_depth,
        &mut downloader,
    )
    .await;

    info!(
        count = referenced_papers.len(),
        "Recursive search completed"
    );

    let paper_graph = data_processing::graph_creation::create_graph_from_papers(&referenced_papers);
    info!(
        nodes = paper_graph.node_count(),
        edges = paper_graph.edge_count(),
        "Graph created"
    );

    let graph_without_weights = paper_graph.map(|_, _| (), |_, _| ());

    let native_options = eframe::NativeOptions::default();

    run_native(
        "Paper graph interactive",
        native_options,
        Box::new(|cc| Ok(Box::new(DemoApp::new(cc, graph_without_weights)))),
    )
    .expect("failed to launch GUI");
}
