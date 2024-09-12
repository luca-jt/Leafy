use crate::rendering::batch_renderer::BatchRenderer;
use crate::rendering::mesh::Mesh;
use crate::rendering::shader::ShaderProgram;
use crate::utils::tools::WeakPtr;

pub(crate) struct FontRenderer {
    batch_renderer: BatchRenderer,
}

impl FontRenderer {
    pub(crate) fn new(mesh: WeakPtr<Mesh>, shader: &ShaderProgram) -> Self {
        Self {
            batch_renderer: BatchRenderer::new(mesh, 100, shader),
        }
    }

    pub(crate) fn init(&mut self) {}
    pub(crate) fn end(&mut self) {}
}
