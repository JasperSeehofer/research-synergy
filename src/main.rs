mod data_aggregation;
mod data_processing;
mod datamodels;
mod visualization;
use datamodels::{
    graph::PaperGraph,
    paper::{self, Paper},
};
use eframe::run_native;
use visualization::force_graph_app::DemoApp;

fn main() {
    /* let search_query_handler: SearchQueryHandler = SearchQueryHandler::new()
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
    */
    let paper_id = "2503.18887";
    let arxiv_paper_by_id =
        data_aggregation::arxiv_api::get_paper_by_id(paper_id).unwrap_or_default();

    let paper_by_id = Paper::from_arxiv_paper(&arxiv_paper_by_id);

    let referenced_papers =
        data_aggregation::arxiv_utils::recursive_paper_search_by_references(&paper_by_id.id, 3);

    println!(
        "Recursive search accumulated {} papers.",
        referenced_papers.len()
    );

    let paper_graph: petgraph::prelude::StableGraph<Paper, f32> =
        data_processing::graph_creation::create_graph_from_papers(&referenced_papers);
    println!(
        "paper_graph has {} nodes and {} edges.",
        paper_graph.node_count(),
        paper_graph.edge_count()
    );

    let graph_without_weights = paper_graph.map(|_, _| (), |_, _| ());
    println!(
        "paper_graph has {} nodes and {} edges.",
        graph_without_weights.node_count(),
        graph_without_weights.edge_count()
    );

    let native_options = eframe::NativeOptions::default();

    run_native(
        "Paper graph interactive",
        native_options,
        Box::new(|cc| Ok(Box::new(DemoApp::new(cc, graph_without_weights)))),
    )
    .unwrap();
}
