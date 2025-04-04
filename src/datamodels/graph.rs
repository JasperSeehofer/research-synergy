use crate::datamodels::paper::Paper;
use std::collections::HashMap;

pub struct Graph {
    vertices: HashMap<i32, Paper>,
    adjacency: HashMap<i32, Vec<(i32, f32)>>,
}

impl Graph {
    pub fn new() -> Graph {
        Graph {
            vertices: HashMap::new(),
            adjacency: HashMap::new(),
        }
    }

    pub fn push_vertex(self: &mut Graph, vid: i32, vertex: Paper) {
        self.vertices.insert(vid, vertex);
    }

    pub fn push_edge(self: &mut Graph, from_vertex_id: i32, to_vertex_id: i32, edge: f32) {
        let adjacent_to_from = self.adjacency.entry(from_vertex_id).or_default();
        adjacent_to_from.push((to_vertex_id, edge));
    }
}
