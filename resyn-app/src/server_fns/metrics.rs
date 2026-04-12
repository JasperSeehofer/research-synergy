use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// Entry for the "Most Influential Papers" ranked list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankedPaperEntry {
    pub arxiv_id: String,
    pub title: String,
    pub year: String,
    pub pagerank: f32,
}

/// Status of metrics computation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MetricsStatus {
    Ready { corpus_fingerprint: String },
    NotAvailable,
}

/// Return the top papers ranked by PageRank score.
///
/// Returns an empty vec when no metrics have been computed yet.
#[server(GetTopPageRankPapers, "/api")]
pub async fn get_top_pagerank_papers(limit: usize) -> Result<Vec<RankedPaperEntry>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use resyn_core::database::queries::{GraphMetricsRepository, PaperRepository};
        use std::sync::Arc;

        let db = use_context::<Arc<resyn_core::database::client::Db>>()
            .ok_or_else(|| ServerFnError::new("Database not available"))?;
        let metrics_repo = GraphMetricsRepository::new(&db);
        let paper_repo = PaperRepository::new(&db);

        let top = metrics_repo
            .get_top_by_pagerank(limit)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        let mut entries = Vec::new();
        for m in top {
            if let Ok(Some(paper)) = paper_repo.get_paper(&m.arxiv_id).await {
                let year = if paper.published.len() >= 4 {
                    paper.published[..4].to_string()
                } else {
                    String::new()
                };
                entries.push(RankedPaperEntry {
                    arxiv_id: m.arxiv_id.clone(),
                    title: paper.title.clone(),
                    year,
                    pagerank: m.pagerank,
                });
            }
        }
        Ok(entries)
    }
    #[cfg(not(feature = "ssr"))]
    unreachable!()
}

/// Check whether graph metrics have been computed for the current corpus.
#[server(GetMetricsStatus, "/api")]
pub async fn get_metrics_status() -> Result<MetricsStatus, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use resyn_core::database::queries::GraphMetricsRepository;
        use std::sync::Arc;

        let db = use_context::<Arc<resyn_core::database::client::Db>>()
            .ok_or_else(|| ServerFnError::new("Database not available"))?;
        let repo = GraphMetricsRepository::new(&db);

        // Cheap probe: check if any metrics exist
        let top = repo
            .get_top_by_pagerank(1)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        match top.first() {
            Some(m) => Ok(MetricsStatus::Ready {
                corpus_fingerprint: m.corpus_fingerprint.clone(),
            }),
            None => Ok(MetricsStatus::NotAvailable),
        }
    }
    #[cfg(not(feature = "ssr"))]
    unreachable!()
}

/// Fetch all computed metrics as a flat list for client-side lookup.
///
/// Used by the "Size by" dropdown to populate `GraphState.metrics` for node sizing.
/// Returns an empty vec when no metrics have been computed yet.
#[server(GetAllMetrics, "/api")]
pub async fn get_all_metrics() -> Result<Vec<RankedPaperEntry>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use resyn_core::database::queries::GraphMetricsRepository;
        use std::sync::Arc;

        let db = use_context::<Arc<resyn_core::database::client::Db>>()
            .ok_or_else(|| ServerFnError::new("Database not available"))?;
        let metrics_repo = GraphMetricsRepository::new(&db);

        let all = metrics_repo
            .get_all_metrics()
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        Ok(all
            .into_iter()
            .map(|m| RankedPaperEntry {
                arxiv_id: m.arxiv_id,
                title: String::new(),
                year: String::new(),
                pagerank: m.pagerank,
            })
            .collect())
    }
    #[cfg(not(feature = "ssr"))]
    unreachable!()
}

/// Fetch all computed betweenness centrality scores.
///
/// Separate from `get_all_metrics` to avoid a heavy join; betweenness is only
/// needed when SizeMode::Betweenness is active.
#[server(GetAllBetweenness, "/api")]
pub async fn get_all_betweenness() -> Result<Vec<(String, f32)>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use resyn_core::database::queries::GraphMetricsRepository;
        use std::sync::Arc;

        let db = use_context::<Arc<resyn_core::database::client::Db>>()
            .ok_or_else(|| ServerFnError::new("Database not available"))?;
        let metrics_repo = GraphMetricsRepository::new(&db);

        let all = metrics_repo
            .get_all_metrics()
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        Ok(all
            .into_iter()
            .map(|m| (m.arxiv_id, m.betweenness))
            .collect())
    }
    #[cfg(not(feature = "ssr"))]
    unreachable!()
}

