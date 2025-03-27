use arxiv::{Arxiv, ArxivQueryBuilder};

#[tokio::main]
pub async fn get_papers() -> Vec<Arxiv> {
    let query = ArxivQueryBuilder::new()
        .search_query("gravitational+waves")
        .start(0)
        .max_results(5)
        .sort_by("submittedDate")
        .sort_order("descending")
        .build();
    arxiv::fetch_arxivs(query)
        .await
        .inspect(|x| println!("Fetched {} papers.", x.len()))
        .inspect_err(|x| println!("Fetching failed: {x}"))
        .unwrap_or_default()
}
