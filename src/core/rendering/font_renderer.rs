use crate::rendering::batch_renderer::BatchRenderer;
use crate::rendering::mesh::Mesh;
use crate::utils::tools::shared_ptr;

pub(crate) struct FontRenderer {
    batch_renderer: BatchRenderer,
}

impl FontRenderer {
    pub(crate) fn new() -> Self {
        Self {
            batch_renderer: BatchRenderer::new(shared_ptr(Mesh::new("plane.obj")), 100),
        }
    }

    pub(crate) fn init(&mut self) {}
    pub(crate) fn end(&mut self) {}
}
