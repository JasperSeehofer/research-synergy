use clap::{Args, Subcommand};
use resyn_core::data_aggregation::arxiv_source::ArxivSource;
use resyn_core::data_aggregation::chained_source::ChainedPaperSource;
use resyn_core::data_aggregation::html_parser::ArxivHTMLDownloader;
use resyn_core::data_aggregation::inspirehep_api::InspireHepClient;
use resyn_core::data_aggregation::rate_limiter::{
    SharedRateLimiter, make_arxiv_limiter, make_inspirehep_limiter,
    make_semantic_scholar_limiter, wait_for_token,
};
use resyn_core::data_aggregation::semantic_scholar_api::SemanticScholarSource;
use resyn_core::data_aggregation::traits::PaperSource;
use resyn_core::database::crawl_queue::CrawlQueueRepository;
use resyn_core::database::queries::PaperRepository;
use resyn_core::datamodels::progress::ProgressEvent;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::sync::Semaphore;
use tracing::{error, info, warn};

use crate::commands::analyze::{AnalyzeArgs, run_analysis_pipeline};

/// Queue management subcommands for `resyn crawl`
#[derive(Subcommand, Debug)]
pub enum CrawlSubcommand {
    /// Show crawl queue summary (pending/done/failed counts)
    Status {
        /// Database connection string (e.g. "surrealkv://./data")
        #[arg(long, default_value = "surrealkv://./data")]
        db: String,
    },
    /// Clear the crawl queue entirely
    Clear {
        /// Database connection string (e.g. "surrealkv://./data")
        #[arg(long, default_value = "surrealkv://./data")]
        db: String,
    },
    /// Mark all failed entries as pending for retry
    Retry {
        /// Database connection string (e.g. "surrealkv://./data")
        #[arg(long, default_value = "surrealkv://./data")]
        db: String,
    },
}

#[derive(Args, Debug)]
pub struct CrawlArgs {
    /// arXiv paper ID to use as seed (e.g. 2503.18887)
    #[arg(short = 'p', long, default_value = "2503.18887")]
    pub paper_id: String,

    /// Maximum depth for recursive reference search
    #[arg(short = 'd', long, default_value_t = 3)]
    pub max_depth: usize,

    /// Enable parallel crawling; optional value sets max concurrency (default: 4)
    #[arg(long, num_args = 0..=1, default_missing_value = "4")]
    pub parallel: Option<usize>,

    /// Data source: "arxiv", "inspirehep", "semantic_scholar", or a comma-separated
    /// fallback chain e.g. "arxiv,inspirehep,semantic_scholar" (tried left-to-right per paper)
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

    /// Start SSE progress server on this port (default: 3001 if flag given without value)
    #[arg(long, num_args = 0..=1, default_missing_value = "3001")]
    pub progress: Option<u16>,

    /// Queue management subcommand (status, clear, retry)
    #[command(subcommand)]
    pub subcmd: Option<CrawlSubcommand>,
}

fn make_single_source(name: &str) -> Box<dyn PaperSource> {
    let client = resyn_core::utils::create_http_client();
    match name.trim() {
        "inspirehep" => {
            let inspire = InspireHepClient::new(client).with_rate_limit(Duration::ZERO);
            Box::new(inspire)
        }
        "semantic_scholar" => {
            // Internal rate limit zeroed: the external SharedRateLimiter (2200 ms
            // unkeyed / 400 ms keyed) covers both fetch_paper + fetch_references
            // per paper to stay within S2's 1 rps unauthenticated limit.
            let s2 = SemanticScholarSource::from_env(client).with_rate_limit(Duration::ZERO);
            Box::new(s2)
        }
        _ => {
            let downloader = ArxivHTMLDownloader::new(client).with_rate_limit(Duration::ZERO);
            Box::new(ArxivSource::new(downloader))
        }
    }
}

/// Build a `PaperSource` from a (possibly comma-separated) source spec.
/// A single name returns that source directly; multiple names return a `ChainedPaperSource`.
fn make_source(source_spec: &str) -> Box<dyn PaperSource> {
    let parts: Vec<&str> = source_spec.split(',').collect();
    if parts.len() == 1 {
        make_single_source(parts[0])
    } else {
        let sources = parts.into_iter().map(make_single_source).collect();
        Box::new(ChainedPaperSource::new(sources))
    }
}

/// Pick the external rate limiter for a source spec.
/// For chains, use the first source's rate — primary source drives the crawl cadence.
fn make_rate_limiter_for_source(source_spec: &str) -> SharedRateLimiter {
    let first = source_spec.split(',').next().unwrap_or("arxiv").trim();
    match first {
        "inspirehep" => make_inspirehep_limiter(),
        "semantic_scholar" => make_semantic_scholar_limiter(),
        _ => make_arxiv_limiter(),
    }
}

