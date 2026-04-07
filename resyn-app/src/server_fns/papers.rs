use leptos::prelude::*;
use resyn_core::datamodels::paper::Paper;
use serde::{Deserialize, Serialize};

/// Summary statistics shown on the dashboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    pub total_papers: usize,
    pub contradiction_count: usize,
    pub bridge_count: usize,
    pub open_problems_count: usize,
    pub method_coverage_pct: f32,
}

/// Paper + optional annotation bundled for the detail drawer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperDetail {
    pub paper: Paper,
    pub annotation: Option<resyn_core::datamodels::llm_annotation::LlmAnnotation>,
    pub extraction: Option<resyn_core::datamodels::extraction::TextExtractionResult>,
}

/// Return all papers in the database.
#[server(GetPapers, "/api")]
pub async fn get_papers() -> Result<Vec<Paper>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use resyn_core::database::queries::PaperRepository;
        let db = use_context::<std::sync::Arc<resyn_core::database::client::Db>>()
            .ok_or_else(|| ServerFnError::new("Database not available"))?;
        let papers = PaperRepository::new(&db)
            .get_all_papers()
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;
        Ok(papers)
    }
    #[cfg(not(feature = "ssr"))]
    unreachable!()
}

/// Return the paper and its LLM annotation (if any) for the given arXiv ID.
#[server(GetPaperDetail, "/api")]
pub async fn get_paper_detail(id: String) -> Result<PaperDetail, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use resyn_core::database::queries::{
            ExtractionRepository, LlmAnnotationRepository, PaperRepository,
        };
        let db = use_context::<std::sync::Arc<resyn_core::database::client::Db>>()
            .ok_or_else(|| ServerFnError::new("Database not available"))?;
        let paper = PaperRepository::new(&db)
            .get_paper(&id)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?
            .ok_or_else(|| ServerFnError::new(format!("Paper not found: {id}")))?;
        let annotations = LlmAnnotationRepository::new(&db)
            .get_all_annotations()
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;
        let annotation = annotations.into_iter().find(|a| a.arxiv_id == id);
        let extraction = ExtractionRepository::new(&db)
            .get_extraction(&id)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;
        Ok(PaperDetail {
            paper,
            annotation,
            extraction,
        })
    }
    #[cfg(not(feature = "ssr"))]
    unreachable!()
}

/// Return aggregated dashboard statistics.
#[server(GetDashboardStats, "/api")]
pub async fn get_dashboard_stats() -> Result<DashboardStats, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use resyn_core::analysis::aggregation::aggregate_open_problems;
        use resyn_core::database::queries::{
            GapFindingRepository, LlmAnnotationRepository, PaperRepository,
        };
        use resyn_core::datamodels::gap_finding::GapType;

        let db = use_context::<std::sync::Arc<resyn_core::database::client::Db>>()
            .ok_or_else(|| ServerFnError::new("Database not available"))?;

        let papers = PaperRepository::new(&db)
            .get_all_papers()
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        let gap_findings = GapFindingRepository::new(&db)
            .get_all_gap_findings()
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        let annotations = LlmAnnotationRepository::new(&db)
            .get_all_annotations()
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        let total_papers = papers.len();
        let contradiction_count = gap_findings
            .iter()
            .filter(|g| g.gap_type == GapType::Contradiction)
            .count();
        let bridge_count = gap_findings
            .iter()
            .filter(|g| g.gap_type == GapType::AbcBridge)
            .count();
        let open_problems_count = aggregate_open_problems(&annotations).len();

        // Method coverage: percentage of papers that have at least one method annotated.
        let papers_with_methods = annotations.iter().filter(|a| !a.methods.is_empty()).count();
        let method_coverage_pct = if total_papers == 0 {
            0.0
        } else {
            (papers_with_methods as f32 / total_papers as f32) * 100.0
        };

        Ok(DashboardStats {
            total_papers,
            contradiction_count,
            bridge_count,
            open_problems_count,
            method_coverage_pct,
        })
    }
    #[cfg(not(feature = "ssr"))]
    unreachable!()
}

