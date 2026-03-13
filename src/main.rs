#[allow(dead_code)]
mod data_aggregation;
#[allow(dead_code)]
mod data_processing;
#[allow(dead_code)]
mod database;
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
use data_aggregation::arxiv_source::ArxivSource;
use data_aggregation::inspirehep_api::InspireHepClient;
use data_aggregation::traits::PaperSource;
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

    /// Data source: "arxiv" or "inspirehep"
    #[arg(long, default_value = "arxiv")]
    source: String,

    /// Database connection string (e.g. "mem://", "surrealkv://./data")
    #[arg(long)]
    db: Option<String>,

    /// Skip crawling, load graph from database only
    #[arg(long, default_value_t = false)]
    db_only: bool,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    if let Err(e) = validation::validate_arxiv_id(&cli.paper_id) {
        error!(error = %e, "Invalid paper ID");
        std::process::exit(1);
    }

    // Connect to database if requested
    let db = if let Some(ref db_endpoint) = cli.db {
        match database::client::connect(db_endpoint).await {
            Ok(db) => {
                info!(endpoint = db_endpoint.as_str(), "Connected to database");
                Some(db)
            }
            Err(e) => {
                error!(error = %e, "Failed to connect to database");
                std::process::exit(1);
            }
        }
    } else {
        None
    };

    // If db-only mode, load from database and skip crawling
    if cli.db_only {
        let Some(ref db) = db else {
            error!("--db-only requires --db to be specified");
            std::process::exit(1);
        };
        let repo = database::queries::PaperRepository::new(db);
        let (papers, _edges) = match repo.get_citation_graph(&cli.paper_id, cli.max_depth).await {
            Ok(result) => result,
            Err(e) => {
                error!(error = %e, "Failed to load graph from database");
                std::process::exit(1);
            }
        };
        info!(count = papers.len(), "Loaded papers from database");
        launch_visualization(&papers);
        return;
    }

    // Create the appropriate paper source
    let client = utils::create_http_client();
    let mut source: Box<dyn PaperSource> = match cli.source.as_str() {
        "inspirehep" => {
            let inspire_client = InspireHepClient::new(client);
            Box::new(inspire_client)
        }
        _ => {
            let downloader = data_aggregation::html_parser::ArxivHTMLDownloader::new(client)
                .with_rate_limit(std::time::Duration::from_secs(cli.rate_limit_secs));
            Box::new(ArxivSource::new(downloader))
        }
    };

    // Run the BFS crawler
    let papers = data_aggregation::arxiv_utils::recursive_paper_search_by_references(
        &cli.paper_id,
        cli.max_depth,
        source.as_mut(),
    )
    .await;

    info!(count = papers.len(), "Recursive search completed");

    // Persist to database if connected
    if let Some(ref db) = db {
        let repo = database::queries::PaperRepository::new(db);
        for paper in &papers {
            if let Err(e) = repo.upsert_paper(paper).await {
                error!(paper_id = paper.id, error = %e, "Failed to upsert paper");
            }
        }
        for paper in &papers {
            if let Err(e) = repo.upsert_citations(paper).await {
                error!(paper_id = paper.id, error = %e, "Failed to upsert citations");
            }
        }
        info!(count = papers.len(), "Persisted papers to database");
    }

    launch_visualization(&papers);
}

fn launch_visualization(papers: &[Paper]) {
    let paper_graph = data_processing::graph_creation::create_graph_from_papers(papers);
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
