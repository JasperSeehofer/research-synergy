use crate::datamodels::graph::Graph;
use crate::datamodels::paper::Paper;

pub fn create_graph_from_papers(papers: Vec<Paper>) -> Graph {
    let mut paper_graph: Graph = Graph::new();
    for paper in papers {
        paper_graph.push_paper(paper);
    }
    paper_graph.remove_open_edges();
    paper_graph
}
