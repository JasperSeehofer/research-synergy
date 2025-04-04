use crate::data_aggregation::search_query_handler;
use crate::paper;
use arxiv::{Arxiv, ArxivQueryBuilder};

#[tokio::main]
pub async fn get_papers(
    search_query_handler: search_query_handler::SearchQueryHandler,
    start: i32,
    max_results: i32,
    sort_by: &str,
    sort_order: &str,
) -> Vec<Arxiv> {
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
    println!("{}", query.to_url());
    arxiv::fetch_arxivs(query)
        .await
        .inspect(|x| println!("Fetched {} papers.", x.len()))
        .inspect_err(|x| println!("Fetching failed: {x}"))
        .unwrap_or_default()
}

#[tokio::main]
pub async fn get_paper_by_id(id: &str) -> Arxiv {
    let query = ArxivQueryBuilder::new().id_list(id).build();
    let arxivs = arxiv::fetch_arxivs(query).await.unwrap_or_default();
    arxivs[0].clone()
}

pub fn get_related_papers_by_reference(
    initial_arxiv_id: &str,
    max_depth: &i32,
) -> Vec<paper::Paper> {
    let initial_paper = paper::Paper::from_arxiv_paper(&get_paper_by_id(initial_arxiv_id));
    let related_papers: Vec<paper::Paper> = Vec::new();

    related_papers
}
