mod data_aggregation;
mod datamodels;
use data_aggregation::search_query_handler::SearchQueryHandler;
use datamodels::paper::Paper;

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

    let arxivs = data_aggregation::arxiv_api::get_papers(
        search_query_handler,
        0,
        100,
        "submittedDate",
        "descending",
    );

    for arxiv in arxivs.iter() {
        println!("{:?}", arxiv.title);
    }
    if !arxivs.is_empty() {
        let paper: Paper = Paper::from_arxiv_paper(&arxivs[0]);
        println!("{}", paper);
    }
}
