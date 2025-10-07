use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct EdgeRenderData {
    pub vertices: Vec<Vec3>,
    pub edges: Vec<[u32; 2]>,
    pub dirty: bool,
}

impl EdgeRenderData {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            edges: Vec::new(),
            dirty: true,
        }
    }

    pub fn update_vertices(&mut self, vertices: Vec<Vec3>) {
        self.vertices = vertices;
        self.dirty = true;
    }

    pub fn update_edges(&mut self, edges: Vec<[u32; 2]>) {
        self.edges = edges;
        self.dirty = true;
    }

    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}