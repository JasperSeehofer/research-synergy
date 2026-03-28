use clap::{Parser, Subcommand};
use resyn_server::commands;
use resyn_server::commands::{analyze::AnalyzeArgs, crawl::CrawlArgs, serve::ServeArgs};

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
        Commands::Serve(args) => commands::serve::run(args).await,
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