/// Fetch all computed metrics (both PageRank and betweenness) as (arxiv_id, pagerank, betweenness) tuples.
///
/// Used to populate the `GraphState.metrics` HashMap when metrics become available.
#[server(GetMetricsPairs, "/api")]
pub async fn get_metrics_pairs() -> Result<Vec<(String, f32, f32)>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use resyn_core::database::queries::GraphMetricsRepository;
        use std::sync::Arc;

        let db = use_context::<Arc<resyn_core::database::client::Db>>()
            .ok_or_else(|| ServerFnError::new("Database not available"))?;
        let metrics_repo = GraphMetricsRepository::new(&db);

        let all = metrics_repo
            .get_all_metrics()
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        Ok(all
            .into_iter()
            .map(|m| (m.arxiv_id, m.pagerank, m.betweenness))
            .collect())
    }
    #[cfg(not(feature = "ssr"))]
    unreachable!()
}

/// Trigger background computation of PageRank and betweenness centrality.
///
/// Returns immediately. The computation runs asynchronously in a spawned task.
/// A corpus fingerprint guard prevents redundant recomputation (T-23-03).
#[server(TriggerMetricsCompute, "/api")]
pub async fn trigger_metrics_compute() -> Result<String, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use std::sync::Arc;

        let db = use_context::<Arc<resyn_core::database::client::Db>>()
            .ok_or_else(|| ServerFnError::new("Database not available"))?;

        let db_clone = db.clone();
        tokio::task::spawn(async move {
            if let Err(e) = compute_and_store_metrics(&db_clone).await {
                tracing::error!("Metrics computation failed: {e}");
            }
        });

        Ok("Computing started".to_string())
    }
    #[cfg(not(feature = "ssr"))]
    unreachable!()
}

/// Compute PageRank and betweenness centrality for all papers and persist to DB.
///
/// Called both from `trigger_metrics_compute` (manual API trigger) and from the
/// analysis pipeline after all stages complete (auto-compute, per D-07).
///
/// Corpus fingerprint caching (D-09): if the set of papers has not changed since
/// the last computation, the function returns early without recomputing.
///
/// Betweenness runs in `spawn_blocking` to avoid blocking the async runtime for
/// large graphs (T-23-05).
#[cfg(feature = "ssr")]
pub async fn compute_and_store_metrics(
    db: &resyn_core::database::client::Db,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use resyn_core::data_processing::graph_creation::create_graph_from_papers;
    use resyn_core::database::queries::{GraphMetricsRepository, PaperRepository};
    use resyn_core::datamodels::graph_metrics::GraphMetrics;
    use resyn_core::graph_analytics::{betweenness, pagerank};
    use resyn_core::nlp::tfidf::corpus_fingerprint;
    use resyn_core::utils::strip_version_suffix;

    let paper_repo = PaperRepository::new(db);
    let metrics_repo = GraphMetricsRepository::new(db);

    let papers = paper_repo.get_all_papers().await?;
    if papers.is_empty() {
        return Ok(());
    }

    // Compute corpus fingerprint using the same approach as similarity/TF-IDF (D-09)
    let ids: Vec<String> = papers.iter().map(|p| strip_version_suffix(&p.id)).collect();
    let fingerprint = corpus_fingerprint(&ids);

    // Early-exit guard: skip if corpus unchanged (T-23-03 DoS mitigation)
    let existing = metrics_repo.get_top_by_pagerank(1).await?;
    if let Some(first) = existing.first() {
        if first.corpus_fingerprint == fingerprint {
            tracing::info!(
                "Graph metrics already computed for current corpus fingerprint, skipping"
            );
            return Ok(());
        }
    }

    let graph = create_graph_from_papers(&papers);

    // PageRank — O(V * iterations), fast enough on async thread
    let pr_scores = pagerank::compute_pagerank(&graph);

    // Betweenness — O(VE), CPU-bound; run in blocking thread pool (T-23-05)
    let graph_clone = graph.clone();
    let bc_scores =
        tokio::task::spawn_blocking(move || betweenness::compute_betweenness(&graph_clone)).await?;

    let now = chrono::Utc::now().to_rfc3339();

    for paper in &papers {
        let id = strip_version_suffix(&paper.id);
        let pr = pr_scores.get(&id).copied().unwrap_or(0.0);
        let bc = bc_scores.get(&id).copied().unwrap_or(0.0);
        let m = GraphMetrics {
            arxiv_id: id.clone(),
            pagerank: pr,
            betweenness: bc,
            corpus_fingerprint: fingerprint.clone(),
            computed_at: now.clone(),
        };
        if let Err(e) = metrics_repo.upsert_metrics(&m).await {
            tracing::error!("Failed to upsert metrics for {id}: {e}");
        }
    }

    tracing::info!(papers = papers.len(), "Graph metrics computed and stored");
    Ok(())
}
