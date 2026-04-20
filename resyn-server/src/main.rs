use clap::{Parser, Subcommand};
use resyn_server::commands;
use resyn_server::commands::{
    analyze::AnalyzeArgs, bulk_ingest::BulkIngestArgs, crawl::CrawlArgs, export::ExportArgs,
    serve::ServeArgs,
};

#[derive(Parser, Debug)]
#[command(
    name = "resyn",
    about = "Research Synergy - Literature Based Discovery"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Crawl arXiv papers starting from a seed paper ID
    Crawl(CrawlArgs),
    /// Run analysis pipeline on papers already in the database
    Analyze(AnalyzeArgs),
    /// Export the Louvain community graph to JSON for external tooling (e.g. Kuramoto-LBD)
    ExportLouvainGraph(ExportArgs),
    /// Bulk-ingest papers from OpenAlex REST API into a local SurrealDB
    BulkIngest(BulkIngestArgs),
    /// Start the web server (not yet implemented)
    Serve(ServeArgs),
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Crawl(args) => commands::crawl::run(args).await,
        Commands::Analyze(args) => commands::analyze::run(args).await,
        Commands::ExportLouvainGraph(args) => commands::export::run(args).await,
        Commands::BulkIngest(args) => commands::bulk_ingest::run(args).await,
        Commands::Serve(args) => commands::serve::run(args).await,
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
