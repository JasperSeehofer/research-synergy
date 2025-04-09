use crate::datamodels::paper::Paper;
use petgraph::prelude::NodeIndex;
use petgraph::stable_graph::StableGraph;
use petgraph::{self, Directed};
use std::collections::HashMap;

pub fn create_graph_from_papers(papers: &Vec<Paper>) -> StableGraph<Paper, f32, Directed, u32> {
    let mut paper_graph = StableGraph::<Paper, f32, Directed, u32>::new();
    let mut paper_to_node_id_mapping: HashMap<String, NodeIndex> = HashMap::new();
    for paper in papers {
        let paper_id = paper.id.split("v").next().unwrap();
        if paper_to_node_id_mapping.contains_key(paper_id) {
            continue;
        }
        let node_index: NodeIndex = paper_graph.add_node(paper.clone());
        paper_to_node_id_mapping.insert(paper_id.to_string(), node_index);
    }

    for paper in papers {
        let paper_id = paper.id.split("v").next().unwrap();
        for arxiv_reference in paper.get_arxiv_references_ids() {
            println!(
                "Edge from paper id: {} to paper id {}",
                paper_id, arxiv_reference
            );
            if paper_to_node_id_mapping.contains_key(paper_id)
                && paper_to_node_id_mapping.contains_key(&arxiv_reference)
            {
                paper_graph.add_edge(
                    *paper_to_node_id_mapping.get(paper_id).unwrap(),
                    *paper_to_node_id_mapping.get(&arxiv_reference).unwrap(),
                    1.0,
                );
            }
        }
    }
    paper_graph
}
