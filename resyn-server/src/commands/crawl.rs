use clap::Args;
use resyn_core::data_aggregation::arxiv_source::ArxivSource;
use resyn_core::data_aggregation::inspirehep_api::InspireHepClient;
use resyn_core::data_aggregation::traits::PaperSource;
use tracing::{error, info};

use crate::commands::analyze::{run_analysis_pipeline, AnalyzeArgs};

#[derive(Args, Debug)]
pub struct CrawlArgs {
    /// arXiv paper ID to use as seed (e.g. 2503.18887)
    #[arg(short = 'p', long, default_value = "2503.18887")]
    pub paper_id: String,

    /// Maximum depth for recursive reference search
    #[arg(short = 'd', long, default_value_t = 3)]
    pub max_depth: usize,

    /// Rate limit delay between requests in seconds
    #[arg(short = 'r', long, default_value_t = 3)]
    pub rate_limit_secs: u64,

    /// Data source: "arxiv" or "inspirehep"
    #[arg(long, default_value = "arxiv")]
    pub source: String,

    /// Database connection string (e.g. "surrealkv://./data")
    #[arg(long, default_value = "surrealkv://./data")]
    pub db: String,

    /// Run text extraction and analysis after crawl
    #[arg(long, default_value_t = false)]
    pub analyze: bool,

    /// Skip full-text extraction; all papers use abstract only
    #[arg(long, default_value_t = false)]
    pub skip_fulltext: bool,

    /// LLM provider for semantic extraction: claude, ollama, noop
    #[arg(long)]
    pub llm_provider: Option<String>,

    /// LLM model override (e.g. claude-sonnet-4-20250514, llama3.2)
    #[arg(long)]
    pub llm_model: Option<String>,

    /// Expand ABC-bridge scope to all papers in SurrealDB (not just current crawl)
    #[arg(long, default_value_t = false)]
    pub full_corpus: bool,

    /// Show full justifications in gap output table (default: truncated at 60 chars)
    #[arg(long, default_value_t = false)]
    pub verbose: bool,
}

pub async fn run(args: CrawlArgs) -> anyhow::Result<()> {
    if let Err(e) = resyn_core::validation::validate_arxiv_id(&args.paper_id) {
        error!(error = %e, "Invalid paper ID");
        std::process::exit(1);
    }

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

    let client = resyn_core::utils::create_http_client();
    let mut source: Box<dyn PaperSource> = match args.source.as_str() {
        "inspirehep" => {
            let inspire_client = InspireHepClient::new(client);
            Box::new(inspire_client)
        }
        _ => {
            let downloader =
                resyn_core::data_aggregation::html_parser::ArxivHTMLDownloader::new(client)
                    .with_rate_limit(std::time::Duration::from_secs(args.rate_limit_secs));
            Box::new(ArxivSource::new(downloader))
        }
    };

    let papers = resyn_core::data_aggregation::arxiv_utils::recursive_paper_search_by_references(
        &args.paper_id,
        args.max_depth,
        source.as_mut(),
    )
    .await;

    info!(count = papers.len(), "Recursive search completed");

    let repo = resyn_core::database::queries::PaperRepository::new(&db);
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

    if args.analyze {
        run_analysis_pipeline(
            &db,
            AnalyzeArgs {
                db: args.db.clone(),
                llm_provider: args.llm_provider.clone(),
                llm_model: args.llm_model.clone(),
                force: false,
                full_corpus: args.full_corpus,
                verbose: args.verbose,
            },
            args.rate_limit_secs,
            args.skip_fulltext,
        )
        .await?;
    }

    Ok(())
}
