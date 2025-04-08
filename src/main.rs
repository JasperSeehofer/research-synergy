mod data_aggregation;
use std::collections::HashSet;
mod datamodels;
use data_aggregation::search_query_handler::SearchQueryHandler;
use datamodels::paper::{self, Paper};

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
        1,
        "submittedDate",
        "descending",
    )
    .unwrap_or_default();

    let paper = Paper::from_arxiv_paper(&arxiv_papers[0]);

    println!("{}", paper.id);
    let paper_id = "2503.18887";
    let arxiv_paper_by_id =
        data_aggregation::arxiv_api::get_paper_by_id(paper_id).unwrap_or_default();

    let paper_by_id = Paper::from_arxiv_paper(&arxiv_paper_by_id);

    let referenced_papers =
        data_aggregation::arxiv_utils::recursive_paper_search_by_references(&paper_by_id.id, 3);

    for paper in referenced_papers {
        println!("{}", paper);
    }
}
