use arxiv::{Arxiv, ArxivQueryBuilder};

struct SearchQuery {
    title: String,
    author: String,
    paper_abstract: String,
    comment: String,
    journal_reference: String,
    report_number: String,
    id: String,
    all_categories: String,
}

impl SearchQuery {
    fn new() -> SearchQuery {
        SearchQuery {
            title: String::new(),
            author: String::new(),
            paper_abstract: String::new(),
            comment: String::new(),
            journal_reference: String::new(),
            report_number: String::new(),
            id: String::new(),
            all_categories: String::new(),
        }
    }
    fn title(&self) -> Self {
        todo!()
    }
}

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
