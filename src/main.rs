mod data_aggregation;
mod datamodels;
use data_aggregation::search_query_handler::SearchQueryHandler;
use datamodels::paper::{self, Paper};
use scraper::Selector;

fn main() {
    let search_query_handler: SearchQueryHandler = SearchQueryHandler::new()
        .title("Dark Sirens")
        .author("")
        .paper_abstract("")
        .category("")
        .id("")
        .report_number("")
        .journal_reference("")
        .comment("")
        .all_categories("");

    let arxiv_papers = data_aggregation::arxiv_api::get_papers(
        search_query_handler,
        0,
        100,
        "submittedDate",
        "descending",
    );

    let mut papers: Vec<Paper> = Vec::new();
    for arxiv_paper in arxiv_papers.iter() {
        papers.push(Paper::from_arxiv_paper(arxiv_paper));
    }
    let sample_paper = &papers[0];
    let paper_references = data_aggregation::html_parser::get_references_for_arxiv_paper(
        &sample_paper
            .pdf_url
            .replace(".pdf", "")
            .replace("pdf", "html"),
    );
}
