use crate::rendering::batch_renderer::BatchRenderer;
use crate::rendering::mesh::Mesh;
use crate::rendering::shader::ShaderType;
use crate::utils::file::PLANE_MESH;

pub(crate) struct FontRenderer {
    batch_renderer: BatchRenderer,
    mesh: Mesh,
}

impl FontRenderer {
    pub(crate) fn new() -> Self {
        let mesh = Mesh::from_bytes(PLANE_MESH);
        Self {
            batch_renderer: BatchRenderer::new(&mesh, ShaderType::Passthrough),
            mesh,
        }
    }

    pub(crate) fn init(&mut self) {}
    pub(crate) fn end(&mut self) {}
}