/// A single search result returned to the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub arxiv_id: String,
    pub title: String,
    pub authors: Vec<String>,
    pub year: String,
    pub score: f32,
}

/// Full-text search across paper title, abstract, and authors.
/// Returns up to `limit` results ranked by BM25 relevance score.
#[server(SearchPapers, "/api")]
pub async fn search_papers(
    query: String,
    limit: Option<usize>,
) -> Result<Vec<SearchResult>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        if query.trim().is_empty() {
            return Ok(vec![]);
        }
        use resyn_core::database::queries::SearchRepository;
        let db = use_context::<std::sync::Arc<resyn_core::database::client::Db>>()
            .ok_or_else(|| ServerFnError::new("Database not available"))?;
        let max = limit.unwrap_or(10);
        let rows = SearchRepository::new(&db)
            .search_papers(&query, max)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;
        Ok(rows
            .into_iter()
            .map(|r| SearchResult {
                arxiv_id: r.arxiv_id,
                title: r.title,
                authors: r.authors,
                year: if r.published.len() >= 4 {
                    r.published[..4].to_string()
                } else {
                    r.published
                },
                score: r.score,
            })
            .collect())
    }
    #[cfg(not(feature = "ssr"))]
    unreachable!()
}

/// Start a new crawl by seeding the CrawlQueue and launching a background crawl task.
///
/// Implementation:
/// 1. Validates the arXiv paper ID format.
/// 2. Enqueues the seed paper via `CrawlQueueRepository::enqueue_if_absent`.
/// 3. Spawns a background tokio task running the queue-driven crawl loop.
/// 4. Returns immediately — SSE `/progress` endpoint reports live progress.
#[server(StartCrawl, "/api")]
pub async fn start_crawl(
    paper_id: String,
    max_depth: usize,
    source: String,
) -> Result<String, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use resyn_core::database::crawl_queue::CrawlQueueRepository;
        use resyn_core::validation::validate_arxiv_id;

        // Validate paper ID format.
        validate_arxiv_id(&paper_id)
            .map_err(|e| ServerFnError::new(format!("Invalid paper ID: {e}")))?;

        let db = use_context::<std::sync::Arc<resyn_core::database::client::Db>>()
            .ok_or_else(|| ServerFnError::new("Database not available"))?;

        // Seed the crawl queue with the root paper (depth 0).
        let queue = CrawlQueueRepository::new(&db);
        queue
            .enqueue_if_absent(&paper_id, &paper_id, 0)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to enqueue seed paper: {e}")))?;

        // Spawn a background crawl task. This task runs independently after the
        // server function returns. The /progress SSE endpoint broadcasts updates.
        let db_bg = db.clone();
        let paper_id_bg = paper_id.clone();
        tokio::spawn(async move {
            use resyn_core::data_aggregation::arxiv_source::ArxivSource;
            use resyn_core::data_aggregation::html_parser::ArxivHTMLDownloader;
            use resyn_core::data_aggregation::inspirehep_api::InspireHepClient;
            use resyn_core::data_aggregation::rate_limiter::{
                make_arxiv_limiter, make_inspirehep_limiter, wait_for_token,
            };
            use resyn_core::data_aggregation::traits::PaperSource;
            use resyn_core::database::crawl_queue::CrawlQueueRepository;
            use resyn_core::database::queries::PaperRepository;
            use std::sync::Arc;
            use std::time::Duration;
            use tokio::sync::Semaphore;
            use tracing::{info, warn};

            fn make_source_bg(name: &str) -> Box<dyn PaperSource> {
                let client = resyn_core::utils::create_http_client();
                match name {
                    "inspirehep" => Box::new(
                        InspireHepClient::new(client).with_rate_limit(Duration::from_millis(350)),
                    ),
                    _ => {
                        let downloader = ArxivHTMLDownloader::new(client)
                            .with_rate_limit(Duration::from_secs(3));
                        Box::new(ArxivSource::new(downloader))
                    }
                }
            }

            let rate_limiter = if source == "inspirehep" {
                make_inspirehep_limiter()
            } else {
                make_arxiv_limiter()
            };
            let sem = Arc::new(Semaphore::new(4));
            let mut join_set: tokio::task::JoinSet<()> = tokio::task::JoinSet::new();
            let mut retried = false;

            loop {
                let queue = CrawlQueueRepository::new(&db_bg);
                let entry = match queue.claim_next_pending().await {
                    Ok(e) => e,
                    Err(e) => {
                        warn!(error = %e, "claim_next_pending failed in web crawl task");
                        break;
                    }
                };

                let Some(entry) = entry else {
                    if join_set.is_empty() {
                        // No in-flight tasks and no pending entries.
                        if !retried {
                            retried = true;
                            let queue2 = CrawlQueueRepository::new(&db_bg);
                            match queue2.retry_failed().await {
                                Ok(n) if n > 0 => {
                                    info!(count = n, "Retrying failed entries (web crawl)");
                                    continue;
                                }
                                _ => {}
                            }
                        }
                        break;
                    }
                    // In-flight workers may enqueue new references — wait then re-check.
                    while let Some(res) = join_set.join_next().await {
                        if let Err(e) = res {
                            warn!(error = %e, "Worker panicked in web crawl task");
                        }
                    }
                    continue;
                };

                // Skip entries beyond max_depth.
                if entry.depth_level > max_depth {
                    let queue2 = CrawlQueueRepository::new(&db_bg);
                    queue2
                        .mark_done(&entry.paper_id, &entry.seed_paper_id)
                        .await
                        .ok();
                    continue;
                }

                // Skip papers already in DB.
                let paper_repo = PaperRepository::new(&db_bg);
                if paper_repo
                    .paper_exists(&entry.paper_id)
                    .await
                    .unwrap_or(false)
                {
                    let queue2 = CrawlQueueRepository::new(&db_bg);
                    queue2
                        .mark_done(&entry.paper_id, &entry.seed_paper_id)
                        .await
                        .ok();
                    continue;
                }

                let permit = Arc::clone(&sem).acquire_owned().await.unwrap();
                let db_task = db_bg.clone();
                let limiter = Arc::clone(&rate_limiter);
                let src = source.clone();
                let seed_id = paper_id_bg.clone();

                join_set.spawn(async move {
                    let _permit = permit;
                    let mut source_impl = make_source_bg(&src);
                    let queue = CrawlQueueRepository::new(&db_task);
                    let paper_repo = PaperRepository::new(&db_task);

                    wait_for_token(&limiter).await;

                    match source_impl.fetch_paper(&entry.paper_id).await {
                        Ok(mut paper) => {
                            if let Err(e) = source_impl.fetch_references(&mut paper).await {
                                warn!(paper_id = entry.paper_id.as_str(), error = %e, "fetch_references failed");
                            }
                            let ref_ids = paper.get_arxiv_references_ids();
                            for arxiv_id in &ref_ids {
                                if let Err(e) = queue
                                    .enqueue_if_absent(arxiv_id, &seed_id, entry.depth_level + 1)
                                    .await
                                {
                                    warn!(arxiv_id = arxiv_id.as_str(), error = %e, "enqueue ref failed");
                                }
                            }
                            paper_repo.upsert_paper(&paper).await.ok();
                            paper_repo.upsert_citations(&paper).await.ok();
                            queue.mark_done(&entry.paper_id, &seed_id).await.ok();
                        }
                        Err(e) => {
                            warn!(paper_id = entry.paper_id.as_str(), error = %e, "fetch_paper failed");
                            queue.mark_failed(&entry.paper_id, &seed_id).await.ok();
                        }
                    }
                });
            }
        });

        Ok(format!(
            "Crawl started for paper {} (max depth {})",
            paper_id, max_depth
        ))
    }
    #[cfg(not(feature = "ssr"))]
    unreachable!()
}
