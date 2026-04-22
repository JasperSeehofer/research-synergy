use clap::Args;
use std::collections::HashMap;
use std::time::Duration;
use tracing::info;

use resyn_core::data_aggregation::openalex_bulk::OpenAlexBulkLoader;
use resyn_core::database::queries::PaperRepository;

const DEFAULT_FILTER: &str =
    "primary_location.source.id:S4306400194,concepts.id:C154945302|C121332964|C41008148";

/// Physics/cond-mat corpus filter: arXiv papers in Condensed matter physics (C26873012)
/// or Statistical physics (C121864883). Use with --db surrealkv://./data-physics.
#[allow(dead_code)]
const DEFAULT_FILTER_PHYSICS: &str =
    "primary_location.source.id:S4306400194,concepts.id:C26873012|C121864883";

#[derive(Args, Debug)]
pub struct BulkIngestArgs {
    /// Database connection string
    #[arg(long, default_value = "surrealkv://./data-openalex")]
    pub db: String,

    /// OpenAlex filter expression.
    /// Default covers arXiv ML papers (Machine Learning + stat.ML + Neural Networks).
    /// For cond-mat/stat-phys corpus use DEFAULT_FILTER_PHYSICS or pass --filter directly.
    #[arg(long, default_value = DEFAULT_FILTER)]
    pub filter: String,

    /// OpenAlex API key — required since 2026-02-13 (free at openalex.org/settings/api).
    /// Can also be set via OPENALEX_API_KEY environment variable.
    #[arg(long, env = "OPENALEX_API_KEY")]
    pub api_key: Option<String>,

    /// Delay between API pages in milliseconds (default 100 = 10 req/s)
    #[arg(long, default_value_t = 100)]
    pub page_delay_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    // Wrapping CLI for argument parsing tests
    #[derive(clap::Parser)]
    struct Cli {
        #[command(flatten)]
        args: BulkIngestArgs,
    }

    #[test]
    fn test_api_key_flag_parsed() {
        let cli = Cli::parse_from(["test", "--api-key", "sk-test-123"]);
        assert_eq!(cli.args.api_key.as_deref(), Some("sk-test-123"));
    }

    #[test]
    fn test_no_api_key_is_none() {
        let cli = Cli::parse_from(["test"]);
        assert!(cli.args.api_key.is_none());
    }
}

pub async fn run(args: BulkIngestArgs) -> anyhow::Result<()> {
    let api_key = match args.api_key {
        Some(ref k) if !k.is_empty() => k.clone(),
        _ => {
            tracing::error!(
                "OPENALEX_API_KEY not set — register a free key at openalex.org/settings/api"
            );
            std::process::exit(1);
        }
    };

    let db = match resyn_core::database::client::connect(&args.db).await {
        Ok(db) => {
            info!(endpoint = args.db.as_str(), "Connected to database");
            db
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to connect to database");
            std::process::exit(1);
        }
    };

    let client = resyn_core::utils::create_http_client();
    let loader = OpenAlexBulkLoader::new(client, &api_key);
    let repo = PaperRepository::new(&db);

    info!(filter = args.filter.as_str(), "Starting OpenAlex bulk ingest");

    // Phase 1: stream all pages, upsert papers, build id_map and raw citation list.
    // openalex_id (full URL) → arxiv_id
    let mut id_map: HashMap<String, String> = HashMap::new();
    // (from_arxiv_id, Vec<openalex_ref_ids>)
    let mut raw_citations: Vec<(String, Vec<String>)> = Vec::new();
    let mut papers_upserted = 0usize;
    let mut cursor = "*".to_string();
    let mut page_num = 0u64;

    loop {
        let page = loader.fetch_page(&args.filter, &cursor).await?;
        if page.results.is_empty() {
            break;
        }

        let mut batch = Vec::with_capacity(page.results.len());
        for work in &page.results {
            if let Some(arxiv_id) = work.arxiv_id() {
                id_map.insert(work.id.clone(), arxiv_id.clone());
                if let Some(paper) = work.to_paper() {
                    batch.push(paper);
                }
                if !work.referenced_works.is_empty() {
                    raw_citations.push((arxiv_id, work.referenced_works.clone()));
                }
            }
        }

        let batch_size = batch.len();
        repo.upsert_papers_batch(&batch).await?;
        papers_upserted += batch_size;

        page_num += 1;
        if page_num % 10 == 0 || papers_upserted % 1000 < batch_size {
            info!(
                papers = papers_upserted,
                pages = page_num,
                "Ingest progress"
            );
        }

        match page.meta.next_cursor {
            Some(ref c) if !c.is_empty() => cursor = c.clone(),
            _ => break,
        }

        if args.page_delay_ms > 0 {
            tokio::time::sleep(Duration::from_millis(args.page_delay_ms)).await;
        }
    }

    info!(
        papers = papers_upserted,
        pending_citation_batches = raw_citations.len(),
        "Phase 1 complete — starting citation translation"
    );

    // Phase 2: translate OpenAlex IDs to arXiv IDs and batch-insert citation edges.
    let mut citations_upserted = 0usize;
    for (from_arxiv, openalex_refs) in &raw_citations {
        let pairs: Vec<(String, String)> = openalex_refs
            .iter()
            .filter_map(|oa_id| id_map.get(oa_id).map(|to| (from_arxiv.clone(), to.clone())))
            .collect();
        if !pairs.is_empty() {
            citations_upserted += pairs.len();
            repo.upsert_citations_batch(&pairs).await?;
        }
    }

    info!(
        papers = papers_upserted,
        citations = citations_upserted,
        "Bulk ingest complete"
    );

    Ok(())
}
