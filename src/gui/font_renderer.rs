use crate::rendering::batch_renderer::BatchRenderer;
use crate::rendering::mesh::Mesh;
use crate::rendering::shader::ShaderType;
use crate::utils::file::get_model_path;

pub(crate) struct FontRenderer {
    batch_renderer: BatchRenderer,
    mesh: Mesh,
}

impl FontRenderer {
    pub(crate) fn new() -> Self {
        let mesh = Mesh::new(get_model_path("plane.obj"));
        Self {
            batch_renderer: BatchRenderer::new(&mesh, ShaderType::Passthrough),
            mesh,
        }
    }

    pub(crate) fn init(&mut self) {}
    pub(crate) fn end(&mut self) {}
}
