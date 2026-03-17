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
        use resyn_core::database::queries::{LlmAnnotationRepository, PaperRepository};
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
        Ok(PaperDetail { paper, annotation })
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

/// Start a new crawl (placeholder stub — wired to CrawlQueue in Plan 06 Task 2).
///
/// TODO: Wire to CrawlQueue in Plan 06 Task 2.
#[server(StartCrawl, "/api")]
pub async fn start_crawl(
    _paper_id: String,
    _max_depth: usize,
    _source: String,
) -> Result<String, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        // TODO: Wire to CrawlQueue in Plan 06 Task 2.
        Ok("Crawl started.".to_string())
    }
    #[cfg(not(feature = "ssr"))]
    unreachable!()
}
