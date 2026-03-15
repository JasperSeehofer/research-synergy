use crate::data_aggregation::search_query_handler;
use crate::error::ResynError;
use arxiv::{Arxiv, ArxivQueryBuilder};
use tracing::{debug, warn};

pub async fn get_papers(
    search_query_handler: search_query_handler::SearchQueryHandler,
    start: i32,
    max_results: i32,
    sort_by: &str,
    sort_order: &str,
) -> Result<Vec<Arxiv>, ResynError> {
    let query = ArxivQueryBuilder::new()
        .search_query(
            search_query_handler
                .get_arxiv_search_query_string()
                .as_str(),
        )
        .start(start)
        .max_results(max_results)
        .sort_by(sort_by)
        .sort_order(sort_order)
        .build();
    debug!("arXiv query URL: {}", query.to_url());
    arxiv::fetch_arxivs(query).await.map_err(|e| {
        warn!("arXiv API fetch failed: {e}");
        ResynError::ArxivApi(e.to_string())
    })
}

pub async fn get_paper_by_id(id: &str) -> Result<Arxiv, ResynError> {
    let query = ArxivQueryBuilder::new().id_list(id).build();
    let arxivs = arxiv::fetch_arxivs(query).await.map_err(|e| {
        warn!("arXiv API fetch failed for ID {id}: {e}");
        ResynError::ArxivApi(e.to_string())
    })?;
    arxivs
        .into_iter()
        .next()
        .ok_or_else(|| ResynError::PaperNotFound(id.to_string()))
}
