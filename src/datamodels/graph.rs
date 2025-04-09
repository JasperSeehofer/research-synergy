use crate::datamodels::paper::Paper;
use std::{collections::HashMap, usize};

pub struct PaperGraph {
    vertices: HashMap<String, Paper>,
    adjacency: HashMap<String, Vec<(String, f32)>>,
}

impl PaperGraph {
    pub fn new() -> PaperGraph {
        PaperGraph {
            vertices: HashMap::new(),
            adjacency: HashMap::new(),
        }
    }

    pub fn push_vertex(self: &mut PaperGraph, vid: &str, vertex: Paper) {
        self.vertices.insert(vid.to_string(), vertex);
    }

    pub fn push_edge(self: &mut PaperGraph, from_vertex_id: &str, to_vertex_id: &str, edge: f32) {
        let adjacent_to_from = self
            .adjacency
            .entry(from_vertex_id.to_string())
            .or_default();
        adjacent_to_from.push((to_vertex_id.to_string(), edge));
    }

    pub fn remove_edge(self: &mut PaperGraph, from_vertex_id: &str, to_vertex_id: &str) {
        let adjacent_to_from = self
            .adjacency
            .entry(from_vertex_id.to_string())
            .or_default();
        adjacent_to_from.retain(|(s, _)| s != to_vertex_id);
    }

    pub fn push_paper(self: &mut PaperGraph, paper: Paper) {
        for arxiv_reference in paper.get_arxiv_references_ids() {
            self.push_edge(&paper.id, &arxiv_reference, 1.0);
        }

        self.push_vertex(&paper.id, paper.clone());
    }

    pub fn remove_open_edges(self: &mut PaperGraph) {
        let mut open_edges_counter: usize = 0;
        for (from_paper_id, edge_collection) in self.adjacency.clone().iter() {
            for to_paper_id in edge_collection.iter().map(|(string, _)| string) {
                if !self.vertices.contains_key(to_paper_id) {
                    open_edges_counter += 1;
                    self.remove_edge(from_paper_id, to_paper_id);
                }
            }
        }
        println!(
            "Removed {} edges because target vertex does not exist.",
            open_edges_counter
        )
    }
}
