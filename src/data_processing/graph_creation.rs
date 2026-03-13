use crate::datamodels::paper::Paper;
use crate::utils::strip_version_suffix;
use petgraph::prelude::NodeIndex;
use petgraph::stable_graph::StableGraph;
use petgraph::{self, Directed};
use std::collections::HashMap;
use tracing::debug;

pub fn create_graph_from_papers(papers: &[Paper]) -> StableGraph<Paper, f32, Directed, u32> {
    let mut paper_graph = StableGraph::<Paper, f32, Directed, u32>::new();
    let mut paper_to_node_id_mapping: HashMap<String, NodeIndex> = HashMap::new();
    for paper in papers {
        let paper_id = strip_version_suffix(&paper.id);
        if paper_to_node_id_mapping.contains_key(&paper_id) {
            continue;
        }
        let node_index: NodeIndex = paper_graph.add_node(paper.clone());
        paper_to_node_id_mapping.insert(paper_id, node_index);
    }

    for paper in papers {
        let paper_id = strip_version_suffix(&paper.id);
        for arxiv_reference in paper.get_arxiv_references_ids() {
            let ref_id = strip_version_suffix(&arxiv_reference);
            debug!(from = %paper_id, to = %ref_id, "Adding edge");
            if let (Some(&from_idx), Some(&to_idx)) = (
                paper_to_node_id_mapping.get(&paper_id),
                paper_to_node_id_mapping.get(&ref_id),
            ) {
                paper_graph.add_edge(from_idx, to_idx, 1.0);
            }
        }
    }
    paper_graph
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datamodels::paper::{Link, Reference};

    fn make_paper(id: &str, ref_ids: &[&str]) -> Paper {
        Paper {
            title: format!("Paper {id}"),
            authors: vec![],
            summary: String::new(),
            id: id.to_string(),
            last_updated: String::new(),
            published: String::new(),
            pdf_url: String::new(),
            comment: None,
            references: ref_ids
                .iter()
                .map(|rid| Reference {
                    author: String::new(),
                    title: String::new(),
                    links: vec![Link::from_url(&format!("https://arxiv.org/abs/{rid}"))],
                })
                .collect(),
        }
    }

    #[test]
    fn test_empty_input() {
        let graph = create_graph_from_papers(&[]);
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_single_paper_no_refs() {
        let papers = vec![make_paper("2301.12345", &[])];
        let graph = create_graph_from_papers(&papers);
        assert_eq!(graph.node_count(), 1);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_paper_with_citations() {
        let papers = vec![
            make_paper("2301.11111", &["2301.22222"]),
            make_paper("2301.22222", &[]),
        ];
        let graph = create_graph_from_papers(&papers);
        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn test_version_dedup() {
        let papers = vec![
            make_paper("2301.12345v1", &[]),
            make_paper("2301.12345v2", &[]),
        ];
        let graph = create_graph_from_papers(&papers);
        assert_eq!(graph.node_count(), 1);
    }

    #[test]
    fn test_citation_to_missing_paper() {
        let papers = vec![make_paper("2301.11111", &["2301.99999"])];
        let graph = create_graph_from_papers(&papers);
        assert_eq!(graph.node_count(), 1);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_version_suffix_in_citation_edge() {
        let papers = vec![
            make_paper("2301.11111", &["2301.22222v3"]),
            make_paper("2301.22222v1", &[]),
        ];
        let graph = create_graph_from_papers(&papers);
        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);
    }
}