pub async fn run(args: CrawlArgs) -> anyhow::Result<()> {
    // Handle queue management subcommands before any crawl logic.
    if let Some(subcmd) = &args.subcmd {
        let db_str = match subcmd {
            CrawlSubcommand::Status { db } => db,
            CrawlSubcommand::Clear { db } => db,
            CrawlSubcommand::Retry { db } => db,
        };
        let db = match resyn_core::database::client::connect(db_str).await {
            Ok(db) => {
                info!(endpoint = db_str.as_str(), "Connected to database");
                db
            }
            Err(e) => {
                error!(error = %e, "Failed to connect to database");
                std::process::exit(1);
            }
        };
        let queue = CrawlQueueRepository::new(&db);
        match subcmd {
            CrawlSubcommand::Status { .. } => {
                let counts = queue.get_counts().await?;
                println!("Crawl Queue Status:");
                println!("  Total:    {}", counts.total);
                println!("  Pending:  {}", counts.pending);
                println!("  Fetching: {}", counts.fetching);
                println!("  Done:     {}", counts.done);
                println!("  Failed:   {}", counts.failed);
            }
            CrawlSubcommand::Clear { .. } => {
                queue.clear_queue().await?;
                println!("Crawl queue cleared.");
            }
            CrawlSubcommand::Retry { .. } => {
                let count = queue.retry_failed().await?;
                println!("Marked {count} failed entries as pending for retry.");
            }
        }
        return Ok(());
    }

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

    let queue_repo = CrawlQueueRepository::new(&db);
    let paper_repo = PaperRepository::new(&db);

    // Crash recovery: reset stale 'fetching' entries from a previous interrupted run.
    let reset_count = queue_repo.reset_stale_fetching().await?;
    if reset_count > 0 {
        info!(
            count = reset_count,
            "Reset stale fetching entries to pending (crash recovery)"
        );
    }

    // Resume from existing queue or enqueue seed.
    let pending = queue_repo.pending_count().await?;
    if pending > 0 {
        info!(pending, "Resuming crawl from existing queue");
    } else {
        info!(paper_id = args.paper_id.as_str(), "Enqueuing seed paper");
        queue_repo
            .enqueue_if_absent(&args.paper_id, &args.paper_id, 0)
            .await?;
    }

    // Shared crawl resources.
    let rate_limiter: SharedRateLimiter = make_rate_limiter_for_source(&args.source);
    let concurrency = args.parallel.unwrap_or(4);
    let sem = Arc::new(Semaphore::new(concurrency));
    let (progress_tx, _) = tokio::sync::broadcast::channel::<ProgressEvent>(256);
    let start = std::time::Instant::now();
    let papers_found = Arc::new(AtomicU64::new(0));
    let papers_failed = Arc::new(AtomicU64::new(0));

    // Start SSE progress server if --progress flag was given.
    if let Some(port) = args.progress {
        let tx_for_sse = progress_tx.clone();
        tokio::spawn(async move {
            use axum::Router;
            use axum::extract::State;
            use axum::response::sse::{Event, KeepAlive, Sse};
            use axum::routing::get;
            use futures::StreamExt;
            use tokio_stream::wrappers::BroadcastStream;

            async fn sse_handler(
                State(tx): State<tokio::sync::broadcast::Sender<ProgressEvent>>,
            ) -> Sse<impl futures::Stream<Item = Result<Event, std::convert::Infallible>>>
            {
                let rx = tx.subscribe();
                let stream = BroadcastStream::new(rx).filter_map(|msg| async move {
                    msg.ok().and_then(|e| {
                        serde_json::to_string(&e)
                            .ok()
                            .map(|data| Ok(Event::default().data(data)))
                    })
                });
                Sse::new(stream)
                    .keep_alive(KeepAlive::new().interval(std::time::Duration::from_secs(5)))
            }

            let app = Router::new()
                .route("/progress", get(sse_handler))
                .with_state(tx_for_sse);

            let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}"))
                .await
                .expect("Failed to bind SSE server");
            tracing::info!(port, "SSE progress server started");
            axum::serve(listener, app).await.ok();
        });
    }

    info!(
        concurrency,
        source = args.source.as_str(),
        "Starting queue-driven crawl"
    );

    let mut join_set: tokio::task::JoinSet<()> = tokio::task::JoinSet::new();
    let mut retried = false;

    loop {
        let entry = queue_repo.claim_next_pending().await?;

        let Some(entry) = entry else {
            // No pending — wait for all in-flight tasks to finish.
            while let Some(res) = join_set.join_next().await {
                if let Err(e) = res {
                    warn!(error = %e, "Worker task panicked");
                }
            }

            // One automatic retry of failed entries.
            if !retried {
                retried = true;
                let retry_count = queue_repo.retry_failed().await?;
                if retry_count > 0 {
                    info!(count = retry_count, "Retrying failed entries");
                    continue; // re-enter loop to process retried entries
                }
            }

            break;
        };

        // Skip entries beyond max_depth.
        if entry.depth_level > args.max_depth {
            queue_repo
                .mark_done(&entry.paper_id, &entry.seed_paper_id)
                .await?;
            continue;
        }

        // Skip if paper already in DB (references were already enqueued in a prior run).
        if paper_repo
            .paper_exists(&entry.paper_id)
            .await
            .unwrap_or(false)
        {
            queue_repo
                .mark_done(&entry.paper_id, &entry.seed_paper_id)
                .await?;
            papers_found.fetch_add(1, Ordering::Relaxed);
            continue;
        }

        // Acquire a semaphore permit before spawning to bound concurrency.
        let permit = Arc::clone(&sem).acquire_owned().await.unwrap();
        let db_clone = db.clone();
        let limiter = Arc::clone(&rate_limiter);
        let tx = progress_tx.clone();
        let seed_id = args.paper_id.clone();
        let source_name = args.source.clone();
        let found_counter = Arc::clone(&papers_found);
        let failed_counter = Arc::clone(&papers_failed);
        let max_depth = args.max_depth;
        let elapsed_at_spawn = start.elapsed().as_secs_f64();

        join_set.spawn(async move {
            let _permit = permit;

            let mut source = make_source(&source_name);
            let queue = CrawlQueueRepository::new(&db_clone);
            let paper_repo_task = PaperRepository::new(&db_clone);

            // Rate limit before fetch.
            wait_for_token(&limiter).await;

            match source.fetch_paper(&entry.paper_id).await {
                Ok(mut paper) => {
                    let title = if paper.title.is_empty() {
                        None
                    } else {
                        Some(paper.title.clone())
                    };

                    // Fetch references into the paper (mutates paper.references).
                    if let Err(e) = source.fetch_references(&mut paper).await {
                        warn!(
                            paper_id = entry.paper_id.as_str(),
                            error = %e,
                            "Failed to fetch references"
                        );
                    }

                    // Enqueue all discovered arXiv references (depth filter happens at claim time).
                    let ref_ids = paper.get_arxiv_references_ids();
                    for arxiv_id in &ref_ids {
                        if let Err(e) = queue
                            .enqueue_if_absent(arxiv_id, &seed_id, entry.depth_level + 1)
                            .await
                        {
                            warn!(
                                arxiv_id = arxiv_id.as_str(),
                                error = %e,
                                "Failed to enqueue reference"
                            );
                        }
                    }

                    if let Err(e) = paper_repo_task.upsert_paper(&paper).await {
                        warn!(
                            paper_id = entry.paper_id.as_str(),
                            error = %e,
                            "Failed to upsert paper"
                        );
                    }
                    if let Err(e) = paper_repo_task.upsert_citations(&paper).await {
                        warn!(
                            paper_id = entry.paper_id.as_str(),
                            error = %e,
                            "Failed to upsert citations"
                        );
                    }

                    let found = found_counter.fetch_add(1, Ordering::Relaxed) + 1;
                    queue
                        .mark_done_with_source(
                            &entry.paper_id,
                            &seed_id,
                            source.last_resolving_source(),
                        )
                        .await
                        .ok();

                    let _ = tx.send(ProgressEvent {
                        event_type: "paper_fetched".to_string(),
                        papers_found: found,
                        papers_pending: 0,
                        papers_failed: failed_counter.load(Ordering::Relaxed),
                        current_depth: entry.depth_level,
                        max_depth,
                        elapsed_secs: elapsed_at_spawn,
                        current_paper_id: Some(entry.paper_id.clone()),
                        current_paper_title: title,
                        analysis_stage: None,
                    });
                }
                Err(e) => {
                    warn!(
                        paper_id = entry.paper_id.as_str(),
                        error = %e,
                        "Failed to fetch paper"
                    );
                    let failed = failed_counter.fetch_add(1, Ordering::Relaxed) + 1;
                    queue.mark_failed(&entry.paper_id, &seed_id).await.ok();

                    let _ = tx.send(ProgressEvent {
                        event_type: "paper_failed".to_string(),
                        papers_found: found_counter.load(Ordering::Relaxed),
                        papers_pending: 0,
                        papers_failed: failed,
                        current_depth: entry.depth_level,
                        max_depth,
                        elapsed_secs: elapsed_at_spawn,
                        current_paper_id: Some(entry.paper_id.clone()),
                        current_paper_title: None,
                        analysis_stage: None,
                    });
                }
            }
        });
    }

    // Final stats.
    let elapsed = start.elapsed();
    let found = papers_found.load(Ordering::Relaxed);
    let failed = papers_failed.load(Ordering::Relaxed);
    info!(
        papers_found = found,
        papers_failed = failed,
        elapsed_secs = elapsed.as_secs_f64(),
        "Crawl complete"
    );

    // Broadcast completion event.
    let _ = progress_tx.send(ProgressEvent {
        event_type: "complete".to_string(),
        papers_found: found,
        papers_pending: 0,
        papers_failed: failed,
        current_depth: 0,
        max_depth: args.max_depth,
        elapsed_secs: elapsed.as_secs_f64(),
        current_paper_id: None,
        current_paper_title: None,
        analysis_stage: None,
    });

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
            3,
            args.skip_fulltext,
        )
        .await?;
    }

    Ok(())
}
