use crate::data_aggregation::search_query_handler;
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
