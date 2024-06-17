use crate::rendering::batch_renderer::BatchRenderer;
use crate::systems::rendering_system::Renderer;
use stb_image::image::Image;

pub struct SpriteRenderer {
    batch_renderer: BatchRenderer,
    sprite_sheet: Image<u8>,
}

impl SpriteRenderer {
    // render with sprite index
}

impl Renderer for SpriteRenderer {
    fn render(&mut self) {
        todo!()
    }
}
