use crate::rendering::batch_renderer::BatchRenderer;
use crate::systems::rendering_system::Renderer;

pub struct FontRenderer {
    batch_renderer: BatchRenderer,
}

impl Renderer for FontRenderer {
    fn render(&mut self) {
        todo!()
    }
}
