use crate::rendering::batch_renderer::BatchRenderer;
use crate::rendering::mesh::SharedMesh;

pub struct FontRenderer {
    batch_renderer: BatchRenderer,
}

impl FontRenderer {
    /*pub fn new() -> Self {
        Self {
            batch_renderer: BatchRenderer::new(SharedMesh::from_file(""), 100),
        }
    }*/

    pub fn init(&mut self) {}
    pub fn end(&mut self) {}
}
